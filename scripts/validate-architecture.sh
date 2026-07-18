#!/usr/bin/env bash
set -euo pipefail

root="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
context_pattern='ste-(radio-acquisition|signal-observation|physiology-estimation|state-inference|personalization-memory|experiment-validation|device-interaction|consent-governance)'
failed=0

while IFS= read -r manifest; do
  crate="$(basename "$(dirname "$manifest")")"
  [[ "$crate" =~ ^${context_pattern}$ ]] || continue
  while IFS= read -r dependency; do
    [[ -z "$dependency" || "$dependency" == "$crate" ]] && continue
    echo "BOUNDARY: $crate must not depend directly on $dependency ($manifest)" >&2
    failed=1
  done < <(sed -nE "s/^[[:space:]]*(${context_pattern})[[:space:]]*=.*/\1/p" "$manifest")
done < <(find "$root/crates" -mindepth 2 -maxdepth 2 -name Cargo.toml -type f 2>/dev/null | sort)

# Rust paths reaching into another context's private modules are forbidden even
# when introduced through test fixtures or path aliases.
contexts=(radio_acquisition signal_observation physiology_estimation state_inference personalization_memory experiment_validation device_interaction consent_governance)
for owner in "${contexts[@]}"; do
  owner_dir="$root/crates/ste-${owner//_/-}"
  [[ -d "$owner_dir" ]] || continue
  for target in "${contexts[@]}"; do
    [[ "$owner" == "$target" ]] && continue
    private_pattern="ste_${target}::(domain|infrastructure)::"
    if command -v rg >/dev/null 2>&1; then
      matches="$(rg -n --glob '*.rs' "$private_pattern" "$owner_dir" || true)"
    else
      matches="$(grep -REn --include='*.rs' "$private_pattern" "$owner_dir" || true)"
    fi
    if [[ -n "$matches" ]]; then
      printf '%s\n' "$matches"
      echo "BOUNDARY: direct $owner -> $target private-module import" >&2
      failed=1
    fi
  done
done

exit "$failed"
