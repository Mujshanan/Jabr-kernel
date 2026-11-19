//! One-based source locations.

/// One-based line and Unicode-scalar column, not byte or UTF-16 offsets.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Location {
    line: u32,
    column: u32,
}
impl Location {
    /// Constructs a location; line-zero takes precedence when both are invalid.
    pub const fn new(line: u32, column: u32) -> Result<Self, LocationError> {
        if line == 0 {
            Err(LocationError::ZeroLine)
        } else if column == 0 {
            Err(LocationError::ZeroColumn)
        } else {
            Ok(Self { line, column })
        }
    }
    /// One-based line.
    pub const fn line(self) -> u32 {
        self.line
    }
    /// One-based Unicode-scalar column.
    pub const fn column(self) -> u32 {
        self.column
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Location {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Repr {
            line: u32,
            column: u32,
        }
        let r = <Repr as serde::Deserialize>::deserialize(d)?;
        Self::new(r.line, r.column).map_err(serde::de::Error::custom)
    }
}
/// Invalid source coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocationError {
    /// Line must be positive.
    ZeroLine,
    /// Column must be positive.
    ZeroColumn,
}
impl core::fmt::Display for LocationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroLine => f.write_str("source line must be greater than zero"),
            Self::ZeroColumn => f.write_str("source column must be greater than zero"),
        }
    }
}
impl std::error::Error for LocationError {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_order() {
        assert_eq!(Location::new(0, 0), Err(LocationError::ZeroLine));
        assert_eq!(Location::new(1, 0), Err(LocationError::ZeroColumn));
    }
}
