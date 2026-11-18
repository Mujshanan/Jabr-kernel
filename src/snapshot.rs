//! Complete, immutable revisioned payload snapshots.

use crate::Revision;

/// A complete payload captured at a particular revision.
///
/// Snapshots are used for persistence/load and full-state publication. They are
/// distinct from incremental change notifications.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Snapshot<T> {
    /// Revision represented by this complete payload.
    pub revision: Revision,
    /// Complete opaque payload.
    pub payload: T,
}

impl<T> Snapshot<T> {
    /// Creates a revisioned snapshot.
    pub const fn new(revision: Revision, payload: T) -> Self {
        Self { revision, payload }
    }

    /// Maps the payload while preserving its revision.
    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> Snapshot<U> {
        Snapshot { revision: self.revision, payload: map(self.payload) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_preserves_complete_payload_and_revision() {
        let snapshot = Snapshot::new(Revision(7), vec!["a", "b"]);
        assert_eq!(snapshot.revision, Revision(7));
        assert_eq!(snapshot.payload, vec!["a", "b"]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn snapshot_round_trips_inside_versioned_envelope() {
        let snapshot = Snapshot::new(Revision(4), "opaque");
        let envelope = crate::Envelope::success(snapshot);
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded: crate::Envelope<Snapshot<&str>> =
            serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.result.unwrap().revision, Revision(4));
        assert_eq!(decoded.result.unwrap().payload, "opaque");
    }
}
