# Bevy — 后端候选 A（**D4 终裁未采纳**，2026-09-03：采纳 B 案 wgpu 直用自研）

**快照 2026-09-03 实测**：48k★；最新 0.19.1（2026-08-13）；main=0.20.0-dev，MSRV 1.96。MIT OR Apache-2.0。

> 全文证据（逐条 URL、迁移指南计数、issue 编号）：[2026-09-03-bevy-embed.md](evidence/2026-09-03-bevy-embed.md)

## 画像
Rust 全栈 ECS 游戏引擎（Bevy Funded，cart 全职+志愿者）。渲染自研于 wgpu 之上。**官方 README 自述破坏性节奏 ~3 月一次；迁移指南序言自称"experimental phase"；sitemap 1374 页无 1.0 时间表。**

## 关键实测结论（D4 终裁输入，逐条有 URL，存于会话备忘录）
1. **bevy_render 与 App/ECS 不可分，且 0.19 更深**：#22144 RenderGraph→ECS Schedule（v0.19.0 源码树已无 render_graph/ 目录）；无"device+surface+scene→一帧"函数式入口。
2. **宿主主循环**：官方 `externally_driven_headless_renderer` 2026-01 出生、2026-07 还在修渲染同步 panic；**宿主窗口（Qt widget/C# 容器）内渲染：零第一方/零第三方先例**。godot-bevy（540★）刻意只用 Bevy ECS、渲染交给 Godot——最强旁证。
3. **破坏面**：0.18→0.19 = 120 个小节；生态插件锁步 hard-block 实案（SpawnForge 五方同 PR）。
4. **能力面正面清单**：`Projection` enum 双投影运行时切换（projection_zoom 示例）、自定义 render phase 挂点、65,536 实例自动合 1 draw call、0.19 主题 "Render Bigger Scenes Faster"。
5. **体积**：无 native 受控对比数据（本机无 cargo，标记必测 spike）；wasm 样本 jlg.io 自报 13MB/3.4MB gzip。
6. **wasm**：#4078 线程仍 open；0.20 正在 WGSL→WESL（shader 接口又变）。

## 对 VisiaEngine
- **若选**：换来 ~一个 release 周期的渲染 API 手术 + "稳定 C ABI 下完全屏蔽 bevy 类型" 的隔离纪律 + 放弃插件升级红利。
- **借鉴无论选否**：extract/prepare/queue 三相数据流（主世界→渲染世界复制）对动态数据源（孪生 IoT）架构极有价值；自动实例化合批设计；feature 集合（2d_api/3d_api 不含 render backend = "自定义渲染器"作为一等公民的姿态）。
- **规避**：Reflection 不可裁剪（#20337）对 ≤10MB 是逆风；渲染架构文档章"hidden"（未定稿）——把文档不稳定当信号读。

## 来源
bevy.md 内联全部 URL：github releases API / bevy.org 迁移指南与 book / docs.rs/bevy_render/0.19.1 / project-forge #8887 / jlg.io PR #550。
