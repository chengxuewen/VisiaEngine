#!/usr/bin/env bash
# smoke-qt（Qt 轮 v1.3/Q3）：三态门——无 qt-spike 环境/无 X 显示=SKIP exit0（gate 族纪律）；
# 双在=configure+build+6 帧真窗跑，stdout 断言 "OK qt pump"。
# 调用形态：pixi run -e qt-spike smoke-qt（env 已激活：cmake/$CONDA_PREFIX 就位）
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
if [ ! -f ".pixi/envs/qt-spike/lib/cmake/Qt6/Qt6Config.cmake" ]; then
    echo "SMOKE-QT SKIP (qt-spike 未装：pixi install -e qt-spike)"
    exit 0
fi
DISP="${DISPLAY:-:0}"
if [ ! -S "/tmp/.X11-unix/X${DISP#:}" ]; then
    echo "SMOKE-QT SKIP (无 X 显示 $DISP——xvfb/镜像 CI 轮接三态)"
    exit 0
fi
cmake --preset qt-pixi >/dev/null || { echo "SMOKE-QT ✗ configure"; exit 1; }
cmake --build --preset qt-pixi >/dev/null || { echo "SMOKE-QT ✗ build"; exit 1; }
out=$(DISPLAY=$DISP QT_QPA_PLATFORM=xcb \
      QT_QPA_PLATFORM_PLUGIN_PATH="$CONDA_PREFIX/lib/qt6/plugins" \
      timeout -k 5 120 ./target/qt-build/examples/qt/E703_qt_viewer --frames 6 2>&1) \
    || { echo "SMOKE-QT ✗ run"; echo "$out" | tail -3; exit 1; }
grep -q "OK qt pump" <<<"$out" || { echo "SMOKE-QT ✗ 出口断言缺失"; echo "$out" | tail -3; exit 1; }
echo "SMOKE-QT ✓（真窗 6 帧 @ $DISP，$out" | head -1
echo "$out" | grep "T3 人检" | head -1 || true
