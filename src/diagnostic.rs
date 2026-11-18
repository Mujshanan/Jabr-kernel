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
    /// Stable namespaced code, such as kernel.input.invalid or math.parse.invalid.
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
        let d = Diagnostic::new("kernel.input.invalid", Severity::Error, "Invalid input");
        assert_eq!(d.code, "kernel.input.invalid");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn diagnostic_round_trips_with_a_source_span() {
        let diagnostic = Diagnostic::new("kernel.input.invalid", Severity::Error, "Invalid input")
            .with_span(Span::new(4, 9).unwrap());

        let json = serde_json::to_string(&diagnostic).unwrap();
        let decoded: Diagnostic = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, diagnostic);
    }
}
