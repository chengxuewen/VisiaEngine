---
name: subagent-orchestration
description: "Orchestrate subagents for this repo: decide parallel vs serial, define isolated scopes, merge results, and avoid duplicated exploration. Use when a task spans many files/areas, needs independent verification, or you are about to launch more than one subagent."
---

# Subagent Orchestration (this workspace)

> Subagents buy isolation and parallelism — and cost coordination. Spend them deliberately.

## Overview

- A subagent gets a **fresh context** and a bounded scope; it is good for broad exploration (`code-explorer`) and independent verification, bad for a task that needs your live conversation state.
- Parallel is only safe when scopes **do not overlap** and neither depends on the other's output.
- The main agent keeps the synthesis: never relay a subagent's raw dump; extract findings + evidence.

## When to Use

- A task touches many files/areas and reading them all would flood your context.
- You need a second, independent read of a risky hunk (unsafe/FFI/concurrency).
- You are about to launch 2+ subagents and need to partition the work.

## Process (steps + checkpoints + exit)

1. **Decide if a subagent is warranted**: broad exploration or independent verification. If a single `read_file`/`search_content` answers it, do it yourself.
2. **Write a self-contained brief**: the exact question, relevant paths, thoroughness level, and the read-only/edit scope.
3. **Partition for parallelism**: split by disjoint files/areas; if two tasks share a file or depend on each other, run serially.
4. **Checkpoint**: each brief can be understood by someone who did not see the chat.
5. **Merge**: dedupe, cross-check agreement, and keep absolute paths + short evidence; do not paste raw output.
6. **Verify**: the main agent re-checks the highest-risk claim before acting.
7. **Exit**: findings are attributed, non-overlapping work was parallel, and the synthesis is the main agent's.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "launch 5 agents, it's faster" | Overlapping scopes duplicate work and race; partition first. |
| "the brief can be vague, it has context" | A subagent has no chat history; vagueness returns noise. |
| "just paste its output" | Raw dumps flood context; synthesize with paths + evidence. |
| "parallel is always better" | Dependent tasks must serialize; parallel here is a bug. |
| "skip the second read" | The independent read is the point on risky hunks. |

## Red Flags

- Two subagents assigned the same file/area.
- A brief without paths or a clear question.
- A subagent given write access it does not need.
- Findings relayed without evidence.
- Dependent tasks launched in parallel.

## Verification (evidence)

- Each brief is self-contained (question + paths + scope).
- Scopes are disjoint (or the run was serial).
- The synthesis cites absolute paths + short evidence, and the riskiest claim was re-checked.

## References

- The host's subagent mechanism (this repo uses `code-explorer` / `Task`).
- `.agents/skills/planning-and-task-breakdown/SKILL.md` — slicing the work first.
- `.agents/skills/think-before-act/SKILL.md` — the decision gate before acting.
