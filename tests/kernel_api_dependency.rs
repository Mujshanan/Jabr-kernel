//! Verify that downstream engines can depend on the standalone API crate.
use jabr_kernel_api::{
    Diagnostic, Envelope, Location, NodeIdAllocator, Port, Severity, Span,
};

#[test]
fn api_crate_exposes_contract_without_kernel_implementation() {
    let id = NodeIdAllocator::default().allocate().unwrap();
    let port = Port::new(id, 0);
    assert_eq!(port.node_id, id);
    assert_eq!(port.index, 0);

    let span = Span::new(4, 8).unwrap();
    let location = Location::new(1, 3).unwrap();
    let diagnostic = Diagnostic::new("math.parse.invalid", Severity::Error, "Invalid expression")
        .with_span(span);
    let envelope: Envelope<()> = Envelope::failure(vec![diagnostic]);
    assert_eq!(envelope.schema_version(), 1);
    assert_eq!(envelope.result, None);
    assert_eq!(location.column(), 3);
}
