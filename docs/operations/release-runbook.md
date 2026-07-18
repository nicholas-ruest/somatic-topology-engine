# Evidence-bearing release runbook

1. Freeze the exact source revision, Rust toolchain, target image, hardware/firmware matrix, configuration schemas, migrations, models, datasets, feature and capability policies, claims, markets, and support organization.
2. Build hermetically for ARM and host verification; produce signed manifests, software/firmware/model/data SBOMs, provenance, checksums, licenses, model/dataset cards, compatibility and rollback metadata. Do not rebuild between candidate, pilot, and production.
3. Run full tests, architecture, Clippy, formatting, dependency/security/secret/unsafe/fuzz/penetration gates, numerical replay, scientific validation, benchmark regression, physical HIL/thermal/power/soak, update/rollback/recovery, installation, and erasure evidence.
4. Verify every enabled claim against promotion, signed policy, operating envelope, site acceptance, jurisdiction approval, intended use, hazard control, residual risk, and support readiness.
5. Confirm pilot, manufacturing/supplier, warranty/RMA, update window, incident/recall, complaint/privacy, EOL, telemetry, CAPA, training, contacts, and funding/capacity evidence.
6. Convene the named production-readiness board. Any missing or failed mandatory evidence blocks approval. Record scoped, time-bound decision and signatures; no blanket exception may accept an unacceptable hazard or unsupported claim.
7. Promote the identical digest through approved channels, verify installation/rollback, monitor predetermined signals, and retain immutable distribution evidence. Suspend affected capabilities immediately when trigger criteria fire.

Current outcome: blocked. This runbook has not been completed for a physical production candidate.
