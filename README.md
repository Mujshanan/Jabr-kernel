# Jabr Kernel

Jabr Kernel is the small, domain-neutral Rust foundation shared by the Jabr engineering worksheet ecosystem. It provides generic infrastructure only. It is not a calculation engine, worksheet engine, or application orchestrator.

## Repository responsibilities

| Component | Responsibility |
| --- | --- |
| **Jabr-kernel** | Generic identifiers, opaque ports, dependency-graph primitives, source spans/locations, structured diagnostics, and versioned envelopes |
| **Jabr-math** | Mathematical values and quantities, units, expression syntax, parsing, AST, evaluation, mathematical types, scopes, and mathematical dependency semantics |
| **Jabr-table** | Table document model, cell identity/content, formula text, and table-specific operations |
| **Jabr-diagram** | Diagram document model, geometry, drawing/editing operations, and rendering semantics |
| **Orchestrator** | Cross-engine graph construction from engine-provided information, evaluation scheduling, result storage, caching, and change publication |
| **Jabr-UI** | Unified document experience, worksheet composition, editing interactions, presentation, and UI-side document integration |

Math, Table, and Diagram are independent domain engines. Each remains independently developable and testable. The Orchestrator composes their capabilities. The UI consumes the Orchestrator's application-facing contract; it must not coordinate the engines directly.

## Dependency direction

```text
                 Jabr-kernel
                /     |     \
               v      v      v
             Math    Table  Diagram
                \     |     /
                 v    v    v
                  Orchestrator
                       |
                       v
                     Jabr-UI
```

Kernel must not depend on Math, Table, Diagram, Orchestrator, or UI. Do not introduce a monorepo or shared cross-repository Cargo workspace as part of Kernel 0.1.

## Kernel 0.1 public scope

Kernel owns exactly six infrastructure areas:

1. **Identifiers** — stable, domain-neutral identity and documented validity/serialization rules.
2. **Ports** — generic endpoint identity associated with a Kernel identifier; port meaning remains opaque to Kernel.
3. **Dependency graph** — generic graph representation and data-structure operations. A generic topological-order algorithm may report prerequisite-first ordering, but it is not an application scheduler.
4. **Source spans and locations** — source-independent ranges and location primitives with explicit offset and line/column conventions. The current implementation has a UTF-8 byte-based, half-open `Span`. `Location` is implemented with validated one-based line and Unicode-scalar column coordinates. Its fields are private; construct it with `Location::new` and read them with `line()` and `column()`. `Span` remains the UTF-8 byte-based, half-open range.
5. **Diagnostics** — generic structured diagnostics with stable owner-namespaced machine-readable codes.
6. **Versioned envelopes** — a generic transport envelope with an explicit schema version and operation result/diagnostics. Serde deserialization accepts only the current schema version; migrations require an explicit adapter.

Kernel does not interpret domain payloads or define engine semantics.

## Explicit exclusions

The following are not Kernel responsibilities:

- Variable/assignment state, scopes, namespaces, revision counters, state lifecycle, change tracking, snapshots, or state history.
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

See [development.md](./development.md) for the authoritative scope, M0 audit record, milestones, and integration gates.

Run the repository checks locally:

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features
```

Keep the crate small, independently buildable, and domain-neutral. Do not add dependencies on Math, Table, Diagram, Orchestrator, BlockSuite, Yjs, or the UI.
