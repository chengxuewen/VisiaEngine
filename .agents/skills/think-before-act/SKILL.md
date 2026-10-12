---
name: think-before-act
description: "Meta-constraint: investigate before acting, present options for user approval, do not act rashly. Use BEFORE any non-trivial action (debug/test/implement/refactor/fix/config/upgrade). Especially when facing silent failures, runtime errors not caught by compiler, or behavior that contradicts documentation."
---

# think-before-act — Investigate → List options → User approval → (Team review) → Execute

> Do not act rashly.

## Trigger Conditions

Any **non-trivial** operation: debugging, testing, implementing, refactoring, fixing, config changes, upgrades.
Trigger especially when failure modes are odd: silent failures, runtime errors the compiler cannot catch, behavior that contradicts the documentation.

---

## Decision Gate (ask yourself before every operation)

1. **"What does the official recommendation say?"** → if you don't know, look it up — don't guess.
2. **"Has this project hit this before?"** → check the project knowledge base (.agents/memorys/ / git log).
3. **"Did the community already solve it?"** → search issues / forums / StackOverflow (search the error message verbatim).
4. **"Does the user approve this approach?"** → list the options and wait for approval; don't just edit.
5. **"Why did the last attempt fail?"** → if you can't say, don't try the next one.

---

## Tiers

| Tier | Criteria | Process |
|------|------|------|
| **Trivial** | no interface/build/dependency change, reversible, single file, no side effects | just do it + one-line note |
| **Standard** | touches interface/build/dependencies/multiple files/irreversible/framework-level issues | Phase 1→2→3 |
| **Urgent** | the user explicitly declares "urgent / fix first, report later" (production down, data loss) | do it now + file the report afterward |

---

## Phase 1: Investigation (standard tier must not skip)

### 5-layer knowledge sources (by priority)

1. **Project history**: .agents/memorys/decisions.md / pitfalls.md / git log
2. **Language/toolchain native docs**: cargo doc / rustdoc / --help / man
3. **Framework/tool official docs**: find the recommended practice, not just any search hit
4. **Community experience**: GitHub issues / StackOverflow / forums (search the error message verbatim)
5. **Similar projects**: how open-source implementations on the same stack do it

### Extra checklist for test scenarios

- [ ] project testing conventions (framework? AAA? naming? coverage?)
- [ ] spec/contract documents → extract test scenarios
- [ ] existing test patterns (how is the same module tested? fixtures?)

### Output

An investigation summary: what was found, what the official recommendation is, what the project convention is.

---

## Phase 2: List options (standard tier; auto-execution forbidden)

### Format

```
Option A: [description] — pros / cons / impact scope
Option B: [description] — pros / cons / impact scope
Recommendation: [X], reason: [...]
```

### Code changes must list

- which files will be modified
- change summary (not the full diff)
- potential risks
- rollback method

### Approval

Use the `question` tool and wait for the user's confirmation.
- Silence / timeout / "continue" ≠ approval
- An explicit affirmative response from the user is required
- Partial approval ("do A but not B") → execute only the approved part

### Complex changes: team review

For complex changes (build/architecture changes, framework-level issues, impact on >5 files),
after user approval and before execution, add a team review round:

```
1. team_create — create the review team (3 perspectives)
2. team_task_create — assign review tasks
3. team_send_message — send the plan draft
4. collect review feedback → revise the plan
5. user confirms the revision → enter Phase 3
```

CRITICAL items found in review must be fixed. LOW items may be recorded as technical debt.

---

## Phase 3: Execution (user has approved)

- Execute the option the user chose
- Verify each step (never accumulate unverified changes)
- On failure: roll back + report, **do not auto-try option B** (return to Phase 1 for more investigation)

---

## Core Iron Rules

1. **No new information = no new attempt.** After a failure, retrying with different parameters without new diagnostic information is acting rashly.
2. **Test at the right layer.** Unit (logic) / integration (boundaries) / E2E (user flows) / acceptance (business) — different layers catch different faults. If unit tests pass but the feature is broken, you tested the wrong layer.
3. **Trivial code needs no test.** One line, no branch, no side effects → YAGNI applies to tests too.

---

## Test-layer quick reference

| Layer | Catches | Rust tooling |
|------|----------|-----------|
| Unit | logic errors | `cargo test` |
| Integration | interface mismatch | `cargo test --test '*'` |
| E2E | broken user paths | Playwright / Python WS scripts |
| Acceptance | unmet requirements | manual verification / pixi run test-sfu |

---

## Forbidden list (verify Y before doing X)

| Want to do | Verify first |
|------|--------|
| Change config/build | consulted the official recommended configuration method |
| Retry after a failure | you have new diagnostic information (not parameter-guessing luck) |
| Edit generated files/build artifacts | confirmed the edit won't be overwritten at next build |
| Delete what you don't understand | confirmed it isn't depended on by other modules |
| Edit a file directly | the user has approved the plan |
| Change deps/pixi.toml in production | verify locally first, then change |

---

## Relations to existing skills

| Skill | Relationship |
|------|------|
| `lesson-memory` (C9) | think-before-act consults past lessons; lesson-memory records new ones |
| `lesson-review` | think-before-act is prevention; lesson-review is retrospection |
| `systematic-debugging` | think-before-act governs "look it up before fixing"; debugging governs "how to fix" |
| `verification-before-completion` | think-before-act governs "before acting"; verification governs "after completing" |

## Multi-ecosystem examples

| Ecosystem | Typical "investigate before acting" scenario |
|------|----------------------|
| Rust | lifetime error → first read the corresponding rustbook chapter; don't sprinkle `.clone()` |
| Python | dependency conflict → first check the pip/uv resolution rules; don't repeat `pip install` |
| Go | interface not satisfied → first read the godoc interface contract; don't blindly change signatures |
| JS/TS | duplicated modules → first check the bundler's official recommendation; don't thrash aliases |
| Java | version conflict → first run `mvn dependency:tree`; don't exclude by hand |
| K8s | pod crash → first check the official troubleshooting + `kubectl describe`; don't randomly edit yaml |

---

## Karpathy four-principles mapping

This project's think-before-act skill aligns naturally with Andrej Karpathy's four engineering-philosophy principles:

| Karpathy principle | This skill's counterpart | Note |
|--------------|-----------|------|
| **Think Before Coding** | Phase 1: investigation + 5-layer knowledge sources | check project history, official docs, community experience first; never act on guesses |
| **Simplicity First** | Core iron rule #3 + tiering (trivial/standard) | simple solutions first, YAGNI, no over-design |
| **Surgical Changes** | Phase 2: option list + forbidden list | precise edits, stated impact scope, don't touch unrelated code |
| **Goal-Driven Execution** | Phase 3: execution + decision gate | execute the approved option, verify every step, no drift |

> **Community reference**: [multica-ai/andrej-karpathy-skills](https://github.com/multica-ai/andrej-karpathy-skills) — systematizes Karpathy's engineering philosophy into AI-executable skills, consistent with this skill's design intent.
