# ADR-031: Use Immutable Feature and Evidence Artifacts

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: features, provenance, artifacts, ml

## Context

Training/runtime skew and missing provenance can make a high-performing model irreproducible. A vector without its window, DSP schema, quality, units, and source partition is not valid evidence.

## Decision

Define a content-addressed `FeatureArtifact` containing feature schema, ordered names/dimensions, units, DSP version, calibration profile, window policy, evidence references, quality, missingness, participant/session pseudonyms, partition role, and artifact digest. Artifacts are immutable; corrections create new linked versions.

Use the same Rust feature implementation for training export and edge inference whenever practical. Where batch tools use another language, require parity fixtures against Rust. Reject model activation when its required feature schema is unavailable or differs semantically.

## Consequences

### Positive
- Prevents silent training-serving skew.
- Supports exact lineage from output to frames and code.

### Negative
- Metadata overhead and artifact management increase.
- Feature evolution requires explicit migrations or retraining.

### Neutral
- Raw frame retention can expire while feature provenance remains, subject to policy.

## Links

**Depends on**: [ADR-018](ADR-018-use-an-append-only-journal-with-versioned-projections.md), [ADR-030](ADR-030-version-the-dsp-pipeline-and-require-numerical-replay.md)
