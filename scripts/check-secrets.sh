#!/usr/bin/env bash
set -euo pipefail

root="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
patterns='-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|gh[pousr]_[A-Za-z0-9]{36,255}|xox[baprs]-[A-Za-z0-9-]{10,}'

if command -v rg >/dev/null 2>&1; then
  matches="$(rg -n --hidden \
    --glob '!.git/**' --glob '!target/**' --glob '!fuzz/artifacts/**' \
    --glob '!scripts/check-secrets.sh' -- "$patterns" "$root" || true)"
else
  matches="$(grep -REn --exclude-dir=.git --exclude-dir=target \
    --exclude-dir=.agents --exclude-dir=.claude --exclude-dir=.claude-flow \
    --exclude-dir=.codex --exclude-dir=.swarm \
    --exclude=check-secrets.sh -E -- "$patterns" "$root" || true)"
fi

if [[ -n "$matches" ]]; then
  printf '%s\n' "$matches"
  echo "secret-like material detected; revoke it before removing this gate" >&2
  exit 1
fi

echo "repository secret canaries passed"
