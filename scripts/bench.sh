#!/usr/bin/env bash
# [6b] bench 制品链（计划 4c §2.4）：跑压力例 → 解析 RESULT → 写 git 跟踪的
# resources/bench/<name>.json → 与上一版对比，劣化>20% 红字但 exit 0（观测非门禁）。
# ⚠ 数字=本机 lavapipe 软光栅，**非 CI 门禁**（总计划 v-B4 裁决）。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
mkdir -p resources/bench
GPU_LABEL="lavapipe (Mesa software Vulkan; 本机独有，非 CI)"
MACHINE=$(uname -n)
DATE=$(date +%F)

# 逐例执行 → 抓 RESULT 行 → 合并写 JSON（保留旧值做对比）
for spec in "bench_twin:cargo run --release -p visiaengine-render-wgpu --example E601_bench_twin -- --frames 3" \
            "bench_pick:cargo run --release -p visiaengine-render-wgpu --example E401_pick_demo -- --bench"; do
    name="${spec%%:*}"
    cmd="${spec#*:}"
    old_json="resources/bench/${name}.json"
    # shellcheck disable=SC2086
    out=$($cmd 2>/dev/null | grep '^RESULT ' || true)
    [ -z "$out" ] && { echo "BENCH ✗ ${name} 无 RESULT 行" >&2; continue; }
    # 写新 JSON
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
    # 对比劣化>20%（基线=git HEAD 版；首版无基线自动跳过）
    python3 - "$old_json" <<'PYEOF'
import json, sys
# 注：上一步已覆盖写入，此处对比 git HEAD 版本
import subprocess
old_raw = subprocess.run(["git", "show", f"HEAD:{sys.argv[1]}"], capture_output=True)
if old_raw.returncode != 0:
    sys.exit(0)  # 首版，无对比基线
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
echo "BENCH done（观测档，非门禁；真阈值=用户裁）"
