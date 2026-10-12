#!/usr/bin/env bash
# Band 7 J1 build surface: wasm32 compile -> wasm-bindgen dual glue (web = browser ESM / nodejs = the mirror test)
# -> wasm-opt size (degrades when the binary is absent, [FFI-R:env-N1] keeps the exit) -> WEB-SIZE print (the print IS the measurement,
# no baseline file -- the v1.1 ruling). Run as: pixi run -e wasm-spike bash scripts/web-build.sh
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
OUT=target/web
mkdir -p "$OUT"
cargo build -p visiaengine-wasm --release --target wasm32-unknown-unknown || { echo "WEB-BUILD ✗ cargo"; exit 1; }
WASM=target/wasm32-unknown-unknown/release/visiaengine_wasm.wasm
CLI=$(command -v wasm-bindgen || echo .pixi/envs/wasm-spike/bin/wasm-bindgen)
[ -x "$CLI" ] || { echo "WEB-BUILD ✗ wasm-bindgen-cli missing (pin: see the capi Cargo)"; exit 1; }
"$CLI" --target web --out-dir "$OUT/pkg-web" "$WASM" || { echo "WEB-BUILD ✗ bindgen web"; exit 1; }
"$CLI" --target nodejs --out-dir "$OUT/pkg-node" "$WASM" || { echo "WEB-BUILD ✗ bindgen node"; exit 1; }
OPT=$(command -v wasm-opt || echo .pixi/envs/wasm-spike/bin/wasm-opt)
if [ -x "$OPT" ]; then
  cp "$OUT/pkg-web/visiaengine_wasm_bg.wasm" "$OUT/pkg-web/visiaengine_wasm_bg.raw.wasm"
  "$OPT" -Oz -o "$OUT/pkg-web/visiaengine_wasm_bg.wasm" "$OUT/pkg-web/visiaengine_wasm_bg.raw.wasm" || echo "WASM-OPT ⚠ failed; keeping the original"
  cp "$OUT/pkg-node/visiaengine_wasm_bg.wasm" "$OUT/pkg-node/visiaengine_wasm_bg.raw.wasm"
  "$OPT" -Oz -o "$OUT/pkg-node/visiaengine_wasm_bg.wasm" "$OUT/pkg-node/visiaengine_wasm_bg.raw.wasm" 2>/dev/null || true
else
  echo "WASM-OPT SKIP (no wasm-opt; the raw size is what gets accounted)"
fi
for f in "$OUT"/pkg-web/visiaengine_wasm_bg.wasm "$OUT"/pkg-web/visiaengine_wasm.js "$OUT"/pkg-web/visiaengine_wasm.d.ts; do
  [ -f "$f" ] || { echo "WEB-BUILD ✗ missing artifact $f"; exit 1; }
done
RAW=$(stat -c%s "$OUT/pkg-web/visiaengine_wasm_bg.wasm")
GZ=$(gzip -c "$OUT/pkg-web/visiaengine_wasm_bg.wasm" | wc -c)
echo "WEB-SIZE raw=${RAW}B gz=${GZ}B (web pkg; the node glue is accounted separately)"
echo "WEB-BUILD ✓"
