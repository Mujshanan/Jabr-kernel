# Jabr Kernel

Jabr Kernel is the small, domain-neutral Rust foundation shared by the Jabr engineering worksheet ecosystem. It provides generic infrastructure only. It is not a mathematical engine, worksheet engine, calculation-result store, or application orchestrator.

## Repository responsibilities

| Component | Responsibility |
| --- | --- |
| **Jabr-kernel** | Generic identifiers and ports, dependency-graph primitives/algorithms, source spans and locations, structured diagnostics, and versioned envelopes |
| **Jabr-math** | Mathematical syntax, parsing, AST, values, quantities, units, dimensional rules, expression dependencies, and evaluation |
| **Jabr-table** | Table document model, stable cell/row/column identity, authored cell content, and table-specific operations |
| **Jabr-diagram** | Renderer-independent figure model, geometry, parametric updates, and SVG rendering/export |
| **Jabr Engine (planned)** | Worksheet state and calculation records, domain-to-node mapping, cross-engine graph construction, dependency resolution, revisions/generations, snapshots, result storage/cache, scheduling, and recalculation lifecycle |
| **Jabr-UI** | Unified BlockSuite worksheet experience, authored document persistence/history, editing interactions, focus/keyboard routing, presentation, and transient rendering |

Math, Table, and Diagram are independent domain engines. Each remains independently developable and testable. The planned Jabr Engine composes their capabilities. The UI consumes the Engine's application-facing contract; it must not coordinate the engines directly.

## Dependency direction

```text
                 jabr-kernel-api
                /       |       \
               v        v        v
             Math     Table    Diagram
                \       |       /
                 v      v      v
                  Jabr Engine
                       |
                       v
                     Jabr-UI

jabr-kernel implements the generic jabr-kernel-api contract.
```

Kernel must not depend on Math, Table, Diagram, Engine, or UI. Domain engines and Engine may consume the public generic Kernel contract; they must not depend on Kernel implementation internals. Do not introduce a monorepo or shared cross-repository Cargo workspace as part of Kernel 0.1.

## Kernel 0.1 public scope

Kernel owns exactly these generic infrastructure areas:

1. **Identifiers** — stable, domain-neutral identity and documented validity/serialization rules.
2. **Ports** — generic endpoint identity associated with a Kernel identifier; port meaning remains opaque to Kernel.
3. **Dependency graph** — generic port-aware graph representation and algorithms, including prerequisite-first ordering and deterministic dependent-closure queries for explicit roots. These operations do not decide worksheet dirty-state policy or schedule execution.
4. **Source spans and locations** — source-independent ranges and location primitives with explicit offset and line/column conventions.
5. **Diagnostics** — generic structured diagnostics with stable owner-namespaced machine-readable codes.
6. **Versioned envelopes** — generic transport envelope with explicit schema version and operation result/diagnostics. Domain payloads remain opaque to Kernel.

Kernel does not interpret domain payloads or define engine semantics.

## Explicit exclusions

The following are not Kernel responsibilities:

- Variable/assignment state, scopes, namespaces, revisions, state lifecycle, change tracking, snapshots, or state history.
- Worksheet/document/node revisions or calculation generations.
- Calculation-result storage, invalidation, cache validity, or publication.
- Cross-engine graph construction, dependency interpretation, or worksheet binding resolution.
- Evaluation scheduling, recalculation policy, orchestration, or change publication.
- Mathematical values, units, parsing, AST, evaluation, or mathematical diagnostics.
- Table models/cells/operations or diagram models/geometry/rendering.
- Worksheet/document ownership, BlockSuite/Yjs integration, UI behavior, or authored persistence/history.

These responsibilities belong to Math, Table, Diagram, Jabr Engine, or Jabr-UI as appropriate. They must not be added to Kernel as implementation conveniences.

## Development and validation

See [Math–Kernel Integration Specification](docs/specs/math-kernel-integration-spec.md) for the public Kernel/Math boundary and Engine-owned lifecycle. See [API-COMPATIBILITY.md](./API-COMPATIBILITY.md) for the reviewed Kernel public and serialization contract.

Run the repository checks locally:

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features
```

Keep the crate small, independently buildable, and domain-neutral. Do not add dependencies on Math, Table, Diagram, Engine, BlockSuite, Yjs, or the UI.
