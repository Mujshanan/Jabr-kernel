# Jabr-kernel development plan

**Architecture baseline:** 2026-11-18  
**Status:** revised ownership contract; implementation alignment and audit in progress.

This file is the authoritative Kernel development plan. It supersedes earlier proposals that made Kernel responsible for variable/scope assignment state or a cross-engine recalculation API.

## 1. System boundaries

- **Kernel** provides generic identity, revision, snapshot, result-storage, graph, diagnostic and source-span primitives.
- **Math** owns mathematical values, quantities, units, parsing, AST, evaluation, and mathematical dependency semantics.
- **Table** owns table structure, stable row/column identity, authored cell text and table metadata. It does not parse cell expressions or store Math evaluation values.
- **Diagram** owns its renderer-independent diagram model, geometry and diagram-specific operations.
- **Engine** (planned as a separate repository; not yet present in the connected repository set) constructs worksheet graph nodes, binds Math/Table/Diagram capabilities, schedules evaluation, and owns calculation-result lifecycle.
- **UI** owns the complete worksheet experience and authored BlockSuite/Yjs document. It persists source and presentation choices, not calculated results.

Dependency direction:

```text
Jabr-UI -> Jabr-engine -> { Jabr-kernel, Jabr-math, Jabr-table, Jabr-diagram }
```

No circular dependencies. No shared cross-repository Cargo workspace or monorepo.

## 2. Kernel's approved minimal API

### 2.1 Identity

- NodeId is the stable generic identity used by Kernel graph and result-store primitives.
- Port is a generic endpoint/port identity. It must not encode worksheet, table, expression, or math semantics.
- Engine decides how authored blocks, table columns, or other calculation units map to nodes and ports.

### 2.2 Revision

Provide Revision(u64) as a strongly typed monotonic counter.

- Revisions increase within one Engine document/session.
- A restore or undo of authored content creates a new revision; it never rewinds the counter.
- Revision exhaustion is an explicit error; do not wrap.
- UI can use revisions to reject stale responses.

Kernel exposes the primitive; Engine owns the revision lifecycle and decides when a calculation/document change advances it.

### 2.3 Snapshot

Provide the agreed generic shape:

```rust
pub struct Snapshot<T> {
    pub revision: Revision,
    pub payload: T,
}
```

A snapshot is a complete state payload for persistence/load and result publication, not an incremental update. Keep it aligned with the existing versioned Envelope conventions so saved state and load-time results share a predictable serialization boundary. Do not introduce a competing wire format or duplicate schema-version fields without an explicit API decision.

### 2.4 ResultStore

Provide ResultStore<V> keyed by NodeId.

- Kernel stores and retrieves opaque values only.
- Kernel does not define MathValue, Quantity, units, errors-as-values, variable names, scopes, or evaluation behavior.
- Engine chooses the value type and owns when results are inserted, replaced, invalidated, or removed.
- Table never depends on or sees this store.
- Keep any change notification contract generic and separate from complete snapshots.

Do not generalize the public API into arbitrary variable/scope transactions unless a demonstrated non-domain-specific use case requires it.

### 2.5 Dependency graph

Kernel provides generic graph structure and deterministic graph algorithms. It does not decide what a node means, parse dependencies, schedule recalculation, propagate domain errors, or decide worksheet invalidation policy. Engine constructs and mutates the graph and owns cycle/error interpretation at the worksheet level.

### 2.6 Diagnostics and spans

- Diagnostics carry stable, owner-namespaced codes and human-readable messages.
- Source spans are generic source offsets; Math owns their interpretation against expression text.
- Kernel must not encode Math parser diagnostics or Table-specific validation rules.

## 3. Audit of current implementation

The current implementation branch has:

| API | Audit result |
| --- | --- |
| NodeId and allocator | Implemented in PR #2; retain as generic identity |
| Port | Implemented in PR #2 as a node ID plus opaque index |
| Revision | Implemented in PR #2 with checked monotonic progression |
| Snapshot<T> | Implemented in PR #2 as a complete revisioned payload |
| ResultStore<V> | Implemented in PR #2 and keyed only by NodeId |
| DependencyGraph<N> | Generic graph retained; Engine decides graph contents and semantics |
| Diagnostic / Span | Generic primitives retained; domain codes remain owner-defined |
| Envelope<T> | Existing versioned transport shape; Snapshot is composable as Envelope<Snapshot<T>> |
| StateStore<K, V> / StateSnapshot<K, V> | Removed from the public module surface in PR #2 |
| Scope/Variable state proposal | Superseded; do not merge or extend as the Kernel architecture |

PR #2 has been updated to the narrower API. Its Rust formatting, default/Serde tests and Clippy still require fresh CI evidence before the milestone can be marked verified.

## 4. Development milestones

### M0 — Contract and source audit

- [x] Inspect existing Kernel API and active implementation/documentation branches.
- [x] Identify conflict between the older variable-state proposal and the revised generic infrastructure boundary.
- [x] Identify current StateStore, StateSnapshot, Envelope, graph, ID, diagnostic and span APIs.
- [ ] Record the final Envelope / Snapshot<T> serialization composition in a small compile-time API test.
- [ ] Record whether a generic incremental change type is required independently of snapshots.

**Exit:** public API contract is explicit and no Kernel type encodes Math, Table, worksheet, or Engine concepts.

### M1 — Minimal Kernel primitives

- [x] Finalize NodeId and generic Port.
- [x] Add Revision(u64) with checked monotonic progression.
- [x] Implement Snapshot<T> { revision, payload }.
- [x] Implement ResultStore<V> keyed only by NodeId.
- [x] Keep generic DependencyGraph without recalculation policy.
- [x] Preserve generic Diagnostic and Span.
- [x] Remove the old StateStore/StateSnapshot public module surface from PR #2.

**Implementation status:** the above API work is present in PR #2; verification remains pending because no workflow run was returned for its latest commit.

**Exit:** the crate exposes only the minimal generic contract and compiles without any Jabr domain dependency.

### M2 — Contract and serialization tests

- [ ] Compile-time API tests for public signatures and generic bounds.
- [ ] Snapshot clone/isolation and serialization round-trip tests.
- [ ] Revision monotonicity, overflow and stale-revision boundary tests.
- [ ] ResultStore insert/replace/remove/missing-key tests.
- [ ] Deterministic graph order, unknown-node and cycle tests.
- [ ] Diagnostics and spans serialization tests.
- [ ] Default and Serde-feature CI; formatting, Clippy and docs checks.

**Exit:** Kernel behavior is deterministic, tested and independently consumable.

### M3 — Cross-repository integration contracts

- [ ] Publish a tagged Kernel API version.
- [ ] Add consumer-side compile tests in Math and Engine.
- [ ] Verify Engine can instantiate ResultStore<MathValue> without Kernel depending on Math.
- [ ] Verify Table has no dependency on Math or Kernel result storage.
- [ ] Verify UI consumes revisioned Engine snapshots/updates without persisting derived results to Yjs.

**Exit:** each repository remains independently buildable and the boundaries are proven by compile-time tests.

## 5. Non-goals

- Mathematical variables, assignments, scopes, parsing or evaluation.
- Math values, quantities, dimensions, units or conversion.
- Table cell text parsing or table-specific data semantics.
- Worksheet graph construction, dependency interpretation or recalculation scheduling.
- Result lifecycle policy beyond generic storage operations.
- BlockSuite/Yjs document ownership, collaboration or undo/redo.
- Cross-engine orchestration or an application API.
- A monorepo or shared Cargo workspace.

## 6. Change policy

Any change to these ownership boundaries requires an explicit architecture review. Keep implementation changes focused, independently tested, and in pull requests. Do not merge any open Kernel PR as part of this documentation update.
