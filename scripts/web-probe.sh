#!/usr/bin/env bash
# Band 7 J0 acceptance: the wasm target std is in place + the rust/rust-std versions are equal ([FFI-R:env-B1/B2]).
# Must run inside the wasm-spike environment: pixi run -e wasm-spike bash scripts/web-probe.sh
set -uo pipefail
L=$(rustc --print target-libdir --target wasm32-unknown-unknown)
if ls "$L"/libstd-*.rlib >/dev/null 2>&1; then echo "TARGET-STD ✓ $L"; else echo "TARGET-STD ✗ ($L empty)"; exit 1; fi
RV=$(rustc --version | sed -E 's/rustc ([0-9]+\.[0-9]+\.[0-9]+).*/\1/')
# Read conda-meta directly (the first version's dirname chain computed it wrong, leaving SVv empty; the measured fix = CONDA_PREFIX IS the sysroot root)
CM="${CONDA_PREFIX:-$(rustc --print sysroot)}/conda-meta"
SVv=$(ls "$CM"/rust-std-wasm32-unknown-unknown-*.json 2>/dev/null | head -1 | xargs -r basename | sed -E 's/rust-std-wasm32-unknown-unknown-([0-9]+\.[0-9]+\.[0-9]+)-.*/\1/')
if [ "$RV" = "$SVv" ]; then echo "VERSION-SYNC ✓ $RV == $SVv"; else echo "VERSION-SYNC ✗ rustc=$RV std=$SVv"; exit 1; fi
echo "PROBE-OK"
