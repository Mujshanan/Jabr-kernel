# Jabr Rust Dependency Policy

**Status:** proposed dependency policy  
**Scope:** Jabr-kernel, Jabr Math (`Jabr-math`), Jabr Diagram (`Jabr-diagram`), Jabr Table (`Jabr-table`), and Rust-to-UI adapters.

This records the initial dependency audit and preferred evaluation order. It does **not** authorize adding every candidate, pin versions, or replace per-repository design decisions. Each dependency must be justified by a concrete feature and validated against the supported Rust toolchain, Windows targets, WebAssembly targets where applicable, license policy, compile time, binary size, and maintenance activity.

## 1. General rules

1. Prefer the Rust standard library and small focused crates over broad frameworks.
2. Add a dependency only when a concrete requirement cannot be met cleanly with existing code.
3. Keep domain-specific dependencies in the repository that owns that domain.
4. Keep core libraries deterministic and independent of UI frameworks, wall-clock timing, and platform-specific services.
5. Prefer feature-gated optional dependencies when a capability is genuinely optional.
6. Check current release, MSRV, license, transitive dependency tree, target support, and maintenance before adoption and upgrades.
7. Avoid duplicate abstractions for the same job without a demonstrated need.
8. Use cargo tree -e features, cargo tree -i <crate>, cargo deny check, and target-specific builds to understand dependency impact.

## 2. Jabr-kernel

Kernel is the shared, domain-neutral foundation. Keep its dependency surface deliberately small.

| Candidate | Initial policy |
| --- | --- |
| serde | Optional feature only, for explicit serialization boundaries where required. |
| serde_json | Development-only initially, for tests and fixtures. Do not make JSON the internal state model. |
| thiserror | Do not add unless the public error model demonstrates a material benefit over the standard library. |
| indexmap | Do not add without a concrete insertion-order requirement in a public Kernel abstraction. |
| smallvec | Do not add without profiling evidence that small heap allocations matter. |

Kernel must not depend on Math, Diagram, Table, Engine, UI, or their domain-specific libraries. It exposes only generic identity, port, graph, source-coordinate, diagnostic, and envelope contracts; worksheet state and calculation lifecycle belong to Engine.

## 3. Jabr Math (`Jabr-math`)

### Parsing and diagnostics

- **Authoritative parser:** Calcpad is the authoritative parser and worksheet-language implementation for Jabr Math. Keep it behind a Jabr-owned adapter/facade so the public Math API and worksheet execution semantics remain Jabr-owned.
- Do not introduce logos/Pratt or chumsky as a competing authoritative parser. Reconsider parser replacement only through an explicit architecture decision supported by compatibility and diagnostic-parity tests.
- **CLI diagnostics:** choose one of ariadne or codespan-reporting for source-labelled terminal diagnostics. Keep terminal presentation out of the core parser/evaluator API.

### Numeric foundations

Evaluate by required semantics rather than adding a numeric stack wholesale:

- num-traits: shared numeric traits where generic algorithms need them.
- num-complex: only when complex values are in the supported language.
- num-rational and num-bigint: candidates for exact rational/integer arithmetic; verify interoperability and representation semantics.
- nalgebra or ndarray: prototype required matrix/vector operations and compare ergonomics, compile cost, and WASM impact before selecting one.
- malachite or dashu: candidates for arbitrary-precision arithmetic; benchmark actual operations and target compatibility.
- Avoid rug for the initial cross-platform/WASM-oriented core unless a demonstrated precision or performance requirement outweighs its native GMP/toolchain burden.

### Units, symbols, and errors

- Implement Jabr's runtime unit and dimensional semantics in Jabr Math. Review uom and dimensioned as design references or possible aids, but do not let an external crate dictate Jabr's language-level quantity model.
- Keep Symbolica out of the initial dependency set until its license, redistribution terms, and deployment constraints have been explicitly reviewed.
- Consider rustc-hash or lasso only after profiling identifies hashing or symbol interning as a bottleneck or stable architectural need.
- thiserror is a reasonable candidate for Math's structured errors if it improves maintainability; it is not automatically required in Kernel.

### CLI-only dependencies

- clap for command-line argument parsing.
- Choose one interactive line editor if a REPL is required: compare reedline and rustyline. Keep either dependency in the CLI binary, not the reusable Math library.

## 4. Jabr Diagram (`Jabr-diagram`)

Keep the diagram document model and geometry in Rust. Keep interactive presentation/rendering in the TypeScript UI using SVG/Canvas unless a measured requirement justifies a native renderer.

| Area | Candidates and guidance |
| --- | --- |
| 2D geometry | kurbo is a candidate for paths, curves, and geometric primitives. |
| Vectors/transforms | glam is a lightweight candidate; nalgebra is an alternative if the wider Math ecosystem already selects it. Avoid carrying both without a concrete boundary. |
| Topology/geometry algorithms | geo and geo-types provide broader geometry types and algorithms. Check their dependency tree before adding overlapping crates separately. |
| Boolean operations/offsets | i_overlay is a focused candidate; test required fill rules, robustness, and performance. |
| Spatial indexing | rstar only when hit-testing or spatial queries need an index at measured scale. |
| Tessellation | lyon only for triangle tessellation or another specific geometry need, not as a general rendering commitment. |
| SVG | Evaluate usvg, resvg, and tiny-skia for import/export, rasterization, or deterministic rendering workflows. Do not add all three unless the pipeline needs all three. |
| CAD exchange | dxf as a candidate for DXF import/export; validate supported entities and round-trip expectations. |
| Text | ttf-parser for font metadata/outlines and rustybuzz for shaping when the Rust-side text pipeline requires them. |

Important overlap: geo's dependency graph may already include i_overlay and rstar. Inspect actual enabled features and versions before adding those crates directly. Avoid vello unless Jabr makes a deliberate move to a native renderer.

## 5. Jabr Table (`Jabr-table`)

- calamine: preferred initial candidate for reading XLSX and other supported spreadsheet formats.
- csv: focused CSV import/export.
- rust_xlsxwriter: candidate for generating new XLSX workbooks. It is a writer, not an in-place editor for arbitrary existing workbooks; preserve this distinction in product requirements.
- indexmap: optional if stable insertion order is part of the table model's required semantics.
- smallvec and compact_str: defer until profiling or memory measurements justify them.
- Implement the A1-style cell/range reference parser in Jabr Table unless a concrete compatibility requirement justifies an external parser.
- Keep formula text and cell identity in Table. Formula parsing, evaluation, units, and calculation semantics belong to Jabr Math.
- Defer polars and arrow; they are likely excessive for the initial worksheet table model unless large-dataframe or columnar interchange requirements emerge.

## 6. Rust-to-UI and desktop boundaries

- wasm-bindgen: standard Rust/WASM boundary candidate for browser-facing Rust modules.
- serde-wasm-bindgen: preferred candidate for mapping Serde values to/from JavaScript values without routing every interaction through JSON strings.
- Evaluate tsify or ts-rs for TypeScript type generation. Select one approach based on generated type quality, build integration, and compatibility with the chosen WASM boundary.
- serde_json: retain where JSON is the explicit wire format, fixture format, or persistence format; do not use it by default for every in-process JS boundary.
- console_error_panic_hook: optional debug/development WASM diagnostics hook; keep it out of production paths unless justified.
- Tauri belongs in a separate desktop adapter/application layer. Core libraries and browser UI contracts must not depend on Tauri.

## 7. Cross-repository development and quality tools

| Tool | Policy |
| --- | --- |
| proptest | Preferred property-based testing candidate for parsers, numeric semantics, geometry, state transitions, and reference parsing. |
| insta | Use selectively for stable snapshots such as diagnostics, serialization fixtures, and rendered textual output. Avoid brittle snapshots of incidental formatting. |
| criterion | Add when representative benchmarks and a performance question exist. |
| cargo-deny | Use for license, advisory, source, and duplicate-dependency policy checks once package policy is defined. |

Do not introduce rayon into core libraries by default. Parallelism should follow measured workloads and explicit determinism/target requirements. Rayon can have a sequential fallback on some targets, but real WebAssembly multithreading requires additional platform setup.

Avoid using std::time::Instant as part of deterministic core semantics or persisted calculation state. Benchmarking and elapsed-time measurement belong at test, benchmark, or application boundaries.

## 8. WebAssembly and platform checks

- Check every transitive dependency on the actual WASM target used by Jabr; native compilation alone is not sufficient.
- For getrandom 0.4, the browser JavaScript backend feature is named wasm_js. Do not enable it globally or in a reusable library without understanding target implications; prefer enabling it at the narrowest appropriate application/test boundary.
- Keep browser-specific adapters separate from generic core crates so non-browser WASM targets are not accidentally coupled to JavaScript bindings.
- Validate Windows x64 native builds independently from browser WASM builds.
- Record and enforce the project's chosen MSRV. Do not assume the newest dependency release supports it.

## 9. Adoption checklist

Before adding or upgrading a dependency, record:

- [ ] The concrete feature and why the standard library/existing code is insufficient.
- [ ] Owning repository and architectural layer.
- [ ] Current crate release, maintenance activity, MSRV, and license.
- [ ] Transitive dependency and enabled-feature impact.
- [ ] Windows x64 build/test result.
- [ ] WASM target build/test result, if the crate is on a WASM path.
- [ ] Binary size and compile-time impact where relevant.
- [ ] Security/advisory and license-policy result.
- [ ] Whether the dependency duplicates functionality already present elsewhere in Jabr.

A crate listed in this document is a candidate, not an approval to add it. Cargo manifests and lockfiles remain the authoritative record of adopted dependencies.
