# ADR-011: Abstract CrowPi Hardware Behind Rust Ports

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: crowpi, gpio, hardware, simulation

## Context

CrowPi integrates the Pi, OLED, RGB LEDs, touch controls, and DHT11, but revisions and pin mappings may differ. Hardware-only development would make CI and replay difficult. Environmental readings are useful covariates but have no validated direct mapping to model confidence.

## Decision

Define Rust ports for display, LEDs, touch, ambient environment, sensing indicator, and physical off state. Provide physical adapters selected by an explicit `HardwareRevision` and simulator adapters used by tests and development.

Probe peripherals at startup, report health, and degrade independently. Touch events are debounced domain input. DHT11 readings are timestamped environmental observations only; they cannot alter softmax temperature or confidence without a promoted validation protocol.

Pin and document voltage, bus, address, pin, timing, and safe shutdown behavior for the verified CrowPi unit before enabling physical output.

## Consequences

### Positive

- Most behavior is testable without hardware.
- Hardware revisions and driver changes remain outside the domain.

### Negative

- Adapter and simulator parity tests add work.
- The exact purchased unit must still be physically verified.

### Neutral

- CrowPi is the reference deployment, not a domain type.

## Links

**Depends on**: [ADR-001](ADR-001-adopt-a-rust-first-modular-monolith.md), [ADR-010](ADR-010-use-deterministic-policy-approved-ui-projections.md)
**Related**: [Device Interaction context](../ddd/contexts/device-interaction.md)
