# Local observability and support bundles

STE observability is local-only by default and separates diagnostics, security
events, domain audit, and traces. Stores rotate independently with explicit
dropped-record counters. Metrics accept only registered label keys and bounded
series counts. Raw CSI, inferred outputs, participant labels, identifiers,
credentials, keys, and free-form payload dumps are not supported diagnostic
fields.

The authenticated operator surface supports:

- `ste diagnostics health --json` for the stable payload-free health snapshot;
- `ste diagnostics support preview --json` for the exact checksummed manifest
  only.

Direct process invocation has no authenticated IPC identity or fresh governance
decision and exits fail-closed. Bundle content is schema-allowlisted: fields not
explicitly allowed for the exact record class/code are removed. Export requires
the exact preview token generated from current redacted content; any intervening
change invalidates confirmation. Preview lists logical names, sizes, and SHA-256
checksums but not record values.

Before export, the user reviews purpose, destination, manifest, retention, and
whether the report could identify a site/device. Run the redaction canary tests
and inspect the generated files. Never add a sensitive field to an allowlist to
work around a support request; use a separately consented governed evidence
workflow.

Operational checks:

```bash
cargo test -p ste-observability --all-targets
cargo test -p ste-cli --test diagnostics_commands
bash tests/phase08/run-fault-matrix.sh
```

An elevated dropped counter, saturation count, or cardinality rejection is a
health signal, not permission to make stores unbounded.
