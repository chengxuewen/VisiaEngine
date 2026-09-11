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

## D5: 开发环境全 pixi 统一管理，rust 工具链含在内（2026-09-03）
- **决策**: 开发环境以 pixi（conda-forge 单源）统一管理，**rust 工具链亦由 conda-forge 提供**；不采用 rustup 主路径。新机器唯一先决 = `source bootstrap.sh`（自动装 pixi → 一切从 lock 来）。
- **原因**: 实测 conda-forge Rust 全家桶完整（rust 1.98 月内跟版 + rustfmt/clippy/rust-src/rust-analyzer + `rust-std-wasm32-unknown-unknown`/android + cargo-deny/wasm-bindgen-cli/cargo-binstall）；单 lockfile、国内 conda 镜像成熟、心智模型唯一。前身项目日常主力开发全程跑在 conda rust 上，其踩坑集中于交叉/厂商栈场景（非 conda rust 装不动）。曾评估"pixi 外壳+rustup 工具链"混合方案（C），因 wasm/交叉 std 论据被探测推翻而降级。
- **条款（边界）**:
  1. Windows 逃生舱：win-64 入 platforms；若 win conda rust 摩擦超标，单机 rustup+MSVC 覆盖，不污染主线。
  2. 嵌入式交叉（RK3588/Jetson 类厂商设备）一律 Docker/厂商工具链，conda rust 只做 host 编译——前项目 PIT-85（conda rust 注入 `CARGO_TARGET_*_LINKER` 优先级高于 `.cargo/config.toml`）的边界化处理。
  3. conda 无包的 cargo 工具（tarpaulin/cbindgen 等）统一经 cargo-binstall（conda 有包）获取预编译二进制。
  4. CI 钉死 pixi-version，锁漂移用 `--frozen` 把关。
  5. `pixi.lock` 与 `Cargo.lock` 同纪律必入库。
- **影响**: 动工日生成 pixi.toml / bootstrap.{sh,bat} / pixi.{sh,bat} / docs/env.md；`rules/common/constraints.md` 增补环境纪律；机器上永不出现 rustup（除条款①逃生场景）。

## D6: 技术命名空间统一为 visiaengine-*（2026-09-03，用户裁决）
- **决策**: 品牌名 = **VisiaEngine / 维视引擎**（PascalCase，文档/UI/正式称谓）；技术前缀 = **`visiaengine-`**（小写全品牌名：crate 包名/lib 下划线名/workspace 路径/CI 与门禁引用全链）。C ABI 层：符号 `visiaengine_*`、头文件 `visiaengine.h`、库名 `libvisiaengine.{so,dylib,dll}`（capi 片兑现）。环境变量前缀预留 `VISIAENGINE_`。分层例外：**产品称谓保持原样**（编辑器 "Visia Studio" 非技术命名空间）。
- **修订面**: D4 历史条目中 crate 名表述按史实保留不改写；自本条起现在时文档全用新名。SDD 条款号（CORE/REND/WGPU-*）独立于 crate 命名，零改动。
- **原因**: ①"visia" 非官方产品称谓，全名三位一体（仓名=品牌=前缀）检索独占零歧义；②实测无障碍——crates.io 两候选命名空间均空闲、GitHub C 符号 `visia_` 零使用；③前项目 D209→D221 两次全量改名税的终态教训：一次到位=品牌小写全名；④原反对案"C ABI 短前缀双轨"论据经用户质询推翻（C 前缀可同步改，长前缀 gdk_pixbuf_* 有 C 惯例先例）。
- **影响**: 单笔原子改名（3 crate 目录+包名+引用+CI+现时文档，`pixi run ci` 与 golden 三测全绿后提交）；历史面（decisions/pitfalls 旧条目、docs/reference 快照、commit 史）不改写。未来品牌若再演化，重蹈改名税的成本由"发布前定死"纪律兜底。

## D7: P1 裁决——RTC 采用分层 origin 方案（viewer-origin rebase，2026-09-03 用户裁决）
- **决策**: 坐标精度机制定死三层合体：①core 世界=**f64 全局**（ECEF/投影域数值，语义归属 io 数据源侧，与本题正交）；②顶点存储=**局部 f32**（相对 model/tile 级 origin；local ≤64km 下 f32 保 mm）；③draw uniform 平移=**f64 `origin−eye` 后降 f32**（每帧/相机动阈值，v0 每帧）。不采 CesiumJS 全局绝对顶点+high/low 拆分（那是其历史包袱的补偿）；不采 shader f64（WebGPU 禁）；不采每帧重写顶点（大模型 VBO churn=死）。
- **原因**: 四方独立收敛（3D Tiles tile transform / MapLibre tile-local+u_matrix / deck.gl layer origin / UE5 LWC——其 per-component 特化方案在 5.0 被废弃回退，即"过度分层 origin"的反面教材）；Cesium `czm_translateRelativeToEye.glsl` 源码实证 viewer-origin rebase 语义；本方案纯 f32 上传面 → **Web SDK 路线零额外工作**；顺带显式化 G2 遗留的 `model f64→f32` 大坐标步长债（1e7 m 级步长约 1m）。
- **影响**: 实施时机=**GeoJSON 片**（glTF demo 坐标小，现路径兼容零改动）；core SDD 届时补 CORE-11/12 重基行为断言；tile 切分粒度（地图级子毫米需求时）=io-geo 自决，引擎机制不变；极端远程精度需求预留 Cesium 式 high/low 为升级路径（不默认实现）。architecture.md ④/未决点表同步销账。
- **证据**: `docs/reference/evidence/2026-09-03-rtc-hierarchy.md`
- **追加勘误（2026-09-03 规划轮）**：'core SDD 补 CORE-11/12'归属错——rebase 组合数学主场在 render（core 无矩阵/wgpu 语义），实际编号=REND-19/20；CORE 命名空间本链无新增。

## D8（预登记，未终审）: 宿主嵌入面三原则（2026-09-03 计划 v1.1 批次 0c）
- **来源**: Easy3D 嵌入域调研 [E3D:C1/C2/C5]（docs/reference/easy3d.md），其 Qt 集成靠 QApplication::notify 私有 hack、ImGui 版丢鼠标坐标、GLFW 指针泄进 public ABI、绑定自动全扫——四坑全为我方反证。
- **预登记内容**: ①引擎零事件 API：宿主统一经 `visiaengine_on_input(ev)` 注入，出向仅 `frame_requested`/`redraw_needed` 回调，永不要求宿主子类化/事件 poll；②渲染契约面（render trait + IR）禁携带任何窗口/surface 后端类型，rwh handle 只在 capi 边界出现（Easy3D renderer 模块 glfw 零引用实证该纯度可达）；③C ABI 首版=6 行 demo 证通最小集（create 即渲染就绪，零多阶段 init），未绑定项入 graveyard 流程，禁 GLOB 式全量扫绑。
- **转正程序**: 宿主嵌入片计划轮（批 2）审查通过后，本条改写为正式 D8 裁决并补 rwh 路径 A/Qt feature 环境细节；被否内容留"修订面"注记。计划期间引用一律带"预登记"字样。

## D9: P2 裁决——样式 spec 采 MapLibre v8 paint 别名子集（2026-09-03，用户裁决 B 案）
- **决策**: 在既有 simplestyle 六键之上，接受 MapLibre v8 style spec 的**静态 paint 布局键别名**：`fill-color`→fill、`line-color`→stroke、`line-width`→stroke-width、`circle-color`→marker-color、`circle-radius`→marker-radius（`fill-opacity` 同名）。**不采** v8 的 expressions / zoom stops / source-layer / data-driven（那是 Visia Studio 商业面 + 完整 spec 轮）。simplestyle 主键优先，别名回退。
- **原因**: GIS 行业现成样式资产多为 v8 格式（MapTiler/OSM thisdir 生态）；别名映射 ~30 LoC 即吃下兼容面，为 SDK 嵌入门槛降低；完整 v8 表达式引擎 3-4 周且侵蚀商业层（Open Core 边界），故取"静态别名子集"这一最小兼容切面。
- **参考**: C1 四案对比（A 维持/B 别名子集/C 完整/D 自有），B 案 = docs/reference/maplibre.md 活标杆生态兼容 + three-js.md 库形态先例；否决 D（自造=零生态+隐性锁定）。
- **影响**: geo::style 增别名表（GEO-19）；值格式复用 GEO-12 色解析（#hex/rgb()）；样式系统商业面（Visia Studio 表达式/图层编排）边界清晰上移至未决 P4 邻近。后续 3b 标量→LUT 建在此样式口上。

## D10（预登记，未终审）: Web/wasm 绑定=路线 A（双面口单源）（2026-09-11 批 7 计划批准轮登记）
- **预登记内容**: ① 浏览器面=capi crate 的 `wasm` feature，js.rs 与 ffi.rs 双叶共 engine.rs（单 crate 双面，非 per-repo——单厂商全控面下更懒且免疫 MapLibre 式"同名异核"）；② 薄口纪律：js.rs 只限编组/生命周期/常量再导出/错误翻译，语义唯一住 engine.rs；常量 Rust pub const 单源进 .d.ts 与 C 头（镜像 CAPI-09 机器守卫）；③ 无 free 函数纪律维持（borrowed + caller-alloc 两式，SQLite/[MS] 双派实证）；④ emscripten 路线否决（语言匹配律：C/C++ 核心→emscripten，Rust 核心→wasm-bindgen，三例全合）；⑤ u64≡BigInt、异步工厂（web GPU 交互无同步路径，wgpu 30 Future 实证）。
- **转正程序**: J1-J3 落地后本条改写为正式 D10（含 .d.ts 面冻结清单+M2 边界）；被否方案留修订面。计划期间引用带"预登记"字样。证据链 docs/reference/ffi-patterns.md [FFI-R:BS-5/6/7, v13-FEAS-2/7]。
