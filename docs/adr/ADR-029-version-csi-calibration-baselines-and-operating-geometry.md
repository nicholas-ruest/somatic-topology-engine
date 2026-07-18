# ADR-029: Version CSI Calibration, Baselines, and Operating Geometry

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: csi, calibration, geometry, drift

## Context

CSI depends on hardware, firmware, AP traffic, channel, antenna geometry, room layout, occupancy, and time. Treating calibration as a one-time scalar would produce silent domain shift.

## Decision

Create immutable `CalibrationProfile` artifacts covering capture profile, empty-room and reference states, geometry, environment, packet statistics, quality distribution, algorithms, and acceptance results. Bind every observation to one profile and scope models to compatible profiles.

Detect baseline drift online using preregistered thresholds and change-point evidence. Drift transitions the affected capability to recalibration-required; it never silently updates the baseline used for confirmatory evaluation. Support guided setup, periodic verification, and explicit profile supersession.

## Consequences

### Positive
- Environmental change is observable and traceable.
- Model scope becomes enforceable.

### Negative
- Installation and room changes require user workflow.
- Calibration profiles add storage and compatibility dimensions.

### Neutral
- Online adaptive baselines may be used for engineering quality only when isolated from scientific evaluation baselines.

## Links

**Depends on**: [ADR-004](ADR-004-use-rvcsi-and-nexmon-behind-a-versioned-capture-port.md), [ADR-005](ADR-005-make-quality-gating-and-abstention-mandatory.md), [ADR-016](ADR-016-use-validated-layered-configuration-and-secret-isolation.md)
