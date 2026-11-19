//! Verify downstream use of the standalone API crate and state contracts.
use jabr_kernel_api::{
    Diagnostic, Envelope, Location, NodeId, NodeIdAllocator, Port, Revision, Severity, Span,
    State,
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

fn id(raw: u64) -> NodeId {
    NodeId::new(raw).unwrap()
}

#[test]
fn downstream_state_revision_and_snapshot_contract() {
    let mut state = State::new();
    assert_eq!(state.revision(), Revision::INITIAL);

    let first = state.set(id(7), String::from("value")).unwrap();
    assert_eq!(first.get(), 1);
    let snapshot = state.snapshot();

    state.set(id(7), String::from("updated")).unwrap();
    assert_eq!(snapshot.revision(), first);
    assert_eq!(snapshot.get(id(7)).map(String::as_str), Some("value"));
    assert_eq!(state.get(id(7)).map(String::as_str), Some("updated"));

    let before_missing_remove = state.revision();
    assert_eq!(state.remove(id(99)).unwrap(), None);
    assert_eq!(state.revision(), before_missing_remove);
    assert_eq!(state.remove(id(7)).unwrap(), Some(String::from("updated")));
    assert_eq!(state.revision().get(), 3);
}

#[cfg(feature = "serde")]
#[test]
fn revision_serde_preserves_full_u64_precision_as_decimal_string() {
    let revision = Revision::INITIAL;
    let max_json = format!("\"{}\"", u64::MAX);
    let max: Revision = serde_json::from_str(&max_json).unwrap();
    assert_eq!(max.get(), u64::MAX);
    assert_eq!(serde_json::to_string(&max).unwrap(), max_json);
    for invalid in ["-1", "1.5", "\"\"", "\"01\"", "\"18446744073709551616\""] {
        assert!(serde_json::from_str::<Revision>(invalid).is_err(), "{invalid}");
    }
    assert_eq!(revision.get(), 0);
}
