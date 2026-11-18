//! Domain-neutral shared infrastructure for Jabr.
//! Mathematical values, units, parsing, evaluation, and worksheet orchestration
//! belong to their owning engine repositories.

pub mod diagnostic;
pub mod envelope;
pub mod graph;
pub mod id;
pub mod port;
pub mod result_store;
pub mod revision;
pub mod snapshot;
pub mod span;

pub use diagnostic::{Diagnostic, Severity};
pub use envelope::{Envelope, ENVELOPE_SCHEMA_VERSION};
pub use graph::{DependencyGraph, GraphError};
pub use id::{IdAllocationError, NodeId, NodeIdAllocator};
pub use port::Port;
pub use result_store::ResultStore;
pub use revision::{Revision, RevisionError};
pub use snapshot::Snapshot;
pub use span::{Location, Span, SpanError};
