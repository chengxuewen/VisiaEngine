#!/usr/bin/env bash
# cmake-smoke (plan §2.4): a three-state gate -- no cmake = SKIP exit 0 (the gate-abi family discipline);
# cmake present = bare configure+build+ctest, plus negative-path message asserts (SYSTEM anti-pollution / missing prebuilt artifact).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
CM=$(command -v cmake || true)
[ -n "$CM" ] || CM=.pixi/envs/default/bin/cmake
if [ ! -x "$CM" ]; then
    echo "CMAKE-SMOKE SKIP (no cmake -- run pixi install first; this segment never goes red on a bare-clone machine)"
    exit 0
fi
CT="$(dirname "$CM")/ctest"
# Artifact file name for the manifest check (single source = the CMake naming table,
# VisiaEngineFfi.cmake:26-35; duplicated here only as a per-OS literal).
case "$(uname -s)" in
  Linux)  VE_SHARED=libvisiaengine.so ;;
  Darwin) VE_SHARED=libvisiaengine.dylib ;;
  *)      VE_SHARED=visiaengine.dll ;;
esac
B=target/cmake-smoke
rm -rf "$B" "$B-neg" "$B-negconf"

"$CM" -S . -B "$B" -G Ninja -DCMAKE_BUILD_TYPE=Debug -DVISIAENGINE_QT=OFF >/dev/null \
    || { echo "CMAKE-SMOKE ✗ bare configure"; exit 1; }
"$CM" --build "$B" >/dev/null || { echo "CMAKE-SMOKE ✗ build"; exit 1; }
"$CT" --test-dir "$B" -LE display --output-on-failure >/dev/null 2>&1 \
    || { echo "CMAKE-SMOKE ✗ ctest (probe/headless example failed)"; exit 1; }
# ↑ -LE display: the probe has no label and the headless examples really run; the display family (popup windows) is covered by smoke-x11/smoke-qt, the dedicated xvfb runners

# README count lock (2026-10-08: README said 40 while the registry actually had 43, and nobody was watching). The ground truth lives only in
# this script (it needs a configured tree), so the lock lives here; the README-wall-count half lives in gate-docs ⑦.
NC=$( "$CT" --test-dir "$B" -N 2>/dev/null | grep -c '^  Test ' )
RN=$(grep -oE 'ctest unified example registry \([0-9]+ entries' README.md | grep -oE '[0-9]+' | head -1 || true)
[ -n "$RN" ] || { echo "CMAKE-SMOKE ✗ failed to read the README count (an assert with no object is vacuously true)"; exit 1; }
[ "$NC" = "$RN" ] || { echo "CMAKE-SMOKE ✗ README count=$RN ≠ ctest actually registered=$NC"; exit 1; }

# Fourth state: the bare-PATH triple anchor (the IDE direct-invocation shape guard; B1 adjudication: configure probe + double build anchor, no fallback added after the Rust example family changed shape)
# (2026-09-15, measured: the pixi run wrapper masked it; the user's IDE /usr/bin/cmake direct call blew up with "could not execute rustc")
SYS_CM=$(command -v /usr/bin/cmake || true); [ -n "$SYS_CM" ] || SYS_CM="$CM"  # the probe = the user's real IDE shape (system cmake; machines without a system cmake fall back to the conda binary)
VE_CONDA_BIN="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/.pixi/envs/default/bin"  # the conda toolchain anchor, ABSOLUTE (a relative CXX path fails project()'s compiler probe -- measured); the D5 environment is the single source for the prefix
# Pinning the compiler to absolute paths keeps the probe focused on cargo's self-sufficiency (host reality: no system g++; all C++ lives on the conda side; the real IDE shape is covered separately by state ①)
env PATH=/usr/bin:/bin "$SYS_CM" -S . -B "$B-negconf" -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Debug -DVISIAENGINE_QT=OFF \
    -DCMAKE_C_COMPILER=/usr/bin/gcc "-DCMAKE_CXX_COMPILER=${VE_CONDA_BIN}/x86_64-conda-linux-gnu-c++" \
    >/dev/null 2>&1 \
    || { echo "CMAKE-SMOKE ✗ bare-PATH configure (SDK/cargo self-sufficiency broken -- the .pixi fallback snap)"; exit 1; }
rm -rf "$B-negconf"
for _t in visiaengine-ffi-build visiaengine-example-build; do
  env PATH=/usr/bin:/bin "$CM" --build "$B" --target "$_t" >/dev/null 2>&1 \
      || { echo "CMAKE-SMOKE ✗ bare PATH ${_t} (cargo environment self-sufficiency broken = the IDE would explode)"; exit 1; }
done

# Negative path 1: SYSTEM semantics (activated shell = reject pollution; a genuine system cargo = a legitimate pass; anything else must error with a message containing SYSTEM)
out=$("$CM" -S . -B "$B-neg" -G Ninja -DVISIAENGINE_QT=OFF -DVISIAENGINE_RUST_SDK=SYSTEM 2>&1)
rc=$?
if [ $rc -eq 0 ]; then
    echo "  note: SYSTEM mode ran on a genuine system cargo (a legitimate environment state)"
elif ! grep -q "SYSTEM" <<<"$out"; then
    echo "CMAKE-SMOKE ✗ SYSTEM negative-path message missing"; exit 1
fi
# Negative path 3: PolyOrch absent (submodule not pulled / bare clone) -- the shim FATAL message must be readable and point the way
# (D39 embed contract 2026-10-10; message asserts grep short phrases so cmake line wrapping cannot evade them -- same discipline as negative paths 1/2)
mv PolyOrch PolyOrch.smoke-hidden 2>/dev/null  # when it is a submodule the directory exists; machines missing the directory naturally take the same assert
rm -rf "$B-neg3"  # a separate directory: the SYSTEM cache from negative path 1 would blow first (SDK:86) before the shim ever runs -- measured trap
out=$("$CM" -S . -B "$B-neg3" -G Ninja -DVISIAENGINE_QT=OFF 2>&1)  # purely bare: no escape-hatch flags, otherwise SDK resolution dies first and the shim never runs
rm -rf "$B-neg3"
mv PolyOrch.smoke-hidden PolyOrch 2>/dev/null || true
if ! grep -q "submodule update --init" <<<"$out"; then
    echo "CMAKE-SMOKE ✗ the PolyOrch-absent message lost its remediation pointer (the shim's fix guidance is gone)"; exit 1
fi

# Negative path 5: stale supplier floor -- an outer sentinel reporting 0.1.1 must FATAL on
# the EXAMPLE-kind floor (T-D 2026-10-12: keyword capability is invisible to COMMAND probes,
# so the floor rides the sentinel's INTERFACE_POLYORCH_VERSION)
rm -rf "$B-neg5" "$B-neg5-host"
mkdir -p "$B-neg5-host"
cat > "$B-neg5-host/CMakeLists.txt" <<'NEGEOF'
cmake_minimum_required(VERSION 3.25)
project(neg5 LANGUAGES CXX)
add_library(PolyOrch::PolyOrch INTERFACE IMPORTED GLOBAL)
set_property(TARGET PolyOrch::PolyOrch PROPERTY INTERFACE_POLYORCH_CMAKE_DIR "/nonexistent")
set_property(TARGET PolyOrch::PolyOrch PROPERTY INTERFACE_POLYORCH_VERSION "0.1.1")
add_subdirectory("${CMAKE_CURRENT_LIST_DIR}/../.." V)
NEGEOF
out=$("$CM" -S "$B-neg5-host" -B "$B-neg5" -G Ninja 2>&1); rc5=$?
rm -rf "$B-neg5" "$B-neg5-host"
[ "$rc5" -ne 0 ] && grep -q "needs PolyOrch >= 0.2.0" <<<"$out" \
    || { echo "CMAKE-SMOKE ✗ stale PolyOrch did not trip the version floor (rc=$rc5)"; exit 1; }

# Negative path 4: master-switch contradiction (VISIAENGINE_USE_PIXI=OFF + VISIAENGINE_RUST_SDK=PIXI)
# must FATAL with the contradiction message (T-B1 2026-10-10). Own build dir (negative-path-1
# cache discipline); phrase grep survives cmake line wrapping.
rm -rf "$B-neg4"
out=$("$CM" -S . -B "$B-neg4" -G Ninja -DVISIAENGINE_QT=OFF -DVISIAENGINE_USE_PIXI=OFF \
    -DVISIAENGINE_RUST_SDK=PIXI 2>&1); rc4=$?
rm -rf "$B-neg4"
[ "$rc4" -ne 0 ] && grep -q "Contradiction" <<<"$out" \
    || { echo "CMAKE-SMOKE ✗ the master-switch contradiction did not go loud (rc=$rc4)"; exit 1; }

# Negative path 2: the prebuilt escape hatch with a missing artifact = hard error (environment-independent determinism)
out=$("$CM" -S . -B "$B-neg" -G Ninja -DVISIAENGINE_QT=OFF -DVISIAENGINE_ARTIFACT_PATH=/tmp/ve-nope 2>&1)
[ $? -ne 0 ] && grep -q "no capi artifact" <<<"$out" \
    || { echo "CMAKE-SMOKE ✗ the escape hatch did not hard-error on the missing artifact (silent consumption crept back)"; exit 1; }  # phrase assert: cmake messages wrap lines; a whole-sentence grep would always miss

# Hygiene lock · the FOLDER position rule: every literal must be a real directory on disk (invented words like _rs-steps/7x-host/_bridge die at the rules layer); ${} template lines are exempt
BAD=""
for v in $(grep -rhoE 'FOLDER "[^"]+"' cmake examples bindings 2>/dev/null | sed 's/^FOLDER "//; s/"$//'); do
  case "$v" in *'${'*) continue;; esac
  [ -d "$v" ] || BAD="$BAD $v"
done
[ -z "$BAD" ] || { echo "CMAKE-SMOKE ✗ FOLDER is not a disk directory (the position-mirror rule; identity semantics belong to LABELS, see D14/R9):$BAD"; exit 1; }

# State 4, positive (packaging-round v1.3 P2: the S-c beta "install must fail"
# guard retired). The mirror-image risk now is a *silent partial or wrong tree*,
# so the checks are: rc==0, manifest equality, and .pc prefix self-consistency.
# Cleanup runs LAST -- checking a tree you already deleted is a vacuous pass.
"$CM" --install "$B" --prefix "$B-inst" >"$B-inst.out" 2>&1; i_rc=$?
if [ $i_rc -ne 0 ]; then
  echo "CMAKE-SMOKE ✗ install rc=$i_rc"; tail -3 "$B-inst.out"; rm -rf "$B-inst" "$B-inst.out"; exit 1
fi
MISS=""
for f in "lib/$VE_SHARED" "lib/pkgconfig/visiaengine.pc" \
         "lib/cmake/visiaengine/visiaengineConfig.cmake" \
         "lib/cmake/visiaengine/visiaengineConfigVersion.cmake" \
         "include/visiaengine.h" \
         "include/visiaengine/visiaengine.hpp" \
         "include/visiaengine_widget.hpp" \
         "share/licenses/visiaengine/LICENSE-MIT" \
         "share/licenses/visiaengine/LICENSE-APACHE"; do
  [ -f "$B-inst/$f" ] || MISS="$MISS $f"
done
if [ -n "$MISS" ]; then
  echo "CMAKE-SMOKE ✗ installed tree missing files:$MISS"; rm -rf "$B-inst" "$B-inst.out"; exit 1
fi
NTREE=$(find "$B-inst" -type f | wc -l)
TREE_NOTE="install tree ${NTREE} files"
# File presence does not prove the .pc is right: a configure-time prefix bake
# reports /usr/local while the tree sits elsewhere (caught live in this slice).
# Asserted on the file text, not via pkg-config: `pkg-config --variable=libdir`
# was measured to keep returning the tree path even when the file said
# /usr/local, i.e. it cannot be the witness for this property.
pc="$B-inst/lib/pkgconfig/visiaengine.pc"
grep -q '^prefix=${pcfiledir}/' "$pc" \
  || { echo "CMAKE-SMOKE ✗ .pc prefix is not in pcfiledir form (a baked prefix = the tree is dead on arrival)"; rm -rf "$B-inst" "$B-inst.out"; exit 1; }
grep -q "$PWD/target" "$pc" \
  && { echo "CMAKE-SMOKE ✗ .pc contains a build-directory absolute path (release-artifact leakage)"; rm -rf "$B-inst" "$B-inst.out"; exit 1; }
grep -q '^URL: https\?://' "$pc" \
  || { echo "CMAKE-SMOKE ✗ .pc URL was not substituted from Cargo.toml repository"; rm -rf "$B-inst" "$B-inst.out"; exit 1; }
TREE_NOTE="$TREE_NOTE / .pc text triple-check"
rm -rf "$B-inst" "$B-inst.out"

# display sub-state (B2): Xvfb self-starts and really runs the window family (the E702/E801 auto-execution track; same shape as apt CI)
# if it cannot start = the XKB/pixi clobber trap (PIT-19) -> a note, never a false green; xvfb-run is a Debian wrapper and conda lacks it, so start Xvfb directly
XV=$(command -v Xvfb || true); [ -n "$XV" ] || XV=.pixi/envs/default/bin/Xvfb
XRV=$(command -v xvfb-run || true)
if [ -n "$XRV" ]; then
    "$XRV" -a "$CT" --test-dir "$B" -L display --output-on-failure >/dev/null 2>&1 \
        || { echo "CMAKE-SMOKE ✗ xvfb display family (the window examples failed their real run)"; exit 1; }
    DISPLAY_NOTE="xvfb display ✓"
elif [ -x "$XV" ]; then
    "$XV" :78 -screen 0 1024x768x24 >/tmp/cmake-smoke-xvfb.log 2>&1 & XPID=$!
    sleep 1.5
    if kill -0 $XPID 2>/dev/null; then
        DISPLAY=:78 "$CT" --test-dir "$B" -L display --output-on-failure >/dev/null 2>&1
        DRV=$?
        kill $XPID 2>/dev/null; wait $XPID 2>/dev/null
        [ $DRV -eq 0 ] && DISPLAY_NOTE="Xvfb:78 display ✓" || { echo "CMAKE-SMOKE ✗ display family (the Xvfb:78 real run failed)"; exit 1; }
    else
        kill $XPID 2>/dev/null; wait $XPID 2>/dev/null
        DISPLAY_NOTE="display family not executed (Xvfb failed to start = the PIT-19 clobber, not a pass)"
    fi
else
    DISPLAY_NOTE="display family not executed (no Xvfb -- not a pass)"
fi

echo "CMAKE-SMOKE ✓ (full bare chain + four negative-path messages + the bare-PATH triple anchor + ${TREE_NOTE} + ${DISPLAY_NOTE})"
