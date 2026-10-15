#!/usr/bin/env bash
# bootstrap.sh -- first-time initialization of the VisiaEngine dev environment (idempotent, safe to re-run)
# Usage: bash bootstrap.sh
# D5: the environment is managed entirely by pixi (conda-forge single source, rust toolchain included). Never bring in rustup on this machine.
# Note: no set -e -- the script is designed to be sourceable without polluting the user shell's errexit state; all errors are handled explicitly
# T-B2 (2026-10-10): stages [1/2] now delegate to the PolyOrch standard pixi-init
# (submodule scripts/pixi-init.sh, --manifest-path parameterized); this script keeps
# only what is VisiaEngine-specific: the submodule init prerequisite (cold clones),
# the cargo mirror policy (host network policy, not the supplier's), and the smoke.

set -uo pipefail

BOOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd -P)"
PIXI_INIT="$BOOT_DIR/PolyOrch/scripts/pixi-init.sh"
MANIFEST="$BOOT_DIR/pixi.toml"

# --- [0/3] submodule prerequisite (cold clones) ---
# The standard pixi-init lives in the PolyOrch submodule; a fresh clone has an
# empty PolyOrch/ until it is initialized. Idempotent: a populated checkout is a no-op.
if [ ! -f "$PIXI_INIT" ]; then
    echo "[0/3] PolyOrch submodule not populated -- initializing ..."
    git -C "$BOOT_DIR" submodule update --init --recursive \
        || { echo "ERROR: git submodule update --init --recursive failed (network? gitee access?)" >&2; exit 1; }
fi
[ -f "$PIXI_INIT" ] || { echo "ERROR: still missing $PIXI_INIT after submodule init" >&2; exit 1; }

# Activation trio comes from the supplier; root copies are generated content.
cp -f "$BOOT_DIR/PolyOrch/scripts/pixi.sh" "$BOOT_DIR/PolyOrch/scripts/pixi.bat" "$BOOT_DIR/PolyOrch/scripts/pixi.ps1" "$BOOT_DIR/"

# --- [1/3] pixi + environment via the PolyOrch standard bootstrap ---
bash "$PIXI_INIT" --manifest-path "$MANIFEST" || exit 1
PIXI="$HOME/.pixi/bin/pixi"
command -v pixi >/dev/null 2>&1 && PIXI="$(command -v pixi)"

# --- [2/3] cargo mirror (VisiaEngine host policy; NOT part of the supplier bootstrap) ---
# crates.io mirror for China (rsproxy.cn by ByteDance). Sparse protocol must use the
# /index/ path (the /crates.io-index/ form is the git-protocol address; 404 under sparse).
# Fallback: switch replace-with to ustc-sparse if rsproxy is down.
if [ ! -f "$HOME/.cargo/config.toml" ] || ! grep -q "rsproxy" "$HOME/.cargo/config.toml" 2>/dev/null; then
    echo "[2/3] configuring the crates.io mirror (rsproxy.cn, sparse)..."
    mkdir -p "$HOME/.cargo"
    cat > "$HOME/.cargo/config.toml" <<'TOML'
[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"

[registries.rsproxy]
index = "sparse+https://rsproxy.cn/index/"
TOML
    echo "      written $HOME/.cargo/config.toml (skip this stage by pre-seeding the file)"
else
    echo "[2/3] cargo mirror already configured -- skipped"
fi

# --- [3/3] Smoke ---
echo "[3/3] Smoke: pixi run verify ..."
"$PIXI" run --manifest-path "$MANIFEST" verify || { echo "ERROR: verify failed" >&2; exit 1; }

echo ""
echo "=============== VisiaEngine environment ready ==============="
echo "  Daily activation:  source pixi.sh"
echo "  One-shot commands: pixi run <task>     (all tasks: pixi task list)"
echo "  Handy tasks:      pixi run verify (toolchain smoke) / pixi run lint (clippy -D warnings)"
echo "  Guide:             docs/env.md"
echo "===================================================="
