# VisiaEngine Status

**生成**: 2026-09-03 | Phase: 项目初始化 | 分支: main（首 6 提交见 git log，工作区 clean 为常态）

> 前身项目 MediaServo 的全部历史（D/PIT/C 编号体系、crate 矩阵、Phase 记录）见本机归档 `.refinfo/MediaServo/.agents/memorys/`（已被 gitignore，不随 clone 分发）。本目录自 2026-09-03 起为 VisiaEngine 从零积累。

## 概览

| 项 | 状态 |
|----|------|
| 技术栈 | ✅ Rust 核心 + wgpu 渲染（白皮书 v0.1.0，2026-09-03 定）；**后端 = D4 终审定案：wgpu 直用自研管线 `visia-render-wgpu`（不采用 Bevy）**；SDK 形态（C API FFI）；Open Core 商业模型 |
| 许可证 | ✅ 已落地：MIT OR Apache-2.0 双许可正本文件（LICENSE-MIT/LICENSE-APACHE），不可撤销承诺入 README |
| 源码 | crates/ 3 crate（visia-core / visia-render / visia-render-wgpu），~1.1k 行，20 cargo 测试 + L2 smoke，9 测试套件绿 |
| 项目定位 | ✅ 多维空间可视化引擎（2D/2.5D/3D 统一，GIS/数字孪生/AV 仿真/BIM），非游戏引擎 |
| Agent 工具链 | ✅ 配置中性化 + 根 SKILL.md 技能注册表（22 项）+ 双层 AGENTS.md；Rust 规则回填 instructions 待执行 |

## Phase 状态

| Phase | 状态 |
|-------|:----:|
| 0 项目初始化（配置中性化/白皮书/架构基线/参考库/许可证） | ✅ |
| 1 MVP | 🔨 开工骨架完成（S0-S4 ✓：workspace/场景图 v0/渲染契约/wgpu 实例+L2 example/离屏 golden）；glTF/GeoJSON/2D-3D 切换 = 下轮功能片；glTF/GeoJSON/切换为后续片） |
| 2 Alpha / 3 Beta / 4 1.0 | — 白皮书路线图 |

## 下一步

1. **MVP 功能片规划轮**（glTF+GeoJSON 加载 / 2D↔3D 切换演示 / 宿主嵌入示例——白皮书 MVP 定义 → 新计划文档 → Momus 审）
2. P1 裁决（RTC 粒度）= core 坐标系功能片前置；届时补重基测试（SDD core.md 已留位）
3. CI 激活：GitHub 镜像仓决策日（ci.yml 已待命）；Gitee remote push 待指令
4. wgpu 升级窗口（季度）：重跑 PIT-3 破坏面清单
5. 环境/许可证/工具链条款不变（D5）；wasm/GDAL/Qt 到货日见 pixi.toml 注释与架构⑦

## 开工骨架基线（2026-09-03）

`pixi run ci` 全绿（fmt/lint/check/test/audit）· spec-trace 20↔20 双向对齐 · 不变式①②机器门禁 PASS · golden L1 本机 lavapipe 实跑 · L2 待 CI xvfb · 提交面 feat(S0-S4)+test(RED)×5=10 笔（S2 RED 拆两笔，规则17收口，K4 偏差已记账）
