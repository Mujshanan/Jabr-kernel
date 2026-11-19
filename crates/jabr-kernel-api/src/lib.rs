//! Versioned, domain-neutral public contracts shared by Jabr engines.
//!
//! This crate defines data types and generic infrastructure contracts only.
//! It does not depend on Jabr Math, worksheet execution, UI, or Kernel internals.

pub mod diagnostic;
pub mod envelope;
pub mod graph;
pub mod id;
pub mod location;
pub mod port;
pub mod span;

pub use diagnostic::{Diagnostic, Severity};
pub use envelope::{Envelope, ENVELOPE_SCHEMA_VERSION};
pub use graph::{DependencyGraph, GraphError};
pub use id::{IdAllocationError, NodeId, NodeIdAllocator, MAX_NODE_ID};
pub use location::{Location, LocationError};
pub use port::Port;
pub use span::{Span, SpanError};
