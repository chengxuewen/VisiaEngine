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

## Verification discipline (hand-written)

A check that has never been seen failing is not a check, it is decoration.

- Adding or changing a gate requires ONE planted-violation demo: break the rule on purpose → the gate goes red and its output **names the defect**; restore → green. Record the command and the result in the commit message.
- A gate whose red state was never demonstrated counts as **not done** — "the report says it is enforced" is not evidence.
- New gate scripts carry their own `--self-test` (in-built probe, temp files only, never touches the repo): `rule-ref-lint.py`, `skill-lint.py`, `check-promises.sh` are the pattern to follow.
- The probe must prove the plant **took effect** before trusting the exit code (a `sed`/edit that silently misses prints the same green as a passing gate — PIT-97), and attribution must be read: which segment went red, and does its message name the planted defect?
- Backlog (not yet enforced per-gate): in-built self-tests for the contract-watching gates, in this order — `gate-abi` → `gate-mcp` → `spec-trace` → `gate-web` legs. A false green there promises an ABI/protocol shape to consumers.

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
13. `rule-ref`
14. `gate-mcp`
15. `check-promises`
16. `cmake-smoke`
17. `pack-check`
18. `i18n`
<!-- END:ci-gates -->
