# Jabr Kernel Shared Variable-State Contract

**Status:** Initial contract proposal. The repository currently contains architecture documentation only; no Rust implementation is claimed by this specification.

## Purpose

Jabr Kernel provides shared, domain-neutral state infrastructure for Jabr libraries. Its first integration target is revisioned variable assignment state shared with Jabr Math.

Kernel is not a calculation engine. It must not parse expressions, evaluate formulas, interpret units, know diagram geometry, or depend on a UI/editor framework.

## Ownership

- Kernel: stable IDs, scopes/namespaces, opaque typed value envelopes, immutable snapshots, monotonic revisions, atomic transactions and change sets.
- Math: language symbols, parsing, evaluation, units, numeric/value semantics and conversion to/from opaque envelopes.
- Diagram: diagram documents and geometry.
- Table: table documents and data operations.
- Jabr-UI/AFFiNE: authored document persistence, collaboration provider, block lifecycle and presentation.

Kernel is independent of every domain library; dependencies must point from domain adapters to Kernel, never from Kernel to a domain.

## Core concepts

- `VariableId`: stable, opaque identifier; never use a display name or array index as durable identity.
- `ScopeId`: stable namespace/scope identifier.
- `ValueEnvelope`: `type_id`, `schema_version`, and opaque payload. Kernel validates envelope structure but does not interpret domain payloads.
- `VariableRecord`: variable ID, scope ID, optional display/name metadata, envelope and last-changed revision.
- `VariableSnapshot`: immutable snapshot at one global revision.
- `StateTransaction`: expected base revision and a set of create/update/delete operations.
- `ChangeSet`: committed revision and exact IDs changed.
- `KernelError`: typed conflict, missing ID/scope, duplicate ID, invalid envelope, unsupported schema or invalid operation.

## Required semantics

1. Every successful transaction is atomic: all operations commit or none do.
2. Every successful commit increments the global revision exactly once.
3. A transaction whose base revision is stale is rejected; it never silently overwrites newer state.
4. Snapshots are immutable values. A later update cannot alter an earlier snapshot.
5. A no-op transaction does not increment the revision.
6. Updates identify records by stable ID, not mutable display name.
7. Rename is metadata change, not identity replacement.
8. Scope deletion must define explicit cascade/reject behavior and may not silently orphan variables.
9. Kernel stores opaque values and does not normalize floating point, units, decimals or matrices.
10. Serialization must be deterministic for equal logical state and include schema/protocol versions.

## Math adapter flow

```text
Math source + immutable Kernel snapshot
                 |
                 v
       Jabr Math evaluation
                 |
       Evaluation result + proposed changes
                 |
                 v
       Math -> Kernel transaction
                 |
        revision check / atomic commit
                 |
          committed ChangeSet
                 |
                 v
        new immutable snapshot
```

Math must evaluate against a captured snapshot. It returns proposed assignments tagged with the snapshot revision. The Kernel transaction either commits all changes or returns a conflict. Math then re-evaluates against the new snapshot when appropriate; it must not patch old evaluation results in place.

## API shape to test

The first Rust crate should remain small and domain-neutral. Exact names are open to review, but the API should be equivalent to:

- `Kernel::snapshot(scope)`
- `Kernel::begin_transaction(expected_revision)`
- `Transaction::insert(scope, id, envelope)`
- `Transaction::update(id, envelope)`
- `Transaction::delete(id)`
- `Transaction::commit()`
- `Snapshot::get(id)`
- `Snapshot::revision()`

Avoid a shared generic expression/value AST in Kernel. Avoid embedding Math's `Value` enum in Kernel. Keep concurrency strategy internal; the first implementation can use a single-writer lock if semantics are explicit and tested.

## Contract test matrix

- Snapshot is immutable after a later commit.
- Two transactions from the same base revision: first commit succeeds, second conflicts.
- Multi-operation transaction failure rolls back all operations.
- Duplicate IDs and unknown IDs return typed errors.
- Opaque unknown type IDs round-trip without Kernel interpretation.
- Schema-version mismatch is surfaced, not silently coerced.
- Scope isolation prevents accidental cross-scope reads.
- Deterministic serialization round-trips.
- No dependency on Jabr Math, Diagram, Table, AFFiNE, BlockSuite, React or browser APIs.

## Implementation order

1. Add a minimal Rust workspace and a single `jabr-kernel-core` crate.
2. Implement identity, opaque envelope, snapshot and transaction semantics.
3. Add property/unit tests for atomicity, revisioning and snapshot isolation.
4. Publish the crate only after API and serialization compatibility are tested.
5. Add a Math-side adapter; Kernel must not import Jabr Math.
