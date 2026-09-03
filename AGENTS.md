# PROJECT KNOWLEDGE BASE

**Generated:** 2026-09-03 | **Branch:** main | 仓库已初始化：6 提交（见 git log），工作区 clean

## OVERVIEW

VisiaEngine（维视引擎）— 多维空间可视化引擎：统一 2D/2.5D/3D 渲染管线，面向 GIS/数字孪生/自动驾驶仿真/BIM 展示，以 SDK 形态（C API FFI）嵌入 Qt/Flutter/C#/Web，Open Core 模式。技术栈 2026-09-03 白皮书 v0.1.0 定案：**Rust 核心 + wgpu 渲染**（D4 终审：wgpu 直用自研管线 `visia-render-wgpu`，不采用 Bevy）。**开工骨架已落地（S0-S4，2026-09-03）**：三 crate workspace + 20 测试 + golden/不变式/追溯三门禁；agent 配置由前身项目 MediaServo（Rust WebRTC，栈不同勿混淆）移植并已中性化。

## STRUCTURE

```
./
├── Cargo.toml/lock   # workspace（crates/* 三成员，版本 pin 见计划锚点）；deny.toml licenses/bans
├── crates/           # visia-core（场景图/坐标，永无渲染依赖）→ visia-render（trait+IR 契约）→ visia-render-wgpu（wgpu 后端+examples+offscreen golden）
├── docs/sdd/         # 行为契约条款（CORE/REND/WGPU-NN，与测试 // spec: 双向追溯：scripts/spec-trace.sh）
├── .github/workflows # ci.yml 待命（GitHub 镜像日激活；本机等价=pixi run ci+同款 grep）
├── .agents/          # 项目记忆+规则+技能（见其 AGENTS.md）
├── .opencode/        # opencode.json（instructions/MCP/LSP 装配点）+ init-mcp-*.mjs / init-lsp-wrap.mjs 桥脚本
├── .omo/omo.jsonc    # agent 模型分层/team 配置；.gitignore 排除 .omo/*，仅 omo.jsonc 入库
├── .refinfo/         # ⚠ MediaServo 完整归档（21M，gitignored，本机独有）— 只读，永不编辑
├── docs/             # whitepaper.md（定位一手事实源）+ architecture.md（v0.1 设计基线，D4 对齐）+ reference/（参考项目库 + evidence/ 证据快照）
├── pixi.toml/lock    # D5 环境单源（conda-forge 全锁含 rust 工具链）；bootstrap.{sh,bat} 首启 / pixi.{sh,bat} 激活
├── LICENSE-MIT / LICENSE-APACHE   # 双许可正本（不可撤销承诺见 README）
├── SKILL.md          # 项目技能注册表（22 项）
└── README.md         # 白皮书摘要版
```

有完整工程层：Cargo workspace（3 crate，~1.1k 行）+ pixi 环境（D5）+ SDD 契约 + CI 待命 + 双许可。Phase 1 MVP 功能片（glTF/GeoJSON/2D-3D）未动工——待新计划轮。

## WHERE TO LOOK

| 任务 | 位置 | 备注 |
|------|------|------|
| 每轮会话加载了什么上下文 | `.opencode/opencode.json` → `instructions`（恰 14 条） | 新增条目 = 每轮固定 token 成本，先证明"每轮必需"再加 |
| 项目状态/约定/决策/踩坑 | `.agents/memorys/` | 模板态，从零积累；编号体系见其 AGENTS.md |
| 语言规则 | `.agents/rules/<lang>/` | rust/{coding-style,hooks} 已入 instructions（16 条）|
| 技能 | `.agents/skills/*/SKILL.md` | 22 个；frontmatter description = 路由触发面 |
| 前项目 MediaServo 的任何历史 | `.refinfo/MediaServo/` | 决策史/踩坑史/完整 rules——查证用，禁止引用编号进新仓 |

## CODE MAP

`Scene/EntityId`(crates/visia-core/src/scene.rs：Vec+free-list+代际+脏标记，spike 实测 100k 实体 ~12ms) → `RenderBackend/Frame/DrawCommand`(crates/visia-render/src/contract.rs，object-safe，测试内 stub=不变式②构造证明) → `create_instance/available_adapters/render_offscreen_triangle`(crates/visia-render-wgpu/src/，examples/clear.rs L2)。依赖单向：core ← render ← render-wgpu；wgpu 类型止步后端 crate。

## CONVENTIONS

- C1（方案对比格式）/ C9（教训即时沉淀）/ C14（子代理产物必验证）在 `.agents/memorys/conventions.md`，随 instructions 每轮加载——本文件不复述。
- `Cargo.lock` 必入库等构建约束在 `rules/common/constraints.md`（承自前项目，若选非 Rust 栈需重审）。
- remote 为 gitee VisiaEngine——commit 已获首轮授权（2026-09-03），push 与后续提交仍逐次等用户显式指令。

## ANTI-PATTERNS (THIS PROJECT)

- **NEVER** 编辑 `.refinfo/`（归档只读）或将其 `git add`。
- **NEVER** 在新文档引用 MediaServo 的 D-nnn/PIT-nn/C-nn 编号——新库无对应条目，悬空引用=幻觉源。改配置后跑下方 grep 门禁。
- **NEVER** 在用户显式裁决前建脚手架/Cargo workspace——栈与后端已定 ≠ 授权动工（提交/落地动作始终等用户指令）。项目定位与栈的一手事实以 `.agents/memorys/decisions.md` D2/D4 + 白皮书为准。
- **子代理派发限制**（本会话实证）：fast 层模型别名网关失效（`Model not exist`）→ explore/librarian/quick 类会死。可用：`deep`/`unspecified-high`/`visual-engineering`/`writing`（premium）。
- **NEVER** 无差别 `cargo fmt`（workspace 格式漂移史）——单文件用 `rustfmt --edition <ed> <file>`。

## UNIQUE STYLES

- 配置即产品：`.agents` 三层（memorys=易变事实 / rules=恒常约束 / skills=按需深度）与 opencode.json 装配点的分层纪律是本仓的核心不变式。

## COMMANDS

```bash
# 配置卫生门禁（改 .agents/.opencode 后必跑，期望：仅 memorys 归档指针/历史条目命中）
grep -rniE 'mediaservo|audemsp|mediasoup|omsp|msrtc|oxmgr' .agents .opencode --include='*.md' --include='*.json' | grep -v '.refinfo'
grep -rnE 'PIT-[0-9]+|D1[0-9][0-9]|D2[0-9][0-9]' .agents/rules .agents/skills
python3 -m json.tool .opencode/opencode.json >/dev/null && echo json-OK
```
bash bootstrap.sh    # 首次环境初始化（幂等，实测二跑 0.24s；pixi 钉 0.78.0）
source pixi.sh       # 日常激活（或单命令 pixi run <task>）
pixi run ci          # fmt+lint+check+test+audit 聚合（开工门禁）
pixi run <check|build|test|lint|fmt|audit|verify>   # 单任务
bash scripts/spec-trace.sh    # SDD↔测试双向追溯

## NOTES

- `.sisyphus` 在 `.gitignore` L92 预留（plans 不落库）；`www/` 并无预留——工程未建，勿假设目录存在。
- MCP：local-github 需 `GITHUB_TOKEN` 环境变量；local-playwright/postgres/websearch 默认禁用，按需开启。
- Rust 栈回填点（解锁待执行）：instructions += `rules/rust/{coding-style,hooks}.md`；`rules/rust/testing.md` L38 placeholder 校准；docker.md 保留为按需参考。
