# ADR-028: Govern Retention, Export, Backup, Recovery, and Deletion

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: data-lifecycle, retention, backup, deletion

## Context

Raw CSI, derived evidence, anchors, models, and audits have different sensitivity and recovery value. Unbounded retention threatens privacy and storage reliability; simplistic deletion may leave projections, vectors, backups, or keys behind.

## Decision

Define lifecycle policy per data class: collection purpose, default retention, maximum retention, export eligibility, backup eligibility, recovery objective, deletion method, and legal/ethics hold rules. Default raw CSI to the shortest practical retention and derived summaries to explicit participant choice.

Exports are portable, encrypted, signed, and manifest-driven. Backups are opt-in/local-first, encrypted with separate keys, restore-tested, and covered by deletion propagation. Deletion traverses journals, projections, vectors, caches, exports under management, and backups; completion produces a payload-minimized audit proof.

## Consequences

### Positive
- Privacy promises and recovery expectations are operationally testable.
- Storage remains bounded.

### Negative
- Complete deletion across derived artifacts is complex.
- Short raw retention can limit later debugging and research.

### Neutral
- Audit proof records completion, not deleted sensitive content.

## Links

**Depends on**: [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-018](ADR-018-use-an-append-only-journal-with-versioned-projections.md), [ADR-025](ADR-025-use-device-identity-encryption-and-managed-keys.md)
