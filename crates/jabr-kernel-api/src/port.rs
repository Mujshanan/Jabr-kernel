//! Generic node port identity; endpoint meaning belongs to the consuming engine.
use crate::NodeId;
/// Indexed endpoint on a generic graph node. The index is opaque and zero-based.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Port {
    /// Node that owns this port.
    pub node_id: NodeId,
    /// Opaque zero-based endpoint index.
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
    #[test]
    fn index_is_part_of_identity() {
        let n = NodeId::new(1).unwrap();
        assert_ne!(Port::new(n, 0), Port::new(n, 1));
    }
}
