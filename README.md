# Jabr Kernel

Jabr Kernel is the small, domain-neutral Rust foundation shared by the Jabr engineering worksheet ecosystem. It provides reusable infrastructure only. It is not a calculation engine, worksheet state manager, scheduler, persistence layer, or application orchestrator.

## Repository responsibilities

| Component | Responsibility |
| --- | --- |
| **Jabr Kernel** | Generic identifiers, opaque ports, dependency-graph primitives, source spans/locations, structured diagnostics, and versioned envelopes |
| **Jabr Math** | Mathematical values and quantities, units, expression syntax, parsing, AST, evaluation, mathematical types, scopes, and mathematical dependency semantics |
| **Jabr Table** | Table document model, cell identity/content, formula text, and table-specific operations |
| **Jabr Draw / Diagram** | Diagram document model, geometry, drawing/editing operations, and rendering semantics |
| **Jabr Engine** | Worksheet state, cross-engine graph composition, revisions, snapshots, result lifecycle, invalidation, recalculation scheduling, and coordination |
| **Jabr UI** | Unified document experience, worksheet composition, editing interactions, presentation, and UI-side document integration |

Math, Table, and Draw are independently developable and testable domain engines. Jabr Engine composes their capabilities and owns worksheet-level lifecycle. The UI consumes the Engine's application-facing contract; it must not coordinate the domain engines directly.

## Dependency direction

```text
                 Jabr Kernel
                /     |     \
               v      v      v
             Math    Table   Draw
                \     |     /
                 v    v    v
                 Jabr Engine
                      |
                      v
                    Jabr UI
```

Kernel must not depend on Math, Table, Draw, Engine, or UI. Do not introduce a monorepo or shared cross-repository Cargo workspace as part of Kernel 0.1.

## Package and public API design

Kernel is intentionally **one Rust crate: `jabr-kernel`**. Its public contract is the documented, externally tested public surface of that crate. Rust module visibility separates public types from private implementation; a second `jabr-kernel-api` crate is not justified by the current requirements.

Other Jabr repositories depend on `jabr-kernel` and use only its documented public API. The API is kept stable through external-consumer tests, explicit compatibility notes, and review of wire-format changes. Extract a separate API crate only if independent implementations, independent release cadence, or a concrete dependency-isolation requirement emerges.

## Kernel 0.1 public scope

Kernel owns exactly six infrastructure areas:

1. **Identifiers** — stable, domain-neutral identity and documented validity/serialization rules.
2. **Ports** — generic endpoint identity associated with a Kernel identifier; port meaning remains opaque to Kernel.
3. **Dependency graph** — generic port-aware graph representation and graph algorithms, including prerequisite-first ordering and deterministic dependent-closure queries for explicit roots. These operations do not decide worksheet dirty-state policy or schedule execution.
4. **Source spans and locations** — source-independent ranges and location primitives with explicit offset and line/column conventions. `Span` uses UTF-8 byte offsets and a half-open range. `Location` uses validated one-based line and Unicode-scalar column coordinates.
5. **Diagnostics** — generic structured diagnostics with stable owner-namespaced machine-readable codes.
6. **Versioned envelopes** — a generic transport envelope with a fixed schema version and operation result/diagnostics. The version is read-only; deserialization rejects unsupported versions.

Kernel does not interpret domain payloads or define engine semantics.

## Explicit exclusions

The following are not Kernel responsibilities:

- Variable/assignment state, mathematical scopes, worksheet namespaces, revision counters, state lifecycle, change tracking, snapshots, or state history.
- Calculation-result storage, invalidation, or caching.
- Cross-engine graph construction or dependency interpretation.
- Evaluation scheduling, recalculation policy, orchestration, or change publication. Generic graph closure and ordering operations do not themselves schedule or execute worksheet work.
- Mathematical values, units, parsing, AST, evaluation, or mathematical diagnostics.
- Table models/cells/operations or diagram models/geometry/rendering.
- Worksheet/document ownership, BlockSuite/Yjs integration, UI behavior, or persistence policy.

Mathematical variable semantics belong to Math. Persistent worksheet state and calculation lifecycle belong to Jabr Engine. Kernel may provide generic primitives but does not own variable state.

## Development and validation

See [development.md](./development.md) for the authoritative scope, audit record, milestones, and integration gates. See [API-COMPATIBILITY.md](./API-COMPATIBILITY.md) for the reviewed public and serialization contract. See [docs/KERNEL-0.1-COMPLETION.md](./docs/KERNEL-0.1-COMPLETION.md) for the final hardening scope and release checklist.

Run the repository checks locally:

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features
```

The CI workflow additionally checks the declared Rust 1.75 MSRV. Keep the crate small, independently buildable, and domain-neutral. Do not add dependencies on Math, Table, Draw, Engine, BlockSuite, Yjs, or the UI.
