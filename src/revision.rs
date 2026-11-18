//! Monotonic revision primitive; lifecycle is owned by Engine.

/// Revision counter for one document/session stream.
///
/// When serialized for JSON transport, the revision is encoded as a decimal
/// string. This preserves the full u64 range when consumed by JavaScript.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(pub u64);

impl Revision {
    /// The initial revision.
    pub const ZERO: Self = Self(0);

    /// Returns the underlying counter.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Advances without wrapping.
    pub fn checked_next(self) -> Result<Self, RevisionError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(RevisionError::Exhausted)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Revision {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Revision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        value
            .parse::<u64>()
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

/// Failure to advance a revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevisionError {
    /// The u64 revision space is exhausted.
    Exhausted,
}

impl core::fmt::Display for RevisionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Exhausted => f.write_str("revision counter exhausted"),
        }
    }
}

impl std::error::Error for RevisionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revisions_advance_monotonically() {
        assert_eq!(Revision::ZERO.checked_next().unwrap(), Revision(1));
        assert!(Revision(8) < Revision(9));
    }

    #[test]
    fn revision_overflow_is_explicit() {
        assert_eq!(
            Revision(u64::MAX).checked_next(),
            Err(RevisionError::Exhausted)
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_preserves_full_revision_precision_as_a_decimal_string() {
        let revision = Revision(u64::MAX);
        let json = serde_json::to_string(&revision).unwrap();
        assert_eq!(json, "\"18446744073709551615\"");
        assert_eq!(serde_json::from_str::<Revision>(&json).unwrap(), revision);
        assert!(serde_json::from_str::<Revision>("42").is_err());
    }
}
