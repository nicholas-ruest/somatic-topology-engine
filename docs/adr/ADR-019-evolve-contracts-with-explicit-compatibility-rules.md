# ADR-019: Evolve Contracts with Explicit Compatibility Rules

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: schemas, compatibility, migrations, api

## Context

Persisted events, replay files, Rust contexts, and optional TypeScript adapters will evolve at different speeds. Accidental schema compatibility can corrupt scientific provenance or prevent rollback.

## Decision

Assign semantic versions to public contracts and artifact schemas. Additive optional fields are minor changes; changed meaning, required fields, units, enum removal, or default behavior are major changes. Consumers reject unsupported major versions and preserve unknown optional fields where round-tripping is required.

Generate JSON Schema and TypeScript bindings from Rust-owned DTOs. Maintain golden compatibility fixtures, upgrade/downgrade tests for supported release windows, event upcasters, and a published support matrix. No safety-relevant field receives an implicit default.

## Consequences

### Positive
- Safe rolling upgrades of adapters and reliable rollback.
- Contract drift is detected in CI.

### Negative
- Multiple schema versions and fixtures require maintenance.
- Breaking changes become deliberately expensive.

### Neutral
- Private intra-context types are not subject to public compatibility guarantees.

## Links

**Depends on**: [ADR-012](ADR-012-use-versioned-contracts-and-an-anti-corruption-layer.md), [ADR-017](ADR-017-standardize-identifiers-time-units-and-numeric-semantics.md), [ADR-018](ADR-018-use-an-append-only-journal-with-versioned-projections.md)
