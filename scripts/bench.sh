#!/usr/bin/env bash
# [6b] bench artifact chain (plan 4c §2.4): run the stress examples → parse RESULT →
# write the git-tracked resources/bench/<name>.json → compare with the previous version;
# a >20% regression prints red but exits 0 (observation band, not a gate).
# ⚠ numbers = this machine's lavapipe software rasterizer, **not a CI gate** (master plan v-B4 ruling).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
mkdir -p resources/bench
GPU_LABEL="lavapipe (Mesa software Vulkan; local-only, not CI)"
MACHINE=$(uname -n)
DATE=$(date +%F)

# per-example run → grab RESULT lines → merge-write the JSON (old values kept for comparison)
MISSING=0
for spec in "bench_twin:cargo run --release -p examples --example E601_bench_twin -- --frames 3" \
            "bench_pick:cargo run --release -p examples --example E401_pick_demo -- --bench" \
            "bench_pcl:cargo run --release -p examples --example E601_bench_twin -- --points 1000000 --frames 1"; do
    name="${spec%%:*}"
    cmd="${spec#*:}"
    old_json="resources/bench/${name}.json"
    # shellcheck disable=SC2086
    out=$($cmd 2>/dev/null | grep '^RESULT ' || true)
    [ -z "$out" ] && { echo "BENCH ✗ ${name} produced no RESULT lines" >&2; MISSING=$((MISSING+1)); continue; }
    # write the new JSON
    {
        printf '{\n  "date": "%s",\n  "machine": "%s",\n  "gpu": "%s",\n  "values": {\n' \
            "$DATE" "$MACHINE" "$GPU_LABEL"
        first=1
        while read -r _ rname val unit; do
            [ $first -eq 0 ] && printf ',\n'
            printf '    "%s": [%s, "%s"]' "$rname" "$val" "$unit"
            first=0
        done <<< "$out"
        printf '\n  }\n}\n'
    } > "$old_json"
    echo "BENCH ✓ $name → $old_json"
    # compare, >20% regression is flagged (baseline = the git HEAD version; the first version has no baseline and is skipped)
    python3 - "$old_json" <<'PYEOF'
import json, sys
# note: the previous step already overwrote the file; this compares against the git HEAD version
import subprocess
old_raw = subprocess.run(["git", "show", f"HEAD:{sys.argv[1]}"], capture_output=True)
if old_raw.returncode != 0:
    sys.exit(0)  # first version, no baseline to compare
try:
    old = json.loads(old_raw.stdout)
except Exception:
    sys.exit(0)
new = json.load(open(sys.argv[1]))
for k, (nv, _u) in new.get("values", {}).items():
    ov = old.get("values", {}).get(k)
    if ov and float(nv) > float(ov[0]) * 1.2:
        sys.stderr.write(f"\033[31mREGRESSION {k}: {ov[0]} → {nv} (>20%)\033[0m\n")
PYEOF
done
echo "BENCH done (observation band, not a gate; real thresholds are the user's call)"

# absence = toolchain assertion (package-name drift / missing example — the PIT-23
# constant-true family lesson: it once silently continued for two rounds without writing artifacts)
if [ "$MISSING" -gt 0 ]; then echo "BENCH ✗ ${MISSING} examples produced nothing (not a performance regression — a toolchain failure)" >&2; exit 1; fi
