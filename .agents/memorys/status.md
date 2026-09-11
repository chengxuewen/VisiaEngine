# VisiaEngine Status

**生成**: 2026-09-03 | Phase: 项目初始化 | 分支: main（首 6 提交见 git log，工作区 clean 为常态）

> 前身项目 MediaServo 的全部历史（D/PIT/C 编号体系、crate 矩阵、Phase 记录）见本机归档 `.refinfo/MediaServo/.agents/memorys/`（已被 gitignore，不随 clone 分发）。本目录自 2026-09-03 起为 VisiaEngine 从零积累。

## 概览

| 项 | 状态 |
|----|------|
| 技术栈 | ✅ Rust 核心 + wgpu 渲染（白皮书 v0.1.0，2026-09-03 定）；**后端 = D4 终审定案：wgpu 直用自研管线 `visiaengine-render-wgpu`（不采用 Bevy）**；SDK 形态（C API FFI）；Open Core 商业模型 |
| 许可证 | ✅ 已落地：MIT OR Apache-2.0 双许可正本文件（LICENSE-MIT/LICENSE-APACHE），不可撤销承诺入 README |
| 源码 | crates/ **5** crate（core/render/render-wgpu/io-gltf/geo），~4.7k 行，**68 cargo 测试全绿** + 4×L2 smoke 接线；离屏 golden 9/9 本机 lavapipe 无 SKIP；fixture 已迁 `resources/data/`（1b） |
| 项目定位 | ✅ 多维空间可视化引擎（2D/2.5D/3D 统一，GIS/数字孪生/AV 仿真/BIM），非游戏引擎 |
| Agent 工具链 | ✅ 中性化+注册表+双层 AGENTS；Rust 规则已入 instructions（16 条）；MCP 修复轮：nodejs 入 pixi 环境+with-node 包装器，codegraph/github 通路绿，openspace 归按需族（PIT-6） |

## Phase 状态

| Phase | 状态 |
|-------|:----:|
| 0 项目初始化（配置中性化/白皮书/架构基线/参考库/许可证） | ✅ |
| 1 MVP | 🔨 内容轮+GeoJSON 片完成（G1-G4 ✓；H1-H4 ✓：geo 解析/细分样式/D7 落地/geo_viewer）；宿主嵌入/capi = 后续片） |
| 2 Alpha / 3 Beta / 4 1.0 | — 白皮书路线图 |

## 下一步

1. **push gitee**（47 笔单磁盘风险持续）+ GitHub 镜像决策（ci.yml 四步 L2 smoke 即转现役）——**用户侧另需 GITHUB_TOKEN+重启验 MCP**
2. ~~P1 裁决~~ ✅ D7 已裁 + **H3 已实施**（compose_mvp 落地，WGPU-10 远原点像素一致实证）
3. **宿主嵌入片**（rwh 路径 A + Qt demo conda qt-main 环境扩张单独立项；spike-3）
4. capi 片（ABI 面 D6 已锁：visiaengine_*/visiaengine.h）；纹理/材质 PBR、屏幕空间线宽、MVT/tile 流式（⑦）= Alpha 档
5. P2 样式 spec 兼容性裁决（simplestyle 六键之上：MapLibre v8 子集？）——样式系统设计轮
4. CI 激活：GitHub 镜像仓决策日（ci.yml 已三连 smoke）；Gitee push 待指令
5. wgpu 升级窗口（季度）：重跑 PIT-3/PIT-5 破坏面清单
6. 环境/许可证条款不变（D5/D6）；镜像 CI 待命期本机 `pixi run ci` 为唯一门禁（G 轮实证）

## MVP 内容轮基线（2026-09-03，G1-G4）

`pixi run ci` 全绿 · spec-trace **66↔66**（CORE/REND/WGPU/GLTF/GEO 五命名空间）· 立方 golden 7/7 本机 lavapipe 真机 · L2 三 smoke 接线（运行绿地点=CI 待命）· 提交面 11 笔（计划 9 + S2/G1 补采 + G4 拆 b，K5 弹性记账）· PIT-5 入档（wgpu [0,1] 静默裁剪）

## GeoJSON 片基线（2026-09-03，H1-H4）

geo crate 16 测试（GEO-01..14）· D7 全链：`tessellate` 输入 local（`GeoKind::shifted` 原语）+ `rebase::compose_mvp` f64 相减 + 远原点立方/geo 全链路像素级验证 · Web Mercator 球面式（EPSG:3857 定义语义入 SDD）· geo_viewer example + 第 4 路 smoke · 提交 12 笔（计划 8+RED 补采/H1 收口各 1，弹性条款覆盖）· 事故三笔如实记于 message（双 view_rotation 并行编辑 / contract 批内未写盘 / 51 声称口径）

## 计划 v1.1 批次 0/1 基线（2026-09-03，Easy3D 借鉴移植开工轮）

- 批次 0：测试三层法(T1/T2/T3)+示例门禁+D8 预登记入 rules/decisions（`0464fd2`）
- 批次 1a：LoadReport+RepairPolicy 五类分型（GEO-15/16）+ GLTF-09 模式过滤（**顺带修 POINTS 假三角存量 bug**）（RED `6892b81`+GREEN `07999cd`）
- 批次 1b：testdata→resources/data 迁移 14 处同步，grep 清零（`229f77f`）
- 基线：66↔66 · 68 测试 · ci exit=0 · golden 无 SKIP；边界事实：JSON 数字超域在文档层拒=双策略 Err（条款注记）；非有限坐标 parse 入口不可达（字段留 web_mercator 直调路径）
- 计划文档 `.omo/plans/easy3d-adoption-plan.md`（v1.1 已批准）；下一步=批 2 计划轮（宿主嵌入，含 D8 转正）或批 3（∥3c→3b）+ 批 3.5 交互片

## 批次 3a 基线（2026-09-03，AttrSet 属性列化轮）

- core::AttrSet（CORE-11/12/13）：三型闭合列存、行对齐不变式、缺失≠零值、首写定型
- geo（GEO-17/18）：props 解析后不再丢弃（宿主查询口打开）；样式经 typed 列读，style.rs 零 JSON 依赖（gate-style 任务入 ci 链）
- 基线：**71↔71 · 74 passed · ci exit=0 · golden 9/9 无 SKIP**；RED `070ce59` GREEN `28c250f`
- 教训两处如实：GREEN 回归网抓到 feed 普通件分支绕过 accept 分类器；grep 门禁谓词首版被自家注释文本绊倒（谓词收窄至代码形态）
- 3c 已裁：**D9=B**（v8 paint 别名子集，主键优先/表达式边界冻结）→ GEO-19 实施（`str_first/f64_first` 回退映射）
- 基线刷新：**72↔72 · 76 passed · ci exit=0（含 gate-style 段）· golden 9/9 无 SKIP**
- 下一步：3b 标量→LUT（键 schema 已按 D9 就绪）；批 2 计划轮候令
