#!/usr/bin/env bash
# bootstrap.sh -- first-time initialization of the VisiaEngine dev environment (idempotent, safe to re-run)
# Usage: bash bootstrap.sh
# D5: the environment is managed entirely by pixi (conda-forge single source, rust toolchain included). Never bring in rustup on this machine.
# Note: no set -e -- the script is designed to be sourceable without polluting the user shell's errexit state; all errors are handled explicitly

set -uo pipefail

BOOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd -P)"
PIXI_PIN="0.78.0"   # D5 clause ④: version pinned (CI pins the same value)

ver_ge() { [ "$(printf '%s\n%s\n' "$2" "$1" | sort -V | head -n1)" = "$2" ]; }

pixi_version() { "$1" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1; }

# --- [1/3] Locate or install pixi ---
PIXI=""
if command -v pixi >/dev/null 2>&1; then
    PIXI="$(command -v pixi)"
elif [ -x "$HOME/.pixi/bin/pixi" ]; then
    PIXI="$HOME/.pixi/bin/pixi"
fi

if [ -n "$PIXI" ]; then
    have="$(pixi_version "$PIXI")"
    if ver_ge "${have:-0}" "$PIXI_PIN"; then
        echo "[1/3] pixi ${have} already present (${PIXI})"
    else
        echo "[1/3] existing pixi ${have} is below the pinned version ${PIXI_PIN}; reinstalling into ~/.pixi ..."
        PIXI=""
    fi
fi

if [ -z "$PIXI" ]; then
    echo "[1/3] network pre-check: conda-forge reachability..."
    if ! curl -sI --max-time 10 https://conda.anaconda.org/conda-forge/noarch/repodata.json >/dev/null 2>&1; then
        echo "ERROR: cannot reach conda-forge. Configure a mirror and retry (see the mirror section of docs/env.md)" >&2
        exit 1
    fi
    tmp_inst="$(mktemp)"
    if ! curl -fsSL --proto '=https' --tlsv1.2 "https://pixi.sh/install.sh" -o "$tmp_inst"; then
        echo "ERROR: pixi installer download failed" >&2
        rm -f "$tmp_inst"
        exit 1
    fi
    # Execute from disk (auditable), not a curl|sh pipe; the version assert below is the backstop
    PIXI_VERSION="v${PIXI_PIN}" bash "$tmp_inst"
    rc=$?
    rm -f "$tmp_inst"
    [ "$rc" -eq 0 ] || { echo "ERROR: pixi installer execution failed (rc=$rc)" >&2; exit 1; }
    PIXI="$HOME/.pixi/bin/pixi"
fi

got="$(pixi_version "$PIXI")"
if ! ver_ge "${got:-0}" "$PIXI_PIN"; then
    echo "ERROR: pixi version assert failed: got '${got:-none}', required >= ${PIXI_PIN}" >&2
    exit 1
fi

# --- [2/3] Resolve and install dependencies ---
echo "[2/3] pixi install (first conda-forge solve includes downloads; may take minutes)..."
if ! "$PIXI" install --manifest-path "$BOOT_DIR/pixi.toml"; then
    echo "      First solve failed -> regenerate the lock and retry once (lock-drift recovery, archived proven pattern)"
    "$PIXI" update --manifest-path "$BOOT_DIR/pixi.toml" || { echo "ERROR: pixi update failed" >&2; exit 1; }
    "$PIXI" install --manifest-path "$BOOT_DIR/pixi.toml" || { echo "ERROR: pixi install failed" >&2; exit 1; }
fi

# --- [3/3] Smoke ---
echo "[3/3] Smoke: pixi run verify ..."
"$PIXI" run --manifest-path "$BOOT_DIR/pixi.toml" verify || { echo "ERROR: verify failed" >&2; exit 1; }

echo ""
echo "=============== VisiaEngine environment ready ==============="
echo "  Daily activation:  source pixi.sh"
echo "  One-shot commands: pixi run <task>     (all tasks: pixi task list)"
echo "  Guide:             docs/env.md"
echo "===================================================="
