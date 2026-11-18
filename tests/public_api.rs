use jabr_kernel::{
    DependencyGraph, Envelope, NodeIdAllocator, Port, Revision, ResultStore, Snapshot,
};

#[test]
fn public_generic_contract_composes_without_domain_types() {
    let node = NodeIdAllocator::default().allocate().unwrap();
    let port = Port::new(node, 0);

    let mut graph = DependencyGraph::default();
    graph.add_node(node);
    assert_eq!(graph.topological_order().unwrap(), vec![node]);

    let mut results = ResultStore::new();
    results.insert(node, String::from("opaque engine payload"));
    assert_eq!(results.get(node).map(String::as_str), Some("opaque engine payload"));

    let snapshot = Snapshot::new(Revision(12), results.len());
    let response: Envelope<Snapshot<usize>> = Envelope::success(snapshot);
    let snapshot = response.result.unwrap();

    assert_eq!(snapshot.revision, Revision(12));
    assert_eq!(snapshot.payload, 1);
    assert_eq!(port.node_id, node);
}
