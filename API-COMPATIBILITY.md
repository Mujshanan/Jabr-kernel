# Jabr Kernel 0.1 API and compatibility note

**Reviewed against:** `feat/kernel-core-implementation-0.1`  
**Review date:** 2026-11-18  
**Crate:** `jabr-kernel` 0.1.0  
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

The crate root re-exports the principal public types. Module paths remain public as well.

## Serialization contract

Serialization support is optional and enabled with the `serde` feature. Without that feature, the core types remain usable without a runtime Serde dependency.

With `serde` enabled:

- `NodeId` serializes as its integer value and validates the non-zero JavaScript-safe range when deserialized.
- `Port`, `Diagnostic`, and `Severity` use their derived Serde representation; severity values use snake_case strings.
- `Span` and `Location` validate their invariants when deserialized.
- `Envelope<T>` serializes its `schema_version`, optional `result`, and `diagnostics`; deserialization accepts only schema version `1`.

Envelope schema versioning is separate from diagnostic-code ownership/versioning. A consumer that needs another envelope schema must use an explicit version-specific adapter or migration; Kernel does not silently accept older or future versions.

## Compatibility cautions

- `Envelope::schema_version` is currently a public field. Constructors set it to the supported version, but callers can manually construct or mutate an envelope before serialization. The current serializer does not independently validate that field. Consumers should use `Envelope::success` or `Envelope::failure` and must not change the schema version. Encapsulating this field would be a public API change and is intentionally not made in this 0.1 review.
- Allocator-generated IDs are only unique within the allocator/document namespace chosen by the owning application. Independent allocators can generate equal numeric IDs.
- Graph ordering is deterministic for a given graph and node ordering, but it does not prescribe evaluation, scheduling, parallelism, invalidation, or result storage.
- Kernel does not own variables, scopes, revisions, snapshots, result stores, caching, recalculation, cross-engine dependency interpretation, or change publication.

## Dependency and toolchain boundary

Kernel 0.1 is a standalone Rust 2021 package with Rust 1.75 as its declared minimum. Its only optional runtime dependency is Serde; `serde_json` is development-only. It has no path dependency or workspace coupling to Math, Table, Diagram, Orchestrator, or UI.

## Consumer guidance

Consumers should depend on the published Kernel crate contract only for generic infrastructure. Domain meanings, source mapping, mathematical semantics, table/diagram models, cross-engine scheduling, and application coordination remain with their owning repositories. Treat changes to these documented public types, wire representations, validation rules, and graph behavior as compatibility-sensitive and review them explicitly before release.
