# Preliminary Hazard Analysis and Control Register

**Record:** ASSURANCE-HAZARD-001
**Version:** 1.0.0
**Status:** Accepted preliminary baseline; implementation verification pending
**Safety owner:** Product Safety Lead
**Approvers:** Product Owner; Product Security Lead; Privacy and Data Governance Lead; Scientific Assurance Lead
**Review:** before capability promotion/release and after any incident, complaint, field anomaly, claim, operating-envelope, or control change

STE is not approved for medical diagnosis/treatment, emergency detection, employment/insurance decisions, deception detection, identity inference, or safety-critical control. Calling a feature “wellness” does not reduce actual risk or regulatory obligations.

## Method

Severity: `S1` negligible, `S2` minor/reversible, `S3` serious, `S4` severe or broad privacy harm, `S5` catastrophic/irreversible. Probability: `P1` remote through `P5` frequent. Detectability: `D1` almost certain detection through `D5` unlikely detection before harm. Critical hazards are inherent severity S5 or risk priority `S × P × D >= 40`. Unverified controls receive no residual-risk credit.

## Hazard register

| ID | Hazardous situation / foreseeable harm | Initial S/P/D | Critical | Controls and safe state | Verification reference | Residual target | Owner / acceptance |
|---|---|---|---|---|---|---|---|
| H-001 | Capture occurs without valid consent or after revocation, causing covert surveillance and loss of autonomy | 5/4/4 | Yes | fail-closed `SensingAuthorization` before publication; continuous indicator; physical off; immediate local revocation; capture-disabled safe state | VT-PRIV-001, VT-UI-001, VT-CONSENT-002 | 5/1/2 | Privacy Lead; no release acceptance until tests pass |
| H-002 | Identity, diagnosis, employment scoring, deception, or unrelated secondary use causes discrimination or clinical/privacy harm | 5/4/4 | Yes | prohibited-purpose domain type; no override via config/admin/feature/model/sidecar; claim and API review; no generic inference endpoint | VT-ABUSE-001, VT-CLAIM-001 | 5/1/2 | Product Owner; prohibited, never exception-accepted |
| H-003 | False, stale, OOD, or overconfident physiology/state estimate induces harmful reliance | 5/4/4 | Yes | quality/operating-envelope gate; event-time freshness; calibrated uncertainty; abstain; approved deterministic projection; non-diagnostic warning | VT-INFER-001, VT-UI-002, VT-VALID-001 | 5/2/2 | Safety Lead; capability-specific acceptance required |
| H-004 | Wrong participant/space association attributes sensitive state to another person | 5/3/4 | Yes | site qualification; explicit space/participant binding; ambiguity detection; stop/abstain on multiple or unknown occupants; no identity inference | VT-CONSENT-002, VT-CAL-001, HIL multi-person protocol | 5/1/3 | Privacy and Scientific Leads; controlled sites only |
| H-005 | Malformed CSI or patched firmware compromise corrupts evidence, exposes data, or disables controls | 5/4/4 | Yes | pinned/reviewed firmware; isolation; bounded validated parser; finite/range checks; fuzzing; provenance; supervisor disables capture on adapter failure | VT-RADIO-001, VT-UNSAFE-001, VT-HIL-001 | 5/2/2 | Capture Platform and Security Leads; pilot-only pending evidence |
| H-006 | Model/dataset/anchor poisoning systematically produces unsafe or biased outputs | 5/4/4 | Yes | signed immutable artifacts; dataset lineage/splits; reward constraints; anomaly/OOD checks; preregistered promotion; instant rollback/revocation | VT-MODEL-001, VT-DATASET-001, VT-PERS-001 | 5/2/2 | ML Assurance Lead; per-model acceptance |
| H-007 | Storage corruption, disk full, rollback, or lost key produces misleading state, privacy leak, or irrecoverable consent/deletion state | 5/3/4 | Yes | checksummed append journal; atomic checkpoint; quotas/reserve; corruption detection; encrypted classes; tested restore; capture-disabled safe state | VT-STORE-001, VT-CRYPTO-001, VT-DATA-002 | 5/1/2 | Storage Owner; acceptance after recovery corpus |
| H-008 | Update/config/model activation fails or rolls back to vulnerable/unsafe behavior while appearing healthy | 5/3/4 | Yes | signed compatibility policy; schema validation; staged atomic update; anti-rollback; known-good recovery; activation health check; visible degraded state | VT-UPDATE-001, VT-CONFIG-001 | 5/1/2 | Release Engineering Lead; release evidence required |
| H-009 | Thermal/power/watchdog fault damages hardware or leaves sensing/UI in inconsistent state | 5/3/3 | Yes | thermal/power thresholds; watchdog; bounded workload; coordinated shutdown; capture disabled on uncertainty; indicator self-test; CrowPi electrical envelope | VT-HIL-001, soak/thermal report | 5/1/2 | Device Platform Lead; HIL evidence required |
| H-010 | Local admin, IPC sidecar, or support workflow bypasses policy or leaks sensitive content | 5/3/4 | Yes | authenticated least-privilege IPC; Rust re-authorization; no sidecar raw/key/store access; bounded contracts; redaction canaries; audited commands | VT-IPC-002, VT-ADMIN-001, VT-LOG-001 | 5/1/2 | Product Security Lead; per-release acceptance |
| H-011 | UI indicator/display is stale, inaccessible, dark, frozen, or misleading, hiding active capture or uncertainty | 4/3/4 | Yes | hardware-visible sensing state; heartbeat/freshness; display-independent physical off; accessibility review; peripheral failure degrades capture safely | VT-UI-001, VT-UI-002, peripheral fault injection | 4/1/2 | Device Interaction Lead; acceptance after HIL |
| H-012 | Revocation/deletion fails across journals, projections, cache, vector memory, backup, or export | 5/3/4 | Yes | synchronous publication stop; enumerated store registry; per-class keys; idempotent deletion; signed receipt; export warning/manifest; retry with visible incomplete status | VT-DATA-002, VT-EXPORT-001 | 5/1/3 | Privacy Lead; no acceptance for silent partial completion |
| H-013 | RF/resource denial of service suppresses critical audit, off command, or safe-state transition | 4/4/3 | Yes | bounded queues; priority/reserved capacity; shedding hierarchy; local physical off; watchdog; critical audit failure aborts command | VT-RUNTIME-001, VT-RADIO-002, overload benchmark | 4/1/2 | Runtime Lead; benchmark acceptance required |
| H-014 | Incorrect geometry/calibration/clock synchronization produces plausible but invalid scientific evidence | 4/4/4 | Yes | signed versioned calibration; monotonic/event time; clock uncertainty; supported envelope; expiry/drift checks; abstain outside calibration | VT-CAL-001, VT-TIME-001, replay benchmark | 4/1/2 | Scientific Assurance Lead; per-site acceptance |
| H-015 | Participant relies on product in emergency, medical, or safety-critical situation despite excluded use | 5/3/4 | Yes | product/API/UI/packaging claim controls; onboarding acknowledgment; no emergency alerts; uncertainty and non-medical warnings; complaint surveillance | VT-CLAIM-001, human-factors validation | 5/1/3 | Product Owner; market-specific qualified review |
| H-016 | Export/support data reaches wrong recipient or persists beyond purpose | 5/3/4 | Yes | recipient-bound encryption; preview; explicit authorization; signed expiry/purpose manifest; minimization; no automatic upload | VT-EXPORT-001, VT-SUPPORT-001 | 5/1/3 | Privacy Lead; user-directed export residual accepted |
| H-017 | Physical controls or peripherals cause shock, heat, accessibility, entrapment, or destructive reset harm | 4/2/3 | No | supported CrowPi wiring; current/thermal limits; debounced/stuck inputs; accessible interaction; reset requires authenticated confirmation and returns capture-disabled | VT-HIL-001, electrical/accessibility review | 4/1/2 | Device Platform Lead; hardware-revision acceptance |
| H-018 | Loss of network prevents consent, revocation, safety, update, or recovery functions | 4/3/3 | No | all capture policy/revocation/off/safe-state local; offline signed update path; supported-version expiry visible; no cloud dependency | VT-OFFLINE-001, incident exercise | 4/1/2 | Fleet Operations Lead; accepted local-first residual |

## Critical-hazard traceability rule

Every row marked Critical has at least one prevention/control, an explicit safe state, a verification ID, and an accountable role. Before a release, each verification ID must resolve to immutable test/report evidence containing version, environment, result, reviewer, date, and artifact hash. “Planned” or “manual inspection” without a signed result does not close a critical hazard. Failed or missing evidence leaves the residual score equal to the initial score and blocks release.

## Universal safe states

- **Authorization unknown/revoked:** stop publication immediately; display capture disabled; retain only minimal policy audit.
- **Inference uncertain/stale/outside envelope:** publish an explicit unavailable/uncertain projection, never the last estimate as current.
- **Adapter, peripheral, thermal, storage, update, model, or integrity failure:** isolate the failed capability; disable capture if authorization, audit, or safe indication cannot be guaranteed.
- **Unrecoverable corruption or factory reset:** preserve permitted forensic integrity metadata, erase eligible keys/data, boot into unprovisioned capture-disabled state.

## Warning baseline

Participant-facing material states that STE uses ambient RF; may detect people beyond the intended participant; is not medical or emergency equipment; can be wrong or unavailable; must not support identity, deception, employment, insurance, or safety decisions; and stops/erases data subject to explicit lifecycle limits. Active sensing, degraded state, and uncertainty must be perceivable without opening a support tool.

## Residual-risk acceptance

The approving roles accept the preliminary residual targets solely as design objectives for development and controlled pilots, expiring 2026-10-18. They do not accept unverified controls or authorize production. H-001 through H-016 remain release-blocking until their evidence resolves and named humans record date, rationale, scope, expiry, and corrective actions. H-002 prohibited uses and any covert-capture path cannot be accepted. Field complaints and incidents reopen the affected hazard immediately.
