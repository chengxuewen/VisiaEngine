---
name: debugging-and-error-recovery
description: "Systematic debugging and error recovery for this engine: reproduce, localize, narrow (bisect), minimal fix, add a regression guard, and safe rollback. Use when a test fails, behavior is wrong, a gate reddens, or an incident needs a root cause."
---

# Debugging & Error Recovery (this workspace)

## Overview

- This repo's failure surface is broad: Rust crates + C ABI (`gate-abi`) + wasm binding + the web wall (`web-check`/`gate-web`) + many `gate-*` segments in `pixi run ci`.
- The discipline is **reproduce before you fix** (the same "verify before claim" rule used across this repo). A fix without a reproduction is a guess.
- This skill is the *workflow*; the Rust/testing knowledge lives in `.agents/rules/rust/testing.md` and `.agents/rules/rust/patterns.md`.

## When to Use

- A test, build, clippy, or a `gate-*` segment is red.
- Behavior is wrong but nothing crashes (silent bug).
- A web page's beacon never fires / frames stall.
- An incident needs root cause and a guard against recurrence.

## Process (steps + checkpoints + exit)

1. **Reproduce deterministically**:
   - Rust: `cargo test <name> -- --nocapture`, `RUST_BACKTRACE=1`; narrow with `-p <crate>`.
   - Web: `pixi run web-check`; inspect `examples/wasm/common.js` heartbeat + the page beacon / `VE_GATE_FRAMES`.
   - Checkpoint: you can make it fail on demand (or prove nondeterminism with a rate).
2. **Localize the layer**: is it core / ffi / wasm / web / a gate script? Read the failing gate's script (e.g. `scripts/web-build.sh`, `gate-abi`).
3. **Narrow with bisection / minimal input**: shrink the repro (smallest scene, single frame, one page); for regressions, `git bisect` when the change window is known.
4. **Fix minimally**: change the smallest surface that removes the cause; do not refactor while debugging.
5. **Add a regression guard**: a failing-then-passing test — unit for Rust, `proptest` for input-space bugs, `insta` snapshot for output shape, or a beacon assertion for web.
6. **Verify the guard**: the new test must fail without the fix and pass with it. Then run the gates (`pixi run fmt/lint/check/test`, `web-check` when web).
7. **Recovery**: if the fix is risky, prefer a reversible/patch-level change or a documented rollback over a rushed rewrite.
8. **Exit condition**: reproducible → fixed → guard in place → gates green, with the evidence (before/after output) quoted.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "I'll just change this and see" | Without a reproduction you cannot tell whether it is fixed or moved. |
| "the test is flaky, ignore it" | Flakiness is a bug; capture the rate and root-cause it. |
| "add `#[ignore]`/skip to get green" | That is hiding the symptom, not fixing it; the gate must stay honest. |
| "it only fails on my machine" | Compare `pixi run verify` + env; the toolchain is pixi-pinned. |
| "no time for a regression test" | Without a guard it will regress; the test is the fix's proof. |

## Red Flags

- A fix with no reproduction or no before/after evidence.
- `#[ignore]`, `#[allow]`, `--no-verify`, or deleted assertions used to "go green".
- Refactoring during debugging.
- A web fix verified only by "the page loads" without the beacon/gate-web evidence.
- Guessing at a `gate-*` failure without opening that gate's script.

## Verification (evidence)

- The exact command that fails before and passes after (paste both).
- The regression test: name + proof it fails without the fix.
- For web: `pixi run web-check` output + the beacon/heartbeat sample for the affected page.

## References (read on demand)

- `.agents/rules/rust/testing.md` — unit/property/snapshot patterns (`proptest`, `insta`).
- `.agents/rules/rust/patterns.md` — error propagation, panic vs Result.
- `scripts/web-build.sh` + `pixi.toml` `web-check` — web gate.
- `docs/env.md` — toolchain + `ci` segment reference.
