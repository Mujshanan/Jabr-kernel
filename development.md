# Jabr-kernel development and architecture contract

**Contract status:** Approved architecture baseline for Jabr 0.1  
**Scope:** Architecture and interface responsibilities across the Jabr ecosystem  
**Implementation status:** This document records approved contracts; it does not assert that they are implemented.

This document is the authoritative development contract for Jabr-kernel. The eight decisions in the architecture review are approved. Any change to these boundaries should be proposed explicitly and reviewed before implementation.

## 1. System architecture

Jabr is developed as a set of cooperating, independently maintained projects:

- **Jabr-kernel:** shared, domain-neutral Rust infrastructure.
- **Jabr Math (currently Jabr-cli):** mathematical language and calculation engine.
- **Jabr Diagram (Jabr-draw):** diagram model, geometry, drawing operations, and rendering.
- **Jabr-table:** structured table model and cell operations.
- **Jabr API:** cross-engine orchestration and the application-facing contract.
- **Jabr-UI:** the complete TypeScript application UI, built around AFFiNE/BlockSuite.

The intended dependency direction is:

```text
                    Jabr-kernel
                   /     |     \
                  v      v      v
                Math    Draw   Table
                   \     |     /
                    v    v    v
                      Jabr API
                         |
                         v
                      Jabr-UI
```

The arrows indicate dependency direction: Math, Draw, and Table depend on Kernel; the API composes the engines; the UI depends on the API boundary. Kernel must not depend on any engine or application layer.

### Repository organization

Independent repositories remain the initial development model. Each Rust project must remain independently developable and testable. Do not consolidate repositories into a monorepo or introduce a shared Cargo workspace as part of this contract. Cross-repository integration should be achieved through explicit, versioned crate/API contracts.

## 2. Approved ownership contracts

### 2.1 Jabr-kernel

Kernel owns shared state-management infrastructure:

- Variable/state identity and creation.
- Assignment/state updates as state transitions.
- Revisions and change tracking.
- Snapshots and state history needed to preserve prior states.
- Generic dependency-graph primitives.

Kernel is responsible for state lifecycle and generic infrastructure, not the meaning of mathematical expressions. It must not parse or evaluate formulas, define mathematical types, interpret units, or decide mathematical dependency semantics.

Kernel must remain domain-neutral. In particular, it must not take dependencies on Math, Draw, Table, Jabr API, or Jabr-UI.

The concrete representation of engine-specific values inside shared state must preserve this boundary: Kernel must not acquire ownership of Math's `Value` or `Quantity` model merely to store or version state. The precise payload/serialization interface is an implementation-level contract to specify without moving mathematical semantics into Kernel.

### 2.2 Jabr Math (currently Jabr-cli)

Math owns mathematical meaning and calculation:

- Mathematical `Value` and `Quantity` models.
- Units, dimensions, conversions, and display-unit behavior.
- Expression syntax, parsing, and evaluation.
- Mathematical type rules, scopes, and dependency interpretation.
- Evaluation diagnostics and mathematical error semantics.

Math evaluates right-hand sides and produces mathematical results. Kernel manages shared state transitions and generic graph structures; it does not become the evaluator.

### 2.3 Jabr Diagram (Jabr-draw)

Diagram owns its diagram document model, geometry, drawing/editing operations, and rendering semantics. It may use Kernel's genuinely shared primitives but must not make Kernel responsible for diagram-domain concepts.

### 2.4 Jabr-table

Table owns table structure and cell identity/content, including formula text. A table cell's formula is stored by Table; Math parses and evaluates that formula. Kernel may provide generic dependency-graph primitives, but it does not interpret the formula or determine mathematical dependencies.

### 2.5 Jabr API

The Jabr API owns cross-engine orchestration, including the recalculation loop. It coordinates Table, Math, Kernel, and other engine capabilities without moving their domain responsibilities into the API or Kernel.

The API is the application-facing boundary. Its external contract must be explicit and versioned. The TypeScript UI should consume this boundary rather than importing engine internals directly.

### 2.6 Jabr-UI

Jabr-UI owns the complete product experience: worksheet composition, AFFiNE/BlockSuite integration, custom blocks, editing interactions, and presentation. It remains a TypeScript application and does not own Rust engine semantics.

## 3. Approved cross-cutting contracts

### 3.1 Wire format

The initial wire format is JSON carried inside a versioned `Envelope`.

The envelope schema must make its version explicit and provide a stable place for operation results and diagnostics. Exact field names, compatibility policy, and TypeScript type-generation tooling remain schema-design work; they must not be silently assumed by individual engines.

### 3.2 Diagnostics

Diagnostic codes are part of the public API and must be stable. Codes are namespaced by their owning crate:

- `kernel.*`
- `math.*`
- `draw.*`
- `table.*`

The owning crate defines the meaning of its codes. Cross-engine orchestration may transport and aggregate diagnostics but must not redefine their domain meaning. Human-readable messages may evolve independently of stable machine-readable codes, subject to the eventual compatibility policy.

### 3.3 Dependency graph

Kernel provides generic graph primitives only. Math owns the interpretation of mathematical dependencies. The API coordinates recalculation across engines using the graph and engine-provided information. Table owns cell identity and formula text, not the formula evaluator.

### 3.4 Value and unit ownership

Math owns `Value`, `Quantity`, dimensions, unit conversions, and display-unit behavior. These concepts are not Kernel primitives. Shared state storage must accommodate engine-owned payloads without requiring Kernel to understand their semantics.

## 4. First vertical slice: `x = 3 m`

The first end-to-end integration slice is deliberately narrow:

1. The user enters `x = 3 m` in Jabr-UI.
2. Math parses the expression and identifies the assignment and mathematical dependencies.
3. Kernel supplies the relevant shared identity and state-management operations; graph relationships use Kernel's generic primitives.
4. Math evaluates the right-hand side and produces its `Value::Quantity` result.
5. The Jabr API coordinates the operation and packages the result and any diagnostics in a versioned JSON `Envelope`.
6. Jabr-UI receives the API response and renders the value and unit.

### Acceptance criteria

- The expression is parsed and evaluated by Math, not Kernel.
- The resulting quantity and unit semantics are owned by Math.
- Kernel records the appropriate state transition and revision.
- A captured snapshot remains unchanged after a later update to the variable.
- The API response uses the versioned JSON Envelope.
- Diagnostic codes retain their owning crate namespace.
- The UI uses the API contract and does not depend on internal Rust engine types.

Only infrastructure required to support this vertical slice should be added to Kernel. Do not expand Kernel into a general calculation engine.

## 5. Development and compatibility rules

- Preserve the dependency direction and ownership boundaries in this document.
- Keep each repository independently buildable and testable.
- Document public APIs and externally observable behavior.
- Treat diagnostic codes and the versioned Envelope as public compatibility surfaces.
- Add cross-repository contract tests at integration boundaries, while retaining unit tests in each owning repository.
- Prefer minimal dependencies and explicit types.
- Changes that alter ownership, dependency direction, state/snapshot semantics, or wire compatibility require an explicit architecture-contract review before implementation.

## 6. Deferred design details

The following details are intentionally not decided by the eight approved architecture decisions. They must be specified when designing the corresponding interfaces, without changing the approved ownership boundaries:

- Exact Kernel state payload representation and how engine-owned values are carried without Kernel owning their types.
- Exact identifiers, revision granularity, snapshot retention/history policy, and conflict/stale-update semantics.
- Dependency-graph API shape, edge metadata, invalidation semantics, and cycle handling.
- Exact JSON Envelope fields, error/result representation, schema evolution policy, and generated TypeScript type tooling (including whether to use `tsify` or `ts-rs`).
- Exact diagnostic structure, source-span representation, and compatibility policy for diagnostic details.
- WASM/Tauri/native transport adapters and their implementation details.
- Cargo crate packaging and release/version coordination across independent repositories.

Deferred details are not permission to change ownership. For example, deciding how to serialize a Math quantity does not make Kernel the owner of the quantity model.

## 7. Explicit non-goals for this contract

- No Kernel mathematical parser or evaluator.
- No Kernel ownership of `Value`, `Quantity`, units, or display-unit semantics.
- No Kernel-specific Table, Diagram, or UI domain model.
- No cross-engine recalculation loop inside Kernel.
- No repository migration or monorepo conversion.
- No claim that the architecture described here has already been implemented.

---

**Approval record:** The eight architecture decisions reviewed on 2026-11-18 were approved: Kernel state lifecycle; Math ownership of mathematical values and semantics; generic Kernel dependency-graph primitives; Table ownership of formula text/cell identity with Math evaluation; API-owned cross-engine recalculation; versioned JSON Envelope; stable crate-namespaced diagnostics; and independent repositories as the initial development model.
