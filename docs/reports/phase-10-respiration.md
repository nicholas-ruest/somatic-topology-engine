# Phase 10 respiration validation report

Status: **engineering implementation under validation; capability disabled**

## Exact gate result

`respiration-v1`: **NOT PROMOTED**.

The current implementation can be exercised with deterministic known-answer evidence, but no completed authorized human respiratory-belt study is recorded. Consequently there is no defensible reference agreement, held-out-day coverage, or human failure-rate result. The validation registry must return disabled, and CLI/UI/API surfaces must describe the output as non-medical and unavailable absent promotion.

## Evidence available

- Rust domain policy tests for minimum horizon, stillness, quality, OOD, calibration, operating envelope, and validation promotion.
- Immutable assessment repository tests, including idempotent retry and conflicting replacement rejection.
- Validation-registry adapter tests proving only an exact promoted Experiment Validation decision enables the capability.
- Governed CLI tests proving status and validation operations fail closed without fresh authorization.
- Deterministic estimator known-answer execution and current-host benchmark procedure.

## Reference agreement

Pending. An authorized frozen human protocol must align respiratory-belt reference observations using the Experiment Validation reference adapter and report alignment uncertainty, reference quality, bias, absolute error/RMSE, limits of agreement, coverage, abstention, failure rate, confidence intervals, calibration, and baseline comparison. Difficult or rejected samples must remain in the immutable report.

## Raspberry Pi validation

Pending physical reference-Pi execution. The procedure is documented in the Phase 10 benchmark report and script output. Current-host measurements must not be relabeled as Pi measurements.

## Safety and claims

This is not a medical device output. It cannot diagnose, treat, monitor, alarm, or support emergency decisions. Failed scientific or resource gates cannot be overridden by configuration. Cardiac, HRV-like, stress, valence, workload, and decision-state outputs remain outside this phase.
