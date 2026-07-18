# Edge model package card

Status: lifecycle infrastructure only; no latent-state production capability promoted.

## Package identity and provenance

Every package must bind a stable package/version identifier to immutable weight, feature-schema, preprocessing, calibration, and known-answer digests. It also declares its model format/runtime, source and training/evaluation lineage, SPDX license expression, supplier provenance, supported software/firmware/hardware matrix, operating scope, resource profile, and rollback constraints.

Unsigned packages and packages with unknown provenance, incompatible licensing, mismatched digests, unsupported operators, incompatible feature semantics, or unsupported deployment profiles remain quarantined.

## Activation gates

Activation requires all of the following exact evidence:

- trusted signature verification over canonical package content;
- artifact digest and size/bounds validation;
- software, firmware, hardware, feature, and preprocessing compatibility;
- license/provenance and security approval;
- deterministic known-answer parity;
- calibrated uncertainty and OOD/scope policy;
- scientific promotion for the exact capability/model;
- unexpired signed capability policy for the exact jurisdiction, purpose, participant scope, operating envelope, and device profile;
- applicable current-host and reference-device resource gates.

Configuration and compile-time inclusion cannot bypass these gates.

## Lifecycle and recovery

Registry states are append-only transitions through quarantined, evaluated, promoted, active, suspended, retired, and revoked. Activation swaps the candidate atomically only after verification and records the previous active package. A failed post-activation health or known-answer check suspends the candidate and atomically restores the previous verified package. Revocation is terminal; downgrade protection permits only an explicitly authorized, compatible rollback target.

## Output and claims

Raw model scores never reach product UI/API surfaces. Calibrated outputs are emitted only inside the signed scope; OOD, expired, unpromoted, policy-disabled, or unhealthy inputs abstain. Experimental outputs remain visibly non-production and isolated. This package card does not assert medical, affective, workload, valence, stress, or decision-state validity.
