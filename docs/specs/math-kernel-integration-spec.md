# Math–Kernel Integration Specification

**Status:** M0 architecture draft — revised for Architecture A  
**Contract target:** `jabr-kernel-api` (domain-neutral Kernel primitives only)  
**Implementation:** `jabr-kernel`  
**Orchestration owner:** planned Jabr Engine subsystem  
**Version target:** `kernel-api-v1`  
**Freeze rule:** This document is a proposal until reviewed and explicitly accepted.

## 1. Purpose and governing decision

Define the boundary between Jabr Math and Jabr Kernel while preserving the current repository ownership model.

**Architecture A is normative for this draft:** Kernel remains a small, domain-neutral infrastructure library. It does not own worksheet state, variable state, document or node revision counters, calculation results, result caches, snapshots/history, dirty-state policy, or recalculation scheduling. Those responsibilities belong to the planned Jabr Engine subsystem or, for authored document persistence/history, Jabr-UI and BlockSuite/Yjs.

The previously proposed expanded Kernel model (`KernelStateRead`, `KernelStateWrite`, `CellState`, Kernel-owned revisions/snapshots/result store) is explicitly not part of this contract.

## 2. Ownership and dependency direction

| Responsibility | Owner | Contract boundary |
|---|---|---|
| Stable generic identity and ports | `jabr-kernel-api` | Domain-neutral IDs and endpoints |
| Generic dependency graph primitives and algorithms | `jabr-kernel-api` / `jabr-kernel` | No worksheet scheduling semantics |
| Generic source spans and diagnostic envelope | `jabr-kernel-api` | Namespaced codes; domain meaning stays with emitter |
| Versioned generic transport envelope | `jabr-kernel-api` | Payload remains opaque |
| Concrete storage and graph algorithms | `jabr-kernel` | Implements public contract |
| Mathematical syntax, parser, AST, symbols, values, quantities, units and evaluation | Jabr Math | Math-owned domain model |
| Expression-local reference extraction | Jabr Math | Returns symbolic references, not Kernel graph mutations |
| Worksheet node mapping and cross-engine graph construction | Jabr Engine | Maps domain references to generic IDs/ports |
| Document/node/source revisions, authored/derived lifecycle, result cache, snapshots and history of calculation state | Jabr Engine | Engine-owned lifecycle |
| Dirty propagation, affected-node selection, scheduling and recalculation orchestration | Jabr Engine | Engine-owned policy |
| Authored document, collaboration, persistence and authored undo/redo | Jabr-UI / BlockSuite / Yjs | Durable authored content |
| Result presentation and transient rendering cache | Jabr-UI | Not a second calculation authority |

### 2.1 Dependency direction

```text
Jabr-UI (authored document + presentation)
        |
        v
Jabr Engine (worksheet state, graph mapping, revisions,
             result lifecycle, scheduling, orchestration)
        |                  |
        v                  v
jabr-kernel-api          Jabr Math
        ^                  |
        |                  |
   jabr-kernel       parse / analyze / evaluate
 (implementation)

Jabr Table and Jabr Diagram expose their own domain models;
Jabr Engine maps their references into the worksheet graph.
```

- `jabr-kernel-api` must not depend on Math, Engine, Table, Diagram, BlockSuite, Yjs, or UI.
- `jabr-kernel` implements the generic public API and must not depend on Math.
- Math and other consumers may depend on `jabr-kernel-api`, never on Kernel implementation modules.
- Jabr Engine may depend on `jabr-kernel-api` and domain-engine facades.
- The application composition root creates concrete implementations and injects them.
- No `Arc<Mutex<...>>`, trait object tied to a concrete runtime, or implementation-owned reference crosses a serialized inter-engine boundary.

## 3. Kernel public contract scope

The public crate exposes only the infrastructure already assigned to Kernel:

1. **Identity:** stable, domain-neutral `NodeId` and generic `Port` identity.
2. **Dependency graph:** generic graph representation, edge operations, cycle detection and deterministic topological ordering.
3. **Source locations:** validated source-independent spans and locations with documented coordinate conventions.
4. **Diagnostics:** generic structured diagnostic envelope and severity; stable codes are namespaced by the emitting domain.
5. **Versioned envelopes:** generic serialized request/result/diagnostic transport with explicit schema version.

Kernel does **not** provide:
- variable or assignment state, scopes, namespaces or symbol tables;
- document, node, source or calculation revisions;
- authored/derived/error/empty worksheet state;
- calculation-result storage, invalidation, cache validity or publication;
- snapshots, undo/redo, state history or restore policy;
- cross-engine graph construction, dependency interpretation or dirty propagation;
- recalculation scheduling, execution policy, notifications or orchestration;
- Math values, Quantity, Unit, AST, expression diagnostics or evaluator behavior.

The dedicated `jabr-kernel-api` crate is still the accepted public-contract packaging direction. Under Architecture A, it packages the generic capabilities above; it does not reintroduce the expanded state/revision/snapshot API.

## 4. Proposed generic Rust contract shapes

These signatures are illustrative contract candidates. They are not claims about current exports and must be checked against the implementation and compile-time tests before freeze.

### 4.1 Identity

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(/* private, stable representation */);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Port {
    pub node_id: NodeId,
    pub index: u32,
}
```

The owning document/runtime defines the identity namespace. Kernel does not infer whether a node represents an expression, table cell, figure parameter or another domain object. Port meaning remains opaque.

### 4.2 Generic dependency graph

```rust
pub trait DependencyGraphApi<N>
where
    N: Clone + Ord,
{
    fn add_node(&mut self, node: N);
    fn add_dependency(
        &mut self,
        node: N,
        prerequisite: N,
    ) -> Result<(), GraphError<N>>;
    fn remove_node(&mut self, node: &N) -> bool;
    fn topological_order(&self) -> Result<Vec<N>, GraphError<N>>;
}
```

Edge direction is normative: `node -> prerequisite` means the node depends on the prerequisite. Returned topological order is prerequisite-first. Kernel may detect generic graph cycles; Engine decides how those cycles map to worksheet diagnostics and recovery behavior.

Kernel's topological ordering is a graph algorithm, not a scheduler. It does not choose affected nodes, execute expressions, publish results or trigger recalculation.

### 4.3 Diagnostics and spans

```rust
pub enum Severity { Info, Warning, Error }

pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
}

pub struct Span {
    // Validated half-open UTF-8 byte range.
}
```

- Codes are stable and owner-namespaced: `kernel.*`, `math.*`, `engine.*`, `table.*`, `diagram.*`.
- Consumers must not parse human-readable messages.
- Kernel defines the generic envelope and span conventions, not domain-specific codes.
- UTF-8 byte offsets are the cross-engine source convention. UI adapters convert to editor-specific offsets (including UTF-16 where required).

### 4.4 Versioned envelope

```rust
pub struct Envelope<T> {
    pub schema_version: u32,
    pub result: Option<T>,
    pub diagnostics: Vec<Diagnostic>,
}
```

Envelope version and payload/domain version are separate. Unsupported versions fail explicitly; no silent interpretation as the current schema. Exact numeric payloads must use tagged string/integer representations rather than lossy JSON floating-point encoding.

## 5. Math facade contract

Math exposes parsing, analysis and evaluation. It does not expose Kernel storage, mutate the worksheet graph, schedule dependent nodes or commit results.

Math returns symbolic references from its AST. It does not resolve those references to Kernel `NodeId` values; Engine owns that mapping.

Proposed conceptual DTOs:

```rust
pub struct SymbolReference {
    pub name: String,
    pub span: Option<Span>,
}

pub struct MathAnalysis {
    pub references: Vec<SymbolReference>,
    pub diagnostics: Vec<MathDiagnostic>,
    // Parsed AST remains Math-owned.
}

pub struct ResolvedBinding {
    pub name: String,
    pub value: MathValue,
}

pub struct EvaluationContext {
    pub bindings: Vec<ResolvedBinding>,
    pub source_revision: String,
    pub input_fingerprint: String,
}

pub struct MathEvaluation {
    pub value: Option<MathValue>,
    pub diagnostics: Vec<MathDiagnostic>,
    pub source_revision: String,
    pub input_fingerprint: String,
}

pub trait MathEngine {
    fn analyze(&self, source: &str) -> MathAnalysis;
    fn evaluate(
        &self,
        source: &str,
        context: &EvaluationContext,
    ) -> MathEvaluation;
}
```

The exact facade signatures, AST DTO policy and value serialization remain Math-owned and must be frozen in the Math facade contract. The example uses owned, serializable data; no Kernel trait object or borrowed Kernel state crosses the boundary.

### 5.1 Evaluation purity

For a fixed source, explicit context and compatible Math engine version, evaluation must not mutate external worksheet state. It may return diagnostics and a proposed value. Engine decides whether that result is still current and whether to publish it.

Math may use Kernel's generic `NodeId`, `Span` or `Envelope` types where useful, but its parser and evaluator must remain independently testable without a running Kernel implementation.

## 6. Engine orchestration and recalculation lifecycle

**Jabr Engine owns recalculation triggering and orchestration.** Kernel provides generic graph operations; Math provides expression analysis and evaluation.

```text
BlockSuite/Yjs authored change
          |
          v
Jabr-UI sends authored source + block identity
          |
          v
Jabr Engine records authored state and advances Engine revisions
          |
          v
Math analyzes source and returns symbolic references
          |
          v
Engine resolves references to worksheet nodes/ports
          |
          v
Engine updates the generic Kernel dependency graph
          |
          v
Kernel returns generic graph/cycle/topological information
          |
          v
Engine selects affected nodes and schedules evaluation
          |
          v
Engine constructs owned immutable EvaluationContext
          |
          v
Math evaluates and returns value + diagnostics
          |
          v
Engine validates source/input revisions and engine version
          |
          v
Engine commits/publishes accepted derived state in its result lifecycle
          |
          v
Jabr-UI renders revision-associated derived output
```

Normative rules:

1. UI submits authored source and stable block identity; it does not parse mathematical dependencies.
2. Engine owns authored worksheet calculation records, source/document/node revisions and calculation generations.
3. Math parses and extracts expression-local symbolic references.
4. Engine resolves symbols, maps them to generic Kernel identities and updates graph edges.
5. Kernel offers generic graph operations and prerequisite-first topological ordering.
6. Engine computes the affected set, detects worksheet-level cycles and schedules evaluation.
7. Engine resolves prerequisite values and passes an immutable owned context to Math.
8. Math returns proposed values and Math-owned diagnostics; it never mutates Kernel or Engine state.
9. Engine rejects a result if its source revision, input fingerprint, calculation generation or relevant engine version no longer matches.
10. Engine publishes accepted results and owns cache validity/invalidation.
11. UI receives result updates and keeps only a transient presentation cache. It does not persist calculated values in Yjs.

No Kernel change event, Math callback subscription or Kernel-owned scheduler is required by this design.

## 7. Authored/derived state and revision ownership

This integration contract does not define a Kernel `CellState`. The lifecycle is owned by Engine and UI:

- BlockSuite/Yjs persists authored source and user-authored preferences.
- Engine maintains calculation records that associate authored inputs, dependency/input fingerprints, revisions, calculation generation, result and diagnostics.
- A node may retain authored source while having a separate derived result or error. Recalculation never overwrites authored content.
- Engine owns document, node/source and calculation-generation revisions. Their exact semantics and snapshot policy belong to the Engine lifecycle contract.
- A no-op result may avoid a visible derived-state update, but calculation attempts still have their own generation/request identity.
- Undo/redo restores authored document content through BlockSuite/Yjs. It creates a new Engine-observed input state and recalculates; it does not rewind monotonic Engine revisions.
- Engine may snapshot its calculation lifecycle for recovery, but Kernel does not own or serialize worksheet snapshots.

Detailed authored/derived lifecycle and cache rules are normative in `dual-zone-block-architecture.md` for UI-visible behavior and must be completed in the future Jabr Engine specification before Engine implementation.

## 8. Versioning and compatibility

- Version `jabr-kernel-api` independently from `jabr-kernel`, Jabr Math and the wire envelope.
- Consumers compile against the public contract only.
- Breaking Rust API, trait or serialized semantic changes require an explicit major-version decision.
- Unsupported envelope versions fail explicitly; migrations belong in explicit adapters.
- Keep Rust API types distinct from browser DTOs where number/string or offset semantics differ.
- Preserve exact rational/decimal values using tagged serializable forms.
- Native and WASM Math adapters must pass the same golden evaluation fixtures.

## 9. Acceptance criteria

### Kernel API
- Public API crate contains only generic identity, ports, graph, spans, diagnostics and envelope contracts.
- Kernel has no dependency on Math, Engine, Table, Diagram, BlockSuite, Yjs or UI.
- No variable state, revision counter, result store, snapshot/history or recalculation scheduler is exported from Kernel.
- Graph tests prove edge direction, deterministic prerequisite-first ordering, missing-node behavior and cycle reporting.
- Envelope and diagnostic/span tests prove round-trip/version rejection and documented coordinate semantics.

### Math
- Math builds without importing `jabr-kernel` implementation modules.
- Parser/AST/evaluator tests run without a Kernel service.
- Reference extraction returns symbolic names and source spans, not worksheet NodeIds.
- Evaluation consumes owned explicit bindings and returns values/diagnostics without mutating external state.
- Unit and numeric domain types remain Math-owned.

### Engine integration
- Engine maps authored block identity and Math references to generic NodeId/Port.
- Engine owns revisions, result lifecycle, dirty policy and scheduling.
- Engine uses Kernel graph operations but does not delegate scheduling to Kernel.
- Engine rejects stale results using source/input identity.
- UI receives revision-associated output without persisting results in Yjs.

## 10. Open items before freeze

1. Confirm the exact `NodeId` and `Port` representation/namespace rules against the current Kernel implementation.
2. Freeze graph API error variants, duplicate-edge behavior, missing-node behavior and deterministic ordering.
3. Freeze the generic diagnostic/envelope serialization and unknown-field policy.
4. Freeze Math's owned evaluation DTOs and symbolic-reference semantics.
5. Define the Engine calculation-record, revision, cache, snapshot and transaction contracts in an Engine-owned specification.
6. Define how cross-engine values (Math, Table, Diagram) are encoded in Engine-owned result records without adding domain semantics to Kernel.

## 11. Explicit non-goals

- Expanding Kernel into a variable/state/history or result-storage subsystem.
- Kernel-owned revisions, snapshots, cache validity or calculation triggers.
- Math depending on Kernel implementation internals or reading arbitrary Kernel state.
- Math scheduling worksheet recalculation or committing results.
- UI parsing/evaluating expressions or persisting derived results in Yjs.
- Treating a topological order as a complete scheduler.
- Creating a monorepo or forcing all engines into one Cargo workspace.
- Implementing source code or creating the public crate as part of this specification-only change.
