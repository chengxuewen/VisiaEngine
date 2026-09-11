#!/usr/bin/env bash
# smoke-x11（I3）：Xvfb 自管（host-spike 检索）+ demo_x11 三帧 present exit0。
# 真实 present 像素正确性=Tier-C 人检清单（本脚本只锚进程退出码+OK 行，[v13-FEAS-1/R5]）。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
XVFB=$(command -v Xvfb || true); [ -n "$XVFB" ] || XVFB=.pixi/envs/host-spike/bin/Xvfb
CC=$(command -v cc || true);     [ -n "$CC" ]  || CC=.pixi/envs/host-spike/bin/x86_64-conda-linux-gnu-cc
XMULTI=$(command -v Xvfb || true)
if [ ! -x "$XVFB" ] || [ ! -x "$CC" ]; then echo "SMOKE-X11 SKIP (Xvfb/cc 不可用)"; exit 0; fi
cargo build -p visiaengine-capi >/dev/null 2>&1 || { echo "SMOKE-X11 ✗ build"; exit 1; }
"$CC" -I crates/visiaengine-capi/include crates/visiaengine-capi/examples/demo_x11.c \
      -I .pixi/envs/host-spike/include -L target/debug -L .pixi/envs/host-spike/lib \
      -lvisiaengine -lX11 -o target/demo_x11 || { echo "SMOKE-X11 ✗ 编译"; exit 1; }
if [ -n "${DISPLAY:-}" ]; then
  DISP=$DISPLAY; XPID=""   # 宿主已有 X（共享开发机 :0 等），不自启 Xvfb
else
  DISP=:77
  "$XVFB" $DISP -screen 0 1024x768x24 >/tmp/xvfb-x11.log 2>&1 &
  XPID=$!
  sleep 1.5
  if ! kill -0 $XPID 2>/dev/null; then echo "SMOKE-X11 SKIP (Xvfb 起不来)"; cat /tmp/xvfb-x11.log; exit 0; fi
fi
DISPLAY=$DISP LD_LIBRARY_PATH=$PWD/target/debug timeout -k 5 60 ./target/demo_x11 resources/data/twoprim.glb
RC=$?
[ -n "$XPID" ] && { kill $XPID 2>/dev/null; wait $XPID 2>/dev/null; true; }
[ $RC -eq 0 ] && echo "SMOKE-X11 ✓" || echo "SMOKE-X11 ✗ rc=$RC"
exit $RC
