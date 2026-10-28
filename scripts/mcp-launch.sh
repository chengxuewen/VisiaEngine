#!/usr/bin/env bash
# mcp-launch.sh -- MCP launch wrapper (host-neutral, Unix).
#
# Resolves pixi robustly (reuses the PolyOrch/scripts/pixi.sh pattern: PATH first,
# then $HOME/.pixi/bin/pixi), locates the repo manifest relative to this script,
# then execs the pixi task. Used by gitignored host layers (e.g. .codebuddy/mcp.json)
# so they work regardless of the spawn PATH / cwd. Tracked host configs stay
# OS-neutral and rely on pixi being on the GUI-inherited PATH (O5).
#
# Usage: mcp-launch.sh mcp-<name>
set -u
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="$(cd "$DIR/.." && pwd -P)"

PIXI="$(command -v pixi 2>/dev/null || true)"
if [ -z "$PIXI" ] && [ -x "$HOME/.pixi/bin/pixi" ]; then
    PIXI="$HOME/.pixi/bin/pixi"
fi
if [ -z "$PIXI" ]; then
    echo "mcp-launch: pixi not found (provision via bootstrap)" >&2
    exit 127
fi

exec "$PIXI" run --manifest-path "$ROOT/pixi.toml" "$@"
