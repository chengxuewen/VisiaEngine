---
name: lesson-review
description: "Batch session review: systematically extract lessons learned, write them to project memory. Use after long debugging sessions (>1h or >3 failed attempts), when user corrects multiple errors, after build fixes, on '总结经验'/'更新记忆'/'记录教训'/'回顾会话', or via the /lesson-review command."
---

# lesson-review — Batch Lesson Consolidation

## Trigger Conditions

- The user says "总结经验" / "更新记忆" / "记录教训" / "回顾会话" (summarize experience / update memory / record lessons / review the session)
- A long debugging session ends (>1h or >3 failed attempts)
- After the user points out multiple errors
- After solving a problem that took >30min
- The `/lesson-review` command

## Relation to Rules and Skills

```
think-before-act  →  [action]  →  lesson-memory  →  doc-audit
   (consult)                      (instant capture)     (periodic audit)
                    lesson-review ← batch gap-fill
```

| | `lesson-memory` rule (C9) | `lesson-review` skill |
|---|---|---|
| Timing | instant (after every error, auto-triggered) | batch (session end / user-triggered) |
| Method | reflex | interactive review |
| Granularity | single lesson | full-session scan |
| Complement | first line of defense (miss nothing) | second line of defense (no misjudgment + categorization) |

- `think-before-act` (consultation): check existing lessons before acting, avoid repeats
- `lesson-memory` (C9 rule): write **instantly and automatically** after every error, no user request needed
- `lesson-review` (this skill): **batch review** at session end, fill gaps, categorize and archive
- `doc-audit` (audit): periodically check memory-file self-consistency

## Flow

### Step 1: Scan the session

Review which moments in this session triggered the rule but may have been missed:

- compile/build failures
- >1 failed attempt before locating the root cause
- user corrections of approach/preference
- surprise discoveries ("原来如此" / "没想到" — so-that's-how / didn't-expect-that)
- problems that took >30min
- post-edit syntax corruption (unbalanced braces, duplicate lines, etc.)

### Step 2: Extract one by one (5-question checklist)

For each finding, complete:

1. **What went wrong?** (symptom description)
2. **Why did it go wrong?** (root-cause analysis)
3. **What's the right way?** (solution)
4. **How to prevent it?** (check command / constraint)
5. **Where to store it?** (per the storage-target table)

### Step 3: Write

Write using the target file's template format.

- Important lessons → full template (symptom + root cause + solution, three elements)
- Trivial fixes (1 line / no branch / no side effects) → 1-line capture, no full template

### Step 4: Verify

- [ ] every pitfall has a verify field (check command — how to confirm the fix)
- [ ] every convention is grep/lint verifiable
- [ ] no duplicates with existing entries (confirm by grepping the target file)
- [ ] decisions.md numbering is contiguous (no gaps)
- [ ] cross-references correct (e.g. pitfalls citing conventions)
- [ ] high-frequency/high-cost lessons flagged for CI-gate upgrade if needed

## Output Format

```markdown
## Session lessons summary — YYYY-MM-DD

### Recorded (N items)
1. [title] → pitfalls.md
2. [title] → conventions.md
...

### Not recorded (no need)
- [reason: one-off issue / already recorded / environment-specific]

### Upgrade suggestions
- [a lesson suggests adding a CI gate, reason: repeated 3+ times / cost >1h]
```

## Storage targets (per project config)

Applies to any project; just adjust the paths:

```
- technical pitfalls → {pitfall_log}      # VisiaEngine: .agents/memorys/pitfalls.md
- dev constraints    → {conventions}       # VisiaEngine: .agents/memorys/conventions.md
- architecture decisions → {decisions}     # VisiaEngine: .agents/memorys/decisions.md
- runnable checks  → {checks}              # VisiaEngine: .agents/rules/common/edit-safety.md
- testing rules    → {test_rules}          # VisiaEngine: .agents/rules/common/testing.md
- security rules   → {security_rules}      # VisiaEngine: .agents/rules/common/security.md
- Rust coding      → {rust_coding_style}   # VisiaEngine: .agents/rules/rust/coding-style.md
- project status   → {status}              # VisiaEngine: .agents/memorys/status.md
```

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "the session was fine, nothing to record" | Missed lessons are re-learned expensively; scan for corrections. |
| "record it in chat only" | Chat is ephemeral; memory files are the durable store. |
| "one big lesson entry" | Atomic entries are retrievable; one blob is not. |

## Red Flags

- A repeated mistake with no PIT entry.
- Lessons written in chat but not in `.agents/memorys/`.
- A vague lesson ("be careful") instead of a concrete rule.

## Verification (evidence)

- The new C/D/PIT entry exists and is referenced from the session summary.
- `agent-sync` picks it up; `gate-agents` green.
