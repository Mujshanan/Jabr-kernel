//! Immutable point-in-time copies of shared state.
use std::collections::BTreeMap;
use crate::{NodeId, Revision};

/// Immutable captured state and the revision at which it was observed.
#[derive(Clone, Debug)]
pub struct Snapshot<T> {
    revision: Revision,
    entries: BTreeMap<NodeId, T>,
}
impl<T> Snapshot<T> {
    pub(crate) fn from_state(revision: Revision, entries: &BTreeMap<NodeId, T>) -> Self
    where T: Clone {
        Self { revision, entries: entries.clone() }
    }
    /// Returns the revision captured by this snapshot.
    pub fn revision(&self) -> Revision { self.revision }
    /// Returns the value associated with an ID in this snapshot.
    pub fn get(&self, id: NodeId) -> Option<&T> { self.entries.get(&id) }
    /// Returns whether an ID is present in this snapshot.
    pub fn contains(&self, id: NodeId) -> bool { self.entries.contains_key(&id) }
    /// Returns the number of captured values.
    pub fn len(&self) -> usize { self.entries.len() }
    /// Returns whether the snapshot contains no values.
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    /// Iterates over captured values in ascending NodeId order.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.entries.iter().map(|(id, value)| (*id, value))
    }
}
#[cfg(test)]
mod tests {
    use crate::{NodeId, State};
    fn id(raw: u64) -> NodeId { NodeId::new(raw).unwrap() }
    #[test]
    fn snapshot_is_immutable_and_does_not_advance_source() {
        let mut state = State::new();
        state.set(id(1), String::from("before")).unwrap();
        let revision = state.revision();
        let snapshot = state.snapshot();
        assert_eq!(snapshot.revision(), revision);
        assert_eq!(state.revision(), revision);
        assert_eq!(snapshot.get(id(1)).map(String::as_str), Some("before"));
        state.set(id(1), String::from("after")).unwrap();
        state.set(id(2), String::from("new")).unwrap();
        assert_eq!(state.remove(id(1)).unwrap(), Some(String::from("after")));
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot.get(id(1)).map(String::as_str), Some("before"));
        assert!(!snapshot.contains(id(2)));
        assert_eq!(snapshot.iter().next().map(|(id, _)| id), Some(id(1)));
    }
}
