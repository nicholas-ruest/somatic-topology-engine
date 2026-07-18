# Phase 16 optional-sidecar evidence

Status: no sidecar justified or shipped.

The measured-gap review found no capability requiring TypeScript in the production device. The accepted outcome is therefore a smaller attack and support surface. The Rust boundary is disabled by default and accepts no connection; its dormant versioned DTO permits no hardware, raw data, keys, store, authorization, inference, or capability-policy operation.

Executable tests inject absent, hung, corrupt, and killed states and prove core readiness remains unchanged. Contract tests reject unknown fields, unsupported schemas, oversized/unallowlisted operations, and attempts to request capture authorization. Existing governance tests separately prove a sidecar request origin cannot widen purpose.

No sidecar throughput benchmark is reported because no sidecar exists. The validation script records only Rust isolation-test time and dependency absence. A future sidecar requires a new sequential ADR, measured gap, threat-model review, generated bindings, resource/timeout/rate/sandbox tests, SBOM/license evidence, removal drill, and reference-Pi benchmark.

The focused suite passes five sidecar containment tests, the prohibited-purpose widening test, and strict Clippy. It verifies disabled/absent, hung, corrupt, and killed behavior; offline unprivileged sandbox requirements; contract digest, payload, deadline, response, and rate controls; and absence of authoritative/hardware operations. One validation invocation completed the containment-test command in 116,710,532 ns; this is harness timing only.
