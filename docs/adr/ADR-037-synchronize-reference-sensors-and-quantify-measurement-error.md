# ADR-037: Synchronize Reference Sensors and Quantify Measurement Error

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: validation, reference-sensors, synchronization, measurement

## Context

Physiology validity depends on agreement with reference sensors, not merely correlation or visually plausible traces. Clock misalignment and reference error can dominate cardiac and transition metrics.

## Decision

Define a reference-acquisition port and protocol for respiratory belt, ECG/validated PPG, task events, and self-report. Record device clocks, sync pulses or shared triggers, resampling method, alignment uncertainty, reference quality, operator actions, and exclusions.

Predefine modality-specific agreement metrics, bias and limits of agreement, event tolerance, missingness, failure rate, coverage, and acceptable error. Exclude neither difficult samples nor participants post hoc without preserving and reporting the reason. Keep reference adapters out of production runtime unless separately authorized.

## Consequences

### Positive
- Physiological claims rest on traceable measurement validity.
- Clock and reference uncertainty are quantified.

### Negative
- Specialized equipment and trained protocol execution are required.
- Synchronization increases study complexity.

### Neutral
- Reference devices are not assumed perfect; their uncertainty is part of analysis.

## Links

**Depends on**: [ADR-008](ADR-008-promote-capabilities-through-preregistered-validation-gates.md), [ADR-017](ADR-017-standardize-identifiers-time-units-and-numeric-semantics.md), [ADR-036](ADR-036-govern-datasets-partitions-lineage-and-reproducibility.md)
