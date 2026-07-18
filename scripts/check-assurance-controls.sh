#!/usr/bin/env bash
set -euo pipefail

root="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
failed=0

require_file() {
  if [[ ! -s "$root/$1" ]]; then
    echo "ASSURANCE: missing required control: $1" >&2
    failed=1
  fi
}

require_pattern() {
  local file="$1" pattern="$2" description="$3"
  if ! grep -Eq "$pattern" "$root/$file"; then
    echo "ASSURANCE: $description is not enforced by $file" >&2
    failed=1
  fi
}

require_file ".github/workflows/security.yml"
require_file "scripts/check-secrets.sh"
require_file "scripts/generate-sbom.sh"
require_file "scripts/check-reproducible-build.sh"
require_file "fuzz/Cargo.toml"
require_file "fuzz/fuzz_targets/contract_envelope.rs"

if [[ -f "$root/.github/workflows/security.yml" ]]; then
  require_pattern ".github/workflows/security.yml" 'gitleaks' "secret scanning"
  require_pattern ".github/workflows/security.yml" 'cargo deny check' "license/source policy"
  require_pattern ".github/workflows/security.yml" 'generate-sbom\.sh' "SBOM generation"
  require_pattern ".github/workflows/security.yml" 'check-unsafe\.sh' "unsafe-code review"
  require_pattern ".github/workflows/security.yml" 'attest-build-provenance' "build provenance attestation"
fi

if [[ -f "$root/fuzz/Cargo.toml" ]]; then
  require_pattern "fuzz/Cargo.toml" 'cargo-fuzz' "cargo-fuzz metadata"
  require_pattern "fuzz/Cargo.toml" 'ste-contracts' "versioned contract parser fuzzing"
fi

exit "$failed"
