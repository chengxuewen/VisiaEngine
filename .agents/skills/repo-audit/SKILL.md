---
name: repo-audit
description: Whole-repo engineering audit across 10 domains (skills, rules, mcp, scripts, docs, book, cli, tests, structure, naming) + external ecosystem scan for adoptable skills; formerly ecosystem-scan.
---

# repo-audit — Whole-Repo Engineering Audit + Ecosystem Scan

> Two-tier scan (Quick daily / Full deep) across **10 engineering domains**, quality scoring, security-audit gate.
> Governing principle: **gates-first** -- this repo runs 17 gates in `pixi run ci`; the scan never re-checks by hand
> what a gate proves. The scan (a) verifies every surface HAS an enforcement owner, (b) runs gap checks where no
> gate exists, (c) spends deep analysis only where judgment is needed.
> Reference precedents: [autoskills](https://github.com/B143KC47/autoskills), [agent-skill-discovery](https://github.com/ericgandrade/claude-superskills), [skill-update-team](https://github.com/franktsai2008-eng/skill-update-team), [agent-self-audit](https://github.com/Xxt-XN/agent-self-audit).

## Trigger Conditions

- The user says "优化 agents" / "审计 .agents" / "扫描生态" / "找新技能" / "全面体检" (optimize agents / audit .agents / scan ecosystem / find new skills / full checkup)
- `/repo-audit` -- pops up the mode-selection menu
- `/repo-audit quick` -- fast scan (default)
- `/repo-audit full` -- deep scan (auto-escalated on first run or after >5 Quick runs)
- `/repo-audit report` -- view the last scan report
- `/repo-audit <domain>` -- single-domain deep dive (skills|rules|mcp|scripts|docs|book|cli|tests|structure|naming)
- `/ecosystem-scan ...` -- compatibility alias for every form above (retire after a grace period)

## The 10 Audit Domains

| # | Domain | Surface | Enforcement owner (gates) | The scan adds |
|---|--------|---------|---------------------------|---------------|
| 1 | **skills** | `.agents/skills/` | gate-skills (inventory); gate-agents (SKILLS-INDEX segment) | duplicate detection, community comparison, utilization |
| 2 | **rules** | `.agents/rules/` (43 files) | gate-rules, gate-agents | semantic dedup, runnable-command check, quality |
| 3 | **mcp** | `.mcp.json`, host mirrors, `scripts/mcp-*` | gate-mcp (MCP-FILE segment via scripts/mcp-file-guard.py) | launcher portability (PIT-86), pin drift, per-host parity |
| 4 | **scripts** | `scripts/` (~25 py/sh) | gate-docs 12 (names only) | syntax health sweep (`bash -n` / `ast.parse`), orphan detection (not referenced by pixi/gates/docs), CRLF/LF consistency (PIT-93), shebang + exec bits |
| 5 | **docs** | `docs/` (README, env, whitepaper, sdd/, reference/) | gate-prose, gate-docs, gate-trace | cross-reference audit; numbering HOLES detected here, numbering SEMANTICS hand off to doc-audit dimension 3 |
| 6 | **book** | `book/` + `www/book/` | book-check, gate-docs 13 | toc<->SUMMARY<->sidebar nesting consistency, vendored offline assets, fold structure, title-length budget |
| 7 | **cli** | `visiaengine.sh` / `.bat` | gate-docs 12/12b | USAGE<->pixi task sync, sh/bat parity (advertised set diff), front-door discoverability |
| 8 | **tests** | examples/, ctest registry, `smoke-*` | cmake-smoke, pack-check, ctest | orphaned tests, registry<->examples consistency, dual-mode (human/ctest) coverage |
| 9 | **structure** | repo tree | manual | AGENTS.md STRUCTURE tree vs `ls` drift, root pollution (unexpected top-level dirs), territory ownership (`build/` `www/` `target/` `.tools/`) |
| 10 | **naming** | files, tasks, branches | gate-rules (partial) | case-collision detection (PIT-87), task-name style consistency, kebab-case adherence, mixed line-ending detection |

**Reading the table**: if the enforcement owner is green, that domain's mechanical health is proven -- do not re-audit it.
The scan's local value concentrates in domains 4, 9, 10 (thin/no gate coverage), domain 6's nesting consistency
(toc<->SUMMARY<->sidebar has NO gate -- gate-docs 13 only resolves entries), and the judgment checks of 1-3, 5-8.

## Entry Menu

With no arguments, pop up:

```
[1] Quick Scan -- 10-domain fast audit (< 30s)
[2] Full Scan -- deep audit + community sync + security gate (3-5min)
[3] View Report -- view the last scan results
[4] Quick Fix -- auto-fix LOW/MEDIUM issues (non-interactive; NEVER touches
                 .agents/memorys/decisions.md or status.md history -- report only)
[5] Domain dive -- deep-scan one domain (see Trigger Conditions)
```

## Two-Tier Modes

| Feature | Quick | Full |
|------|:-----:|:----:|
| Local audit | 1 fast check per domain (10) | deep checks per domain (24 items) |
| External scan | 3-5 known high-star repos | comprehensive websearch + GitHub search |
| Scoring | 3-dimension fast | 5-dimension full scoring |
| Security audit | none | 6 safety checks |
| Duration | < 30s | 3-5min |
| Trigger | `/ecosystem-scan` | `/ecosystem-scan full` or >=5 Quick runs |

---

## Phase 1: Quick Scan (default)

### 1A: Local fast audit (10 items, one per domain)

```
1. skills:    every SKILL.md has name + description frontmatter; directory table covers all
2. rules:     conventions/decisions/pitfalls numbering contiguous; no orphan rule files
3. mcp:       every instructions file referenced by the host configs exists; pins present
4. scripts:   `bash -n` all *.sh, `ast.parse` all *.py -- zero syntax errors
5. docs:      read the LAST gate output for gate-prose/gate-docs (do not hand-check; re-running gate-docs costs >15s -- its 11 does a network probe)
6. book:      book-check green (no build artifacts = SKIP, same convention as gate-docs 13); SUMMARY entry count == toc chapter count
7. cli:       USAGE-advertised names all exist in pixi.toml (gate-docs 12); .sh/.bat sets equal
8. tests:     every smoke-*/ctest entry has a live example file (spot-check 5)
9. structure: AGENTS.md STRUCTURE tree top-level dirs == actual `ls` (drift = finding)
10. naming:   no case-variant filename collisions in .agents/ and docs/; no mixed EOL in docs/ book/
```

### 1B: External fast scan

Search 3-5 known high-star repos:
- VoltAgent/awesome-agent-skills (index)
- addyosmani/agent-skills (engineering skills)
- ECC/affaan-m everything-claude-code (full-stack configs)

Fetch READMEs with `webfetch`; 3-dimension quick scoring: tech-stack fit / already-present-in-project / quality.

### 1C: Quick output

```
🟢 No issues -- N checks passed, no deep scan needed
🟡 Found N items worth attention (CRITICAL/HIGH/MEDIUM/LOW ladder, matching code review) -- recommend /repo-audit full
```

---

## Phase 2: Full Scan

### 2A: Local deep audit (24 items across 10 domains)

| Domain | Deep checks |
|--------|-------------|
| skills | duplicate detection; community comparison; utilization vs install count; frontmatter quality |
| rules | semantic dedup; runnable-command verification; cross-reference integrity; staleness |
| mcp | launcher portability matrix (per-host cwd); pin vs installed drift; single-source consistency |
| scripts | orphan candidates (referenced by nothing); duplicated logic; CRLF/LF mix; arg convention consistency; error-message convention (`GATE-* ✗` shapes) |
| docs | dead-link detection; §N/PIT/D numbering contiguity; glossary consistency; polish-scope coverage |
| book | toc/SUMMARY/sidebar triple consistency; fold structure; vendor freshness + license files; mermaid/lazy-loader pairing |
| cli | full USAGE parity both shells; task-advertising drift; exit-code convention |
| tests | registry<->example bidirectional completeness; dual-mode entry coverage; golden-file freshness |
| structure | territory ownership audit; gitignore gaps; generated-vs-canonical tree diff |
| naming | case-collision scan (PIT-87); naming-convention adherence per file type; branch/tag style |

### 2B: External deep scan

#### Search strategy (4 parallel lanes)

1. **Known repos**: anthropics/skills, addyosmani/agent-skills, VoltAgent/awesome-agent-skills, ECC/affaan-m, superpowers
2. **High-star discovery**: `websearch: "github opencode skills popular stars"`
3. **Tech-stack specialization**: `websearch: "best AI agent skills for Rust/TypeScript/DevOps github"`
4. **MCP search**: GitHub MCP-server repos (Rust, cargo, git, docker)

#### Scoring system (5 dimensions, max 10)

| Dimension | Weight | Criteria |
|------|:---:|------|
| **Fit** | 0.30 | tech-stack match (Rust/TS/DevOps/Web) |
| **Trust** | 0.20 | repo stars + owner reputation + LICENSE |
| **Track-record** | 0.20 | proven real-world use (not auto-generated) |
| **Freshness** | 0.15 | last update (deduct if >180 days stale) |
| **Specificity** | 0.15 | focused content vs generic talk |

**Sanity Gate**: any Trust < 2, or unreadable content -> discard outright.

#### Security-audit gate (Full mode, before install)

| Check | Severity | Content |
|--------|:---:|------|
| repo-trust | **block** | stars, owner reputation, LICENSE, not archived |
| code-review | **block** | no `curl \| sh`, no `eval()`, no unauthorized file access |
| permissions-scope | **block** | no global filesystem access, no sudo |
| dependency-audit | warn | dependency audit, CVE check |
| data-exfil | **block** | no unauthorized data transmission |
| freshness | warn | recent commits < 180 days |

- any **block** -> reject
- any **warn** -> warning + confirmation required
- all pass -> SAFE

---

## Phase 3: Synthesis & Recommendation

### 3A: Cross-comparison

- multiple sources recommending the same content -> +2 points
- tech-stack mismatch but portable pattern -> annotate "rewrite-to-fit"

### 3B: Output format

```markdown
## Ecosystem Scan Report -- {date}

### 🟢 Phase 1: Quick (N passed)
### 🔴 Phase 2: Full Local Audit (M issues found, by domain)
### 🟡 Phase 2: External Scan (K recommendations)
### ❌ Rejected (L items)

#### Per-domain verdict
| Domain | Enforcement | Finding | Action |
|--------|-------------|---------|--------|

#### P1: Strongly recommended
| # | Item | Source | Score | Security | Effort |
|---|------|------|:---:|:---:|:---:|

#### P2: Worth considering
| # | Item | Source | Score | Security | Effort |
|---|------|------|:---:|:---:|:---:|

#### Rejected
| Item | Reason | Security |
|------|------|:---:|
```

### 3C: Persistence

Record scan results in `.agents/memorys/status.md` (a fixed "repo-audit" paragraph -- the memorys
surface is a fixed four-file set, and new files change the agent-links sync face; keep the text English, C17/C20):
- each scan's findings, scores, decisions; the next scan first checks whether last time's issues were fixed
- the Quick-run counter lives in the same paragraph
- >=5 Quick runs -> automatically suggest Full

---

## Team-mode configuration (Full scan only; Quick runs in the main session)

```
member 1: structure-analyst (deep) -- domains 9+10 + scripts syntax sweep
member 2: content-auditor (deep)   -- domains 1+2+5 (skills/rules/docs content)
member 3: platform-auditor (deep)  -- domains 3+6+7+8 (mcp/book/cli/tests)
member 4: ecosystem-scanner (deep) -- Phase 2B external, 4 lanes parallel
member 5: security-auditor (deep)  -- Phase 2B security audit
member 6: synthesizer (deep)       -- Phase 3 synthesis (waits for 1-5; may be the main session)
```

## Community references

This skill's composite design borrows from the following community precedents:

| Precedent | Pattern borrowed |
|------|-----------|
| [autoskills](https://github.com/B143KC47/autoskills) | 5-dimension scoring system, Sanity Gate, persistent memory |
| [agent-skill-discovery](https://github.com/ericgandrade/claude-superskills) | two-tier scope (installed / repo), platform detection |
| [skill-update-team](https://github.com/franktsai2008-eng/skill-update-team) | security-audit gate (6 checks), score weighting |
| [agent-self-audit](https://github.com/Xxt-XN/agent-self-audit) | Quick/Full two-tier design, auto-escalation |
| [claude-ecosystem](https://github.com/melodic-software/claude-code-plugins) | meta-skill architecture, audit agents, component health |
| [skill-optimizer](https://github.com/hqhq1025/skill-optimizer) | skill lifecycle (miner -> personalizer -> generalizer) |
| [large-codebase-audit](https://github.com/MJWNA/large-codebase-audit-skill) | multi-surface AI-layer audit, aligned with Anthropic best practices |

## Adapting to any project

Replace the variables:
```
- tech stack: Rust + TypeScript + Docker + React -> {your tech stack}
- agent platform: OpenCode -> {your platform}
- rules path: .agents/ -> {your rules path}
- package manager: pixi -> {your package manager}
- memory path: .agents/memorys/ -> {your memory path}
- gate list: pixi.toml `ci = { depends-on = [...] }` -> {your gate list}
- entry docs: AGENTS.md + docs/env.md -> {your entry docs}
```

## Boundary with doc-audit

- **repo-audit** (this skill): audits the **whole engineering surface** (10 domains above) and scans the
  **external** ecosystem for adoptable content.
- **doc-audit**: audits **project documentation** consistency (architecture/design/decision records) in depth;
  it already carries the "Division of labor with repo-audit" table.
- Rule of thumb: adopting/optimizing skills or rules, or a periodic whole-repo checkup -> this skill;
  docs/architecture consistency deep-dive -> `doc-audit`.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "more skills = better" | AI-slop skills dilute routing; adopt only high-fit, safe ones. |
| "skip the security gate, it's a popular repo" | Popularity is not safety; run the 6 checks. |
| "one quick scan is enough" | Quick misses duplicates/security; escalate to Full periodically. |
| "the gates are green, nothing to scan" | Gates prove mechanical health per surface; they do NOT prove coverage (a surface without a gate is invisible), nor naming/structure/duplication quality -- that is exactly the scan's gap analysis. |
| "audit .agents only, the rest is product code" | Scripts, CLI, tests and tree structure ARE the agent system's execution surface -- PIT-86/87/92/93 all lived outside .agents/. |

## Red Flags

- Installing a skill without the security gate.
- A skill whose frontmatter lacks name/description.
- Duplicated skills competing for the same trigger.
- No record of the scan result in memory.
- A domain in the matrix whose enforcement owner column is empty AND has no scan finding -- that is a coverage hole, not a pass.

## Verification (evidence)

- The scan report with per-domain verdict, per-item score + security verdict.
- `GATE-SKILLS` and the gate-agents SKILLS-INDEX segment cover every skill (inventory consistent); the 17-gate ci chain green.
- Gap findings cite file:line and name the enforcement owner that should own them.
