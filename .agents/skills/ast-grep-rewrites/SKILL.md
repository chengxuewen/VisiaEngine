---
name: ast-grep-rewrites
description: "Structural search and rewrite with the ast-grep CLI (installed in the pixi env): pattern-based bulk edits that respect syntax, metavariable captures preserved, gates run after. Use for bulk call-pattern changes or cross-crate renames; not for one-off small edits."
---

# ast-grep Rewrites (this workspace)

## Overview

- `ast-grep` (0.51.0, pixi pypi `ast-grep-cli`) matches code by **syntax tree**, not text: strings and comments are not false matches, and metavariables (`$A`, `$B`) capture arguments so rewrites preserve them.
- Boundary: `code-simplification` decides *what* to restructure; this skill is the *mechanical rewrite* tool; `ai-slop-cleanup` handles *text noise*.
- `.agents/rules/common/edit-safety.md` forbids `sed` for code — this is the sanctioned alternative for pattern rewrites.

## When to Use

- A bulk call-pattern change (e.g. `foo(x)` → `bar(x)`) across crates.
- A cross-crate rename of an API usage pattern.
- A structural audit ("how many places call X?") before deciding a refactor.

## Process (steps + checkpoints + exit)

1. **Self-check**: `ast-grep --version` — if missing, the env is stale; `pixi install` first.
2. **Dry-run the pattern**: `ast-grep run -p '<pattern>' -l rust` — inspect every hit; tighten the pattern until **only intended sites** match. Checkpoint: the hit list is exactly the change set.
3. **Preview the rewrite**: add `-r '<replacement>'` — ast-grep prints a diff; read it site by site.
4. **Apply**: `--update-all` (or `-U`) writes the files.
5. **Gate**: `pixi run fmt && pixi run lint && pixi run check && pixi run test` — a mechanical rewrite is only done when gates pass.
6. **Review the diff**: revert if any unintended site changed; a mechanical edit still gets read.
7. **Exit**: hits list + applied diff + gates green, all quoted.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "regex is faster" | Regex matches strings and comments; a bad rewrite ships in a commit. |
| "skip the dry-run" | The pattern IS the contract; the dry-run is what proves it. |
| "skip the gate, it's mechanical" | Mechanical edits still break builds; the gates are cheap. |
| "use it for this one small edit" | Overkill — a direct `edit` is safer for one site (edit-safety). |
| "loosen the pattern to catch more" | Over-matching is the failure mode; tighten instead. |

## Red Flags

- A rewrite applied without a dry-run hit list.
- The applied diff not read site by site.
- Gates not run after the rewrite.
- Using ast-grep where a single direct `edit` would do.
- A pattern that only matches "enough" sites after loosening.

## Verification (evidence)

- The dry-run hit list (before) and the applied diff (after), both quoted.
- `pixi run fmt/lint/check/test` green.
- Site count before == intended sites after.

## References

- `.agents/rules/common/edit-safety.md` — why `sed`/regex rewrites are forbidden.
- `.agents/skills/code-simplification/SKILL.md`, `.agents/skills/ai-slop-cleanup/SKILL.md` — different axes.
- `pixi.toml` `[pypi-dependencies]` — the pinned `ast-grep-cli`.
