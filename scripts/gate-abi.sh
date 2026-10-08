#!/usr/bin/env bash
# gate-abi（I2 / 计划 v1.4 §6；P1 瓦片两口 33→35；R 带栅格两口 44→46）：默认 feature 构建 + nm 白名单==46 + demo 编译运行。
# 工具检索序：PATH → host-spike conda 前缀件（[FFI-R:FC-M1] 裸 nm/cc 无实证）；
# 两者皆缺=SKIP exit0（条件段 shell 门自写，[v13-LEDG/env-W4]——CI/新克隆不误红）。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
NM=$(command -v nm || true); [ -n "$NM" ] || NM=.pixi/envs/host-spike/bin/x86_64-conda-linux-gnu-nm
CC=$(command -v cc || true);   [ -n "$CC" ] || CC=.pixi/envs/host-spike/bin/x86_64-conda-linux-gnu-cc
RE=$(command -v readelf || true); [ -n "$RE" ] || RE=.pixi/envs/host-spike/bin/x86_64-conda-linux-gnu-readelf
if [ ! -x "$NM" ] || [ ! -x "$CC" ] || [ ! -x "$RE" ]; then echo "GATE-ABI SKIP (无 nm/cc/readelf，host-spike 未装)"; exit 0; fi
cargo build -p visiaengine-capi >/dev/null 2>&1 || { echo "GATE-ABI ✗ build"; exit 1; }
SO=target/debug/libvisiaengine.so
[ -f "$SO" ] || { echo "GATE-ABI ✗ 缺 $SO（[lib] name 检查）"; exit 1; }
# P1（packaging-round v1.3）：cdylib 必须自带名牌（SONAME）。无名牌时消费者记的是
# 「链接命令行上那串字」——本机实测两形：给绝对路径 → DT_NEEDED=/abs/….so；有名牌 → 裸名。
# want 现场从 [lib] name 推导：名牌与产物名同源，将来改库名漏改此处=本检红。
libname=$(grep -A2 '^\[lib\]' bindings/c/visiaengine-capi/Cargo.toml | sed -n 's/^name = "\([a-z_]*\)".*/\1/p' | head -1)
want="lib${libname}.so"
have=$("$RE" -d "$SO" | sed -n 's/.*SONAME.*\[\(.*\)\].*/\1/p')
[ -n "$want" ] || { echo "GATE-ABI ✗ [lib] name 读取失败（want 空=断言恒真，不可信）"; exit 1; }
[ "$have" = "$want" ] || { echo "GATE-ABI ✗ SONAME: got='${have:-无}' want='$want'（名牌缺位→消费者 DT_NEEDED 随命令行漂移）"; exit 1; }
N=$("$NM" -D "$SO" | grep -c ' T visiaengine_' || true)
echo "ABI-SYMBOLS="$N/49" | SO_SIZE=$(du -h "$SO" | cut -f1)"
[ "$N" = "49" ] || { echo "GATE-ABI ✗ 符号数 $N"; exit 1; }
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
[ -z "$MISSING" ] || { echo "GATE-ABI ✗ hpp 缺转发: $MISSING"; exit 1; }
"$CC" -I bindings/c/visiaengine-capi/include examples/c/E701_demo_headless.c \
      -L target/debug -lvisiaengine -o target/demo_headless || { echo "GATE-ABI ✗ demo 编译"; exit 1; }
LD_LIBRARY_PATH=$PWD/target/debug ./target/demo_headless resources/data/twoprim.glb \
      || { echo "GATE-ABI ✗ demo 运行"; exit 1; }
# M3 出口判据 4：纹理件链路（texquad=io-gltf builder 真源生成，GLTF-11 fixture）
LD_LIBRARY_PATH=$PWD/target/debug ./target/demo_headless resources/data/texquad.glb \
      || { echo "GATE-ABI ✗ demo 纹理件运行"; exit 1; }
echo "GATE-ABI ✓"
