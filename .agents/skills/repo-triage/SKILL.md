---
name: repo-triage
description: "Layer triage for an ambiguous task in this polyglot engine: map the ask to core-engine / FFI / Qt / wasm / CLI-infra / docs, list affected crates and ABI surfaces, then dispatch to the right skill. Use when a task's scope is unclear or spans crates, bindings, and scripts."
---

# repo-triage — Which Layer, Which Skill?

> Most slow tasks are mis-routed tasks. Find the layer first, then the skill and the gate.

## When to Use

- The request mixes languages or directories and the boundary is unclear.
- You cannot name the affected crate(s)/binding(s) from the ask.
- A change touches more than one of: Rust core, C ABI, Qt, wasm, scripts/CI, docs.

## Layers (this repo's real map)

| Layer | Paths | Ruleset | Usual skills |
|-------|-------|---------|--------------|
| **core-engine** | `crates/visiaengine-core`, `-render`, `-render-wgpu`, `-geo`, `-io-*` | `rules/rust`, `rules/common` | `rust-engineering`, `performance-optimization`, `debugging-and-error-recovery` |
| **ffi** | `crates/visiaengine-ffi`, `bindings/c`, `bindings/cpp` | `rules/rust`, `rules/cpp` | `api-interface-design`, `security-hardening` |
| **qt** | `bindings/qt`, `examples/qt` | `rules/cpp` | `api-interface-design` |
| **wasm** | `bindings/wasm`, `examples/wasm` | `rules/rust`, `rules/web`, `rules/typescript` | `browser-testing`, `rust-engineering` |
| **cli-infra** | `scripts/`, `pixi.toml`, `.githooks/`, `.agents/` | `rules/common` | `ci-cd-automation`, `ecosystem-scan`, `constraint-driven-development` |
| **docs** | `docs/`, `AGENTS.md`, `CONSTRAINTS.md` | `rules/common` | `doc-audit` |

## Process (steps + checkpoints + exit)

1. **Restate the ask** in one sentence (what changes, for whom).
2. **Map to layer(s)** using the table above; if two layers, name the boundary (e.g. FFI).
3. **List affected artifacts**: crate(s), exported symbol(s), header/hpp/wasm mirror(s), gate(s).
4. **Checkpoint**: you can state layer + artifacts + the gate that proves it.
5. **Dispatch** to the layer skill(s) from the table, and load the ruleset via `context-engineering`.
6. **State the verification** before editing: e.g. `cargo clippy -p <crate> -- -D warnings`, `pixi run web-check`, `pixi run gate-abi`.
7. **Exit**: layer + artifacts + gate are named; then hand off to the specific skill.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "just start editing" | Without the layer you will load the wrong rules and miss the gate. |
| "it's all one repo, so it's one layer" | Core / FFI / Qt / wasm have different rulesets and gates. |
| "the ask is clear enough" | Clear asks still cross a boundary; name the affected artifacts. |
| "I'll find the gate later" | Naming the gate up front is how you avoid a false-green. |

## Red Flags

- Editing before naming the layer and the gate.
- Treating an FFI change as a pure Rust change (mirror drift).
- Loading every ruleset instead of the layer's.
- Dispatching to no skill at all ("just code").

## Verification (evidence)

- The layer → artifacts → gate mapping is stated in the response.
- The chosen skill(s) exist in the `AGENTS.md` SKILL DIRECTORY.
- The gate output is quoted after the change.

## References

- `context-engineering` — file → language → ruleset routing.
- `skill-router` — task → skill classification when intent is vague.
- `docs/env.md` — toolchain + task reference.
