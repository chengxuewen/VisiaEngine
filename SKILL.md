# VisiaEngine Skills Registry

This repo's skills come in two layers: **global plugin skills** (not distributed with the repo) and **project skills** (`.agents/skills/`, versioned with the repo).

## Global Plugin Skills

Loaded by the `superpowers`, `ponytail`, `oh-my-openagent`, and `context-mode` plugins (assembly point: `.opencode/opencode.json` → `plugin`). Generic methodology, applicable to all projects; not duplicated here.

## Project Skills (under `.agents/skills/`)

The routing trigger surface = each SKILL.md frontmatter's `description`; agents activate them automatically by task context, no manual invocation needed.

The directory count is deliberately not written here: run `ls -d .agents/skills/*/ | wc -l`. Single source is `.agents/skills/`; host activation layers (e.g. `.codebuddy/skills/`) are generated links checked by `pixi run gate-skills`.

<!-- BEGIN:skill-registry -->

| Skill | Purpose |
|-------|---------|
| `adjudication-walkthrough` | one-at-a-time adjudication protocol. |
| `ai-slop-cleanup` | Remove AI-generated noise from this repo's... |
| `api-interface-design` | Contract-first interface design for polygl... |
| `ast-grep-rewrites` | Structural search and rewrite with the ast... |
| `book-to-skill` | Converts books and documents (PDF, EPUB, D... |
| `browser-testing` | Browser-side verification of this engine's... |
| `ci-cd-automation` | Generic CI/CD pipeline management: Docker... |
| `code-review-and-quality` | Code review and quality for this engine: f... |
| `code-simplification` | Reduce complexity in Rust/TS code. |
| `constraint-driven-development` | Constraint-driven development for this eng... |
| `context-engineering` | Route language-specific rules to the right... |
| `debugging-and-error-recovery` | Systematic debugging and error recovery fo... |
| `deprecation-and-migration` | Deprecate and migrate safely in this repo:... |
| `doc-audit` | Project documentation and architecture audit. |
| `documentation-and-adrs` | Record architecture decisions and keep doc... |
| `ecosystem-scan` |  |
| `incremental-implementation` | Thin vertical slice methodology — implemen... |
| `lesson-review` | Batch session review: systematically extra... |
| `observability-and-instrumentation` | Observability and instrumentation for this... |
| `openspec-apply-change` | Implement tasks from an OpenSpec change. |
| `openspec-archive-change` | Archive a completed change in the experime... |
| `openspec-explore` | Enter explore mode — a thinking partner fo... |
| `openspec-propose` | Propose a new change with structured artif... |
| `openspec-sync-specs` | Sync delta specs from a change to main specs. |
| `performance-optimization` | Generic performance profiling and optimiza... |
| `planning-and-task-breakdown` | Turn a spec or a vague ask into ordered, v... |
| `repo-triage` | Layer triage for an ambiguous task in this... |
| `review-hardcode` (deprecated alias) | Hardcoded secrets/ports/URLs scan. |
| `rust-engineering` | Rust engineering workflow for this engine... |
| `security-hardening` | Security audit and hardening: OWASP Top 10... |
| `shipping-and-launch` | Release and launch checklist for this engi... |
| `skill-router` | Analyze user intent and output a recommend... |
| `source-driven-development` | Ground every external dependency decision... |
| `subagent-orchestration` | Orchestrate subagents for this repo: decid... |
| `test-driven-development` | Red-green-refactor for this engine: write... |
| `test-harness` | Multi-language automated test harness. |
| `think-before-act` | Meta-constraint: investigate before acting... |
<!-- END:skill-registry -->

## Division of Labor vs Rules/Memory

- **Rules** (`.agents/rules/`) = constant constraints, some loaded every turn (list in `.opencode/opencode.json` → `instructions`, 16 entries)
- **Memory** (`.agents/memorys/`) = volatile facts (the C/D/PIT numbering system)
- **Skills** (this table) = on-demand deep workflows
- Layer details: root `AGENTS.md` + `.agents/AGENTS.md`
