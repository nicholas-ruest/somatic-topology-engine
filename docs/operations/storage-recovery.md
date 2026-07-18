# Storage inspection, recovery, and lifecycle operations

These local operations are privileged and fail closed. Every invocation must
carry a fresh exact governance decision from the authenticated local IPC
boundary. Configuration, administrators, adapters, and sidecars cannot bypass
or cache authorization. Backend errors use redacted stable messages; raw CSI,
participant data, encryption keys, and plaintext exports must never be printed.

## Command contract

Arguments following `ste storage` are parsed by the Rust operator boundary:

| Operation | Arguments | Safe default |
| --- | --- | --- |
| Journal inspection | `journal inspect` | Read-only, payload-redacted |
| Projection rebuild | `journal rebuild [--apply]` | Dry-run |
| Portable export | `export <destination>` | Encrypted manifest only |
| Recovery | `recover [--apply]` | Dry-run to last verified state |
| Participant deletion | `delete <pseudonym> [--apply]` | Dry-run |
| Factory reset | `factory-reset --confirm` | Refused without confirmation |
| Decommission | `decommission --confirm` | Refused without confirmation |

The checked-in CLI library defines a narrow `StorageOperations` port and a
`SteStorageOperations` adapter over the Rust `JournalStore`, `EventUpcaster`,
and `LifecycleManager` APIs. Encrypted export remains an explicit injected
service because only the composition root has the authorized manifest,
plaintext source, key provider, and destination policy; the CLI does not invent
them. Alternate adapters must pass the same authorization and fault tests. A
dry-run describes affected data classes and records but performs no journal,
projection, chunk, key, or identity mutation.

## Recovery procedure

1. Stop capture and verify the runtime is in a safe capture-disabled state.
2. Authenticate locally and obtain a fresh purpose/space/participant-exact
   governance decision.
3. Run inspection. Preserve its redacted report and current journal digest.
4. Run recovery or rebuild without `--apply`; review the verified checkpoint,
   ignored torn tail, migrations, and affected projections.
5. Resolve disk capacity, key availability, or unsupported schema errors. Never
   skip a corrupt interior record or rewrite source bytes in place.
6. Reauthorize, apply, and verify the resulting checkpoint and projection
   digest. Re-enable capture only through a separate authorization operation.

Unrecoverable interior corruption is explicit and terminal for automatic
recovery. Torn tails may recover only through the last checksummed record.
Interrupted migrations preserve original bytes. Factory reset and decommission
perform cryptographic erasure; reset returns to capture-disabled defaults while
decommission also retires device identity and is not an update mechanism.

## Export and deletion

Portable export contains a versioned encrypted manifest, algorithm/key
identifier (never key material), partitions, chunk digests, schema versions,
retention metadata, and ciphertext integrity data. Plaintext portable export is
unsupported. Participant deletion must propagate through journals according to
lifecycle policy, projections, caches, chunks, vector indexes, backups where
eligible, and encryption keys, and must produce a payload-minimized completion
event.

## Fault evidence

Run `bash tests/phase05/run-storage-fault-matrix.sh`. It validates the versioned
corpus under `tests/fixtures/storage` and explicitly executes deterministic
rebuild/upcast, torn write, checksum corruption, disk full, atomic checkpoint,
interrupted migration, chunk bound, compaction, and CLI authorization cases.
Fixture checksum placeholders are intentionally not accepted as production
records; the adapter test reconstructs valid records with the Rust storage API
before fault injection.
