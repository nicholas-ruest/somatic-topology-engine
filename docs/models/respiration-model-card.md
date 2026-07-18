# Respiration estimator model card

Status: **disabled pending promotion**

## Intended use

This package estimates breathing cadence for a single consenting, stationary participant in a qualified site. It is a wellness-oriented, non-medical signal and must not be used for diagnosis, treatment, emergency detection, sleep-apnea detection, or clinical monitoring.

## Model and inputs

- Capability: `respiration-v1`
- Implementation: deterministic Rust baseline
- Input: promoted signal-observation evidence, never raw identity data
- Minimum evidence horizon: 30 seconds
- Output: breaths per minute, calibrated confidence, declared error bound, model version, source evidence, and evidence age
- Presentation refresh does not shorten the evidence horizon

The exact DSP schema, calibration profile, model version, and operating-envelope version must be content-addressed in release evidence before activation.

## Mandatory abstention

No estimate is emitted when validation promotion is absent or withdrawn, motion exceeds the stillness requirement, signal quality is insufficient, evidence duration is short, calibration is invalid, input is out of distribution, or the operating envelope is violated. Configuration cannot bypass a failed promotion gate.

## Validation status

Engineering known-answer tests are not scientific validity evidence. A promotion requires a frozen protocol, participant/session/day-held-out data, respiratory-belt reference alignment with uncertainty and reference quality, preregistered agreement/error/coverage gates, immutable results, and the reference deployment resource budget.

No human-reference result or Raspberry Pi benchmark is currently claimed by this card. Until those artifacts exist and the validation registry records promotion, UI and API projections must show the capability as unavailable, never medical-grade.

## Known limitations

CSI respiration is sensitive to posture, gross motion, multiple people, pets, fans, multipath changes, packet loss, interference, room geometry, hardware/firmware, calibration drift, and distance/orientation. Supported scope does not include multi-person source separation, cardiac coherence, heart-rate variability, stress, affect, workload, or decision labels.

## Monitoring and withdrawal

Integrity, calibration, scope, quality, coverage, latency, and resource signals are monitored locally without diagnostic payloads. Promotion is withdrawn or suspended when its exact evidence is missing, expired, revoked, incompatible, or a safety/quality gate fails. Prior negative evidence remains immutable.
