# ADR-040: Version Peripheral Drivers and Design Accessible Interactions

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: peripherals, accessibility, ux, hardware

## Context

OLED, RGB, touch, and DHT hardware may vary by CrowPi revision. Color-only or rapidly changing output can be inaccessible, and a touch strip is error-prone as the only control.

## Decision

Maintain signed hardware profiles specifying revision, buses, pins, addresses, voltage, timing, capabilities, and tested driver versions. Probe safely and refuse incompatible writes. Keep physical and simulator adapters contract-equivalent.

Design interaction states with text plus color, adjustable brightness, stable/debounced transitions, stale/error indicators, and alternatives for color-vision deficiency. Require deliberate gestures for destructive actions, feedback for accepted anchors, and an accessible local administrative path. User-test onboarding, calibration, consent, anchoring, faults, and shutdown.

## Consequences

### Positive
- Hardware variation and accessibility become release-tested.
- Reduces accidental anchors and misleading displays.

### Negative
- Small OLED/touch constraints limit interaction design.
- Multiple revisions expand HIL coverage.

### Neutral
- The touch strip remains the normal in-session input, not the sole maintenance interface.

## Links

**Depends on**: [ADR-010](ADR-010-use-deterministic-policy-approved-ui-projections.md), [ADR-011](ADR-011-abstract-crowpi-hardware-behind-rust-ports.md), [ADR-038](ADR-038-require-hardware-in-loop-fault-injection-and-soak-testing.md)
