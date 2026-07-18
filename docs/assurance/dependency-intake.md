# Dependency Intake and Commercial License Policy

**Record ID:** STE-QMS-SC-001
**Owner:** Supply-Chain & Licensing Lead
**Technical reviewer:** Runtime Engineering Lead
**Security reviewer:** Security & Privacy Lead

This policy covers Rust crates, TypeScript/npm packages, Git dependencies, build tools, firmware/patches, native libraries, models, datasets, fixtures, container images, and generated code. No material dependency enters a release merely because a package manager resolves it.

## Required intake record

Record component name and ecosystem, exact version/commit and source, cryptographic digest, owner, purpose, alternatives, enabled features, transitive graph, maintenance health, ARM/support targets, runtime privileges/data access, known vulnerabilities, license and notices, export/patent concerns, update cadence, reproducibility/provenance, test evidence, and removal/replaceability plan. Firmware, models, and datasets additionally require origin, transformation lineage, redistribution rights, and compatibility scope.

## License policy

- Pre-approved subject to automated and human verification: Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, CC0-1.0 for code/metadata where applicable, and Zlib.
- Review required: MPL-2.0, LGPL family, EPL family, CDDL, OpenSSL/SSLeay variants, BSL, source-available, non-commercial, research-only, custom, missing, ambiguous, patent-encumbered, or dataset/model/firmware-specific terms.
- Forbidden in a commercial release without a separately approved distribution architecture and qualified-counsel opinion: AGPL family, SSPL, Commons Clause, licenses prohibiting commercial use or redistribution, licenses incompatible with intended linking/distribution, and components with no demonstrable grant of rights.

The root product license does not relicense third-party work. Required copyright, attribution, source-offer, modification notice, or relinking obligations must be packaged and tested. The canonical machine policy is `deny.toml`; any discrepancy is resolved to the stricter rule until reviewed. A license exception identifies exact component/version, distribution mode, obligations, counsel opinion, owner, expiry, and removal trigger.

## Technical rules

Use registry versions or immutable Git commits; floating branches, tags without verified digest, `latest`, unpinned install scripts, and runtime downloads are forbidden. Lockfiles are release inputs. Minimize default features, reject duplicate/abandoned critical libraries where practical, isolate patched Wi-Fi firmware and optional TypeScript sidecars, and verify source/archive checksums. Production builds do not fetch executable code or models dynamically.

## Gate and re-review

Automated checks must produce advisory, license/source, SBOM, provenance, and secret-scan artifacts. A new or changed component fails intake on unresolved provenance, forbidden/unknown license, critical reachable vulnerability without accepted risk, unbounded parser/native attack surface, unsupported target, or missing owner/removal plan. Re-review occurs on version, features, license, source, maintainer/control, CVE, target, data access, or distribution change.

## Ruvnet components

rvCSI/Nexmon, RuView, RuVector/RVF, ruv-FANN, and Ruflo-derived product dependencies are evaluated individually. Development orchestration files and agent tooling are not product dependencies or release artifacts. Preference for the Ruvnet stack does not waive compatibility, quality, licensing, security, or replaceability gates.

## References

- [ADR-023](../adr/ADR-023-govern-dependencies-licenses-and-software-supply-chain.md)
- [Vulnerability disclosure](vulnerability-disclosure.md)
- [Quality records](quality-records.md)
