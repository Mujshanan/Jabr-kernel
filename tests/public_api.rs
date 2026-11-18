//! Compile-time and integration checks for the public Kernel API.

use jabr_kernel::{
    DependencyGraph, Diagnostic, Envelope, Location, LocationError, NodeIdAllocator, Port,
    Severity, Span,
};

#[test]
fn public_generic_contract_composes_without_domain_types() {
    let node = NodeIdAllocator::default().allocate().unwrap();
    let port = Port::new(node, 0);
    let location = Location::new(2, 5).unwrap();
    assert_eq!(location.line(), 2);
    assert_eq!(location.column(), 5);
    assert_eq!(Location::new(0, 1), Err(LocationError::ZeroLine));
    assert_eq!(Location::new(1, 0), Err(LocationError::ZeroColumn));

    let mut graph = DependencyGraph::default();
    graph.add_node(node);
    assert_eq!(graph.topological_order().unwrap(), vec![node]);

    let diagnostic = Diagnostic::new("kernel.input.invalid", Severity::Error, "Invalid input")
        .with_span(Span::new(4, 9).unwrap());

    let response: Envelope<String> =
        Envelope::success("opaque payload".to_owned()).with_diagnostic(diagnostic);
    assert_eq!(
        response.schema_version,
        jabr_kernel::ENVELOPE_SCHEMA_VERSION
    );
    assert_eq!(response.result.as_deref(), Some("opaque payload"));
    assert_eq!(response.diagnostics.len(), 1);
    assert_eq!(response.diagnostics[0].span.unwrap().start(), 4);
    assert_eq!(port.node_id, node);
}

#[test]
fn failure_envelope_has_no_result_and_preserves_diagnostics() {
    let diagnostic = Diagnostic::new(
        "kernel.operation.failed",
        Severity::Error,
        "Operation failed",
    );
    let response: Envelope<()> = Envelope::failure(vec![diagnostic.clone()]);

    assert_eq!(response.schema_version, jabr_kernel::ENVELOPE_SCHEMA_VERSION);
    assert_eq!(response.result, None);
    assert_eq!(response.diagnostics, vec![diagnostic]);
}
