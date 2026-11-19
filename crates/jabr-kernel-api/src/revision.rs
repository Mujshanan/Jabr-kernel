//! Monotonic revisions for committed shared-state changes.
use core::fmt;

/// Identifies a committed version of a complete shared state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(u64);

/// Failure to advance a revision without wrapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RevisionError {
    /// Revision that could not be advanced.
    pub current: Revision,
}

impl Revision {
    /// Initial revision of a newly created state.
    pub const INITIAL: Self = Self(0);

    /// Constructs a revision for internal boundary tests.
    #[cfg(test)]
    pub(crate) const fn from_raw_for_test(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the numeric revision.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Returns the next revision, or an error at the maximum value.
    pub fn checked_next(self) -> Result<Self, RevisionError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(RevisionError { current: self })
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for RevisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "revision {} cannot be advanced", self.current)
    }
}

impl std::error::Error for RevisionError {}

#[cfg(feature = "serde")]
impl serde::Serialize for Revision {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Revision {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        value
            .parse::<u64>()
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_zero_and_increments() {
        assert_eq!(Revision::INITIAL.get(), 0);
        assert_eq!(Revision::INITIAL.checked_next().unwrap().get(), 1);
    }

    #[test]
    fn maximum_revision_does_not_wrap() {
        let max = Revision(u64::MAX);
        assert_eq!(max.checked_next(), Err(RevisionError { current: max }));
    }
}
