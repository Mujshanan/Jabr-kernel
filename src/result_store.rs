//! Opaque calculation-result storage keyed by generic node identity.

use std::collections::BTreeMap;

use crate::NodeId;

/// Generic result store. Kernel never inspects or interprets V.
#[derive(Clone, Debug)]
pub struct ResultStore<V> {
    values: BTreeMap<NodeId, V>,
}

impl<V> Default for ResultStore<V> {
    fn default() -> Self {
        Self { values: BTreeMap::new() }
    }
}

impl<V> ResultStore<V> {
    /// Creates an empty result store.
    pub fn new() -> Self {
        Self { values: BTreeMap::new() }
    }

    /// Returns the value currently associated with a node.
    pub fn get(&self, node_id: NodeId) -> Option<&V> {
        self.values.get(&node_id)
    }

    /// Returns mutable access to a node's opaque value.
    pub fn get_mut(&mut self, node_id: NodeId) -> Option<&mut V> {
        self.values.get_mut(&node_id)
    }

    /// Inserts or replaces a node's opaque value, returning the old value.
    pub fn insert(&mut self, node_id: NodeId, value: V) -> Option<V> {
        self.values.insert(node_id, value)
    }

    /// Removes a node's value, if present.
    pub fn remove(&mut self, node_id: NodeId) -> Option<V> {
        self.values.remove(&node_id)
    }

    /// Returns whether a node has a stored value.
    pub fn contains(&self, node_id: NodeId) -> bool {
        self.values.contains_key(&node_id)
    }

    /// Returns the number of stored node values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Iterates over node IDs and opaque values in deterministic key order.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &V)> {
        self.values.iter().map(|(node_id, value)| (*node_id, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeIdAllocator;

    #[test]
    fn stores_opaque_values_by_node_id() {
        let allocator = NodeIdAllocator::default();
        let first = allocator.allocate().unwrap();
        let second = allocator.allocate().unwrap();

        let mut store = ResultStore::new();
        assert!(store.insert(first, String::from("3 m")).is_none());
        assert!(store.insert(second, String::from("4 m")).is_none());
        assert_eq!(store.get(first).map(String::as_str), Some("3 m"));
        assert_eq!(store.insert(first, String::from("5 m")).as_deref(), Some("3 m"));
        assert_eq!(store.get(first).map(String::as_str), Some("5 m"));
        assert_eq!(store.remove(second).as_deref(), Some("4 m"));
        assert!(!store.contains(second));
        assert_eq!(store.len(), 1);
    }
}
