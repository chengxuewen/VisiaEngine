---
name: documentation-and-adrs
description: "Record architecture decisions and keep docs consistent in this repo: D-series entries in .agents/memorys/decisions.md, architecture doc updates, and the sync/gate loop. Use when a non-trivial design decision is made, an architecture doc needs updating, or docs drift from reality."
---

# Documentation & ADRs (this workspace)

## Overview

- This repo records decisions as **D-series entries in `.agents/memorys/decisions.md`** (English, C17) — there is **no separate `docs/adr/` tree** and there should not be one (one source of truth).
- Architecture narrative lives in `docs/architecture.md`; environment/tasks in `docs/env.md`. `AGENTS.md` is the generated hub (memory digest + skill directory).
- Edits to memory go through `agent-sync` (the hub regenerates); `gate-agents` asserts consistency.

## When to Use

- A non-trivial design decision is made (architecture, contract, tooling, process).
- `docs/architecture.md` no longer matches the code.
- A decision is superseded and the old entry is now misleading.

## Process (steps + checkpoints + exit)

1. **Decide if it is a real decision** (affects future behavior / is hard to reverse). If yes, continue; if it is a trivial choice, don't record it.
2. **Write the D-entry** in `.agents/memorys/decisions.md`:
   ```
   ## D<N>: <title> (YYYY-MM-DD)

   Context: why this came up.
   Decision: what was chosen.
   Consequences / Status: what it implies, and any deferred items.
   ```
   (Numbering is contiguous; the next N is the last + 1.)
3. **Update the architecture doc** (`docs/architecture.md`) if the decision changes the described system.
4. **Checkpoint**: the decision is findable by grepping `decisions.md`; `docs/architecture.md` no longer contradicts it.
5. **Regenerate + gate**: `pixi run agent-sync` then `pixi run gate-agents`.
6. **Exit**: the D-entry exists, the doc is consistent, and the gate is green — with evidence.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "I'll remember why" | Session memory ends; the D-entry is the durable record. |
| "create a new docs/adr/ folder" | That forks the decision source; use `decisions.md`. |
| "skip the architecture doc" | A code change without a doc update is a contradiction in waiting. |
| "one big entry covers it" | Atomic entries are retrievable and supersedable. |

## Red Flags

- A non-trivial decision with no D-entry.
- A parallel ADR tree created next to `decisions.md`.
- `docs/architecture.md` describing behavior the code no longer has.
- A superseded decision left without a status note.

## Verification (evidence)

- `grep "^## D<N>" .agents/memorys/decisions.md` returns the new entry.
- The `docs/architecture.md` diff reflects the change.
- `pixi run gate-agents` green (hub regenerated).

## References

- `.agents/memorys/decisions.md` — the decision log.
- `docs/architecture.md`, `docs/env.md`.
- `doc-audit` — consistency audit across docs.
