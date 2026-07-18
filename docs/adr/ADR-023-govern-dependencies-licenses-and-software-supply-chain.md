# ADR-023: Govern Dependencies, Licenses, and the Software Supply Chain

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: supply-chain, sbom, licenses, dependencies

## Context

STE depends on Rust crates, npm packages, patched firmware, model artifacts, and external repositories. Commercial distribution requires provenance, license compatibility, vulnerability response, and reproducible inputs.

## Decision

Pin dependencies and Git sources to reviewed versions/commits; forbid floating `latest` in builds and runtime. Generate CycloneDX/SPDX SBOMs for software, firmware, models, and datasets. Enforce license allow/deny policy, source/archive checksums, vulnerability and malware scans, provenance attestations, and signed release manifests.

Minimize dependencies and features. Require owner, purpose, update policy, maintenance health, ARM support, license, and removal plan for material dependencies. Vendoring is allowed when required for reproducibility and license terms are preserved.

## Consequences

### Positive
- Commercial licensing and incident scope are knowable.
- Builds can be reconstructed from verified inputs.

### Negative
- Updates require review and compatibility testing.
- Some attractive dependencies may be rejected or replaced.

### Neutral
- Vulnerability findings are risk-assessed; severity alone does not automatically determine exploitability.

## Links

**Depends on**: [ADR-013](ADR-013-define-the-cargo-workspace-and-crate-dependency-policy.md), [ADR-019](ADR-019-evolve-contracts-with-explicit-compatibility-rules.md)
