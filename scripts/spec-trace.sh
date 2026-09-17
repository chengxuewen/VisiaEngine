#!/usr/bin/env bash
# SDD 条款号 ↔ 测试 `// spec:` 双向覆盖门禁（计划 §4）。L2 不占号，天然不在扫描面。
# 条款起号以实施日 docs/sdd 实际 max+1 为准（现况 CORE10/REND20/WGPU11/GLTF8/GEO14）。
# ⚠ 新命名空间：扩 PREFIX_RE 单源即可（两处正则+第三检同表）；盘上出现白名单外前缀=第三检红。
# 示例门禁见 rules/common/development-workflow.md（API 新增必带可运行 example）。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."

# 前缀单源（扩命名空间只改这一行——两侧正则与第三检共用同表白名单）
PREFIX_RE='CORE|REND|WGPU|GLTF|GEO|CAPI|IO'
# 只认条款标题行（`## CORE-NN: ...`），正文交叉引用/区间记法不计条款
sdds=$(grep -ohE "^## ($PREFIX_RE)-[0-9]{2}\b" docs/sdd/*.md 2>/dev/null | sed 's/^## //' | sort -u)
tags=$(grep -rhoE "// spec: ($PREFIX_RE)-[0-9]{2}" crates/ bindings/ 2>/dev/null | grep -oE "($PREFIX_RE)-[0-9]{2}" | sort -u)

# 第三检（点云带共识 3）：盘上出现「白名单外前缀」的条款形/标签 = 红——
# 把「扩命名空间漏改正则=双侧静默失明假绿」从禁忌降维成门禁问题。
ghost=$( { grep -ohE '^## [A-Z]+-[0-9]{2}\b' docs/sdd/*.md 2>/dev/null | sed 's/^## //;s/-[0-9][0-9]//'
           grep -rhoE '// spec: [A-Z]+-[0-9]{2}' crates/ bindings/ 2>/dev/null | sed 's|// spec: ||;s/-[0-9][0-9]//'; } \
         | sort -u | grep -vxE "$PREFIX_RE" || true)
if [ -n "$ghost" ]; then
    echo "ERROR: 白名单外前缀（漏扩 spec-trace 或幽灵条款号）: $(echo $ghost)" >&2
    exit 1
fi

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
