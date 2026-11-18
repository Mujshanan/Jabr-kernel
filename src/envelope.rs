//! Generic versioned JSON transport container.
//! Cross-engine graph construction and execution coordination are owned by Jabr Engine.
use crate::Diagnostic;

/// Initial version of the envelope schema.
pub const ENVELOPE_SCHEMA_VERSION: u32 = 1;

/// Versioned result and diagnostics container.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Envelope<T> {
    /// Envelope schema version; independent of diagnostic-code versions.
    pub schema_version: u32,
    /// Optional operation result.
    pub result: Option<T>,
    /// Diagnostics emitted by the operation.
    pub diagnostics: Vec<Diagnostic>,
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for Envelope<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
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

    #[cfg(feature = "serde")]
    #[test]
    fn envelope_rejects_unsupported_schema_versions() {
        let old = r#"{"schema_version":0,"result":42,"diagnostics":[]}"#;
        let future = r#"{"schema_version":2,"result":42,"diagnostics":[]}"#;
        assert!(serde_json::from_str::<Envelope<u32>>(old).is_err());
        assert!(serde_json::from_str::<Envelope<u32>>(future).is_err());
    }
}
