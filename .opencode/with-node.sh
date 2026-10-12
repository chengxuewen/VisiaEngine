#!/usr/bin/env bash
# with-node.sh — MCP launch wrapper: resolve the node runtime, then exec "$@" with it.
# Order: repo pixi env (D5 single source) -> user-level pixi env -> PATH.
# Also prepends the node directory to PATH — the bridge scripts' child processes need npm/npx on PATH (launching node by absolute path alone cannot do this).
set -u
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# pixi itself injected (the openspace and other bridge scripts' child processes call execSync('pixi ...') and need it)
if ! command -v pixi >/dev/null 2>&1 && [ -x "$HOME/.pixi/bin/pixi" ]; then
    export PATH="$HOME/.pixi/bin:$PATH"
fi

for cand in "$DIR/../.pixi/envs/default/bin" "$HOME/.pixi/envs/default/bin"; do
    if [ -x "$cand/node" ]; then
        export PATH="$cand:$PATH"
        exec "$@"
    fi
done

if command -v node >/dev/null 2>&1; then
    exec "$@"
fi

echo "with-node.sh: node runtime not found (run pixi install or source pixi.sh first)" >&2
exit 127
