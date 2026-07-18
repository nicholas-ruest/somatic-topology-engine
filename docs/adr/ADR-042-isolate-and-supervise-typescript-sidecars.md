# ADR-042: Isolate and Supervise TypeScript Sidecars

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: typescript, sidecars, isolation, nodejs

## Context

AgentDB, DSPy.ts, and Ruflo are TypeScript-first, but Node.js dependencies and agent behavior must not weaken deterministic runtime, privacy, or resource guarantees.

## Decision

Do not include TypeScript in the critical capture-to-projection path. Run an optional sidecar only for a justified capability, under a dedicated unprivileged account/process sandbox, offline by default, with CPU/memory/storage limits, authenticated local IPC, explicit allowlisted contracts, and no direct hardware or authoritative-store access.

The Rust runtime starts, health-checks, rate-limits, and can terminate the sidecar without losing core operation. Pin Node/npm packages and lockfiles. Sidecar absence or failure yields a documented degraded optional feature, never a fabricated fallback. DSPy.ts remains offline copy optimization unless a later ADR changes scope.

## Consequences

### Positive
- Preserves Rust-first safety and bounds ecosystem risk.
- Optional integrations cannot block core sensing.

### Negative
- IPC and sandboxing add operational complexity.
- Some upstream APIs require adapter maintenance.

### Neutral
- A sidecar must justify its own resource and commercial-license budget.

## Links

**Depends on**: [ADR-001](ADR-001-adopt-a-rust-first-modular-monolith.md), [ADR-012](ADR-012-use-versioned-contracts-and-an-anti-corruption-layer.md), [ADR-014](ADR-014-use-a-bounded-asynchronous-runtime-with-backpressure.md), [ADR-019](ADR-019-evolve-contracts-with-explicit-compatibility-rules.md), [ADR-024](ADR-024-maintain-a-living-threat-model-and-trust-boundaries.md)
