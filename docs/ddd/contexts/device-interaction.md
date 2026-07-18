# Bounded Context: Device Interaction

## Purpose

Own CrowPi peripheral behavior, deterministic display projection, touch anchors, simulation, and safe degradation. It translates domain state into interaction without inventing scientific meaning.

## Aggregate: `InteractionSession`

**Identity:** `InteractionSessionId`

**Entities:** `DisplayState`, `LedState`, `TouchGesture`, `PeripheralHealth`

**Value objects:** `Projection`, `DisplayLabel`, `ConfidenceBand`, `QualityIndicator`, `RgbColor`, `RefreshPolicy`, `PeripheralId`, `AnchorRequest`, `HardwareRevision`

### Invariants

1. Only enumerated, policy-approved projections may reach OLED or RGB outputs.
2. Unknown, contaminated, unauthorized, and unavailable states have explicit projections.
3. Display refresh cadence is independent of evidence horizon and shows stale status when appropriate.
4. A touch anchor requires a debounced physical gesture and active authorization.
5. RGB does not claim emotional valence until that construct passes validation; initially it encodes signal quality or user-labeled arousal.
6. Peripheral failures cannot stop capture supervision or silently freeze a stale “healthy” display.
7. The simulator and physical adapters implement the same Rust ports.

### Commands

- `StartInteractionSession`
- `RenderProjection`
- `HandleTouchGesture`
- `RecordPeripheralFailure`
- `StopInteractionSession`

### Events

- `InteractionSessionStarted`
- `ProjectionRendered`
- `AnchorRequested`
- `PeripheralFailed`
- `InteractionSessionStopped`

### Repository port

`InteractionSessionRepository` stores only the current/recoverable session state. `InteractionAuditJournal` separately records projection and touch events when policy permits. GPIO/I2C/SPI drivers are infrastructure adapters selected by CrowPi hardware revision.

## Rust and TypeScript boundaries

Physical I/O and live templates are Rust. DSPy.ts may optimize an offline copy deck, whose approved results are compiled back into enumerated Rust templates.
