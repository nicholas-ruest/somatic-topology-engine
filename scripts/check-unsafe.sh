#!/usr/bin/env bash
set -euo pipefail
root="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
pattern='(^|[^[:alnum:]_])unsafe[[:space:]]*(\{|fn|impl|trait)'
if command -v rg >/dev/null 2>&1; then
  matches="$(rg -n --glob '*.rs' "$pattern" "$root/crates" || true)"
else
  matches="$(grep -REn --include='*.rs' "$pattern" "$root/crates" || true)"
fi
if [[ -n "$matches" ]]; then
  printf '%s\n' "$matches"
  echo "unsafe Rust requires a dedicated ADR exception and audited allowlist" >&2
  exit 1
fi
