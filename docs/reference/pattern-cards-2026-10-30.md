# Practice-scan pattern cards -- 2026-10-30 (repo-audit 2C, first batch)

Adoptable practices extracted from other projects' engineering workflows. Unit of
output: a pattern card (pattern / source / here / Fit / cost / verdict). P1 =
adopt; P2 = consider; have-it = this repo already does it (kept for validation);
discard = not applicable.

| # | Pattern | Source | Here | Verdict |
|---|---------|--------|------|---------|
| PC-1 | Tiered ratchet floors (kernel 100% / enforcement 85% / general 70%) | opaque.info coverage gate | uniform 60% chapter floor (book-i18n FLOOR) | consider -- per-chapter tiers when translation batches resume |
| PC-2 | Unidirectional ratchet, CI-enforced between sprints | Citrate .agentile/coverage/GATES.md | book-i18n FLOOR gate implements exactly this | have-it (independent convergence) |
| PC-3 | Staged pipeline: cheapest gates first, slow stages parallel (~14 min total) | GitCode RustTraining CI series | the `ci` gate chain is ordered by dependency, not cost | consider -- pixi depends-on can express a cheap-first split |
| PC-4 | Every CI command passes --locked/--frozen | rs4ts.dev pitfall 3 | all pixi CI commands run --frozen | have-it |
| PC-5 | Machine-readable approved-spec registry (JSON + governance version) | traverse-framework .github/quality-standards | gate-trace has the substance, no JSON registry | consider -- medium cost |
| PC-6 | ADR immutability + SUPERSEDED marker | AWS ADR governance | decisions.md SUPERSEDED convention matches; the D45/D46 duplicate repair was a one-time annotated fix | have-it |
| PC-7 | Router-scoped on-demand ADR reading | chronowarden .ai/adr/README.md | skill-router implements the same discipline | have-it |
| PC-8 | MADR decision template | stackfyi ADR guide | this repo's decision format is strictly richer | discard |

Fit check: none of the above violates the constitution (C17 English-only,
append-only memorys, gates-first). Sources were read on 2026-10-30; re-verify
before acting on a consider item.
