# STE integration contracts

`ste-contracts` owns the stable, Rust-authored transport DTOs described by
ADR-012, ADR-017, and ADR-019. It does not contain domain entities, policies,
application services, persistence, or infrastructure adapters.

Every integration event uses `ContractEnvelopeV1`, whose required metadata
preserves identity, timing, causality, provenance, and idempotency. Contract
floating-point values use `FiniteF64`; NaN and infinities are rejected.
Consumers explicitly check `SchemaSupport` before interpreting an envelope.

Generate the canonical JSON Schema bundle with:

```text
cargo run -p ste-contracts --example generate-schemas
```

The command writes deterministic JSON to standard output so CI or binding
generation can compare or redirect it without hidden filesystem mutations.
