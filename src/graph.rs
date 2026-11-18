//! Generic dependency graph; it does not interpret formulas.
use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

/// Directed graph in which each node records its prerequisites.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyGraph<N: Ord> {
    dependencies: BTreeMap<N, BTreeSet<N>>,
}
impl<N: Ord> Default for DependencyGraph<N> {
    fn default() -> Self {
        Self {
            dependencies: BTreeMap::new(),
        }
    }
}
impl<N: Ord + Clone> DependencyGraph<N> {
    /// Inserts a node if absent.
    pub fn add_node(&mut self, node: N) {
        self.dependencies.entry(node).or_default();
    }
    /// Adds dependency as a prerequisite of node.
    pub fn add_dependency(&mut self, node: N, dependency: N) -> Result<(), GraphError<N>> {
        if !self.dependencies.contains_key(&node) {
            return Err(GraphError::UnknownNode(node));
        }
        if !self.dependencies.contains_key(&dependency) {
            return Err(GraphError::UnknownNode(dependency));
        }
        self.dependencies
            .get_mut(&node)
            .expect("validated")
            .insert(dependency);
        Ok(())
    }
    /// Removes a node and all references to it.
    pub fn remove_node(&mut self, node: &N) -> bool {
        let removed = self.dependencies.remove(node).is_some();
        if removed {
            for deps in self.dependencies.values_mut() {
                deps.remove(node);
            }
        }
        removed
    }
    /// Returns a deterministic prerequisite-first topological order.
    ///
    /// If a cycle prevents a complete ordering, `GraphError::Cycle` contains
    /// every node left unresolved by the topological-sort algorithm. This may
    /// include nodes downstream of a cycle, not only nodes in the cycle itself.
    pub fn topological_order(&self) -> Result<Vec<N>, GraphError<N>> {
        let mut remaining: BTreeMap<N, usize> = self
            .dependencies
            .iter()
            .map(|(n, deps)| (n.clone(), deps.len()))
            .collect();
        let mut dependents: BTreeMap<N, BTreeSet<N>> = BTreeMap::new();
        for (node, deps) in &self.dependencies {
            for dep in deps {
                dependents
                    .entry(dep.clone())
                    .or_default()
                    .insert(node.clone());
            }
        }
        let mut ready: BTreeSet<N> = remaining
            .iter()
            .filter_map(|(n, count)| (*count == 0).then_some(n.clone()))
            .collect();
        let mut order = Vec::with_capacity(remaining.len());
        while let Some(node) = ready.pop_first() {
            order.push(node.clone());
            if let Some(children) = dependents.get(&node) {
                for child in children {
                    let count = remaining.get_mut(child).expect("indexed graph node");
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(child.clone());
                    }
                }
            }
        }
        if order.len() == remaining.len() {
            Ok(order)
        } else {
            Err(GraphError::Cycle(
                remaining
                    .into_iter()
                    .filter_map(|(n, count)| (count > 0).then_some(n))
                    .collect(),
            ))
        }
    }
    /// Returns node count.
    pub fn len(&self) -> usize {
        self.dependencies.len()
    }
    /// Returns whether the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.dependencies.is_empty()
    }
}

/// Dependency graph error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphError<N> {
    /// Referenced node does not exist.
    UnknownNode(N),
    /// Nodes left unresolved by topological ordering.
    ///
    /// This includes cycle members and nodes blocked by those cycles.
    Cycle(Vec<N>),
}
impl<N: fmt::Debug> fmt::Display for GraphError<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownNode(n) => write!(f, "unknown graph node: {n:?}"),
            Self::Cycle(nodes) => write!(
                f,
                "topological order blocked by a dependency cycle; unresolved nodes: {nodes:?}"
            ),
        }
    }
}
impl<N: fmt::Debug> std::error::Error for GraphError<N> {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn returns_deterministic_dependency_order() {
        let mut g = DependencyGraph::default();
        for n in [1, 2, 3, 4] {
            g.add_node(n);
        }
        g.add_dependency(3, 1).unwrap();
        g.add_dependency(3, 2).unwrap();
        g.add_dependency(4, 3).unwrap();
        assert_eq!(g.topological_order().unwrap(), vec![1, 2, 3, 4]);
    }
    #[test]
    fn rejects_unknown_nodes_and_reports_cycle_blocked_nodes() {
        let mut g = DependencyGraph::default();
        for node in [1, 2, 3] {
            g.add_node(node);
        }
        assert_eq!(g.add_dependency(1, 4), Err(GraphError::UnknownNode(4)));
        g.add_dependency(1, 2).unwrap();
        g.add_dependency(2, 1).unwrap();
        g.add_dependency(3, 2).unwrap();

        // Node 3 is not itself cyclic, but it cannot be ordered because it
        // depends on a node in the 1 <-> 2 cycle.
        assert_eq!(g.topological_order(), Err(GraphError::Cycle(vec![1, 2, 3])));
    }
}
