//! Source-independent UTF-8 byte spans.

/// A half-open UTF-8 byte range in a source document.
///
/// The range is `[start, end)`: `start` is inclusive and `end` is
/// exclusive. Empty spans are valid. This type does not validate source length
/// or UTF-8 character boundaries because Kernel does not own source text.
///
/// When transporting offsets through JavaScript `Number`, callers must keep
/// them at or below `2^53 - 1`, or use a lossless representation such as a
/// decimal string or fixed-width integer encoding.
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
            Self::Reversed { start, end } => write!(f, "span end {end} precedes start {start}"),
        }
    }
}
impl std::error::Error for SpanError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_half_open_ranges_and_allows_empty_spans() {
        assert_eq!(
            Span::new(8, 3),
            Err(SpanError::Reversed { start: 8, end: 3 })
        );
        let span = Span::new(3, 7).unwrap();
        assert_eq!((span.start(), span.end(), span.len()), (3, 7, 4));
        assert!(!span.is_empty());
        let empty = Span::new(0, 0).unwrap();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn error_has_stable_display_and_error_trait() {
        let error = SpanError::Reversed { start: 8, end: 3 };
        assert_eq!(error.to_string(), "span end 3 precedes start 8");
        fn assert_error<E: std::error::Error>() {}
        assert_error::<SpanError>();
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trips_regular_and_empty_spans() {
        for span in [Span::new(2, 5).unwrap(), Span::new(0, 0).unwrap()] {
            let json = serde_json::to_string(&span).unwrap();
            assert_eq!(serde_json::from_str::<Span>(&json).unwrap(), span);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_rejects_malformed_and_reversed_spans() {
        for json in [
            r#"{"start":1}"#,
            r#"{"start":"1","end":2}"#,
            r#"{"start":-1,"end":2}"#,
            r#"{"start":1.5,"end":2}"#,
            r#"{"start":8,"end":3}"#,
        ] {
            assert!(
                serde_json::from_str::<Span>(json).is_err(),
                "accepted {json}"
            );
        }
    }
}
