---
name: context-engineering
description: "Route language-specific rules to the right code in a multi-language workspace. Loads Rust rules for .rs tasks, FFI/native rules for binding boundaries, protocol rules for wire-contract work, by file extension and module domain. Prevents wrong-language lint violations and out-of-scope analysis. Use BEFORE any cross-module or multi-language task."
---

# context-engineering — Right Context, Right Language

> Route language-specific rules to the right module. Don't apply C++ lints to Rust code. Don't audit Python scripts for Rust ownership semantics.

## When to Use

| Trigger | Reason |
|---------|--------|
| Task touches 2+ languages | Avoids wrong-language rule pollution |
| New module/package being created | Establishes correct context boundaries |
| FFI/SDK work (napi-rs, cbindgen, FlatBuffers, bindings of any kind) | Multi-language contract, not single-lang |
| Web/dashboard work | HTML/CSS/JS + backend, different rulesets |
| Code review spanning modules | Each module needs its own rule set applied |

## Scope → Language → Ruleset Routing

Do NOT keep a hardcoded project map here. Derive the routing each time:

1. **List the files the task touches** (git diff / planned edits).
2. **Group by language** (file extension: `.rs` / `.ts` `.tsx` / `.py` / `.c` `.cpp` / `.sh` / `.yaml` …).
3. **Map each language to its ruleset directory** under `.agents/rules/` (e.g. `rules/rust/` for `.rs`, `rules/common/` always).
4. **Flag boundary files** — FFI shims, protocol/schema definitions, embedded web assets — these need the contract rules in addition to the language rules.

Once the project defines its module matrix, record the concrete mapping in `.agents/memorys/` — not in this skill.

## Context Selection Protocol

### Phase 1: Identify the Scope

Before loading any rules, map the task to modules and languages:

```
Task → files touched → languages involved → applicable rulesets
```

**Example:**
```
Task: "Add a new WS message type"
→ shared protocol module + server handler (both .rs) + dashboard JS (if it consumes the message)
→ Rust: rules/rust/* ; protocol discipline: api-interface-design skill
→ JS side: rules/common/* + wire-format check against the Rust serde config
```

### Phase 2: Apply Rules Per Module

**NEVER** batch-apply all rules to all files. Each file gets its relevant subset:

```
for each file in scope:
  - Load rules/<language>/* matching the file extension
  - Load rules/common/* (security, coding-style, testing) always
  - Skip language-specific hooks for non-matching files
    (e.g. rules/rust/hooks.md must NOT fire on .ts files)
  - Apply platform/constraint rules only where relevant
```

### Phase 3: Verify Per Module

| Scope | Verification |
|-------|-------------|
| Any .rs change | `cargo clippy -p <crate> -- -D warnings` |
| FFI boundary change | `cargo check -p <crate>` with the relevant feature enabled + compile the native side |
| Web/dashboard change | Type-check (`npx tsc --noEmit` if TS) + visual QA via Playwright |
| Protocol/schema change | Serde roundtrip tests + E2E script |
| Full workspace | The project's aggregate check task (define in memorys once the stack exists) |

## Common Context Mistakes

| Anti-Pattern | Why Wrong | Fix |
|-------------|-----------|-----|
| Loading all language rulesets for a single-language task | Wastes context, confuses agent | Route by file extension (see Routing) |
| Applying C++ ownership rules to Rust code | Wrong paradigm | Rust borrow checker is the authority |
| Skipping platform constraint rules for FFI work | Misses linker/toolchain flags documented there | Always include constraints for FFI |
| Applying `rules/rust/hooks.md` to TypeScript files | Wrong hooks fire | Only load per-language hooks |
| Running the full workspace test suite for a single-module change | Slow | Scope to the affected package/crate |
| Not loading the project pitfalls memory for protocol/FFI work | Re-invents known wire-format and boundary bugs | Check `.agents/memorys/pitfalls.md` first |

## Multi-Language Boundary Rules

### Native (Rust) → C/C++ FFI
- Only plain bytes (`&[u8]`) and POD structs across the FFI boundary
- `unsafe` blocks require `// SAFETY:` comment
- Shared-pointer types need explicit thread-safety certification

### Rust → Protocol (WebSocket JSON)
- All messages use a declared serde tag convention (e.g. `#[serde(tag = "type", rename_all = "snake_case")]`)
- Browser clients must emit the exact same tags — verify both sides of the wire format
- Protocol changes go through the contract-first process (`api-interface-design`)
- E2E tests must validate new message types

### Backend → Embedded Web UI
- Decide the embedding strategy early (embedded assets vs separate deploy) and record it in project memory
- No shared type system across the boundary — validate the JSON envelope server-side

## Gate Checklist

Before submitting work touching multiple languages/modules:

```
[ ] file scope mapped, languages identified
[ ] only relevant rulesets loaded per file
[ ] per-module check passes (not just whole-workspace)
[ ] lints pass per module with -D warnings (Rust) / equivalent
[ ] platform constraints checked for gotchas
[ ] project pitfalls memory checked for known boundary patterns
[ ] protocol changes have backward-compat review
```

## Related Skills

| Skill | Relationship |
|-------|-------------|
| `think-before-act` | Context selection IS part of its "investigate before acting" discipline |
| `api-interface-design` | Context-engineering informs protocol contracts |
| `test-harness` | Cross-module testing needs correct per-module context |
| `lesson-memory` (C9) | New multi-language pitfalls → write to `pitfalls.md` |

## Task → Skill Routing

After recognizing user intent, proactively suggest loading the matching skill:

| Keywords/scenario | Suggested skill | Trigger condition |
|------------|---------|---------|
| "实现/添加/创建 + 功能" (implement/add/create + feature) | `incremental-implementation` | cross-file changes |
| "修复/bug/报错/不工作" (fix/bug/error/not working) | `systematic-debugging` | runtime errors |
| "怎么用/文档/API + 库名" (how-to-use/docs/API + library name) | `source-driven-development` | external dependencies |
| "写测试/测试失败" (write tests/tests failing) | `test-driven-development` | testing-related |
| "页面/UI/前端/仪表盘" (page/UI/frontend/dashboard) | `browser-testing` | browser changes |
| "安全/密钥/认证/漏洞" (security/secret/auth/vulnerability) | `security-hardening` | security-related |
| "审查/review/检查代码" (review/check code) | code-review rules | after code changes |
| "性能/慢/卡顿/延迟" (performance/slow/lag/latency) | `performance-optimization` | performance issues |
| "架构/设计/trait/协议" (architecture/design/trait/protocol) | `api-interface-design` | API design |
| "切换/上下文/语言" (switch/context/language) | `context-engineering` | multi-language tasks |
| "CI/CD/Docker/pipeline" | `ci-cd-automation` | CI changes |
| "简化/重构/清理" (simplify/refactor/cleanup) | `code-simplification` | reduce complexity |
| "优化 agent/审计/技能" (optimize agents/audit/skills) | `ecosystem-scan` | agent-system work |
| "总结/记录/教训/经验" (summarize/record/lessons/experience) | `lesson-review` | session end |
| before any non-trivial operation | `think-before-act` | automatic |
| "审计/一致性/文档矛盾" (audit/consistency/doc contradictions) | `doc-audit` | documentation audit |
| "方案/提案/标准化变更" (plan/proposal/spec-driven change) | `openspec-propose` | architecture changes |
| "实施/按方案/逐步" (implement/per-plan/stepwise) | `openspec-apply-change` | plan execution |
| "归档/完成变更" (archive/complete change) | `openspec-archive-change` | change archival |
| "探索/调研/思路" (explore/research/ideas) | `openspec-explore` | option research |
| "生成测试/测试骨架" (generate tests/test skeletons) | `test-harness` | test framework |
| "硬编码/密钥/端口扫描" (hardcode/secret/port scan) | `review-hardcode` | security scan |
| "文档转技能/书籍" (docs-to-skill/book) | `book-to-skill` | document conversion |
| "同步规格/delta" (sync specs/delta) | `openspec-sync-specs` | spec synchronization |

## Proactive Suggestion Mechanism

When the agent detects the keywords above but the matching skill is **not** loaded:
1. Briefly prompt: "This task may need the `X` skill — shall I load it?"
2. **Never force** — the user may skip
3. **Never repeat** — suggest a given skill only once per session
4. Suggestion format:

```
Suggestion: this task involves [scenario]; `skill(name="X")` can provide [value]. Want it?
```
