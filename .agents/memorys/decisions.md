# VisiaEngine 架构决策记录

> 记录影响接口、数据流、依赖方向或构建形态的决策。每条含编号、日期、决策内容、原因、影响。方案对比的呈现格式遵循 C1（优缺点/来源/影响/推荐四要素）。被推翻的决策标注 SUPERSEDED 与新编号，不删除条目。

## 格式示例

## D1: <决策标题> (YYYY-MM-DD)
- **背景**: <为什么需要这个决策>
- **决策**: <选了什么，弃了什么>
- **原因**: <关键理由，含来源/参考实践>
- **影响**: <波及的模块、约定、后续开发>

<!-- 自此往下追加真实条目。前项目 MediaServo 决策史见 .refinfo 归档。 -->

## D1: 移植配置中性化（MediaServo → VisiaEngine 全新起点） (2026-09-03)
- **背景**: 仓内 .agents/.opencode/.omo 整体移植自已完成项目 MediaServo（Rust WebRTC），3437 行旧项目记忆 + 14 个硬引用技能会误导新项目。
- **决策**: 定位为全新项目→记忆清零为模板（仅沉淀 C1/C9/C14 三条通用方法论，保留原编号防悬空引用）；14 技能原地中性化改写不删除；rules 定点通用化 + platform.md 删除；instructions 去 docker/platform/rust（栈未定不预载语言规则）。
- **原因**: 旧项目历史已完整归档于 .refinfo/MediaServo（gitignore，本机），覆盖/删除零丢失。
- **影响**: 每轮会话加载的配置不再包含 MediaServo 架构；技术栈选定后按占位处回填（见 status.md 下一步）。验证：双 grep 门禁通过。

## D2: 技术栈与产品形态定案（白皮书 v0.1.0 初稿） (2026-09-03)

> **更新**：本条 impact ④ 的渲染后端终审已由 **D4** 取代（采纳 wgpu 直用自研）。
- **背景**: 用户提交《VisiaEngine 白皮书》初稿，定位多维空间可视化引擎（2D/2.5D/3D 统一视口，面向 GIS/数字孪生/AV 仿真/BIM），明确非游戏引擎。
- **决策**: Rust 内核（`visia-core` 场景图/坐标系/空间索引 + `visia-render` 抽象层）；渲染基于 wgpu（Vulkan/Metal/DX12/WebGPU），白皮书暂定 Bevy 为默认后端（`visia-render-bevy`）；C API 为唯一稳定 FFI 边界，交付 SDK 形态不绑架宿主主循环；Open Core：核心永久 MIT/Apache-2.0，商业产品 = Visia Studio 编辑器 + 企业插件/支持；启动体积目标 ≤10MB。
- **原因**: 填补游戏引擎（重、GIS 弱）与 GIS/Web 库（难嵌原生桌面、2D/3D 不同核）之间的空档；对标先例：Cesium（统一视口但 JS 为主）、MapLibre（纯地图）、Three.js（无空间数据层）。
- **影响**: ① 栈=确定，rules/rust 回填 instructions 解锁待执行；② crate 命名 visia-* 定，与前项目 MediaServo 完全无关（仅移植了工具链配置）；③ 白皮书 §3.2"原生解析 ODR/OSC"与 §6.2"Beta 插件"承诺级别矛盾，发布前须统一；④ **Bevy 绑定方式为未终审承重决策**（bevy_render 深度 ECS 耦合，"解耦 ECS 用其渲染框架"现实上近不可行）；⑤ 白皮书全文当前仅存于会话，待裁决入库位置。

## D3: 白皮书 v0.1.0 入库与措辞修订 (2026-09-03)
- **决策**: 用户白皮书初稿经审修后入库 docs/whitepaper.md + README.md 摘要版。修订范围（用户批准）：①§4 去除"Bevy 默认后端"承诺，改为"wgpu 基座 + 后端两候选待终审"，架构图后端框改 visia-render-<backend>；②§3.2/§5 ODR/OSC 由"原生解析"降为"官方插件（Beta）"，与路线图对齐；⑤§1.1 对 Cesium 的批评改为精确表述（跨维度场景语义不同一、原生嵌入代价），避免事实性硬伤；⑥日期改 2026-09、license 补"已发布版本不可撤销、不追溯闭源"承诺、Visia 词源改为诚实的"vision 语族造词"。未修订：§3.3 10MB 数字与投影范围表述（未列入批准，随 D4 后端终审一并复核）。
- **原因**: 白皮书是对外稳定承诺的载体，凡与工程现实/事实冲突处先修再入库；后端终审属未决事项，文档不得先行锁死。
- **影响**: 白皮书成为项目定位一手事实源（AGENTS.md 已指向）；D4（后端终审）产出后 §4 需二次回填。

## D4: 渲染后端终裁 = wgpu 直用自研管线（B 案） (2026-09-03)
- **背景**: 白皮书 §4 曾暂定"默认后端 Bevy"，D2 impact ④ 挂起终审。四路调研交付（evidence/2026-09-03-wgpu-direct / -bevy-embed / -angle-integration / legacy-platforms.md，全 2026-09-03 实时引用）。
- **决策**: **B 案采纳**——基于 wgpu 直用自研渲染管线，默认后端 crate 更名 **`visia-render-wgpu`**；`visia-render-bevy` 不立项。A 案（复用 Bevy 栈）否决；C 案（先 A 后 B）否决（最贵层写两遍 + C ABI 泄漏）。
- **原因**（A 否决点逐条有证据）: bevy_render 与 App/ECS 不可分且 0.19 加深（RenderGraph 删除实测）；宿主窗口内渲染零先例（godot-bevy 弃渲染器旁证）；无 1.0 承诺+单版 120 破坏+生态锁步实案，与"稳定 C API SDK"冲突；wgpu 直用则主循环/体积/兼容 tier 全自持。B 成本已知：frame graph/管线缓存/GPU 剔除/3D Tiles 流式四无成熟件（永归自持），MVP 15-27 人月合成估计。
- **legacy 一并裁定**: XP 撤出一切对外文本（ANGLE D3D9 已移除 + Rust MSVC=Win10 底线）；Win7 = 非官方 best-effort（不进承诺）；Android 底线 API 24（系统 GLES/Vulkan 双路，ANGLE 无关）；RK3588(Valhall G610)= Panvk 或厂商 libmali blob，**发布二进制锁一种驱动栈**。§3.3 措辞升级：一等 = Vulkan/Metal/DX12/WebGPU，降级档 = GL 3.3+/GLES 3.0+/WebGL2（"一致的是 API 面与 tier 语义，非一致画质"）。
- **影响**: 白皮书 §3.3/§4.1/§4.2/§6.2 回填（同日完成）；README 架构行更新；AGENTS.md CODE MAP 规划 crate 变更；MVP 期风险自测项：10 万 entity 帧预算、wgpu 季度破坏性 pin 策略、cosmic-text CJK 图集质量。
- **SUPERSEDED 声明**: 本决策取代 D2 中"白皮书暂定 Bevy 默认后端（待终审）"表述；D2 其余内容（栈=Rust/SDK 形态/Open Core）继续有效。
