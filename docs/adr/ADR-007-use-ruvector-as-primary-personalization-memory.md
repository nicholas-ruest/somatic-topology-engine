# ADR-007: Use RuVector as Primary Personalization Memory

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: memory, personalization, ruvector, agentdb

## Context

STE needs local pattern memory, anchors, provenance, similarity retrieval, feedback, and deletion. RuVector provides a Rust-native embedded path. AgentDB is TypeScript-first and overlaps significantly. Running both as authoritative stores would create unclear ownership and feedback divergence.

## Decision

Use an append-only Rust domain journal plus RuVector/RVF as the initial authoritative vector-memory adapter behind `VectorMemory`. The domain owns anchors, rewards, data partitions, adaptation lineage, retention, and deletion semantics; the vector engine owns indexing and retrieval mechanics.

AgentDB is not part of the initial runtime. It may be evaluated later as an optional TypeScript sidecar for a demonstrated capability missing from RuVector. It must never become an independent source of domain truth.

Evaluation partitions are read-only and cannot feed online learning. Claims of improvement require prospective held-out results.

## Consequences

### Positive

- Rust-native local storage with one authority and explicit learning provenance.
- Avoids dual-write and ranking divergence.

### Negative

- Some AgentDB convenience APIs and integrations are deferred.
- RuVector integration maturity and ARM resource use require benchmarks.

### Neutral

- The port permits replacement if measured evidence favors another local engine.

## Links

**Depends on**: [ADR-001](ADR-001-adopt-a-rust-first-modular-monolith.md), [ADR-002](ADR-002-keep-the-edge-runtime-local-and-deterministic.md)
**Related**: [Personalization Memory context](../ddd/contexts/personalization-memory.md), [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md)
