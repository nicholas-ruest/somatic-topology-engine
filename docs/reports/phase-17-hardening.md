# Phase 17 integrated hardening evidence

Status: current-host regression and deterministic fault evidence available; production hardening incomplete.

`scripts/validate-phase17-hardening.sh` runs runtime fault/SLO tests, IPC security, supervision, journal recovery, model lifecycle/rollback, assurance controls, architecture checks, warning-free Clippy, the current-host benchmark suite, and its machine-readable self-comparison. The output records explicit pending Pi temperature, power, and soak flags.

A self-comparison proves harness wiring, not absence of regression against a prior release. Release CI must supply an immutable approved baseline and compare the exact candidate. No code optimization is recorded in Phase 17 because no measured bottleneck was shown to block a declared current-host gate; speculative changes would add fidelity risk.

The focused harness passes 22 runtime IPC/SLO/fault/supervision tests, eight journal recovery tests, four model lifecycle tests, four release-hardening update/backup/key/reset tests, assurance/architecture gates, and workspace Clippy. One x86_64 host run observed startup-to-exit p95 3,555 µs, idle RSS 2,212 KiB, acquisition approximately 14.13 million frames/second, observation approximately 1.19 million frames/second and 2,328 windows/second, with zero numerical delta in the observation fixture. These are current-host development observations, not reference-Pi release claims.
