//! Domain-neutral shared infrastructure for Jabr.
//! Mathematical values, units, parsing, and evaluation belong to Jabr Math.

pub mod diagnostic;
pub mod envelope;
pub mod graph;
pub mod id;
pub mod snapshot;
pub mod span;
pub mod state;

pub use diagnostic::{Diagnostic, Severity};
pub use envelope::{Envelope, ENVELOPE_SCHEMA_VERSION};
pub use graph::{DependencyGraph, GraphError};
pub use id::{IdAllocationError, NodeId, NodeIdAllocator};
pub use snapshot::{SnapshotEntry, StateSnapshot};
pub use span::{Location, Span, SpanError};
pub use state::{Change, ChangeKind, ChangeSet, StateError, StateStore};
