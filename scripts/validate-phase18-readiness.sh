#!/usr/bin/env bash
set -euo pipefail
project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.."&&pwd);cd "$project_root"
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/test-architecture.sh
bash scripts/check-assurance-controls.sh
for file in docs/templates/commercial-pilot.md docs/reports/phase-18-pilot-status.md docs/assurance/phase-18-claim-evidence-matrix.md docs/operations/commercial-operations.md docs/operations/release-runbook.md docs/reports/phase-18-production-readiness.md;do test -s "$file";done
output="$project_root/target/validation/phase-18-readiness-decision.json";mkdir -p "$(dirname "$output")";cargo run --quiet -p ste-cli --example production_readiness_status > "$output"
jq -e '.decision.status == "Blocked" and (.decision.corrective_actions|length)>=12' "$output" >/dev/null
status="$project_root/target/validation/phase-18-status.json";jq -n --sort-keys --arg decision_sha "$(sha256sum "$output"|cut -d' ' -f1)" '{schema:"ste-phase18-status-v1",decision:"NOT_APPROVED",commercial_sale:false,production_deployment:false,pilot:"not_conducted",legal_regulatory_approvals:"not_recorded",physical_reference_device:"unqualified",hil_soak:"pending",penetration_fuzz:"pending",human_claim_validation:"pending",commercial_operations:"documented_not_exercised",readiness_decision_sha256:$decision_sha}' > "$status"
echo "phase 18 automated evidence passed; production remains NOT APPROVED: $status"
