# VisiaEngine Skills Registry

This repo's skills come in two layers: **global plugin skills** (not distributed with the repo) and **project skills** (`.agents/skills/`, versioned with the repo).

## Global Plugin Skills

Loaded by the `superpowers`, `ponytail`, `oh-my-openagent`, and `context-mode` plugins (assembly point: `.opencode/opencode.json` → `plugin`). Generic methodology, applicable to all projects; not duplicated here.

## Project Skills (under `.agents/skills/`)

The routing trigger surface = each SKILL.md frontmatter's `description`; agents activate them automatically by task context, no manual invocation needed.

The directory count is deliberately not written here: run `ls -d .agents/skills/*/ | wc -l`. Single source is `.agents/skills/`; host activation layers (e.g. `.codebuddy/skills/`) are generated links checked by `pixi run gate-skills`.

| Skill | Type | Purpose |
|-------|------|---------|
| `think-before-act` | meta-constraint | research first → list options → user approval → execute |
| `skill-router` | meta-constraint | intent analysis, output a recommended skill set |
| `ecosystem-scan` | meta-constraint | .agents system audit + community ecosystem scan |
| `adjudication-walkthrough` | meta-constraint | item-by-item adjudication (one at a time · seven-part template · plain language) |
| `lesson-review` | memory | batch session review, deposit lessons into memorys |
| `doc-audit` | docs | documentation and architecture self-consistency audit (interactive) |
| `book-to-skill` | docs | books/documents → structured skills |
| `openspec-explore` | spec | pre-change exploration / idea clarification |
| `openspec-propose` | spec | generate the proposal + design + tasks trio |
| `openspec-apply-change` | spec | implement per OpenSpec tasks |
| `openspec-sync-specs` | spec | sync delta specs into the main specs |
| `openspec-archive-change` | spec | archive a completed change |
| `api-interface-design` | engineering | contract-first design (trait / WS protocol / REST) |
| `context-engineering` | engineering | context routing for multi-language workspaces |
| `incremental-implementation` | engineering | thin vertical-slice implementation loop |
| `code-simplification` | engineering | over-engineering detection and complexity reduction |
| `source-driven-development` | engineering | dependency decisions follow official docs |
| `performance-optimization` | engineering | four-dimension latency/throughput/rendering/benchmark triage |
| `ci-cd-automation` | engineering | CI gate and build-matrix management |
| `test-harness` | testing | spec → test skeleton generation and verification |
| `browser-testing` | testing | Playwright verification of the web panel |
| `security-hardening` | security | OWASP + secret scanning + audit flow |
| `review-hardcode` | security | quick hardcoded-value scan (merged into security-hardening) |

## Division of Labor vs Rules/Memory

- **Rules** (`.agents/rules/`) = constant constraints, some loaded every turn (list in `.opencode/opencode.json` → `instructions`, 16 entries)
- **Memory** (`.agents/memorys/`) = volatile facts (the C/D/PIT numbering system)
- **Skills** (this table) = on-demand deep workflows
- Layer details: root `AGENTS.md` + `.agents/AGENTS.md`
