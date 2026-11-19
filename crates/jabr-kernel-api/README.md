# jabr-kernel-api

Versioned, domain-neutral public contracts shared by Jabr engines.

This crate is the dependency boundary for consumers that need Kernel-owned
identifiers, source coordinates, diagnostics, envelopes, ports, and generic
graph primitives. It intentionally does not depend on Jabr Math, worksheet
execution, the UI, or the `jabr-kernel` implementation crate.

## Dependency

```toml
[dependencies]
jabr-kernel-api = "0.1"
```

Enable the optional `serde` feature only when serialization is required:

```toml
jabr-kernel-api = { version = "0.1", features = ["serde"] }
```

The `jabr-kernel` implementation crate depends on this API crate and
re-exports its public types for compatibility. New engine crates should
depend directly on `jabr-kernel-api`, never on Kernel implementation internals.

## Contract stability

The crate starts at API 0.1. Public DTO and wire-format changes require explicit
compatibility review. Envelope schema versioning is independent of crate
versioning. Domain-specific semantics and execution policy remain with the
owning engine.
