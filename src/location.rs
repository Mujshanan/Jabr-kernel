//! One-based source locations independent of any source encoding.

/// A one-based line and Unicode-scalar column in a source document.
///
/// Columns count Unicode scalar values from the beginning of the line. They are
/// not UTF-8 byte offsets, UTF-16 code-unit offsets, grapheme counts, or visual
/// columns. Use the Span type for UTF-8 byte ranges.
///
/// The fields are private so invalid zero coordinates cannot be constructed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Location {
    line: u32,
    column: u32,
}

impl Location {
    /// Constructs a location, rejecting zero-valued coordinates.
    pub const fn new(line: u32, column: u32) -> Result<Self, LocationError> {
        if line == 0 {
            return Err(LocationError::ZeroLine);
        }
        if column == 0 {
            return Err(LocationError::ZeroColumn);
        }
        Ok(Self { line, column })
    }

    /// Returns the one-based line number.
    pub const fn line(self) -> u32 {
        self.line
    }

    /// Returns the one-based Unicode-scalar column.
    pub const fn column(self) -> u32 {
        self.column
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Location {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Repr {
            line: u32,
            column: u32,
        }

        let repr = <Repr as serde::Deserialize>::deserialize(deserializer)?;
        Self::new(repr.line, repr.column).map_err(serde::de::Error::custom)
    }
}

/// Invalid one-based source-location coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocationError {
    /// Line numbers must be greater than zero.
    ZeroLine,
    /// Column numbers must be greater than zero.
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
    fn coordinates_are_one_based() {
        let location = Location::new(1, 1).unwrap();
        assert_eq!(location.line(), 1);
        assert_eq!(location.column(), 1);
        assert_eq!(Location::new(0, 1), Err(LocationError::ZeroLine));
        assert_eq!(Location::new(1, 0), Err(LocationError::ZeroColumn));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_rejects_zero_coordinates_and_round_trips_valid_locations() {
        assert!(serde_json::from_str::<Location>(r#"{"line":0,"column":1}"#).is_err());
        assert!(serde_json::from_str::<Location>(r#"{"line":1,"column":0}"#).is_err());

        let location = Location::new(3, 7).unwrap();
        let json = serde_json::to_string(&location).unwrap();
        assert_eq!(serde_json::from_str::<Location>(&json).unwrap(), location);
    }
}
