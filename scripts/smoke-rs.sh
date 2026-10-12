#!/usr/bin/env bash
# smoke-rs.sh -- a ctest forwarding shell (S3): argv has its single home in the R9 registry (cmake/VisiaEngineBindings.cmake);
# this script transcribes zero args. Two real asserts: ① at least one test must be selected (an empty ctest -R match exits 0 = a false green, measured in the second review round)
# ② an idempotent pre-step = configure+build (incremental, seconds; the ci-chain flip-flop fix = carrying its own pre-step).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
STEM="${1:?usage: smoke-rs.sh <E###_stem>}"
CM=$(command -v cmake || true); [ -n "$CM" ] || CM=.pixi/envs/default/bin/cmake
CT=$(command -v ctest || true);  [ -n "$CT" ]  || CT=.pixi/envs/default/bin/ctest
"$CM" --preset default >/dev/null || { echo "SMOKE-RS ✗ ${STEM} configure"; exit 1; }
"$CM" --build --preset default >/dev/null || { echo "SMOKE-RS ✗ ${STEM} build"; exit 1; }
out=$("$CT" --test-dir cmake-build -R "^example_${STEM}\$" --output-on-failure 2>&1); rc=$?
echo "$out" | tail -3
echo "$out" | grep -qE 'out of [1-9]' || { echo "SMOKE-RS ✗ ${STEM} was not selected (empty-match false-green interception)"; exit 1; }
[ $rc -eq 0 ] && echo "SMOKE-RS ✓ ${STEM}" || echo "SMOKE-RS ✗ ${STEM} rc=$rc"
exit $rc
