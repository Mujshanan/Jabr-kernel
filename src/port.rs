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
    use crate::NodeId;

    fn id(value: u64) -> NodeId {
        NodeId::new(value).unwrap()
    }

    #[test]
    fn port_identity_and_order_are_node_then_index() {
        let p10 = Port::new(id(1), 0);
        let p11 = Port::new(id(1), 1);
        let p20 = Port::new(id(2), 0);
        assert_eq!(
            p10,
            Port {
                node_id: id(1),
                index: 0
            }
        );
        assert_ne!(p10, p11);
        assert!(p10 < p11);
        assert!(p11 < p20);
        assert_eq!(p20.cmp(&p10), core::cmp::Ordering::Greater);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn port_serde_round_trips() {
        let port = Port::new(id(123), u32::MAX);
        let json = serde_json::to_string(&port).unwrap();
        assert_eq!(serde_json::from_str::<Port>(&json).unwrap(), port);
    }
}
