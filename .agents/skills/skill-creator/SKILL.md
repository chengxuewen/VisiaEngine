---
name: skill-creator
description: "Create a new skill the VisiaEngine way: the five hard rules (frontmatter, naming, single-source sync, router registration, gate verification), the directory recipe, and the post-creation checklist. Use when adding a skill to .agents/skills/, fixing a skill's frontmatter, or wiring a new trigger into skill-router."
---

# Skill Creator (VisiaEngine edition)

Create skills that pass gate-skills on the first try and route correctly on the
second. The generic platform guidance does not know this repo's constitution --
this file is the constitution, operationalized.

## The five hard rules

1. **Frontmatter is mandatory**: every SKILL.md starts with `name:` + `description:`
   (gate-skills checks; the description doubles as the router's classification text).
2. **Naming**: kebab-case directory == `name:` field; NEVER create a case-variant of
   an existing file (PIT-87: a 0-byte `AGENTS.md` next to `agents.md` became a
   generated rule silently, and is a checkout collision on case-insensitive systems).
3. **English-only source** (C15/C17): gate-i18n scans SKILL.md files; non-English
   literals need a whitelist justification or `\u` escapes.
4. **One skill = one trigger domain**: check skill-router's category table before
   picking trigger words; two skills competing for the same trigger is a red flag
   (repo-audit domain 1).
5. **Sync is part of creation**: `.agents/skills/` is the single source; the host
   mirrors are generated. Hand-editing a mirror is forbidden.

## Creation recipe

```
1. mkdir .agents/skills/<kebab-name>/ && write SKILL.md
   - frontmatter (rule 1), Overview, When to Use, boundary with sibling skills
   - end with a Verification section (evidence the skill ran)
2. python3 scripts/agent-links.py sync     # rules/memory hubs
3. python3 scripts/skills-links.py sync    # host skill mirrors (.codebuddy ...)
4. pixi run gate-skills                    # inventory == directory table
5. pixi run gate-agents && pixi run i18n   # hubs clean, English-only
6. add a skill-router category row ONLY if the trigger domain is new
7. commit canonical + regenerated mirrors together
```

## Adopting a platform skill instead of writing one

Run it through repo-audit first (2B: 5-dimension Fit + security gate; 2C: pattern
card if the value is the PRACTICE rather than the artifact). A platform skill whose
description already mentions this repo is usually already project-level -- check
`.agents/skills/` before importing anything.

## Lessons baked in

- PIT-87 (case variants), the routing-dilution stance (repo-audit's anti-excuse
  table), and the D45/D46 renumber lesson: unique numbering + date + title triple
  for anything that supersedes.
