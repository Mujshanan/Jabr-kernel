//! Opaque, revisioned shared object state.
use std::collections::BTreeMap;

use crate::{NodeId, Revision, RevisionError, Snapshot};

/// Current committed mapping from stable object IDs to opaque values.
#[derive(Debug)]
pub struct State<T> {
    revision: Revision,
    entries: BTreeMap<NodeId, T>,
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> State<T> {
    /// Creates an empty state at revision zero.
    pub fn new() -> Self {
        Self {
            revision: Revision::INITIAL,
            entries: BTreeMap::new(),
        }
    }

    /// Returns the current committed revision.
    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// Returns the value associated with an ID.
    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.entries.get(&id)
    }

    /// Returns whether an ID is present.
    pub fn contains(&self, id: NodeId) -> bool {
        self.entries.contains_key(&id)
    }

    /// Returns the number of stored values.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the state contains no values.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates over values in ascending NodeId order.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.entries.iter().map(|(id, value)| (*id, value))
    }

    /// Inserts or replaces a value and advances the revision exactly once.
    ///
    /// Every call is an explicit assignment, even if the value compares equal
    /// to the previous value. The next revision is checked before mutation.
    pub fn set(&mut self, id: NodeId, value: T) -> Result<Revision, RevisionError> {
        let next = self.revision.checked_next()?;
        self.entries.insert(id, value);
        self.revision = next;
        Ok(next)
    }

    /// Removes a value, advancing the revision only when the ID exists.
    ///
    /// If revision advancement fails, the state is left unchanged.
    pub fn remove(&mut self, id: NodeId) -> Result<Option<T>, RevisionError> {
        if !self.entries.contains_key(&id) {
            return Ok(None);
        }

        let next = self.revision.checked_next()?;
        let removed = self.entries.remove(&id);
        self.revision = next;
        Ok(removed)
    }

    /// Captures an immutable point-in-time copy of this state.
    pub fn snapshot(&self) -> Snapshot<T>
    where
        T: Clone,
    {
        Snapshot::from_state(self.revision, &self.entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: u64) -> NodeId {
        NodeId::new(raw).unwrap()
    }

    #[test]
    fn mutations_advance_once_and_reads_do_not() {
        let mut state = State::new();
        assert_eq!(state.revision(), Revision::INITIAL);

        let first = state.set(id(2), "two").unwrap();
        assert_eq!(first.get(), 1);
        assert_eq!(state.get(id(2)), Some(&"two"));
        assert!(state.contains(id(2)));
        assert_eq!(state.len(), 1);

        let _: Vec<_> = state.iter().collect();
        assert_eq!(state.revision(), first);
        assert_eq!(state.set(id(2), "two").unwrap().get(), 2);
    }

    #[test]
    fn remove_existing_advances_and_missing_remove_is_noop() {
        let mut state = State::new();
        state.set(id(1), 10).unwrap();
        let revision = state.revision();

        assert_eq!(state.remove(id(99)).unwrap(), None);
        assert_eq!(state.revision(), revision);
        assert_eq!(state.remove(id(1)).unwrap(), Some(10));
        assert_eq!(state.revision(), revision.checked_next().unwrap());
        assert!(state.is_empty());
    }

    #[test]
    fn iteration_is_sorted_by_node_id() {
        let mut state = State::new();
        state.set(id(9), "nine").unwrap();
        state.set(id(2), "two").unwrap();
        state.set(id(5), "five").unwrap();

        assert_eq!(
            state.iter().map(|(id, _)| id.get()).collect::<Vec<_>>(),
            vec![2, 5, 9]
        );
    }

    #[test]
    fn overflow_leaves_set_and_remove_unchanged() {
        let mut state = State::new();
        state.set(id(1), "original").unwrap();
        state.revision = Revision::from_raw_for_test(u64::MAX);
        let error = RevisionError {
            current: Revision::from_raw_for_test(u64::MAX),
        };

        assert_eq!(state.set(id(1), "replacement"), Err(error));
        assert_eq!(state.get(id(1)), Some(&"original"));
        assert_eq!(state.remove(id(1)), Err(error));
        assert_eq!(state.get(id(1)), Some(&"original"));
        assert_eq!(state.revision().get(), u64::MAX);
    }
}
