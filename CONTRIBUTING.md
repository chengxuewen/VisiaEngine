# Contributing

## Setup

```bash
bash bootstrap.sh   # first time: pixi + toolchain + cargo mirror (idempotent)
source pixi.sh      # daily activation; or prefix every command with `pixi run`
pixi run verify     # six-piece toolchain smoke (rustc/cargo/rustfmt/clippy/deny/rg)
```

## Before you open a change

- `pixi run ci` is the quality gate: 16 segments (fmt, lint, check, test, audit,
  the `gate-*` checks, cmake-smoke, pack-check, i18n). It must be green.
- Commit discipline is `C24` in `.agents/memorys/conventions.md`: subject <= 72
  characters, no `;`, body bullets with `- `, one logical change per commit.
- Generated host layers (`.codebuddy/`, `.trae/`, `.mcp.json`, `AGENTS.md`,
  `CONSTRAINTS.md`, `SKILL.md`) are produced from `.agents/` by
  `pixi run agents sync`. The pre-commit hook re-syncs and verifies them; never
  hand-edit a generated file.
- A change that touches `unsafe`, FFI, serialization or a public API gets a second
  read (`.agents/skills/code-review-and-quality`).

## Where to look

- `docs/env.md` — environment, tasks, build trees, MCP lifecycle, gates.
- `AGENTS.md` and `.agents/` — the single source for rules, skills, memory and MCP.
- `CONSTRAINTS.md` — anti-gaming rules plus the generated gate list.
