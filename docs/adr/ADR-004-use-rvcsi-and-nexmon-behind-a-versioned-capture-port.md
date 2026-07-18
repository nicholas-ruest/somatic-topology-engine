# ADR-004: Use rvCSI and Nexmon Behind a Versioned Capture Port

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: csi, rvcsi, nexmon, hardware, replay

## Context

Raspberry Pi 4 CSI requires compatible Broadcom firmware, Nexmon, a configured radio link, and stable capture settings. rvCSI provides Rust validation, normalized frames, DSP primitives, and replay, but upstream APIs and firmware can change.

## Decision

Adopt rvCSI as the preferred normalized CSI runtime and Nexmon as the initial Pi 4 capture mechanism, both behind STE's `CsiCaptureSource` and `FrameJournal` ports.

Pin and record Pi revision, chipset, OS, kernel, firmware, rvCSI revision, adapter ABI, AP/transmitter, band, channel, bandwidth, packet pattern, antenna/link geometry, and calibration. Store raw PCAP when authorized and normalized replay fixtures. Validate every frame before publication.

No downstream context may depend on Nexmon headers, rvCSI internal types, or live hardware handles.

## Consequences

### Positive

- Uses the strongest existing Rust fit while containing upstream change.
- Enables deterministic hardware-free development and failure reproduction.

### Negative

- A known-good image and compatibility matrix must be maintained.
- Physical traffic source and geometry become documented system dependencies.

### Neutral

- ESP32 or future 802.11bf adapters can implement the same port later.

## Links

**Depends on**: [ADR-001](ADR-001-adopt-a-rust-first-modular-monolith.md), [ADR-002](ADR-002-keep-the-edge-runtime-local-and-deterministic.md)
**Related**: [Radio Acquisition context](../ddd/contexts/radio-acquisition.md)
