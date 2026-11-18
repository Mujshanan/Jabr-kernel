# Jabr-kernel

The foundational Rust library for the Jabr engineering calculation worksheet ecosystem.

Jabr-kernel is intended to provide shared, application-independent infrastructure used by the Jabr libraries. It should remain small and focused: domain-specific mathematics, diagram behavior, table behavior, and the Studio UI belong in their respective projects.

## Jabr ecosystem

| Repository | Responsibility |
| --- | --- |
| [Jabr-kernel](https://github.com/Mujshanan/Jabr-kernel) | Shared Rust foundation and common infrastructure |
| [Jabr-math](https://github.com/Mujshanan/Jabr-math) | Mathematical language, parsing, evaluation, and calculation engine (Jabr Math) |
| [Jabr-diagram](https://github.com/Mujshanan/Jabr-diagram) | Diagram model, geometry, drawing operations, and rendering (Jabr Diagram) |
| [Jabr-table](https://github.com/Mujshanan/Jabr-table) | Structured table model, cell operations, and data handling |
| [Jabr-UI](https://github.com/Mujshanan/Jabr-UI) | TypeScript application UI, AFFiNE/BlockSuite integration, and user interactions |

## Architectural boundaries

- **Kernel:** shared primitives and infrastructure that are genuinely common across libraries.
- **Math:** mathematical syntax, evaluation, variables, units, and calculation behavior.
- **Diagram:** diagram document model, geometry, editing operations, and rendering.
- **Table:** table document model, cells, ranges, and structured data operations.
- **UI:** worksheet experience, custom blocks, editing interactions, and presentation.

The Rust libraries should be independently buildable and testable. The UI should communicate with them through explicit, versioned interfaces rather than depending on their internal implementation details.

## Design principles

- Keep the kernel minimal and independent of domain-specific libraries.
- Keep public APIs deliberate, documented, and stable.
- Prefer explicit types and structured errors over implicit conventions.
- Avoid unnecessary dependencies and premature abstraction.
- Test each library independently and test cross-library contracts at integration boundaries.
- Do not introduce a dependency from the kernel to Math, Diagram, Table, or the UI.

## Project status

The architecture and responsibilities described here are the intended direction for the Jabr ecosystem. They do not imply that every listed capability has already been implemented.


## Shared variable-state contract

The initial revisioned variable-state boundary is specified in [docs/VARIABLE-STATE-CONTRACT.md](docs/VARIABLE-STATE-CONTRACT.md). The contract is domain-neutral; Math adapters translate their own values to opaque Kernel envelopes.
