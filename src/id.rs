//! Stable identifiers used by shared Kernel structures.
use core::{
    fmt,
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};
use std::sync::Arc;

/// Largest integer that JavaScript can represent exactly.
pub const MAX_NODE_ID: u64 = (1_u64 << 53) - 1;

/// A non-zero, JavaScript-safe node identifier.
///
/// IDs are scoped by the allocator/document that creates them; independently
/// created allocators may produce the same numeric IDs. Persisted documents
/// must preserve their IDs, and Engine is responsible for document scoping.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct NodeId(NonZeroU64);

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for NodeId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = <u64 as serde::Deserialize>::deserialize(d)?;
        Self::new(raw).map_err(serde::de::Error::custom)
    }
}
impl NodeId {
    /// Validates and constructs an identifier.
    pub fn new(raw: u64) -> Result<Self, IdAllocationError> {
        if raw > MAX_NODE_ID {
            return Err(IdAllocationError::Exhausted);
        }
        NonZeroU64::new(raw)
            .map(Self)
            .ok_or(IdAllocationError::Zero)
    }
    /// Returns the integer representation.
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}
impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

/// Identifier allocation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdAllocationError {
    /// Zero is reserved.
    Zero,
    /// The JavaScript-safe identifier range is exhausted.
    Exhausted,
}
impl fmt::Display for IdAllocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => f.write_str("node ID zero is reserved"),
            Self::Exhausted => f.write_str("node ID space exhausted"),
        }
    }
}
impl std::error::Error for IdAllocationError {}

/// Monotonic allocator; cloned allocators share one atomic sequence.
///
/// Separate allocator instances have independent sequences. Use one allocator
/// per identity namespace, or let Engine provide the namespace boundary.
#[derive(Debug, Clone)]
pub struct NodeIdAllocator {
    next: Arc<AtomicU64>,
}
impl Default for NodeIdAllocator {
    fn default() -> Self {
        Self::new(1).expect("one is valid")
    }
}
impl NodeIdAllocator {
    /// Creates an allocator starting at first.
    pub fn new(first: u64) -> Result<Self, IdAllocationError> {
        NodeId::new(first)?;
        Ok(Self {
            next: Arc::new(AtomicU64::new(first)),
        })
    }
    /// Allocates the next safe identifier.
    pub fn allocate(&self) -> Result<NodeId, IdAllocationError> {
        let raw = self
            .next
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                (n <= MAX_NODE_ID).then_some(n + 1)
            })
            .map_err(|_| IdAllocationError::Exhausted)?;
        NodeId::new(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_range() {
        assert_eq!(NodeId::new(0), Err(IdAllocationError::Zero));
        assert_eq!(
            NodeId::new(MAX_NODE_ID + 1),
            Err(IdAllocationError::Exhausted)
        );
    }
    #[test]
    fn clones_share_allocator_sequence() {
        let a = NodeIdAllocator::default();
        let b = a.clone();
        assert_eq!(a.allocate().unwrap().get(), 1);
        assert_eq!(b.allocate().unwrap().get(), 2);
    }
    #[cfg(feature = "serde")]
    #[test]
    fn serde_rejects_unsafe_ids() {
        assert!(serde_json::from_str::<NodeId>("9007199254740992").is_err());
    }
}
