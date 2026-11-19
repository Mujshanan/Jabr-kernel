# Jabr Kernel 0.1 API and compatibility note

**Reviewed against:** `jabr-kernel-api` 0.1.0 and `jabr-kernel` compatibility re-exports  
**Review date:** 2026-11-18  
**Public contract crate:** `jabr-kernel-api` 0.1.0  
**Implementation crate:** `jabr-kernel` 0.1.0  
**MSRV:** Rust 1.75

This note records the public contract implemented by Kernel 0.1. It is a compatibility reference for engine consumers, not a promise that excluded engine or application behavior is supplied by Kernel.

## Public surface

Kernel exposes six domain-neutral infrastructure areas:

- **Identifiers:** `NodeId`, `NodeIdAllocator`, `IdAllocationError`, and `MAX_NODE_ID`. IDs are non-zero integers in the JavaScript-safe range `1..=2^53-1`. Clones of one allocator share an atomic sequence; separately created allocators do not share a namespace.
- **Ports:** `Port` identifies a node and an opaque zero-based `u32` index. Kernel assigns no input/output or domain meaning to the index.
- **Dependency graph:** `DependencyGraph<N>` and `GraphError<N>` provide generic graph mutation and deterministic prerequisite-first topological ordering. A cycle error reports all unresolved nodes, including nodes blocked downstream of a cycle. This is a graph primitive, not an execution scheduler.
- **Source coordinates:** `Span` is a UTF-8 byte-based half-open range (start inclusive, end exclusive). `Location` is one-based for both line and column; column counts Unicode scalar values, not bytes, UTF-16 code units, graphemes, or visual tab width. Engines derive locations from their own source text.
- **Diagnostics:** `Diagnostic` and `Severity` provide a generic structured diagnostic. Machine-readable codes are owned by the emitting crate and should be namespaced (for example, `kernel.*` or `math.*`). Human-readable messages are not identifiers and must not be parsed.
- **Versioned envelopes:** `Envelope<T>` and `ENVELOPE_SCHEMA_VERSION` define a generic JSON transport container. The current schema version is `1`; Serde deserialization rejects every other version.

The `jabr-kernel-api` crate root re-exports the principal public types. Its module paths remain public. `jabr-kernel` re-exports the same types and module paths for source compatibility; new engine crates should depend directly on `jabr-kernel-api`.

## Serialization contract

Serialization support is optional and enabled with the `serde` feature. Without that feature, the core types remain usable without a runtime Serde dependency.

With `serde` enabled:

- `NodeId` serializes as its integer value and validates the non-zero JavaScript-safe range when deserialized.
- `Port`, `Diagnostic`, and `Severity` use their derived Serde representation; severity values use snake_case strings.
- `Span` and `Location` validate their invariants when deserialized.
- `Envelope<T>` serializes its private schema version, optional `result`, and `diagnostics`; deserialization accepts only schema version `1`. Consumers inspect the version through `schema_version()`.

Envelope schema versioning is separate from diagnostic-code ownership/versioning. A consumer that needs another envelope schema must use an explicit version-specific adapter or migration; Kernel does not silently accept older or future versions.

## Compatibility cautions

- `Envelope::schema_version` is private. Constructors set it to the supported version, and consumers can inspect it through `schema_version()`. This prevents callers from constructing or mutating an unsupported version through the public API.
- Allocator-generated IDs are only unique within the allocator/document namespace chosen by the owning application. Independent allocators can generate equal numeric IDs.
- Graph ordering is deterministic for a given graph and node ordering, but it does not prescribe evaluation, scheduling, parallelism, invalidation, or result storage.
- The currently implemented API does not yet include State, Revision, or Snapshot. Their proposed ownership and exact interfaces are documented in [STATE-REVISION-SNAPSHOT-API.md](./docs/STATE-REVISION-SNAPSHOT-API.md); that proposal is not implemented or accepted yet. Mathematical scopes, dependency interpretation, recalculation, result stores, and caching remain outside Kernel.

## Dependency and toolchain boundary

Kernel 0.1 is a standalone Rust 2021 package with Rust 1.75 as its declared minimum. Its only optional runtime dependency is Serde; `serde_json` is development-only. The implementation crate has a local path dependency on `jabr-kernel-api`; the API crate has no dependency on the implementation or on Math, Table, Diagram, Orchestrator, or UI.

## Consumer guidance

Consumers should depend on the published Kernel crate contract only for generic infrastructure. Domain meanings, source mapping, mathematical semantics, table/diagram models, cross-engine scheduling, and application coordination remain with their owning repositories. Treat changes to these documented public types, wire representations, validation rules, and graph behavior as compatibility-sensitive and review them explicitly before release.
