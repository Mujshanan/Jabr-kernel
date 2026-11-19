# State, Revision, and Snapshot — Kernel API Contract Proposal

**Status:** Approved and implemented 2026-11-19; CI-validated on PR #16  
**Target:** `jabr-kernel-api` 0.1  
**Repository:** `Mujshanan/Jabr-kernel`  
**Date:** 2026-11-19

## 1. Purpose

This document settles a proposed, deliberately small Rust contract for shared state, monotonic revisions, and immutable snapshots. It is the design gate before implementation: the types and signatures below are proposals for explicit review, not claims about APIs that already exist.

The intent is to give Jabr engines a common way to store opaque object values, observe committed state changes, identify a state version, and capture a consistent historical view. The contract must not turn Kernel into a mathematical evaluator, worksheet interpreter, recalculation scheduler, or application orchestrator.

## 2. Architecture ownership

The following ownership is the controlling boundary for this proposal:

| Concern | Owner |
| --- | --- |
| Stable object identity and generic state storage | Kernel |
| Assignment/update/removal of opaque object values | Kernel |
| Monotonic state revision and change tracking | Kernel |
| Immutable snapshots of committed state | Kernel |
| Mathematical values, types, units, scopes, and variable semantics | Math |
| Parsing, expression evaluation, and right-hand-side computation | Math |
| Mathematical dependency meaning and recalculation decisions/order | Math |
| Cross-engine execution requests/results and application coordination | API / Orchestrator |
| Persistence policy, durable document ownership, UI history presentation | Owning application layer |

Kernel treats each stored value as opaque `T`. It does not inspect, compare, evaluate, normalize, or interpret it. A `NodeId` identifies the object whose current value is stored; Kernel does not decide how an engine maps authored objects or mathematical variables to IDs.

**Architecture correction:** earlier Kernel M0/M3 documentation excluded state, revisions, and snapshots entirely. That was an earlier scope decision and conflicts with the subsequently accepted Jabr architecture in which Kernel owns shared object/variable state, revisions, change tracking, and snapshots while Math owns mathematical semantics and recalculation. This proposal supersedes those exclusions only for the generic facilities defined here. It does not move scopes, mathematical dependency semantics, result storage, caching, or scheduling into Kernel.

## 3. Design principles

1. **Opaque payloads:** Kernel stores `T`; only the owning engine understands it.
2. **One state, one revision sequence:** a `State<T>` has one monotonically increasing revision covering its complete object map.
3. **Committed mutations only:** each successful effective mutation advances the revision exactly once.
4. **No revision for no-op removal:** removing an absent ID changes nothing and does not advance the revision.
5. **Overflow is atomic:** if the revision cannot advance, the mutation fails and the state remains unchanged.
6. **Snapshots are immutable values:** a snapshot records the revision and a point-in-time copy of all entries. Later state mutations cannot alter it.
7. **No hidden history service:** Kernel returns snapshots when requested; it does not retain an unbounded history or choose retention policy.
8. **No scheduling:** a revision or changed ID is evidence that state changed, not an instruction to recalculate anything.
9. **Deterministic enumeration:** entries are keyed and enumerated in `NodeId` order.
10. **Single-writer API in 0.1:** mutation requires `&mut State<T>`. Synchronization and concurrent transaction semantics are outside this contract.

## 4. Exact proposed Rust interfaces

The following declarations are the proposed public surface. Implementation details such as internal locking, storage layout, and notification transport remain private to `jabr-kernel`.

### 4.1 Revision

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RevisionError {
    pub current: Revision,
}

impl Revision {
    pub const INITIAL: Self = Self(0);

    pub const fn get(self) -> u64;

    pub fn checked_next(self) -> Result<Self, RevisionError>;
}
```

Contract:

- Revisions are non-negative, monotonically increasing integers represented internally as `u64`.
- A newly created `State<T>` starts at `Revision::INITIAL` (zero).
- The first effective mutation produces revision 1.
- `checked_next` returns the next revision or `RevisionError` at `u64::MAX`; it never wraps.
- Equality and ordering compare the numeric revision.
- With the optional `serde` feature, serialize revisions as **decimal strings**, not JSON numbers, so all `u64` values round-trip exactly through JavaScript-based clients. Deserialization accepts only a valid unsigned decimal string in the `u64` range.
- Do not expose a public unchecked constructor or public mutable field.

### 4.2 State

```rust
pub struct State<T> {
    // Private: current revision and NodeId-keyed opaque values.
}

impl<T> State<T> {
    pub fn new() -> Self;

    pub fn revision(&self) -> Revision;

    pub fn get(&self, id: NodeId) -> Option<&T>;

    pub fn contains(&self, id: NodeId) -> bool;

    pub fn len(&self) -> usize;

    pub fn is_empty(&self) -> bool;

    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &T)>;

    pub fn set(
        &mut self,
        id: NodeId,
        value: T,
    ) -> Result<Revision, RevisionError>;

    pub fn remove(
        &mut self,
        id: NodeId,
    ) -> Result<Option<T>, RevisionError>;

    pub fn snapshot(&self) -> Snapshot<T>
    where
        T: Clone;
}
```

Contract:

- `State<T>` is the current committed mapping from `NodeId` to opaque `T` values plus its revision.
- `new` creates an empty state at revision zero.
- `get`, `contains`, `len`, `is_empty`, and `iter` are read-only and do not change the revision.
- `iter` yields entries in ascending `NodeId` order.
- `set` inserts or replaces the value for `id`. Every successful call is an explicit assignment and advances the revision once, including replacement with a value that happens to be equal. No `PartialEq` bound is imposed on `T`.
- `set` computes the next revision before changing the map. On overflow it returns `RevisionError` and leaves both the map and revision unchanged.
- `remove` advances the revision once and returns `Ok(Some(old_value))) when an entry existed. If no entry existed, it returns `Ok(None)` and leaves the revision unchanged. On revision overflow for an existing entry, it returns `Err` and leaves the entry and revision unchanged.
- No public mutable access to the backing map is provided; all effective writes must pass through revision tracking.
- The type does not define variable names, scopes, namespaces, types, units, dependency rules, or evaluation behavior.
- The type does not publish callbacks or events. The implementation may expose an internal change record/notification mechanism to its coordinating layer, but no public event bus is part of this minimal contract.

### 4.3 Snapshot

```rust
pub struct Snapshot<T> {
    // Private: captured revision and NodeId-keyed opaque values.
}

impl<T> Snapshot<T> {
    pub fn revision(&self) -> Revision;

    pub fn get(&self, id: NodeId) -> Option<&T>;

    pub fn contains(&self, id: NodeId) -> bool;

    pub fn len(&self) -> usize;

    pub fn is_empty(&self) -> bool;

    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &T)>;
}
```

Contract:

- `Snapshot<T>` is an immutable, point-in-time copy of a `State<T>`.
- `State::snapshot` clones the entries and captures the same revision observed at the time of the call.
- Snapshot reads and iteration have the same semantics and deterministic ordering as State reads.
- A snapshot is unaffected by later inserts, replacements, or removals in the source state.
- A snapshot does not offer mutation methods and does not refer back to a live `State<T>`.
- Snapshot creation requires `T: Clone`; no Clone bound is imposed on the State or Snapshot type itself.
- Snapshot lifetime and retention are controlled by the caller. Kernel does not maintain a history database or decide which snapshots to keep.

## 5. Mutation and revision semantics

| Operation | Result | Revision effect |
| --- | --- | --- |
| Read / contains / enumerate | Current value(s) | Unchanged |
| Set a new ID | `Ok(new_revision)` | Advance once |
| Replace an existing ID | `Ok(new_revision)` | Advance once |
| Set an equal value | `Ok(new_revision)` | Advance once; assignment is explicit |
| Remove an existing ID | `Ok(Some(old_value))` | Advance once |
| Remove an absent ID | `Ok(None)` | Unchanged |
| Any effective write at `u64::MAX` | `Err(RevisionError)` | Unchanged; no partial mutation |
| Create snapshot | `Snapshot<T>` | Unchanged |

A revision identifies the complete committed state after a mutation. It is not an object ID, timestamp, dependency-graph generation, calculation-result version, or promise that recalculation has completed.

For this minimal 0.1 contract, one method call is one commit boundary. Multi-object atomic batches/transactions are deliberately not included. If worksheet execution requires several assignments to become visible atomically, that requirement must be specified and reviewed before implementation rather than simulated by callers with repeated `set` calls.

## 6. Serialization and compatibility

- `Revision` has an explicit optional Serde representation as a decimal string.
- `State<T>` and `Snapshot<T>` do **not** receive derived Serde implementations in this proposal. Their durable wire/storage representation is not yet defined; serializing them would prematurely establish persistence and schema policy.
- `State<T>` and `Snapshot<T>` keep their fields private. Consumers use constructors and accessors.
- Do not serialize internal maps by relying on incidental Rust field layout.
- The existing `Envelope<T>` remains a transport wrapper and does not replace the state/snapshot contract.
- Any future wire representation for state or snapshots requires an explicit schema/versioning decision.

## 7. Explicit non-goals

This contract does not introduce:

- Mathematical variable types, values, units, names, scopes, namespaces, or shadowing rules.
- Parsing, expression evaluation, or right-hand-side computation.
- Mathematical dependency extraction, dirty propagation, recalculation order, or scheduling.
- Result stores, calculation caches, or calculated-output history.
- Cross-engine transactions or graph construction.
- Persistence, database storage, document serialization, or snapshot retention policy.
- Concurrent writers, locks, async APIs, subscriptions, or a public event bus.
- A public object-ID type separate from the existing `NodeId`.

## 8. Required contract tests before implementation is accepted

The API implementation must prove at least:

### Revision
- Initial revision is zero.
- Increment is monotonic and checked.
- `u64::MAX` fails without wrapping.
- Serde decimal-string round-trip includes values above JavaScript's exact integer limit and `u64::MAX`; malformed, negative, fractional, and overflowing strings are rejected.

### State
- New state is empty at revision zero.
- Insert and replacement each advance exactly once.
- Equal-value assignment advances exactly once without requiring `T: PartialEq`.
- Reads and iteration do not advance the revision.
- Removing an existing ID returns the old value and advances once.
- Removing an absent ID returns `None` without advancing.
- Overflow leaves the entire state unchanged.
- Enumeration is deterministic by `NodeId`.

### Snapshot
- Snapshot captures the source revision and all current entries.
- Later source mutations do not alter the snapshot.
- Snapshot reads and iteration are deterministic.
- Snapshot creation does not change the source revision.
- Snapshot does not expose mutation.

### Public boundary
- A downstream test imports these contracts through `jabr_kernel_api`, without importing `jabr_kernel`.
- The API crate remains independent of Math and Kernel implementation internals.
- Kernel implementation depends on the API crate; the API crate never depends on Kernel implementation.
- No tests imply that Kernel performs mathematical evaluation or scheduling.

## 9. Decisions requiring explicit approval

The following are the complete proposed 0.1 decisions to review:

1. Adopt `Revision(u64)`, starting at zero, checked increment, decimal-string Serde.
2. Adopt `State<T>` as a `NodeId`-keyed opaque value map with one state-wide revision.
3. Treat every successful `set` as a revision-advancing assignment, even when the new value compares equal.
4. Treat removal of a missing ID as a no-op with no revision advance.
5. Adopt immutable `Snapshot<T>` as a cloned point-in-time map plus captured revision.
6. Keep mutation single-writer via `&mut self`; exclude multi-object transactions and concurrent writers from this minimal contract.
7. Do not define persistence serialization or snapshot retention in API 0.1.
8. Keep mathematical scopes, dependency semantics, recalculation, and evaluation outside Kernel.

**Approval and implementation:** The user approved all eight decisions in section 9 on 2026-11-19. The interfaces are implemented in `jabr-kernel-api`, with compatibility re-exports from `jabr-kernel`. The API crate owns the public generic contract types and their domain-neutral behavior; Kernel remains the architecture owner of the shared-state responsibility. The older M0/M3 blanket exclusions are historical and superseded by this accepted contract. Formatting, tests (default and Serde), Clippy, rustdoc, clean-tree verification, and Rust 1.75 checks passed in PR #16 CI run [#44462489225](https://github.com/Mujshanan/Jabr-kernel/actions/runs/44462489225).
