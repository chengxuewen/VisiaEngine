#!/usr/bin/env bash
# SDD clause numbers ↔ test `// spec:` bidirectional coverage gate (plan §4). L2 takes no clause
# number and naturally stays outside the scan surface.
# Clause numbering starts at the docs/sdd live max+1 on implementation day (current: CORE10/REND20/WGPU11/GLTF8/GEO14).
# ⚠ new namespace: extend PREFIX_RE in this single source (two regexes + the third check share this table);
# a clause shape/tag with a non-whitelisted prefix on disk = third check red.
# Example gate: see rules/common/development-workflow.md (every new API must carry a runnable example).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."

# prefix single source (extend namespaces only on this line — both regexes and the third check share the same whitelist)
PREFIX_RE='CORE|REND|WGPU|GLTF|GEO|CAPI|IO'
# only clause title lines count (`## CORE-NN: ...`); in-prose cross references / range notation are not clauses
sdds=$(grep -ohE "^## ($PREFIX_RE)-[0-9]{2}\b" docs/sdd/*.md 2>/dev/null | sed 's/^## //' | sort -u)
tags=$(grep -rhoE "// spec: ($PREFIX_RE)-[0-9]{2}" crates/ bindings/ 2>/dev/null | grep -oE "($PREFIX_RE)-[0-9]{2}" | sort -u)

# third check (point-cloud band consensus 3): a clause shape/tag with a non-whitelisted prefix on disk = red —
# this turns "extend the namespace but forget the regex = silent double-sided blindness, fake green" from a taboo into a gate problem.
ghost=$( { grep -ohE '^## [A-Z]+-[0-9]{2}\b' docs/sdd/*.md 2>/dev/null | sed 's/^## //;s/-[0-9][0-9]//'
           grep -rhoE '// spec: [A-Z]+-[0-9]{2}' crates/ bindings/ 2>/dev/null | sed 's|// spec: ||;s/-[0-9][0-9]//'; } \
         | sort -u | grep -vxE "$PREFIX_RE" || true)
if [ -n "$ghost" ]; then
    echo "ERROR: prefix outside the whitelist (spec-trace not extended, or a ghost clause number): $(echo $ghost)" >&2
    exit 1
fi

if [ -z "$sdds" ] && [ -z "$tags" ]; then
    echo "spec-trace OK: 0 clauses (empty set is legal during skeleton phase)"
    exit 0
fi
if [ "$sdds" != "$tags" ]; then
    echo "ERROR: SDD/spec trace mismatch (left = clause without test, right = test without clause):" >&2
    diff <(printf '%s\n' "$sdds") <(printf '%s\n' "$tags") >&2 || true
    exit 1
fi
echo "spec-trace OK: $(printf '%s\n' "$sdds" | wc -l) pairs aligned"
