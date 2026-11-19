//! Domain-neutral shared infrastructure implementation for Jabr.
//! Public contracts are defined by the independent `jabr-kernel-api` crate.

pub use jabr_kernel_api::{
    diagnostic::{Diagnostic, Severity},
    envelope::{Envelope, ENVELOPE_SCHEMA_VERSION},
    graph::{DependencyGraph, GraphError},
    id::{IdAllocationError, NodeId, NodeIdAllocator, MAX_NODE_ID},
    location::{Location, LocationError},
    port::Port,
    revision::{Revision, RevisionError},
    snapshot::Snapshot,
    span::{Span, SpanError},
    state::State,
};

/// Compatibility module; the public contract is owned by `jabr-kernel-api`.
pub mod diagnostic {
    pub use jabr_kernel_api::diagnostic::*;
}
/// Compatibility module; the public contract is owned by `jabr-kernel-api`.
pub mod envelope {
    pub use jabr_kernel_api::envelope::*;
}
/// Compatibility module; the public contract is owned by `jabr-kernel-api`.
pub mod graph {
    pub use jabr_kernel_api::graph::*;
}
/// Compatibility module; the public contract is owned by `jabr-kernel-api`.
pub mod id {
    pub use jabr_kernel_api::id::*;
}
/// Compatibility module; the public contract is owned by `jabr-kernel-api`.
pub mod location {
    pub use jabr_kernel_api::location::*;
}
/// Compatibility module; the public contract is owned by `jabr-kernel-api`.
pub mod port {
    pub use jabr_kernel_api::port::*;
}
/// Compatibility module; the public contract is owned by `jabr-kernel-api`.
pub mod revision {
    pub use jabr_kernel_api::revision::*;
}
/// Compatibility module; the public contract is owned by `jabr-kernel-api`.
pub mod snapshot {
    pub use jabr_kernel_api::snapshot::*;
}
/// Compatibility module; the public contract is owned by `jabr-kernel-api`.
pub mod span {
    pub use jabr_kernel_api::span::*;
}
/// Compatibility module; the public contract is owned by `jabr-kernel-api`.
pub mod state {
    pub use jabr_kernel_api::state::*;
}
