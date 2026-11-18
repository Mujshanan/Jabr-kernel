# Jabr Kernel

Jabr Kernel is the small, domain-neutral Rust foundation shared by the Jabr engineering worksheet ecosystem. It provides reusable infrastructure; it is not a math engine, worksheet engine, or application coordinator.

## System responsibilities

| Component | Responsibility |
| --- | --- |
| **Jabr-kernel** | Generic identifiers, opaque ports, dependency-graph primitives, source spans/locations, structured diagnostics, and versioned envelopes |
| **Jabr-math** | Mathematical values and quantities, units, expression syntax, parsing, AST, evaluation, mathematical types, and mathematical dependency semantics |
| **Jabr-table** | Table document model, cell identity/content, formula text, and table-specific operations |
| **Jabr-diagram** | Diagram document model, geometry, drawing/editing operations, and rendering semantics |
| **Jabr-engine** | Worksheet orchestration: cross-engine graph construction, dependency binding, scheduling, recalculation, result storage, revisions, snapshots, and result lifecycle |
| **Jabr-UI** | Unified document experience, worksheet composition, editing interactions, presentation, and UI-side document integration |

Math, Table, and Diagram remain independently developable and testable. Engine composes their capabilities. The UI consumes Engine's application-facing contract and does not coordinate the domain engines directly.

## Dependency direction

\`\`\`text
                 Jabr-kernel
                /     |     \\
               v      v      v
             Math    Table  Diagram
                \\     |     /
                 v    v    v
                  Engine
                    |
                    v
                  Jabr-UI
\`\`\`

Kernel must not depend on Math, Table, Diagram, Engine, or UI. Do not introduce a monorepo or shared cross-repository Cargo workspace as part of Kernel 0.1.

## Kernel 0.1 public scope

Kernel owns exactly six infrastructure areas:

1. **Identifiers** — stable, domain-neutral node identity and documented validity/serialization rules.
2. **Ports** — generic endpoint identity associated with a Kernel identifier; port meaning remains opaque.
3. **Dependency graph** — generic graph representation and operations, including deterministic prerequisite-first topological ordering. This is a graph algorithm, not a scheduler.
4. **Source spans and locations** — UTF-8 byte-based, half-open spans and validated one-based line/Unicode-scalar-column locations.
5. **Diagnostics** — generic structured diagnostics with stable owner-namespaced machine-readable codes.
6. **Versioned envelopes** — generic transport envelope with a schema version, optional result, and diagnostics. Serde deserialization accepts only the current schema version.

Kernel does not interpret domain payloads or define worksheet, mathematical, table, or diagram semantics.

## Explicit exclusions

Kernel does not own variable or worksheet state, scopes, revisions, snapshots, result storage, invalidation, caching, cross-engine graph construction, dependency interpretation, scheduling, recalculation policy, or change publication. These worksheet-level responsibilities belong to **Engine**. Math owns mathematical parsing and evaluation; Table and Diagram own their respective domain models and operations. UI owns presentation and document interaction.

The former \`Revision\`, \`Snapshot\`, and \`ResultStore\` implementations were intentionally removed from Kernel. They are not part of the Kernel 0.1 API. The historical removal and ownership decision are recorded in [development.md](development.md).

## Development and validation

- [Development record](development.md) — architecture, scope, audit findings, and release gate.
- [API compatibility](API-COMPATIBILITY.md) — public Rust API and Serde compatibility contract.

Run the repository checks locally:

\`\`\`bash
cargo fmt --all -- --check
cargo test --all-targets
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features
\`\`\`

Keep the crate small, independently buildable, and domain-neutral. Do not add dependencies on Math, Table, Diagram, Engine, BlockSuite, Yjs, or UI.
