# ADR-024: Maintain a Living Threat Model and Trust Boundaries

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: security, threat-model, trust, abuse

## Context

STE processes sensitive ambient signals, patched Wi-Fi firmware, physical inputs, removable storage, model artifacts, local APIs, and optional sidecars. Privacy harm can result from legitimate features as well as conventional attacks.

## Decision

Maintain a versioned threat model covering assets, actors, physical access, radio attackers, malicious frames, supply chain, local users, TypeScript sidecars, exported bundles, model poisoning, inference abuse, denial of service, rollback, and covert sensing. Map trust boundaries and mitigations to tests and owners.

Use defense in depth: least privilege, process/device isolation, authenticated contracts, bounded parsing, fail-closed policy, signed artifacts, secure defaults, redaction, and visible sensing state. Revisit the model for every new interface, data class, capability, hardware revision, and deployment scenario.

## Consequences

### Positive
- Security work is tied to actual architecture and abuse cases.
- Privacy and safety threats receive equal treatment with confidentiality/integrity threats.

### Negative
- Threat-model maintenance and penetration testing require continuing investment.
- Some integrations may be rejected on isolation grounds.

### Neutral
- The model documents residual risk; it cannot prove absence of vulnerabilities.

## Links

**Depends on**: [ADR-009](ADR-009-enforce-consent-privacy-and-prohibited-use-in-the-domain.md), [ADR-012](ADR-012-use-versioned-contracts-and-an-anti-corruption-layer.md), [ADR-023](ADR-023-govern-dependencies-licenses-and-software-supply-chain.md)
