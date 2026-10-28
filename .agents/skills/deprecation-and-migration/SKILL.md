---
name: deprecation-and-migration
description: "Deprecate and migrate safely in this repo: mark the old surface, publish a migration path, keep an alias until consumers move, then remove. Use when removing an API/flag/skill, renaming an export, or replacing a dependency surface."
---

# Deprecation & Migration (this workspace)

## Overview

- Deprecation is a **two-phase** move: mark + migrate first, remove later. Removing without a migration path breaks consumers silently.
- Precedent in this repo: `review-hardcode` is kept as an **explicit deprecated alias** (`status: deprecated`, description points to `security-hardening`) rather than deleted.
- For Rust APIs use `#[deprecated(note = "...")]`; for skills use `status: deprecated`; for protocol/ABI changes follow `api-interface-design`.

## When to Use

- Removing/renaming an exported symbol, CLI flag, config key, or skill.
- Replacing a dependency's surface (e.g. an MCP server, a crate).
- A breaking change that needs a migration window.

## Process (steps + checkpoints + exit)

1. **List consumers**: grep the repo (and known external users) for the symbol/flag/skill name. Checkpoint: you know who breaks.
2. **Mark the old surface**: `#[deprecated(note = "use <new> instead")]`, or `status: deprecated` + a description pointing at the replacement.
3. **Publish the migration path**: update the docs/README and, for ABI, the header mirror (`api-interface-design`).
4. **Keep the alias** until consumers migrate; a removed alias is a silent break.
5. **Remove** only after the migration window, with the removal recorded in `decisions.md` (D-series).
6. **Verify**: old references gone (grep), new path tested, gates green.
7. **Exit**: marked → migrated → removed, each step evidenced; removal has a D-entry.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "just delete it, nobody uses it" | Grep first; "nobody" is an assumption. |
| "skip the alias" | Without it, a consumer fails with no pointer to the replacement. |
| "one release removes it all" | A breaking change needs a window; migrate, then remove. |
| "no need to record the removal" | The D-entry is how the next person knows why it vanished. |

## Red Flags

- Deleting a symbol/skill with live references.
- A breaking change with no migration note.
- An alias with no deprecation marker or replacement pointer.
- Removal not recorded in `decisions.md`.

## Verification (evidence)

- `grep -rn '<name>'` shows only the migration note / no references.
- The replacement path has a passing test or a documented runbook.
- `gate-agents` / `gate-abi` green.

## References

- `api-interface-design` — contract/ABI mirror discipline.
- `documentation-and-adrs` — record the removal decision.
- `.agents/skills/review-hardcode/SKILL.md` — the deprecated-alias precedent.
