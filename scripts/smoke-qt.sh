#!/usr/bin/env bash
# smoke-qt (Qt round v1.3/Q3): a three-state gate -- no qt-spike environment / no X display = SKIP exit 0 (the gate family discipline);
# both present = configure+build+6 real-window frames, with a stdout assert of "OK qt pump".
# Call shape: pixi run -e qt-spike smoke-qt (env activated: cmake/$CONDA_PREFIX in place)
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
if [ ! -f ".pixi/envs/qt-spike/lib/cmake/Qt6/Qt6Config.cmake" ]; then
    echo "SMOKE-QT SKIP (qt-spike not installed: pixi install -e qt-spike)"
    exit 0
fi
DISP="${DISPLAY:-:0}"
if [ ! -S "/tmp/.X11-unix/X${DISP#:}" ]; then
    echo "SMOKE-QT SKIP (no X display $DISP -- the xvfb/mirror CI rounds carry the three states)"
    exit 0
fi
cmake --preset ci-strict >/dev/null || { echo "SMOKE-QT ✗ configure"; exit 1; }
cmake --build --preset ci-strict >/dev/null || { echo "SMOKE-QT ✗ build"; exit 1; }
out=$(DISPLAY=$DISP QT_QPA_PLATFORM=xcb \
      QT_QPA_PLATFORM_PLUGIN_PATH="$CONDA_PREFIX/lib/qt6/plugins" \
      timeout -k 5 120 ./cmake-build-ci/bin/E703_qt_viewer --frames 6 2>&1) \
    || { echo "SMOKE-QT ✗ run"; echo "$out" | tail -3; exit 1; }
grep -q "OK qt pump" <<<"$out" || { echo "SMOKE-QT ✗ exit assert missing"; echo "$out" | tail -3; exit 1; }
echo "SMOKE-QT ✓ (real window, 6 frames @ $DISP, $out" | head -1
echo "$out" | grep "T3 manual" | head -1 || true
