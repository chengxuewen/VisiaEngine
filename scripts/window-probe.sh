#!/usr/bin/env bash
# window-probe.sh — 窗例"第一帧"机器守卫（N 系列黑屏三连的根治：E305/E508/E505
# 全靠人工抓出，机器面全绿——本探针把"窗里有没有画面"变成 CI 断言）。
# 用法: window-probe.sh <E###_stem> [lit_ratio_min_percent]
#   零参跑例（人验形态=C15）→ Xvfb 截屏 → 采样亮度比 ≥ 阈值（默认 5%）。
# skip 形：无 xwd/Xvfb/X_display → exit 0（CI 无显示环境不误红，gate 惯例）。
# 黑屏=0/7500 实锤（E505）；清屏色也算 lit（>40 含深蓝黑 13+18+25=56……注意
# 清屏色亮度 56 刚过 40——采样步长 8 下全清屏也 100% lit，故阈值语义=「非纯黑
# swapchain 未初始化」，不代表内容正确；内容正确由各自 golden 像素门守）。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."

STEM="${1:?用法: window-probe.sh <E###_stem> [min_pct]}"
MINPCT="${2:-5}"

XVFB=$(command -v Xvfb || true)
XWD=$(command -v xwd || true)
if [ -z "$XVFB" ] || [ -z "$XWD" ]; then
  echo "WINDOW-PROBE SKIP (no Xvfb/xwd)"
  exit 0
fi

BIN="target/debug/examples/${STEM}"
[ -f "$BIN" ] || BIN="target/cmake-bare/examples/rs/${STEM}"
[ -f "$BIN" ] || { echo "WINDOW-PROBE ✗ ${STEM} 无二进制（先 build）"; exit 1; }

# 启 Xvfb（离散显示号避开 CI 并发）
DISP=":78"
"$XVFB" "$DISP" -screen 0 800x600x24 > /dev/null 2>&1 &
XVPID=$!
sleep 1
cleanup() { kill "$XVPID" 2>/dev/null; }
trap cleanup EXIT

# 零参跑例（人验形态），6s 后截屏（窗生命周期自管；常驻例被 kill 收）
DISPLAY="$DISP" "$BIN" > /dev/null 2>&1 &
APPID=$!
sleep 6
ALIVE="yes"
kill -0 "$APPID" 2>/dev/null || ALIVE="no"
# 先截屏后杀（root 截屏依赖 app 窗口存活——先杀=截到黑底，首版踩坑）
DISPLAY="$DISP" "$XWD" -root -display "$DISP" -out /tmp/wp_probe.xwd 2>/dev/null

# 自退例（--frames 族）若已退出仍可截到残帧？不能——swapchain 随进程消失。
# 故自退例 skip（它们的画面由各自 golden 断言守）；本探针专守常驻窗。
[ "$ALIVE" = "yes" ] || { echo "WINDOW-PROBE SKIP ${STEM} (self-exiting example)"; exit 0; }

[ -s /tmp/wp_probe.xwd ] || { echo "WINDOW-PROBE ✗ ${STEM} 截屏失败"; exit 1; }

# 亮度比：stdlib 解析 xwd（head 25×u32 BE + colormap；BGRA 像素）
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
# 非零=纯黑/近纯黑=swapchain 未初始化（黑屏家族信号）
[ $rc -eq 0 ] && echo "WINDOW-PROBE ✓ ${STEM}" || echo "WINDOW-PROBE ✗ ${STEM} 黑屏（第一帧未渲）"
exit $rc
