# Jabr Kernel

Jabr Kernel is the small, domain-neutral Rust foundation shared by the Jabr engineering worksheet ecosystem. It provides generic infrastructure only. It is not a calculation engine, worksheet engine, or application orchestrator.

## Repository responsibilities

| Component | Responsibility |
| --- | --- |
| **jabr-kernel-api** | Versioned public contract types consumed directly by Jabr engine crates |
| **jabr-kernel** | Kernel implementation of the public API; preserves compatibility re-exports for existing consumers |
| **Jabr-math** | Mathematical values and quantities, units, expression syntax, parsing, AST, evaluation, mathematical types, scopes, and mathematical dependency semantics |
| **Jabr-table** | Table document model, cell identity/content, formula text, and table-specific operations |
| **Jabr-diagram** | Diagram document model, geometry, drawing/editing operations, and rendering semantics |
| **Orchestrator** | Cross-engine graph construction from engine-provided information, evaluation scheduling, result storage, caching, and change publication |
| **Jabr-UI** | Unified document experience, worksheet composition, editing interactions, presentation, and UI-side document integration |

Math, Table, and Diagram are independent domain engines. Each remains independently developable and testable. The Orchestrator composes their capabilities. The UI consumes the Orchestrator's application-facing contract; it must not coordinate the engines directly.

## Dependency direction

```text
                 jabr-kernel-api
                 ^      ^      ^
                 |      |      |
             jabr-kernel (implementation)
                 |
          compatibility re-exports
```

Math, Table, Diagram, Orchestrator, and UI depend on `jabr-kernel-api` whenever they need Kernel-owned types. The implementation crate `jabr-kernel` depends on the API crate; the API crate never depends on the implementation. The API crate is a local package within this repository, not a cross-repository workspace.

## Kernel 0.1 public scope

The `jabr-kernel-api` crate owns the public definitions and minimal generic behavior for the infrastructure contract; `jabr-kernel` is the Kernel implementation facade and preserves the established crate paths through re-exports:

1. **Identifiers** — stable, domain-neutral identity and documented validity/serialization rules.
2. **Ports** — generic endpoint identity associated with a Kernel identifier; port meaning remains opaque to Kernel.
3. **Dependency graph** — generic graph representation and data-structure operations. A generic topological-order algorithm may report prerequisite-first ordering, but it is not an application scheduler.
4. **Source spans and locations** — source-independent ranges and location primitives with explicit offset and line/column conventions. The current implementation has a UTF-8 byte-based, half-open `Span`. `Location` is implemented with validated one-based line and Unicode-scalar column coordinates. Its fields are private; construct it with `Location::new` and read them with `line()` and `column()`. `Span` remains the UTF-8 byte-based, half-open range.
5. **Diagnostics** — generic structured diagnostics with stable owner-namespaced machine-readable codes.
6. **Versioned envelopes** — a generic transport envelope with an explicit schema version and operation result/diagnostics. Serde deserialization accepts only the current schema version; migrations require an explicit adapter.
7. **Shared state, revisions, and snapshots** — `State<T>` stores opaque values by `NodeId`, `Revision` tracks committed changes with checked monotonic increments, and `Snapshot<T>` captures an immutable point-in-time copy. The approved and implemented contract is documented in [STATE-REVISION-SNAPSHOT-API.md](./docs/STATE-REVISION-SNAPSHOT-API.md).

Kernel does not interpret domain payloads or define engine semantics.

## Explicit exclusions

The following are not Kernel responsibilities:

- Mathematical variable semantics, scopes, namespaces, and mathematical state interpretation.
- Calculation-result storage, invalidation, or caching.
- Cross-engine graph construction or dependency interpretation.
- Evaluation scheduling, recalculation policy, orchestration, or change publication.
- Mathematical values, units, parsing, AST, evaluation, or mathematical diagnostics.
- Table models/cells/operations or diagram models/geometry/rendering.
- Worksheet/document ownership, BlockSuite/Yjs integration, UI behavior, or persistence policy.

These responsibilities belong to the relevant engine, Orchestrator, or UI. They must not be added to Kernel as implementation conveniences.

## M0 and M1

The M0 audit compared the restructuring against the pre-change source, verified the retained public APIs against their implementations, and removed the out-of-scope `revision`, `snapshot`, and `result_store` modules and exports. Their code and tests were not migrated elsewhere; the removed behavior inventory is recorded in [development.md](./development.md).

The three M0 contract decisions are recorded in the development plan: `Location` uses one-based line/column coordinates with Unicode-scalar columns; envelope deserialization rejects schema versions other than the current version; and `DependencyGraph::topological_order` remains a generic graph algorithm, never an Orchestrator scheduler. M1 implements the two previously outstanding API behaviors; the M0 technical audit is recorded separately from formal milestone acceptance. Envelope deserialization rejects both older and future unsupported schema versions.

## Development and validation

See [development.md](./development.md) for the historical M0 audit record, milestones, and integration gates. The approved shared-state contract is implemented in this branch and documented in [STATE-REVISION-SNAPSHOT-API.md](./docs/STATE-REVISION-SNAPSHOT-API.md). CI validates both the root facade and the standalone API crate. See [API-COMPATIBILITY.md](./API-COMPATIBILITY.md) for the reviewed Kernel 0.1 public and serialization contract, and [crates/jabr-kernel-api/README.md](./crates/jabr-kernel-api/README.md) for direct engine-consumer guidance.

Run the repository checks locally:

```bash
cargo fmt --all -- --check
cargo fmt --manifest-path crates/jabr-kernel-api/Cargo.toml -- --check
cargo test --all-targets
cargo test --all-targets --all-features
cargo test --manifest-path crates/jabr-kernel-api/Cargo.toml --all-targets
cargo test --manifest-path crates/jabr-kernel-api/Cargo.toml --all-targets --features serde
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --manifest-path crates/jabr-kernel-api/Cargo.toml --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features
cargo doc --manifest-path crates/jabr-kernel-api/Cargo.toml --no-deps --all-features
```

Keep the crate small, independently buildable, and domain-neutral. Do not add dependencies on Math, Table, Diagram, Orchestrator, BlockSuite, Yjs, or the UI.
