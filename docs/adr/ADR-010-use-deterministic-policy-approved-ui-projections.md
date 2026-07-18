# ADR-010: Use Deterministic, Policy-Approved UI Projections

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: ui, oled, rgb, dspy, safety

## Context

Generating natural language every 500 ms with an LLM or prompt chain adds nondeterminism, latency, flicker, and unsupported interpretation. The current evidence does not justify displaying deep focus, valence, or decision threshold as facts.

## Decision

Render the live OLED and RGB state from an enumerated Rust `Projection` using fixed, versioned templates. Every projection is derived from approved claim level, quality, confidence band, evidence horizon, age, and abstention state.

Initially RGB represents signal quality or explicitly user-labeled arousal, not inferred valence. The OLED includes calibrating, contaminated, insufficient evidence, stale, unauthorized, and unavailable states.

DSPy.ts may optimize copy offline against a defined human-rated clarity metric. Approved text is reviewed and compiled back into deterministic templates; DSPy.ts cannot run inference or change state in the live path.

## Consequences

### Positive

- Predictable low-latency UI with testable safety language.
- Prevents generative text from amplifying model uncertainty.

### Negative

- Less expressive than free-form generated descriptions.
- Template/version governance is required.

### Neutral

- UI may refresh at 2 Hz while underlying evidence changes more slowly.

## Links

**Depends on**: [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md), [ADR-006](ADR-006-use-multi-timescale-event-time-processing.md), [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md)
**Related**: [Device Interaction context](../ddd/contexts/device-interaction.md)
