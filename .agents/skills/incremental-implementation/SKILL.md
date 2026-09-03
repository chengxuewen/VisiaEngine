---
name: incremental-implementation
description: "Thin vertical slice methodology — implement→test→verify→commit per slice, each leaving the workspace compilable and tests green. Use for multi-file changes spanning modules or layers (any backend↔frontend pair), dependency/build-config changes, or pipeline work needing end-to-end verification. Triggers: 'incremental', 'slice by slice', 'one module at a time', 'one crate at a time', multi-layer refactor."
---

# Incremental Implementation

## Overview

Build in thin vertical slices — one module, one layer, one behavior at a time. Implement → test → verify → commit. Each slice leaves the workspace compilable and tests green. This is how large multi-layer codebases stay manageable: every commit is a working checkpoint, and a regression is bisectable to a single small slice.

## When to Use

- Multi-layer changes (e.g., a new protocol message or type flowing from a shared core module through a backend handler to a frontend UI)
- Any backend↔frontend feature (core type → server handler → client wiring)
- Pipeline changes that must be verified end to end (processing stage → transport → consumer → UI/browser test)
- Any change touching dependency manifests, build configuration, or feature flags

**When NOT to use:** Single-function bugfix confined to one module where a local check is the only verification needed.

## The Slice Cycle

```
Implement → Verify → Test → Commit
    ▲                            │
    └──────── Next slice ────────┘
```

Verification gates depend on what the slice touched (adapt commands to the stack in use):

| Slice Type | Verify (adapt to stack) |
|------------|-------------------------|
| Single module change | module-level type check + its tests |
| Multi-layer change | whole-workspace check + test run |
| Feature-gated change | check + test with each relevant feature combo |
| Frontend change | type check (`tsc --noEmit` or equivalent) + runtime verify in browser/UI |
| Native/platform-bound work | platform-appropriate build + real runtime verification |

### Vertical Slice Example

```
Slice 1: Add the message/type to the shared core module
  → check + test that module → green ✓ → commit

Slice 2: Handle it in the backend
  → check + test that module → green ✓ → commit

Slice 3: Wire it up in the frontend
  → type check + runtime verify ✓ → commit

Slice 4: End-to-end integration test across the wire
  → run the integration suite ✓ → commit
```

### Risk-First Slicing

Prove the riskiest piece first — the unknown integration link, not the well-trodden UI:

```
Slice 1: Riskiest link (new connection/transport path)
  → verify the handshake/completion actually works ✓
Slice 2: Core relay through that link
  → verify end-to-end delivery reaches the consumer ✓
Slice 3: Surface it in the UI
  → verify the real flow renders ✓
```

## Rules

### Rule 0: Feature Flag / Build Variant Awareness

If the project uses feature flags or build variants, each slice must state which variant it touches and verify that variant. A slice that breaks a flagged configuration is a broken slice.

### Rule 1: Layer Boundary Discipline

Don't cross layer boundaries in one slice without verification:

```
// BAD:  one slice that adds a type to shared core AND uses it in the consumer
// GOOD: Slice 1: add to core (test it). Slice 2: use in consumer (test it).
```

### Rule 2: Keep the Workspace Compilable

A workspace-wide check must pass after every slice. If a slice breaks another module, it's too big.

### Rule 3: Document Platform/Toolchain Constraints

If some verification only runs on one platform (native deps, OS-specific features), state the platform alongside the slice.

### Rule 4: Commit Atomicity

Each commit should be independently revertable:

```
feat(core): add <new type/enum>
feat(server): handle <new case>
feat(web): wire <new behavior> in the UI
```

## Verification Checklist

After each slice, verify with the commands for what changed:

- [ ] Type check of the changed module/package passes
- [ ] Tests of the changed module/package pass (all green)
- [ ] Linter clean on changed files
- [ ] Feature-gated / build-variant configs checked with all relevant combos
- [ ] Frontend changes: type check passes
- [ ] UI changes: runtime/browser verification (console clean, no errors)
- [ ] Infra changes: container/stack starts successfully
- [ ] Commit with a conventional commit message

## Slice Size Limits

| Metric | Max | Red Flag |
|--------|-----|----------|
| Lines per slice | 150 | > 200 = split |
| Layers touched | 2 | > 2 = too wide |
| Files changed | 5 | > 5 = slice deeper |
| Time to verify | 60s | > 2 min = too much |

## Common Rationalizations

| Rationalization | Reality |
|---|---|
| "I'll test all layers at the end" | A type change in shared core cascades. Test each layer that changed. |
| "The type check is enough, skip tests" | Type checks catch types, not logic. Run tests on what changed. |
| "It's a small UI change, skip runtime verify" | Compiles ≠ renders. Runtime verification catches breakage. |
| "The native part only builds on one platform, I'll test later" | At minimum, run a check of that configuration now. |
| "I'll add the feature flag later" | If Slice 1 breaks a flagged config, later slices are built on sand. |

## Red Flags

- > 150 lines in one commit
- Workspace check broken between slices
- Unverified UI changes ("looks right in code")
- Pipeline changes not exercised against the real runtime
- Feature-flag / build-variant combinations not checked
- Platform-dependent work verified only on another platform

## See Also

- `.agents/memorys/pitfalls.md` — known sharp edges for this project
- `.agents/memorys/conventions.md` — project conventions and boundaries
- `.agents/rules/` — per-language coding style rules
