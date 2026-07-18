#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 2 ]]; then
  echo "usage: $0 <source-report> <report-name>" >&2
  exit 2
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_report="$1"
name="$2"
destination="$root/docs/assurance/reports/$name"

[[ "$name" =~ ^[a-z0-9][a-z0-9._-]*\.(json|md|txt)$ ]] || {
  echo "report name must be lowercase and use json, md, or txt" >&2
  exit 2
}
[[ -s "$source_report" ]] || { echo "source report is empty or missing" >&2; exit 1; }
mkdir -p "$(dirname "$destination")"
install -m 0644 "$source_report" "$destination"
(cd "$(dirname "$destination")" && sha256sum "$(basename "$destination")") \
  >"$destination.sha256"
echo "recorded $destination with SHA-256 sidecar"
