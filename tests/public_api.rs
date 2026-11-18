//! External-consumer contract tests: use only the crate's public API.

use jabr_kernel::id::MAX_NODE_ID;
use jabr_kernel::{
    DependencyGraph, Diagnostic, Envelope, GraphError, IdAllocationError, Location, LocationError,
    NodeId, NodeIdAllocator, Port, Severity, Span, SpanError,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn identifiers_validate_boundaries_and_expose_stable_value_semantics() {
    assert_eq!(NodeId::new(0), Err(IdAllocationError::Zero));
    assert_eq!(
        NodeId::new(MAX_NODE_ID + 1),
        Err(IdAllocationError::Exhausted)
    );

    let min = NodeId::new(1).unwrap();
    let max = NodeId::new(MAX_NODE_ID).unwrap();
    assert_eq!(min.get(), 1);
    assert_eq!(max.get(), MAX_NODE_ID);
    assert!(min < max);
    assert_eq!(min.to_string(), "1");

    let mut first = DefaultHasher::new();
    let mut second = DefaultHasher::new();
    min.hash(&mut first);
    NodeId::new(1).unwrap().hash(&mut second);
    assert_eq!(first.finish(), second.finish());

    assert_eq!(
        NodeIdAllocator::new(0).unwrap_err(),
        IdAllocationError::Zero
    );
    assert_eq!(
        NodeIdAllocator::new(MAX_NODE_ID + 1).unwrap_err(),
        IdAllocationError::Exhausted
    );
    let last = NodeIdAllocator::new(MAX_NODE_ID).unwrap();
    assert_eq!(last.allocate().unwrap(), max);
    assert_eq!(last.allocate(), Err(IdAllocationError::Exhausted));
}

#[test]
fn cloned_allocators_share_sequence_and_independent_allocators_are_independent() {
    let allocator = NodeIdAllocator::new(10).unwrap();
    let clone = allocator.clone();
    assert_eq!(allocator.allocate().unwrap().get(), 10);
    assert_eq!(clone.allocate().unwrap().get(), 11);
    assert_eq!(
        NodeIdAllocator::new(10).unwrap().allocate().unwrap().get(),
        10
    );
}

#[test]
fn concurrent_cloned_allocators_issue_unique_monotonic_ids() {
    let allocator = NodeIdAllocator::new(100).unwrap();
    let mut workers = Vec::new();
    for _ in 0..8 {
        let local = allocator.clone();
        workers.push(std::thread::spawn(move || {
            (0..100)
                .map(|_| local.allocate().unwrap().get())
                .collect::<Vec<_>>()
        }));
    }

    let mut allocated = workers
        .into_iter()
        .flat_map(|worker| worker.join().unwrap())
        .collect::<Vec<_>>();
    allocated.sort_unstable();
    assert_eq!(allocated, (100..900).collect::<Vec<_>>());
}

#[test]
fn port_identity_includes_node_and_opaque_index() {
    let a = NodeId::new(1).unwrap();
    let b = NodeId::new(2).unwrap();
    assert_eq!(
        Port::new(a, 0),
        Port {
            node_id: a,
            index: 0
        }
    );
    assert_ne!(Port::new(a, 0), Port::new(a, 1));
    assert_ne!(Port::new(a, 0), Port::new(b, 0));
    assert_eq!(Port::new(a, u32::MAX).index, u32::MAX);
}

#[test]
fn graph_handles_empty_duplicate_and_removal_operations() {
    let mut graph = DependencyGraph::<u32>::default();
    assert!(graph.is_empty());
    assert_eq!(graph.len(), 0);
    assert_eq!(graph.topological_order().unwrap(), Vec::<u32>::new());

    graph.add_node(1);
    graph.add_node(1);
    graph.add_node(2);
    assert_eq!(graph.len(), 2);
    assert!(!graph.is_empty());

    graph.add_dependency(2, 1).unwrap();
    graph.add_dependency(2, 1).unwrap();
    assert_eq!(graph.topological_order().unwrap(), vec![1, 2]);

    assert!(graph.remove_node(&1));
    assert!(!graph.remove_node(&1));
    assert_eq!(graph.len(), 1);
    assert_eq!(graph.topological_order().unwrap(), vec![2]);
    assert!(graph.remove_node(&2));
    assert!(graph.is_empty());
}

#[test]
fn graph_distinguishes_unknown_dependent_and_prerequisite() {
    let mut graph = DependencyGraph::<u32>::default();
    graph.add_node(1);
    assert_eq!(graph.add_dependency(9, 1), Err(GraphError::UnknownNode(9)));
    assert_eq!(graph.add_dependency(1, 9), Err(GraphError::UnknownNode(9)));
}

#[test]
fn graph_order_is_deterministic_across_insertion_orders() {
    let mut first = DependencyGraph::default();
    for node in [4, 3, 2, 1] {
        first.add_node(node);
    }
    first.add_dependency(4, 2).unwrap();
    first.add_dependency(3, 1).unwrap();

    let mut second = DependencyGraph::default();
    for node in [1, 2, 3, 4] {
        second.add_node(node);
    }
    second.add_dependency(3, 1).unwrap();
    second.add_dependency(4, 2).unwrap();

    assert_eq!(first.topological_order().unwrap(), vec![1, 2, 3, 4]);
    assert_eq!(first.topological_order(), second.topological_order());
}

#[test]
fn graph_reports_self_cycles_and_all_unresolved_nodes() {
    let mut self_cycle = DependencyGraph::default();
    self_cycle.add_node(1);
    self_cycle.add_dependency(1, 1).unwrap();
    assert_eq!(
        self_cycle.topological_order(),
        Err(GraphError::Cycle(vec![1]))
    );

    let mut graph = DependencyGraph::default();
    for node in [1, 2, 3, 4, 5] {
        graph.add_node(node);
    }
    graph.add_dependency(1, 2).unwrap();
    graph.add_dependency(2, 1).unwrap();
    graph.add_dependency(3, 2).unwrap();
    assert_eq!(
        graph.topological_order(),
        Err(GraphError::Cycle(vec![1, 2, 3]))
    );
}

#[test]
fn spans_cover_empty_and_maximum_ranges_and_reject_reversal() {
    assert_eq!(Span::new(0, 0).unwrap().len(), 0);
    assert!(Span::new(usize::MAX, usize::MAX).unwrap().is_empty());
    assert_eq!(
        Span::new(usize::MAX, 0),
        Err(SpanError::Reversed {
            start: usize::MAX,
            end: 0
        })
    );
    let span = Span::new(2, 7).unwrap();
    assert_eq!((span.start(), span.end(), span.len()), (2, 7, 5));
}

#[test]
fn locations_validate_zero_and_accept_maximum_coordinates() {
    assert_eq!(Location::new(0, 1), Err(LocationError::ZeroLine));
    assert_eq!(Location::new(1, 0), Err(LocationError::ZeroColumn));
    let location = Location::new(u32::MAX, u32::MAX).unwrap();
    assert_eq!(location.line(), u32::MAX);
    assert_eq!(location.column(), u32::MAX);
}

#[test]
fn diagnostics_preserve_machine_code_and_human_message_separately() {
    for severity in [Severity::Info, Severity::Warning, Severity::Error] {
        let diagnostic = Diagnostic::new("kernel.test.example", severity, "Human-readable text");
        assert_eq!(diagnostic.code, "kernel.test.example");
        assert_eq!(diagnostic.message, "Human-readable text");
        assert_eq!(diagnostic.severity, severity);
        assert_eq!(diagnostic.span, None);
    }
    let with_span = Diagnostic::new("kernel.test.span", Severity::Warning, "Located")
        .with_span(Span::new(5, 8).unwrap());
    assert_eq!(with_span.span.unwrap().start(), 5);
}

#[test]
fn envelopes_construct_success_and_failure_without_domain_semantics() {
    let success: Envelope<&str> = Envelope::success("opaque");
    assert_eq!(success.schema_version(), jabr_kernel::ENVELOPE_SCHEMA_VERSION);
    assert_eq!(success.result, Some("opaque"));
    assert!(success.diagnostics.is_empty());

    let diagnostic = Diagnostic::new("kernel.test.failure", Severity::Error, "Failed");
    let failure: Envelope<()> = Envelope::failure(vec![diagnostic.clone()]);
    assert_eq!(failure.schema_version(), jabr_kernel::ENVELOPE_SCHEMA_VERSION);
    assert_eq!(failure.result, None);
    assert_eq!(failure.diagnostics, vec![diagnostic.clone()]);

    let appended = success.with_diagnostic(diagnostic.clone());
    assert_eq!(appended.schema_version(), jabr_kernel::ENVELOPE_SCHEMA_VERSION);
    assert_eq!(appended.result, Some("opaque"));
    assert_eq!(appended.diagnostics, vec![diagnostic]);
}

#[cfg(feature = "serde")]
mod serde_contract {
    use super::*;

    #[test]
    fn identifiers_and_ports_round_trip_and_reject_invalid_ids() {
        let id = NodeId::new(MAX_NODE_ID).unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(serde_json::from_str::<NodeId>(&json).unwrap(), id);
        assert!(serde_json::from_str::<NodeId>("0").is_err());
        assert!(serde_json::from_str::<NodeId>("9007199254740992").is_err());

        let port = Port::new(id, u32::MAX);
        let json = serde_json::to_string(&port).unwrap();
        assert_eq!(serde_json::from_str::<Port>(&json).unwrap(), port);
    }

    #[test]
    fn span_and_location_round_trip_and_reject_invalid_shapes() {
        let span = Span::new(3, 9).unwrap();
        assert_eq!(
            serde_json::from_str::<Span>(&serde_json::to_string(&span).unwrap()).unwrap(),
            span
        );
        assert!(serde_json::from_str::<Span>(r#"{"start":9,"end":3}"#).is_err());
        assert!(serde_json::from_str::<Span>(r#"{"start":1}"#).is_err());

        let location = Location::new(3, 7).unwrap();
        assert_eq!(
            serde_json::from_str::<Location>(&serde_json::to_string(&location).unwrap()).unwrap(),
            location
        );
        assert!(serde_json::from_str::<Location>(r#"{"line":0,"column":1}"#).is_err());
        assert!(serde_json::from_str::<Location>(r#"{"line":1,"column":0}"#).is_err());
        assert!(serde_json::from_str::<Location>(r#"{"line":1}"#).is_err());
    }

    #[test]
    fn diagnostic_severities_have_stable_snake_case_wire_values() {
        assert_eq!(serde_json::to_string(&Severity::Info).unwrap(), r#""info""#);
        assert_eq!(
            serde_json::to_string(&Severity::Warning).unwrap(),
            r#""warning""#
        );
        assert_eq!(
            serde_json::to_string(&Severity::Error).unwrap(),
            r#""error""#
        );

        let diagnostic = Diagnostic::new("kernel.test", Severity::Warning, "message");
        assert_eq!(
            serde_json::from_str::<Diagnostic>(&serde_json::to_string(&diagnostic).unwrap())
                .unwrap(),
            diagnostic
        );
    }

    #[test]
    fn envelopes_round_trip_success_failure_and_optional_result() {
        let success = Envelope::success(vec![1_u32, 2, 3]);
        let json = serde_json::to_string(&success).unwrap();
        assert_eq!(
            serde_json::from_str::<Envelope<Vec<u32>>>(&json).unwrap(),
            success
        );

        let failure: Envelope<Vec<u32>> = Envelope::failure(vec![Diagnostic::new(
            "kernel.test.failure",
            Severity::Error,
            "failed",
        )]);
        let json = serde_json::to_string(&failure).unwrap();
        assert_eq!(
            serde_json::from_str::<Envelope<Vec<u32>>>(&json).unwrap(),
            failure
        );
    }

    #[test]
    fn envelopes_reject_unsupported_versions_and_malformed_required_fields() {
        for version in [0, 2, u32::MAX] {
            let json = format!(r#"{{"schema_version":{version},"result":null,"diagnostics":[]}}"#);
            assert!(serde_json::from_str::<Envelope<()>>(&json).is_err());
        }

        assert!(
            serde_json::from_str::<Envelope<()>>(r#"{"schema_version":1,"result":null}"#).is_err()
        );
        assert!(serde_json::from_str::<Envelope<()>>(
            r#"{"schema_version":1,"result":null,"diagnostics":"not-an-array"}"#
        )
        .is_err());
        assert!(serde_json::from_str::<Envelope<u32>>(
            r#"{"schema_version":1,"result":"not-a-number","diagnostics":[]}"#
        )
        .is_err());
    }
}
