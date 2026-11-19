//! Generic versioned result and diagnostics transport container.
use crate::Diagnostic;
/// Initial envelope schema version.
pub const ENVELOPE_SCHEMA_VERSION: u32 = 1;
/// Versioned result and diagnostics container.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Envelope<T> {
    schema_version: u32,
    /// Optional operation result.
    pub result: Option<T>,
    /// Diagnostics emitted by the operation.
    pub diagnostics: Vec<Diagnostic>,
}
#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Envelope<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Repr<T> {
            schema_version: u32,
            result: Option<T>,
            diagnostics: Vec<Diagnostic>,
        }
        let r = Repr::deserialize(d)?;
        if r.schema_version != ENVELOPE_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(format_args!(
                "unsupported envelope schema version {}; expected {}",
                r.schema_version, ENVELOPE_SCHEMA_VERSION
            )));
        }
        Ok(Self {
            schema_version: r.schema_version,
            result: r.result,
            diagnostics: r.diagnostics,
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
    /// Read-only schema version.
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }
    /// Appends a diagnostic without changing the schema version.
    pub fn with_diagnostic(mut self, diagnostic: Diagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_is_pinned() {
        assert_eq!(Envelope::success(7).schema_version(), 1);
    }
}
