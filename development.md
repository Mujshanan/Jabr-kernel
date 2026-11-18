# Jabr-kernel development plan

**Architecture baseline:** 2026-11-18  
**Target:** Jabr Kernel 0.1  
**Status:** M0 repository alignment in progress; contract decisions remain open  
**Implementation branch:** `feat/kernel-core-implementation-0.1`

This plan implements the locked Jabr architecture without changing responsibility ownership. Updating this plan or completing M0 does not authorize engine integration, Orchestrator implementation, UI integration, or changes to other repositories.

## 1. Locked system architecture

Jabr is developed as independently maintained repositories with explicit contracts:

- **Kernel:** identifiers, ports, generic dependency-graph primitives, source spans/locations, diagnostics, and versioned envelopes.
- **Math:** mathematical values and quantities, units, expression syntax, parsing, AST, evaluation, mathematical types, scopes, and mathematical dependency semantics.
- **Table:** table document model, cell identity/content, formula text, and table-specific operations.
- **Diagram:** diagram document model, geometry, drawing/editing operations, and rendering semantics.
- **Orchestrator:** cross-engine graph construction from engine-provided information, evaluation scheduling, result storage, caching, and change publication.
- **UI:** unified document experience, worksheet composition, editing interactions, presentation, and UI-side document integration.

Dependency direction:

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

Kernel is domain-neutral and must not depend on Math, Table, Diagram, Orchestrator, or UI. Math, Table, and Diagram remain independently developable and testable. The Orchestrator composes engine capabilities. UI consumes the Orchestrator's application-facing contract and does not coordinate engines directly.

Do not introduce a monorepo or shared cross-repository Cargo workspace as part of Kernel 0.1.

## 2. Kernel 0.1 scope

Kernel owns exactly these six areas:

1. Identifiers.
2. Ports.
3. Generic dependency-graph primitives.
4. Source spans and locations.
5. Structured diagnostics.
6. Versioned communication envelopes.

### 2.1 Identifiers

- Provide stable, domain-neutral identifiers.
- Document validity, construction, equality, ordering, hashing, and serialization behavior.
- If JSON numbers are used, enforce the JavaScript-safe integer range; otherwise specify the wire representation.
- Kernel does not decide how engines map authored objects to identifiers.

### 2.2 Ports

- Provide generic endpoint identity associated with a Kernel identifier.
- Keep port meaning opaque.
- Do not encode worksheet, expression, table, diagram, input/output semantics, or engine-specific metadata.
- The owning engine defines how its ports are interpreted.

### 2.3 Generic dependency-graph primitives

- Provide a domain-neutral graph representation and generic operations for inspecting and mutating it.
- Kernel does not construct the cross-engine worksheet graph or interpret formulas/dependencies.
- Kernel does not own scheduling, recalculation policy, invalidation, result propagation, caching, or change publication.
- The Orchestrator constructs the graph from engine-provided dependency information and owns cross-engine evaluation coordination.
- Any generic graph algorithm must remain a data-structure operation with documented behavior. It must not become an application scheduler.

The current implementation exposes `DependencyGraph::topological_order`, which computes a deterministic prerequisite-first order and reports unresolved nodes when a cycle blocks ordering. The M0 audit recommends retaining this as a generic graph algorithm; explicit confirmation of that API boundary remains pending. The Orchestrator alone decides what to execute, when, and how.

### 2.4 Source spans and locations

- Provide source-independent span and location primitives.
- Define offset units, range convention, and line/column convention explicitly.
- The current `Span` uses UTF-8 byte offsets and a half-open range (start inclusive, end exclusive).
- Kernel validates generic span structure; the owning engine interprets spans against its source.
- Kernel does not implement parsers or domain-specific source mapping.
- A `Location` type is not yet implemented. Its representation and line/column convention require an explicit decision before implementation.

### 2.5 Diagnostics

- Provide a generic structured diagnostic representation.
- Machine-readable codes are stable public API and namespaced by the emitting crate (for example, `kernel.*`, `math.*`, `table.*`, and `diagram.*`).
- The emitting crate owns the meaning of its codes. The Orchestrator may transport and aggregate diagnostics but does not redefine domain semantics.
- Human-readable messages are not machine-readable identifiers and may evolve under the compatibility policy.
- Kernel does not encode domain-specific parser, table, diagram, or scheduling errors.

### 2.6 Versioned envelopes

- The initial transport format is JSON carried in a versioned `Envelope<T>`.
- The envelope provides an explicit schema version, an optional operation result, and diagnostics.
- Kernel owns the generic envelope shape and serialization contract, not operation execution or domain result types.
- Exact field names, optionality, compatibility behavior, error/result representation, and TypeScript type-generation tooling must be documented before being treated as stable.
- Do not add duplicate snapshot/revision protocols or engine-specific payload models.
- The current implementation writes schema version 1 but accepts arbitrary versions on deserialization. Whether unsupported versions are rejected at deserialization or validated by consumers remains an open contract decision.

## 3. Explicit exclusions

The following must not be implemented in Kernel 0.1:

- Variable/assignment state, scopes, namespaces, and state lifecycle.
- Revision counters, revision lifecycle, change tracking, snapshots, or state history.
- Calculation-result storage, invalidation, or caching.
- Cross-engine graph construction or dependency interpretation.
- Evaluation scheduling, recalculation loops, or orchestration.
- Change publication, subscriptions, or application event coordination.
- Mathematical values, quantities, units, parsing, AST, evaluation, or mathematical diagnostics.
- Table models, cells, formula text, or table-specific operations.
- Diagram models, geometry, drawing operations, or rendering.
- Worksheet/document ownership, BlockSuite/Yjs integration, UI behavior, or persistence policy.

An implementation convenience is not grounds for moving any excluded responsibility into Kernel.

## 4. M0 audit record — 2026-11-18

M0 is a contract and repository audit. It establishes scope and aligns the repository; it does not implement the missing `Location` type or finalize unresolved wire-contract decisions.

### 4.1 Repository inventory

| Path / API | Audit classification | M0 disposition |
| --- | --- | --- |
| `src/id.rs`: `NodeId`, `NodeIdAllocator`, `IdAllocationError`, `MAX_NODE_ID` | In scope | Retain |
| `src/port.rs`: `Port` | In scope | Retain |
| `src/graph.rs`: `DependencyGraph<N>`, `GraphError<N>` | In scope, API boundary to confirm | Retain generic graph operations; no scheduler policy |
| `src/span.rs`: `Span`, `SpanError` | In scope | Retain; UTF-8 byte offsets, half-open range |
| Location representation | In scope, not implemented | Contract decision pending |
| `src/diagnostic.rs`: `Diagnostic`, `Severity` | In scope | Retain; remove stale state-specific example |
| `src/envelope.rs`: `Envelope<T>`, `ENVELOPE_SCHEMA_VERSION` | In scope, wire compatibility incomplete | Retain; version policy pending |
| `src/revision.rs` | Out of scope | Remove from Kernel |
| `src/snapshot.rs` | Out of scope | Remove from Kernel |
| `src/result_store.rs` | Out of scope | Remove from Kernel |
| `tests/public_api.rs` | Mixed public API contract test | Remove excluded API usage; retain tests for in-scope API |
| `Cargo.toml` | In scope for dependency audit | No domain/application dependencies found |
| `.github/workflows/rust.yml` | In scope for CI audit | Existing format, test, Serde, Clippy, docs, and Rust 1.75 checks retained |
| `README.md` / `development.md` | In scope | Align to locked architecture and this audit |

No `Cargo.lock`, `rust-toolchain.toml`, or root `.gitignore` was present in the audited branch tree. The package declares Rust 1.75; CI checks that MSRV. The README must not claim that a toolchain file pins Rust.

### 4.2 Intended public API inventory

- **Identifiers:** `NodeId`, `NodeIdAllocator`, `IdAllocationError`, `MAX_NODE_ID`.
- **Ports:** `Port`.
- **Graph:** `DependencyGraph<N>`, `GraphError<N>`.
- **Spans and locations:** `Span`, `SpanError`, and a location representation after its convention is approved.
- **Diagnostics:** `Diagnostic`, `Severity`.
- **Envelopes:** `Envelope<T>`, `ENVELOPE_SCHEMA_VERSION`.

The exact stable API and wire compatibility rules remain subject to the open decisions below.

### 4.3 Out-of-scope removal

The M0 restructuring removes revision counters/lifecycle, revisioned snapshots, and calculation-result storage from the Kernel crate, including their module declarations, public exports, and references in the public API test. Their implementation is not moved to another repository by this change. No state-management, caching, scheduling, or orchestration replacement is introduced.

### 4.4 Open contract decisions

These are deliberately not inferred from the existing implementation:

1. **Location convention:** Should Kernel provide a `Location` type with one-based line and column? If so, are columns UTF-8 byte columns, Unicode scalar-value columns, or another explicitly defined unit?
2. **Envelope version compatibility:** Should deserialization reject unsupported schema versions, or should consumers deserialize the envelope and perform compatibility validation themselves?
3. **Graph algorithm boundary:** Confirm that `topological_order()` remains a generic graph operation only. It reports a valid prerequisite-first ordering; it does not decide which nodes Jabr executes, when execution occurs, parallelism, recalculation, retries, or result handling.

Until these are decided, M0 remains **in progress**. No ambiguous behavior is silently promoted to an approved contract.

## 5. Development milestones

### M0 — Contract and repository audit

- [x] Align the architecture description with the locked six-responsibility Kernel contract.
- [x] Inventory public modules, types, exports, dependencies, tests, and documentation.
- [x] Classify existing items as in-scope, out-of-scope, or requiring a contract decision.
- [x] Record the intended public API inventory.
- [x] Identify revision, snapshot, result-store, state-management, caching, scheduling, and change-publication code for exclusion.
- [x] Remove out-of-scope revision, snapshot, and result-store modules/exports/test usage from the M0 branch restructuring.
- [ ] Resolve the three open contract decisions in Section 4.4.
- [ ] Review and approve the completed M0 audit.

**Exit criteria:** the scope inventory is reviewed, the open contract decisions are resolved, and the repository's public structure and documentation agree with the approved six-area contract.

### M1 — Minimal Kernel primitives

Implement only:

- [ ] Identifiers and documented validity/serialization rules.
- [ ] Generic opaque ports.
- [ ] Generic dependency-graph representation and approved primitive operations.
- [ ] Source spans and locations with explicit offset conventions.
- [ ] Generic structured diagnostics and stable Kernel diagnostic-code conventions.
- [ ] Generic versioned envelopes and documented schema-version behavior.
- [ ] Keep dependencies minimal and domain-neutral.

**Exit criteria:** Kernel exposes only the six approved infrastructure areas and has no engine or application dependencies.

### M2 — Unit and public-contract tests

- [ ] Identifier construction, invalid values, range boundaries, ordering, hashing, and allocator edge cases.
- [ ] Port identity, equality, and serialization behavior.
- [ ] Graph insertion/removal, duplicate operations, unknown references, deterministic behavior, and cycle behavior for any included generic graph algorithm.
- [ ] Span boundaries, empty spans, reversed ranges, and documented offset conventions.
- [ ] Location convention and validation tests.
- [ ] Diagnostic structure, code namespace conventions, optional span, and serialization.
- [ ] Envelope schema version, result/diagnostic representation, serialization round-trip, and malformed/unsupported-version handling.
- [ ] External-consumer compile test using only the public Kernel API.
- [ ] Compile-time checks that examples use no Math, Table, Diagram, Orchestrator, or UI types.

**Exit criteria:** the six public contracts are independently testable, documented, and consumable without domain dependencies.

### M3 — Architectural and repository validation

- [ ] Audit Cargo dependencies and public exports for domain/application leakage.
- [ ] Confirm no revision, snapshot, result-storage, state-lifecycle, scheduling, caching, or change-publication responsibility remains in Kernel.
- [ ] Confirm Kernel does not parse/evaluate formulas or interpret engine dependency semantics.
- [ ] Confirm all public types have appropriate API documentation.
- [ ] Confirm the repository builds and tests independently without a shared workspace.
- [ ] Review the public API for accidental commitments beyond Kernel 0.1.

**Exit criteria:** implementation conforms to the architecture contract, not merely to its own unit tests.

### M4 — Release readiness and Kernel gate

- [ ] Run formatting checks.
- [ ] Run tests with default features.
- [ ] Run tests for every supported optional feature, including serialization.
- [ ] Run Clippy with warnings denied.
- [ ] Run rustdoc/documentation checks.
- [ ] Verify the declared Rust MSRV against actual code and CI.
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

A future end-to-end slice such as `x = 3 m` may then be planned: Math parses/evaluates; Kernel supplies only generic infrastructure; Orchestrator coordinates and publishes; UI renders. This future work does not authorize adding math, state lifecycle, result storage, or scheduling to Kernel.

## 7. CI and quality requirements

The Kernel CI validates the crate independently and includes, as applicable:

- `cargo fmt --all -- --check`
- `cargo test --all-targets`
- `cargo test --all-targets --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo doc --no-deps --all-features` with warnings denied
- A check against the declared MSRV

A passing local test run is useful evidence but does not replace required GitHub checks. Do not claim a milestone is verified until checks have actually completed successfully.

## 8. Change and approval policy

- Preserve the six Kernel responsibilities and ownership boundaries in this plan.
- Keep implementation changes limited to Kernel 0.1.
- Do not modify Math, Table, Diagram, Orchestrator, or UI as part of this phase.
- Do not merge an implementation PR merely because the documentation plan is updated.
- Any proposed change to ownership, dependency direction, public wire compatibility, or Kernel scope requires explicit architecture review before implementation.
- M4 is a gate, not automatic authorization to begin integration. Explicit Kernel milestone approval is required.

---

**Controlling rule:** Kernel first, and Kernel only. Complete and validate the six-area Kernel foundation before touching engine integration or Orchestrator implementation.
