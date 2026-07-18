# ADR-052: Separate Audit, Security, and Diagnostic Records

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: audit, logs, redaction, integrity

## Context

Debug logs, participant history, security events, and compliance audit have different audiences and retention. Combining them increases disclosure, tampering, and accidental-deletion risk.

## Decision

Maintain separate schemas and stores for diagnostics, security events, domain audit, participant-visible history, and scientific provenance. Apply independent access, integrity, retention, export, and redaction policies. Audit privileged changes and policy decisions with hashes/identifiers rather than sensitive payloads.

Centralize structured redaction and test it with forbidden-field canaries. Forbid free-form dumping of configs, frames, embeddings, prompts, model inputs/outputs, secrets, and participant labels. Clock uncertainty and dropped-log counts are observable. Critical audit writes fail the associated privileged command rather than disappearing silently.

## Consequences

### Positive
- Least-privilege access and lifecycle become practical.
- Support bundles cannot accidentally inherit the full audit history.

### Negative
- Multiple stores and correlation tools add complexity.
- Over-redaction can hinder diagnosis.

### Neutral
- Tamper evidence detects modification; it does not guarantee truthful event generation.

## Links

**Depends on**: [ADR-018](ADR-018-use-an-append-only-journal-with-versioned-projections.md), [ADR-020](ADR-020-build-local-first-observability-and-support-bundles.md), [ADR-025](ADR-025-use-device-identity-encryption-and-managed-keys.md), [ADR-028](ADR-028-govern-retention-export-backup-recovery-and-deletion.md)
