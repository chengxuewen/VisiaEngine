---
name: openspec-propose
description: "Propose a new change with structured artifacts (proposal, design, tasks). Generates .sisyphus/plans/<name>/proposal.md + design.md + tasks.md. Use when the user describes what they want to build and needs a complete proposal ready for implementation."
license: MIT
compatibility: Designed for Claude Code, GitHub Copilot, and similar agents.
disable-model-invocation: false
metadata:
  author: openspec
  version: "2.0"
  category: workflow
---

# OpenSpec Propose

Create a structured change proposal. Produce three artifacts that together answer
"what are we building, how does it fit, and what's the plan?"

When ready to implement, follow with `/openspec-apply`.

---

**Input**: The user describes a feature, fix, or refactor. Do not start without a feature description.

---

## Steps

### 1. Confirm the change name

Ask: "What should we call this change? (kebab-case, e.g. `add-monitoring-panel`)"

**DO NOT auto-generate without asking.** Validate: lowercase letters, digits, hyphens only.

### 2. Gather context

Before writing any artifact, understand the existing surface area:

#### a. Read relevant specs

Search `openspec/specs/` for specs whose module overlaps with the change. Read every matching spec. Note if no relevant spec exists.

#### b. Read project memory

- `.agents/memorys/status.md` — current phase, module status, known gaps
- `.agents/memorys/decisions.md` — architecture decisions and rationale
- `.agents/memorys/pitfalls.md` — known sharp edges and gotchas
- `.agents/memorys/conventions.md` — naming, immutability, language conventions

#### c. Assess affected layers

Probe the actual source tree to discover the layer map (it may not exist yet in a young project — if it doesn't, that's the answer: "greenfield").

| Layer | Location | When affected |
|-------|----------|---------------|
| **Shared core** | (types, traits, primitives) | New cross-cutting types or contracts |
| **Backend / services** | | Logic and handler changes |
| **Frontend / UI** | | Interface changes |
| **Schema / bindings** | | New or changed data formats |

Fill each with the real path from the tree, or drop the row if the layer doesn't exist.

### 3. Create the proposal directory

```bash
mkdir -p .sisyphus/plans/<change-name>
```

### 4. Write proposal.md

Create `.sisyphus/plans/<change-name>/proposal.md` with these sections:
- **What** — 2-4 sentences, specific
- **Why** — problem, use case, gap
- **Scope** — in scope / out of scope
- **Layers Affected** — checklist filled from step 2c
- **Existing Specs** — list `openspec/specs/<name>.md` with one-line description each
- **New Specs Needed** — list or "None"
- **Risks** — 2-4 bullet points (thread safety, boundaries, build, interop)
- **Success Criteria** — how we know it's done
- **References** — links to issues, design docs, external references

### 5. Write design.md

Create `.sisyphus/plans/<change-name>/design.md` with these sections:
- **Architecture** — ASCII diagram or text description showing modules, data flow, ownership
- **Files to Touch** — Create / Modify / Delete sub-tables with file paths and purpose
- **Data Flow** — critical path from entry to exit (request → handler → state → response)
- **Integration Points** — the module/schema/UI boundaries the change crosses
- **Stack Specifics** — language- or framework-specific concerns for the layers touched (types, concurrency, error model)
- **Error Handling** — how errors are typed, propagated, and surfaced
- **Testing Strategy** — checklist: unit, integration, end-to-end, plus any project QA gate
- **Dependencies** — new third-party deps or schema changes (or "None")

### 6. Write tasks.md

Create `.sisyphus/plans/<change-name>/tasks.md`. Tasks must be **atomic, ordered, independently testable** — each produces one verifiable result. Structure in phases (adapt phase names to the layers actually affected):

```markdown
# Tasks: <Change-Name>

## Phase 1: Foundation

- [ ] **Add `<type/contract>` to shared core**
  - File: `<path>/<file>`
  - Verify: `<compile/type-check command for that module>`

## Phase 2: Implementation

- [ ] **Implement handler / logic in backend**
  - File: `<path>/<file>`
  - Verify: `<compile + test command>`

## Phase 3: Wiring & UI

- [ ] **Wire up in frontend**
  - File: `<path>/<file>`
  - Verify: `<type check>` + browser/runtime verify

## Phase 4: Tests

- [ ] **Unit tests** (AAA pattern)
  - File: same as implementation
  - Verify: `<test command>`

- [ ] **Integration / end-to-end tests**
  - File: `tests/<name>_test`
  - Verify: `<integration test command>`

- [ ] **Run project QA gate** (if any)
  - Verify: `<qa script>`

## Phase 5: Documentation & Cleanup

- [ ] **Write/update spec file**
  - File: `openspec/specs/<name>.md` (SDD format: ID→precondition→operation→expected→edge cases)

- [ ] **Update project memory** (after implementation)
  - `.agents/memorys/status.md`, `decisions.md`, `pitfalls.md` as applicable
```

Adjust phases to fit the change: single-file fix → 3 tasks; multi-layer feature → 15+ tasks across 5 phases. Drop phases for layers the project doesn't have.

### 7. Present and iterate

Display summary — change name, artifact list, line counts. Let user request changes, iterate until approved.

---

## File Path Conventions

| Purpose | Path |
|---------|------|
| Specs | `openspec/specs/` |
| Plans | `.sisyphus/plans/<change-name>/` |
| Project memory | `.agents/memorys/` |

Source-code paths come from the tree probe in step 2c — never hardcode a layout the project doesn't have.

---

## Guardrails

- **Always ask for the change name** — do not generate one without user confirmation
- **Read specs before proposing** — ignoring existing SDD contracts is waste
- **Layer assessment must be explicit** — "maybe affects a layer" is not acceptable; decide and document
- **Tasks must be atomic** — each task produces one verifiable result (compiling code, passing tests)
- Always reference actual file paths from the tree probe — never invent a structure
- If context is critically unclear, ask — but prefer reasonable decisions to keep momentum
- If a proposal with that name already exists, ask to continue or create new
- Do NOT propose changes to version files — versioning is user-managed
- Do NOT propose changes to external dependencies — separate repositories
- Verify each artifact file exists after writing before proceeding
