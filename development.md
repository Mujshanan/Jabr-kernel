# Jabr Kernel 0.1 — development record

**Baseline:** 2026-11-18  
**Status:** Kernel implementation and M0–M4 technical validation complete; documentation cleanup is the remaining 0.1 release task.  
**Implementation:** [PR #2](https://github.com/Mujshanan/Jabr-kernel/pull/2), merged to \`main\`.

## 1. Architecture and ownership

Jabr is developed as independent repositories with explicit contracts:

- **Kernel** supplies domain-neutral identifiers, ports, graph primitives, source spans/locations, diagnostics, and versioned envelopes.
- **Math** owns mathematical values, units, expression syntax, parsing, AST, evaluation, and mathematical semantics.
- **Table** owns table models, cells, and table operations.
- **Diagram** owns diagram models, geometry, drawing/editing, and rendering.
- **Engine** owns worksheet-level orchestration: cross-engine graph construction, dependency binding, scheduling, recalculation, worksheet revisions/snapshots, result storage, and result lifecycle.
- **UI** owns the unified document experience and presentation; it consumes Engine's application-facing contract.

Math, Table, and Diagram remain independently developable and testable. Engine composes them. Kernel must not depend on any of these components. No shared cross-repository Cargo workspace is introduced for Kernel 0.1.

## 2. Kernel 0.1 contract

Kernel exposes six infrastructure areas:

| Area | Public API | Boundary |
| --- | --- | --- |
| Identifiers | \`NodeId\`, \`NodeIdAllocator\`, \`IdAllocationError\`, \`MAX_NODE_ID\` | Nonzero JavaScript-safe node IDs; allocator clones share a sequence |
| Ports | \`Port\` | Opaque node/port identity; no domain meaning |
| Graph | \`DependencyGraph<N>\`, \`GraphError<N>\` | Generic graph operations and deterministic prerequisite-first topological ordering; no scheduling |
| Source coordinates | \`Span\`, \`SpanError\`, \`Location\`, \`LocationError\` | UTF-8 byte, half-open spans; one-based line and Unicode-scalar column locations |
| Diagnostics | \`Diagnostic\`, \`Severity\` | Generic structured diagnostics; code meaning belongs to the emitting component |
| Envelopes | \`Envelope<T>\`, \`ENVELOPE_SCHEMA_VERSION\` | Generic versioned result/diagnostic transport; unsupported schema versions are rejected on deserialization |

Kernel does not parse or evaluate formulas, interpret dependency meaning, build worksheet graphs, own worksheet state, store calculation results, or schedule/recalculate work. In particular, Kernel's topological sort reports graph order only; Engine decides what to execute and when.

## 3. Audit: revisions, snapshots, and result storage

The active \`src/\` tree was audited for \`Revision\`, \`Snapshot\`, and \`ResultStore\`. None of these types or modules remains in the shipped source or public exports. The retained source modules are limited to diagnostics, envelopes, graph, identifiers, locations, ports, and spans.

The old implementations were deliberately removed during the Kernel scope reduction:

- **Revision:** checked monotonic revision values and their serialization behavior.
- **Snapshot:** revision-associated owned payloads and payload mapping.
- **ResultStore:** deterministic node-keyed opaque result storage and CRUD/iteration operations.

Their former unit and integration tests were removed with those modules; their behavior was not silently ported into Kernel. This is an intentional boundary decision, not accidental dead code. The corresponding worksheet-level concepts belong to **Engine**, which owns revisions, snapshots, result storage, and their lifecycle. Kernel remains free of these responsibilities.

Historical references to the removed names in this audit are retained to explain the change; they are not active API.

## 4. Release validation

The merged Kernel implementation was validated independently with:

\`\`\`bash
cargo fmt --all -- --check
cargo test --all-targets
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features
\`\`\`

CI also checks Rust 1.75 and a clean working tree. The M4 GitHub Actions runs passed:

- [Push validation](https://github.com/Mujshanan/Jabr-kernel/actions/runs/44369076724)
- [Pull-request validation](https://github.com/Mujshanan/Jabr-kernel/actions/runs/44369097371)

See [API-COMPATIBILITY.md](API-COMPATIBILITY.md) for the published Rust and Serde contract.

## 5. Release gate and change policy

- Keep Kernel 0.1 limited to the six infrastructure areas above.
- Do not reintroduce revision, snapshot, result-store, worksheet-state, caching, or scheduling behavior into Kernel.
- Do not add dependencies on Math, Table, Diagram, Engine, BlockSuite, Yjs, or UI.
- Changes to ownership, dependency direction, public API, or wire compatibility require explicit architecture review.
- Engine contract design and Engine implementation are separate work; this Kernel cleanup does not authorize either.
