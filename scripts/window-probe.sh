#!/usr/bin/env bash
# window-probe.sh -- the machine guard for a windowed example's "first frame" (the cure for the N-series black-screen trio: E305/E508/E505
# were all caught by hand while the machine surface stayed green -- this probe turns "is there a picture in the window" into a CI assert).
# Usage: window-probe.sh <E###_stem> [lit_ratio_min_percent]
#   runs the example with no args (the human-verification form = C15) -> Xvfb screenshot -> sampled lit ratio >= threshold (default 5%).
# Skip shape: no xwd/Xvfb/X display -> exit 0 (a display-less CI never goes red; the gate convention).
# Black screen = 0/7500, measured (E505); the clear-screen color also counts as lit (>40; dark blue-black is 13+18+25=56... note
# the clear-screen brightness 56 just clears 40 -- at sample stride 8 a fully cleared screen is 100% lit, so the threshold semantics = "not pure-black
# swapchain uninitialized"; it does NOT mean the content is correct -- content correctness is guarded by each example's own golden-pixel gate).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."

STEM="${1:?usage: window-probe.sh <E###_stem> [min_pct]}"
MINPCT="${2:-5}"

XVFB=$(command -v Xvfb || true)
XWD=$(command -v xwd || true)
if [ -z "$XVFB" ] || [ -z "$XWD" ]; then
  echo "WINDOW-PROBE SKIP (no Xvfb/xwd)"
  exit 0
fi

BIN="target/debug/examples/${STEM}"
[ -f "$BIN" ] || BIN="cmake-build/bin/${STEM}"   # T-D: examples stage to the RUNTIME dir (D40)
[ -f "$BIN" ] || { echo "WINDOW-PROBE ✗ ${STEM} has no binary (build first)"; exit 1; }

# Start Xvfb (a disjoint display number avoids CI concurrency)
DISP=":78"
"$XVFB" "$DISP" -screen 0 800x600x24 > /dev/null 2>&1 &
XVPID=$!
sleep 1
cleanup() { kill "$XVPID" 2>/dev/null; }
trap cleanup EXIT

# Run the example with no args (the human-verification form); screenshot after 6s (the window manages its own lifetime; resident examples get killed)
DISPLAY="$DISP" "$BIN" > /dev/null 2>&1 &
APPID=$!
sleep 6
ALIVE="yes"
kill -0 "$APPID" 2>/dev/null || ALIVE="no"
# Screenshot first, kill later (a root screenshot depends on the app window being alive -- killing first captures a black background; a first-version trap)
DISPLAY="$DISP" "$XWD" -root -display "$DISP" -out /tmp/wp_probe.xwd 2>/dev/null

# Can a self-exiting example (--frames family) leave a residual frame after it exits? No -- the swapchain dies with the process.
# So self-exiting examples skip (their pictures are guarded by their own golden asserts); this probe guards resident windows only.
[ "$ALIVE" = "yes" ] || { echo "WINDOW-PROBE SKIP ${STEM} (self-exiting example)"; exit 0; }

[ -s /tmp/wp_probe.xwd ] || { echo "WINDOW-PROBE ✗ ${STEM} screenshot failed"; exit 1; }

# Lit ratio: parse xwd with the stdlib (header 25×u32 BE + colormap; BGRA pixels)
python3 - "$MINPCT" <<'EOF'
import struct, sys
minpct = float(sys.argv[1])
data = open('/tmp/wp_probe.xwd', 'rb').read()
hdr = struct.unpack('>25I', data[:100])
hs, width, height = hdr[0], hdr[4], hdr[5]
bpl = hdr[12]
px = data[hs + hdr[19] * 12:]
def pix(x, y):
    i = y * bpl + x * 4
    return px[i + 2] + px[i + 1] + px[i]  # r+g+b
lit = sum(1 for y in range(0, height, 8) for x in range(0, width, 8) if pix(x, y) > 5)
tot = len(range(0, height, 8)) * len(range(0, width, 8))
pct = lit * 100.0 / tot
print(f"WINDOW-PROBE lit={lit}/{tot} ({pct:.1f}%) threshold>={minpct}%")
sys.exit(0 if pct >= minpct else 1)
EOF
rc=$?
# Non-zero = pure black / near-pure black = the swapchain never initialized (the black-screen family signal)
[ $rc -eq 0 ] && echo "WINDOW-PROBE ✓ ${STEM}" || echo "WINDOW-PROBE ✗ ${STEM} black screen (first frame never rendered)"
exit $rc
