# ADR-018: Use an Append-Only Journal with Versioned Projections

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: persistence, events, migrations, recovery

## Context

STE needs replay, provenance, audit, anchors, and recovery while operating on constrained storage. Mutable rows alone would obscure how estimates and adaptations were produced.

## Decision

Persist authoritative domain facts in checksummed append-only journals partitioned by data class and session. Build versioned query projections for runtime reads. Use atomic batches for event plus projection checkpoints, idempotent handlers, and explicit migration/upcaster code.

Raw frame payloads and large feature arrays use bounded chunk stores referenced by digest rather than aggregate journals. Detect torn writes and corruption at startup; recover to the last verified checkpoint and surface data loss. Compaction preserves provenance and follows retention policy.

## Consequences

### Positive
- Auditable lineage, deterministic reconstruction, and migration safety.
- Corruption boundaries are explicit.

### Negative
- Projection rebuild and compaction logic are operationally complex.
- Storage use must be carefully budgeted.

### Neutral
- This is selective event journaling, not universal event sourcing of ephemeral samples.

## Links

**Depends on**: [ADR-007](ADR-007-use-ruvector-as-primary-personalization-memory.md), [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-012](ADR-012-use-versioned-contracts-and-an-anti-corruption-layer.md), [ADR-017](ADR-017-standardize-identifiers-time-units-and-numeric-semantics.md)
