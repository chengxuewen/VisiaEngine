---
name: ecosystem-scan
description: "Audit the .agents/ system and scan the community ecosystem for adoptable skills/rules/MCP. Two-tier (Quick/Full) + quality scoring + security gate. Use when the user asks to 'optimize agents', 'scan ecosystem', 'find new skills', 'audit .agents/', or '/ecosystem-scan'."
---

# ecosystem-scan — Project Agent-System Audit + Ecosystem Scan

> Two-tier scan (Quick daily / Full deep), quality scoring, security-audit gate.
> Reference precedents: [autoskills](https://github.com/B143KC47/autoskills), [agent-skill-discovery](https://github.com/ericgandrade/claude-superskills), [skill-update-team](https://github.com/franktsai2008-eng/skill-update-team), [agent-self-audit](https://github.com/Xxt-XN/agent-self-audit).

## Trigger Conditions

- The user says "优化 agents" / "审计 .agents" / "扫描生态" / "找新技能" (optimize agents / audit .agents / scan ecosystem / find new skills)
- `/ecosystem-scan` — pops up the mode-selection menu
- `/ecosystem-scan quick` — fast scan (default)
- `/ecosystem-scan full` — deep scan (auto-escalated on first run or after >5 Quick runs)
- `/ecosystem-scan report` — view the last scan report

## Entry Menu

With no arguments, pop up:

```
[1] Quick Scan — 8-item fast audit (< 30s)
[2] Full Scan — 15-item deep audit + community sync + security gate (3-5min)
[3] View Report — view the last scan results
[4] Quick Fix — auto-fix LOW/MEDIUM issues (non-interactive)
```

## Two-Tier Modes

| Feature | Quick | Full |
|------|:-----:|:----:|
| Local audit | 8 fast checks | 15 deep checks |
| External scan | 3-5 known high-star repos | comprehensive websearch + GitHub search |
| Scoring | 3-dimension fast | 5-dimension full scoring |
| Security audit | none | 6 safety checks |
| Duration | < 30s | 3-5min |
| Trigger | `/ecosystem-scan` | `/ecosystem-scan full` or >=5 Quick runs |

---

## Phase 1: Quick Scan (default)

### 1A: Local fast audit (8 items)

```
1. all files referenced by opencode.json instructions exist
2. every SKILL.md has name + description frontmatter
3. conventions/decisions/pitfalls numbering contiguous
4. memorys cross-reference integrity
5. duplicate-content detection (grep key paragraphs)
6. orphan-file detection (present but never referenced)
7. skill-directory-table sync: AGENTS.md SKILL DIRECTORY covers every skill
8. task-routing-table sync: context-engineering routing table matches the actual skill list
```

### 1B: External fast scan

Search 3-5 known high-star repos:
- VoltAgent/awesome-agent-skills (index)
- addyosmani/agent-skills (engineering skills)
- ECC/affaan-m everything-claude-code (full-stack configs)

Fetch READMEs with `webfetch`; 3-dimension quick scoring: tech-stack fit / already-present-in-project / quality.

### 1C: Quick output

```
🟢 No issues — 3 checks passed, no deep scan needed
🟡 Found N items worth attention — recommend /ecosystem-scan full
```

---

## Phase 2: Full Scan

### 2A: Local deep audit (15 items)

| # | Check | Quick | Full |
|---|--------|:---:|:---:|
| 1 | config-file health | counts | 5-dimension scoring + compression/split suggestions |
| 2 | skill inventory | counts | duplicate detection + community comparison |
| 3 | security | plaintext secrets | permission audit |
| 4 | memory system | counts | staleness + structure |
| 5 | rule quality | — | runnable-command check |
| 6 | update availability | — | changelog + priority |
| 7 | skill utilization | — | usage rate vs install count |
| 8 | orphan recovery | — | recovery candidates |
| 9 | agent audit quality | — | compliance spot-check |
| 10 | environment | 4 atomic checks | toolchain + packages + network |
| 11 | cross-references | — | dead-link detection |
| 12 | duplicate rules | — | semantic dedup |
| 13 | community trends | — | market scan with 24h cache |
| 14 | skill-directory table | counts | AGENTS.md SKILL DIRECTORY vs actual skill consistency |
| 15 | task-routing table | — | context-engineering routing table vs actual skill consistency |

### 2B: External deep scan

#### Search strategy (4 parallel lanes)

1. **Known repos**: anthropics/skills, addyosmani/agent-skills, VoltAgent/awesome-agent-skills, ECC/affaan-m, superpowers
2. **High-star discovery**: `websearch: "github opencode skills popular stars"`
3. **Tech-stack specialization**: `websearch: "best AI agent skills for <tech-stack> github"`
4. **MCP search**: GitHub MCP-server repos (Rust, cargo, git, docker)

#### Scoring system (5 dimensions, max 10)

| Dimension | Weight | Criteria |
|------|:---:|------|
| **Fit** | 0.30 | tech-stack match (Rust/TS/DevOps/Web) |
| **Trust** | 0.20 | repo stars + owner reputation + LICENSE |
| **Track-record** | 0.20 | proven real-world use (not auto-generated) |
| **Freshness** | 0.15 | last update (deduct if >180 days stale) |
| **Specificity** | 0.15 | focused content vs generic talk |

**Sanity Gate**: any Trust < 2, or unreadable content → discard outright.

#### Security-audit gate (Full mode, before install)

| Check | Severity | Content |
|--------|:---:|------|
| repo-trust | **block** | stars, owner reputation, LICENSE, not archived |
| code-review | **block** | no `curl | sh`, no `eval()`, no unauthorized file access |
| permissions-scope | **block** | no global filesystem access, no sudo |
| dependency-audit | warn | dependency audit, CVE check |
| data-exfil | **block** | no unauthorized data transmission |
| freshness | warn | recent commits < 180 days |

- any **block** → reject
- any **warn** → warning + confirmation required
- all pass → SAFE

---

## Phase 3: Synthesis & Recommendation

### 3A: Cross-comparison

- multiple sources recommending the same content → +2 points
- tech-stack mismatch but portable pattern → annotate "rewrite-to-fit"

### 3B: Output format

```markdown
## Ecosystem Scan Report — {date}

### 🟢 Phase 1: Quick (N passed)
### 🔴 Phase 2: Full Local Audit (M issues found)
### 🟡 Phase 2: External Scan (K recommendations)
### ❌ Rejected (L items)

#### P1: Strongly recommended
| # | Item | Source | Score | Security | Effort |
|---|------|------|:---:|:---:|:---:|
| 1 | ... | repo | 9/10 | SAFE | low |

#### P2: Worth considering
| # | Item | Source | Score | Security | Effort |
|---|------|------|:---:|:---:|:---:|

#### Rejected
| Item | Reason | Security |
|------|------|:---:|
```

### 3C: Persistence

Record Quick-scan results in `.agents/memorys/`:
- each scan's findings, scores, decisions
- the next scan prioritizes checking whether last time's issues were fixed
- >=5 Quick runs → automatically suggest Full

---

## Team-mode configuration

```
member 1: structure-analyst (deep) — Phase 2A structural audit (items 2-6 in parallel)
member 2: content-auditor (deep)   — Phase 2A content audit (items 7-13 in parallel)
member 3: ecosystem-scanner (deep) — Phase 2B external, 4 lanes parallel
member 4: security-auditor (deep)  — Phase 2B security audit (Full mode)
member 5: synthesizer (deep)       — Phase 3 synthesis (waits for 1-4)
```

---

## Community references

This skill's composite design borrows from the following community precedents:

| Precedent | Pattern borrowed |
|------|-----------|
| [autoskills](https://github.com/B143KC47/autoskills) | 5-dimension scoring system, Sanity Gate, persistent memory |
| [agent-skill-discovery](https://github.com/ericgandrade/claude-superskills) | two-tier scope (installed / repo), platform detection |
| [skill-update-team](https://github.com/franktsai2008-eng/skill-update-team) | security-audit gate (6 checks), score weighting |
| [agent-self-audit](https://github.com/Xxt-XN/agent-self-audit) | Quick/Full two-tier design, 13 checks, auto-escalation |
| [claude-ecosystem](https://github.com/melodic-software/claude-code-plugins) | meta-skill architecture, 16 audit agents, component health |
| [skill-optimizer](https://github.com/hqhq1025/skill-optimizer) | skill lifecycle (miner → personalizer → generalizer) |
| [large-codebase-audit](https://github.com/MJWNA/large-codebase-audit-skill) | 9-surface AI-layer audit, aligned with Anthropic best practices |

---

## Adapting to any project

Replace the variables:
```
- tech stack: Rust + TypeScript + Docker + React → {your tech stack}
- agent platform: OpenCode → {your platform}
- rules path: .agents/ → {your rules path}
- package manager: pixi → {your package manager}
- memory path: .agents/memorys/ → {your memory path}
```
