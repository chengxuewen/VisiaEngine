#!/usr/bin/env bash
# 批 7 J1 构建面：wasm32 编译 → wasm-bindgen 双胶水（web=浏览器 ESM / nodejs=镜像测试）
# → wasm-opt 体积（缺件降级，[FFI-R:env-N1] 保留退路）→ WEB-SIZE 打印（打印即测量，
# 无 baseline 文件——v1.1 裁决）。跑法：pixi run -e wasm-spike bash scripts/web-build.sh
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
OUT=target/web
mkdir -p "$OUT"
cargo build -p visiaengine-wasm --release --target wasm32-unknown-unknown || { echo "WEB-BUILD ✗ cargo"; exit 1; }
WASM=target/wasm32-unknown-unknown/release/visiaengine_wasm.wasm
CLI=$(command -v wasm-bindgen || echo .pixi/envs/wasm-spike/bin/wasm-bindgen)
[ -x "$CLI" ] || { echo "WEB-BUILD ✗ wasm-bindgen-cli 缺（pin 见 capi Cargo）"; exit 1; }
"$CLI" --target web --out-dir "$OUT/pkg-web" "$WASM" || { echo "WEB-BUILD ✗ bindgen web"; exit 1; }
"$CLI" --target nodejs --out-dir "$OUT/pkg-node" "$WASM" || { echo "WEB-BUILD ✗ bindgen node"; exit 1; }
OPT=$(command -v wasm-opt || echo .pixi/envs/wasm-spike/bin/wasm-opt)
if [ -x "$OPT" ]; then
  cp "$OUT/pkg-web/visiaengine_wasm_bg.wasm" "$OUT/pkg-web/visiaengine_wasm_bg.raw.wasm"
  "$OPT" -Oz -o "$OUT/pkg-web/visiaengine_wasm_bg.wasm" "$OUT/pkg-web/visiaengine_wasm_bg.raw.wasm" || echo "WASM-OPT ⚠ 失败保留原件"
  cp "$OUT/pkg-node/visiaengine_wasm_bg.wasm" "$OUT/pkg-node/visiaengine_wasm_bg.raw.wasm"
  "$OPT" -Oz -o "$OUT/pkg-node/visiaengine_wasm_bg.wasm" "$OUT/pkg-node/visiaengine_wasm_bg.raw.wasm" 2>/dev/null || true
else
  echo "WASM-OPT SKIP（无 wasm-opt，raw 计账）"
fi
for f in "$OUT"/pkg-web/visiaengine_wasm_bg.wasm "$OUT"/pkg-web/visiaengine_wasm.js "$OUT"/pkg-web/visiaengine_wasm.d.ts; do
  [ -f "$f" ] || { echo "WEB-BUILD ✗ 缺产物 $f"; exit 1; }
done
RAW=$(stat -c%s "$OUT/pkg-web/visiaengine_wasm_bg.wasm")
GZ=$(gzip -c "$OUT/pkg-web/visiaengine_wasm_bg.wasm" | wc -c)
echo "WEB-SIZE raw=${RAW}B gz=${GZ}B (web pkg；node 胶水另算)"
echo "WEB-BUILD ✓"
