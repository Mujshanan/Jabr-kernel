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
///
/// Codes should use a lowercase, dot-separated owner namespace such as
/// `<owner>.<domain>.<reason>` (for example, `math.parse.invalid_token`).
/// Consumers should branch on `code`; `message` is human-readable text and
/// must not be parsed for program behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    /// Stable owner-namespaced machine-readable code.
    pub code: String,
    /// Severity.
    pub severity: Severity,
    /// Human-readable message; consumers must not parse it.
    pub message: String,
    /// Optional half-open UTF-8 source byte span.
    pub span: Option<Span>,
}
impl Diagnostic {
    /// Creates a diagnostic without a source span.
    pub fn new(code: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
        Self { code: code.into(), severity, message: message.into(), span: None }
    }
    /// Sets or replaces the source span without changing other diagnostic fields.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constructor_starts_without_span() {
        let diagnostic = Diagnostic::new("kernel.input.invalid", Severity::Error, "Invalid input");
        assert_eq!(diagnostic.code, "kernel.input.invalid");
        assert_eq!(diagnostic.span, None);
    }

    #[test]
    fn with_span_sets_and_replaces_span_without_changing_other_fields() {
        let diagnostic = Diagnostic::new("math.parse.invalid", Severity::Warning, "Unexpected token");
        let first = Span::new(1, 3).unwrap();
        let second = Span::new(4, 8).unwrap();
        let updated = diagnostic.clone().with_span(first).with_span(second);
        assert_eq!(updated.code, diagnostic.code);
        assert_eq!(updated.severity, diagnostic.severity);
        assert_eq!(updated.message, diagnostic.message);
        assert_eq!(updated.span, Some(second));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn severity_uses_stable_snake_case_values_and_round_trips() {
        for (severity, expected) in [
            (Severity::Info, r#""info""#),
            (Severity::Warning, r#""warning""#),
            (Severity::Error, r#""error""#),
        ] {
            let json = serde_json::to_string(&severity).unwrap();
            assert_eq!(json, expected);
            assert_eq!(serde_json::from_str::<Severity>(&json).unwrap(), severity);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn diagnostic_without_span_round_trips() {
        let diagnostic = Diagnostic::new("kernel.input.invalid", Severity::Error, "Invalid input");
        let json = serde_json::to_string(&diagnostic).unwrap();
        let decoded: Diagnostic = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, diagnostic);
        assert_eq!(decoded.span, None);
    }
}
