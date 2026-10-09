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

## C18: 共享件的边界 = 例子的教学主张（2026-10-08 band V 用户裁决 C2=A/C3=B）

**约束**: 把跨例重复的代码收进共享件时，只收**逐字相同且不含主张**的部分（设备/表面/上下文的开机白、
argv 解析）。例子用来教那一件事的代码（present 路径选择、鼠标键盘语义、像素谓词）**不得被共享吃掉**——
`render_view_rects` 是 ⑤b 的课文，drag 在 E402/E403 是拾取/框选而非轨道。判据：若某段代码删掉后
该例的 tutorials.md 主张就无人演示，它是课文不是重复。

**检查命令**: `grep -n '非目标\|留本地' .omo/plans/viewer-shared-band-2026-10-08.md | head`；
迁移批的门（确定性预检→sha/时序→交互三态）见 `rules/common/testing.md`「Batch-migration gates」。

## C19: 自检/剪枝/门禁类步骤的判据必须 token 级，且上线前跑破坏探针 (2026-10-08 band V 实锤)

**约束**: "这个符号还有人用吗""这条串还在吗"一类判据，按**文本行**判会被注释与字符串污染。
实锤：batch 12 的剪 import 只剥了 `use` 行，于是 `Window` 因注释提及而存活——抓它的是本仓
`lint`（`cargo clippy --workspace --all-targets -- -D warnings`），**不是脚本**；而当时我在提交里
把脚本能力写得比实现更好（声称"注释提及不算使用"）。两个失败同族：①判据太粗 ②自述超出实现。

正确判据次序：剥 `//` 注释 → 剥字符串字面量 → 剥 `use` 行 → 再按词边界查。
新增自检/门禁上线前必跑**破坏探针**（改名/删掉被测物，门禁必须红且报文只指它）。

**检查命令**:
```bash
# ① 谓词自证：给检查本身种破坏探针（命中>=1 才是真门；仓内必须为 0）
printf 's = re.sub(r"use [^\\n]+", "", s)\n' > /tmp/c19-probe.py
grep -c 're\\.sub(r\"use ' /tmp/c19-probe.py                       # 期望 1：谓词能命中粗判据形
grep -rn 're\\.sub(r\"use ' scripts examples/rs/src crates bindings 2>/dev/null | wc -l   # 期望 0
rm -f /tmp/c19-probe.py
# ② 反向优势（本仓真判据）：lint 段是 -D warnings，剪枝漏网必被它抓
grep -n '^lint = ' pixi.toml
```
**Note (honest, 2026-10-08)**: the band's codemod (`/tmp/migrate_viewer.py`, session-scoped
and NOT in the repo -- plans and one-off scripts are deliberately not tracked) still carries
the crude form at lines 207/211: it strips `use` lines only, no comment/string stripping.
The token-level fix was implemented in the batch 13/14 one-off invocations, whose corrected
predicates are likewise untracked. Therefore the durable guarantee is ② -- `lint` with
`-D warnings` -- which is what actually caught `Window` in batch 12. ① exists so the next
person who writes a prune step can prove their predicate bites before trusting it.


### C19 addendum (2026-10-08, adopted after lesson-review)

Adopted ruling: **`lint` is the durable gate for prune-type steps; no new CI segment.**
The discipline that was missing is "do not claim the script checks more than it does".

- Durable gate (verified in-repo): `pixi.toml` L76
  `lint = "cargo clippy --workspace --all-targets -- -D warnings"` — an unused import that
  survives a crude prune becomes an error here. This is what caught `Window` (batch 12).
- Self-test for any new prune/gate predicate (no /tmp dependency, run it in one line):
  `printf 's = re.sub(r"use x","",s)\n' > /tmp/p; grep -c 're\.sub(r"use ' /tmp/p; rm /tmp/p`
  — must print **1**; if it prints 0 the predicate is decorative and must be rewritten
  before it is written into any rule or script.
- Pointers added where they will be seen: `rules/common/testing.md` (Batch-migration
  gates) and `rules/rust/coding-style.md` (可执行检查 section) both now name `lint -D
  warnings` as the true judge of import-pruning steps.
