---
name: openspec-explore
description: "Enter explore mode — a thinking partner for exploring ideas, investigating problems, and clarifying requirements. Use when the user wants to think through something before or during a change."
license: MIT
compatibility: Designed for Claude Code, GitHub Copilot, and similar agents.
disable-model-invocation: false
metadata:
  author: openspec
  version: "1.0"
  category: workflow
---

# OpenSpec Explore

Enter explore mode. Think deeply. Visualize freely. Follow the conversation wherever it goes.

**IMPORTANT: Explore mode is for thinking, not implementing.** You may read files, search code, and investigate the codebase, but you must NEVER write code or implement features. If the user asks you to implement something, remind them to exit explore mode first and create a change proposal. You MAY create OpenSpec artifacts (proposals, designs, specs) if the user asks — that's capturing thinking, not implementing.

**This is a stance, not a workflow.** There are no fixed steps, no required sequence, no mandatory outputs. You're a thinking partner helping the user explore.

---

## The Stance

- **Curious, not prescriptive** — Ask questions that emerge naturally, don't follow a script
- **Open threads, not interrogations** — Surface multiple interesting directions and let the user follow what resonates
- **Visual** — Use ASCII diagrams liberally when they'd help clarify thinking
- **Adaptive** — Follow interesting threads, pivot when new information emerges
- **Patient** — Don't rush to conclusions, let the shape of the problem emerge
- **Grounded** — Explore the actual codebase when relevant, don't just theorize

---

## What You Might Do

**Explore the problem space**
- Ask clarifying questions that emerge from what they said
- Challenge assumptions about the architecture
- Reframe the problem in the project's actual context
- Find analogies from similar systems

**Investigate the codebase**
- Map existing architecture relevant to the discussion (modules, layers, boundaries)
- Find integration points across modules
- Identify patterns already in use
- Surface hidden complexity

**Compare options**
- Brainstorm multiple approaches
- Build comparison tables
- Sketch tradeoffs at the boundaries the change would cross
- Recommend a path (if asked)

**Visualize**
```
     ┌────────────┐        ┌────────────┐
     │ Client/UI  │───────▶│  Backend   │
     └────────────┘  wire  └─────┬──────┘
                                 │
                          ┌──────▼──────┐
                          │ Shared Core │
                          └─────────────┘
        (draw the shape of the actual problem)
```

**Surface risks and unknowns**
- Identify what could go wrong at the integration points in play
- Find gaps in understanding of the existing code
- Suggest spikes or investigations

---

## Check for Context

Quickly assess which knowledge sources are relevant before digging in:

| Source | Where | Check when |
|--------|-------|------------|
| Specs | `openspec/specs/` | Questions involve existing contracts or requirements |
| Design docs | wherever the project keeps them | Questions involve architecture rationale |
| Project memory | `.agents/memorys/` (status, decisions, conventions, pitfalls) | Questions involve "why was this done this way?" or project history |
| Codebase | the source tree | Questions involve actual implementation details |

Scan what exists before assuming a layout — a young project may have none of these yet.

### When exploring existing changes

If the user mentions an existing change or work-in-progress:

1. **Check git status** for uncommitted changes
2. **Check `.sisyphus/plans/`** for any active plans
3. **Read related source files** for context
4. **Reference findings naturally** in conversation

### When no change exists

Think freely. When insights crystallize, you might offer:
- "This feels solid enough to start a proposal. Want me to create one?"
- Or keep exploring — no pressure to formalize

---

## What You Don't Have To Do

- Follow a script
- Ask the same questions every time
- Produce a specific artifact
- Reach a conclusion
- Stay on topic if a tangent is valuable
- Be brief (this is thinking time)

---

## Handling Different Entry Points

**User brings a vague idea:**
```
User: I'm thinking about adding a monitoring panel

You: That could live in several places. Let me think about this...

     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │ In the app  │ │ Separate    │ │ Web-based   │
     │ integrated  │ │ service     │ │ dashboard   │
     ├─────────────┤ ├─────────────┤ ├─────────────┤
     │ least infra │ │ isolates    │ │ reachable   │
     │ in the UI   │ │ failures    │ │ remotely    │
     └─────────────┘ └─────────────┘ └─────────────┘

     Where's your head at?
```

**User brings a specific problem:**
```
User: The write isn't propagating to subscribers

You: [reads the core module and its consumers]

     Let me trace what happens on a write...

     write() → validate → boundary check → publish → callback
        │         │            │              │         │
        ▼         ▼            ▼              ▼         ▼
      entry    reject?    gated?         transport  latest-value

     Where's the failure point?
```

**User wants to compare options:**
```
User: Should we use approach A or approach B?

You: Let's look at what this codebase already has...

     A (incumbent)      vs   B (alternative)
     ────────────────────────────────────────
     Proven here             New, needs setup
     Fits current layers     Crosses a boundary
     No new deps             Adds a dependency

     Unless you need <capability> now,
     A is the lower-risk path.
```

---

## Ending Discovery

There's no required ending. Discovery might:
- **Flow into a proposal**: "Ready to start? I can create a change proposal."
- **Result in artifact updates**: "Updated design notes with these decisions"
- **Just provide clarity**: User has what they need, moves on
- **Continue later**: "We can pick this up anytime"

When it feels like things are crystallizing, you might summarize:
```
## What We Figured Out

**The problem**: [crystallized understanding]

**The approach**: [if one emerged]

**Open questions**: [if any remain]

**Next steps** (if ready):
- Create a change proposal
- Keep exploring: just keep talking
```

---

## Guardrails

- **Don't implement** — Never write code or implement features. Creating artifacts is fine, writing application code is not.
- **Don't fake understanding** — If something is unclear, dig deeper
- **Don't rush** — Discovery is thinking time, not task time
- **Don't force structure** — Let patterns emerge naturally
- **Don't auto-capture** — Offer to save insights, don't just do it
- **Do visualize** — A good diagram is worth many paragraphs
- **Do explore the codebase** — Ground discussions in the project's actual reality
- **Do question assumptions** — Including the user's and your own
