# Optional TypeScript sidecar decision

Decision: **no measured capability gap; do not ship a sidecar**.

The Rust modular monolith currently provides capture, DSP, domain policy, inference ports, validation, personalization, persistence, device interaction, CLI, commissioning, metrics, and deterministic UI templates. No production requirement has been demonstrated that maintained TypeScript software uniquely satisfies. Adding Node/npm, AgentDB, DSPy.ts, or Ruflo to the device would add supply-chain, memory, CPU, sandbox, IPC, privacy, and support cost without closing an evidenced gap.

Ruflo remains development orchestration only. DSPy.ts may be reconsidered for offline copy experiments whose reviewed result is compiled into Rust. AgentDB may be reconsidered only after a benchmarked feature gap in the authoritative Rust vector/journal path. Neither may read hardware, raw CSI, keys, authoritative stores, consent state, or model authority.

## Dormant contract mechanism

Rust defines `SidecarRequest`/`SidecarResponse` with a pinned generated-contract digest, bounded payloads, correlated identities, authentication digest, deadlines, rate limits, and exactly two advisory operations: deidentified summarization and offline copy suggestions. This is a verification mechanism, not an enabled IPC endpoint or generated TypeScript package. If a future ADR approves a sidecar, bindings must be generated from the reviewed schema in reproducible CI and checked for semantic parity; handwritten widening is prohibited.

## SBOM and license status

No Node runtime, npm lockfile, TypeScript compiler, or sidecar package is part of the production dependency graph, SBOM, license inventory, image, or update bundle. Consequently there are no new Phase 16 third-party licenses. A future sidecar requires pinned Node/npm versions, immutable lockfile, CycloneDX/SPDX entries, license and vulnerability review, archive digests, maintainer/removal plan, and offline reproducible installation.

## Removal and fallback

The current removal plan is trivial: nothing is installed or started. Rust is the complete fallback and normal path. A future optional feature must degrade to “unavailable” when absent, hung, corrupt, rate-limited, or killed; it cannot fabricate output. Removing its process, account, sandbox, socket, and packages must not migrate domain data or change capture, authorization, inference, journal, or safe UI behavior.
