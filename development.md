# Jabr-kernel development plan

**Architecture baseline:** 2026-11-18  
**Target:** Jabr Kernel 0.1  
**Status:** M1 implementation and M2 public-contract tests committed; M2 CI validation passed  
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

The current implementation exposes `DependencyGraph::topological_order`, which computes a deterministic prerequisite-first order and reports unresolved nodes when a cycle blocks ordering. **M0 decision: retain it as a generic graph algorithm.** It reports graph structure only; the Orchestrator alone decides what to execute, when, and how.

### 2.4 Source spans and locations

- Provide source-independent span and location primitives.
- Define offset units, range convention, and line/column convention explicitly.
- The current `Span` uses UTF-8 byte offsets and a half-open range (start inclusive, end exclusive).
- Kernel validates generic span structure; the owning engine interprets spans against its source.
- Kernel does not implement parsers or domain-specific source mapping.
- **M0 decision:** `Location` is a source coordinate with one-based `line` and one-based `column`; `column` counts Unicode scalar values from the start of the line (not UTF-8 bytes, UTF-16 code units, grapheme clusters, or visual tab width). `Span` remains the canonical UTF-8 byte-based, half-open source range. The owning engine derives `Location` from its source text; Kernel does not perform source mapping.
- The `Location` type itself is an M1 implementation item.

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
- **M0 decision:** deserialization rejects any `schema_version` other than `ENVELOPE_SCHEMA_VERSION` (currently `1`). Consumers must not silently accept unknown future or unsupported historical schema versions. Compatibility with another version requires an explicit version-specific adapter or migration path.
- The current derive-based deserializer does not yet enforce this rule; implementing validation is an M1 item.

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

M0 is a contract and repository audit. It establishes scope, resolves contract decisions, and aligns the repository. It does not implement the `Location` type or custom envelope deserialization; those are M1 implementation work.

### 4.1 Repository inventory

| Path / API | Audit classification | M0 disposition |
| --- | --- | --- |
| `src/id.rs`: `NodeId`, `NodeIdAllocator`, `IdAllocationError`, `MAX_NODE_ID` | In scope | Retain |
| `src/port.rs`: `Port` | In scope | Retain |
| `src/graph.rs`: `DependencyGraph<N>`, `GraphError<N>` | In scope | Retain generic graph operations, including `topological_order`; no scheduler policy |
| `src/span.rs`: `Span`, `SpanError` | In scope | Retain; UTF-8 byte offsets, half-open range |
| `src/location.rs`: `Location`, `LocationError` | In scope | Implemented in M1; private validated fields, one-based line and Unicode-scalar column |
| `src/diagnostic.rs`: `Diagnostic`, `Severity` | In scope | Retain; remove stale state-specific example |
| `src/envelope.rs`: `Envelope<T>`, `ENVELOPE_SCHEMA_VERSION` | In scope | M1 custom Serde deserializer rejects schema versions other than the current version |
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
- **Spans and locations:** `Span`, `SpanError`, `Location`, and `LocationError`; `Location` has private `u32` fields and a validated constructor with one-based Unicode-scalar coordinates.
- **Diagnostics:** `Diagnostic`, `Severity`.
- **Envelopes:** `Envelope<T>`, `ENVELOPE_SCHEMA_VERSION`.

The M0 inventory records the intended public API. M1 now implements `Location` and strict envelope-version deserialization; current implementation details are recorded below.

### 4.3 Out-of-scope removal and behavior-loss inventory

The M0 restructuring removed `revision.rs`, `snapshot.rs`, and `result_store.rs`, their `lib.rs` module declarations/re-exports, and their use in the public API integration test. This is consistent with the locked Kernel exclusions. No implementation or test migration to another repository was performed.

The removal did eliminate useful, self-contained behavior and regression coverage from this repository:

| Removed area | Behavior removed | Tests removed from Kernel |
| --- | --- | --- |
| `Revision` | Checked monotonic increment; overflow error; decimal-string Serde representation preserving full `u64` precision | 3 unit tests: increment/order, overflow, full-range Serde round-trip |
| `Snapshot<T>` | Owned payload associated with a revision; payload mapping while preserving revision | 2 unit tests: payload/clone independence and Serde round-trip inside `Envelope` |
| `ResultStore<V>` | Deterministic `NodeId`-keyed opaque values; get/insert/replace/remove/contains/len/is_empty/iteration | 1 unit test covering insert/replace/get/remove/contains/len |
| `tests/public_api.rs` | Former integration example composed `ResultStore`, `Revision`, `Snapshot`, and `Envelope` | Replaced by an in-scope API composition test; old integration scenario no longer exists |

These are intentional removals from Kernel, not evidence that the behaviors are worthless. Their future ownership and any test porting must be decided in the responsible repository/workstream. Do not reintroduce them into Kernel or silently migrate them as part of M0.

### 4.4 Retained public API verification

The retained API was compared with its source implementations at the pre-restructure commit and re-fetched from the post-restructure branch. The new public API test uses existing constructors and methods; it does not invent or assume an API that is absent from the source.

| Public API | Verified implementation contract | Current test coverage / gaps |
| --- | --- | --- |
| `NodeId`, `NodeIdAllocator`, `IdAllocationError`, `MAX_NODE_ID` | Nonzero `u64` constrained to JavaScript-safe integer range; checked construction; allocator clones share an atomic sequence; `new`, `get`, `allocate`, `Display` | Tests zero/above-max rejection, clone sequence, unsafe Serde rejection; add max-valid and allocator exhaustion boundary tests in M2 |
| `Port` | Public `node_id: NodeId` and opaque zero-based `index: u32`; `Port::new`; optional Serde | Equality/identity test; serialization coverage should be added in M2 |
| `DependencyGraph<N>`, `GraphError<N>` | `N: Ord`; add/remove node, add dependency, deterministic prerequisite-first topological order, len/empty; cycle error includes all unresolved nodes (including downstream blocked nodes) | Tests deterministic order, unknown node, and cycle-blocked nodes; add remove/idempotency/len/empty/duplicate-edge tests in M2 |
| `Span`, `SpanError` | Private UTF-8 byte offsets; `new` rejects reversed ranges; start inclusive/end exclusive; `len` and `is_empty`; optional Serde validates reversed input | Tests normal/empty/reversed ranges and Serde round-trip/rejection |
| `Location`, `LocationError` | Private one-based `u32` line and Unicode-scalar column; `new` rejects zero; accessors `line()` and `column()`; optional Serde validates coordinates | Tests zero boundaries, valid coordinates, and Serde round-trip/rejection |
| `Diagnostic`, `Severity` | Public `code`, `severity`, `message`, optional `span`; `new` and `with_span`; `Info`/`Warning`/`Error`; optional Serde | Code/message separation and Serde round-trip with span |
| `Envelope<T>`, `ENVELOPE_SCHEMA_VERSION` | Public `schema_version`, optional `result`, `diagnostics`; `success`, `failure`, `with_diagnostic`; schema constant currently `1`; custom Serde deserializer rejects every other version | Tests explicit version, JSON round-trip, and rejection of both older and future versions |

`src/lib.rs` declares and re-exports only the six in-scope areas, including the M1 `Location` and `LocationError` types. The M0 restructuring itself did not alter retained API behavior; M1 adds the validated location API and strict envelope deserialization.

### 4.5 Resolved contract decisions

1. **Location convention — resolved and implemented in M1:** `Location` uses private `u32` line and column fields with a validated `new` constructor and read-only accessors. Both coordinates are one-based; column counts Unicode scalar values from the beginning of the line. It is a human/source coordinate, not a byte offset, UTF-16 index, grapheme count, or rendered visual column. `Span` remains the canonical UTF-8 byte range. Engines derive locations from their source text; Kernel only represents and validates the coordinate.
2. **Envelope schema compatibility — resolved and implemented in M1:** `Envelope<T>` deserialization rejects every schema version other than the current `ENVELOPE_SCHEMA_VERSION`. Do not silently accept future or unsupported historical versions. Version migration belongs in an explicit adapter.
3. **Graph algorithm boundary — resolved:** retain `DependencyGraph::topological_order()` as a generic graph algorithm. It returns deterministic prerequisite-first order or unresolved nodes when a cycle blocks ordering. It does not identify changed/dirty nodes, choose recalculation scope, schedule work, choose parallelism, invoke engines, retry, store results, or publish changes. Those are Orchestrator responsibilities.

These decisions close the M0 contract questions. The user has authorized M1 implementation; they do not authorize engine/Orchestrator integration.

## 5. Development milestones

### M0 — Contract and repository audit

- [x] Align the architecture description with the locked six-responsibility Kernel contract.
- [x] Inventory public modules, types, exports, dependencies, tests, and documentation.
- [x] Classify existing items as in-scope, out-of-scope, or requiring a contract decision.
- [x] Record the intended public API inventory.
- [x] Identify revision, snapshot, result-store, state-management, caching, scheduling, and change-publication code for exclusion.
- [x] Remove out-of-scope revision, snapshot, and result-store modules/exports/test usage from the M0 branch restructuring.
- [x] Resolve and record the three contract decisions in Section 4.5.
- [x] Compare the restructuring against the pre-change source and verify retained public APIs against their implementations.
- [x] Record behavior and tests removed with the excluded modules.
- [x] Review the M0 audit against the locked architecture.
- [x] User authorized proceeding to M1 on 2026-11-18; this does not authorize engine/Orchestrator integration.

**Technical exit criteria:** the scope inventory is reviewed; contract decisions are resolved; excluded code is absent; retained APIs are verified; and repository documentation agrees with the six-area contract. These criteria are met on this branch. User acceptance is tracked separately and is not inferred from CI.

### M1 — Minimal Kernel primitives

Implement only:

- [x] Identifiers and documented validity/serialization rules (retained and audited in M0).
- [x] Generic opaque ports (retained and audited in M0).
- [x] Generic dependency-graph representation and approved primitive operations (retained and audited in M0).
- [x] Source spans and locations with explicit offset conventions; add validated one-based Location.
- [x] Generic structured diagnostics and stable owner-namespaced Kernel diagnostic-code conventions.
- [x] Generic versioned envelopes; reject unsupported schema versions during Serde deserialization.
- [x] Keep dependencies minimal and domain-neutral; no engine or application dependencies introduced.

**Implementation:** src/location.rs adds Location and LocationError; src/envelope.rs replaces derived deserialization with version-checked deserialization. The public API integration test now compiles against Location. Unit tests cover invalid zero coordinates, valid coordinates, valid envelope round-trips, and rejection of older and future envelope versions.

**Validation:** Passed on the M1 branch: formatting, default tests, Serde tests, Clippy with warnings denied, documentation build with warnings denied, and Rust 1.75 all-feature check. GitHub Actions run [44351582992](https://github.com/Mujshanan/Jabr-kernel/actions/runs/44351582992) completed successfully.

**Exit criteria:** Kernel exposes only the six approved infrastructure areas, enforces the documented source-location and envelope-version contracts, and has no engine or application dependencies.

### M2 — Unit and public-contract tests

**Implementation:** expanded `tests/public_api.rs` into an external-consumer contract suite using only public Kernel exports. Coverage includes identifier validity and maximum-range allocation, display/order/hash semantics, cloned and concurrent allocator behavior, port identity, graph mutation/deduplication/removal/unknown references/deterministic ordering/cycles, span and location boundaries, diagnostic structure, envelope success/failure, and feature-gated Serde wire behavior including malformed inputs and unsupported versions. No runtime dependency or domain type was added.

**Validation:** GitHub Actions push run [44362880839](https://github.com/Mujshanan/Jabr-kernel/actions/runs/44362880839) completed successfully on commit `7cbdfeb78069231d5ff7a16b5c0f1881117a5f01`. Formatting, default-feature tests, Serde-feature tests, Clippy, documentation build, and the Rust 1.75 all-feature check all passed. An initial test compile failure was caused by importing `MAX_NODE_ID` from the crate root rather than its existing public `jabr_kernel::id` module; the test import was corrected without changing the production API. A subsequent formatting-only import-order failure was also corrected. M2 test coverage and CI validation are complete; this does not constitute M0/M1/M2 user acceptance or authorize integration.

### M3 — Architectural and repository validation

**Audit result:** the M3 architecture and repository audit is complete. No production code or public API was changed.

- [x] Audit Cargo dependencies and public exports for domain/application leakage.
- [x] Confirm no revision, snapshot, result-storage, state-lifecycle, scheduling, caching, or change-publication responsibility remains in Kernel.
- [x] Confirm Kernel does not parse/evaluate formulas or interpret engine dependency semantics.
- [x] Confirm all public types have appropriate API documentation.
- [x] Confirm the repository builds and tests independently without a shared workspace.
- [x] Review the public API for accidental commitments beyond Kernel 0.1.

**Audit evidence (M3 audit commit: `d97fbf5166068cc1b63683b8f33c3760f814969a`):**

1. **Package and dependency boundary:** the repository is a single Rust package, not a Cargo workspace. `Cargo.toml` has one optional runtime dependency (`serde`, enabled only by the `serde` feature) and one development-only dependency (`serde_json`). No engine, application, UI, BlockSuite, Yjs, or orchestration dependency is present. The declared MSRV is Rust 1.75 and CI checks it.
2. **Public surface:** `src/lib.rs` exposes only the approved infrastructure modules: `diagnostic`, `envelope`, `graph`, `id`, `location`, `port`, and `span`. Root re-exports are limited to the corresponding public types/constants. The extra module count reflects that source locations are split into `Span` and `Location`; it does not add a seventh responsibility.
3. **Responsibility scan:** the source tree contains no revision, snapshot, result-store, variable/assignment state, scope, history, cache, scheduler, cross-engine graph builder, formula parser/evaluator, or change-publication implementation. Historical mentions of removed modules occur only in the audit/development documentation.
4. **Domain neutrality:** graph nodes are generic; ports carry only a node ID and opaque index; spans/locations carry source coordinates; diagnostics carry generic code/severity/message/span; and envelopes carry a generic payload. No module interprets mathematical, table, or diagram semantics.
5. **API documentation:** public modules, types, fields, constructors, methods, enum variants, and error variants have rustdoc descriptions. The crate keeps `missing_docs = "warn"`; CI's Clippy and documentation jobs pass with warnings denied.
6. **Independent validation:** there is no workspace dependency or sibling-crate path dependency. The package's integration test is compiled as an external consumer crate against `jabr_kernel` and uses only public exports. The M3 audit commit was validated by GitHub Actions push run [44363787779](https://github.com/Mujshanan/Jabr-kernel/actions/runs/44363787779) and matching PR run [44363802625](https://github.com/Mujshanan/Jabr-kernel/actions/runs/44363802625); both completed successfully. Formatting, default-feature tests, Serde-feature tests, Clippy, rustdoc, and the Rust 1.75 all-feature check passed.
7. **Public-contract review:** current commitments are limited to the documented ID range and allocator clone semantics; opaque `Port` identity; deterministic prerequisite-first graph ordering and the documented unresolved-node meaning of `GraphError::Cycle`; UTF-8 byte/half-open `Span`; one-based Unicode-scalar `Location`; snake_case severity wire values; and schema-version-1 envelopes with strict deserialization. The optional Serde feature remains the only serialization feature.

**Review note (not changed in M3):** `Envelope::schema_version` is currently a public field, so a caller can manually construct or mutate an envelope with a version that the deserializer would reject. Making that field private would be a public API change and would require an agreed getter/compatibility decision. The current constructors always set the supported version, and deserialization rejects unsupported versions. Record this as an API-hardening decision for release review rather than silently changing the established M2 contract.

**Exit criteria:** the implementation conforms to the architecture contract, not merely to its own unit tests. M3 technical audit and CI criteria are complete on this branch. Formal user acceptance remains separate and has not been inferred. M3 completion does not authorize engine integration, Orchestrator implementation, M4 work, or a merge.

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
