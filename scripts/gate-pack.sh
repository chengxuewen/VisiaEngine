#!/usr/bin/env bash
# gate-pack (packaging-round v1.3 P3): the installed tree must be SELF-SUFFICIENT
# and carry no build-machine paths. Deliberately does NOT run patchelf: cleaning
# the artifact's RUNPATH belongs to the assembly step of the publish band (a
# tarball/wheel), and putting it in install(CODE) would force every installer to
# have patchelf. Here the RUNPATH is reported, never judged.
#
# Tool discipline (same as gate-abi/cmake-smoke): missing cmake/pkg-config ->
# SKIP exit 0, so a bare clone never goes red for an absent optional tool.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
CM=$(command -v cmake || true); [ -n "$CM" ] || CM=.pixi/envs/default/bin/cmake
RE=$(command -v readelf || true); [ -n "$RE" ] || RE=.pixi/envs/default/bin/readelf
if [ ! -x "$CM" ] || [ ! -x "$RE" ]; then echo "GATE-PACK SKIP (no cmake/readelf)"; exit 0; fi

P=${1:-target/gate-pack-prefix}
B=${2:-target/cmake-smoke}
fail=0
say() { echo "$1"; fail=1; }

# Build + install into a throwaway prefix (never touches a user prefix).
[ -d "$B" ] || { echo "GATE-PACK SKIP (no $B — run cmake-smoke or pixi run cmake --preset default first)"; exit 0; }
rm -rf "$P"
"$CM" --install "$B" --prefix "$P" >/dev/null 2>&1 \
  || { say "GATE-PACK ✗ install failed"; exit 1; }

SO="$P/lib/$(case "$(uname -s)" in Linux) echo -n libvisiaengine.so ;; Darwin) echo -n libvisiaengine.dylib ;; *) echo -n visiaengine.dll ;; esac)"
[ -f "$SO" ] || { say "GATE-PACK ✗ installed tree missing the library $SO"; exit 1; }

# --- 1) Manifest equality: everything a consumer needs is in the tree -------
NEED="lib/cmake/visiaengine/visiaengineConfig.cmake
lib/cmake/visiaengine/visiaengineConfigVersion.cmake
lib/pkgconfig/visiaengine.pc
include/visiaengine.h
include/visiaengine/visiaengine.hpp
include/visiaengine_widget.hpp
share/licenses/visiaengine/LICENSE-MIT
share/licenses/visiaengine/LICENSE-APACHE"
for f in $NEED; do
  [ -f "$P/$f" ] || say "GATE-PACK ✗ installed tree missing item: $f"
done
[ $fail -eq 0 ] || exit 1
echo "GATE-PACK ①manifest: $(find "$P" -type f | wc -l) items present"

# --- 2) Self-sufficiency: every DT_NEEDED must be a system library ----------
# A shipped SDK that pulls non-system shared libs needs them bundled too; ours
# does not (wgpu loads Vulkan via dlopen, so no libvulkan in NEEDED).
# The whitelist is inline in the case below (a second regex copy = a second truth).
BAD=""
NEEDED=$("$RE" -d "$SO" | sed -n 's/.*NEEDED.*\[\(.*\)\].*/\1/p')
for n in $NEEDED; do
  case "$n" in
    libdl.so*|libpthread.so*|libm.so*|librt.so*|libgcc_s.so*|libc.so*|ld-linux*|libstdc++.so*|libSystem*) ;;
    *) BAD="$BAD $n" ;;
  esac
done
if [ -n "$BAD" ]; then
  say "GATE-PACK ✗ NEEDED contains non-system libraries (bundle them, or extend the whitelist with a stated reason):$BAD"
else
  echo "GATE-PACK ②self-sufficient: all NEEDED inside the system-library whitelist ($(echo "$NEEDED" | wc -l) entries)"
fi

# --- 3) No build-machine paths in the shipped description files -------------
# Positive control performed 2026-10-08: a planted build-dir string turns this
# red; the predicate is not vacuous.
BUILDHINT="$PWD/target"
for f in lib/cmake/visiaengine/visiaengineConfig.cmake lib/pkgconfig/visiaengine.pc; do
  if grep -q "$BUILDHINT" "$P/$f"; then
    say "GATE-PACK ✗ $f contains an absolute build-directory path (publish-artifact leak)"
  fi
done
grep -q '^prefix=${pcfiledir}/' "$P/lib/pkgconfig/visiaengine.pc" \
  || say "GATE-PACK ✗ .pc prefix is not the pcfiledir form (tree dies on relocation)"
[ $fail -eq 0 ] && echo "GATE-PACK ③no leaks: config/.pc carry none of this repo's build paths"

# --- Report only (see header): RUNPATH is harmless at install time ----------
RP=$("$RE" -d "$SO" | sed -n 's/.*RUNPATH.*\[\(.*\)\].*/\1/p')
echo "GATE-PACK note: in-library RUNPATH=${RP:-none} (the publish band's assembly step owns the cleaning; this band does not judge)"

[ $fail -eq 0 ] && echo "GATE-PACK ✓"
exit $fail
