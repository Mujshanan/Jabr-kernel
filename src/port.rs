//! Generic node port identity; port meaning belongs to the consuming engine.

use crate::NodeId;

/// An indexed endpoint on a generic graph node.
///
/// Kernel treats the index as opaque. Engine defines whether a port represents
/// an input, output, table range, or another domain-specific endpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Port {
    /// Node that owns this port.
    pub node_id: NodeId,
    /// Opaque zero-based port index within the node.
    pub index: u32,
}

impl Port {
    /// Creates a port identity.
    pub const fn new(node_id: NodeId, index: u32) -> Self {
        Self { node_id, index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeIdAllocator;

    #[test]
    fn port_identity_is_node_and_index() {
        let node = NodeIdAllocator::default().allocate().unwrap();
        assert_eq!(Port::new(node, 0), Port { node_id: node, index: 0 });
        assert_ne!(Port::new(node, 0), Port::new(node, 1));
    }
}
