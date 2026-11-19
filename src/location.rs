//! One-based source locations independent of any source encoding.

/// A one-based line and Unicode-scalar column in a source document.
///
/// Columns count Unicode scalar values from the beginning of the line. They are
/// not UTF-8 byte offsets, UTF-16 code-unit offsets, grapheme counts, or visual
/// columns. Use `Span` for half-open UTF-8 byte ranges.
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
    ///
    /// Validation checks the line before the column, so `(0, 0)` reports
    /// `LocationError::ZeroLine`.
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
    fn coordinates_are_one_based_with_line_first_validation() {
        let location = Location::new(12, 34).unwrap();
        assert_eq!((location.line(), location.column()), (12, 34));
        assert_eq!(Location::new(0, 0), Err(LocationError::ZeroLine));
        assert_eq!(Location::new(0, 4), Err(LocationError::ZeroLine));
        assert_eq!(Location::new(4, 0), Err(LocationError::ZeroColumn));
    }

    #[test]
    fn errors_have_stable_display_and_error_trait() {
        assert_eq!(
            LocationError::ZeroLine.to_string(),
            "source line must be greater than zero"
        );
        assert_eq!(
            LocationError::ZeroColumn.to_string(),
            "source column must be greater than zero"
        );
        fn assert_error<E: std::error::Error>() {}
        assert_error::<LocationError>();
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trips_multi_digit_location() {
        let location = Location::new(123, 456).unwrap();
        let json = serde_json::to_string(&location).unwrap();
        assert_eq!(serde_json::from_str::<Location>(&json).unwrap(), location);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_rejects_malformed_and_zero_coordinates() {
        for json in [
            r#"{"line":0,"column":0}"#,
            r#"{"line":0,"column":4}"#,
            r#"{"line":4,"column":0}"#,
            r#"{"line":-1,"column":2}"#,
            r#"{"line":"1","column":2}"#,
            r#"{"line":1.5,"column":2}"#,
            r#"{"line":1}"#,
        ] {
            assert!(
                serde_json::from_str::<Location>(json).is_err(),
                "accepted {json}"
            );
        }
    }
}
