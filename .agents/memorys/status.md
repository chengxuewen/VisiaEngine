# VisiaEngine Status

**生成**: 2026-09-03 | Phase: 项目初始化 | commits: 0 | 分支: main

> 前身项目 MediaServo 的全部历史（D/PIT/C 编号体系、crate 矩阵、Phase 记录）见本机归档 `.refinfo/MediaServo/.agents/memorys/`（已被 gitignore，不随 clone 分发）。本目录自 2026-09-03 起为 VisiaEngine 从零积累。

## 概览

| 项 | 状态 |
|----|------|
| 技术栈 | ✅ Rust 核心 + wgpu 渲染（白皮书 v0.1.0，2026-09-03 定）；**后端 = D4 终审定案：wgpu 直用自研管线 `visia-render-wgpu`（不采用 Bevy）**；SDK 形态（C API FFI）；Open Core 商业模型 |
| 源码 | 尚无 |
| 项目定位 | ✅ 多维空间可视化引擎（2D/2.5D/3D 统一，GIS/数字孪生/AV 仿真/BIM），非游戏引擎 |
| Agent 工具链 | ✅ 移植配置已中性化（2026-09-03）；Rust 规则回填 instructions 待执行 |

## Phase 状态

| Phase | 状态 |
|-------|:----:|
| 0 项目初始化（配置中性化） | ✅ |
| 1 MVP（visia-core / visia-render / visia-render-wgpu 骨架、glTF+GeoJSON 加载、2D↔3D 切换、宿主嵌入示例） | 📋 规划中（待动工指令） |
| 2 Alpha / 3 Beta / 4 1.0 | — 白皮书路线图 |

## 下一步

1. ~~白皮书入库~~ ✅ docs/whitepaper.md + README.md（2026-09-03，修正①②⑤⑥后入库）
2. 建 Cargo workspace 骨架（visia-core / visia-render / visia-render-wgpu，依 D4）——待用户裁决动工
3. 规则回填：rules/rust/{coding-style,hooks}.md 入 instructions（栈=确认后解锁）
4. 四路调研全部交付 ✅；**D4 已拍板：B 案 wgpu 直用自研（2026-09-03）**，legacy 裁定（XP 撤出/Win7 best-effort/Android API24/RK3588 锁单一驱动栈）已入 decisions.md D4，白皮书/README 已回填
5. ✅ docs/reference/ 参考项目库 15 篇 + 索引（2026-09-03 实测数据快照）——供 D4 终裁与 MVP 设计输入
