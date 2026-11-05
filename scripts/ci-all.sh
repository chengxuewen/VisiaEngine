#!/bin/sh
# ci-all — DIAGNOSTIC gate sweep: run every ci site, then summarize. Never abort early.
#
# The gate list has ONE source: pixi.toml `[tasks] ci` depends-on, read here and never
# copied into this file (a copied list drifts the day someone adds a gate).
#
# Why this exists: pixi runs depends-on in order and stops at the first failure (measured
# in the same-lineage remote_control_host repo, probe 2026-11-02), so a workspace that is
# red in several places reveals exactly one defect per run -- N gates, N round trips.
#
# What this is NOT: an acceptance gate. Acceptance stays `pixi run ci` green (fail-fast,
# runner-side, and the version .github/workflows/ci.yml will call). Passing here only
# means "you now know every current red".
#
# Usage:
#   bash scripts/ci-all.sh                 # sweep all ci sites
#   bash scripts/ci-all.sh gate-skills ... # sweep only the named sites (fast inner loop)
set -u
cd "$(git rev-parse --show-toplevel)"
export PATH="$HOME/.pixi/bin:$PATH"

if ! command -v pixi >/dev/null 2>&1; then
  echo "ci-all: pixi not found on PATH; run 'bash bootstrap.sh' first" >&2
  exit 1
fi

# Sandboxed / GUI hosts can mount /tmp read-only; pixi environment activation writes
# there and dies with ReadOnlyFilesystem (same finding as the remote_control_host
# governance entry point). Fall back to a workspace-local TMPDIR instead of failing
# every gate for the wrong reason.
if ! touch "${TMPDIR:-/tmp}/.ci-all-probe" 2>/dev/null; then
  mkdir -p "$PWD/.cache/tmp"
  export TMPDIR="$PWD/.cache/tmp"
  echo "ci-all: TMPDIR unwritable, using $TMPDIR"
else
  rm -f "${TMPDIR:-/tmp}/.ci-all-probe"
fi

# The gate list has ONE reader: pixi.toml `[tasks] ci` depends-on. Both the sweep and
# --list call this function, so the two can never disagree.
read_gates() {
  python3 - <<'PY'
import pathlib
import re

text = pathlib.Path("pixi.toml").read_text(encoding="utf-8")
match = re.search(r"^\s*ci\s*=\s*\{.*?depends-on\s*=\s*\[(.*?)\]", text, re.M | re.S)
if match:
    print(" ".join(x.strip().strip('"') for x in match.group(1).split(",") if x.strip()))
PY
}

# --list first: naming the sweep must not cost an environment activation.
if [ "${1:-}" = "--list" ]; then
  printf 'ci-all sites: %s\n' "$(read_gates)"
  exit 0
fi

# Entry duty, not a check: regenerate the host layers from the single source before any
# gate reads them. Deliberately NOT part of the gate list (sync is not an assertion).
pixi run agents sync || { echo "ci-all FAIL: agents sync"; exit 1; }

gates=$(read_gates)
if [ -z "$gates" ]; then
  echo "ci-all FAIL: could not read the ci depends-on list from pixi.toml" >&2
  exit 1
fi
filters="$*"   # `$*` expands safely under set -u when no args were given; "$1" would abort
[ -n "$filters" ] && gates="$filters"

fails=0
report=""
for gate in $gates; do
  echo "== $gate =="
  if pixi run "$gate"; then
    report="$report\n  OK   $gate"
  else
    rc=$?
    report="$report\n  FAIL $gate (rc=$rc)"
    fails=$((fails + 1))
  fi
done

printf '%b\n' "$report"
if [ "$fails" -gt 0 ]; then
  echo "ci-all: $fails site(s) red  [swept: $gates]"
  echo "ci-all is diagnostic only -- acceptance requires 'pixi run ci' green."
  exit 1
fi
echo "ci-all: all swept sites green  [swept: $gates]"
