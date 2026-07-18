# Bounded Context: Signal Observation

## Purpose

Transform validated CSI frames into signal-domain observations while preserving evidence, quality, and deterministic replay. This context owns DSP windows, not physiological meaning.

## Aggregate: `ObservationWindow`

**Identity:** `ObservationWindowId`

**Entities:** `FrameEvidenceSet`, `FeatureSet`, `QualityAssessment`

**Value objects:** `WindowBounds`, `WindowPolicy`, `MotionEnergy`, `PeriodicityCandidate`, `PresenceScore`, `SignalQuality`, `EvidenceReference`, `AlgorithmVersion`

### Invariants

1. Window bounds are non-empty, ordered, and use a named immutable policy.
2. Every feature traces to accepted frame references and an algorithm version.
3. Features never carry physiology or mental-state labels.
4. Motion, missingness, baseline drift, and interference contribute to quality explicitly.
5. A closed window is immutable and replay must reproduce its deterministic features within declared numerical tolerance.
6. A contaminated window remains stored as contaminated; it is not silently discarded or normalized into a clean result.

### Commands

- `OpenObservationWindow`
- `AppendValidatedFrame`
- `RecordSignalAnomaly`
- `CloseObservationWindow`

### Events

- `ObservationWindowOpened`
- `SignalAnomalyObserved`
- `ObservationWindowContaminated`
- `ObservationWindowClosed`

### Repository port

`ObservationWindowRepository` persists closed window metadata and feature records. Source frames remain referenced through the acquisition journal.

## Rust boundaries

DSP is implemented in Rust using rvCSI/RuView primitives where their pinned APIs meet requirements. SIMD or native acceleration is permitted behind pure Rust traits and golden replay tests.
