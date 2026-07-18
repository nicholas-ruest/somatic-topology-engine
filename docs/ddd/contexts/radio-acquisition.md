# Bounded Context: Radio Acquisition

## Purpose

Own the trustworthy conversion of supported Wi-Fi hardware output into validated, replayable CSI frames. This context integrates rvCSI/Nexmon through infrastructure adapters and remains ignorant of physiology or cognitive labels.

## Aggregate: `CaptureSession`

**Identity:** `CaptureSessionId`

**Entities:** `CaptureLink`, `FrameSequence`, `CalibrationRun`

**Value objects:** `CaptureProfile`, `FirmwareFingerprint`, `RadioChannel`, `PacketRate`, `LinkGeometry`, `FrameQuality`, `MonotonicTimestamp`, `CaptureFailure`

### Invariants

1. A session cannot start without active `CapturePermission` and a fully pinned `CaptureProfile`.
2. Profile identity is immutable after the first accepted frame.
3. Accepted frame timestamps and sequence numbers are monotonic within a link; gaps are recorded, never concealed.
4. Every frame records adapter, hardware, firmware, channel, bandwidth, and calibration provenance.
5. Malformed or implausible frames are rejected before crossing the public boundary.
6. Session health distinguishes accepted, degraded, rejected, and missing frames.
7. A stopped or revoked session cannot accept new frames.

### Commands

- `StartCapture`
- `AcceptRawFrame`
- `RecordFrameGap`
- `CompleteCalibration`
- `StopCapture`
- `RevokeCapture`

### Events

- `CaptureSessionStarted`
- `CsiFrameValidated`
- `CsiFrameRejected`
- `CaptureQualityDegraded`
- `CaptureProfileDriftDetected`
- `CaptureSessionStopped`

### Repository port

`CaptureSessionRepository` loads and stores session metadata. Bulk frame payloads use a separate append-only `FrameJournal` port so aggregate loading stays bounded.

## Rust boundaries

Domain types are pure Rust. `NexmonCaptureAdapter` and `RvcsiAdapter` live in infrastructure. The public application API publishes `ValidatedCsiFrameV1` through bounded async channels with explicit backpressure.
