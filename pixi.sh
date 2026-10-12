#!/usr/bin/env bash
# pixi.sh -- activate the VisiaEngine pixi environment in the current shell
# Usage: source pixi.sh      (do NOT run 'bash pixi.sh' -- the subshell deactivates on exit)
# No environment yet? Run first: bash bootstrap.sh

_vdir="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd -P)"

_vpixi=""
if command -v pixi >/dev/null 2>&1; then
    _vpixi="$(command -v pixi)"
elif [ -x "$HOME/.pixi/bin/pixi" ]; then
    _vpixi="$HOME/.pixi/bin/pixi"
fi
if [ -z "$_vpixi" ]; then
    echo "pixi not found -- run first: bash bootstrap.sh" >&2
    unset _vdir
    return 1
fi

# TTY guard (archived H1 lesson): in non-interactive use (piped capture / script consumption), print no banner
if [ -t 1 ]; then
    echo "Activating the VisiaEngine pixi environment ($_vpixi)..."
fi
eval "$("$_vpixi" shell-hook --manifest-path "$_vdir/pixi.toml" --shell bash)"
if [ -t 1 ]; then
    echo ""
    echo "Environment activated. Common tasks:"
    echo "  pixi run verify   -- toolchain smoke test"
    echo "  pixi run lint     -- clippy -D warnings (once the workspace lands)"
    echo "  pixi task list    -- all tasks"
    echo "To deactivate: exit, or close this shell"
fi
unset _vdir _vpixi
