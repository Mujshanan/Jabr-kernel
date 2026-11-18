//! Source-independent UTF-8 byte spans.

/// Half-open UTF-8 byte range in a source document.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    /// Constructs a span and rejects reversed ranges.
    pub fn new(start: usize, end: usize) -> Result<Self, SpanError> {
        if start > end {
            return Err(SpanError::Reversed { start, end });
        }
        Ok(Self { start, end })
    }

    /// Returns the inclusive starting byte offset.
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the exclusive ending byte offset.
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns the span's byte length.
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    /// Returns whether the span is empty.
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Span {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Repr {
            start: usize,
            end: usize,
        }

        let repr = <Repr as serde::Deserialize>::deserialize(deserializer)?;
        Self::new(repr.start, repr.end).map_err(serde::de::Error::custom)
    }
}

/// Invalid source-span data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpanError {
    /// The exclusive end offset precedes the inclusive start offset.
    Reversed {
        /// Inclusive starting byte offset.
        start: usize,
        /// Exclusive ending byte offset.
        end: usize,
    },
}

impl core::fmt::Display for SpanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Reversed { start, end } => {
                write!(f, "span end {end} precedes start {start}")
            }
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
        let span = Span::new(3, 7).unwrap();
        assert_eq!(span.start(), 3);
        assert_eq!(span.end(), 7);
        assert_eq!(span.len(), 4);
        assert!(!span.is_empty());
        assert!(Span::new(3, 3).unwrap().is_empty());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_rejects_reversed_ranges() {
        assert!(serde_json::from_str::<Span>(r#"{"start":8,"end":3}"#).is_err());
        let span = Span::new(2, 5).unwrap();
        let json = serde_json::to_string(&span).unwrap();
        assert_eq!(serde_json::from_str::<Span>(&json).unwrap(), span);
    }
}
