# Jabr-kernel

Jabr-kernel is the small, application-independent Rust foundation shared by the Jabr engineering calculation ecosystem.

Its approved 0.1 architecture contract is documented in [development.md](./development.md). That contract defines the boundaries between Kernel, Math, Diagram, Table, the Jabr API, and Jabr-UI. It is an architectural agreement, not a claim that the described capabilities have already been implemented.

## Jabr ecosystem

| Project | Responsibility |
| --- | --- |
| [Jabr-kernel](https://github.com/Mujshanan/Jabr-kernel) | Shared variable state, revisions, change tracking, snapshots, generic dependency-graph primitives, and shared infrastructure |
| [Jabr-cli / Jabr Math](https://github.com/Mujshanan/Jabr-cli) | Mathematical language, parsing, evaluation, values, quantities, units, and conversions |
| [Jabr-draw / Jabr Diagram](https://github.com/Mujshanan/Jabr-draw) | Diagram document model, geometry, editing operations, and rendering |
| [Jabr-table](https://github.com/Mujshanan/Jabr-table) | Table document model, cell identity/content, ranges, and table operations |
| Jabr API | Cross-engine orchestration, recalculation, and versioned application boundary |
| [Jabr-UI](https://github.com/Mujshanan/Jabr-UI) | Complete TypeScript application UI, AFFiNE/BlockSuite integration, worksheet interactions, and presentation |

## Approved dependency direction

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

Dependencies point toward Kernel, never from Kernel back to an engine. Math, Draw, and Table remain independently developable repositories. The API composes their capabilities; Jabr-UI consumes the API boundary rather than importing engine internals.

## Approved architecture principles

- Kernel owns shared variable-state lifecycle: state creation and updates, revisions, change tracking, snapshots, and generic dependency-graph primitives.
- Math owns mathematical values and semantics, including `Value`, `Quantity`, units, conversions, parsing, and evaluation.
- Table owns cell identity and formula text. Math interprets and evaluates formulas; Kernel provides generic graph primitives; the API coordinates recalculation.
- The Jabr API owns cross-engine recalculation orchestration.
- JSON inside a versioned `Envelope` is the initial wire format.
- Public diagnostic codes are stable and namespaced by owning crate (for example, `kernel.*`, `math.*`, `draw.*`, and `table.*`).
- Independent repositories remain the initial development model. A monorepo or shared Cargo workspace is not part of the approved 0.1 contract.
- Keep Kernel minimal, domain-neutral, independently testable, and free of dependencies on Math, Draw, Table, the API, or the UI.

## First vertical slice

The first integration slice is `x = 3 m`: Math parses and evaluates the expression; Kernel supplies identity/state and dependency-graph primitives; the API returns a versioned JSON Envelope; and Jabr-UI renders the value and unit. The slice must verify state revisions and that previously captured snapshots remain unchanged after subsequent updates.

## Project status

The architecture contract is approved for documentation and implementation planning. It does **not** mean the capabilities are implemented. No implementation, repository consolidation, or migration is implied by this document. See [development.md](./development.md) for ownership details, contracts, the vertical-slice acceptance criteria, and deferred decisions.
