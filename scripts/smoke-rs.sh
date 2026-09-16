#!/usr/bin/env bash
# smoke-rs.sh —— ctest 转发壳（S3）：argv 唯一户口在 R9 注册表（cmake/VisiaEngineBindings.cmake），
# 本脚本零参数抄写。两道实断：①必选中 ≥1 条（ctest -R 空匹配=exit 0 假绿，二轮评审实锤）
# ②幂等前置=configure+build（增量秒级；ci 链翻转解法=自带前置）。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
STEM="${1:?用法: smoke-rs.sh <E###_茎名>}"
CM=$(command -v cmake || true); [ -n "$CM" ] || CM=.pixi/envs/default/bin/cmake
CT=$(command -v ctest || true);  [ -n "$CT" ]  || CT=.pixi/envs/default/bin/ctest
"$CM" --preset bare >/dev/null || { echo "SMOKE-RS ✗ ${STEM} configure"; exit 1; }
"$CM" --build --preset bare >/dev/null || { echo "SMOKE-RS ✗ ${STEM} build"; exit 1; }
out=$("$CT" --test-dir target/cmake-bare -R "^example_${STEM}\$" --output-on-failure 2>&1); rc=$?
echo "$out" | tail -3
echo "$out" | grep -qE 'out of [1-9]' || { echo "SMOKE-RS ✗ ${STEM} 未被选中（空匹配假绿拦截）"; exit 1; }
[ $rc -eq 0 ] && echo "SMOKE-RS ✓ ${STEM}" || echo "SMOKE-RS ✗ ${STEM} rc=$rc"
exit $rc
