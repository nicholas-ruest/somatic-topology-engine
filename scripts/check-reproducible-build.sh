#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
epoch="${SOURCE_DATE_EPOCH:-$(git -C "$root" log -1 --pretty=%ct 2>/dev/null || printf '0')}"
first="$(mktemp -d)"
second="$(mktemp -d)"
trap 'rm -rf "$first" "$second"' EXIT

build() {
  local target_dir="$1"
  SOURCE_DATE_EPOCH="$epoch" CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$target_dir" \
    cargo build --manifest-path "$root/Cargo.toml" --release --locked -p ste-cli
}

build "$first"
build "$second"
sha256sum "$first/release/ste" "$second/release/ste"
cmp "$first/release/ste" "$second/release/ste"
echo "release binary is reproducible for SOURCE_DATE_EPOCH=$epoch"
