# PROJECT KNOWLEDGE BASE

**Generated:** 2026-09-03（批 5 校正 2026-09-14）| **Branch:** main | 提交/工作区状态以 `git log`/`git status` 实测为准

## OVERVIEW

VisiaEngine（维视引擎）— 多维空间可视化引擎：统一 2D/2.5D/3D 渲染管线，面向 GIS/数字孪生/自动驾驶仿真/BIM 展示，以 SDK 形态（C API FFI）嵌入 Qt/Flutter/C#/Web，Open Core 模式。技术栈 2026-09-03 白皮书 v0.1.0 定案：**Rust 核心 + wgpu 渲染**（D4 终审：wgpu 直用自研管线 `visiaengine-render-wgpu`，不采用 Bevy）。**批 0-5 已收官（2026-09-14）**：八 crate workspace、**125 条** SDD 契约（spec-trace 双向锁）、ctest 统一例子清单（23 条）+ 四 gate（style/trace/abi/docs）+ cmake-smoke 三态三锚；golden 真机无 SKIP；agent 配置由前身项目 MediaServo（Rust WebRTC，栈不同勿混淆）移植并已中性化。

## STRUCTURE

```
./
├── Cargo.toml/lock   # workspace（members=crates/*+examples/rs+bindings 两 FFI crate，S1/S2 定）；deny.toml licenses/bans
├── crates/           # 纯核心 6：core→render（trait+IR/camera/rebase）→render-wgpu（wgpu 后端/管线/offscreen）+ io-gltf + io-points（PLY 点云，IO-* 族）+ geo（GeoJSON→3857→细分→样式/GPU 扩片输出 GEO-24）
├── examples/         # 例子按语言：rs=cargo example 教程系×12（[[example]] 注册壳包）；c=C 例（E701/702；S4 起 E8xx+template）；cpp/qt 随 S4 开；跑面单源=ctest/pixi smoke
├── bindings/         # 实现面一窝同栖：c/visiaengine-capi（C ABI 21 入口+手写头+CAPI 条款测试）· c/probe · cpp/include（S4 hpp）· qt/widget.hpp · js/rust/visiaengine-wasm（CAPI-09 镜像）+ js/demo
├── docs/sdd/         # 行为契约条款（CORE/REND/WGPU-NN，与测试 // spec: 双向追溯：scripts/spec-trace.sh）
├── docs/tutorials.md # E 编号教程索引（文件名=头注=索引三方锁=scripts/gate-docs.sh）
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

工程层：Cargo workspace（7 crate，规模以 `cargo test --workspace` 实报为准）+ pixi 环境（D5）+ SDD 契约 + CI 待命 + 双许可。Phase 1 MVP 已收口（批 2 宿主嵌入/批 7 Web/批 4 渲染强化/批 5 文档归位）；批 4g/4h 与 Alpha 档（Qt widget、npm/pip 打包、真 PBR）候令。

## WHERE TO LOOK

| 任务 | 位置 | 备注 |
|------|------|------|
| 每轮会话加载了什么上下文 | `.opencode/opencode.json` → `instructions`（恰 14 条） | 新增条目 = 每轮固定 token 成本，先证明"每轮必需"再加 |
| 项目状态/约定/决策/踩坑 | `.agents/memorys/` | 模板态，从零积累；编号体系见其 AGENTS.md |
| 语言规则 | `.agents/rules/<lang>/` | rust/{coding-style,hooks} 已入 instructions（16 条）|
| 技能 | `.agents/skills/*/SKILL.md` | 22 个；frontmatter description = 路由触发面 |
| 前项目 MediaServo 的任何历史 | `.refinfo/MediaServo/` | 决策史/踩坑史/完整 rules——查证用，禁止引用编号进新仓 |

## CODE MAP

`Scene/EntityId`(core/src/scene.rs：slab+代际+脏标记，100k 实体 ~12ms spike 实测) → `RenderBackend/MeshDesc/CameraRig`(render/src/{contract,camera}.rs：object-safe，深度变体锁 [0,1] 见 PIT-5) → `MeshCore/headless/offscreen`(render-wgpu/src/：真网格管线+立方 golden) + `load_gltf`(io-gltf/src/lib.rs：gltf from_slice+util::Iter，GLB-only)。例子域（v1.3 起）：examples/rs ×12 cargo example（[[example]] 注册壳包，ctest 转发壳跑）+ examples/{c,cpp,qt} 原生真身（E70x/E80x）。依赖单向 core←{render, io-gltf}←render-wgpu(dev 合流)。

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
- MCP（2026-09-03 修复轮后）：nodejs 在 pixi 默认环境，全部 local 桥经 `bash .opencode/with-node.sh` 拉起；local-github 仍需 `GITHUB_TOKEN` 环境变量（environment 键，勿写 env）；local-playwright/postgres/websearch/openspace 默认禁用，按需开启（openspace 另需 python 树+LLM key）。
- Rust 栈回填点（解锁待执行）：instructions += `rules/rust/{coding-style,hooks}.md`；`rules/rust/testing.md` L38 placeholder 校准；docker.md 保留为按需参考。
