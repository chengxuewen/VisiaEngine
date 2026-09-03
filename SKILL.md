# VisiaEngine Skills Registry

本仓技能分两层：**全局插件技能**（不随仓库分发）与**项目技能**（`.agents/skills/`，随仓库版本化）。

## 全局插件技能

由 `superpowers`、`ponytail`、`oh-my-openagent`、`context-mode` 插件加载（装配点：`.opencode/opencode.json` → `plugin`）。通用方法论，适用所有项目，本文件不重复列举。

## 项目 Skills（22，位于 `.agents/skills/`）

路由触发面 = 各 SKILL.md frontmatter 的 `description`；代理按任务上下文自动激活，无需手动调用。

| Skill | 类型 | 说明 |
|-------|------|------|
| `think-before-act` | 元约束 | 先调研→列方案→用户审批→执行 |
| `skill-router` | 元约束 | 意图分析，输出推荐技能组合 |
| `ecosystem-scan` | 元约束 | .agents 体系审计 + 社区生态扫描 |
| `lesson-review` | 记忆 | 批量会话回顾，教训沉淀入 memorys |
| `doc-audit` | 文档 | 文档与架构自洽性审计（交互式） |
| `book-to-skill` | 文档 | 书籍/文档 → 结构化技能 |
| `openspec-explore` | 规范 | 变更前探索/思路澄清 |
| `openspec-propose` | 规范 | 生成 proposal + design + tasks 三件套 |
| `openspec-apply-change` | 规范 | 按 OpenSpec 任务实施 |
| `openspec-sync-specs` | 规范 | delta 规格同步主规格 |
| `openspec-archive-change` | 规范 | 完成变更归档 |
| `api-interface-design` | 工程 | 契约优先设计（trait / WS 协议 / REST） |
| `context-engineering` | 工程 | 多语言 workspace 上下文路由 |
| `incremental-implementation` | 工程 | 薄垂直切片实施循环 |
| `code-simplification` | 工程 | 过度工程识别与降复杂度 |
| `source-driven-development` | 工程 | 依赖决策以官方文档为准 |
| `performance-optimization` | 工程 | 延迟/吞吐/渲染/基准四维排查 |
| `ci-cd-automation` | 工程 | CI 门禁与构建矩阵管理 |
| `test-harness` | 测试 | 规格→测试骨架生成与验证 |
| `browser-testing` | 测试 | Web 面板 Playwright 验证 |
| `security-hardening` | 安全 | OWASP + 密钥扫描 + 审计流 |
| `review-hardcode` | 安全 | 硬编码快速扫描（已并入 security-hardening） |

## 与规则/记忆的分工

- **规则**（`.agents/rules/`）= 恒常约束，部分每轮加载（清单见 `.opencode/opencode.json` → `instructions`，14 条）
- **记忆**（`.agents/memorys/`）= 易变事实（C/D/PIT 编号体系）
- **技能**（本表）= 按需深度工作流
- 层级详情：根 `AGENTS.md` + `.agents/AGENTS.md`
