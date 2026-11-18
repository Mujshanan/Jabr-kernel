//! Source-independent text spans and positions.
use core::fmt;

/// Half-open byte range in a source document.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span {
    /// Inclusive starting byte offset.
    pub start: usize,
    /// Exclusive ending byte offset.
    pub end: usize,
}
impl Span {
    /// Constructs a span and rejects reversed ranges.
    pub fn new(start: usize, end: usize) -> Result<Self, SpanError> {
        if start > end {
            return Err(SpanError::Reversed { start, end });
        }
        Ok(Self { start, end })
    }
    /// Returns byte length.
    pub const fn len(self) -> usize {
        self.end - self.start
    }
    /// Returns whether the span is empty.
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// One-based line and UTF-8 byte-column position.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Location {
    /// One-based line.
    pub line: usize,
    /// One-based UTF-8 byte column, measured from the start of the line.
    pub column: usize,
}
impl Location {
    /// Constructs a validated one-based location.
    pub fn new(line: usize, column: usize) -> Result<Self, SpanError> {
        if line == 0 || column == 0 {
            return Err(SpanError::ZeroLocation);
        }
        Ok(Self { line, column })
    }
}

/// Invalid source-position data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpanError {
    /// End offset precedes start offset.
    Reversed { start: usize, end: usize },
    /// Line and column must be one-based.
    ZeroLocation,
}
impl fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reversed { start, end } => write!(f, "span end {end} precedes start {start}"),
            Self::ZeroLocation => f.write_str("line and column numbers are one-based"),
        }
    }
}
impl std::error::Error for SpanError {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_ranges_are_rejected() {
        assert!(Span::new(8, 3).is_err());
        assert!(Span::new(3, 3).unwrap().is_empty());
        assert!(Location::new(0, 1).is_err());
    }
}
