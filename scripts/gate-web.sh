#!/usr/bin/env bash
# gate-web — headless page gate for the wasm example wall (gate server :8018).
# Asserts: served wall renders every wall.json entry, a live page boots
# the engine without a fail mark, and its rAF heartbeat turns (dual channel:
# server-side beacon log + data-frame snapshot).
# SKIP contract: missing chromium / build tree / wall.json => SKIP rc=0.
# Serving: the cmake serve button (:8017) replays web-demo staging exactly as
# before; the gate OWNS a stdlib-python server on VE_GATE_PORT (8018) over the
# assembled dir. E101's anchor <img> is held by this server until the
# post-await beacon (VE_GATE_FRAMES frames turned) arrives: the browser then
# survives to see it, load fires after boot, and the dump lands on the truth.
set -u
CHROME="${VE_CHROMIUM:-}"
[ -x "$CHROME" ] || CHROME="$HOME/.pixi/global/bin/chromium"
[ -x "$CHROME" ] || CHROME="$(ls "$HOME"/.cache/ms-playwright/chromium-*/chrome-linux/chrome 2>/dev/null | head -1)"
TREE="${VISIAENGINE_BUILD_DIR:-cmake-build-debug}"
WALL=examples/wasm/wall.json
PORT="${VE_GATE_PORT:-8018}"
GATE_DIR="$TREE/web-demo"
BEACON=/tmp/opencode/ve-gate-beacon.log
SRVLOG=/tmp/opencode/ve-gate-srv.log
GPU_FLAGS="--no-sandbox --enable-unsafe-webgpu --use-angle=swiftshader --enable-unsafe-swiftshader --enable-features=Vulkan --use-vulkan=swiftshader"  # 2026-10-16 spike measured: the Vulkan pair is REQUIRED (Chrome 131 ships WebGPU off on Linux without it); --no-sandbox required in this container
VE_GATE_MIN_FRAMES="${VE_GATE_MIN_FRAMES:-8}"  # COUPLING: matches VE_GATE_FRAMES in examples/wasm/common.js veLoop — change the two together
VE_GATE_BEACON_BUDGET_MS="${VE_GATE_BEACON_BUDGET_MS:-20000}"  # beacon arrival budget; floor 20s per ruling; set >= measured*3 (see run log / report)
# D-Bus env scrub (2026-10-16 live catch): the system proxy config delivered
# over the session bus makes chromium's network service open loopback sockets
# then send nothing (every http page hangs, file:// unaffected). Removing
# DBUS_SESSION_BUS_ADDRESS from the env is the measured fix; --no-proxy-server
# and --proxy-bypass-list are NOT sufficient.
CHROME_ENV=(env -u DBUS_SESSION_BUS_ADDRESS)

skip() { echo "GATE-WEB SKIP: $1"; exit 0; }
fail() { echo "GATE-WEB FAIL: $1"; cleanup; exit 1; }
[ -x "$CHROME" ] || skip "no chromium binary"
[ -d "$TREE" ]   || skip "no build tree $TREE"
[ -f "$WALL" ]   || skip "no wall.json"

cleanup() {
  [ -n "${SERVE_PID:-}" ] && kill -- -"$SERVE_PID" 2>/dev/null || true
  [ -n "${GATE_PID:-}" ]  && kill -- -"$GATE_PID"  2>/dev/null || true
}  # process groups: serve and gate server are group leaders via setsid
trap cleanup EXIT

N=$(python3 -c "import json;print(len(json.load(open('$WALL'))['entries']))") || fail "wall.json unreadable"
rm -f "$BEACON" "$SRVLOG"
cat > /tmp/opencode/ve-gate-srv.py <<'PYEOF'
import http.server, socketserver, sys, time, threading, struct, zlib
from collections import deque
root, port, log = sys.argv[1], int(sys.argv[2]), sys.argv[3]
def _png1x1():
    ihdr = struct.pack('>IIBBBBB', 1, 1, 8, 6, 0, 0, 0)
    raw = b'\x00' + b'\x00\x00\x00\x00'
    def chunk(t, d):
        c = t + d
        return struct.pack('>I', len(d)) + c + struct.pack('>I', zlib.crc32(c) & 0xffffffff)
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', ihdr)
            + chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b''))
PNG1x1 = _png1x1()
pending, lock = deque(), threading.Lock()
class H(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map,
                      'wasm': 'application/wasm'}
    def __init__(self, *a, **k):
        super().__init__(*a, directory=root, **k)
    def do_GET(self):
        if self.path.startswith('/ve-gate-ready'):
            if 't=' in self.path:            # the beacon: proof the loop turned
                with lock:
                    while pending:
                        pending.popleft().set()  # release ALL held anchors (chrome may issue the img twice)
                with open(log, 'a') as f:
                    f.write('READY %.6f\n' % time.time())
                self._png()
            else:                            # the anchor: hold load until beacon
                ev = threading.Event()
                with lock:
                    pending.append(ev)
                ev.wait()
                try:
                    self._png()
                except Exception:
                    pass                     # browser gave up at its cap
            return
        super().do_GET()
    def _png(self):
        self.send_response(200); self.send_header('Content-Type', 'image/png')
        self.send_header('Content-Length', str(len(PNG1x1)))
        self.end_headers(); self.wfile.write(PNG1x1)
    def log_message(self, *a):
        pass
class S(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True
try:
    srv = S(('127.0.0.1', port), H)
except OSError as e:
    print('GATESRV BUSY %s' % e); sys.exit(3)
print('GATESRV LISTENING'); sys.stdout.flush()
srv.serve_forever()
PYEOF
setsid python3 /tmp/opencode/ve-gate-srv.py "$PWD/$GATE_DIR" "$PORT" "$BEACON" >"$SRVLOG" 2>&1 &
GATE_PID=$!
setsid cmake --build "$TREE" --target visiaengine-wasm-serve >/tmp/opencode/ve-serve.log 2>&1 &
SERVE_PID=$!
for i in $(seq 60); do  # 60s budget: staging replay + gate server bind; warm the cargo tree first, that is not the gate's job
  grep -q BUSY "$SRVLOG" 2>/dev/null && fail "VE_GATE_PORT $PORT busy, pass another"
  grep -q LISTENING "$SRVLOG" 2>/dev/null && curl -sf --noproxy "*" -o /dev/null "http://127.0.0.1:$PORT/index.html" && break
  sleep 1
  [ "$i" = 60 ] && fail "serve not reachable on :$PORT"
done

# -o counts occurrences; -c counts lines and dump-dom serializes JS-generated
# siblings onto ONE line. The wall renders after an async wall.json fetch, so a
# single dump can catch it mid-fetch (cold-start race, measured: 0 != 43).
# Re-roll with a fresh load up to 3 times — the verdict stays exact.
M=0
for t in 1 2 3; do
  M=$("${CHROME_ENV[@]}" "$CHROME" --headless=new --timeout=15000 --dump-dom "http://127.0.0.1:$PORT/index.html" 2>/dev/null | grep -o 'class="item' | wc -l)
  [ "$M" -eq "$N" ] && break
  sleep 0.5
done
[ "$M" -eq "$N" ] || fail "wall items $M != wall.json $N (3 rolls)"

# Engine boot leg: the anchor img holds load until the post-8-frame beacon
# releases it, so this dump lands AFTER boot: a fail mark or an uncaught
# error is visible, and so is the ready text. Hard RED on either failure sign.
D=$("${CHROME_ENV[@]}" "$CHROME" --headless=new $GPU_FLAGS --enable-logging=stderr --timeout=30000 --dump-dom \
     "http://127.0.0.1:$PORT/E101_clear.html" 2>&1)
echo "$D" | grep -q '✗'          && fail "engine reported a fail mark"
echo "$D" | grep -qiE 'uncaught' && fail "console shows uncaught error"

# Heartbeat leg, dual channel: (1) server-side READY line after this leg's
# start — race-free proof the rAF loop turned 8 frames (the page issues the
# beacon only after veLoop's promise settles); (2) if the dump carries
# data-frame it must still be >= VE_GATE_MIN_FRAMES. virtual-time stays
# banned: it never resolves the boot promise (measured, both headless forms).
F_T0=$(date +%s.%N)
F=$("${CHROME_ENV[@]}" "$CHROME" --headless=new $GPU_FLAGS --timeout=30000 --dump-dom \
    "http://127.0.0.1:$PORT/E101_clear.html" 2>/dev/null | grep -oE 'data-frame="[0-9]+"' | grep -oE '[0-9]+' | tail -1)
B_MS=$(python3 -c "
import sys, time
t0 = float(sys.argv[1])
try:
    ts = [float(l.split()[1]) for l in open(sys.argv[2]) if l.startswith('READY')]
except FileNotFoundError:
    ts = []
fresh = [x for x in ts if x >= t0]
print(int((max(fresh) - t0) * 1000) if fresh else -1)" "$F_T0" "$BEACON")
[ "$B_MS" -ge 0 ] || fail "no beacon (engine never turned ${VE_GATE_MIN_FRAMES} frames)"
[ "$B_MS" -le "$VE_GATE_BEACON_BUDGET_MS" ] || fail "beacon too slow (${B_MS}ms > ${VE_GATE_BEACON_BUDGET_MS}ms budget)"
if [ -n "$F" ]; then [ "$F" -ge "$VE_GATE_MIN_FRAMES" ] || fail "heartbeat stalled (frames=$F < $VE_GATE_MIN_FRAMES)"; fi

"${CHROME_ENV[@]}" "$CHROME" --headless=new --timeout=15000 --screenshot=/tmp/opencode/ve-wall.png --window-size=1280,800 \
  "http://127.0.0.1:$PORT/index.html" >/dev/null 2>&1
echo "GATE-WEB OK ($N wall items, beacon ${B_MS}ms, frames=${F:-snapshot-pre-boot}, shot /tmp/opencode/ve-wall.png)"
