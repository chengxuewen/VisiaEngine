#!/usr/bin/env bash
# gate-abi（I2 / 计划 v1.4 §6）：默认 feature 构建 + nm 白名单==14 + demo 编译运行。
# 工具检索序：PATH → host-spike conda 前缀件（[FFI-R:FC-M1] 裸 nm/cc 无实证）；
# 两者皆缺=SKIP exit0（条件段 shell 门自写，[v13-LEDG/env-W4]——CI/新克隆不误红）。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
NM=$(command -v nm || true); [ -n "$NM" ] || NM=.pixi/envs/host-spike/bin/x86_64-conda-linux-gnu-nm
CC=$(command -v cc || true);   [ -n "$CC" ] || CC=.pixi/envs/host-spike/bin/x86_64-conda-linux-gnu-cc
if [ ! -x "$NM" ] || [ ! -x "$CC" ]; then echo "GATE-ABI SKIP (无 nm/cc，host-spike 未装)"; exit 0; fi
cargo build -p visiaengine-capi >/dev/null 2>&1 || { echo "GATE-ABI ✗ build"; exit 1; }
SO=target/debug/libvisiaengine.so
[ -f "$SO" ] || { echo "GATE-ABI ✗ 缺 $SO（[lib] name 检查）"; exit 1; }
N=$("$NM" -D "$SO" | grep -c ' T visiaengine_' || true)
echo "ABI-SYMBOLS=$N/14 | SO_SIZE=$(du -h "$SO" | cut -f1)"
[ "$N" = "14" ] || { echo "GATE-ABI ✗ 符号数 $N"; exit 1; }
"$CC" -I crates/visiaengine-capi/include crates/visiaengine-capi/examples/demo_headless.c \
      -L target/debug -lvisiaengine -o target/demo_headless || { echo "GATE-ABI ✗ demo 编译"; exit 1; }
LD_LIBRARY_PATH=$PWD/target/debug ./target/demo_headless resources/data/twoprim.glb \
      || { echo "GATE-ABI ✗ demo 运行"; exit 1; }
# M3 出口判据 4：纹理件链路（texquad=io-gltf builder 真源生成，GLTF-11 fixture）
LD_LIBRARY_PATH=$PWD/target/debug ./target/demo_headless resources/data/texquad.glb \
      || { echo "GATE-ABI ✗ demo 纹理件运行"; exit 1; }
echo "GATE-ABI ✓"
