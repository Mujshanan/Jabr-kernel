//! Generic port-aware dependency graph; it does not interpret formulas.
use crate::{NodeId, Port};
use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

/// Directed graph in which each node records the prerequisite ports it reads.
///
/// Port identities are retained in the forward index. Scheduling is performed
/// at node granularity, so consuming multiple ports from one prerequisite
/// creates only one scheduling edge.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DependencyGraph {
    prerequisites: BTreeMap<NodeId, BTreeSet<Port>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    dependents: BTreeMap<NodeId, BTreeSet<NodeId>>,
}

impl DependencyGraph {
    /// Inserts a node if absent.
    pub fn add_node(&mut self, node: NodeId) {
        self.prerequisites.entry(node).or_default();
        self.dependents.entry(node).or_default();
    }

    /// Adds a dependency on the prerequisite node's port at index zero.
    ///
    /// This compatibility convenience is equivalent to
    /// `add_port_dependency(node, Port::new(dependency, 0))`.
    pub fn add_dependency(&mut self, node: NodeId, dependency: NodeId) -> Result<(), GraphError> {
        self.add_port_dependency(node, Port::new(dependency, 0))
    }

    /// Adds one prerequisite port to a node.
    ///
    /// An exact duplicate port is idempotent. Different port indexes on the
    /// same prerequisite remain distinct in storage but count as one
    /// prerequisite for scheduling.
    pub fn add_port_dependency(&mut self, node: NodeId, port: Port) -> Result<(), GraphError> {
        self.ensure_node(node)?;
        self.ensure_node(port.node_id)?;
        let inserted = self
            .prerequisites
            .get_mut(&node)
            .expect("validated node")
            .insert(port);
        if inserted {
            self.dependents
                .get_mut(&port.node_id)
                .expect("validated prerequisite")
                .insert(node);
        }
        Ok(())
    }

    /// Atomically replaces all prerequisite ports for `node`.
    ///
    /// The target and every prerequisite are validated before either index is
    /// changed. Exact duplicate ports are collapsed by the ordered set.
    pub fn set_dependencies<I>(&mut self, node: NodeId, dependencies: I) -> Result<(), GraphError>
    where
        I: IntoIterator<Item = Port>,
    {
        self.ensure_node(node)?;
        let replacement: BTreeSet<Port> = dependencies.into_iter().collect();
        for port in &replacement {
            self.ensure_node(port.node_id)?;
        }

        let previous = self
            .prerequisites
            .get(&node)
            .expect("validated node")
            .clone();
        for port in &previous {
            if let Some(nodes) = self.dependents.get_mut(&port.node_id) {
                nodes.remove(&node);
            }
        }
        self.prerequisites.insert(node, replacement.clone());
        for port in replacement {
            self.dependents
                .get_mut(&port.node_id)
                .expect("validated prerequisite")
                .insert(node);
        }
        Ok(())
    }

    /// Returns the prerequisite ports read by a node.
    pub fn dependencies_of(&self, node: NodeId) -> Result<&BTreeSet<Port>, GraphError> {
        self.prerequisites
            .get(&node)
            .ok_or(GraphError::UnknownNode(node))
    }

    /// Returns the direct nodes that depend on `node`.
    pub fn dependents_of(&self, node: NodeId) -> Result<&BTreeSet<NodeId>, GraphError> {
        self.dependents
            .get(&node)
            .ok_or(GraphError::UnknownNode(node))
    }

    /// Removes a node and every incoming and outgoing dependency reference.
    pub fn remove_node(&mut self, node: NodeId) -> bool {
        let Some(prerequisites) = self.prerequisites.remove(&node) else {
            return false;
        };
        self.dependents.remove(&node);

        // Remove this node from each prerequisite's reverse index.
        for port in prerequisites {
            if let Some(nodes) = self.dependents.get_mut(&port.node_id) {
                nodes.remove(&node);
            }
        }

        // Remove every port reference to the removed node from its dependents.
        for (dependent, ports) in &mut self.prerequisites {
            ports.retain(|port| port.node_id != node);
            let _ = dependent;
        }
        true
    }

    /// Returns the roots and all transitive dependents in deterministic order.
    pub fn dirty_set<I>(&self, dirty: I) -> Result<BTreeSet<NodeId>, GraphError>
    where
        I: IntoIterator<Item = NodeId>,
    {
        let roots: BTreeSet<NodeId> = dirty.into_iter().collect();
        for node in &roots {
            self.ensure_node(*node)?;
        }

        let mut result = roots.clone();
        let mut pending: Vec<NodeId> = roots.into_iter().collect();
        while let Some(node) = pending.pop() {
            if let Some(children) = self.dependents.get(&node) {
                for child in children {
                    if result.insert(*child) {
                        pending.push(*child);
                    }
                }
            }
        }
        Ok(result)
    }

    /// Returns a deterministic prerequisite-first topological order.
    ///
    /// When a cycle exists, the error contains only nodes that belong to
    /// directed cycles (including self-loops), never merely blocked nodes.
    pub fn topological_order(&self) -> Result<Vec<NodeId>, GraphError> {
        let all = self.prerequisites.keys().copied().collect();
        self.order_within(&all)
    }

    /// Returns prerequisite-first order for the dirty closure only.
    ///
    /// Prerequisites outside the dirty closure are treated as already clean
    /// and do not appear in the returned order.
    pub fn recalculation_order<I>(&self, dirty: I) -> Result<Vec<NodeId>, GraphError>
    where
        I: IntoIterator<Item = NodeId>,
    {
        let closure = self.dirty_set(dirty)?;
        self.order_within(&closure)
    }

    /// Returns node count.
    pub fn len(&self) -> usize {
        self.prerequisites.len()
    }

    /// Returns whether the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.prerequisites.is_empty()
    }

    fn ensure_node(&self, node: NodeId) -> Result<(), GraphError> {
        if self.prerequisites.contains_key(&node) {
            Ok(())
        } else {
            Err(GraphError::UnknownNode(node))
        }
    }

    fn order_within(&self, included: &BTreeSet<NodeId>) -> Result<Vec<NodeId>, GraphError> {
        let mut remaining: BTreeMap<NodeId, usize> = BTreeMap::new();
        for node in included {
            let count = self
                .prerequisites
                .get(node)
                .expect("included nodes exist")
                .iter()
                .map(|port| port.node_id)
                .filter(|dependency| included.contains(dependency))
                .collect::<BTreeSet<_>>()
                .len();
            remaining.insert(*node, count);
        }

        let mut ready: BTreeSet<NodeId> = remaining
            .iter()
            .filter_map(|(node, count)| (*count == 0).then_some(*node))
            .collect();
        let mut order = Vec::with_capacity(remaining.len());
        while let Some(node) = ready.pop_first() {
            order.push(node);
            if let Some(children) = self.dependents.get(&node) {
                for child in children.iter().filter(|child| included.contains(child)) {
                    let count = remaining.get_mut(child).expect("indexed graph node");
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(*child);
                    }
                }
            }
        }

        if order.len() == remaining.len() {
            return Ok(order);
        }
        let unresolved: BTreeSet<NodeId> = remaining
            .into_iter()
            .filter_map(|(node, count)| (count > 0).then_some(node))
            .collect();
        Err(GraphError::Cycle(self.cycle_members(&unresolved)))
    }

    fn cycle_members(&self, unresolved: &BTreeSet<NodeId>) -> Vec<NodeId> {
        struct TarjanState {
            next_index: usize,
            indices: BTreeMap<NodeId, usize>,
            lowlink: BTreeMap<NodeId, usize>,
            stack: Vec<NodeId>,
            on_stack: BTreeSet<NodeId>,
            cycles: BTreeSet<NodeId>,
        }

        fn visit(
            graph: &DependencyGraph,
            node: NodeId,
            allowed: &BTreeSet<NodeId>,
            state: &mut TarjanState,
        ) {
            let index = state.next_index;
            state.next_index += 1;
            state.indices.insert(node, index);
            state.lowlink.insert(node, index);
            state.stack.push(node);
            state.on_stack.insert(node);

            let neighbors: BTreeSet<NodeId> = graph.prerequisites[&node]
                .iter()
                .map(|port| port.node_id)
                .filter(|next| allowed.contains(next))
                .collect();
            for next in neighbors {
                if !state.indices.contains_key(&next) {
                    visit(graph, next, allowed, state);
                    let child_low = state.lowlink[&next];
                    let node_low = state.lowlink[&node];
                    state.lowlink.insert(node, node_low.min(child_low));
                } else if state.on_stack.contains(&next) {
                    let next_index = state.indices[&next];
                    let node_low = state.lowlink[&node];
                    state.lowlink.insert(node, node_low.min(next_index));
                }
            }

            if state.lowlink[&node] == state.indices[&node] {
                let mut component = Vec::new();
                loop {
                    let member = state.stack.pop().expect("Tarjan stack is non-empty");
                    state.on_stack.remove(&member);
                    component.push(member);
                    if member == node {
                        break;
                    }
                }
                let self_loop = graph.prerequisites[&node]
                    .iter()
                    .any(|port| port.node_id == node);
                if component.len() > 1 || self_loop {
                    state.cycles.extend(component);
                }
            }
        }

        let mut state = TarjanState {
            next_index: 0,
            indices: BTreeMap::new(),
            lowlink: BTreeMap::new(),
            stack: Vec::new(),
            on_stack: BTreeSet::new(),
            cycles: BTreeSet::new(),
        };
        for node in unresolved {
            if !state.indices.contains_key(node) {
                visit(self, *node, unresolved, &mut state);
            }
        }
        state.cycles.into_iter().collect()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DependencyGraph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Repr {
            prerequisites: BTreeMap<NodeId, BTreeSet<Port>>,
        }
        let repr = <Repr as serde::Deserialize>::deserialize(deserializer)?;
        let mut graph = Self {
            dependents: repr
                .prerequisites
                .keys()
                .map(|node| (*node, BTreeSet::new()))
                .collect(),
            prerequisites: repr.prerequisites,
        };
        let entries: Vec<(NodeId, Port)> = graph
            .prerequisites
            .iter()
            .flat_map(|(node, ports)| ports.iter().map(move |port| (*node, *port)))
            .collect();
        for (dependent, port) in entries {
            let Some(reverse) = graph.dependents.get_mut(&port.node_id) else {
                return Err(serde::de::Error::custom(format!(
                    "dependency graph references unknown prerequisite node {}",
                    port.node_id
                )));
            };
            reverse.insert(dependent);
        }
        Ok(graph)
    }
}

/// Dependency graph error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphError {
    /// Referenced node does not exist.
    UnknownNode(NodeId),
    /// Actual members of one or more directed cycles, sorted by NodeId.
    Cycle(Vec<NodeId>),
}
impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode(node) => write!(f, "unknown graph node: {node}"),
            Self::Cycle(nodes) => write!(
                f,
                "dependency graph contains a cycle involving nodes: {nodes:?}"
            ),
        }
    }
}
impl std::error::Error for GraphError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeId;

    fn id(value: u64) -> NodeId {
        NodeId::new(value).unwrap()
    }
    fn graph(nodes: &[u64]) -> DependencyGraph {
        let mut graph = DependencyGraph::default();
        for node in nodes {
            graph.add_node(id(*node));
        }
        graph
    }
    fn port(node: u64, index: u32) -> Port {
        Port::new(id(node), index)
    }

    #[test]
    fn returns_deterministic_prerequisite_first_order_and_deduplicates_node_edges() {
        let mut graph = graph(&[1, 2, 3, 4]);
        graph
            .set_dependencies(id(3), [port(1, 0), port(1, 1), port(2, 0)])
            .unwrap();
        graph.set_dependencies(id(4), [port(3, 0)]).unwrap();
        assert_eq!(graph.dependencies_of(id(3)).unwrap().len(), 3);
        assert_eq!(
            graph.topological_order().unwrap(),
            vec![id(1), id(2), id(3), id(4)]
        );
    }

    #[test]
    fn exact_duplicate_ports_are_collapsed() {
        let mut graph = graph(&[1, 2]);
        graph
            .set_dependencies(id(2), [port(1, 0), port(1, 0)])
            .unwrap();
        assert_eq!(graph.dependencies_of(id(2)).unwrap().len(), 1);
        assert_eq!(
            graph.dependents_of(id(1)).unwrap(),
            &BTreeSet::from([id(2)])
        );
    }

    #[test]
    fn replacement_is_atomic_for_unknown_prerequisite_and_target() {
        let mut graph = graph(&[1, 2, 3]);
        graph.set_dependencies(id(3), [port(1, 0)]).unwrap();
        let before = graph.clone();
        assert_eq!(
            graph.set_dependencies(id(3), [port(99, 0)]),
            Err(GraphError::UnknownNode(id(99)))
        );
        assert_eq!(graph, before);
        assert_eq!(
            graph.set_dependencies(id(99), [port(1, 0)]),
            Err(GraphError::UnknownNode(id(99)))
        );
        assert_eq!(graph, before);
    }

    #[test]
    fn reverse_index_and_removal_remain_consistent() {
        let mut graph = graph(&[1, 2, 3, 4]);
        graph.set_dependencies(id(2), [port(1, 0)]).unwrap();
        graph
            .set_dependencies(id(3), [port(2, 0), port(1, 1)])
            .unwrap();
        graph.set_dependencies(id(4), [port(3, 0)]).unwrap();
        assert!(graph.remove_node(id(2)));
        assert_eq!(
            graph.dependencies_of(id(3)).unwrap(),
            &BTreeSet::from([port(1, 1)])
        );
        assert_eq!(
            graph.dependents_of(id(1)).unwrap(),
            &BTreeSet::from([id(3)])
        );
        assert_eq!(
            graph.dirty_set([id(1)]).unwrap(),
            BTreeSet::from([id(1), id(3), id(4)])
        );
        assert_eq!(
            graph.topological_order().unwrap(),
            vec![id(1), id(3), id(4)]
        );
        assert!(!graph.remove_node(id(99)));
    }

    #[test]
    fn dirty_closure_and_recalculation_order_are_deterministic() {
        let mut graph = graph(&[1, 2, 3, 4, 5, 6]);
        graph.set_dependencies(id(3), [port(1, 0)]).unwrap();
        graph
            .set_dependencies(id(4), [port(1, 0), port(2, 0)])
            .unwrap();
        graph
            .set_dependencies(id(5), [port(3, 0), port(4, 0)])
            .unwrap();
        graph.set_dependencies(id(6), [port(5, 0)]).unwrap();
        assert_eq!(graph.dirty_set([]).unwrap(), BTreeSet::new());
        assert_eq!(
            graph.dirty_set([id(1), id(2)]).unwrap(),
            BTreeSet::from([id(1), id(2), id(3), id(4), id(5), id(6)])
        );
        assert_eq!(
            graph.recalculation_order([id(1)]).unwrap(),
            vec![id(1), id(3), id(4), id(5), id(6)]
        );
        assert_eq!(
            graph.dirty_set([id(99)]),
            Err(GraphError::UnknownNode(id(99)))
        );
    }

    #[test]
    fn cycle_reports_only_actual_members_including_self_loops() {
        let mut graph = graph(&[1, 2, 3, 4, 5, 6, 7, 8]);
        graph.set_dependencies(id(1), [port(2, 0)]).unwrap();
        graph.set_dependencies(id(2), [port(1, 0)]).unwrap();
        graph.set_dependencies(id(3), [port(2, 0)]).unwrap();
        graph.set_dependencies(id(4), [port(4, 0)]).unwrap();
        graph.set_dependencies(id(5), [port(6, 0)]).unwrap();
        graph.set_dependencies(id(6), [port(5, 0)]).unwrap();
        graph.set_dependencies(id(7), [port(6, 0)]).unwrap();
        graph.set_dependencies(id(8), [port(7, 0)]).unwrap();
        assert_eq!(
            graph.topological_order(),
            Err(GraphError::Cycle(vec![id(1), id(2), id(4), id(5), id(6)]))
        );
        assert_eq!(
            graph.recalculation_order([id(1)]),
            Err(GraphError::Cycle(vec![id(1), id(2)]))
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip_rebuilds_reverse_index_and_rejects_dangling_edges() {
        let mut graph = graph(&[1, 2, 3]);
        graph
            .set_dependencies(id(3), [port(1, 0), port(1, 1), port(2, 0)])
            .unwrap();
        let json = serde_json::to_string(&graph).unwrap();
        let decoded: DependencyGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, graph);
        assert_eq!(
            decoded.dependents_of(id(1)).unwrap(),
            &BTreeSet::from([id(3)])
        );

        let malformed = r#"{"prerequisites":{"1":[{"node_id":99,"index":0}]}}"#;
        assert!(serde_json::from_str::<DependencyGraph>(malformed).is_err());
    }
}
