#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixtures="$root/tests/fixtures/storage"
jq -e '.schema == "ste-storage-fault-corpus-v1" and (.cases | length == 5)' \
  "$fixtures/corpus.json" >/dev/null
while IFS= read -r fixture; do
  [[ -s "$fixtures/$fixture" ]] || { echo "missing storage fixture: $fixture" >&2; exit 1; }
done < <(jq -r '.cases[].file' "$fixtures/corpus.json")

tests=(
  partitioned_journal_rebuild_is_deterministic_and_upcasted
  torn_tail_recovers_only_last_verified_record_and_reports_loss
  checksum_corruption_is_explicit_and_never_silently_skipped
  disk_full_append_is_atomic_and_does_not_consume_sequence
  checkpoint_commit_is_atomic_and_replay_is_idempotent
  interrupted_migration_preserves_original_bytes
  bounded_chunks_are_content_addressed_and_reject_overflow
  verified_compaction_is_atomic_and_retains_requested_suffix
)
for test_name in "${tests[@]}"; do
  cargo test --manifest-path "$root/Cargo.toml" --locked -p ste-storage \
    --test journal_recovery "$test_name" -- --exact
done
cargo test --manifest-path "$root/Cargo.toml" --locked -p ste-cli --test storage_commands
echo "Phase 05 storage fault matrix passed (${#tests[@]} journal faults plus CLI boundary)"
