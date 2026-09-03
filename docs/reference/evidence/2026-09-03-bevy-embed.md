# Bevy-as-default-backend 调研备忘录（供 C1 决策表合并）

方法说明：全部证据 2026-09-03 经网络实时核验（GitHub API / bevy.org / docs.rs / crates.io / MDN BCD）。`raw.githubusercontent.com` 在本机被墙，全部改用 `api.github.com` raw 路由——不影响结论。本机无 cargo/rustc，无法实测体积（见 Q4）。

**版本事实（先锚定）**：最新 release **v0.19.1（2026-08-13）**，0.19.0 = 2026-06-18；main = **0.20.0-dev**，edition 2024，MSRV 1.96。
- https://github.com/bevyengine/bevy/releases.atom （fetched 2026-09-03）
- https://github.com/bevyengine/bevy/blob/main/Cargo.toml （API fetched 2026-09-03）
- https://crates.io/api/v1/crates/bevy → max_version 0.19.1

---

## Q1 API 稳定性记录

**破坏性节奏（GitHub releases API 全量 tag 实测）**：0.x minor = 破坏性版本。2024：0.13(2月)/0.14(7月)/0.15(11月)=3 次；2025：0.16(4月)/0.17(9月)=2 次；2026 至 9 月：0.18(1月)/0.19(6月)=2 次。→ **约每年 2–3 次破坏性 minor**，近 18 个月节奏放缓为 ~6 个月一次。
- URL: https://api.github.com/repos/bevyengine/bevy/releases （fetched 2026-09-03）

**官方立场（原文）**：
- README WARNING：「breaking changes … released approximately once every 3 months … We provide migration guides, but we can't guarantee migrations will always be easy」「MSRV generally close to the latest stable release of Rust」 — https://github.com/bevyengine/bevy#readme （fetched 2026-09-03）
- 迁移指南序言：「Bevy is still in the "experimentation phase", which means each release has its fair share of breaking changes.」 — https://bevy.org/learn/migration-guides/introduction/ （fetched 2026-09-03）
- sitemap 1374 个 URL 中**无任何 1.0 roadmap / API stability 政策文章**（检索词 1.0/roadmap/stability/vision 全落空）→ 1.0 时间表无官方承诺。

**单版本破坏面量级**：0.18→0.19 迁移指南含 **120 个 H3/H4 小节**（绝大多数为破坏性变更），渲染相关：`RenderSystems::ManageViews` 改名、`shadow_pass` 拆分 per_view/shared、Camera TextureFormat rework、Post Processing Split、新 crate `bevy_material`、旧 `bevy_scene` 更名 `bevy_world_serialization`、`bevy_reflect` 重组、2d/3d/ui feature 不再互相蕴含。 — https://bevy.org/learn/migration-guides/0-18-to-0-19/ （fetched 2026-09-03）

**cargo features/模块策略**：0.20-dev 将 API 面重组为 feature 集合（`default_app`/`common_api`/`2d_api`/`3d_api`/`ui_api`，注释明确「2d_api … does not include a render backend… unless you are using a custom renderer」）——feature→crate 映射本身即公开 API，且**每个 release 都在重排**（0.19 迁移含「bevy_window … features moved to alternate feature」条目）。

**长期钉死一个版本的实证成本**：钉版技术上可行（crates.io 保留全部 63 个版本；0.19.1 自身 MSRV=1.95，可长期构建），但**生态锁步**是主要代价。实例：SpawnForge 引擎项目 issue 记录 bevy 0.18.1→0.19 迁移因 `bevy_rapier*` 声明 `bevy ^0.18.1` 而「**HARD-BLOCKED since 2026-07-02**」，解除后需 bevy+rapier+hanabi+panorbit+本地 gizmo fork **五方同 PR 协调迁移** — https://github.com/Tristan578/project-forge/issues/8887 （fetched 2026-09-03）。

---

## Q2 bevy_render 能否脱离 App/ECS 使用

**结论：不能，且 0.19 后耦合更深。**
1. **RenderGraph 已被删除**：#22144「Replace `RenderGraph` with systems」（2025-12-15 开，**milestone 0.19**）——渲染 pass 现在是 ECS `Schedule` 中的 systems，顶层 `RenderGraph` schedule + `camera_driver` system 驱动 per-camera schedule；新增 `CurrentView`/`ViewQuery`/`RenderContext`(system param)/`SystemBuffer`。作者自述动机：「Schedule has basically become co-extensive with what the RenderGraph API is doing…iterate on a more ECS based approach」。**v0.19.0 tag 的 `crates/bevy_render/src/` 已无 `render_graph/` 目录**（git tree 实测）。 — https://github.com/bevyengine/bevy/issues/22144
2. **0.19.1 公开 API 形状**（docs.rs）：re-export `ExtractSchedule`、`MainWorld`；模块 `extract_component/extract_instances/extract_plugin/extract_resource`、`sync_component`、`sync_world`、`pipelined_rendering` ——数据流即「main world → extract → render world」的 ECS 复制管线，不存在「给定 device+surface+scene，画一帧」的函数式入口。 — https://docs.rs/bevy_render/0.19.1/bevy_render/ （fetched 2026-09-03）
3. 官方文档自述渲染架构（The Bevy Renderer → render-architecture 章）强调分层与「bypass Bevy's abstractions」的自由度，但该书的 Renderer 章节标注 **「This page is hidden because an ancestor is hidden」——渲染架构章处于未定稿状态**（0.19 重写后文档未跟上）。 — https://bevy.org/learn/book/the-renderer/render-architecture/

→ 对决策表：`visia-render` trait 后面「复用 bevy_render」= 整体吞下 ECS+App+SubApp+asset 管线，剥离不可行；可行的最小形态是 Q3 的 App-disable-winit。

---

## Q3 外部事件循环中手动驱动 Bevy

**官方先例存在，但极年轻且只覆盖 headless/离屏**：
- `examples/app/externally_driven_headless_renderer.rs`：「pumping the update loop manually」——`DefaultPlugins.disable::<WinitPlugin>()` + `WindowPlugin{primary_window:None, exit_condition:DontExit}` + `app.finish(); app.cleanup();` + `std::mem::take(app.sub_apps_mut())` 逐帧手动 `update()` + 手动 `RenderDevice::wgpu_device().poll(PollType::Wait)`（**阻塞式 poll，帧间同步点**）。首次提交 **#22551，2026-01-18** ——该模式被官方背书不足 8 个月。 — https://github.com/bevyengine/bevy/blob/main/examples/app/externally_driven_headless_renderer.rs （含 commit history API，fetched 2026-09-03）
- **踩坑实录（该路径 2026 年仍在出血）**：#24178「headless_renderer: copy render target after RenderQueue has been submitted」（2026-05 修复合入）；#24927「Headless app (`WgpuSettings{backends:None}`) panics on despawn of any render-synced entity: PendingSyncEntity does not exist」（2026-07-09 关闭）——**渲染同步 world 生命周期 bug 在 2026 年 7 月才修**。
- **宿主窗口内渲染（Qt widget / C# 容器 surface）：未找到任何第一方 API 或 issue**。bevy 的窗口创建路径在 bevy_winit（在 `default_platform` feature 内）；issue 检索 external window / parent window / embed in another engine / window handle 均无对应能力请求被接受。
- 最接近的「引擎装进宿主」先例 **godot-bevy**（bytemeadow/godot-bevy，540★，pushed 2026-09-02，v0.11.0）：**刻意不用 bevy 渲染器**——README：「bring Bevy's powerful ECS to Godot, …leveraging Godot's excellent editor and **rendering capabilities**」，`#[bevy_app]` 宏 + `bevy = {version="0.18", default-features=false}` + 按需插件（v0.8+ 明确卖点「Smaller binaries」）。 → 旗舰先例证明 ECS-逻辑复用可行、**渲染器复用无人做成过**。 — https://github.com/bytemeadow/godot-bevy （fetched 2026-09-03）
- 方向相反的 `bevy_child_window`（not-elm，v0.2.1，最后更新 2025-04-28）：往 Bevy 窗口里嵌别的窗口，非我们要的方向，说明窗口嵌入原语全靠社区 DIY。 — https://crates.io/api/v1/crates/bevy_child_window

---

## Q4 体积

**受控对比数据（bevy 最小 3D 应用 vs 裸 wgpu triangle，native 三平台）：未找到可信公开实测 → UNCERTAIN**。本机无 rust 工具链，无法补测（`which cargo rustc` 为空，实测 2026-09-03）。可引用的间接证据：
- 真实 wasm 数据点：jlg.io 首页 bevy 背景应用，PR #550（2026-07-03）自报「**13 MB wasm, ~3.4 MB gzipped over the wire**」。 — https://github.com/jgeschwendt/jlg.io/pull/550
- 官方承认的尺寸负担：#20337 open（2025-07-30）「Reflection results in significant increases to compile times and **binary size**」——bevy_reflect 在 0.19 仍**不可关闭**（无 `reflect` feature flag）。 — https://github.com/bevyengine/bevy/issues/20337
- 官方减脂手段存在但属常规：feature 裁剪（`--no-default-features` + `2d`/`3d`/`ui`/`default_app`）、release-builds 章（opt-level=z/lto=fat/codegen-units=1/strip/wasm-opt）、compiling-less-code 章（「Keeping every feature enabled will result in … a bloated project binary size」）。 — https://bevy.org/learn/book/releasing-projects/release-builds/
- 裸 wgpu triangle 尺寸：无一手公开数字可引 → UNCERTAIN（社区常说"~1-2MB"，未在本次调研中找到可引用来源）。

→ 对 ≤10MB 目标的判定只能写：wasm 侧唯一实测样本（13MB 未裁剪、3.4MB gzip）已贴近/越线；native 侧待你们有工具链后用同一 profile 做 `--features 3d` vs 自研 wgpu 的对照实验，此为 §4 中必须标注的**未决测量项**。

---

## Q5 相机切换 / 自定义 pass / 十万级实例

- **ortho↔perspective 运行时切换 = 一等模式**：官方示例 `projection_zoom.rs` 的 `switch_projection` 直接原地改同一实体的 `Projection` enum（`Projection::Orthographic(_) ⇄ Projection::Perspective(_)`），无重建、无插件重载；另有 `3d/orthographic.rs`（官方注释点名「isometric-look in games or **CAD applications**」）、`camera/custom_projection.rs`（自定义投影 trait）、`pan_orbit_camera_ortho.rs`。**模式成立；切换动画插值需自己做（API 只给突变）**。 — https://github.com/bevyengine/bevy/blob/main/examples/camera/projection_zoom.rs （源码 fetched 2026-09-03）
- **自定义 pass**：0.19 后正确挂点 = 顶层 `RenderGraph` schedule（#22144 原话「This top level schedule provides an extension point for apps that may want to do custom rendering, or non-camera rendering」）+ per-camera schedule 插 phase；示例 `custom_render_phase`/`custom_phase_item`/`custom_post_processing`/`render_depth_to_texture` 全在。**代价：挂点形态 0.19 刚整体换血**（旧 Node 式 graph API 已删），所有旧教程/第三方渲染插件都是待迁移债务。
- **重实例化**：`automatic_instancing.rs` 实测样例即 256×256 = **65,536 立方体单 mesh 单材质自动合成 1 个 draw call**（示例头注释：「should be only a single draw call」，可用图形调试器验证）；`MeshTag` 携带 per-instance 外部数据；`GpuComponentArrayBuffer`（#24922，2026-07 增强为可存任意 per-instance 数据）；0.19 主题「Render Bigger Scenes Faster」+ 0.20-dev 正在合入超大场景示例（#25371「Add the Zero-Day large-scene example」，2026-08-11）。**100k 级 tile/line 在 GPU 侧属设计包线内；未找到 100k *实体* 的 CPU extract/sync 开销公开基准 → UNCERTAIN**（这正是 GIS/孪生场景的形态，建议 §4 把「10万 entity extract 帧预算」列为 bevy 方案的必测风险项）。线绘制：bevy 无内建广域 polyline 线框方案证据（gizmos 仅调试用途）——GIM/道路线数据需要自建 mesh 生成层，无论选哪边都要写，不构成差异项。

---

## Q6 C ABI / FFI 先例

**未找到任何把 bevy 渲染器发布到稳定 C ABI 之后的项目**（GitHub repo 检索 bevy+language:C 无有效命中；bevy ffi/c-api/capi 组合 0 命中；crates.io q=bevy ffi 65 个结果全部是音频/ECS 桥）。最接近的四个，全都绕开了渲染：
| 项目 | 形态 | 渲染谁负责 | 状态（fetched 2026-09-03） |
|---|---|---|---|
| bytemeadow/godot-bevy 540★ | gdext in-process，`#[bevy_app]`，default-features=false | **Godot** | pushed 2026-09-02，活跃 |
| KBVE uniti 18dl | csbindgen 生成 C#↔Rust，仅 game logic | Unity | v0.1.0, 2026-05-05 |
| matthunz/bevy_mod_ffi 59dl | guest/host FFI 工具 | — | v0.2.0, 2026-01-07，低采用 |
| bevy_python_ffi 2.9k dl | Python↔bevy app 控制 | bevy 自己（窗口仍 bevy 建） | 存在但非 C ABI SDK 形态 |

→ 「bevy 渲染器藏在 `extern "C"` 后面」这件事**没有成功先例，也没有失败先例——完全空白**；叠加 Q3「宿主窗口 surface 无第一方路径」，SDK 化风险集中在无人验证过的组合上。

---

## Q7 wasm / WebGPU（2026 现状）

- 能力自 0.11 起（2023-05-17 官方公告《Bevy + WebGPU》，@cart：bevy 原生跑 WebGPU/wgpu，live examples 已上线）；`webgl2` 仍在 0.20-dev `default_platform` 里 = **WebGL2 兜底同时维护**。 — https://bevy.org/news/bevy-webgpu/
- 浏览器基线（MDN BCD `api/GPU`，repo pushed 2026-09-03 当日快照）：Chrome 113 起、**Firefox 141 起、Safari 26 起** → WebGPU 已是三主流浏览器稳定特性。
- **wasm 无多线程**：tracking issue #4078（2022-03 开，89 reactions）**2026-09 仍 open** → wasm 构建单线程跑全部 ECS extract，大场景帧预算直接受压。
- 0.19 修了 web 半句旧账：「Cancellable Web Tasks」——wasm 上 Task drop 不取消、静默泄漏工作（PR #21795，换用新 web-task crate）；同版破坏项「Dropping Tasks in Web Builds」。**0.19 之前 web 端任务语义与 native 不一致是既成事实**。
- 未决问题：#22545 open（2026-01-16）「Rapidly growing number of WebGPU objects causing stuttering」——WebGPU 后端仍有对象增长卡顿在案。
- **0.20-dev 正在换 shader 语言**：#25088「Migrate to WESL」closed **2026-08-05**（main 示例 shader 已是 `.wesl`）——下一个 release 的自定义材质/着色器接口又要变一轮。

---

## 直接可用于 §4 的三行硬结论

1. **稳定性**：每年 2–3 次破坏性 minor、单版本 120 项破坏、无 1.0 承诺、生态锁步迁移有实案（#8887 hard-block）→「长期 API 稳定承诺」只能建立在**自建 C ABI 层完全隔离 bevy 类型**上，且需接受钉版 + 放弃插件升级红利 + 反射不可裁剪。
2. **主循环**：手动驱动有官方示例但 2026-01 才出生、2026-07 还在修 P0 panic；**宿主窗口 surface 渲染无任何第一方/第三方先例**（godot-bevy 用 Godot 渲染绕题）→「不得劫持宿主主循环」条款在 Bevy 方案里是**待发明**，在自研 wgpu 方案里是默认属性。
3. **能力面**：相机双投影切换/自定义 phase/65k→单 draw call 实例化均有官方示例背书（不是 fight）；体积与 10 万实体 CPU 开销两点**数据缺失，列为必测项**——§4 应要求 2 小时 spike（工具链可用后）：`bevy --no-default-features --features 3d` 最小 app 三平台 release 体积 vs 裸 wgpu，以及 100k entity 的 extract 帧耗时。

未修改任何仓库文件。