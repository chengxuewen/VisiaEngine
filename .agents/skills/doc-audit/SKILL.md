---
name: doc-audit
description: "Project documentation and architecture audit. Checks self-consistency, completeness, gaps, and optimization opportunities across architecture docs, design docs, and decision records in parallel. Confirms every finding with the user interactively, item by item, listing details, options with pros/cons, sources, impact, recommendation. Supports team mode (large audits) and background-agent mode (lightweight checks)."
---

# Document & Architecture Audit

Full audit of the project documentation system: doc consistency, decision verification, agent infrastructure, reference benchmarking.

**Philosophy**: an audit is not nitpicking — it pays down debt. Documentation debt is as dangerous as code debt.

---

## Entry: audit type

### `/doc-audit` (no arguments)
Pops up the audit-type menu:

```
[1] Full audit — all 5 dimensions (default)
[2] Decision verification — how well decisions.md landed
[3] Doc consistency — arch ↔ modules ↔ sub-docs cross-check
[4] Agent system — .agents/ rules/skills/memory self-consistency
[5] Gap optimization — security/reliability/ops sweep
[6] Phase audit — Phase progress vs reality
```

### `/doc-audit full`
Starts the full audit directly, skipping the menu. Equivalent to `[1] Full audit`.

### `/doc-audit quick-fix`
Checks only LOW/MEDIUM issues and fixes them automatically, without entering interactive review.

---

## Audit dimensions

### 1. Decision verification
Verify how decisions D1-Dx in `decisions.md` landed in the architecture docs, implementation code, and status records.

**Core questions**:
- Is each decision's conclusion correctly reflected in architecture.md and the module docs?
- Any "decision says A, reality is B" contradictions?
- Do stale references inside decisions need updating?
- **Decision freshness check**: has a later decision superseded it? Are tech-stack versions outdated?

### 2. Doc consistency
Verify the cross-consistency of architecture.md ↔ modules/ ↔ README ↔ AGENTS.md.

**Core questions**:
- Is the same concept described consistently? (crate counts, Phase status, tech-stack versions)
- Do sub-docs duplicate the main doc?
- Are Phase terms ambiguous?
- Are crate names consistent with the C4 convention?

### 3. Agent-system audit (new)
Verify self-consistency among the rules, skills, and memory under .agents/.

**Core questions**:
- conventions.md numbering contiguous? C{n} cross-references valid?
- decisions.md numbering contiguous? D{n} ordering correct?
- pitfalls.md numbering contiguous? PIT-{n} cross-references complete?
- Does the skill directory table (AGENTS.md) match the actual skill list?
- Does the task-routing table (context-engineering) match the actual skill list?
- Do all opencode.json instructions references exist?
- Is every skill's SKILL.md frontmatter complete?

### 4. Gap optimization
Scan for missing key documents/design sections.

**Core questions**:
- Security architecture: are key rotation, token-expiry policy, transport security documented?
- Ops/observability: health checks, metrics export, log aggregation?
- Error model: SFU connection failure, encode degradation, transport-disconnect handling?
- Hardware baseline: minimum CPU/RAM, Docker resource configuration?
- Upgrade strategy: hot updates, crate version migration, config migration?

### 5. Phase audit
Verify Phase progress vs documented claims vs code implementation.

**Core questions**:
- Does status.md's Phase state match git log / test counts?
- Are the decision Phase tags in decisions.md accurate?
- Does the code implementation match the completion state claimed by the docs?

---

## Audit modes

### A. Team mode (recommended — large audits)
3+ large documents → `team_create` with 4-6 members in parallel.

```
team_create(inline_spec={
  name: "doc-audit",
  members: [
    { name: "decision-validator", category: "deep", prompt: "<decision-verification core questions>" },
    { name: "consistency-checker", category: "deep", prompt: "<doc-consistency core questions>" },
    { name: "agent-auditor", category: "deep", prompt: "<agent-system core questions>" },
    { name: "gap-optimizer", category: "deep", prompt: "<gap-optimization core questions>" }
  ]
})
```

**Conductor rules** (orchestrator behavior):
- Right after launch, report to the user: "launched N parallel audit lanes, expect 3-5 minutes"
- While waiting for all lanes, do only "non-overlapping work"
- After all complete: **dedupe and merge** (same issue found by 2+ dimensions → merge into 1 item)
- Sort by severity: CRITICAL → HIGH → MEDIUM → LOW
- Timeout handling: any lane producing nothing for over 10 minutes → mark as "timed out"
- Conflict handling: dimension A says X, dimension B says Y → flag for human review

### B. Background-agent mode (lightweight audits)
Few documents → `task(category="deep", run_in_background=true)` × N in parallel.

### C. Single-thread mode
Tiny scope → check directly with Read/Grep, no subagents.

---

## Interactive review: finding format

**Review item by item**, presenting each finding with the `question()` tool.

```markdown
## 🔴/🟠/🟡/🔵 [number]: [title]

### Details
| Source1 | Location | Content |
|--------|------|------|
| Doc A | line X | ... |
| Doc B | line Y | ... |

### Options
| Option | Pros | Cons |
|------|------|------|
| A. [option name] | ... | ... |
| B. [option name] | ... | ... |

### Recommendation
[Option X]. [reason]
```

Choices: adopt the recommendation / pick another option / won't-fix / custom
Progress: `[item N of M]`

---

## Workflow

### Phase 1: Launch
1. Confirm audit scope and type
2. Choose the mode (team/background/single-thread)
3. Report: "launched N parallel audit lanes"

### Phase 2: Merge
1. Dedupe: same issue from multiple sources → merged with annotations
2. Sort: CRITICAL → HIGH → MEDIUM → LOW
3. Cross-corroboration: issues agreed on by 2+ dimensions get priority raised

### Phase 3: Interactive review
Review item by item with question() interactive confirmation.

### Phase 4: Fix
1. Create the todo list
2. In dependency order: fix decisions first → then docs → status last
3. Verify after every edit

### Phase 5: Report
```
Audit complete — [date]
Audit type: [full/decisions/consistency/agent-system/gaps/phase]
Findings: N | fixed: M | won't-fix: K
Next suggestion: [issue-dense area]
```

---

## Severity criteria

| Severity | Trigger | Blocking? |
|--------|---------|:---:|
| 🔴 CRITICAL | doc contradiction that caused an implementation error / decision overturned / core API missing | ✅ |
| 🟠 HIGH | stale reference / Phase ambiguity / duplicated docs / numbering holes | ⚠️ |
| 🟡 MEDIUM | wording divergence / conflicting examples / gap that doesn't block the current phase | ❌ |
| 🔵 LOW | formatting inconsistency / missing citation / to-be-confirmed marker | ❌ |

## Audit cadence
- After every D# decision change: `/doc-audit decisions`
- Before a Phase transition: `/doc-audit full`
- Weekly during development: `/doc-audit full`
- After any major doc change: `/doc-audit consistency`

---

## Community references

| Precedent | Pattern borrowed |
|------|-----------|
| [large-codebase-audit](https://github.com/MJWNA/large-codebase-audit-skill) | 9-surface AI-layer audit, aligned with Anthropic best practices |
| [claude-ecosystem](https://github.com/melodic-software/claude-code-plugins) | meta-skill architecture, 16 audit agents |
| [agent-self-audit](https://github.com/Xxt-XN/agent-self-audit) | two-tier design, 13 checks, auto-escalation |

---

## Division of labor with ecosystem-scan

| Dimension | ecosystem-scan | doc-audit |
|------|:---:|:---:|
| Agent infrastructure (skills/rules/MCP) | ✅ specialty | ✅ dimension 3 |
| External community comparison | ✅ core capability | ❌ |
| Internal doc consistency | ❌ | ✅ specialty |
| Decision verification | ❌ | ✅ specialty |
| Security audit gate | ✅ Full mode | ❌ |
