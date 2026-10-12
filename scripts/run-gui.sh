#!/bin/sh
# Launch wrapper for the interactive run steps: IDE/remote shells often have no DISPLAY (measured in this repo: the vscode-server process tree carries no display variables).
# Rule: all three variables empty AND a local :0 X socket exists -> fall back to DISPLAY=:0 (this repo's desktop shape); otherwise exec unchanged,
# letting winit/SDL report a clear error. Only cargo-run_* and local manual runs use this -- CI/ctest never pass through this wrapper.
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ] && [ -z "${WAYLAND_SOCKET:-}" ] && [ -S /tmp/.X11-unix/X0 ]; then
    DISPLAY=:0; export DISPLAY
    echo "run-gui: no display variables in the environment, falling back to DISPLAY=:0 (local X socket probe)" >&2
fi
exec "$@"
