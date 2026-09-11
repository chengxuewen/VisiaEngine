#!/usr/bin/env bash
# 批7 J0 验收：wasm 目标 std 到位 + rust/rust-std 版本相等（[FFI-R:env-B1/B2]）。
# 必须在 wasm-spike 环境内跑：pixi run -e wasm-spike bash scripts/web-probe.sh
set -uo pipefail
L=$(rustc --print target-libdir --target wasm32-unknown-unknown)
if ls "$L"/libstd-*.rlib >/dev/null 2>&1; then echo "TARGET-STD ✓ $L"; else echo "TARGET-STD ✗ ($L 空)"; exit 1; fi
RV=$(rustc --version | sed -E 's/rustc ([0-9]+\.[0-9]+\.[0-9]+).*/\1/')
# conda-meta 直读（首版 dirname 链算错致 SVv 空，实测纠=CONDA_PREFIX 即 sysroot 根）
CM="${CONDA_PREFIX:-$(rustc --print sysroot)}/conda-meta"
SVv=$(ls "$CM"/rust-std-wasm32-unknown-unknown-*.json 2>/dev/null | head -1 | xargs -r basename | sed -E 's/rust-std-wasm32-unknown-unknown-([0-9]+\.[0-9]+\.[0-9]+)-.*/\1/')
if [ "$RV" = "$SVv" ]; then echo "VERSION-SYNC ✓ $RV == $SVv"; else echo "VERSION-SYNC ✗ rustc=$RV std=$SVv"; exit 1; fi
echo "PROBE-OK"
