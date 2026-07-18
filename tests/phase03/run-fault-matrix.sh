#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Each named acceptance test is a required fault-control witness. Naming them
# explicitly prevents a broad test command from silently losing a scenario.
tests=(
  bounded_queue_rejects_optional_work_but_never_sheds_accepted_critical_work
  drop_oldest_policy_only_evicts_optional_work
  crashed_optional_task_degrades_without_stopping_critical_tasks
  restart_budget_opens_circuit_after_bounded_retries
  cancellation_and_shutdown_are_coordinated_and_verifiably_safe
  clock_discontinuity_and_low_resources_force_safe_degradation
)

for test_name in "${tests[@]}"; do
  cargo test --manifest-path "$root/Cargo.toml" --locked -p ste-runtime \
    --test supervision "$test_name" -- --exact
done
cargo test --manifest-path "$root/Cargo.toml" --locked -p ste-runtime \
  --test synthetic_pipeline
echo "Phase 03 fault matrix passed (${#tests[@]} fault controls plus replay suite)"
