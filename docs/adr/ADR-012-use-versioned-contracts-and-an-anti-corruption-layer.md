# ADR-012: Use Versioned Contracts and an Anti-Corruption Layer

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: contracts, integration, typescript, anticorruption

## Context

STE integrates fast-moving upstream Rust and TypeScript projects. Letting their types or semantics leak into the domain would couple scientific claims, persistence, and context boundaries to external release cycles.

## Decision

Define stable Rust domain ports and versioned `serde` integration DTOs in `ste-contracts`. Map all upstream rvCSI, RuView, ruv-FANN, RuVector, MidStream, AgentDB, DSPy.ts, and Ruflo interactions through infrastructure anti-corruption layers.

Contracts include schema version, event ID, aggregate ID, source timestamp, emission timestamp, producer version, correlation/causation IDs, provenance reference, and idempotency key where applicable. Reject unknown required semantics; do not silently default safety-relevant fields.

TypeScript adapters consume generated JSON Schema or equivalent bindings from Rust contracts. They cannot import domain internals or write authoritative stores directly.

## Consequences

### Positive

- Upstream churn is isolated and cross-language behavior is contract-tested.
- Domain terminology remains aligned with the evidence model.

### Negative

- Mapping code and schema evolution add maintenance.
- Duplicate transport and domain representations are intentional.

### Neutral

- In-process Rust calls may use richer private types inside one context.

## Links

**Depends on**: [ADR-001](ADR-001-adopt-a-rust-first-modular-monolith.md), [ADR-003](ADR-003-separate-observations-physiology-and-latent-state.md)
**Related**: [DDD context map](../ddd/context-map.md), [ADR-007](ADR-007-use-ruvector-as-primary-personalization-memory.md)
