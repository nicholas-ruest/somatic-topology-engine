# ADR-043: Use Guided Installation, Site Qualification, and Acceptance

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: installation, commissioning, site, diagnostics

## Context

CSI performance depends on AP, channel, geometry, room dynamics, interference, and hardware health. A device cannot be commercially installed by merely powering it on.

## Decision

Provide an offline guided commissioning workflow that verifies hardware revision, firmware, peripherals, power/thermal health, AP/transmitter, channel/bandwidth, packet rate, geometry, authorization, storage, clock, and calibration. Measure empty-room and participant quality, interference, drift stability, and capability-specific evidence coverage.

Produce a signed site-acceptance record with supported operating envelope and failed/deferred capabilities. Block unsupported capabilities rather than lowering thresholds. Provide repeatable requalification after relocation, AP/firmware change, room change, repair, or persistent drift.

## Consequences

### Positive
- Field configuration becomes measurable and supportable.
- Prevents unsupported environments from appearing production-ready.

### Negative
- Installation takes time and may require trained support.
- Some sites will fail acceptance.

### Neutral
- Passing engineering acceptance does not validate cognitive claims.

## Links

**Depends on**: [ADR-029](ADR-029-version-csi-calibration-baselines-and-operating-geometry.md), [ADR-039](ADR-039-handle-power-thermal-storage-and-watchdog-failures.md), [ADR-040](ADR-040-version-peripheral-drivers-and-design-accessible-interactions.md)
