//! Generic revisioned state storage; payload meaning belongs to engine crates.
use core::fmt;
use std::collections::BTreeMap;
use crate::{SnapshotEntry, StateSnapshot};

/// Kind of state transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ChangeKind {
    /// A key was created.
    Created,
    /// An existing key was assigned.
    Updated,
    /// A key was removed.
    Removed,
}

/// One key affected by a transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change<K> {
    /// Affected key.
    pub key: K,
    /// Transition kind.
    pub kind: ChangeKind,
}

/// Changes committed at one revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSet<K> {
    /// Revision assigned to the transition.
    pub revision: u64,
    /// Affected keys.
    pub changes: Vec<Change<K>>,
}

/// Generic state store with monotonic global revisions.
#[derive(Clone, Debug)]
pub struct StateStore<K: Ord, V> {
    revision: u64,
    entries: BTreeMap<K, SnapshotEntry<V>>,
    history: Vec<ChangeSet<K>>,
}
impl<K: Ord, V> Default for StateStore<K, V> {
    fn default() -> Self { Self { revision: 0, entries: BTreeMap::new(), history: Vec::new() } }
}
impl<K: Ord + Clone, V: Clone> StateStore<K, V> {
    /// Current global revision; a new store starts at zero.
    pub fn revision(&self) -> u64 { self.revision }
    /// Reads the current payload for key.
    pub fn get(&self, key: &K) -> Option<&V> { self.entries.get(key).map(|e| &e.value) }
    /// Creates or updates a payload and records one revision.
    pub fn set(&mut self, key: K, value: V) -> Result<u64, StateError> {
        let revision = self.next_revision()?;
        let kind = if self.entries.contains_key(&key) { ChangeKind::Updated } else { ChangeKind::Created };
        self.entries.insert(key.clone(), SnapshotEntry { value, revision });
        self.revision = revision;
        self.history.push(ChangeSet { revision, changes: vec![Change { key, kind }] });
        Ok(revision)
    }
    /// Removes a key; removing a missing key is a no-op.
    pub fn remove(&mut self, key: &K) -> Result<bool, StateError> {
        if !self.entries.contains_key(key) { return Ok(false); }
        let revision = self.next_revision()?;
        self.entries.remove(key);
        self.revision = revision;
        self.history.push(ChangeSet { revision, changes: vec![Change { key: key.clone(), kind: ChangeKind::Removed }] });
        Ok(true)
    }
    /// Returns changes strictly newer than revision.
    pub fn changes_since(&self, revision: u64) -> Result<Vec<ChangeSet<K>>, StateError> {
        if revision > self.revision {
            return Err(StateError::FutureRevision { requested: revision, current: self.revision });
        }
        Ok(self.history.iter().filter(|c| c.revision > revision).cloned().collect())
    }
    /// Captures a detached snapshot unaffected by later writes.
    pub fn snapshot(&self) -> StateSnapshot<K, V> {
        StateSnapshot { revision: self.revision, entries: self.entries.clone() }
    }
    fn next_revision(&self) -> Result<u64, StateError> {
        self.revision.checked_add(1).ok_or(StateError::RevisionExhausted)
    }
}

/// State-store operation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateError {
    /// Revision counter cannot advance further.
    RevisionExhausted,
    /// A query requested a future revision.
    FutureRevision { requested: u64, current: u64 },
}
impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RevisionExhausted => f.write_str("state revision counter exhausted"),
            Self::FutureRevision { requested, current } =>
                write!(f, "requested revision {requested} is newer than current revision {current}"),
        }
    }
}
impl std::error::Error for StateError {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn snapshots_remain_unchanged_after_updates() {
        let mut s = StateStore::default();
        assert_eq!(s.set("x", "3 m").unwrap(), 1);
        let old = s.snapshot();
        assert_eq!(s.set("x", "4 m").unwrap(), 2);
        assert_eq!(old.revision, 1);
        assert_eq!(old.entries["x"].value, "3 m");
        assert_eq!(s.changes_since(1).unwrap()[0].changes[0].kind, ChangeKind::Updated);
    }
    #[test] fn missing_removal_is_noop_and_future_revision_fails() {
        let mut s: StateStore<&str, i32> = StateStore::default();
        assert!(!s.remove(&"missing").unwrap());
        assert_eq!(s.revision(), 0);
        assert_eq!(s.changes_since(1), Err(StateError::FutureRevision { requested: 1, current: 0 }));
    }
}
