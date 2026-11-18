//! Structured diagnostics with stable owner-namespaced codes.
use crate::Span;

/// Diagnostic severity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Severity {
    /// Informational note.
    Info,
    /// Non-fatal issue.
    Warning,
    /// Failed operation.
    Error,
}

/// Machine-readable diagnostic. Codes are stable and owned by the emitting crate.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    /// Stable namespaced code, such as kernel.state.missing or math.parse.invalid.
    pub code: String,
    /// Severity.
    pub severity: Severity,
    /// Human-readable message; consumers must not parse it.
    pub message: String,
    /// Optional source byte span.
    pub span: Option<Span>,
}
impl Diagnostic {
    /// Creates a diagnostic without a source span.
    pub fn new(code: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            span: None,
        }
    }
    /// Adds a source span.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn code_is_separate_from_display_message() {
        let d = Diagnostic::new("kernel.state.missing", Severity::Error, "Missing");
        assert_eq!(d.code, "kernel.state.missing");
    }
}
