#!/usr/bin/env bash
# gate-docs（批 5 T3 / [E3D:D1·D3]；B5 扩六检）——文档零漂移：
#  ① E 三方一致：example 文件名 = 头注释 E### = docs/tutorials.md（预留空号=仅索引行）；
#  ② README 条款数 == spec-trace 实报数（文档只写可校验的数）；
#  ③ visiaengine.h 原型名集 == Rust extern "C" fn 名集（符号数由 gate-abi 的 nm 守）；
#  ④ relative links in README/docs/llms.txt resolve (http/mailto/anchor/build exempt).
# 纯 grep/diff <1s；失配=红+exit 1。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
fail=0
EXDIRS=(examples/rs examples/c examples/cpp examples/qt)   # S1: rs 入表，两个已搬空旧目录退场；capi 条 S2 退

# ① E 三方
files=$(for d in "${EXDIRS[@]}"; do ls "$d" 2>/dev/null; done | grep -oE '^E[0-9]{3}' | sort -u)
headers=$(for d in "${EXDIRS[@]}"; do for x in "$d"/*.rs "$d"/*.c "$d"/*.cpp; do [ -f "$x" ] && head -1 "$x" | grep -oE '^(/\*+|//+) ?!? ?E[0-9]{3}' | grep -oE 'E[0-9]{3}'; done; done 2>/dev/null | sort -u)
real_files=$(for d in "${EXDIRS[@]}"; do ls "$d" 2>/dev/null; done | grep -E '^E[0-9]{3}_' | sort -u)
idx_files=$(grep -oE 'E[0-9]{3}_[A-Za-z0-9_]+\.(rs|cpp|c)' docs/tutorials.md | sort -u)
index=$(grep -oE '^\| E[0-9]{3}' docs/tutorials.md | grep -oE 'E[0-9]{3}' | sort -u)
[ "$files" = "$headers" ] || { echo "GATE-DOCS ✗ ①文件名集≠头注释集:"; diff <(echo "$files") <(echo "$headers"); fail=1; }
[ "$idx_files" = "$real_files" ] || { echo "GATE-DOCS ✗ ①索引路径集≠磁盘实况:"; diff <(echo "$idx_files") <(echo "$real_files"); fail=1; }
for e in $index; do
    grep -q "^${e}" <<< "$files" \
        || { echo "GATE-DOCS ✗ ①索引 $e 无磁盘件"; fail=1; }
done

# ② README 条款数
rn=$(grep -oE '\*\*[0-9]+ 条\*\*' README.md | grep -oE '[0-9]+' | head -1 || true)
sn=$(bash scripts/spec-trace.sh 2>/dev/null | grep -oE '[0-9]+ 条双向' | grep -oE '^[0-9]+' || true)
[ "${rn:-x}" = "${sn:-y}" ] || { echo "GATE-DOCS ✗ ②README '${rn:-缺}' ≠ spec-trace '${sn:-缺}'"; fail=1; }

# ③ 头签名名集
hs=$(grep -oE 'visiaengine_[a-z0-9_]+\(' bindings/c/visiaengine-capi/include/visiaengine.h | sed 's/(//' | sort -u)
rs=$(grep -rhoE 'fn visiaengine_[a-z0-9_]+' bindings/c/visiaengine-capi/src/ | sed 's/^fn //' | sort -u)
[ "$hs" = "$rs" ] || { echo "GATE-DOCS ✗ ③.h 原型集 ≠ Rust extern 集:"; diff <(echo "$hs") <(echo "$rs"); fail=1; }

# ④ relative links — extended this band from .md-only to every markdown link
# destination in README.md + docs/**/*.md + llms.txt. Skips: http(s)/mailto, pure-#
# anchors, destinations containing spaces (prose artifacts, not valid targets), and
# build//target/ paths (generated trees absent in a bare clone — the gallery/
# docs-gen bands own those outputs).
while IFS= read -r f; do
    dir=$(dirname "$f")
    while IFS= read -r dest; do
        case "$dest" in
            http*|mailto:*|\#*|*\ *|build/*|target/*) continue ;;
        esac
        target=${dest%%#*}
        target=${target%%\?*}
        [ -n "$target" ] || continue
        [ -e "$dir/$target" ] || { echo "GATE-DOCS ✗ ④断链: $f → $dest"; fail=1; }
    done < <(grep -oE '\]\([^)]+\)' "$f" 2>/dev/null | sed -E 's/^\]\(//; s/\)$//')
done < <(ls README.md docs/*.md docs/sdd/*.md docs/reference/*.md llms.txt 2>/dev/null)

# ⑤ 门面纯度（CMake 层纪律：根文件 ≤60 行且禁载编译规则——权威=cargo/pixi）
if [ -f CMakeLists.txt ]; then
    rl=$(wc -l < CMakeLists.txt)
    [ "$rl" -le 60 ] || { echo "GATE-DOCS ✗ ⑤根 CMakeLists ${rl} 行超 60（门面纪律）"; fail=1; }
    if grep -qE 'add_executable|add_library\(' CMakeLists.txt; then
        echo "GATE-DOCS ✗ ⑤根门面含编译规则字样（越权 cargo 权威）"; fail=1
    fi
fi

# ⑧ workspace package count == README (ungated numbers drift: clause count needed a lock,
# then ctest/gallery counts, now this). Value comes from cargo metadata, never hand-typed.
pk=$(cargo metadata --no-deps --format-version 1 2>/dev/null | \
     python3 -c 'import json,sys;print(len(json.load(sys.stdin)["packages"]))' 2>/dev/null || true)
rpkg=$(grep -oE 'workspace 共 [0-9]+ 个包' README.md | grep -oE '[0-9]+' | head -1 || true)
if [ -z "${pk:-}" ]; then
    echo "GATE-DOCS ✗ ⑧cargo metadata 读取失败（断言无对象=不可信）"; fail=1
elif [ "${rpkg:-x}" != "$pk" ]; then
    echo "GATE-DOCS ✗ ⑧README 包数 '${rpkg:-缺}' ≠ cargo '$pk'"; fail=1
fi

# ⑨ pixel gates must not hard-code the clear colour: read it out of the frame instead.
# Measured lesson (PIT-45): a one-triple-wrong literal made a coverage assertion count the
# whole frame — constant-true for an entire band, and it passed CI the whole time.
bad=$(grep -rnE 'let clear: \[u8; 3\] = \[[0-9]' crates examples --include='*.rs' 2>/dev/null || true)
if [ -n "$bad" ]; then echo "GATE-DOCS ✗ ⑨手写清屏色字面量（改自标定 + 频率护栏）:"; echo "$bad"; fail=1; fi

# ⑥ gallery manifest lock (B5 "one artifact, four consumers": source+prose+test+card —
# registries are the single accounting point; gallery is DERIVED, never hand-edited)
# manifest 由 `pixi run gallery` 产出（build/ 内，gitignored）；缺席=红（提醒先跑生成器）。
GM=build/gallery/manifest.txt
if [ ! -f "$GM" ]; then
    echo "GATE-DOCS ✗ ⑥画廊 manifest 缺席（先跑 pixi run gallery）"; fail=1
else
    gm_names=$(grep -oE '^E[0-9]{3}' "$GM" | sort -u)
    reg_names=$(for d in examples/rs examples/c examples/cpp examples/qt; do ls "$d" 2>/dev/null; done | grep -oE '^E[0-9]{3}' | sort -u)
    [ "$gm_names" = "$reg_names" ] || { echo "GATE-DOCS ✗ ⑥画廊 manifest ≠ 磁盘注册集:"; diff <(echo "$reg_names") <(echo "$gm_names"); fail=1; }
fi

# ⑦ README 画廊卡数 == manifest 实数（2026-10-08：40≠43 漂移无人管，随装树带补锁；
# 条数那一半住在 cmake-smoke（它才持有 ctest 真值），各锁住在已有数据的工具里。
if [ -f "$GM" ]; then
    mc=$(grep -c '^E' "$GM")
    rg=$(grep -oE '例子画廊（[0-9]+ 卡' README.md | grep -oE '[0-9]+' | head -1 || true)
    [ "${rg:-x}" = "$mc" ] || { echo "GATE-DOCS ✗ ⑦README 画廊 '${rg:-缺}' ≠ manifest '$mc'"; fail=1; }
fi

if [ "$fail" = 0 ]; then
    echo "GATE-DOCS ✓（E $(echo "$files" | wc -l) 件三方 / README=${rn}↔spec=${sn} / 头签名 $(echo "$hs" | wc -l) 名 / 链接存活 / 画廊 $(grep -c '^E[0-9]' "$GM" 2>/dev/null || echo 0) 卡 / 包 ${pk}）"
fi
exit $fail
