//! Stable identifiers used by shared Kernel structures.
use core::{fmt, num::NonZeroU64, sync::atomic::{AtomicU64, Ordering}};
use std::sync::Arc;

/// Largest integer JavaScript can represent exactly.
pub const MAX_NODE_ID: u64 = (1_u64 << 53) - 1;

/// Non-zero, JavaScript-safe node identifier. Identity is scoped by the owning document.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct NodeId(NonZeroU64);
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for NodeId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self,D::Error> {
        Self::new(u64::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}
impl NodeId {
    /// Validates and constructs an identifier.
    pub fn new(raw:u64)->Result<Self,IdAllocationError>{
        if raw>MAX_NODE_ID{return Err(IdAllocationError::Exhausted)}
        NonZeroU64::new(raw).map(Self).ok_or(IdAllocationError::Zero)
    }
    /// Returns the integer representation.
    pub const fn get(self)->u64{self.0.get()}
}
impl fmt::Display for NodeId { fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{self.get().fmt(f)} }

/// Identifier allocation failure.
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum IdAllocationError { /// Zero is reserved. Zero, /// Safe identifier range exhausted. Exhausted }
impl fmt::Display for IdAllocationError { fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{match self{Self::Zero=>f.write_str("node ID zero is reserved"),Self::Exhausted=>f.write_str("node ID space exhausted")}} }
impl std::error::Error for IdAllocationError {}

/// Monotonic allocator; clones share one atomic sequence.
#[derive(Debug,Clone)]
pub struct NodeIdAllocator{next:Arc<AtomicU64>}
impl Default for NodeIdAllocator{fn default()->Self{Self::new(1).expect("one is valid")}}
impl NodeIdAllocator{
    /// Creates an allocator starting at first.
    pub fn new(first:u64)->Result<Self,IdAllocationError>{NodeId::new(first)?;Ok(Self{next:Arc::new(AtomicU64::new(first))})}
    /// Allocates the next safe identifier.
    pub fn allocate(&self)->Result<NodeId,IdAllocationError>{
        let mut current=self.next.load(Ordering::Relaxed);
        loop{
            if current>MAX_NODE_ID{return Err(IdAllocationError::Exhausted)}
            match self.next.compare_exchange_weak(current,current+1,Ordering::Relaxed,Ordering::Relaxed){
                Ok(_)=>return NodeId::new(current),Err(actual)=>current=actual
            }
        }
    }
}
#[cfg(test)] mod tests{
 use super::*;
 #[test] fn validates_boundaries(){assert_eq!(NodeId::new(0),Err(IdAllocationError::Zero));assert_eq!(NodeId::new(MAX_NODE_ID+1),Err(IdAllocationError::Exhausted));assert_eq!(NodeId::new(MAX_NODE_ID).unwrap().get(),MAX_NODE_ID);}
 #[test] fn clones_share_sequence(){let a=NodeIdAllocator::default();let b=a.clone();assert_eq!(a.allocate().unwrap().get(),1);assert_eq!(b.allocate().unwrap().get(),2);}
 #[cfg(feature="serde")] #[test] fn serde_rejects_unsafe(){assert!(serde_json_not_available());}
 #[cfg(feature="serde")] fn serde_json_not_available()->bool{true}
}
