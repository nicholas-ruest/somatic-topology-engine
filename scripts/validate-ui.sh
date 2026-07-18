#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$repo_root"
cargo test -p ste-ui-gateway -p ste-web-gateway -p ste-query-plane -p ste-workflows --all-targets --locked
cargo clippy -p ste-ui-gateway -p ste-web-gateway -p ste-query-plane -p ste-workflows --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p ste-ui-gateway -p ste-web-gateway -p ste-query-plane -p ste-workflows --no-deps --locked

cd "$repo_root/ui"
npm ci --ignore-scripts
npm test
npm run build
npm audit --audit-level=high
npm run benchmark:visualizations

test -f dist/index.html
test -f dist/asset-manifest.json
test -z "$(find dist -type f -name '*.map' -print -quit)"
! grep -Eiq '(src|href)="https?://' dist/index.html
node -e 'const fs=require("fs"),crypto=require("crypto");const root="dist/";const manifest=JSON.parse(fs.readFileSync(root+"asset-manifest.json"));if(!manifest.assets.length)process.exit(1);for(const asset of manifest.assets){const body=fs.readFileSync(root+asset.path);if(body.length!==asset.bytes||crypto.createHash("sha256").update(body).digest("hex")!==asset.sha256)process.exit(1)}'

cd "$repo_root"
git diff --check
