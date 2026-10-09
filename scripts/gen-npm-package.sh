#!/usr/bin/env bash
# gen-npm-package.sh — assemble the npm-shaped deliverable from the EXISTING wasm build.
# Input:  target/web/pkg-{node,web}/ written by scripts/web-build.sh (pixi run web-check).
# Output: build/npm/visiaengine/ (build/ is gitignored) + build/npm/visiaengine-<ver>.tgz
#         produced by `npm pack`. LOCAL artifact only: nothing is published or uploaded,
#         and the package name has deliberately NOT been checked against any registry
#         (we run offline). Registry-name collision is a publish-band concern.
# Version single source: Cargo.toml [workspace.package] version — never hand-typed (#17).
# Freshness guard mirrors scripts/web-mirror.mjs (PIT-44): stale build output must never
# be packaged as if it were current.
# Builder, not a gate — out of the ci chain (same policy as gallery / keys-probe).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."

SRC_NODE=target/web/pkg-node
SRC_WEB=target/web/pkg-web
OUT=build/npm/visiaengine

for f in visiaengine_wasm.js visiaengine_wasm_bg.wasm visiaengine_wasm.d.ts; do
  [ -f "$SRC_NODE/$f" ] || { echo "NPM-PACK ✗ missing $SRC_NODE/$f — run: pixi run web-check"; exit 1; }
  [ -f "$SRC_WEB/$f" ]  || { echo "NPM-PACK ✗ missing $SRC_WEB/$f — run: pixi run web-check"; exit 1; }
done
stale=$(find bindings/js/rust/visiaengine-wasm/src bindings/c/visiaengine-capi/src \
        -type f -newer "$SRC_NODE/visiaengine_wasm.d.ts" -print -quit)
if [ -n "$stale" ]; then
  echo "NPM-PACK ✗ stale build output: $stale is newer than $SRC_NODE — run: pixi run web-check"
  exit 1
fi

ver=$(sed -n '/^\[workspace\.package\]/,/^\[/{s/^version *= *"\([^"]*\)".*/\1/p}' Cargo.toml | head -1)
[ -n "$ver" ] || { echo "NPM-PACK ✗ cannot read [workspace.package] version from Cargo.toml"; exit 1; }
command -v npm >/dev/null || { echo "NPM-PACK ✗ npm not on PATH — run via: pixi run npm-pack"; exit 1; }

rm -rf "$OUT"
mkdir -p "$OUT/node" "$OUT/web"
# ship js / wasm / d.ts per flavor; *.raw.wasm is the pre-wasm-opt intermediate, not shipped
for pair in "$SRC_NODE:$OUT/node" "$SRC_WEB:$OUT/web"; do
  src=${pair%%:*}; dst=${pair##*:}
  for f in visiaengine_wasm.js visiaengine_wasm_bg.wasm visiaengine_wasm.d.ts visiaengine_wasm_bg.wasm.d.ts; do
    [ -f "$src/$f" ] && cp "$src/$f" "$dst/$f"
  done
done
cp LICENSE-MIT LICENSE-APACHE "$OUT/"

# main=CJS(node flavor) / module=ESM(web flavor) / types=d.ts.
# ponytail: no "exports" conditions map yet — add when a bundler consumer needs it.
cat > "$OUT/package.json" <<JSON
{
  "name": "visiaengine",
  "version": "$ver",
  "description": "VisiaEngine embedded 2D/2.5D/3D spatial visualization engine — wasm build of the C ABI mirror (WebGPU backend)",
  "license": "MIT OR Apache-2.0",
  "main": "node/visiaengine_wasm.js",
  "module": "web/visiaengine_wasm.js",
  "types": "node/visiaengine_wasm.d.ts",
  "files": [
    "node/",
    "web/",
    "LICENSE-MIT",
    "LICENSE-APACHE"
  ]
}
JSON

(cd "$OUT" && npm pack --pack-destination ..) || { echo "NPM-PACK ✗ npm pack failed"; exit 1; }
TGZ=build/npm/visiaengine-$ver.tgz
[ -f "$TGZ" ] || { echo "NPM-PACK ✗ expected tarball $TGZ absent"; exit 1; }
echo "NPM-PACK ✓ $OUT (version=$ver read from Cargo.toml [workspace.package])"
echo "NPM-PACK ✓ tarball: $TGZ (local only — not published anywhere)"
