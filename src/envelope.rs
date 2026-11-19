//! Generic versioned JSON transport container.
//!
//! The envelope is domain-neutral: result payloads and diagnostic codes are
//! defined by their owning engines. Schema versioning is independent of
//! diagnostic-code versioning.
use crate::Diagnostic;

/// Initial version of the envelope schema.
pub const ENVELOPE_SCHEMA_VERSION: u32 = 1;

/// Versioned result and diagnostics container.
///
/// Serde deserialization is intentionally pinned to
/// `ENVELOPE_SCHEMA_VERSION`: both older and newer versions are rejected.
/// A future schema-version change must define an explicit migration or
/// compatibility adapter for persisted and cross-crate envelopes.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Envelope<T> {
    /// Envelope schema version; independent of diagnostic-code versions.
    pub schema_version: u32,
    /// Optional operation result. A result may coexist with warning/info diagnostics.
    pub result: Option<T>,
    /// Diagnostics emitted by the operation.
    pub diagnostics: Vec<Diagnostic>,
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for Envelope<T>
where T: serde::Deserialize<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        #[derive(serde::Deserialize)]
        struct Repr<T> {
            schema_version: u32,
            result: Option<T>,
            diagnostics: Vec<Diagnostic>,
        }
        let repr = <Repr<T> as serde::Deserialize>::deserialize(deserializer)?;
        if repr.schema_version != ENVELOPE_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(format_args!(
                "unsupported envelope schema version {}; expected {}",
                repr.schema_version, ENVELOPE_SCHEMA_VERSION
            )));
        }
        Ok(Self {
            schema_version: repr.schema_version,
            result: repr.result,
            diagnostics: repr.diagnostics,
        })
    }
}

impl<T> Envelope<T> {
    /// Wraps a successful result.
    pub fn success(result: T) -> Self {
        Self { schema_version: ENVELOPE_SCHEMA_VERSION, result: Some(result), diagnostics: Vec::new() }
    }
    /// Wraps diagnostics without a result.
    pub fn failure(diagnostics: Vec<Diagnostic>) -> Self {
        Self { schema_version: ENVELOPE_SCHEMA_VERSION, result: None, diagnostics }
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
    use crate::{Severity, Span};

    #[test]
    fn success_can_retain_non_fatal_diagnostics() {
        let envelope = Envelope::success(42).with_diagnostic(Diagnostic::new(
            "math.parse.note", Severity::Info, "parsed with a note",
        ));
        assert_eq!(envelope.schema_version, ENVELOPE_SCHEMA_VERSION);
        assert_eq!(envelope.result, Some(42));
        assert_eq!(envelope.diagnostics.len(), 1);
    }

    #[test]
    fn failure_has_no_result_and_retains_all_diagnostics() {
        let diagnostics = vec![
            Diagnostic::new("math.parse.invalid", Severity::Error, "invalid expression"),
            Diagnostic::new("math.parse.note", Severity::Info, "additional context"),
        ];
        let envelope = Envelope::<u32>::failure(diagnostics.clone());
        assert_eq!(envelope.result, None);
        assert_eq!(envelope.diagnostics, diagnostics);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trips_result_and_diagnostics_with_optional_spans() {
        let envelope = Envelope::success(42_u32)
            .with_diagnostic(Diagnostic::new("math.parse.note", Severity::Warning, "review input"))
            .with_diagnostic(
                Diagnostic::new("math.parse.invalid", Severity::Error, "invalid token")
                    .with_span(Span::new(4, 9).unwrap()),
            );
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded: Envelope<u32> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, envelope);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_rejects_older_and_newer_schema_versions() {
        for version in [0, ENVELOPE_SCHEMA_VERSION + 1] {
            let json = format!(r#"{{"schema_version":{version},"result":42,"diagnostics":[]}}"#);
            assert!(serde_json::from_str::<Envelope<u32>>(&json).is_err());
        }
    }
}
