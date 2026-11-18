# Math–Kernel Integration Specification

**Status:** Contract specification draft for review  
**Scope:** Public Rust contract, Math/Kernel boundary, and Engine-mediated evaluation lifecycle  
**Target contract crate:** \`jabr-kernel-api\`  
**Implementation crate:** \`jabr-kernel\`

## 1. Purpose

Define the stable boundary between Jabr Math and Jabr Kernel so that Math and other Jabr engines can consume shared Kernel contracts without importing Kernel implementation details.

This specification reconciles two requirements:

1. The accepted architecture exposes a dedicated, versioned Rust public-contract crate named \`jabr-kernel-api\`; the implementation remains in \`jabr-kernel\`.
2. Kernel remains domain-neutral. It does not become a mathematical evaluator, worksheet orchestrator, or owner of Math-specific variable semantics.

The separate Jabr Engine subsystem is the worksheet orchestrator. Its repository is planned and is not yet present in the connected repository set. This specification does not create that repository or implement any Kernel code.

## 2. Normative ownership

| Capability | Owner | Boundary |
|---|---|---|
| Public shared Rust types and traits | \`jabr-kernel-api\` | Versioned, domain-neutral contract |
| Storage and generic algorithms | \`jabr-kernel\` | Implements the public contract |
| Mathematical syntax, parser and AST | Jabr Math | Math-owned types and semantics |
| Mathematical values, quantities and units | Jabr Math | Runtime mathematical model |
| Expression-local symbol/reference analysis | Jabr Math | Derived from Math AST |
| Worksheet node/port mapping | Jabr Engine | Maps authored blocks and Math references |
| Cross-block dependency graph construction | Jabr Engine | Uses generic Kernel graph primitives |
| Dirty-state policy and recalculation scheduling | Jabr Engine | Owns the worksheet lifecycle |
| Evaluation orchestration | Jabr Engine | Calls Math in dependency order |
| Calculation-result lifecycle | Jabr Engine | Publishes/invalidate results |
| Authored worksheet and UI | Jabr-UI | BlockSuite/Yjs document and presentation |

### 2.1 Dependency direction

~~~text
Jabr-UI
   |
   v
Jabr Engine --------------------+
   |                            |
   +--> jabr-kernel-api          |
   |       ^                    |
   |       | implemented by     |
   |   jabr-kernel              |
   |                            |
   +--> Jabr Math ---------------+
   +--> Jabr Table
   +--> Jabr Diagram
~~~

- \`jabr-kernel-api\` has no dependency on Math, Engine, Table, Diagram, BlockSuite, Yjs, or UI.
- \`jabr-kernel\` depends on \`jabr-kernel-api\` and implements its contracts.
- Math and other engine repositories may depend on \`jabr-kernel-api\`, never on Kernel implementation modules.
- Jabr Engine depends on the public contracts and domain-engine APIs; it must not import Kernel internals.
- The application composition root constructs the concrete Kernel implementation and injects the public services. The location of that composition root is not fixed by this specification.
- Kernel must never depend on Math.

### 2.2 Math's use of Kernel API

Math may use shared identity, revision, source-span, diagnostic, envelope, and snapshot types where they are genuinely useful at its public boundary.

Math does not use Kernel as its worksheet state manager. It does not own or mutate the cross-block graph, schedule worksheet recalculation, commit calculation results, or decide when another block becomes dirty.

Math remains independently testable without a running Kernel implementation. Its pure parser, AST, unit, and evaluator tests must not require a Kernel service instance.

## 3. Public contract crate

The target crate layout is:

~~~text
jabr-kernel-api/
  src/
    lib.rs
    identity.rs
    revision.rs
    snapshot.rs
    result_store.rs
    graph.rs
    diagnostic.rs
    span.rs
    envelope.rs

jabr-kernel/
  src/
    ... concrete storage and algorithm implementations ...
~~~

The API crate contains public types, traits, error contracts, and serialization definitions. It must not contain storage backends, synchronization policy, database choices, or application orchestration.

The implementation crate provides concrete implementations and internal optimizations. Consumers compile against \`jabr-kernel-api\` only.

**Current repository status:** the open Kernel implementation work contains a domain-neutral Kernel crate and generic graph/identity/diagnostic/envelope code. The separate \`jabr-kernel-api\` crate and the complete public-contract extraction are not yet established. Existing development documentation and implementation branches must be audited against this target before any extraction. This specification is not evidence that the target crate already exists.

## 4. Public API surface and proposed signatures

The following Rust signatures define the proposed contract shape. Names and bounds are subject to compile-time contract tests before the contract is frozen. They are not claims about currently exported implementation symbols.

### 4.1 Identity

~~~rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(/* private representation */);

impl NodeId {
    pub fn new(raw: u64) -> Result<Self, IdError>;
    pub const fn get(self) -> u64;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Port {
    pub node_id: NodeId,
    pub index: u32,
}

impl Port {
    pub const fn new(node_id: NodeId, index: u32) -> Self;
}
~~~

Kernel defines identity only. Engine defines whether a node represents a Math expression, table range, diagram output, or another worksheet object. Port indexes remain opaque to Kernel.

Identity namespace and persistence rules must be explicit. IDs are not globally unique merely because they are represented by integers; the owning document/runtime defines the namespace.

### 4.2 Revision

~~~rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(u64);

impl Revision {
    pub const fn new(value: u64) -> Self;
    pub const fn get(self) -> u64;
    pub fn checked_next(self) -> Result<Self, RevisionError>;
}
~~~

- Revisions are monotonic within the owning Engine document/session.
- Kernel defines the value type and checked progression primitive.
- Engine owns when a revision advances.
- Undo/restore of authored content creates a newer revision; it never rewinds the counter.
- Revision overflow is an explicit error.
- When serialized through JSON/JavaScript, a full-width revision is encoded as a decimal string, not a JSON number.

A Math-local source revision may be used for expression-specific cache invalidation, but it must not be confused with the Engine document revision or a Kernel revision.

### 4.3 Snapshot

~~~rust
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot<T> {
    pub revision: Revision,
    pub payload: T,
}

impl<T> Snapshot<T> {
    pub fn new(revision: Revision, payload: T) -> Self;
}
~~~

A snapshot represents a complete payload at one revision. It is not an event, patch, or incremental update.

Use the agreed envelope composition for transport, for example \`Envelope<Snapshot<T>>\`. Do not add duplicate schema-version fields to the snapshot unless a separate compatibility requirement is documented.

### 4.4 Generic result store

~~~rust
pub trait ResultStoreApi<V> {
    fn get(&self, node: NodeId) -> Option<&V>;
    fn insert(&mut self, node: NodeId, value: V) -> Option<V>;
    fn remove(&mut self, node: NodeId) -> Option<V>;
}
~~~

- Values are opaque to Kernel.
- The Engine selects the stored value type and owns insertion, replacement, invalidation, and removal policy.
- Kernel must not define MathValue, Quantity, units, variable names, scopes, or calculation errors-as-values.
- The API must not expose mutable references that let consumers mutate stored values without an explicit replacement operation.
- If atomic multi-node publication is required, Engine stages a complete result snapshot and publishes it as one Engine-owned operation. Do not add worksheet transactions to Kernel merely to implement one evaluator.

### 4.5 Generic dependency graph

~~~rust
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
~~~

Kernel owns the generic graph representation and deterministic graph algorithms. Engine owns graph contents, node mapping, graph update policy, affected-node selection, and interpretation of cycles as worksheet diagnostics.

The graph API does not expose a method named \`recalculate\`. A topological ordering is a generic graph result, not a calculation schedule by itself.

### 4.6 Diagnostics and source spans

~~~rust
pub enum Severity {
    Info,
    Warning,
    Error,
}

pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
}

pub struct Span {
    // Validated half-open UTF-8 byte range.
}
~~~

- Diagnostic codes are stable and namespaced by the emitting owner: \`kernel.*\`, \`math.*\`, \`engine.*\`, \`table.*\`, or \`diagram.*\`.
- Consumers must not parse the human-readable message.
- Kernel provides the generic shape; each domain owns its diagnostic codes and meaning.
- Source spans use half-open UTF-8 byte offsets.
- UI adapters must convert UTF-8 byte offsets to JavaScript UTF-16 offsets before applying browser/editor selections.

### 4.7 Versioned envelope

~~~rust
pub struct Envelope<T> {
    pub schema_version: u32,
    pub result: Option<T>,
    pub diagnostics: Vec<Diagnostic>,
}
~~~

Envelope schema version and domain payload schema versions are separate concerns. Envelope v1 must retain its documented field meanings. Unsupported schema versions must fail explicitly; they must not be silently interpreted as the current version.

## 5. Math-facing facade contract

Math exposes mathematical analysis and evaluation operations. It does not expose Kernel storage or Engine scheduling.

The following is the proposed conceptual shape; concrete AST and value DTOs remain Math-owned:

~~~rust
pub struct AnalyzeRequest<'a> {
    pub source: &'a str,
    pub source_revision: Revision,
}

pub struct SymbolReference {
    pub name: String,
    pub span: Option<Span>,
}

pub struct MathAnalysis {
    pub references: Vec<SymbolReference>,
    pub diagnostics: Vec<Diagnostic>,
    // Math-owned AST or a separately versioned AST DTO.
}

pub struct EvaluateRequest<'a, C> {
    pub source: &'a str,
    pub source_revision: Revision,
    pub context: &'a C,
    pub preferred_display_unit: Option<String>,
}

pub struct MathEvaluation {
    pub source_revision: Revision,
    // Math-owned exact/approximate value and quantity representation.
    pub diagnostics: Vec<Diagnostic>,
}
~~~

The final facade must not expose Calcpad internals, \`Rc<RefCell<_>>\`, parser implementation types, generated WASM classes, or Kernel implementation types.

Math may return symbol references and typed evaluation results. Engine decides how references bind to worksheet nodes and how results enter the worksheet result lifecycle.

## 6. Recalculation trigger and execution lifecycle

**The Engine triggers and orchestrates worksheet recalculation. Kernel does not trigger Math, and Math does not schedule the worksheet.**

~~~text
Authored BlockSuite/Yjs change
          |
          v
Jabr-UI sends source + document/block identity
          |
          v
Jabr Engine advances its document/source revision
          |
          v
Math analyzes the changed expression
          |
          v
Engine resolves references and updates graph
          |
          v
Kernel graph returns generic topological order
          |
          v
Engine selects affected nodes and schedules evaluation
          |
          v
Engine builds an explicit input context for Math
          |
          v
Math evaluates and returns typed values + diagnostics
          |
          v
Engine validates revision and stages result snapshot
          |
          v
Engine publishes results through Kernel result primitives
          |
          v
Jabr-UI receives revisioned derived output and renders it
~~~

### Required lifecycle rules

1. An authored source change is observed by UI and submitted to Engine.
2. Engine owns document and worksheet revision advancement.
3. Math parses/analyzes source and reports expression-local references.
4. Engine resolves cross-block references and updates the generic graph.
5. Kernel provides generic graph operations and topological ordering.
6. Engine computes the affected-node set and schedules evaluation.
7. Engine supplies Math with an explicit immutable evaluation context.
8. Math returns a result proposal and diagnostics; it does not mutate Kernel state.
9. Engine rejects stale evaluations whose source/input revisions no longer match.
10. Engine commits or publishes the accepted result state through the generic Kernel contract.
11. UI renders the result as derived state; it does not persist calculated values in Yjs.

Kernel may expose generic graph/store primitives, but it does not emit a Math-specific recalculation event. No Math callback, observer, or event subscription is required in Kernel for this lifecycle.

## 7. Versioning and compatibility

- Version \`jabr-kernel-api\` independently from the \`jabr-kernel\` implementation and from the vendored Calcpad engine.
- Consumers pin a compatible API version; implementation upgrades must preserve that contract.
- Breaking public type, trait, serialization, or semantic changes require a major API version.
- Additive fields require explicit serde/default/unknown-field policy and compatibility tests.
- Keep Rust API types distinct from browser wire DTOs where JavaScript number/string semantics require adaptation.
- Preserve exact numeric values using tagged rational/decimal representations; do not serialize exact values as JSON floating-point numbers.
- Native and WASM adapters must pass the same versioned golden fixtures.

## 8. Acceptance tests

### Kernel API contract

- Public API crate builds independently with no Jabr domain dependencies.
- Kernel implementation depends on API; API does not depend on implementation.
- Compile-time tests verify public signatures and generic bounds.
- Graph tests verify deterministic prerequisite-first order, unknown-node errors, and cycle behavior.
- Result-store tests verify insert/replace/get/remove and opaque payload handling.
- Snapshot and envelope tests verify schema and round-trip behavior.
- Revision tests verify monotonic progression, overflow handling, and JSON precision.
- Diagnostic/span tests verify owner-namespaced codes and validated UTF-8 byte spans.

### Math consumer contract

- Math compiles using \`jabr-kernel-api\` without importing \`jabr-kernel\`.
- Math parser/evaluator tests run without a Kernel implementation.
- Math returns expression references with valid source spans.
- Math evaluation is deterministic for an explicit input context.
- Math does not mutate Kernel state or schedule other worksheet nodes.

### Engine integration contract

- Engine maps authored block IDs and Math references to generic NodeId/Port values.
- Engine constructs the cross-block graph using the public graph API.
- Engine schedules only affected nodes in prerequisite-first order.
- Engine rejects stale results.
- Engine publishes a complete revisioned result snapshot.
- A Kernel implementation can be replaced without changing Math or Engine source imports.

## 9. Current gaps and implementation sequence

1. Audit the existing Kernel implementation branch against this proposed API surface; record exactly which types are already present and which are only documented.
2. Freeze the API crate's module layout, signatures, error types, serde policy, and MSRV.
3. Create the dedicated \`jabr-kernel-api\` crate only after explicit implementation authorization.
4. Adapt \`jabr-kernel\` to implement the public contracts without adding Math-specific semantics.
5. Add compile-time consumer tests in Math and Engine.
6. Keep Engine orchestration as a separate subsystem; do not move its scheduler into Kernel or Math.
7. Integrate UI only after the Engine request/result contract is stable.

## 10. Explicit non-goals

- Implementing or modifying Kernel source code as part of this specification.
- Making Kernel a variable/scope database or Math evaluator.
- Moving worksheet dependency resolution or recalculation scheduling into Kernel.
- Making Math depend on Kernel implementation internals.
- Making UI own calculation semantics or persist derived results in Yjs.
- Creating a monorepo or cross-repository Cargo workspace.
- Treating a topological ordering as a complete recalculation scheduler.
