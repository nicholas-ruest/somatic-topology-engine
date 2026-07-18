# Phase 09 validation evidence

Status: implemented and verified with deterministic synthetic evidence.

## Scope

The experiment-validation bounded context now supplies immutable study definitions, authority-gated human-study freeze, dataset lineage and leakage validation, reproducible run evidence, mandatory metric gates, append-only promotion decisions, atomic in-process persistence, and deidentified evidence export. The operator CLI exposes dataset validation, report export, promotion, and rejection only behind a fresh exact governance decision.

No human data was collected. No human study can be frozen without explicit ethics and consent authority. The executable example is a synthetic negative control and deliberately preserves a rejected result and rejected capability decision.

## Verification

Run:

```bash
scripts/validate-study.sh
```

The script runs all experiment-validation targets, the governed CLI contract tests, warning-free Clippy checks, and the deterministic negative-control report twice. Byte comparison must succeed. Generated local evidence is written beneath `target/validation/`, which is build output and is not committed.

Verified controls include:

- immutable identifiers reject conflicting replacement while exact retries are idempotent;
- negative study results and rejection decisions remain present in exports;
- exports omit participant, session, room, and collection-day grouping keys;
- participant/session/room/day partition leakage is rejected;
- mandatory gates reject missing, non-finite, underpowered, or threshold-failing evidence;
- promotion and rejection are blocked before dispatch when authorization is absent;
- every governed CLI operation requests a fresh authorization;
- direct process invocation exits fail-closed because it has no authenticated IPC identity;
- report bytes and SHA-256 digest are stable for identical evidence.

## Release interpretation

This phase establishes validation infrastructure; it does not claim empirical human validity. Capabilities remain unpromoted unless an authorized service verifies a frozen preregistration, locked non-leaking dataset, immutable passing result, and exact evidence digest. Negative and null outcomes are first-class release evidence and must not be removed from the history.
