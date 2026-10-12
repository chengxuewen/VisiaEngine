#!/usr/bin/env bash
# J3 human-verification server: assembles the demo directory (pkg + data + page) -> http://127.0.0.1:8017
# Usage: bash scripts/web-serve.sh (prerequisite: web-build.sh has run). Ctrl-C to stop.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
D=target/web/demo
mkdir -p "$D"
cp bindings/js/demo/index.html "$D/"
cp -r target/web/pkg-web "$D/pkg"
cp resources/data/park.geojson "$D/"
echo "▶ http://127.0.0.1:8017 (T3 manual checklist on the page; Ctrl-C to exit)"
cd "$D" && python3 -m http.server 8017
