# Phase 17 release-blocking gates

These gates are conjunctive. Missing evidence is a failure, not a waiver.

Current-host regression thresholds compare immutable baselines and candidates with the existing machine-readable harness: startup p95, idle RSS, runtime queue p95, acquisition throughput, observation frame throughput, and observation-window throughput may regress by at most 10%. Scientific accuracy, numerical replay, calibration, selective-risk, privacy, authorization, abstention, erasure, update integrity, and rollback are zero-regression gates regardless of performance.

Reference-Pi release gates additionally require declared capture continuity, valid-window coverage, projection freshness, CPU, RSS, storage growth, temperature, power, voltage/throttling, recovery, HIL fault response, and multi-day soak distributions on the exact hardware/image/profile. A current-host pass cannot satisfy those gates.

Security release gates require parser/model/IPC fuzz campaigns with retained corpus and coverage, penetration testing by qualified personnel, dependency/model/firmware review, secret and unsafe-code scans, key compromise/rotation exercises, and closure or explicit acceptance of every finding. Incident, update/rollback, backup/restore/reset, and recovery exercises require signed evidence and accountable reviewers.

Optimization is permitted only for a profiled bottleneck. The change must rerun numerical parity, scientific gates, security/privacy controls, regression comparison, and reference-device thermal/power/HIL/soak evidence. Phase 17 records no optimization because the current evidence does not identify a safe release-blocking bottleneck.
