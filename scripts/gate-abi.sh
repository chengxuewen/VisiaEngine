#!/usr/bin/env bash
# gate-abi (I2 / plan v1.4 s6; P1 tile two ports 33->35; R band raster two ports 44->46): default-feature build + nm whitelist == 49 + demo compile-and-run.
# Tool probe order: PATH -> host-spike conda prefix binaries ([FFI-R:FC-M1] no bare nm/cc on this machine);
# both missing = SKIP exit 0 (the conditional segment writes its own shell gate, [v13-LEDG/env-W4] -- CI/fresh clones never go red spuriously).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
NM=$(command -v nm || true); [ -n "$NM" ] || NM=.pixi/envs/host-spike/bin/x86_64-conda-linux-gnu-nm
CC=$(command -v cc || true);   [ -n "$CC" ] || CC=.pixi/envs/host-spike/bin/x86_64-conda-linux-gnu-cc
RE=$(command -v readelf || true); [ -n "$RE" ] || RE=.pixi/envs/host-spike/bin/x86_64-conda-linux-gnu-readelf
  echo "GATE-ABI SKIP (no nm/readelf/cc -- gate-abi family discipline: a bare clone never goes red spuriously)"
cargo build -p visiaengine-capi >/dev/null 2>&1 || { echo "GATE-ABI ✗ build"; exit 1; }
SO=target/debug/libvisiaengine.so
[ -f "$SO" ] || { echo "GATE-ABI ✗ missing $SO ([lib] name check)"; exit 1; }
# P1 (packaging-round v1.3): the cdylib must carry its own SONAME. Without one the consumer records
# whatever string sat on the link command line -- both forms measured locally: absolute path -> DT_NEEDED=/abs/....so; with a SONAME -> the bare name.
# want is derived live from the Cargo [lib] name: SONAME and artifact name share one source, so a future library rename that misses this line goes red here.
libname=$(grep -A2 '^\[lib\]' bindings/c/visiaengine-capi/Cargo.toml | sed -n 's/^name = "\([a-z_]*\)".*/\1/p' | head -1)
want="lib${libname}.so"
have=$("$RE" -d "$SO" | sed -n 's/.*SONAME.*\[\(.*\)\].*/\1/p')
[ -n "$want" ] || { echo "GATE-ABI ✗ failed to read the [lib] name (empty want = naming contract broken)"; exit 1; }
[ "$have" = "$want" ] || { echo "GATE-ABI ✗ SONAME: got='${have}' want='${want}' (a rename missed this line)"; exit 1; }
N=$("$NM" -D "$SO" | grep -c ' T visiaengine_' || true)
echo "ABI-SYMBOLS="$N/49" | SO_SIZE=$(du -h "$SO" | cut -f1)"
[ "$N" = "49" ] || { echo "GATE-ABI ✗ symbol count $N"; exit 1; }
# hpp mirror gate (N1.1, gap-analysis C1): every .so visiaengine_* symbol must have a
# wrapper method in the hpp. Predicate = short name (prefix stripped) appears as `name(`
# method form; the raw C symbol alone is NOT enough (it always appears as the call target).
# Known semantic renames live in an explicit alias list (RAII destroy, B1 renames) —
# anything absent verbatim AND not aliased = gate red.
# Self-test: rename the fly_state method -> gate goes red (verified 2026-09-28).
HPP=bindings/cpp/include/visiaengine/visiaengine.hpp
MISSING=$(
  "$NM" -D "$SO" | awk '/ T visiaengine_/{print $3}' | sed 's/^visiaengine_//' | sort -u | while read -r m; do
    case "$m" in
      destroy)            pat='destroy_now';;
      entity_set_visible) pat='set_visible';;
      entity_visible)     pat='entity_visible';;
      *)                  pat="$m";;
    esac
    # Declaration-form match: a hpp method line starts with spaces + type +
    # name( — matching `  name(` (indented) kills comment/argument matches.
    grep -qE "^[[:space:]]*[^/]*\b${pat}\(" "$HPP" || echo "visiaengine_${m}";
  done)
  [ -z "$MISSING" ] || { echo "GATE-ABI ✗ hpp missing forwarding methods: $MISSING"; exit 1; }
"$CC" -I bindings/c/visiaengine-capi/include examples/c/E701_demo_headless.c \
-L target/debug -lvisiaengine -o target/demo_headless || { echo "GATE-ABI ✗ demo compile"; exit 1; }
LD_LIBRARY_PATH=$PWD/target/debug ./target/demo_headless resources/data/twoprim.glb \
|| { echo "GATE-ABI ✗ demo run"; exit 1; }
# M3 exit criterion 4: the texture pipeline (texquad generated from the io-gltf builder source, GLTF-11 fixture)
LD_LIBRARY_PATH=$PWD/target/debug ./target/demo_headless resources/data/texquad.glb \
|| { echo "GATE-ABI ✗ demo texture run"; exit 1; }
echo "GATE-ABI ✓"
