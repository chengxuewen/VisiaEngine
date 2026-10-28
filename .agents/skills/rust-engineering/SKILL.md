---
name: rust-engineering
description: "Rust engineering workflow for this engine workspace: fmt/clippy/test/deny gates, unsafe and FFI review, MSRV/edition awareness, and property tests. Use when writing, reviewing, or debugging Rust in this workspace (crates/, examples/rs, visiaengine-ffi, wasm binding), or when a Rust change must pass the local quality gates before commit."
---

# Rust Engineering (this workspace)

## Overview

- This repo is a **Cargo workspace**: `edition = "2024"`, `rust-version = "1.98.0"`, members `crates/*` + `examples/rs` + `crates/visiaengine-ffi` + `bindings/wasm/visiaengine-wasm`; `proptest` is already a workspace dependency.
- The toolchain is provided by **pixi** (no rustup on the machine). Quality gates are pixi tasks, not ad-hoc commands.
- This skill is the **execution workflow**. The *knowledge* (style, patterns, security, testing, hooks) lives in `.agents/rules/rust/*.md` — read the relevant file(s) instead of duplicating them here.
- Before asserting any version/edition, **read the file** (`Cargo.toml`); never guess.

## When to Use

- Writing, reviewing, or debugging Rust code in this workspace.
- Before committing any `.rs` or `Cargo.toml` change.
- Any change touching `unsafe`, the C ABI (`visiaengine-ffi`), or the wasm binding.
- When `cargo` / clippy / fmt fails and you need a triage path.

## Process (steps + checkpoints + exit)

1. **Read the real facts** (do not assume):
   - `Cargo.toml` → `edition`, `rust-version`, workspace members.
   - `pixi.toml` `[tasks]` → the exact gate commands.
   - Checkpoint: you can state the edition, the MSRV, and the gate task names.
2. **Load the relevant rule file(s)** from `.agents/rules/rust/` (`coding-style.md`, `patterns.md`, `security.md`, `testing.md`, `hooks.md`) — only the ones the task actually touches.
3. **Write a thin vertical slice**; keep the change reviewable. Do not add `#[allow(...)]` to silence warnings unless justified in a comment.
4. **Run the gates** (exact commands from `pixi.toml`):
   - `pixi run fmt` — `cargo fmt --all -- --check`
   - `pixi run lint` — `cargo clippy --workspace --all-targets -- -D warnings`
   - `pixi run check` / `pixi run test`
   - `pixi run audit` — `cargo deny check` (any dependency change)
   - Checkpoint: all green. If fmt fails, `pixi run fmt-fix`, then re-run `pixi run fmt`.
5. **Unsafe / FFI / wasm**: every `unsafe` block needs a `// SAFETY:` comment stating the invariant; run the ABI gate (`gate-abi`, part of `pixi run ci`) and the wasm binding tests.
6. **Pre-commit**: run `pixi run verify` (toolchain smoke). Note the `pre-commit` hook auto-runs `agent-sync` + `gate-agents`; do not bypass it with `--no-verify`.
7. **Exit condition**: gates green **and** the evidence (command output) quoted in the change summary. "Looks right" is not evidence.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "I'll run clippy later" | `lint` is `-D warnings`; CI fails on any warning. Run it now. |
| "`#[allow(dead_code)]` is harmless" | It hides real debt; justify it or remove the code. |
| "fmt is cosmetic" | `pixi run fmt` is a gate; failing fmt blocks the commit. |
| "MSRV is probably fine" | Read `Cargo.toml` `rust-version`; do not guess. |
| "unsafe is obvious here" | Every `unsafe` needs `// SAFETY:` + a reviewer-visible invariant. |
| "skip the slow test" | The gate runs `pixi run test`; a silently skipped test is a red flag. |

## Red Flags

- `#[allow(...)]` or `#[ignore]` added without a justification comment.
- `--no-verify` used to commit.
- New dependency without `pixi run audit` (cargo-deny) passing.
- An `unsafe` block with no `// SAFETY:`.
- Editing generated host layers (`.codebuddy/**`) or `.agents/rules/` copies instead of the single source.
- Hardcoding the toolchain version/edition instead of reading `Cargo.toml`.

## Verification (evidence)

- Quote the actual green output of `pixi run fmt && pixi run lint && pixi run check && pixi run test` (plus `pixi run audit` when dependencies changed).
- For unsafe/FFI: the `// SAFETY:` comment and the `gate-abi` result.
- For wasm: the binding tests result.

## References (progressive disclosure — read on demand)

- `.agents/rules/rust/coding-style.md` — naming, structure, idioms.
- `.agents/rules/rust/patterns.md` — ownership, error handling, concurrency patterns.
- `.agents/rules/rust/security.md` — unsafe/FFI/untrusted input.
- `.agents/rules/rust/testing.md` — unit/property/insta patterns (`proptest` available).
- `.agents/rules/rust/hooks.md` — lifecycle/hooks conventions.
- `docs/env.md` — toolchain and gate reference (pixi tasks, `ci` segments).
