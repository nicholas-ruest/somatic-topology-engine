# Phase 13 personalization-memory evidence

Status: deterministic Rust reference adapter; RuVector/RVF integration not verified in this workspace.

## Capability status

The repository contains no pinned RuVector or RVF Cargo dependency or verified compatibility evidence. This phase therefore uses the `VectorMemory` port with a bounded deterministic Rust reference adapter. It does not claim RuVector/RVF performance, persistence semantics, security, or production readiness. A future adapter must pass the identical scoping, poisoning, deletion, rebuild, and benchmark contracts before selection.

## Privacy and evidence controls

- retrieval requires an exact participant scope and cannot search globally;
- anchors and feedback append with immutable assessment/observation provenance;
- corrections link to prior records and never rewrite them;
- evaluation and test partitions cannot update retrieval or adaptation state;
- adaptation candidates retain parent version and the exact qualified feedback set;
- “improved” remains false until prospective held-out evidence is promoted;
- participant deletion appends a non-payload tombstone, erases participant encryption-key material, removes vector payloads, and rebuilds the derived index from remaining authoritative journal records;
- post-deletion retrieval and journal replay tests verify erased payloads cannot reappear.

## User and operator surface

View returns only the authenticated participant’s anchors and provenance. Correct appends a linked correction. Delete is destructive, requires fresh exact authorization and explicit confirmation, and returns a cryptographic-erasure/rebuild receipt. Direct process invocation has no authenticated IPC identity and fails closed.

Run `scripts/validate-personalization.sh` for executable evidence. Human adaptation benefit, RuVector/RVF integration, Raspberry Pi/HIL/soak, and prospective improvement claims remain pending.

The executable suite passes eleven personalization tests, three governed CLI tests, and strict Clippy. The current-host reference scenario confirms participant scoping, encrypted append-only vector authority, key destruction, derived-index rebuild, and retained-user retrieval after erasure.
