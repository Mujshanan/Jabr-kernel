//! Monotonic revision primitive; lifecycle is owned by Engine.

/// Revision counter for one document/session stream.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Revision(pub u64);

impl Revision {
    /// The initial revision.
    pub const ZERO: Self = Self(0);

    /// Returns the underlying counter.
    pub const fn get(self) -> u64 { self.0 }

    /// Advances without wrapping.
    pub fn checked_next(self) -> Result<Self, RevisionError> {
        self.0.checked_add(1).map(Self).ok_or(RevisionError::Exhausted)
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
        assert_eq!(Revision(u64::MAX).checked_next(), Err(RevisionError::Exhausted));
    }
}
