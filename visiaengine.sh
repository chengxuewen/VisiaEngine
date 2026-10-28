#!/usr/bin/env bash
# visiaengine.sh -- execution front door (human convenience; agent/CI keep using `pixi run`).
#
# Pure passthrough:  ./visiaengine.sh <task> [args...]   ==   pixi run <task> [args...]
# There is no command table here on purpose: pixi.toml is the single source of
# task names, so this file cannot drift when a task is added or renamed.
#
# Not source-able -- use `source pixi.sh` to activate a shell instead.
#
# Exit codes owned by this file: -h/--help -> 0, no args -> 2, pixi missing -> 127.
# Everything else is the task's own exit code.

set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
cd "$ROOT"

USAGE='usage: ./visiaengine.sh <task> [args...]

common:
  ./visiaengine.sh mcp status      MCP: pin/installed versions + index state
  ./visiaengine.sh mcp setup       install/repair MCP packages at the pin
  ./visiaengine.sh mcp sync        incremental MCP sync
  ./visiaengine.sh mcp index       build (first run) or rebuild .codegraph
  ./visiaengine.sh agents sync     agent config sync (rules/mcp/memory/skills links)
  ./visiaengine.sh agents check    verify every generated layer (non-zero on drift)
  ./visiaengine.sh agents status   read-only freshness report (always exit 0)
  ./visiaengine.sh gate-agents     single-source consistency gate
  ./visiaengine.sh ci              the whole CI chain
  pixi task list                   every task this workspace defines

`pixi run <task>` is always equivalent; Windows: visiaengine.bat <task>.'

case "${1-}" in
    '') echo "$USAGE"; exit 2 ;;
    -h|--help) echo "$USAGE"; exit 0 ;;
esac

# Resolve pixi the same way scripts/mcp-launch.sh does: PATH first, then ~/.pixi/bin.
# Keep this as `&&` chains: under `set -e` a failing left-hand side is exempt, so the
# checks below must not be "simplified" into `|| exit` (that would abort on success).
PIXI="$(command -v pixi || true)"
[ -z "$PIXI" ] && [ -x "$HOME/.pixi/bin/pixi" ] && PIXI="$HOME/.pixi/bin/pixi"
[ -z "$PIXI" ] && { echo "visiaengine.sh: pixi not found (run bootstrap.sh)" >&2; exit 127; }

exec "$PIXI" run "$@"
