#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output="${1:-$root/target/assurance/ste.cdx.json}"
mkdir -p "$(dirname "$output")"

metadata="$(mktemp)"
trap 'rm -f "$metadata"' EXIT
cargo metadata --manifest-path "$root/Cargo.toml" --locked --format-version 1 >"$metadata"

# A timestamp and random serial number are intentionally omitted. Given the
# same Cargo.lock and generator version, this CycloneDX document is byte-stable.
jq --sort-keys '{
  bomFormat: "CycloneDX",
  specVersion: "1.5",
  version: 1,
  metadata: {
    tools: [{vendor: "STE", name: "cargo-metadata-sbom", version: "1"}],
    component: {type: "application", name: "somatic-topology-engine"}
  },
  components: [.packages[] | {
    type: "library",
    name: .name,
    version: .version,
    purl: ("pkg:cargo/" + .name + "@" + .version),
    licenses: (if .license == null then [] else [{license: {id: .license}}] end)
  }] | unique_by(.purl) | sort_by(.purl)
}' "$metadata" >"$output"

echo "wrote reproducible CycloneDX SBOM: $output"
