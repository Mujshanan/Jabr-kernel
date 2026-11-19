# Kernel 0.1 completion and release checklist

**Branch:** `feat/kernel-0.1-contract-hardening`  
**Baseline:** M0–M4 on `main`  
**Purpose:** finish the Kernel-only contract-hardening work without starting engine integration.

## Decisions locked for Kernel 0.1

1. The accepted architecture separates the versioned public contract (`jabr-kernel-api`) from the implementation (`jabr-kernel`). Engine and domain consumers depend on the public API crate, never on implementation internals. This hardening PR records and prepares the boundary; API-crate extraction/publication is a separate implementation step.
2. Kernel owns only generic infrastructure: identifiers, ports, dependency graph primitives, source spans/locations, diagnostics, and versioned envelopes.
3. Kernel does not own mathematical variable semantics or persistent worksheet variable state. Math owns mathematical identity, types, units, scopes, parsing, and evaluation semantics. Jabr Engine owns persistent worksheet definitions/state, worksheet revisions, snapshots, result lifecycle, invalidation, and recalculation coordination.
4. The envelope schema version is immutable after construction. It is private and available through `Envelope::schema_version()`. Deserialization validates the incoming version; serialization preserves the fixed version-1 wire field.
5. The existing version-1 JSON field names and payload shape remain unchanged.

## Hardening work in this branch

- Encapsulate `Envelope::schema_version` and add a read-only accessor.
- Add rustdoc examples and a compile-fail example proving consumers cannot mutate the schema version.
- Update public API tests to assert the accessor and fixed version behavior.
- Update the compatibility note to remove the previous mutable-field caveat.
- Align README and development documentation with the accepted `jabr-kernel-api` / `jabr-kernel` boundary and Math / Engine / Kernel responsibility split.
- Add the MIT license file to match the Cargo manifest.
- Preserve the existing minimal dependency policy and Rust 1.75 MSRV.

## Validation gate

The pull request is complete only when GitHub Actions reports success for:

- `cargo fmt --all -- --check`
- `cargo test --all-targets`
- `cargo test --all-targets --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo doc --no-deps --all-features` with rustdoc warnings denied
- Rust 1.75 `cargo check --all-targets --all-features`
- Clean working-tree verification

## Explicitly not included

- No variable-state, revision, snapshot, result-store, or scheduler implementation in Kernel.
- No Math, Table, Draw, Engine, UI, BlockSuite, or Yjs dependency.
- No cross-repository integration changes.
- No Engine integration or cross-repository dependency changes. The API crate extraction is not implemented by this documentation/hardening PR.
- No release/tag/publication.
- No merge authorization.

## Completion status

M0–M4 are already present on `main`. This branch adds the final API-hardening and documentation corrections. Kernel 0.1 is ready for user acceptance only after the branch's CI checks pass and the pull request is reviewed. Engine integration remains a separate, explicitly authorized phase.
