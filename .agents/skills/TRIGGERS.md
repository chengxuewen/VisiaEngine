# Skill Trigger Matrix — boundary map

> **This is a boundary map, not a skill.** The skill loader only reads directories that
> contain `SKILL.md`; a bare file here is never activated. It exists because each skill's
> own `## Boundary` section can only describe one side of a pair — nobody can see that two
> skills are claiming the same intent until routing picks the wrong one.
>
> Rule for new work: one intent has exactly ONE owner. If your new skill wants an intent
> that already has a row, you must amend the `Hand-off / non-goal` cell in the same commit,
> and the row must still name every skill that exists (`skill-lint` checks both directions:
> every skill directory appears here, and every owner named here has a directory).
>
> Host built-ins are listed where users collide with them on purpose (`create-skill`,
> `create-subagent`, `canvas`): they live in the IDE, not in this repo, so this repo cannot
> route to them — the row says what to do instead.

## Doing work (what runs when)

| Intent | Owning skill | Hand-off / non-goal |
|---|---|---|
| Investigate before acting; present options for approval | `think-before-act` | Meta-constraint, always first. Task layer mapping → `repo-triage`; skill choice → `skill-router`. |
| Find the layer/files for a broad, ambiguous ask | `repo-triage` | Skill selection → `skill-router`; language-rule selection → `context-engineering`. |
| Route to the right skill for an unclear intent | `skill-router` | Which rules apply to which file → `context-engineering`; it does not execute the work. |
| Break a spec into ordered verifiable slices | `planning-and-task-breakdown` | Executing a slice per slice → `incremental-implementation`. |
| Implement a multi-file / multi-layer change | `incremental-implementation` | Rust quality gates while coding → `rust-engineering`. |
| Write the failing test first, then the code | `test-driven-development` | Test skeletons generated from SDD clauses, coverage reports → `test-harness`. |
| Need a test skeleton from a clause, or a coverage number | `test-harness` | Red-green-refactor discipline → `test-driven-development`. |
| Reproduce and fix a failure / a red gate | `debugging-and-error-recovery` | Performance regressions → `performance-optimization`; design questions → `think-before-act`. |
| Rust work in this workspace (fmt/clippy/unsafe/FFI) | `rust-engineering` | Language-rule routing → `context-engineering`; review of a diff → `code-review-and-quality`. |
| Route language-specific rules to code | `context-engineering` | Picks RULES, never skills — that is `skill-router`. |
| Structural bulk rewrite (pattern → rewrite) | `ast-grep-rewrites` | One-off small edits stay manual; semantic re-analysis → `lsp-code-analysis`. |
| Pre-change impact analysis (who breaks if I edit X) | `lsp-code-analysis` | Post-change proof is the gates' job (`gate-abi`, `spec-trace`), not this skill's. |
| Remove AI-generated noise from a diff | `ai-slop-cleanup` | Reviewing correctness → `code-review-and-quality`. |
| Review a diff or PR (five-axis, severity labels) | `code-review-and-quality` | Rust-specific depth → `rust-engineering`; secrets/OWASP → `security-hardening`. |
| Simplify code that is over-engineered | `code-simplification` | Does not review correctness → `code-review-and-quality`. |
| Add an exported C ABI symbol / trait boundary / protocol message | `api-interface-design` | ABI surface verification → `gate-abi`; review → `code-review-and-quality`. |
| Retire an API, flag, skill, or rename a surface | `deprecation-and-migration` | Retired trigger records live here in this file too. |
| New dependency, or an unfamiliar external API | `source-driven-development` | Upstream docs first; dependency audit after → `security-hardening` / `pixi run audit`. |
| Add or move a check/gate, decide where a check belongs | `constraint-driven-development` | Whole-surface coverage questions → `repo-audit`. |
| Orchestrate more than one subagent | `subagent-orchestration` | Scope/layer mapping before dispatch → `repo-triage`. |
| Release, ship, "ship it" | `shipping-and-launch` | Docs freshness before release → `doc-audit`; CI internals → `ci-cd-automation`. |
| CI pipeline / gate-runner troubleshooting | `ci-cd-automation` | Gate placement policy → `constraint-driven-development`. |
| Instrument: logs, metrics, tracing, alerts | `observability-and-instrumentation` | Browser-side wall verification → `browser-testing`. |
| Browser-side verification of the web wall | `browser-testing` | Headless gate design → `repo-audit` domain 3; the non-default-env gate is `gate-web`. |
| Latency / throughput / benchmark regression | `performance-optimization` | Rendering bugs (not slowness) → `debugging-and-error-recovery`. |
| Security audit, secrets scan, hardening | `security-hardening` | Absorbs `review-hardcode` (deprecated quick scan — same Phase 1, one owner). |
| FFI/binding boundary review | `security-hardening` | Contract design → `api-interface-design`; Rust unsafe depth → `rust-engineering`. |

## Decisions, records, and review of the system itself

| Intent | Owning skill | Hand-off / non-goal |
|---|---|---|
| Architecture decision needs recording | `documentation-and-adrs` | Auditing existing docs → `doc-audit`. |
| Audit project documentation consistency in depth | `doc-audit` | Whole engineering surface + ecosystem → `repo-audit`; numbering semantics belong here, numbering gaps to `repo-audit`. |
| Periodic whole-repo checkup / find adoptable skills | `repo-audit` | Absorbs `ecosystem-scan` (alias); documentation deep-dive → `doc-audit`. |
| Extract lessons after a long session | `lesson-review` | Immediate one-lesson capture during work is C9's duty, not this skill's. |
| Walk user adjudication one item at a time | `adjudication-walkthrough` | Format rules also in C16; it never decides for the user. |
| Propose / explore / apply / archive / sync a spec change | `openspec-propose`, `openspec-explore`, `openspec-apply-change`, `openspec-archive-change`, `openspec-sync-specs` | One family, five stages: propose → apply → archive; explore clarifies before proposing; sync updates main specs without archiving. Not for ad-hoc edits. |
| Turn a book/document into a studyable skill | `book-to-skill` | Creating a hand-written skill → `skill-creator`. |

## Skill-system maintenance

| Intent | Owning skill | Hand-off / non-goal |
|---|---|---|
| Create a skill in this repo | `skill-creator` | The five hard rules + router registration. Host built-in `create-skill` may be invoked by the IDE, but it knows nothing about `agent-targets.json`, `SKILLS-INDEX`, or this file — repo work goes here. |
| Create a subagent | Host built-in `create-subagent` | Not in this repo: nothing here owns subagent definitions yet; if one is added, it must take a row above and the `subagent-orchestration` hand-off must be amended. |
| Edit/add a Qoder IDE skill or MCP entry | Host built-in `qoder-ide-helper` | Single-source discipline still applies: `.agents/` is the source, host layers are generated. |
| Change `.agents/` itself (memorys/rules/skills targets) | `repo-audit` for the checkup; `constraint-driven-development` for gate placement | After any change: `pixi run agent-sync` then `pixi run gate-agents`. |

## Retired triggers (kept here so a new skill does not re-invent them)

| Old name | Status | Now owned by |
|---|---|---|
| `review-hardcode` | deprecated alias, kept as a pointer only | `security-hardening` (Phase 1) |
| `ecosystem-scan` | superseded | `repo-audit` (Phase 2B/2C) |
