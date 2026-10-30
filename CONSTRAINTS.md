# CONSTRAINTS — what "done" means here

Single source of truth for the local quality gate. Two parts:

1. **Anti-gaming rules** (hand-written below) — behaviours that are never allowed to make a gate green.
2. **The CI gate list** — GENERATED from `pixi.toml` (`[tasks] ci` depends-on). Edit `pixi.toml`, not the marker region.

## Anti-gaming rules (hand-written)

- Never commit with `--no-verify` — it bypasses the `pre-commit` hook (auto `agent-sync` + `gate-agents`).
- Never add `#[allow(...)]` to silence a warning without a justification comment naming the invariant.
- Never use `#[ignore]` to skip a failing test. The only sanctioned `#[ignore]` are the wasm/env-gated tests that `web-check` runs explicitly with `-- --ignored`.
- Never delete or weaken an assertion to make a red gate green.
- Never hand-edit generated layers: `.codebuddy/**`, or the `AGENTS.md` memory / skills-directory marker regions.
- Any dependency change must pass `pixi run audit` (cargo-deny); a version bump goes through `pixi add` / `pixi update` and commits the lock.
- Fix the cause, not the gate. If a gate is red, the gate is right until proven otherwise.

<!-- BEGIN:ci-gates -->

## CI gates (generated from `pixi.toml` -> `[tasks] ci` depends-on; edit pixi.toml, not this block)

`pixi run ci` runs, in order:

1. `fmt`
2. `lint`
3. `check`
4. `test`
5. `audit`
6. `gate-style`
7. `gate-trace`
8. `gate-abi`
9. `gate-docs`
10. `gate-prose`
11. `gate-skills`
12. `gate-agents`
13. `gate-mcp`
14. `check-promises`
15. `cmake-smoke`
16. `pack-check`
17. `i18n`
<!-- END:ci-gates -->
