# ADR-017: Standardize Identifiers, Time, Units, and Numeric Semantics

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: types, time, units, numerics

## Context

RF frames, windows, estimates, experiments, and UI projections cross contexts and languages. Ambiguous timestamps, floating-point sentinels, or unitless values would break alignment and scientific interpretation.

## Decision

Use opaque UUIDv7-compatible IDs generated through a port, nanosecond monotonic time for intervals/order, and UTC wall time only for external correlation and audit. Carry clock source, synchronization quality, and uncertainty where alignment matters.

Represent physical units with Rust newtypes, probabilities as validated finite values in `[0,1]`, missing results as enums/options, and never NaN/Infinity across contracts. Specify float precision, rounding, endianness, and numerical tolerances per algorithm and artifact.

## Consequences

### Positive
- Prevents unit and clock ambiguity across the pipeline.
- Supports deterministic replay and reference-sensor alignment.

### Negative
- More wrapper types and explicit conversions.
- Wall/monotonic clock correlation requires periodic sampling.

### Neutral
- Serialized timestamps use integers plus declared scale, not language-specific date objects.

## Links

**Depends on**: [ADR-006](ADR-006-use-multi-timescale-event-time-processing.md), [ADR-012](ADR-012-use-versioned-contracts-and-an-anti-corruption-layer.md)
