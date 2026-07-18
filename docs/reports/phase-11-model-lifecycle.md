# Phase 11 model lifecycle validation

Status: current-host lifecycle implementation verified; reference-device gates pending.

## Required exit gates

- unsigned, corrupted, incompatible, unlicensed, unpromoted, revoked, OOD, expired, or policy-disabled models cannot serve;
- canonical package verification occurs before registry promotion or activation;
- activation is atomic and preserves the prior active package;
- a failing activation known-answer or health check leaves the prior package serving;
- post-activation failure suspends the candidate and rollback restores the prior known-answer behavior;
- immutable lifecycle decisions and negative evidence remain available for audit;
- experimental results cannot enter production projections.

## Scientific and operational interpretation

Passing lifecycle tests establishes artifact and rollout mechanics, not validity of any latent construct. Capability policy may narrow an Experiment Validation promotion but can never broaden or replace it. Calibration, OOD, resource, hardware, and scientific evidence stay bound to the exact package digest and operating scope.

The validation script records current-host package verification/activation throughput only. Raspberry Pi, HIL, thermal, power, soak, and human-validity evidence remain pending unless separately produced on declared equipment under frozen protocols.

## Executed verification

`scripts/validate-model-lifecycle.sh` passes 10 model-runtime package, registry, calibration, OOD, selective-risk, and signed-policy tests; 3 governed lifecycle/KAT integration tests; 2 corrupt/incompatible fixture tests; and strict Clippy. A candidate failing pre-activation KAT leaves the prior model active. A candidate failing post-activation health is suspended and the previous package is restored and KAT-checked. Direct CLI mutation remains unavailable without authenticated local IPC and a fresh exact policy decision.
