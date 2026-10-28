# Rules

Rules are part of the `.agents/` directory, organized into a **common** layer plus **language-specific** directories:

```text
.agents/rules/
├── common/          # Language-agnostic principles (always install)
│   ├── coding-style.md
│   ├── git-workflow.md
│   ├── testing.md
│   ├── performance.md
│   ├── patterns.md
│   ├── hooks.md
│   ├── agents.md
│   └── security.md
├── cpp/             # C++ specific
├── python/          # Python specific
├── rust/            # Rust specific
├── typescript/      # TypeScript/JavaScript specific
└── web/             # Web and frontend specific
```

- **common/** contains universal principles — no language-specific code examples.
- **Language directories** extend the common rules with framework-specific patterns, tools, and code examples. Each file references its common counterpart.

Rules are project-local and read directly by the AI assistant from `.agents/rules/` — no copying to `~/.claude/` needed.

## Rules vs Skills

- **Rules** define standards, conventions, and checklists that apply broadly (e.g., "80% test coverage", "no hardcoded secrets").
- **Skills** (`skills/` directory) provide deep, actionable reference material for specific tasks (e.g., `python-patterns`, `rust-testing`).

Language-specific rule files reference relevant skills where appropriate. Rules tell you *what* to do; skills tell you *how* to do it.

## Adding a New Language

To add support for a new language (e.g., `rust/`):

1. Create an `.agents/rules/rust/` directory
2. Add files that extend the common rules:
   - `coding-style.md` — formatting tools, idioms, error handling patterns
   - `testing.md` — test framework, coverage tools, test organization
   - `patterns.md` — language-specific design patterns
   - `hooks.md` — PostToolUse hooks for formatters, linters, type checkers
   - `security.md` — secret management, security scanning tools
3. Each rule file uses the uniform shape (all 40 whitelisted files follow it):
   ```
   ---
   paths:
     - "**/*.<ext>"
   ---

   # <Title>

   > This file extends [common/xxx.md](../common/xxx.md) with <Language> specific content.
   ```
   - `paths` is optional glob metadata (the `mdc` generator drops it but keeps it as the future glob-mapping source).
   - The first body line MUST be an H1 — the generator derives the rule `description` from it.
   - A `> This file extends ...` note, if present, follows the H1 (never before it).
4. Reference existing skills if available, or create new ones under `skills/`.

For non-language domains like `web/`, follow the same layered pattern when there is enough reusable domain-specific guidance to justify a standalone ruleset.

## Rule Priority

When language-specific rules and common rules conflict, **language-specific rules take precedence** (specific overrides general). This follows the standard layered configuration pattern (similar to CSS specificity or `.gitignore` precedence).

- `.agents/rules/common/` defines universal defaults applicable to all projects.
- `.agents/rules/rust/`, `.agents/rules/cpp/`, `.agents/rules/python/`, `.agents/rules/typescript/`, `.agents/rules/web/`, etc. override those defaults where language idioms differ.

### Example

`common/coding-style.md` recommends immutability as a default principle. A language-specific `rust/coding-style.md` can override this:

> Idiomatic Rust uses `&mut` borrows for in-place mutation rather than copying — see [common/coding-style.md](../common/coding-style.md) for the general principle, but Rust-idiomatic mutation is preferred here.

### Common rules with override notes

Rules in `.agents/rules/common/` that may be overridden by language-specific files are marked with:

> **Language note**: This rule may be overridden by language-specific rules for languages where this pattern is not idiomatic.
