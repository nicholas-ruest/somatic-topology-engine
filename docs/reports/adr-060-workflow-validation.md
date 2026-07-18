# ADR-060 workflow validation

The `ste-workflows` crate implements a closed Rust workflow catalog, checksummed append-only events, deterministic projections, semantic progress, authoritative confirmation challenges, optimistic concurrency, scoped locks, authorization rechecks, preemption, safe cancellation, explicit compensation, idempotent creation, and secret-free terminal receipts. External side effects remain behind prepare/compensate ports.

Automated coverage includes exact retry/body conflicts, challenge binding, stale writes, resource conflicts and release, effect receipts, authorization denial, failure/retry/compensation, checksum chains, and catalog serialization. Durable adapter qualification additionally requires crash injection before and after every adapter fsync and application-effect commit boundary; the in-memory reference journal does not claim disk durability.
