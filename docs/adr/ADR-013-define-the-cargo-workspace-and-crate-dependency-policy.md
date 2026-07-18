# ADR-013: Define the Cargo Workspace and Crate Dependency Policy

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: rust, cargo, modules, dependencies

## Context

The bounded contexts need enforceable Rust boundaries without creating needless deployment units. Uncontrolled crate dependencies would erode the evidence pipeline and make infrastructure leak into domain code.

## Decision

Create a Cargo workspace with `ste-kernel`, `ste-contracts`, context crates, `ste-runtime`, and `ste-cli`. Context domain modules may depend only on `ste-kernel`; application modules may also depend on their own domain; infrastructure depends inward. Cross-context interaction uses application ports or `ste-contracts` only.

Use workspace-pinned dependency versions, resolver v2, a checked-in lockfile, minimal features, `forbid(unsafe_code)` by default, and explicit review for exceptions. CI will enforce dependency direction with metadata analysis and deny cyclic context dependencies.

## Consequences

### Positive
- Compile-time boundary enforcement and independently testable contexts.
- One workspace supports reproducible edge builds.

### Negative
- Additional crates and mapping code increase compile time.
- Exceptions for hardware FFI require narrowly documented unsafe modules.

### Neutral
- Logical contexts may initially share a crate only if private-module boundaries and extraction criteria are documented.

## Links

**Depends on**: [ADR-001](ADR-001-adopt-a-rust-first-modular-monolith.md), [ADR-012](ADR-012-use-versioned-contracts-and-an-anti-corruption-layer.md)
**Related**: [DDD context map](../ddd/context-map.md)
