---
name: planning-and-task-breakdown
description: "Turn a spec or a vague ask into ordered, verifiable work slices for this engine: dependency order, <=7 todos, each slice leaving the workspace green. Use when starting a multi-step change, a plan/proposal needs tasks, or an ask is too large to execute safely."
---

# Planning & Task Breakdown (this workspace)

## Overview

- A plan is not a prose summary — it is an **ordered set of slices**, each with a verification and a dependency.
- The repo's constraint: every slice must leave `pixi run fmt/lint/check/test` green (and the relevant `gate-*`).
- Use `todo_write` for tracking; use `openspec-propose` when the change needs a spec/proposal artifact.
- Size the path first (`think-before-act` path sizing: Spike / Bounded / Architectural) — only Bounded and Architectural need this breakdown.

## When to Use

- A task touches multiple crates/layers and cannot be one commit.
- A spec/proposal needs an executable task list.
- An ask is too vague/large to start safely.

## Process (steps + checkpoints + exit)

1. **Restate the outcome** (what "done" looks like, for whom).
2. **Slice by dependency**, not by file: each slice = one behavior + one verification. Aim for <=7 slices.
3. **Name the verification per slice** (the gate/test that proves it).
4. **Order**: foundations first; each slice's output unblocks the next.
5. **Write the todos** (`todo_write`), first slice `in_progress`.
6. **Execute one slice**, run its verification, then mark it `completed`; a red slice blocks the next.
7. **Exit**: all slices done with their evidence; the workspace is green at every boundary.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "it's one change, no need to slice" | One big change hides which part broke. |
| "I'll verify at the end" | A late red tells you nothing about which slice caused it. |
| "20 micro-todos" | Too many todos is noise; group by behavior, keep <=7. |
| "the spec is the plan" | A spec is intent; the plan is the ordered execution. |

## Red Flags

- Slices with no verification.
- A slice that leaves the workspace red.
- Todos that are file names, not behaviors.
- No dependency order (parallel work that actually serializes).

## Verification (evidence)

- The todo list with each slice's verification.
- The gate/test output at each slice boundary.
- The final `pixi run ci` (or the relevant subset) green.

## References

- `openspec-propose` / `openspec-apply-change` — spec-driven change lifecycle.
- `incremental-implementation` — thin-slice execution.
- `constraint-driven-development` — gates are the verification.
