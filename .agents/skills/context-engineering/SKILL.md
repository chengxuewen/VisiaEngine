---
name: context-engineering
description: "Route language-specific rules to the right code in a multi-language workspace. Loads Rust rules for .rs tasks, FFI/native rules for binding boundaries, protocol rules for wire-contract work, by file extension and module domain. Prevents wrong-language lint violations and out-of-scope analysis. Use BEFORE any cross-module or multi-language task."
---

# context-engineering — Right Context, Right Language

> Route language-specific rules to the right module. Don't apply C++ lints to Rust code. Don't audit Python scripts for Rust ownership semantics.

## When to Use

| Trigger | Reason |
|---------|--------|
| Task touches 2+ languages | Avoids wrong-language rule pollution |
| New module/package being created | Establishes correct context boundaries |
| FFI/SDK work (napi-rs, cbindgen, FlatBuffers, bindings of any kind) | Multi-language contract, not single-lang |
| Web/dashboard work | HTML/CSS/JS + backend, different rulesets |
| Code review spanning modules | Each module needs its own rule set applied |

## Scope → Language → Ruleset Routing

Do NOT keep a hardcoded project map here. Derive the routing each time:

1. **List the files the task touches** (git diff / planned edits).
2. **Group by language** (file extension: `.rs` / `.ts` `.tsx` / `.py` / `.c` `.cpp` / `.sh` / `.yaml` …).
3. **Map each language to its ruleset directory** under `.agents/rules/` (e.g. `rules/rust/` for `.rs`, `rules/common/` always).
4. **Flag boundary files** — FFI shims, protocol/schema definitions, embedded web assets — these need the contract rules in addition to the language rules.

Once the project defines its module matrix, record the concrete mapping in `.agents/memorys/` — not in this skill.

## Context Selection Protocol

### Phase 1: Identify the Scope

Before loading any rules, map the task to modules and languages:

```
Task → files touched → languages involved → applicable rulesets
```

**Example:**
```
Task: "Add a new WS message type"
→ shared protocol module + server handler (both .rs) + dashboard JS (if it consumes the message)
→ Rust: rules/rust/* ; protocol discipline: api-interface-design skill
→ JS side: rules/common/* + wire-format check against the Rust serde config
```

### Phase 2: Apply Rules Per Module

**NEVER** batch-apply all rules to all files. Each file gets its relevant subset:

```
for each file in scope:
  - Load rules/<language>/* matching the file extension
  - Load rules/common/* (security, coding-style, testing) always
  - Skip language-specific hooks for non-matching files
    (e.g. rules/rust/hooks.md must NOT fire on .ts files)
  - Apply platform/constraint rules only where relevant
```

### Phase 3: Verify Per Module

| Scope | Verification |
|-------|-------------|
| Any .rs change | `cargo clippy -p <crate> -- -D warnings` |
| FFI boundary change | `cargo check -p <crate>` with the relevant feature enabled + compile the native side |
| Web/dashboard change | Type-check (`npx tsc --noEmit` if TS) + visual QA via Playwright |
| Protocol/schema change | Serde roundtrip tests + E2E script |
| Full workspace | The project's aggregate check task (define in memorys once the stack exists) |

## Common Context Mistakes

| Anti-Pattern | Why Wrong | Fix |
|-------------|-----------|-----|
| Loading all language rulesets for a single-language task | Wastes context, confuses agent | Route by file extension (see Routing) |
| Applying C++ ownership rules to Rust code | Wrong paradigm | Rust borrow checker is the authority |
| Skipping platform constraint rules for FFI work | Misses linker/toolchain flags documented there | Always include constraints for FFI |
| Applying `rules/rust/hooks.md` to TypeScript files | Wrong hooks fire | Only load per-language hooks |
| Running the full workspace test suite for a single-module change | Slow | Scope to the affected package/crate |
| Not loading the project pitfalls memory for protocol/FFI work | Re-invents known wire-format and boundary bugs | Check `.agents/memorys/pitfalls.md` first |

## Multi-Language Boundary Rules

### Native (Rust) → C/C++ FFI
- Only plain bytes (`&[u8]`) and POD structs across the FFI boundary
- `unsafe` blocks require `// SAFETY:` comment
- Shared-pointer types need explicit thread-safety certification

### Rust → Protocol (WebSocket JSON)
- All messages use a declared serde tag convention (e.g. `#[serde(tag = "type", rename_all = "snake_case")]`)
- Browser clients must emit the exact same tags — verify both sides of the wire format
- Protocol changes go through the contract-first process (`api-interface-design`)
- E2E tests must validate new message types

### Backend → Embedded Web UI
- Decide the embedding strategy early (embedded assets vs separate deploy) and record it in project memory
- No shared type system across the boundary — validate the JSON envelope server-side

## Gate Checklist

Before submitting work touching multiple languages/modules:

```
[ ] file scope mapped, languages identified
[ ] only relevant rulesets loaded per file
[ ] per-module check passes (not just whole-workspace)
[ ] lints pass per module with -D warnings (Rust) / equivalent
[ ] platform constraints checked for gotchas
[ ] project pitfalls memory checked for known boundary patterns
[ ] protocol changes have backward-compat review
```

## Related Skills

| Skill | Relationship |
|-------|-------------|
| `think-before-act` | Context selection IS part of its "investigate before acting" discipline |
| `api-interface-design` | Context-engineering informs protocol contracts |
| `test-harness` | Cross-module testing needs correct per-module context |
| `lesson-memory` (C9) | New multi-language pitfalls → write to `pitfalls.md` |

## Boundary: skill selection belongs to `skill-router`

This skill routes **language rules to modules** — which `.agents/rules/<lang>/` files apply to which files. It does **not** recommend skills for a task.

- For "what skill should I use / 怎么做 / 帮我选择", use `skill-router` (the single source for task → skill classification).
- For the actual skill inventory, read the SKILL DIRECTORY in `AGENTS.md`.

Keeping one routing table (in `skill-router`) avoids two drifting tables with stale names.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "load all rules, be safe" | Wrong-language rules pollute and mislead; route by extension. |
| "one language, skip routing" | FFI/protocol files are multi-language by nature; check boundaries. |
| "the rules don't matter for a small change" | Small changes still cross a module boundary. |

## Red Flags

- Applying `.rs` rules to `.ts` files (or vice versa).
- A single-language task loading every ruleset.
- Missing the boundary files (FFI shims, schema) in the ruleset map.
- No per-module check, only a whole-workspace run.

## Verification (evidence)

- The file → language → ruleset map for the task.
- Per-module check output (e.g. `cargo clippy -p <crate> -- -D warnings`, `tsc --noEmit`).
- Boundary files explicitly listed with their contract rules.
