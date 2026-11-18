# Jabr Kernel

Jabr Kernel is the small, application-independent Rust foundation shared by the Jabr engineering worksheet ecosystem. It provides generic infrastructure; it is not a calculation engine and does not own worksheet, mathematical, table, or diagram semantics.

## Repository responsibilities

| Repository | Responsibility |
| --- | --- |
| [Jabr-kernel](https://github.com/Mujshanan/Jabr-kernel) | Generic NodeId/Port, Revision, Snapshot<T>, ResultStore<V>, dependency-graph primitives, diagnostics and source spans |
| [Jabr-math](https://github.com/Mujshanan/Jabr-math) | Mathematical syntax, parsing, AST, values, quantities, units, dimensional rules and evaluation |
| [Jabr-table](https://github.com/Mujshanan/Jabr-table) | Table structure, stable row/column identities, authored cell text, metadata and table-specific operations |
| [Jabr-diagram](https://github.com/Mujshanan/Jabr-diagram) | Renderer-independent diagram model, geometry and diagram-domain operations |
| Jabr-engine (planned; repository not yet created) | Worksheet node mapping, dependency construction, bindings, evaluation orchestration, scheduling and calculation-result lifecycle |
| [Jabr-UI](https://github.com/Mujshanan/Jabr-UI) | Complete product UI, BlockSuite/Yjs document, MathLive editing, custom blocks and result presentation |

## Dependency direction

```text
Jabr-UI
   |
   v
Jabr-engine
   |------> Jabr-kernel
   |------> Jabr-math
   |------> Jabr-table
   `------> Jabr-diagram (when required)
```

Kernel is a dependency of the engine libraries, never the other way around. Each repository remains independently buildable and testable. Do not introduce a monorepo or shared cross-repository Cargo workspace.

## Kernel API boundary

Kernel should provide only generic, reusable infrastructure:

- NodeId and generic Port identity.
- Revision(u64), a monotonic revision marker.
- Snapshot<T> { revision, payload }, a complete revisioned payload that follows the existing versioned Envelope serialization conventions.
- ResultStore<V>, generic storage keyed by NodeId; Kernel does not inspect or interpret V.
- Generic dependency-graph primitives.
- Generic diagnostics and source spans.

The Engine decides which worksheet objects become graph nodes, how their ports connect, and which Math values are placed in ResultStore<V>. For example, Engine may instantiate ResultStore<MathValue>; Kernel must not depend on or know the Math value type.

Kernel does not own variable names, scopes, assignments, mathematical values, quantities, units, expression parsing, evaluation, table-cell parsing, worksheet recalculation, scheduling, or document undo/redo. BlockSuite/Yjs owns authored document history. Engine owns calculation orchestration.

## Current implementation audit

The current Kernel implementation PR #2 now exposes NodeId, indexed Port, checked Revision, Snapshot<T>, ResultStore<V> keyed by NodeId, generic DependencyGraph, Diagnostic, Span and Envelope<T>. The old generic StateStore<K, V>/StateSnapshot<K, V> surface has been removed from that implementation branch. CI and review are still required; this branch is not a merged release. The older scope/variable-specific state proposal is superseded.

These are audit findings, not claims about the default branch or a merged release. See [development.md](./development.md) for the authoritative development plan and migration sequence.

## Development and validation

Use the Rust toolchain pinned by the repository. Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
```

Keep public APIs documented and generic. Do not add dependencies on Math, Table, Diagram, Engine, BlockSuite, Yjs, or the UI.
