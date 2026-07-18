#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
validator="$repo_root/scripts/validate-architecture.sh"

"$validator" "$repo_root"

fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/crates/ste-radio-acquisition/src" "$fixture/crates/ste-signal-observation/src"
cat >"$fixture/crates/ste-radio-acquisition/Cargo.toml" <<'EOF'
[package]
name = "ste-radio-acquisition"
version = "0.0.0"
edition = "2024"
[dependencies]
ste-signal-observation = { path = "../ste-signal-observation" }
EOF
cat >"$fixture/crates/ste-signal-observation/Cargo.toml" <<'EOF'
[package]
name = "ste-signal-observation"
version = "0.0.0"
edition = "2024"
EOF
touch "$fixture/crates/ste-radio-acquisition/src/lib.rs" "$fixture/crates/ste-signal-observation/src/lib.rs"

if "$validator" "$fixture" >/dev/null 2>&1; then
  echo "architecture validator accepted a cross-context dependency" >&2
  exit 1
fi

echo "architecture boundary tests passed"
