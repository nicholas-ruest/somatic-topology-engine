# STE Threat, Privacy, and Abuse Model

**Record:** ASSURANCE-THREAT-001
**Version:** 1.0.0
**Status:** Accepted initial baseline; implementation evidence pending
**Method:** STRIDE plus LINDDUN-style privacy and product-abuse analysis
**System owner:** Product Owner
**Risk owner:** Product Security Lead
**Privacy owner:** Privacy and Data Governance Lead
**Safety owner:** Product Safety Lead
**Review:** before each release and after any new interface, data class, claim, model, firmware, hardware, deployment, incident, or every 90 days

## Scope and objectives

STE is a local-first ambient RF sensing system on physically accessible Raspberry Pi/CrowPi-class hardware. The baseline covers radio capture and patched Wi-Fi firmware, imports, Rust services, local IPC, optional TypeScript sidecars, storage, models, UI/peripherals, administration, exports, support, and release/update inputs. Cloud services are outside the approved baseline.

Security objectives are: capture only with active consent and purpose; prevent covert or high-impact use; preserve sensing-data confidentiality and provenance; make uncertainty and failures visible; constrain hostile parsing and resource use; protect device/release identity; and recover to capture-disabled safe state.

## Assets and adversaries

Protected assets include participant autonomy and safety; raw/derived sensing data; authorization and deletion state; model/profile integrity; cryptographic keys; device/release identity; audit/provenance; availability; claim evidence; and the visible truthfulness of sensing/uncertainty state.

Threat actors include a radio-range attacker; malicious or curious operator/administrator; person with physical access; compromised AP, firmware, dependency, release account, sidecar, model, or dataset; hostile imported file; unauthorized co-occupant; support insider; and a legitimate product team attempting an unsupported claim or secondary use.

## Risk scale

Likelihood and impact are `1` (low) through `5` (very high). Inherent score is their product. Residual target is recorded after mandatory controls: `Low` (1–4), `Moderate` (5–9), `High` (10–16), or `Critical` (17–25). Any residual High/Critical risk blocks production unless the Product Security, Privacy, Safety, and Product owners jointly approve a time-bounded exception; prohibited uses cannot receive an exception.

## Threat register

| ID | Category | Scenario and affected assets | I | L | Required preventive/detective controls | Verification reference | Residual target | Owner / acceptance |
|---|---|---|---:|---:|---|---|---|---|
| T-001 | S/T/E | Crafted or malformed CSI exploits firmware/driver/parser or forges provenance | 5 | 4 | pinned patched firmware; isolation/least privilege; bounded zero-copy parser; finite/range/length validation; provenance; fuzz corpus; reviewed unsafe | VT-RADIO-001, VT-UNSAFE-001, firmware SBOM | Moderate | Capture Platform Lead; accepted for pilot after gates |
| T-002 | D | RF or replay flood exhausts CPU, memory, disk, queues and hides critical audit | 4 | 4 | bounded channels/storage; quotas; overflow policy; rate limiting; admission control; critical-event reserve; supervisor/safe degradation | VT-RADIO-002, VT-RUNTIME-001, VT-STORE-001 | Moderate | Runtime Lead; accepted after overload benchmark |
| T-003 | S/T | Compromised AP/device substitution injects plausible frames or changes channel/geometry | 4 | 3 | device/link identity; signed compatibility profile; sequence/gap capture; calibration/version binding; plausibility and OOD checks | VT-RADIO-003, VT-CAL-001 | Moderate | Capture Platform Lead; accepted after HIL |
| T-004 | E/T | Physical attacker extracts keys/data, installs media, rolls back software, or enables capture | 5 | 4 | per-device identity; managed keys; encrypted class stores; secure/verified boot where supported; signed anti-rollback updates; fail-closed boot; tamper evidence; physical off | VT-CRYPTO-001, VT-UPDATE-001, VT-PHYS-001 | High on dev hardware; Moderate target | Product Security Lead; pilot-only acceptance expires 2026-10-18 |
| T-005 | S/R/E | Local IPC peer impersonates admin/service, replays commands, or acts as confused deputy | 5 | 3 | peer auth; narrow roles; nonce/idempotency; schema/version/size limits; Rust policy recheck; complete privileged audit | VT-IPC-001, VT-ADMIN-001, VT-AUDIT-001 | Low | Runtime Lead; accepted after test evidence |
| T-006 | I/T | Local storage theft, tampering, torn writes, rollback, swap or diagnostic leakage | 5 | 3 | per-class AEAD; checksums/append-only journal; rollback detection; atomic checkpoints; secret isolation; tested redaction; deletion/zeroization | VT-STORE-001, VT-LOG-001, VT-DATA-002 | Moderate | Storage Security Lead; accepted after recovery/deletion evidence |
| T-007 | T/E/I | Optional TypeScript sidecar exploits dependency surface, widens purpose, reads raw data, or exfiltrates | 5 | 3 | opt-in/disabled; unprivileged isolated process; allowlisted DTOs only; no device/store/key access; authenticated bounded IPC; Rust authorization; independent kill | VT-IPC-002, VT-SBOM-001, penetration test | Moderate | Product Security Lead; acceptance only per enabled release |
| T-008 | T/S | Poisoned model, dataset, anchor, or personalization reward causes unsafe/biased inference | 5 | 4 | signed content-addressed artifacts; lineage; partition controls; robust evaluation; constrained rewards; anomaly/OOD/uncertainty; promotion and rollback | VT-MODEL-001, VT-DATASET-001, VT-PERS-001 | Moderate | ML Assurance Lead; accepted per model package |
| T-009 | T/E | Compromised dependency, build, signing account, Nexmon patch, firmware, or update | 5 | 3 | pinned inputs; reviews; SBOM/licenses; vuln/secret/malware scans; reproducible builds; isolated signing; provenance; atomic rollback | VT-SUPPLY-001, VT-REPRO-001, VT-UPDATE-001 | Moderate | Release Engineering Lead; accepted per release evidence |
| T-010 | L/I/abuse | Covert sensing or collection without all participants' specific active authorization | 5 | 4 | fail-closed domain gate before publication; visible indicator; physical off; time/version/purpose/space checks; revocation immediate; audit | VT-PRIV-001, VT-UI-001 | Low target | Privacy Lead; no acceptance if gate bypass exists |
| T-011 | abuse/non-compliance | Identity inference, diagnosis, employment/insurance scoring, deception detection, or unrelated secondary use | 5 | 4 | hard-coded prohibited-purpose type; signed feature policy cannot enable; no generic raw/model output API; controlled claims; sales/docs review | VT-ABUSE-001, VT-CLAIM-001 | Low target | Product Owner; prohibited—never risk-accepted |
| T-012 | L/I | Linkage of pseudonymous histories, embeddings, timing, fleet records, exports, or support data identifies participants | 5 | 3 | class separation; rotating pseudonyms where valid; minimization; no behavioral fleet telemetry; export preview; access/retention limits | VT-PRIV-002, VT-EXPORT-001 | Moderate | Privacy Lead; accepted with 90-day review |
| T-013 | I/non-repudiation | Diagnostics/support bundle leaks sensing payload, labels, config secrets, or model I/O | 5 | 3 | separate stores; structured allowlist/redaction; forbidden-field canaries; local preview; recipient encryption; explicit generation | VT-LOG-001, VT-SUPPORT-001 | Low | Support Engineering Lead; accepted after canary suite |
| T-014 | R/T | Administrator or service suppresses/forges audit, changes policy, or silently drops critical records | 4 | 3 | append-only tamper evidence; unique actor/device identity; audit-before-command success; critical write fail-closed; independent access/retention | VT-AUDIT-001, VT-AUDIT-002 | Moderate | Quality Lead; accepted after restore/tamper tests |
| T-015 | T/I | Malicious PCAP/rvcsi/export/model/config archive triggers path traversal, bomb, unsafe deserialization, or trust inheritance | 5 | 4 | hostile-input streaming parser; byte/depth/time caps; path normalization; quarantine; signatures/schema; authorization never imported | VT-IMPORT-001, fuzz-importers | Low | Product Security Lead; accepted after fuzz gate |
| T-016 | D/T/safety | Stale, low-quality, OOD, replayed, or overconfident estimate is displayed as current truth | 5 | 4 | event-time/freshness; quality gates; calibrated uncertainty; OOD; abstention; deterministic policy-approved projections; visible degraded state | VT-INFER-001, VT-UI-002 | Moderate | Product Safety Lead; accepted per capability evidence |
| T-017 | I/R | Export goes to wrong recipient, is modified, outlives purpose, or cannot be deleted downstream | 5 | 3 | explicit participant auth; local preview; recipient-bound encryption; signed manifest/expiry; data-class/purpose labels; warning and receipt | VT-EXPORT-001, VT-DATA-003 | Moderate | Privacy Lead; accepted for user-directed exports only |
| T-018 | S/E | Shared/default credentials or device identity cloning permits fleet-wide compromise | 5 | 3 | unique asymmetric identity; no defaults/shared fleet secrets; provisioning attestation; rotation/revocation; rate-limited recovery | VT-IDENT-001, VT-CRYPTO-001 | Low | Product Security Lead; accepted after provisioning audit |
| T-019 | D/safety | Thermal, power, disk, watchdog, peripheral, or update failure leaves sensing active or UI misleading | 5 | 3 | supervised health; independent failure domains; resource thresholds; fail-safe capture disable; atomic update rollback; indicator self-test | VT-HIL-001, VT-UPDATE-001, VT-UI-001 | Moderate | Device Platform Lead; accepted after soak/HIL |
| T-020 | privacy/abuse | Multi-person room includes non-consenting entrant or consent ambiguity | 5 | 4 | explicit space policy; participant procedure; visible continuous indicator; easy off/revocation; cease on uncertainty; deployment exclusion where consent impractical | VT-CONSENT-002, site acceptance | Moderate | Privacy Lead; accepted only for controlled deployments |
| T-021 | I/abuse | Scientifically unsupported capability or marketing language induces reliance/high-impact action | 5 | 3 | claim-evidence matrix; promoted capability policy; operating-envelope enforcement; non-diagnostic warnings; qualified review; complaint monitoring | VT-CLAIM-001, VT-VALID-001 | Moderate | Product Owner and Scientific Lead; accepted per market release |
| T-022 | D/I | Offline device misses revocation/update or incident remediation | 4 | 3 | local revocation; signed offline update; supported-version/expiry policy; device status visible; capability disables after mandatory deadline where safe | VT-OFFLINE-001, incident exercise | Moderate | Fleet Operations Lead; accepted for offline design |

## Abuse-case invariants

The following must remain impossible even for an authenticated administrator: enable sensing without participant/space/purpose authorization; widen a granted purpose; activate a prohibited use; render raw model output as approved UI; silently export; suppress required audit; or let sidecars/firmware authorize themselves. A test must exercise each attempted route through configuration, feature policy, API, CLI, imported state, model metadata, and IPC.

## Privacy analysis

- **Linkability/identifiability:** RF patterns, stable pseudonyms, embeddings, timing, geometry, and exports may identify individuals. Minimize retention and cross-class joins; treat all sensing data as sensitive even when pseudonymous.
- **Detectability/unawareness:** ambient sensing may be invisible. A hardware-backed visible indicator and physical off control accompany participant-facing history and plain-language purpose/retention display.
- **Non-compliance:** authorization, deletion, and claim rules are executable domain invariants. Administrative convenience or a degraded dependency never yields permissive behavior.
- **Data subject rights:** rights workflows operate locally, provide receipts, and enumerate stores; unavailable cloud/agent services cannot delay revocation.

## Residual-risk acceptance

Baseline residual risks are accepted only for development and controlled pilot operation by the role owners shown above, subject to every referenced gate reaching pass. The Product Security Lead owns the consolidated acceptance; the Privacy Lead accepts T-010/T-012/T-017/T-020; the Safety Lead accepts T-016/T-019/T-021; the ML Assurance Lead accepts T-008. Acceptance expires 2026-10-18 and does not authorize production release. Production requires named human approvers, evidence-linked scores, penetration and privacy review, hazard closure, and no High/Critical residual risk. Prohibited uses T-010 bypass and T-011 are never acceptable.

## Maintenance and incident feedback

Every discovered vulnerability, privacy complaint, safety event, anomalous model output, dependency advisory, and field failure is mapped to threat IDs. Material changes update this versioned document, hazard links, tests, owners, and release evidence. Incident containment preserves minimized evidence, supports device/model/key revocation and offline remediation, and produces a blameless corrective-action record.
