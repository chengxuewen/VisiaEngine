# VisiaEngine 约定与约束

> This file is VisiaEngine-only. C2-C8 and C10-C14 are reserved slots (C1/C9 inherited from the predecessor project's general methodology, kept to avoid dangling cross-references; C15/C16/C17 active). New conventions take the smallest free slot. Full predecessor archive: `.refinfo` (read-only).

## C1: 架构决策对比格式

**约束**：任何涉及方案选择的架构讨论，必须逐项列出：
- **优缺点**：每个方案的优点和缺点
- **来源/参考**：借鉴的现有系统/开源项目/行业实践
- **影响**：选择该方案对后续开发的影响
- **推荐**：明确推荐及理由

禁止仅列举选项让用户选择而没有上述分析。

## C9: 经验教训自动沉淀

**约束**：开发过程中发现的问题、教训、经验必须在当轮会话中主动更新到相应记忆文档。

| 情况 | → 更新文件 |
|------|-----------|
| 发现 bug / 踩坑 | `pitfalls.md`（症状+根因+解法+验证，缺一不可） |
| 用户纠正 AI 行为 | `conventions.md`（新约束，C{n} 编号） |
| 架构/配置决策 | `decisions.md`（D{n} 编号 + 原因 + 影响） |
| 项目状态变化 | `status.md`（日期、Phase、测试数） |
| 编码模式/反模式 | `rules/common/coding-style.md` |
| 安全相关教训 | `rules/common/security.md` |
| 编辑工具使用教训 | `rules/common/edit-safety.md` |

**原则**：不等用户要求。识别到可沉淀的经验即主动更新。宁可多记，不可遗漏。

## C14: 子代理产物必须验证（编排者铁律）

**约束**：子代理返回的完成声明不可信。编排者必须验证实际产物后才标记任务完成。

**验证清单**：
- 声称创建的文件 → `cat`/`ls` 确认存在 + 内容完整
- 声称修改的配置 → `grep` 关键字段
- 声称可运行的命令 → 实际执行
- 声称通过的测试 → 重新运行

**失败处理**：验证失败 → 携带具体缺陷续跑同一子代理会话修复（task_id 续接），不自行接手编辑。

## C15: 接口哨兵与参数语义先对既有条款分工表

**约束**: ①新接口定「失败/无效哨兵」前，grep 条款库既有值分工（本案：`add_mesh` 拿 0 当失败哨兵，撞 CAPI-01 白纸黑字「实体位形 slot0gen0=0 合法=有意分工」——改返回码+out 谱才归位）；同一值域在两处含义相反=必埋雷。②一个参数面不侍二主：CI 语义（快进快出 `--frames`）与交互语义（常驻窗）必须分挂两个消费者（run 步骤零参/注册表 argv 专属 ctest 之判）；出现「同一份参数两种甲方」即为设计报警。③检查法：新口 RED 前先写一段「值域分工对照」进条款草稿（谁拥有 0/-1/MAX 的解释权）。
**检查命令**: `grep -n '哨兵\|MAX\| 0=' docs/sdd/capi.md | head`（新条款入库前值域表无撞位）。

## C16: 决策呈交格式＝人话＋示意＋目录＋逐项

**约束**: 凡需用户裁决的方案/缺陷/影响分析，按四件套呈交：①说人话根因（先给「一句话版本」）；②示意图（数据/流程/时线 ASCII，能画就画）；③受影响**目录结构/文件清单**逐项（含落点行位）；④多议题时逐项过（一卡一裁决），禁一次性大列表。术语堆叠版会被打回（本会话「说人话」×6 实锤）。推荐理由复述用 C1 四栏（优缺点/来源/影响/推荐）。

## C17: English-only for all persisted artifacts (2026-09-20 user ruling)

**Constraint**: Every artifact written to the repository MUST be in English — code comments (`//`, `///`, `/* */`, `#`), SDD contract clauses, `docs/`, `.agents/memorys/`, `.agents/rules/`, `.agents/skills/`, AGENTS.md, README, commit messages, example/demo source, config comments. The ONLY exception: live AI↔user chat replies (Chinese allowed when the user writes Chinese).

**Scope**: Prospective only (user chose A). The ~7,300 pre-existing Chinese lines (measured 2026-09-20) stay as-is; translate opportunistically when touching a file, no dedicated migration wave.

**Why**: repo is public-facing SDK distribution; mixed-language corpus is a liability for external contributors and for grep/CI gates that anchor on comment text.

**Check command**:
```bash
# new/changed .rs comments must be ASCII (run on staged diff; 0 matches expected)
git diff --cached -U0 -- '*.rs' | grep -P '^\+.*[\x{4e00}-\x{9fff}]' || echo OK
```
