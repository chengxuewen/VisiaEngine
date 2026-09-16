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
rm -rf "$B" "$B-neg" "$B-negconf"

"$CM" -S . -B "$B" -G Ninja -DCMAKE_BUILD_TYPE=Debug -DVISIAENGINE_QT_SDK=OFF >/dev/null \
    || { echo "CMAKE-SMOKE ✗ bare configure"; exit 1; }
"$CM" --build "$B" >/dev/null || { echo "CMAKE-SMOKE ✗ build"; exit 1; }
"$CT" --test-dir "$B" -LE display --output-on-failure >/dev/null 2>&1 \
    || { echo "CMAKE-SMOKE ✗ ctest（探针/headless example 未过）"; exit 1; }
# ↑ -LE display：探针无标签+headless example 真跑；显示族（弹窗）由 smoke-x11/smoke-qt 专职 xvfb 覆盖

# 第四态：裸 PATH 三锚（IDE 直调形态守卫；B1 裁决：configure 探+双构建锚，Rust 例族域换形后不加回退）
# （2026-09-15 实锤：pixi run 包装掩盖，用户 IDE /usr/bin/cmake 直调爆「could not execute rustc」）
SYS_CM=$(command -v /usr/bin/cmake || true); [ -n "$SYS_CM" ] || SYS_CM="$CM"  # 探针=用户 IDE 真形（系统 cmake；无系统 cmake 机退 conda 件）
# 编译器绝对路径钉死=探针聚焦 cargo 自足面（本机实况：系统无 g++，C++ 全在 conda 侧；IDE 真形另由态①覆盖）
env PATH=/usr/bin:/bin "$SYS_CM" -S . -B "$B-negconf" -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Debug -DVISIAENGINE_QT_SDK=OFF \
    -DCMAKE_C_COMPILER=/usr/bin/gcc "-DCMAKE_CXX_COMPILER=$(dirname "$CM")/x86_64-conda-linux-gnu-c++" \
    >/dev/null 2>&1 \
    || { echo "CMAKE-SMOKE ✗ 裸 PATH configure（SDK/cargo 自足性破坏——.pixi 兜底断）"; exit 1; }
rm -rf "$B-negconf"
for _t in cargo-build_capi cargo-build_examples; do
  env PATH=/usr/bin:/bin "$CM" --build "$B" --target "$_t" >/dev/null 2>&1 \
      || { echo "CMAKE-SMOKE ✗ 裸 PATH ${_t}（cargo 环境自足性破坏=IDE 必炸）"; exit 1; }
done

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

# 卫生锁：FOLDER 词汇表=磁盘单源（examples/{rs,c,cpp,qt} | visiaengine）——7x-宿主 漏网案的机器化堵截
BAD=$(grep -rnE 'FOLDER "examples/' cmake examples bindings 2>/dev/null | grep -vE '"examples/(rs|c|cpp|qt)"' | grep -vF '${' || true)  # 模板行（${_lang} 动态推导）放行
[ -z "$BAD" ] || { echo "CMAKE-SMOKE ✗ FOLDER 词汇越表（带号/方言禁止，见 D14/R9）:"; echo "$BAD"; exit 1; }

# 负路径 3：install 门面必 fail-loud（S-c β；静默 exit-0 说谎回潮=红。短语断言，报文折行教训同款）
"$CM" --install "$B" --prefix "$B-inst" >"$B-inst.out" 2>&1; i_rc=$?
rm -rf "$B-inst" "$B-inst.out"
[ $i_rc -ne 0 ] || { echo "CMAKE-SMOKE ✗ install 静默说谎回潮（应非零）"; exit 1; }

# display 子态（B2）：Xvfb 自启真跑窗口族（E702/E801 自动执行轨；apt CI 同款形）
# 起不来=XKB/pixi clobber 坑（PIT-19）→ note 不假绿；xvfb-run 系 Debian 包裹，conda 无=直启 Xvfb
XV=$(command -v Xvfb || true); [ -n "$XV" ] || XV=.pixi/envs/default/bin/Xvfb
XRV=$(command -v xvfb-run || true)
if [ -n "$XRV" ]; then
    "$XRV" -a "$CT" --test-dir "$B" -L display --output-on-failure >/dev/null 2>&1 \
        || { echo "CMAKE-SMOKE ✗ xvfb display 族（窗口例真跑失败）"; exit 1; }
    DISPLAY_NOTE="xvfb display ✓"
elif [ -x "$XV" ]; then
    "$XV" :78 -screen 0 1024x768x24 >/tmp/cmake-smoke-xvfb.log 2>&1 & XPID=$!
    sleep 1.5
    if kill -0 $XPID 2>/dev/null; then
        DISPLAY=:78 "$CT" --test-dir "$B" -L display --output-on-failure >/dev/null 2>&1
        DRV=$?
        kill $XPID 2>/dev/null; wait $XPID 2>/dev/null
        [ $DRV -eq 0 ] && DISPLAY_NOTE="Xvfb:78 display ✓" || { echo "CMAKE-SMOKE ✗ display 族（Xvfb:78 真跑失败）"; exit 1; }
    else
        kill $XPID 2>/dev/null; wait $XPID 2>/dev/null
        DISPLAY_NOTE="display 族未执行（Xvfb 起不来=PIT-19 clobber，非通过）"
    fi
else
    DISPLAY_NOTE="display 族未执行（无 Xvfb——非通过）"
fi

echo "CMAKE-SMOKE ✓（bare 全链 + 三负路径报文 + 裸 PATH 三锚 + ${DISPLAY_NOTE}）"
