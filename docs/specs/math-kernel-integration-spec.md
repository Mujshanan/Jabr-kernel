# Math–Kernel Integration Specification

**Status:** Accepted architecture baseline; detailed Rust contract shapes remain subject to implementation/API tests  
**Contract target:** `jabr-kernel-api` (domain-neutral Kernel primitives only)  
**Implementation:** `jabr-kernel`  
**Orchestration owner:** planned Jabr Engine subsystem  
**Version target:** `kernel-api-v1`  
**Freeze rule:** This document is a proposal until reviewed and explicitly accepted.

## 1. Purpose and governing decision

Define the boundary between Jabr Math and Jabr Kernel while preserving the current repository ownership model.

**Architecture A is normative for this draft:** Kernel remains a small, domain-neutral infrastructure library. It does not own worksheet state, variable state, document or node revision counters, calculation results, result caches, snapshots/history, worksheet dirty-state policy, or recalculation scheduling. Its generic graph may compute the transitive dependent closure of explicitly supplied roots; Engine decides which roots are dirty and what work is scheduled. Those responsibilities belong to the planned Jabr Engine subsystem or, for authored document persistence/history, Jabr-UI and BlockSuite/Yjs.

The previously proposed expanded Kernel model (`KernelStateRead`, `KernelStateWrite`, `CellState`, Kernel-owned revisions/snapshots/result store) is explicitly not part of this contract.

## 2. Ownership and dependency direction

| Responsibility | Owner | Contract boundary |
|---|---|---|
| Stable generic identity and ports | `jabr-kernel-api` | Domain-neutral IDs and endpoints |
| Generic dependency graph primitives and algorithms | `jabr-kernel-api` / `jabr-kernel` | Includes generic dependent-closure and prerequisite-first ordering; no worksheet dirty-state policy or scheduler |
| Generic source spans and diagnostic envelope | `jabr-kernel-api` | Namespaced codes; domain meaning stays with emitter |
| Versioned generic transport envelope | `jabr-kernel-api` | Payload remains opaque |
| Concrete storage and graph algorithms | `jabr-kernel` | Implements public contract |
| Mathematical syntax, parser, AST, symbols, values, quantities, units and evaluation | Jabr Math | Math-owned domain model |
| Expression-local reference extraction | Jabr Math | Returns symbolic references, not Kernel graph mutations |
| Sequential worksheet interpretation and execution environment | Jabr Engine | Executes authored blocks/statements in document order |
| Document/node/source revisions, authored/derived lifecycle, result cache, snapshots and history of calculation state | Jabr Engine | Engine-owned lifecycle |
| Worksheet rerun, execution batching and recalculation orchestration | Jabr Engine | V1 reruns the worksheet from the top; no dependency scheduler |
| Authored document, collaboration, persistence and authored undo/redo | Jabr-UI / BlockSuite / Yjs | Durable authored content |
| Result presentation and transient rendering cache | Jabr-UI | Not a second calculation authority |

### 2.1 Dependency direction

```text
Jabr-UI (authored document + presentation)
        |
        v
Jabr Engine (worksheet state, sequential interpreter,
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
- cross-engine graph construction, dependency interpretation, or worksheet dirty-state policy;
- recalculation scheduling, execution policy, notifications or orchestration;
- Math values, Quantity, Unit, AST, expression diagnostics or evaluator behavior.

The dedicated, versioned `jabr-kernel-api` crate is the accepted public-contract packaging direction. Under Architecture A, it packages the generic capabilities above; it does not reintroduce the expanded state/revision/snapshot API.

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

### 4.2 Port-aware dependency graph

The concrete graph contract uses `NodeId` and the existing opaque `Port { node_id, index: u32 }` identity. Edge direction is normative: a node records the prerequisite ports it reads. Port indexes remain opaque to Kernel. Distinct ports from one prerequisite remain distinct in storage, while graph ordering deduplicates scheduling edges at the `NodeId` level.

The public operation set includes node insertion/removal, port-dependency insertion, atomic prerequisite replacement, prerequisite/dependent queries, deterministic transitive dependent closure for explicit roots, full prerequisite-first topological ordering, and prerequisite-first ordering restricted to a supplied dirty closure. Failed dependency replacement validates all references before mutation. Serde reconstruction validates references and rebuilds derived reverse indexes.

A cycle error reports only actual cycle members, including self-loops; downstream nodes blocked by a cycle are excluded. Multiple cycle components are reported deterministically.

The graph's closure and ordering methods are generic graph algorithms, not worksheet dirty-state policy or an execution scheduler. They do not choose the dirty roots, execute expressions, publish results, or trigger recalculation. Engine decides when and how to use these primitives.

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
    schema_version: u32,
    pub result: Option<T>,
    pub diagnostics: Vec<Diagnostic>,
}
```

Envelope version and payload/domain version are separate. Unsupported versions fail explicitly; no silent interpretation as the current schema. Exact numeric payloads must use tagged string/integer representations rather than lossy JSON floating-point encoding.

## 5. Math facade contract

Math parses and evaluates individual mathematical expressions against explicit owned bindings. It does not own worksheet execution order, variable storage, assignment lifecycle, worksheet scopes, scheduling or result commits.

Jabr Engine is the sequential worksheet interpreter. It determines which statement executes next, which variables are available, and which values are supplied to Math. Math does not request missing worksheet dependencies or maintain a reactive worksheet dependency graph.

### 5.1 Analysis and expression evaluation

Analysis may return syntactic symbol references and source spans for diagnostics and editor assistance. In V1 these references are informational; they do not create runtime graph edges or make inactive branches mandatory dependencies.

Conditional expressions are lazy: evaluate the condition, select one branch, and evaluate only that branch. An inactive branch does not require runtime bindings and cannot produce runtime evaluation errors. The complete expression must still be syntactically valid.

```rust
pub struct SymbolReference { pub name: String, pub span: Option<Span> }
pub struct MathAnalysis {
    pub references: Vec<SymbolReference>,
    pub diagnostics: Vec<MathDiagnostic>,
}
pub struct ResolvedBinding { pub name: String, pub value: MathValue }
pub struct ExpressionEvaluationRequest {
    pub request_id: String,
    pub source: String,
    pub bindings: Vec<ResolvedBinding>,
    pub options: EvaluationOptions,
}
pub struct ExpressionEvaluationResponse {
    pub request_id: String,
    pub value: Option<MathValue>,
    pub diagnostics: Vec<MathDiagnostic>,
}
pub trait MathFacade {
    fn analyze(&self, source: &str) -> MathAnalysis;
    fn evaluate(&self, request: &ExpressionEvaluationRequest)
        -> ExpressionEvaluationResponse;
}
```

These are conceptual shapes, not frozen public types. Exact value serialization, source formats, spans, numeric policies and DTO versioning remain Math-owned.

### 5.2 Evaluation purity

For fixed expression source, explicit bindings, options and compatible Math semantic version, evaluation must not mutate Engine state. It returns a proposed value and diagnostics. Engine owns assignment application and decides whether the result belongs to the current execution attempt.

### 5.3 Conditional semantics

Math evaluates only the selected branch. Syntactic references in inactive branches are not runtime dependencies. A selected branch referencing a variable unavailable in the current sequential environment returns an unresolved-symbol diagnostic. Math does not issue dependency requests.

## 6. Engine sequential worksheet interpreter

**Jabr Engine owns worksheet interpretation and execution.** V1 uses deterministic top-to-bottom execution, not a reactive dependency graph.

```text
BlockSuite/Yjs authored worksheet
          |
          v
Jabr-UI submits ordered authored blocks
          |
          v
Engine creates a worksheet execution ID, advances generations for scheduled calculation targets, and creates a fresh environment
          |
          v
Engine reads next statement/block
          |
          v
Engine dispatches statement; supplies current bindings to Math
          |
          v
Math returns expression value and diagnostics
          |
          v
Engine records attempt and applies successful assignment
          |
          v
Engine continues through worksheet in document order
          |
          v
Engine publishes execution snapshot and block outcomes
          |
          v
Jabr-UI renders derived output
```

Normative V1 rules:

1. Engine executes authored worksheet statements/blocks in document order, from top to bottom.
2. A variable becomes available only after its assignment executes successfully.
3. Forward references are prohibited in V1. A reference to a variable not yet defined is unresolved at that point.
4. Engine owns the execution environment, variable bindings, assignment application, ordering, per-target source revisions/generations and the worksheet execution ID.
5. Math owns parsing and evaluation of individual expressions against explicit bindings.
6. Conditional expressions and statement blocks execute only their selected branches.
7. A successful assignment updates the environment for subsequent statements; it does not mutate earlier captured values or snapshots.
8. A failed statement records diagnostics and does not partially apply its assignment. Independent later statements may continue; statements requiring unavailable values are blocked or report unresolved symbols.
9. Engine records per-statement/per-block outcomes, target-specific attempt identities and an immutable worksheet execution snapshot.
10. An authored change triggers a full top-to-bottom rerun using a fresh environment. Dependency-based incremental recalculation is not part of V1.
11. Engine validates the worksheet execution ID plus each target's current source revision and per-target calculation generation before publishing its outcome. Late responses from obsolete attempts are rejected.
12. Kernel graph primitives are not used as the V1 worksheet scheduler. They remain generic infrastructure for other uses and possible future optimization.

### 6.1 Error isolation

A failed statement must not corrupt the execution environment or prevent independent later statements from running. A later statement requiring a variable whose assignment failed receives an explicit unavailable/unresolved diagnostic. Engine must not silently substitute a value from a previous execution.

### 6.2 Execution boundary

Engine interprets worksheet order and control flow; Math evaluates expressions. The underlying Calcpad implementation remains behind a replaceable adapter and must not dictate Jabr's worksheet semantics.

## 7. Authored state, execution environment and result lifecycle

- BlockSuite/Yjs persists authored source, block order and user-authored preferences.
- Engine creates a fresh execution environment for each worksheet execution ID.
- Engine owns variable definitions, bindings, assignment updates, worksheet scopes/control flow, per-block outcomes, per-target source revisions, per-target calculation generations, worksheet execution IDs and snapshots.
- Math receives immutable owned values for each expression and cannot mutate Engine state.
- Successful assignments update the environment for subsequent statements only.
- Failed assignments do not overwrite the last successful execution snapshot.
- Engine records failed attempts and diagnostics separately. UI may retain the last successful output only when explicitly marked stale.
- Undo/redo restores authored content; Engine observes a new input state and starts a new execution. Revisions and generations never rewind.
- V1 recalculation starts from the first executable statement. Reuse of prior execution state is not part of the correctness model.

Variable reassignment, scope rules, immutable snapshot contents and transaction semantics must be reviewed in the Engine contract before implementation freeze.



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
- Public API contains only generic identity, ports, graph, spans, diagnostics and envelope contracts.
- Kernel has no dependency on Math, Engine, Table, Diagram, BlockSuite, Yjs or UI.
- No variable state, revision counter, result store, snapshot/history or worksheet scheduler is exported from Kernel.
- Generic graph, envelope and diagnostic tests verify their documented behavior.

### Math
- Math builds without importing Kernel implementation modules.
- Parser/AST/evaluator tests run without a Kernel service.
- Analysis returns syntactic references and source spans without creating worksheet graph edges.
- Conditional expressions evaluate lazily and never evaluate inactive branches.
- Evaluation consumes explicit bindings and returns values/diagnostics without mutating external state.
- Math does not issue worksheet dependency requests or own worksheet variable state.
- Unit and numeric domain types remain Math-owned.

### Engine integration
- Engine executes worksheet statements in deterministic document order.
- Forward references are unavailable/rejected in V1.
- Engine owns the environment and applies successful assignments for subsequent statements.
- Failed statements do not partially mutate the environment; independent later statements can continue.
- Each worksheet run uses a fresh environment and produces an immutable snapshot. Each calculation target retains its own monotonic source revision and calculation generation; a worksheet execution ID groups the target attempts belonging to one run.
- Authored changes trigger a full top-to-bottom rerun in V1.
- Engine rejects late results from obsolete execution generations.
- UI receives derived output without persisting it in Yjs.

## 10. Open items before freeze

1. Keep the `NodeId` and `Port { node_id, index: u32 }` contract aligned with public API tests.
2. Keep port-aware graph operations, atomic replacement, reverse-index invariants, deterministic dependent closure, and actual-cycle-member reporting aligned with implementation and tests.
3. Freeze diagnostic/envelope serialization and unknown-field policy.
4. Freeze Math expression-evaluation DTOs, source formats, symbol lookup diagnostics and lazy conditional semantics.
5. Define Engine execution environment, reassignment, scope, per-target revision/generation, worksheet execution ID, immutable snapshot, failure-isolation and transaction contracts.
6. Define how Table/Diagram values enter the sequential Engine environment without adding domain semantics to Kernel.
7. Decide whether future optimizations may introduce dependency-based scheduling while preserving V1's observable sequential semantics.



## 11. Explicit non-goals

- Expanding Kernel into a variable/state/history or result-storage subsystem.
- Kernel-owned revisions, snapshots, cache validity or calculation triggers.
- Math depending on Kernel implementation internals or reading arbitrary Kernel state.
- Math scheduling worksheet recalculation or committing results.
- UI parsing/evaluating expressions or persisting derived results in Yjs.
- Treating a topological order as the V1 worksheet execution model.
- Dynamic dependency requests or reactive dependency-graph scheduling in V1.
- Forward references in V1.
- Creating a monorepo or forcing all engines into one Cargo workspace.
- Implementing source code or creating the public crate as part of this specification-only change.
