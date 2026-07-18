# Phase 07 deterministic observation replay report

**Scope:** signal observation only. This report makes no physiology, emotion,
stress, valence, workload, decision, identity, or medical claim.

The replay path accepts bounded validated radio records, maps them through a
primitive CSI anti-corruption type, executes a versioned DSP graph, closes a
window with complete source references, units, calibration/DSP/algorithm
versions, bounds, quality and partition role, and stores the immutable artifact
by its SHA-256 digest. Offline CLI selection is limited to development,
validation, or held-out test partitions; it cannot emit production artifacts.

Evidence executed on 2026-07-18:

- identical radio input produced an exactly identical artifact and digest;
- repeated storage returned `AlreadyPresent` and retained one artifact;
- a deliberately occupied digest with different content returned
  `DigestCollision` without overwrite;
- malformed digests were rejected before insertion;
- governance denial occurred before governed capture bytes were read;
- all ordered radio source references were retained;
- numerical/performance benchmark passed with the values recorded in
  `docs/benchmarks/phase-07-observation.md`.

The repository stores immutable clones and verifies both digest occupancy and
content equality for idempotency. Repository synchronization and collisions fail
closed as payload-free errors. Scientific golden-corpus and cross-architecture
evidence are maintained separately from this engineering replay result.
