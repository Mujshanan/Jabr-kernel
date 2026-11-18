//! Generic versioned JSON transport container.
//! Cross-engine graph construction and execution coordination are owned by the Orchestrator.
use crate::Diagnostic;

/// Initial version of the envelope schema.
pub const ENVELOPE_SCHEMA_VERSION: u32 = 1;

/// Versioned result and diagnostics container.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Envelope<T> {
    /// Envelope schema version; independent of diagnostic-code versions.
    pub schema_version: u32,
    /// Optional operation result.
    pub result: Option<T>,
    /// Diagnostics emitted by the operation.
    pub diagnostics: Vec<Diagnostic>,
}
impl<T> Envelope<T> {
    /// Wraps a successful result.
    pub fn success(result: T) -> Self {
        Self {
            schema_version: ENVELOPE_SCHEMA_VERSION,
            result: Some(result),
            diagnostics: Vec::new(),
        }
    }
    /// Wraps diagnostics without a result.
    pub fn failure(diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            schema_version: ENVELOPE_SCHEMA_VERSION,
            result: None,
            diagnostics,
        }
    }
    /// Appends a diagnostic without changing the envelope schema version.
    pub fn with_diagnostic(mut self, diagnostic: Diagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_is_explicit() {
        let e = Envelope::success(42).with_diagnostic(Diagnostic::new(
            "math.parse.note",
            crate::Severity::Info,
            "parsed",
        ));
        assert_eq!(e.schema_version, 1);
        assert_eq!(e.result, Some(42));
        assert_eq!(e.diagnostics.len(), 1);
    }
    #[cfg(feature = "serde")]
    #[test]
    fn envelope_round_trips_json() {
        let e = Envelope::success(42_u32);
        let json = serde_json::to_string(&e).unwrap();
        let decoded: Envelope<u32> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, e);
    }
}
