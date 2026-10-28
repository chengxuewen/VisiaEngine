---
name: code-review-and-quality
description: "Code review and quality for this engine: five-axis review, severity labels (Nit/Optional/FYI/Blocking), ~100-line change discipline, adversarial second pass, and C24 commit hygiene. Use when reviewing a diff or PR, before commit, or after a large or risky change."
---

# Code Review & Quality (this workspace)

## Overview

- The **objective** half of review is already automated: `pixi run fmt/lint/check/test/audit`, `web-check`, and the `gate-*` segments in `pixi run ci`. Do not hand-review what a gate already proves.
- This skill covers the **subjective** half gates cannot: correctness under inputs they do not exercise, readability, maintainability, boundary to unsafe/FFI, and commit hygiene.
- The repo's commit discipline is **C24** (conventions): subject <= 72 chars, no `;` double-head, body uses `- ` bullets, atomic commits.

## When to Use

- Reviewing a diff, PR, or your own staged change before commit.
- After a large or risky change (unsafe, FFI, concurrency, public API, serialization).
- When a gate is red and you need to decide *how* to fix, not just *that* it is red.
- A second opinion ("复审") on the highest-risk hunk.

## Process (steps + checkpoints + exit)

1. **See the actual diff and its size**: `git diff` / `git diff --cached`. If one change is well over ~100 lines, ask to split it (atomic commits, C24). Checkpoint: you know the file:line surface.
2. **Run the gates first**: `pixi run fmt/lint/check/test` (+ `audit` on dependency changes, `web-check` on web). A red gate is not a review finding — it is a blocker to fix before review.
3. **Five axes**, each finding tagged **Blocking / Optional / Nit / FYI** and anchored to `file:line`:
   - Correctness — boundary inputs, error paths, `Result` vs panic, overflow.
   - Readability — naming, structure, dead code.
   - Maintainability — duplication, coupling, Rule of 500, `#[allow]` debt.
   - Performance — only with measurement; no speculative optimization.
   - Security — untrusted input, unsafe/FFI invariants, secrets, dependency risk.
4. **Adversarial second pass** on the riskiest hunk: re-read with fresh context (unsafe/FFI/concurrency/API). Ask "what input breaks this?".
5. **Commit hygiene (C24)**: subject <= 72, no `;`, body `- ` bullets, one logical change per commit.
6. **Exit condition**: every Blocking finding resolved or explicitly deferred with a reason; gates green; the findings list (file:line + severity + resolution) recorded as evidence.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "tests pass, so it's fine" | Tests prove the covered cases, not the uncovered ones; review the gap. |
| "it's a small change" | Small changes hide big risk (one `unsafe` line, one flipped condition). |
| "I'll fix the nit next PR" | If it is really a nit, label it Nit; if it matters, fix it now. |
| "the reviewer will catch it" | Review is not a substitute for the author running the gates. |
| "re-run review on everything" | Do not re-review what gates prove; spend effort where they cannot. |

## Red Flags

- Findings without `file:line` + severity (vague "looks off").
- Re-litigating what `clippy`/tests already cover.
- Nit-bombing a diff with no severity distinction.
- Approving without running the gates.
- A diff far larger than ~100 lines treated as one reviewable change.
- A commit subject over 72 chars, or with a `;` double-head (violates C24).

## Verification (evidence)

- The diff size and file:line list of findings with severity + resolution.
- The gate output (paste the green tail).
- For a risky hunk: the adversarial second-pass note (what input was considered, what broke).

## References (read on demand)

- `.agents/memorys/conventions.md` — C24 commit discipline and other C rules.
- `.agents/skills/security-hardening/SKILL.md` — deep security review (this skill = breadth).
- `.agents/skills/code-simplification/SKILL.md` — Rule of 500 / Chesterton's Fence.
- `docs/env.md` — gate/task reference.
