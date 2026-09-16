#!/usr/bin/env bash
# gate-docs（批 5 T3 / [E3D:D1·D3]）：文档零漂移四检——
#  ① E 三方一致：example 文件名 = 头注释 E### = docs/tutorials.md（预留空号=仅索引行）；
#  ② README 条款数 == spec-trace 实报数（文档只写可校验的数）；
#  ③ visiaengine.h 原型名集 == Rust extern "C" fn 名集（符号数由 gate-abi 的 nm 守）；
#  ④ docs/README 相对 .md 链接存活。
# 纯 grep/diff <1s；失配=红+exit 1。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
fail=0
EXDIRS=(examples/rs crates/visiaengine-capi/examples)   # S1: rs 入表，两个已搬空旧目录退场；capi 条 S2 退

# ① E 三方
files=$(for d in "${EXDIRS[@]}"; do ls "$d" 2>/dev/null; done | grep -oE '^E[0-9]{3}' | sort -u)
headers=$(for d in "${EXDIRS[@]}"; do for x in "$d"/*.rs "$d"/*.c; do [ -f "$x" ] && head -1 "$x" | grep -oE '^(/\*+|//+) ?!? ?E[0-9]{3}' | grep -oE 'E[0-9]{3}'; done; done 2>/dev/null | sort -u)
real_files=$(for d in "${EXDIRS[@]}"; do ls "$d" 2>/dev/null; done | grep -E '^E[0-9]{3}_' | sort -u)
idx_files=$(grep -oE 'E[0-9]{3}_[A-Za-z0-9_]+\.(rs|c)' docs/tutorials.md | sort -u)
index=$(grep -oE '^\| E[0-9]{3}' docs/tutorials.md | grep -oE 'E[0-9]{3}' | sort -u)
[ "$files" = "$headers" ] || { echo "GATE-DOCS ✗ ①文件名集≠头注释集:"; diff <(echo "$files") <(echo "$headers"); fail=1; }
[ "$idx_files" = "$real_files" ] || { echo "GATE-DOCS ✗ ①索引路径集≠磁盘实况:"; diff <(echo "$idx_files") <(echo "$real_files"); fail=1; }
for e in $index; do
    grep -q "^${e}" <<< "$files" || [ "$e" = "E402" ] \
        || { echo "GATE-DOCS ✗ ①索引 $e 无磁盘件且非预留空号"; fail=1; }
done

# ② README 条款数
rn=$(grep -oE '\*\*[0-9]+ 条\*\*' README.md | grep -oE '[0-9]+' | head -1 || true)
sn=$(bash scripts/spec-trace.sh 2>/dev/null | grep -oE '[0-9]+ 条双向' | grep -oE '^[0-9]+' || true)
[ "${rn:-x}" = "${sn:-y}" ] || { echo "GATE-DOCS ✗ ②README '${rn:-缺}' ≠ spec-trace '${sn:-缺}'"; fail=1; }

# ③ 头签名名集
hs=$(grep -oE 'visiaengine_[a-z0-9_]+\(' crates/visiaengine-capi/include/visiaengine.h | sed 's/(//' | sort -u)
rs=$(grep -rhoE 'fn visiaengine_[a-z0-9_]+' crates/visiaengine-capi/src/ | sed 's/^fn //' | sort -u)
[ "$hs" = "$rs" ] || { echo "GATE-DOCS ✗ ③.h 原型集 ≠ Rust extern 集:"; diff <(echo "$hs") <(echo "$rs"); fail=1; }

# ④ 相对链接
while IFS= read -r f; do
    dir=$(dirname "$f")
    while IFS= read -r link; do
        case "$link" in http*) continue ;; esac
        [ -f "$dir/$link" ] || { echo "GATE-DOCS ✗ ④断链: $f → $link"; fail=1; }
    done < <(grep -oE '\]\(([^)#?]+\.md)' "$f" 2>/dev/null | sed 's/^](//')
done < <(ls README.md docs/*.md docs/sdd/*.md docs/reference/*.md 2>/dev/null)

# ⑤ 门面纯度（CMake 层纪律：根文件 ≤60 行且禁载编译规则——权威=cargo/pixi）
if [ -f CMakeLists.txt ]; then
    rl=$(wc -l < CMakeLists.txt)
    [ "$rl" -le 60 ] || { echo "GATE-DOCS ✗ ⑤根 CMakeLists ${rl} 行超 60（门面纪律）"; fail=1; }
    if grep -qE 'add_executable|add_library\(' CMakeLists.txt; then
        echo "GATE-DOCS ✗ ⑤根门面含编译规则字样（越权 cargo 权威）"; fail=1
    fi
fi

if [ "$fail" = 0 ]; then
    echo "GATE-DOCS ✓（E $(echo "$files" | wc -l) 件三方 / README=${rn}↔spec=${sn} / 头签名 $(echo "$hs" | wc -l) 名 / 链接存活）"
fi
exit $fail
