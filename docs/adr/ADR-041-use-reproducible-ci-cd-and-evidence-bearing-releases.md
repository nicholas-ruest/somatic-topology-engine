# ADR-041: Use Reproducible CI/CD and Evidence-Bearing Releases

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: ci, cd, releases, provenance

## Context

A production release combines source, dependencies, firmware compatibility, contracts, models, configuration, migrations, tests, benchmarks, and documentation. Passing a generic build job is insufficient.

## Decision

Use protected, review-gated CI with pinned toolchains and hermetic or containerized builds. Produce reproducible ARM artifacts, SBOMs, test/coverage reports, fuzz status, benchmark comparisons, HIL/soak evidence, security results, model cards, schema compatibility, migration/rollback results, and signed provenance.

Promote immutable artifacts across development, candidate, pilot, and production channels; never rebuild between stages. Require two-person approval for production signing and emergency releases, with documented break-glass audit. Release manifests bind every artifact and supported hardware/configuration combination.

## Consequences

### Positive
- A release is a verifiable evidence bundle.
- Production bits are identical to tested bits.

### Negative
- CI infrastructure and signing ceremonies add latency and cost.
- HIL and soak gates constrain release frequency.

### Neutral
- Fast development builds remain unsigned and cannot activate in production mode.

## Links

**Depends on**: [ADR-021](ADR-021-enforce-slos-and-resource-budgets-with-benchmarks.md), [ADR-022](ADR-022-adopt-a-replay-first-multi-layer-test-strategy.md), [ADR-023](ADR-023-govern-dependencies-licenses-and-software-supply-chain.md), [ADR-026](ADR-026-use-signed-reproducible-updates-with-atomic-rollback.md), [ADR-038](ADR-038-require-hardware-in-loop-fault-injection-and-soak-testing.md)
