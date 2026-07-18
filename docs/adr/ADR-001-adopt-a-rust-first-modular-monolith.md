# ADR-001: Adopt a Rust-First Modular Monolith

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: architecture, rust, ddd, edge

## Context

STE must capture and process high-rate CSI, enforce scientific and privacy invariants, run on a Raspberry Pi 4, and integrate some TypeScript-first projects. Premature services would add serialization, deployment, and failure modes to a single-device system. A TypeScript-first core would conflict with the project's performance, safety, and language goals.

## Decision

Build STE initially as a Rust modular monolith organized by the bounded contexts in [the DDD context map](../ddd/context-map.md). Enforce boundaries through crates/modules, private internals, public application ports, and versioned integration contracts.

Use Rust for the domain, application services, async runtime, DSP, inference, storage, policy, device I/O, CLI, and experiment metrics wherever practical. Use TypeScript only for maintained TypeScript-first capabilities behind explicit sidecar or FFI adapters. Use another language only when neither Rust nor TypeScript provides a scientifically or operationally adequate option.

Do not split contexts into network services without measured isolation, scaling, deployment, or fault-containment need.

## Consequences

### Positive

- One deployable edge process with predictable resource use and compile-time domain separation.
- Rust owns the safety- and latency-critical path.
- Contexts can be extracted later without designing a distributed system now.

### Negative

- Rust integration work may be required for TypeScript-first upstream tools.
- Modular boundaries require review and CI enforcement because they share a process.

### Neutral

- The initial number of Cargo crates may be smaller than the logical context count.

## Links

**Depends on**: [research findings](../research.md)
**Related**: [DDD context map](../ddd/context-map.md)
