#!/usr/bin/env bash
# SDD 条款号 ↔ 测试 `// spec:` 双向覆盖门禁（计划 §4）。L2 不占号，天然不在扫描面。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."

# 只认条款标题行（`## CORE-NN: ...`），正文交叉引用/区间记法不计条款
sdds=$(grep -ohE '^## (CORE|REND|WGPU)-[0-9]{2}\b' docs/sdd/*.md 2>/dev/null | sed 's/^## //' | sort -u)
tags=$(grep -rhoE '// spec: (CORE|REND|WGPU)-[0-9]{2}' crates/ 2>/dev/null | grep -oE '(CORE|REND|WGPU)-[0-9]{2}' | sort -u)

if [ -z "$sdds" ] && [ -z "$tags" ]; then
    echo "spec-trace OK: 0 条款（骨架期空集合法）"
    exit 0
fi
if [ "$sdds" != "$tags" ]; then
    echo "ERROR: SDD/spec 追溯失配（左=有规无测，右=有测无规）:" >&2
    diff <(printf '%s\n' "$sdds") <(printf '%s\n' "$tags") >&2 || true
    exit 1
fi
echo "spec-trace OK: $(printf '%s\n' "$sdds" | wc -l) 条双向对齐"
