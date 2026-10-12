#!/usr/bin/env bash
# smoke-x11 (I3): Xvfb self-managed (host-spike lookup) + demo_x11 three-frame present, exit 0.
# Real present-pixel correctness = the Tier-C manual checklist (this script only anchors the process exit code + the OK line, [v13-FEAS-1/R5]).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
XVFB=$(command -v Xvfb || true); [ -n "$XVFB" ] || XVFB=.pixi/envs/host-spike/bin/Xvfb
CC=$(command -v cc || true);     [ -n "$CC" ]  || CC=.pixi/envs/host-spike/bin/x86_64-conda-linux-gnu-cc
XMULTI=$(command -v Xvfb || true)
if [ ! -x "$XVFB" ] || [ ! -x "$CC" ]; then echo "SMOKE-X11 SKIP (Xvfb/cc unavailable)"; exit 0; fi
cargo build -p visiaengine-capi >/dev/null 2>&1 || { echo "SMOKE-X11 ✗ build"; exit 1; }
"$CC" -I bindings/c/visiaengine-capi/include examples/c/E702_demo_x11.c \
      -I .pixi/envs/host-spike/include -L target/debug -L .pixi/envs/host-spike/lib \
      -lvisiaengine -lX11 -o target/demo_x11 || { echo "SMOKE-X11 ✗ compile"; exit 1; }
if [ -n "${DISPLAY:-}" ]; then
  DISP=$DISPLAY; XPID=""   # the host already has X (a shared dev machine's :0 etc.); do not start Xvfb ourselves
else
  DISP=:77
  "$XVFB" $DISP -screen 0 1024x768x24 >/tmp/xvfb-x11.log 2>&1 &
  XPID=$!
  sleep 1.5
  if ! kill -0 $XPID 2>/dev/null; then echo "SMOKE-X11 SKIP (Xvfb failed to start)"; cat /tmp/xvfb-x11.log; exit 0; fi
fi
DISPLAY=$DISP LD_LIBRARY_PATH=$PWD/target/debug timeout -k 5 60 ./target/demo_x11 resources/data/twoprim.glb --frames 3
RC=$?
[ -n "$XPID" ] && { kill $XPID 2>/dev/null; wait $XPID 2>/dev/null; true; }
[ $RC -eq 0 ] && echo "SMOKE-X11 ✓" || echo "SMOKE-X11 ✗ rc=$RC"
exit $RC
