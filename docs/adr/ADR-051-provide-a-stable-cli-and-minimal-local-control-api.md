# ADR-051: Provide a Stable CLI and Minimal Local Control API

**Status**: Proposed
**Date**: 2026-07-17
**Deciders**:
**Tags**: cli, api, ipc, automation

## Context

Provisioning, capture diagnostics, replay, calibration, export, update, and support need automation. A broad web API would increase attack surface and encourage bypassing domain workflows.

## Decision

Make `ste-cli` the supported operator interface, invoking authenticated Rust application commands over Unix-domain IPC or in-process for recovery mode. Expose narrowly scoped versioned commands for status, doctor, capture test, calibration, replay, validation export, consent, data lifecycle, models, update, support bundle, and reset.

Return stable machine-readable output plus human views, typed exit codes, idempotency keys for mutations, dry-run/confirmation for destructive actions, and no secrets in arguments or process listings. Do not expose raw aggregate mutation or model score injection.

## Consequences

### Positive
- Scriptable operations without a large network service.
- Domain authorization and audit remain centralized.

### Negative
- CLI/API compatibility becomes a maintained product surface.
- Remote fleet management needs a separately secured bridge.

### Neutral
- A future UI invokes the same application contracts rather than duplicating logic.

## Links

**Depends on**: [ADR-019](ADR-019-evolve-contracts-with-explicit-compatibility-rules.md), [ADR-027](ADR-027-authenticate-and-authorize-local-administration.md), [ADR-043](ADR-043-use-guided-installation-site-qualification-and-acceptance.md)
