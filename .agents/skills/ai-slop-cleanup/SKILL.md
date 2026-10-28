---
name: ai-slop-cleanup
description: "Remove AI-generated noise from this repo's code and docs: comments that restate the code, generic summaries, hedging, duplicate explanations, and empty abstractions. Use when a diff reads like generated filler, before a PR, or when a reviewer says the change is noisy."
---

# AI-Slop Cleanup (this workspace)

> AI writes fast and pads. This skill removes the pad, not the meaning.

## Overview

- "Slop" is text that survives review but adds no information: comments restating the next line, "This function does X" docs, generic summaries, hedging ("simply", "just", "basically"), duplicate explanations, and abstractions introduced for a single call site.
- Boundary with `code-simplification`: that skill reduces **structural** complexity (functions, modules, control flow); this one removes **generated text noise** around correct code.
- Never delete a comment that explains *why* (an invariant, a workaround, a SAFETY note). Delete only what explains *what the code already says*.

## When to Use

- A diff or doc reads like generated filler, or a reviewer flags it as noisy.
- Before opening a PR / after a large AI-assisted edit.
- A file has more comment than code, or repeated paragraphs.

## Process (steps + checkpoints + exit)

1. **Read the diff** (`git diff`) and mark candidates: restating comments, generic summaries, hedging, duplicates, one-use abstractions.
2. **Classify each candidate**: *why-comment* (keep) vs *what-comment* (remove) vs *padding* (remove).
3. **Remove only the safe class** — never touch `// SAFETY:`, TODOs with an issue link, or protocol/ABI notes.
4. **Checkpoint**: the diff shrinks, behavior is unchanged, and every removed line was non-informative.
5. **Verify**: tests/`cargo check` unchanged; markdown links still resolve; no `#[allow]` added.
6. **Exit**: the diff is smaller and more informative, with the removed-line rationale stated.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "more comments are better" | A comment that repeats the code is noise; it can drift and mislead. |
| "keep the summary, it's harmless" | Harmless text still costs context and invites stale claims. |
| "delete all comments" | Why-comments are load-bearing; remove only what-comments. |
| "refactor while cleaning" | Mixed changes hide both; clean text first, then restructure. |

## Red Flags

- Comments restating the next line (`// increment i`).
- Generic "This module handles X" headers with no specifics.
- Hedging filler ("basically", "simply", "just") in docs.
- Duplicated paragraphs across README/docs.
- An abstraction with exactly one caller and one method.

## Verification (evidence)

- `git diff --stat` shows a smaller change; behavior unchanged (tests/gate green).
- Each removed comment is listed with its class (what-comment / padding).
- No `// SAFETY:` / TODO-with-link / ABI note removed.

## References

- `.agents/skills/code-simplification/SKILL.md` — structural complexity (different axis).
- `.agents/rules/common/coding-style.md` — comment guidance.
