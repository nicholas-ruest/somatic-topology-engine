# Quality Record Storage and Control

**Record ID:** STE-QMS-REC-001
**Owner:** Quality Lead
**Security owner:** Security & Privacy Lead

Quality records provide durable evidence of design, risk, verification, release, and field decisions. This document defines the repository convention now and the requirements for the future controlled record system; checking a template into Git does not constitute an approved record.

## Record classes

Design/ADR and change control; requirements/traceability; threat/privacy/hazard/risk acceptance; dependency/license/SBOM/provenance; verification/validation/fuzz/benchmark/HIL/soak; models/datasets/calibration; release/signing/commissioning; training/approval; incidents/vulnerabilities/complaints/CAPA; support/decommissioning. Diagnostic logs, security events, participant-visible history, scientific provenance, and domain audit remain separate stores under ADR-052 and are linked by non-sensitive identifiers only.

## Storage layout

- Reviewed templates and policies live under `docs/assurance/` in Git.
- Generated local evidence goes under `artifacts/quality/<record-id>/` and is ignored unless deliberately promoted to an approved immutable evidence store.
- A release evidence manifest contains record IDs, content digests, media type, producer/tool identity, source commit, release/hardware/model/config scope, creation UTC, retention class, access class, and authoritative URI.
- Sensitive evidence is encrypted in the controlled store; Git, public CI logs, issues, and support bundles must contain only approved redacted derivatives or digests.

## Record controls

Records are append-only after approval. Corrections create a signed superseding version and preserve the original. Approval binds human identity/role, decision, content digest, and trustworthy UTC time. Access is least-privilege and audited; export is authorized and redacted. Backups, restore tests, legal holds, retention, and deletion are defined per class. Hashes provide integrity evidence but do not replace authenticated storage, signatures, or provenance.

## Required metadata

`record_id`, title/type/version/status, owners, applicable ADR/requirement/hazard/claim IDs, release and environment scope, source/tool versions, procedure and acceptance criteria, complete result, deviations/negative results, attachments/digests, access/retention class, creation and review UTC times, signatures, supersedes/superseded-by, and authoritative location.

## Retention baseline

Before commercial release, qualified legal/regulatory review sets market- and record-specific periods. Until then, approved design, risk, release, incident, complaint, vulnerability, and CAPA records are retained indefinitely; participant-derived payloads follow the shortest authorized data-class period and must not be copied into quality records. “Indefinite” is an interim control, not a final commercial retention schedule.

## CI and release gate

CI checks schema, required fields, identifier uniqueness, link/digest resolution, accidental sensitive content, stale approvals, and prohibited artifact paths. Release fails if required records are missing, mutable, unsigned where required, outside their evaluated scope, or inaccessible to the reviewer. Evidence deletion or corruption opens an incident and invalidates dependent approval until restored and verified.

## References

- [Verification traceability](verification-traceability.md)
- [ADR evidence gate](adr-implementation-evidence.md)
- [ADR-041](../adr/ADR-041-use-reproducible-ci-cd-and-evidence-bearing-releases.md)
- [ADR-052](../adr/ADR-052-separate-audit-security-and-diagnostic-records.md)
