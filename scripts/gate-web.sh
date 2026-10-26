#!/usr/bin/env bash
# gate-web — headless page gate for the wasm example wall (gate server :8018).
# Asserts: served wall renders every wall.json entry, a live page boots
# the engine without a fail mark, and its rAF heartbeat turns (dual channel:
# server-side beacon log + data-frame snapshot).
# SKIP contract: missing chromium / build tree / wall.json => SKIP rc=0.
# Serving: the cmake serve button (:8017) replays web-demo staging exactly as
# before; the gate OWNS a stdlib-python server on VE_GATE_PORT (8018) over the
# assembled dir. Each live page's anchor <img> is held by this server until
# that page's post-await beacon (VE_GATE_FRAMES frames turned) arrives: the
# browser then survives to see it, load fires after boot, dump lands on truth.
set -u
CHROME="${VE_CHROMIUM:-}"
[ -x "$CHROME" ] || CHROME="$HOME/.pixi/global/bin/chromium"
[ -x "$CHROME" ] || CHROME="$(ls "$HOME"/.cache/ms-playwright/chromium-*/chrome-linux/chrome 2>/dev/null | head -1)"
# Chromium pin (D26 closure lives HERE: conda-forge carries no chromium
# package, so a pixi feature+lock path is impossible; the gate is the
# single-source pin). Re-pinning version/URL/sha is deliberate work: the
# GPU_FLAGS combo below was measured on this exact build and must be
# re-verified on every re-pin.
VE_CHROME_VERSION="Chromium 131.0.6778.33"
VE_CHROME_URL="https://cdn.npmmirror.com/binaries/playwright/builds/chromium/1148/chromium-linux.zip"
VE_CHROME_SHA256="7455d17b407bbcd1cc30c5bc8bc81fa4e67ac13635dbd5ca81f7824ec4b50439"
if [ ! -x "$CHROME" ] && [ "${VE_GATE_PROVISION:-0}" = "1" ]; then  # opt-in ONLY, never auto-download
  ZIP=/tmp/opencode/chromium-1148.zip
  curl -fSL --retry 3 -o "$ZIP" "$VE_CHROME_URL" || fail "provision: download failed"
  echo "$VE_CHROME_SHA256  $ZIP" | sha256sum -c - || fail "provision: sha256 mismatch"
  mkdir -p "$HOME/.cache/ms-playwright/chromium-1148"
  unzip -oq "$ZIP" -d "$HOME/.cache/ms-playwright/chromium-1148" || fail "provision: unzip failed"
  chmod +x "$HOME/.cache/ms-playwright/chromium-1148/chrome-linux/chrome"
  CHROME="$HOME/.cache/ms-playwright/chromium-1148/chrome-linux/chrome"
fi
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
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
cleanup() {
  [ -n "${SERVE_PID:-}" ] && kill -- -"$SERVE_PID" 2>/dev/null || true
  [ -n "${GATE_PID:-}" ]  && kill -- -"$GATE_PID"  2>/dev/null || true
}  # process groups: serve and gate server are group leaders via setsid
trap cleanup EXIT
fail() { echo "GATE-WEB FAIL: $1"; cleanup; exit 1; }
[ -x "$CHROME" ] || skip "no chromium binary"
[ -d "$TREE" ]   || skip "no build tree $TREE"
[ -f "$WALL" ]   || skip "no wall.json"
# the engine prints a trailing space; trim trailing whitespace before the
# exact comparison against the pin
VER=$("$CHROME" --version 2>/dev/null | tr -d '\r' | sed 's/[[:space:]]*$//')
[ "$VER" = "$VE_CHROME_VERSION" ] || fail "chromium drift: found $VER, pinned $VE_CHROME_VERSION -- re-provision (VE_GATE_PROVISION=1) or re-pin deliberately"

N=$(python3 -c "import json;print(len(json.load(open('$WALL'))['entries']))") || fail "wall.json unreadable"
rm -f "$BEACON" "$SRVLOG"
cat > /tmp/opencode/ve-gate-srv.py <<'PYEOF'
import http.server, socketserver, sys, time, threading, struct, zlib
from collections import defaultdict
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
pending, lock = defaultdict(list), threading.Lock()  # page_id -> held anchor events
class H(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map,
                      'wasm': 'application/wasm'}
    def __init__(self, *a, **k):
        super().__init__(*a, directory=root, **k)
    def do_GET(self):
        if self.path.startswith('/ve-gate-ready/'):
            seg = self.path[14:].split('?', 1)
            page = seg[0].strip('/')
            if len(seg) > 1 and (seg[1].startswith('t=') or seg[1].startswith('selftest=')):
                # the beacon: proof the loop turned (t=) or the pick-surface
                # probe reported (?selftest=... -- E401 pick grid, gated below)
                with lock:
                    for ev in pending.pop(page, []):
                        ev.set()  # release THIS page's anchors only (chrome may issue the img twice)
                with open(log, 'a') as f:
                    f.write('READY %s %.6f %s\n' % (page, time.time(), seg[1]))
                self._png()
            else:                            # the anchor: hold load until beacon
                ev = threading.Event()
                with lock:
                    pending[page].append(ev)
                ev.wait()
                try:
                    self._png()
                except Exception:
                    pass                     # browser gave up at its cap
            return
        super().do_GET()                     # passthrough: pages, pkg, fixtures
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

# Engine + heartbeat legs, looped over all 6 live pages. The anchor img
# holds load until the post-8-frame beacon releases it, so each dump lands
# AFTER boot: a fail mark or an uncaught error is visible, and so is the
# ready text. Heartbeat is dual channel: (1) server-side READY line for
# this page after the leg start - race-free proof the rAF loop turned 8
# frames (the page issues the beacon only after veLoop's promise settles);
# (2) if the dump carries data-frame it must still be >= VE_GATE_MIN_FRAMES.
# virtual-time stays banned: it never resolves the boot promise (measured,
# both headless forms).
# live pages = the wall manifest's own list (single source; the hardcoded
# copy drifted from wall.json twice in the live-page expansion)
PAGES=$(python3 -c "
import json
w = json.load(open('$WALL'))
print(' '.join(e['id'] for e in w['entries'] if e.get('kind') == 'live'))")
BEACON_SUM=""
for P in $PAGES; do
  # cold-start race (5th sighting this band): the FIRST page run after a
  # staging swap can miss its beacon entirely. Re-roll the leg up to 2
  # extra times on a red; the verdict itself stays exact (same predicate).
  VERDICT=""
  for ROLL in 1 2 3; do
    D=$("${CHROME_ENV[@]}" "$CHROME" --headless=new $GPU_FLAGS --enable-logging=stderr --timeout=30000 --dump-dom \
         "http://127.0.0.1:$PORT/$P.html" 2>&1)
    if echo "$D" | grep -q '✗'; then VERDICT="fail-mark"; continue; fi
    if echo "$D" | grep -qiE 'uncaught'; then VERDICT="uncaught"; continue; fi
    VERDICT="clean"; break
  done
  [ "$VERDICT" = "fail-mark" ] && fail "engine reported a fail mark ($P)"
  [ "$VERDICT" = "uncaught" ]  && fail "console shows uncaught error ($P)"
  # PIT-82: tag balance is a build-time shape, checked on the SOURCE file --
  # an unclosed module tag never evaluates and never reaches the console.
  _src="$REPO/examples/wasm/$P.html"
  if [ "$(grep -c '<script' "$_src")" != "$(grep -c '</script>' "$_src")" ]; then
    fail "tag imbalance: $(grep -c '<script' "$_src") opens vs $(grep -c '</script>' "$_src") closes ($P)"
  fi
  F_T0=$(date +%s.%N)
  # E401's second run carries ?selftest: the pick-surface probe beacon
  # (its query rides the READY log line, asserted below).
  PURL="$P.html"
  [ "$P" = "E401_pick_demo" ] && PURL="E401_pick_demo.html?selftest"
  B_MS=-1; F=""
  for ROLL in 1 2 3; do
    F_T0=$(date +%s.%N)
    F=$("${CHROME_ENV[@]}" "$CHROME" --headless=new $GPU_FLAGS --timeout=30000 --dump-dom \
        "http://127.0.0.1:$PORT/$PURL" 2>/dev/null | grep -oE 'data-frame="[0-9]+"' | grep -oE '[0-9]+' | tail -1)
    B_MS=$(python3 -c "
import sys
t0, page = float(sys.argv[1]), sys.argv[3]
try:
    ts = [float(l.split()[2]) for l in open(sys.argv[2]) if l.startswith('READY ' + page + ' ')]
except FileNotFoundError:
    ts = []
fresh = [x for x in ts if x >= t0]
print(int((max(fresh) - t0) * 1000) if fresh else -1)" "$F_T0" "$BEACON" "$P")
    [ "$B_MS" -ge 0 ] && break
  done
  [ "$B_MS" -ge 0 ] || fail "no beacon ($P: engine never turned ${VE_GATE_MIN_FRAMES} frames)"
  [ "$B_MS" -le "$VE_GATE_BEACON_BUDGET_MS" ] || fail "beacon too slow ($P ${B_MS}ms > ${VE_GATE_BEACON_BUDGET_MS}ms budget)"
  # the selftest run's dump races the boot (frames snapshot unreliable);
  # its t= beacon IS the heartbeat proof, so the frames check is E401-scoped off
  if [ "$P" != "E401_pick_demo" ] && [ -n "$F" ]; then
    [ "$F" -ge "$VE_GATE_MIN_FRAMES" ] || fail "heartbeat stalled ($P: frames=$F < $VE_GATE_MIN_FRAMES)"
  fi
  # E401 pick-surface assertion: the ?selftest grid must show BOTH cubes
  # pickable (>=1 RED and >=1 GREEN cell) -- pins the pick chain end to end.
  if [ "$P" = "E401_pick_demo" ]; then
    python3 - "$BEACON" <<-'PYCHK' || fail "E401 pick surface: no RED+GREEN verdict in the selftest beacon (pick chain broken or page cached)"
import sys, urllib.parse
try:
    lines = open(sys.argv[1]).read().splitlines()
except FileNotFoundError:
    sys.exit(1)
grids = [l.split(' ', 3)[3] for l in lines
         if l.startswith('READY E401_pick_demo ') and 'selftest=' in l]
if not grids:
    sys.exit(1)
cells = urllib.parse.unquote(grids[-1].split('selftest=', 1)[1]).split(' ')
kinds = {c.rsplit(':', 1)[-1] for c in cells if ':' in c}
if 'RED' not in kinds or 'GREEN' not in kinds:
    sys.exit(1)
print('GATE-WEB E401 pick surface: RED+GREEN present (%d cells)' % len(cells))
PYCHK
  fi
  echo "GATE-WEB page $P: beacon=${B_MS}ms frames=${F:-ready(snapshot)}"
  BEACON_SUM="$BEACON_SUM $P:$B_MS"
done

"${CHROME_ENV[@]}" "$CHROME" --headless=new --timeout=15000 --screenshot=/tmp/opencode/ve-wall.png --window-size=1280,800 \
  "http://127.0.0.1:$PORT/index.html" >/dev/null 2>&1
echo "GATE-WEB OK ($N wall items, $(echo $PAGES | wc -w) live pages, beacons$BEACON_SUM ms, shot /tmp/opencode/ve-wall.png)"
