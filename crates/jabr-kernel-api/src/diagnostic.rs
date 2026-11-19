//! Structured diagnostics with stable machine-readable codes.
use crate::Span;
/// Diagnostic severity.
#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature="serde",serde(rename_all="snake_case"))]
pub enum Severity{/// Informational note. Info,/// Non-fatal issue. Warning,/// Failed operation. Error}
/// Machine-readable diagnostic. Consumers match code, never human-readable message.
#[derive(Clone,Debug,Eq,PartialEq)]
#[cfg_attr(feature="serde",derive(serde::Serialize,serde::Deserialize))]
pub struct Diagnostic{
 /// Stable owner-namespaced code.
 pub code:String,
 /// Severity.
 pub severity:Severity,
 /// Human-readable message.
 pub message:String,
 /// Optional UTF-8 byte span.
 pub span:Option<Span>,
}
impl Diagnostic{
 /// Creates a diagnostic without a source span.
 pub fn new(code:impl Into<String>,severity:Severity,message:impl Into<String>)->Self{Self{code:code.into(),severity,message:message.into(),span:None}}
 /// Adds a source span.
 pub fn with_span(mut self,span:Span)->Self{self.span=Some(span);self}
}
#[cfg(test)]mod tests{use super::*;#[test]fn code_is_machine_readable(){let d=Diagnostic::new("kernel.input.invalid",Severity::Error,"Invalid input");assert_eq!(d.code,"kernel.input.invalid");}}
