# .agents — project memory / rules / skills

Root of the agent toolchain (directory tree below; all .md or skill attachments). The assembly point is `.opencode/opencode.json`.

## STRUCTURE

```
.agents/
├── memorys/   # volatile facts: status (state snapshot) conventions (C conventions) decisions (D decisions) pitfalls (PIT stumbles)
├── rules/     # standing constraints: common/ (loaded into instructions) + <lang>/ (on demand; hierarchy in rules/README.md)
└── skills/    # single source of project skills; one dir per skill (count: `ls -d */ | wc -l`), each registered via SKILL.md frontmatter; host layers (.codebuddy/skills, .claude/skills) are GENERATED links -- see `pixi run gate-skills`, never hand-edit
```

## Loading mechanism (determines writing discipline)

| Layer | When it enters context | Implication |
|----|-------------|------|
| `memorys/status+conventions` + `rules/common` 12 files + `rules/rust/{coding-style,hooks}` | **fully every turn** (`opencode.json` → `instructions`, 16 entries total) | every line carries a fixed token cost; stay lean, never duplicate skills content |
| `rules/<lang>/` | on demand when file extensions match | language details may live here |
| `skills/*/SKILL.md` | frontmatter description matches the trigger | the description is the routing lifeline: generic methodology words, no brand residue |
| `memorys/decisions+pitfalls` | explicit read | may be long, append-only, never rewrite historical entries |

## Numbering discipline

- VisiaEngine's own C{n}/D{n}/PIT-{n} numbering starts from the template (C2-C8 and C10-C13 are reserved gaps; see the header declaration in conventions.md).
- **Forbidden** to cite the predecessor project's numbering — the new stores have no such entries. The predecessor's history lives in `.refinfo/MediaServo/.agents/` (read-only archive).
- The five-part form (symptom / root cause / solution / verification) and the D-entry format are embedded in each template's comments; new entries copy the skeleton verbatim.

## Verification after changes

Run the grep gates from the root `AGENTS.md` COMMANDS section; spot-check frontmatter (`---` pairing, single-line YAML description) with `head -6`.
