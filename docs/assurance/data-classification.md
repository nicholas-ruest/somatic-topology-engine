# Data Classification and Handling Baseline

**Record:** ASSURANCE-DATA-001
**Version:** 1.0.0
**Status:** Accepted baseline
**Effective date:** 2026-07-18
**Owner:** Privacy and Data Governance Lead
**Approvers:** Product Security Lead; Safety Lead; Product Owner
**Review triggers:** new data class, interface, export, deployment, jurisdiction, claim, or material retention change; otherwise every 90 days

This record implements ADR-009, ADR-024, ADR-025, ADR-028, ADR-047, and ADR-052. It is a technical baseline, not a substitute for jurisdiction-specific privacy or regulatory review.

## Classification levels

| Level | Meaning | Default handling |
|---|---|---|
| `STE-P0 Public` | Deliberately public product material | Integrity protected; approved publication only |
| `STE-P1 Internal` | Non-public operational material with low privacy impact | Authenticated access; integrity and retention controls |
| `STE-P2 Confidential` | Security, commercial, or pseudonymous operational data | Least privilege; encryption at rest/in transit; audited export |
| `STE-P3 Sensitive sensing` | Data that reveals or can derive presence, behavior, physiology, identity, or latent state | Explicit active authorization; per-class encryption; strict purpose and retention; no diagnostic logging |
| `STE-P4 Restricted secrets` | Keys, credentials, recovery material, exploitable security evidence | Hardware-backed storage when available; no general export/logging; dual-control recovery/rotation |

Classification follows the highest-risk content in a record. Pseudonymization does not lower a sensing record below `STE-P3`; encryption changes protection, not classification.

## Data inventory

| Class ID | Examples | Level | Permitted purpose | Default retention | Export / backup | Required controls | Accountable owner |
|---|---|---:|---|---|---|---|---|
| DC-01 Raw CSI | complex samples, frame timing, channel/link metadata | P3 | authorized local measurement and validation | disabled unless necessary; bounded session retention | no backup by default; explicit participant-authorized encrypted research export | capture policy gate, bounded parser, per-class AEAD key, provenance, deletion | Signal Data Steward |
| DC-02 Derived observations | motion/respiration/attention features and quality | P3 | promoted observation capability | shorter of authorization or configured policy | encrypted, purpose-bound manifest only | minimization, quality/abstention, per-class key, lineage | Signal Data Steward |
| DC-03 Physiology estimates | pulse/respiration estimates and evidence | P3 | non-diagnostic approved feature | minimal window; no indefinite history by default | participant export only unless separately approved research grant | uncertainty, claim control, purpose gate, deletion | Clinical/Scientific Assurance Lead |
| DC-04 Latent-state estimates | arousal/valence/cognitive-load hypotheses | P3 | explicitly promoted non-diagnostic feature | transient projection by default | disabled by default | uncertainty, abstention, prohibited-use enforcement, visible disclosure | Product Safety Lead |
| DC-05 Personalization anchors | feedback, preference, reward, embeddings/profile vectors | P3 | participant-requested personalization | active authorization lifetime plus bounded grace | participant export; backup opt-in only | constrained adaptation, poisoning defenses, per-participant deletion | ML Assurance Lead |
| DC-06 Consent and policy | grants, versions, purpose, revocation, deletion state | P3 | authorization, proof, rights fulfilment | legal/quality minimum defined per market | participant-visible subset; protected backup permitted | append-only audit, encryption, correction workflow, fail closed | Privacy and Data Governance Lead |
| DC-07 Participant-visible history | sensing state and participant-facing activity | P3 | transparency and rights | market policy; minimized | participant export | authenticated access, provenance, separation from diagnostics | Privacy and Data Governance Lead |
| DC-08 Domain audit | policy decisions and privileged changes using IDs/hashes | P2; P3 if linkable | integrity, accountability, safety case | quality schedule | controlled compliance export | tamper evidence, payload exclusion, independent retention | Quality Lead |
| DC-09 Security events | authentication, tamper, exploit indicators | P2; P4 when exploit details/keys appear | detection and response | incident policy | security-only encrypted export | separate store, restricted role, redaction, integrity | Product Security Lead |
| DC-10 Diagnostics/support | health, versions, bounded metrics, redacted errors | P2 | local operation and authorized support | short rolling window | explicit operator-generated support bundle | denylist/canary redaction; never frames, embeddings, prompts, secrets, or labels | Support Engineering Lead |
| DC-11 Scientific provenance | artifact IDs, hashes, dataset/model versions, calibration and replay lineage | P2; P3 when participant-linkable | reproducibility and claims | quality schedule | signed evidence bundle | immutable IDs, source controls, de-identification | Scientific Assurance Lead |
| DC-12 Models/datasets | packages, evaluation corpora, manifests | P2; P3 if human-derived | approved inference/validation | version-support lifetime | signed controlled distribution | signatures, SBOM, lineage, poisoning review | ML Assurance Lead |
| DC-13 Device/fleet identity | device public identity, inventory, release reachability | P2 | support, updates, incident scope | service lifetime plus warranty record | administrative export | no behavioral telemetry, access audit, minimization | Fleet Operations Lead |
| DC-14 Cryptographic secrets | private keys, data keys, recovery tokens | P4 | encryption, identity, update verification | cryptoperiod only | private keys non-exportable where possible | managed generation, rotation, zeroization, no logs | Product Security Lead |
| DC-15 Configuration | non-secret settings, geometry, policy references | P1/P2 | operation | current plus rollback history | signed support-safe subset | schema validation, digest, secrets isolated | Runtime Owner |
| DC-16 Firmware/build/update artifacts | Nexmon patches, rvCSI build, binaries, SBOM, provenance | P1/P2 | reproducible operation/update | supported release lifetime | signed distribution | pinned source, review, signatures, rollback protection | Release Engineering Lead |

## Mandatory handling rules

1. Collection of DC-01 through DC-05 requires a current `SensingAuthorization` matching space, participants, purpose, policy version, and time. Unknown or unavailable policy means deny.
2. Identity inference, clinical diagnosis/treatment, employment or insurance scoring, deception detection, covert sensing, and unrelated secondary use are prohibited. Configuration, administrators, models, firmware, and sidecars cannot override this rule.
3. Data stores, keys, retention, access, export, deletion, and backup are partitioned by class. Destruction must propagate to projections, caches, vectors, exports under system control, and eligible backups.
4. Local IPC carries the minimum versioned contract. Sensitive payloads require peer authentication, authorization, bounded messages, replay resistance where commands mutate state, and transport protection appropriate to the host boundary.
5. Free-form logs must never contain raw frames, samples, embeddings, prompts, model inputs/outputs, participant labels, secrets, or complete configurations. Automated forbidden-field canaries verify redaction.
6. Support bundles are generated locally, previewed, minimized, encrypted to the intended recipient, time-limited, and explicitly authorized. They do not silently upload.
7. Every export contains class IDs, purpose, authorization reference, schema/artifact versions, time range, integrity manifest, recipient, expiry, and deletion instructions. Import is treated as untrusted input.
8. Development fixtures use synthetic or explicitly approved data. Production data is not copied to development, CI, analytics, or model training by default.

## Rights and lifecycle

Participant access, correction, export, revocation, and deletion commands are available without cloud or agent availability. Revocation stops new capture immediately. A deletion receipt records scope, stores visited, keys destroyed, exceptions, and completion without retaining deleted content. Legal/quality holds require documented authority, scope, expiry, and participant-facing handling where permitted; they cannot silently authorize new processing.

## Verification references

| Control | Required evidence |
|---|---|
| Capture gate | `VT-PRIV-001`: property and integration tests proving all missing/expired/revoked/wrong-purpose grants deny publication |
| Store separation | `VT-DATA-001`: architecture test and storage inventory map DC-01..DC-16 to independent policy/key/retention |
| Redaction | `VT-LOG-001`: canary suite fails on forbidden fields in diagnostics and support bundles |
| Deletion | `VT-DATA-002`: end-to-end deletion corpus covers journals, projections, caches, vectors, exports, and backup eligibility |
| Export | `VT-DATA-003`: tamper, recipient, expiry, authorization, and schema-validation tests |
| Keys | `VT-CRYPTO-001`: generation/rotation/zeroization and software-fallback assurance tests |

## Baseline acceptance

The approving roles accept the residual risk that RF-derived data may remain identifying despite pseudonymization and that operator-controlled copies cannot be technically erased after a valid export. Acceptance is limited to authorized, local-first, non-diagnostic operation with controls above; owner is the Privacy and Data Governance Lead, review due 2026-10-18. Any cloud transfer, identity feature, high-impact decision use, or expanded claim invalidates this acceptance and blocks release pending review.
