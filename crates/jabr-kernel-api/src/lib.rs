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
pub mod revision;
pub mod snapshot;
pub mod span;
pub mod state;

pub use diagnostic::{Diagnostic, Severity};
pub use envelope::{Envelope, ENVELOPE_SCHEMA_VERSION};
pub use graph::{DependencyGraph, GraphError};
pub use id::{IdAllocationError, NodeId, NodeIdAllocator, MAX_NODE_ID};
pub use location::{Location, LocationError};
pub use port::Port;
pub use revision::{Revision, RevisionError};
pub use snapshot::Snapshot;
pub use span::{Span, SpanError};
pub use state::State;
