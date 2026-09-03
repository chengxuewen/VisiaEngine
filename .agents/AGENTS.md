# .agents — 项目记忆 / 规则 / 技能

Agent 工具链根目录（149 文件，全部 .md 或技能附属物）。装配点在 `.opencode/opencode.json`。

## STRUCTURE

```
.agents/
├── memorys/   # 易变事实：status(状态快照) conventions(C 约定) decisions(D 决策) pitfalls(PIT 踩坑)
├── rules/     # 恒常约束：common/(入 instructions) + <lang>/(按需，由层级见 rules/README.md)
└── skills/    # 22 技能目录，各以 SKILL.md frontmatter 注册；.skill_id 为安装元数据，勿动
```

## 加载机制（决定写作纪律）

| 层 | 何时进上下文 | 推论 |
|----|-------------|------|
| `memorys/status+conventions` + `rules/common` 12 文件 | **每轮全量**（opencode.json instructions，总 14 条） | 每行都有固定 token 成本；保持精简、不重复 skills 内容 |
| `rules/<lang>/` | 文件扩展名匹配时按需 | 可放语言细节 |
| `skills/*/SKILL.md` | frontmatter description 命中触发 | description 是路由命脉：通用方法论词，禁品牌残留 |
| `memorys/decisions+pitfalls` | 显式读取 | 可长，追加式，永不重写历史条目 |

## 编号纪律

- VisiaEngine 自己的 C{n}/D{n}/PIT-{n} 从模板起算（C2-C8、C10-C13 保留空号，见 conventions.md 头部声明）。
- **禁止**引用前项目的编号——新库无条目。前项目史在 `.refinfo/MediaServo/.agents/`（只读归档）。
- 五段式（症状/根因/解法/验证）与 D 条目格式已嵌在各模板注释，新增条目照抄骨架。

## 修改后验证

跑根 `AGENTS.md` COMMANDS 的 grep 门禁；frontmatter（`---` 对偶、description 单行 YAML）用 `head -6` 抽查。
