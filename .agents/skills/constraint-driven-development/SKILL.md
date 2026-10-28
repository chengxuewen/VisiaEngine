---
name: constraint-driven-development
description: "Constraint-driven development for this engine: one quality-gate source of truth (CONSTRAINTS.md derived from pixi.toml ci), cost-based check placement, and anti-gaming rules that keep gates honest. Use when adding a check/gate, deciding where a check belongs, before a risky commit, or when tempted to bypass a gate."
---

# Constraint-Driven Development (this workspace)

## Overview

- **Single source of truth** = `CONSTRAINTS.md`, whose gate list is **generated from `pixi.toml` `[tasks] ci` depends-on** (do not hand-edit the generated region; `gate-agents` asserts it matches pixi.toml).
- `pixi run ci` is the full local quality gate; the `pre-commit` hook runs `agent-sync` + `gate-agents`, so the single source and the generated layers stay in lock-step.
- The point of this skill is **anti-gaming**: prevent an agent (or human) from turning a red gate green by disabling the check instead of fixing the cause.

## When to Use

- Adding or changing a quality check / gate.
- Deciding *where* a check belongs (fast local vs slow CI).
- Before any risky commit, or when a gate is red.
- Anytime you feel the urge to bypass a gate "just this once".

## Process (steps + checkpoints + exit)

1. **Read the truth**: `CONSTRAINTS.md` (anti-gaming + gate list) and `pixi.toml` `[tasks] ci` `depends-on`. Checkpoint: you can name the current gates without inventing any.
2. **To add a check**: add it as a `pixi` task, wire it into `ci` `depends-on`, then run `pixi run agent-sync` so `CONSTRAINTS.md` regenerates; `gate-agents` asserts the list matches. Never hand-edit the generated region.
3. **Place by cost**: cheap/fast checks shift left (local, pre-commit); slow/expensive ones live in CI. Do not put a 5-minute job in the pre-commit hook.
4. **Keep gates honest (anti-gaming)**: no `--no-verify`; no `#[allow(...)]` without a justification comment; no `#[ignore]` to skip a failing test (the only sanctioned `#[ignore]` are the wasm/env-gated tests `web-check` runs with `-- --ignored`); never weaken an assertion to pass.
5. **Fix the cause, not the gate.** A red gate is right until you prove otherwise with evidence.
6. **Exit condition**: `pixi run ci` green, `CONSTRAINTS.md` in sync (gate-agents), and the anti-gaming rules respected — with the gate output quoted.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "just this once, `--no-verify`" | The hook exists to prevent exactly this; it will bite the next person. |
| "temporarily `#[ignore]` the failing test" | Temporarily is permanent; fix it or track it explicitly, never silently. |
| "the lint is too strict, add `#[allow]`" | Justify the invariant in a comment, or change the code; do not blanket-silence. |
| "weaken the assert to get green" | That is hiding the symptom the gate found. |
| "the gate is wrong" | Prove it with evidence, then fix the gate as a separate, reviewed change. |

## Red Flags

- `--no-verify`, weakened assertions, or `#[ignore]` used to pass a gate.
- `#[allow(...)]` without a justification comment.
- A new check added but not wired into `ci` (so it never runs).
- `CONSTRAINTS.md` hand-edited (drift from `pixi.toml`); `gate-agents` will redden.
- A slow job stuffed into the pre-commit hook.

## Verification (evidence)

- `pixi run ci` green output (the tail of the run).
- `pixi run gate-agents` → `GATE-CONSTRAINTS ✓ ci-gates match`.
- For a new gate: the pixi task + the `ci` depends-on diff, and `CONSTRAINTS.md` regenerated.

## References (read on demand)

- `CONSTRAINTS.md` — anti-gaming rules + generated gate list.
- `pixi.toml` — `[tasks]` incl. the `ci` depends-on.
- `.githooks/pre-commit` — the auto `agent-sync` + `gate-agents` hook.
- `.agents/skills/code-review-and-quality/SKILL.md` — review discipline (C24 commit hygiene).
- `docs/env.md` — toolchain + task reference.
