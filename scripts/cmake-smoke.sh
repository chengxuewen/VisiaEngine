#!/usr/bin/env bash
# cmake-smoke（计划 §2.4）：三态门——无 cmake=SKIP exit0（gate-abi 族纪律）；
# 有= bare 配置+构建+ctest，另两条负路径报文断言（SYSTEM 反污染/预构建缺物）。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
CM=$(command -v cmake || true)
[ -n "$CM" ] || CM=.pixi/envs/default/bin/cmake
if [ ! -x "$CM" ]; then
    echo "CMAKE-SMOKE SKIP (无 cmake——先 pixi install；本段在裸克隆机不误红)"
    exit 0
fi
CT="$(dirname "$CM")/ctest"
B=target/cmake-smoke
rm -rf "$B" "$B-neg"

"$CM" -S . -B "$B" -G Ninja -DCMAKE_BUILD_TYPE=Debug -DVISIAENGINE_QT_SDK=OFF >/dev/null \
    || { echo "CMAKE-SMOKE ✗ bare configure"; exit 1; }
"$CM" --build "$B" >/dev/null || { echo "CMAKE-SMOKE ✗ build"; exit 1; }
"$CT" --test-dir "$B" -LE display --output-on-failure >/dev/null 2>&1 \
    || { echo "CMAKE-SMOKE ✗ ctest（探针/headless example 未过）"; exit 1; }
# ↑ -LE display：探针无标签+headless example 真跑；显示族（弹窗）由 smoke-x11/smoke-qt 专职 xvfb 覆盖

# 负路径 1：SYSTEM 语义（激活壳=拒收污染；真系统 cargo=合法通过；其余=必错且报文含 SYSTEM）
out=$("$CM" -S . -B "$B-neg" -G Ninja -DVISIAENGINE_QT_SDK=OFF -DVISIAENGINE_RUST_SDK=SYSTEM 2>&1)
rc=$?
if [ $rc -eq 0 ]; then
    echo "  note: SYSTEM 模式走了真·系统 cargo（合法环境态）"
elif ! grep -q "SYSTEM" <<<"$out"; then
    echo "CMAKE-SMOKE ✗ SYSTEM 负路径报文缺失"; exit 1
fi
# 负路径 2：预构建逃生舱缺物=硬错（环境无关确定性）
out=$("$CM" -S . -B "$B-neg" -G Ninja -DVISIAENGINE_QT_SDK=OFF -DVISIAENGINE_ARTIFACT_PATH=/tmp/ve-nope 2>&1)
[ $? -ne 0 ] && grep -q "无 capi" <<<"$out" \
    || { echo "CMAKE-SMOKE ✗ 逃生舱缺物未硬错（静默消费回潮）"; exit 1; }  # 短语断言：cmake 报文会折行，全句 grep 必漏

echo "CMAKE-SMOKE ✓（bare 全链 + 双负路径报文）"
