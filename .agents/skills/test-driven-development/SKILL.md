---
name: test-driven-development
description: "Red-green-refactor for this engine: write the failing test first, make it pass minimally, then refactor, using cargo test plus proptest/insta. Use when implementing a behavior, fixing a bug with a reproduction, or when a change needs a regression guard."
---

# Test-Driven Development (this workspace)

## Overview

- The loop is **red → green → refactor**: a failing test first, the smallest change that passes, then cleanup with the test still green.
- Tooling available: `cargo test`, `proptest` (already a workspace dependency), `insta` snapshots; the gate is `pixi run test`.
- Boundary with `test-harness`: this skill is the **loop**; `test-harness` generates skeletons from a spec.

## When to Use

- Implementing a new behavior (start with the test).
- Fixing a bug (write the failing reproduction first).
- Any change that needs a guard against regression.

## Process (steps + checkpoints + exit)

1. **Write the failing test** for the next behavior (unit, or `proptest` for input-space, or `insta` for output shape).
2. **Run it and see red**: `cargo test <name> -- --nocapture` — the failure must be the intended one.
3. **Make it pass minimally** — no speculative abstraction.
4. **Run green**: `cargo test` (or the crate subset).
5. **Refactor** with the test still green; rerun after each step.
6. **Gate**: `pixi run fmt && pixi run lint && pixi run test`.
7. **Exit**: the test fails without the change, passes with it, and the gate is green — with both outputs quoted.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "write the test after" | Then it proves what you wrote, not what you needed. |
| "I'll assert later" | A test without an assertion is a smoke test at best. |
| "the test is flaky, delete it" | Flakiness is a bug; seed/pin it, don't delete the guard. |
| "refactor first, tests after" | Refactor without a guard is unverifiable change. |
| "the failing test is too slow to run" | Then scope it (`-p <crate>` / `--lib`); don't skip the red step. |

## Red Flags

- Implementation merged before any test.
- A test that passes without the change (proves nothing).
- `#[ignore]` / skipped test to get green.
- Snapshot updated without reading the diff.
- Refactor with no test behind it.

## Verification (evidence)

- The red run (before) and the green run (after), both quoted.
- The test name + the behavior it pins.
- `pixi run test` green (plus `fmt`/`lint`).

## References

- `.agents/rules/rust/testing.md`, `.agents/rules/common/testing.md`.
- `test-harness` — generate skeletons from a spec.
- `debugging-and-error-recovery` — when the failure needs localization first.
