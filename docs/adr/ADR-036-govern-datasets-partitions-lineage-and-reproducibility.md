# ADR-036: Govern Datasets, Partitions, Lineage, and Reproducibility

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: datasets, governance, reproducibility, leakage

## Context

CSI and physiology datasets are sensitive and highly correlated within participant, room, and session. Leakage can produce misleading performance, while missing consent or license provenance blocks commercial use.

## Decision

Register every dataset with consent/purpose, license, collection protocol, hardware/firmware, schema, participant pseudonyms, sessions, rooms, demographics where justified, missingness, transformations, retention, and digest. Freeze split manifests before training; group by participant/session/room according to the target claim and prohibit window-level leakage.

Make all transformations content-addressed and reproducible from code plus manifest. Separate development, calibration, test, and post-deployment evaluation sets. Restrict access by purpose and record use. Publish datasheets and known limitations for every promoted model.

## Consequences

### Positive
- Model claims can be reproduced and audited.
- Commercial data rights and leakage risk are explicit.

### Negative
- Dataset operations and consent management are expensive.
- Strict splits may reveal lower performance.

### Neutral
- De-identification reduces but does not eliminate RF biometric privacy risk.

## Links

**Depends on**: [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md), [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-028](ADR-028-govern-retention-export-backup-recovery-and-deletion.md), [ADR-031](ADR-031-use-immutable-feature-and-evidence-artifacts.md)
