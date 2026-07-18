# Verification Traceability Standard

**Record ID:** STE-QMS-VT-001
**Owner:** Quality Lead

Traceability is bidirectional: every applicable decision, domain invariant, threat, hazard, claim, requirement, and operating limit maps to verification evidence; every verification record states which obligations it verifies. Planned work is not evidence.

## Stable identifiers

Use `ADR-NNN`, `DDD-<context>-INV-NNN`, `REQ-<area>-NNN`, `THR-NNN`, `HAZ-NNN`, `CLM-NNN`, `CTRL-NNN`, `VER-NNN`, `BENCH-NNN`, `DEV-NNN`, `RA-NNN`, and `QR-NNN`. IDs are never reused. Superseded records retain history and links.

## Trace row

```yaml
trace_id: TRACE-NNN
obligation_ids: []
control_ids: []
verification_id: VER-NNN
method: analysis | inspection | unit | property | fuzz | replay | integration | security | hil | soak | benchmark | validation-study
procedure_version: ""
code_commit: ""
artifact_digests: []
environment:
  toolchain: ""
  target: ""
  hardware_firmware: ""
  configuration_digest: ""
predeclared_acceptance_criteria: ""
result: pass | fail | inconclusive | not-run
negative_and_deviation_record_ids: []
executed_by: ""
executed_at_utc: ""
independent_reviewer: ""
reviewed_at_utc: ""
quality_record_uri: ""
```

## Evidence rules

Passing evidence is reproducible from immutable or content-addressed inputs, records exact versions and scope, includes complete machine-readable results, applies a predeclared objective threshold, and is independently reviewed. Flaky, skipped, manually edited, expired, environment-mismatched, or non-resolving evidence cannot close an obligation. A manual verification needs a versioned procedure, trained executor, contemporaneous observations, and attachments; “reviewed code” is not by itself verification.

Critical controls require a negative test proving the unsafe path is blocked and a fault/recovery test proving the safe state. Scientific capability evidence additionally requires frozen protocols/splits, reference quality, uncertainty, negative results, and operating-envelope scope. Performance evidence identifies warm-up, samples, statistics, resource limits, and reference hardware. Security evidence separates absence of a scanner finding from proof of non-exploitability.

## Coverage and release

CI validates identifier syntax, link resolution, duplicate IDs, stale/expired evidence, and that changed obligations have an impact decision. A release traceability export lists uncovered obligations and blocks release if any applicable critical hazard/control, prohibited-use guard, enabled claim, or release gate lacks passing evidence. Non-applicability requires signed rationale and scope, not deletion.

## References

- [ADR implementation evidence](adr-implementation-evidence.md)
- [Claim–evidence matrix](claim-evidence-matrix.md)
- [ADR-022](../adr/ADR-022-adopt-a-replay-first-multi-layer-test-strategy.md)
- [ADR-045](../adr/ADR-045-maintain-a-safety-case-and-hazard-control-traceability.md)
