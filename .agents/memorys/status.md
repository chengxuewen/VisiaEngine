# VisiaEngine Status

**生成**: 2026-09-03 | Phase: 项目初始化 | 分支: main（首 6 提交见 git log，工作区 clean 为常态）

> 前身项目 MediaServo 的全部历史（D/PIT/C 编号体系、crate 矩阵、Phase 记录）见本机归档 `.refinfo/MediaServo/.agents/memorys/`（已被 gitignore，不随 clone 分发）。本目录自 2026-09-03 起为 VisiaEngine 从零积累。

## 概览

| 项 | 状态 |
|----|------|
| 技术栈 | ✅ Rust 核心 + wgpu 渲染（白皮书 v0.1.0，2026-09-03 定）；**后端 = D4 终审定案：wgpu 直用自研管线 `visia-render-wgpu`（不采用 Bevy）**；SDK 形态（C API FFI）；Open Core 商业模型 |
| 许可证 | ✅ 已落地：MIT OR Apache-2.0 双许可正本文件（LICENSE-MIT/LICENSE-APACHE），不可撤销承诺入 README |
| 源码 | 尚无 |
| 项目定位 | ✅ 多维空间可视化引擎（2D/2.5D/3D 统一，GIS/数字孪生/AV 仿真/BIM），非游戏引擎 |
| Agent 工具链 | ✅ 配置中性化 + 根 SKILL.md 技能注册表（22 项）+ 双层 AGENTS.md；Rust 规则回填 instructions 待执行 |

## Phase 状态

| Phase | 状态 |
|-------|:----:|
| 0 项目初始化（配置中性化/白皮书/架构基线/参考库/许可证） | ✅ |
| 1 MVP（visia-core / visia-render / visia-render-wgpu 骨架、glTF+GeoJSON 加载、2D↔3D 切换、宿主嵌入示例） | 📋 规划中（待动工指令） |
| 2 Alpha / 3 Beta / 4 1.0 | — 白皮书路线图 |

## 下一步

1. 环境初始化：全 pixi 方案（D5）已裁决——计划文档过审后待动工指令；随后 Cargo workspace 骨架 + 最小 CI
2. 动工日一次性回填（照抄 .refinfo/MediaServo 归档形态）：rust-toolchain/clippy/deny/tarpaulin.toml、check.sh、docs/modules/NN-*.md 编号设计文档群、docs/sdd/
3. 规则回填：rules/rust/{coding-style,hooks}.md 入 opencode.json instructions
4. push 至 gitee remote——待用户指令
5. 未决点 P1-P4（docs/architecture.md 末表）：RTC 粒度、style spec 兼容性、RK3588 驱动栈（商务输入）、材质 DSL（post-MVP）
