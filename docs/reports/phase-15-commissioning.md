# Phase 15 secure operator and commissioning evidence

Status: authenticated command contract and synthetic workflow verified; physical reference-device qualification pending.

The operator schema covers status, doctor, authorization, capture test, hardware probe, calibration, replay, validation export, models, capability policy, redacted support bundle, signed updates, data lifecycle, recovery, reset, commissioning, and requalification. Every request requires a bounded idempotency key and fresh exact authorization. Stable JSON uses a versioned response envelope; destructive reset requires dry-run or explicit confirmation. Existing specialized replay/storage/model/validation commands retain narrower typed parsers.

Synthetic commissioning exercises all mandatory checks, signed acceptance verification, enabled/blocked coverage, and threshold non-override. It is software evidence only. No physical site, power, thermal, peripheral, packet-rate, geometry, calibration, interference, or CrowPi acceptance is claimed.

Run `scripts/validate-commissioning.sh`. A fresh physical reference device does not satisfy the exit gate until its exact signed profile and HIL/site evidence are attached.

The focused suite passes four commissioning qualification/tamper/recovery tests, four broad operator contract tests, and strict Clippy across the commissioning and CLI integration. Exact idempotent retries return the same receipt; reuse of a key for a different command fails closed.
