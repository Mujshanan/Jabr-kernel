# Jabr-kernel development plan

**Architecture baseline:** 2026-11-18  
**Target:** Jabr Kernel 0.1  
**Status:** Implementation plan aligned with the locked architecture contract  
**Implementation branch:** `feat/kernel-core-implementation-0.1`

This document translates the approved Jabr architecture into an implementation and validation plan for Kernel 0.1. The architecture ownership boundaries are fixed for this milestone. This plan does not authorize changes to those boundaries, engine integration, Orchestrator implementation, or UI integration.

## 1. Locked system architecture

Jabr is developed as independently maintained repositories with explicit, versioned contracts:

- **Kernel:** identifiers, ports, generic dependency-graph primitives, source spans, diagnostics, and versioned envelopes.
- **Math:** mathematical values and quantities, units, expression syntax, parsing, AST, evaluation, mathematical types, scopes, and mathematical dependency semantics.
- **Table:** table document model, cell identity and content, formula text, and table-specific operations.
- **Diagram:** diagram document model, geometry, drawing/editing operations, and rendering semantics.
- **Orchestrator:** graph construction from engine-provided information, cross-engine evaluation scheduling, result storage, caching, and change publication.
- **UI:** unified document experience, worksheet composition, editing interactions, presentation, and UI-side document integration.

The dependency direction is:

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

Kernel must remain domain-neutral and must not depend on Math, Table, Diagram, Orchestrator, or UI. Math, Table, and Diagram remain independently developable and testable. The Orchestrator composes their capabilities; the UI consumes the Orchestrator's application-facing contract rather than importing engine internals directly.

Independent repositories remain the development model. Do not introduce a monorepo or shared cross-repository Cargo workspace as part of Kernel 0.1.

## 2. Kernel 0.1 scope

Kernel owns exactly these six areas:

1. Identifiers.
2. Ports.
3. Generic dependency-graph primitives.
4. Source spans and locations.
5. Structured diagnostics.
6. Versioned communication envelopes.

These are shared infrastructure contracts, not domain engines or application workflows.

### 2.1 Identifiers

- Provide stable, domain-neutral identifiers suitable for references in shared infrastructure.
- Define and document validity, construction, equality, ordering, hashing, and serialization behavior where applicable.
- If identifiers are serialized as JSON numbers, enforce a range that JavaScript can represent exactly; otherwise explicitly specify the chosen wire representation.
- Kernel does not decide how an engine maps authored blocks, table cells, diagram objects, or mathematical expressions to identifiers.

### 2.2 Ports

- Provide generic endpoint identity associated with a Kernel identifier.
- Keep port meaning opaque to Kernel.
- Do not encode worksheet, expression, table, diagram, input/output semantics, or engine-specific metadata into the shared port type unless a future architecture review explicitly approves it.
- The owning engine defines how its ports are interpreted.

### 2.3 Generic dependency-graph primitives

- Provide a domain-neutral graph representation and only the generic operations required to inspect and mutate that representation.
- Kernel must not construct the cross-engine worksheet graph from Math, Table, or Diagram content.
- Kernel must not interpret formulas or determine mathematical dependencies.
- Kernel must not own evaluation scheduling, recalculation order as an application policy, invalidation policy, result propagation, caching, or change publication.
- The Orchestrator constructs the graph from engine-provided dependency information and owns evaluation scheduling and cross-engine coordination.
- Any generic graph algorithms included in Kernel must remain data-structure operations, with their behavior and limits explicitly documented. They must not silently become an evaluation scheduler.

### 2.4 Source spans and locations

- Provide source-independent span and location primitives.
- Define offset units, range convention, and line/column convention explicitly.
- The initial text-span convention is a half-open range: start inclusive, end exclusive. If offsets are byte-based, document the encoding and ensure consumers use the same convention.
- Kernel validates generic span structure; the owning engine interprets spans against its source document or expression text.
- Kernel does not implement parsers or domain-specific source mapping.

### 2.5 Diagnostics

- Provide a generic structured diagnostic representation.
- Machine-readable codes are stable public API. Codes are namespaced by their owning crate, for example `kernel.*`, `math.*`, `table.*`, and `diagram.*`.
- The crate that emits a code owns its meaning. The Orchestrator may transport and aggregate diagnostics but must not redefine domain semantics.
- Human-readable messages are not machine-readable identifiers and may evolve under the compatibility policy.
- Kernel must not encode Math parser errors, Table validation rules, Diagram rendering errors, or Orchestrator scheduling policy as Kernel-owned domain concepts.

### 2.6 Versioned envelopes

- The initial transport format is JSON carried in a versioned `Envelope`.
- The envelope provides an explicit schema version and a stable place for operation results and diagnostics.
- Kernel owns the generic envelope shape and its serialization contract; it does not own operation execution or domain result types.
- Exact field names, optionality, compatibility behavior, error/result representation, and TypeScript type-generation tooling must be explicitly documented before being treated as stable.
- Do not add duplicate snapshot/revision protocols or engine-specific payload models to the Kernel envelope.

## 3. Explicit exclusions

The following are not Kernel responsibilities and must not be implemented in Kernel 0.1:

- Variable or assignment state, scopes, namespaces, and state lifecycle.
- Revision counters, revision lifecycle, change tracking, snapshots, or state history.
- Calculation-result storage, result invalidation, or result caching.
- Cross-engine graph construction or dependency interpretation.
- Evaluation scheduling, recalculation loops, or orchestration.
- Change publication, subscriptions, or application event coordination.
- Mathematical values, quantities, units, parsing, AST, evaluation, or mathematical diagnostics.
- Table models, cells, formula text, or table-specific operations.
- Diagram models, geometry, drawing operations, or rendering.
- Worksheet/document ownership, BlockSuite/Yjs integration, UI behavior, or persistence policy.

Ownership of these excluded responsibilities remains with the relevant domain engine, Orchestrator, or UI as defined in Section 1. An implementation convenience is not grounds for moving one of these responsibilities into Kernel.

## 4. Implementation audit and required alignment

The implementation must be audited against the six-area scope above. The presence of code in the branch does not make that code part of the approved Kernel contract.

The branch was previously observed to contain additional modules/types for revisions, snapshots, and result storage. These are outside the locked Kernel scope:

| Existing or proposed item | Required disposition |
| --- | --- |
| Identifier and allocator | Retain if they satisfy the identifier contract |
| Port | Retain if it remains generic and opaque |
| Dependency graph | Retain only generic graph representation and operations |
| Span and location | Retain if offset and location conventions are explicit |
| Diagnostic | Retain as a generic structured diagnostic |
| Envelope | Retain as a generic versioned envelope |
| Revision / revision lifecycle | Remove from Kernel's public API and Kernel 0.1 scope |
| Snapshot / snapshot history | Remove from Kernel's public API and Kernel 0.1 scope |
| ResultStore / calculation-result storage | Remove from Kernel's public API and Kernel 0.1 scope |
| Scheduling, caching, change publication | Do not implement in Kernel; Orchestrator owns these |

Before Kernel 0.1 can pass architectural validation, remove out-of-scope modules, exports, tests, documentation claims, and dependencies that exist solely to support those excluded responsibilities. Do not move their implementation into another repository as part of this Kernel-only milestone; that would be separate work and requires its own authorization.

## 5. Development milestones

### M0 — Contract and repository audit

- [ ] Confirm this plan matches the locked six-responsibility Kernel contract.
- [ ] Inventory all public modules, types, exports, dependencies, tests, and documentation.
- [ ] Classify each item as in-scope, out-of-scope, or requiring an explicit contract decision.
- [ ] Record the exact public API expected for the six in-scope areas.
- [ ] Identify all revision, snapshot, state-management, result-store, caching, scheduling, and change-publication code for exclusion from Kernel 0.1.

**Exit criteria:** a reviewed scope inventory exists; no ambiguous ownership is silently treated as approved.

### M1 — Minimal Kernel primitives

Implement only:

- [ ] Identifiers and their documented validity/serialization rules.
- [ ] Generic opaque ports.
- [ ] Generic dependency-graph representation and approved primitive operations.
- [ ] Source spans and locations with explicit offset conventions.
- [ ] Generic structured diagnostics and stable Kernel diagnostic-code conventions.
- [ ] Generic versioned envelopes and their documented schema-version behavior.

- [ ] Remove out-of-scope revision, snapshot, result-store, state-management, and orchestration APIs from Kernel's public surface.
- [ ] Keep dependencies minimal and domain-neutral.

**Exit criteria:** Kernel exposes only the six approved infrastructure areas and has no engine or application dependencies.

### M2 — Unit and public-contract tests

- [ ] Identifier construction, invalid values, range boundaries, ordering, hashing, and allocator edge cases.
- [ ] Port identity, equality, and serialization behavior.
- [ ] Graph insertion/removal, duplicate operations, unknown references, deterministic behavior, and cycle behavior for any included generic graph algorithm.
- [ ] Span boundaries, empty spans, reversed ranges, and documented offset conventions.
- [ ] Diagnostic structure, code namespace conventions, optional span, and serialization.
- [ ] Envelope schema version, result/diagnostic representation, serialization round-trip, and malformed/unsupported-version handling where applicable.
- [ ] External-consumer compile test using only the public Kernel API.
- [ ] Compile-time checks that examples use no Math, Table, Diagram, Orchestrator, or UI types.

**Exit criteria:** the six public contracts are independently testable, documented, and consumable without domain dependencies.

### M3 — Architectural and repository validation

- [ ] Audit Cargo dependencies and public exports for domain/application leakage.
- [ ] Confirm no revision, snapshot, result-storage, state-lifecycle, scheduling, caching, or change-publication responsibility remains in Kernel.
- [ ] Confirm Kernel does not parse or evaluate formulas or interpret engine dependency semantics.
- [ ] Confirm all public types have appropriate API documentation.
- [ ] Confirm the repository builds and tests independently without a shared workspace.
- [ ] Review the public API for accidental commitments beyond Kernel 0.1.

**Exit criteria:** the implementation conforms to the architecture contract, not merely to its own unit tests.

### M4 — Release readiness and Kernel gate

- [ ] Run formatting checks.
- [ ] Run tests with default features.
- [ ] Run tests for every supported optional feature, including serialization.
- [ ] Run Clippy with warnings denied.
- [ ] Run rustdoc/documentation checks.
- [ ] Verify the declared Rust MSRV against the actual code and CI toolchain.
- [ ] Confirm a clean build and clean working tree.
- [ ] Confirm all required GitHub checks pass.
- [ ] Publish a concise Kernel 0.1 API and compatibility note after review.

**Exit criteria:** Kernel 0.1 is reviewed, validated, and explicitly approved as complete.

## 6. Integration gates — not part of this implementation phase

No engine integration or Orchestrator implementation begins until M0–M4 are complete and the Kernel milestone is explicitly approved.

After that approval, integration planning may define separate work for:

1. Engine-consumer contract tests for Math, Table, and Diagram.
2. Orchestrator graph construction from engine-provided dependency information.
3. Orchestrator evaluation scheduling, result storage, caching, and change publication.
4. UI consumption of the Orchestrator's application-facing contract.

The first end-to-end slice may then be `x = 3 m`: Math parses and evaluates the expression; Kernel supplies only the generic infrastructure it owns; Orchestrator coordinates the operation and publishes the result through the versioned envelope; UI renders the response. This future slice does not authorize adding mathematical, state-lifecycle, result-storage, or scheduling responsibilities to Kernel.

## 7. CI and quality requirements

The Kernel CI should validate the crate independently and should include, as applicable:

- `cargo fmt --all -- --check`
- `cargo test --all-targets`
- `cargo test --all-targets --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo doc --no-deps` (with warnings treated according to the documented policy)
- A check against the declared MSRV.

The CI matrix and dependency versions must be explicit. A passing local test run is useful evidence but does not replace required GitHub checks. Do not claim the milestone is verified until the relevant checks have actually completed successfully.

## 8. Change and approval policy

- Preserve the six Kernel responsibilities and the ownership boundaries in this plan.
- Keep implementation changes limited to Kernel 0.1.
- Do not modify Math, Table, Diagram, Orchestrator, or UI as part of this phase.
- Do not merge an implementation PR merely because the documentation plan is updated.
- Any proposed change to ownership, dependency direction, public wire compatibility, or Kernel scope requires explicit architecture review before implementation.
- Completion of M4 is a gate, not automatic authorization to begin integration. Explicit Kernel milestone approval is required.

---

**Controlling rule:** Kernel first, and Kernel only. Complete and validate the six-area Kernel foundation before touching engine integration or Orchestrator implementation.
