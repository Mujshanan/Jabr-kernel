//! Immutable snapshots of generic Kernel state.
use std::collections::BTreeMap;

/// Point-in-time state copy. Payloads are opaque to Kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateSnapshot<K, V> {
    /// Global store revision.
    pub revision: u64,
    /// Captured entries and their last-write revisions.
    pub entries: BTreeMap<K, SnapshotEntry<V>>,
}

/// A value captured in a snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotEntry<V> {
    /// Opaque engine-owned payload.
    pub value: V,
    /// Revision at which the value was last written.
    pub revision: u64,
}
