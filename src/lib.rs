//! Domain-neutral shared infrastructure for Jabr.
//! Mathematical values, units, parsing, evaluation, and worksheet orchestration
//! belong to their owning engine repositories.

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
pub use id::{IdAllocationError, NodeId, NodeIdAllocator};
pub use location::{Location, LocationError};
pub use port::Port;
pub use span::{Span, SpanError};
