# VisiaEngine 架构决策记录

> 记录影响接口、数据流、依赖方向或构建形态的决策。每条含编号、日期、决策内容、原因、影响。方案对比的呈现格式遵循 C1（优缺点/来源/影响/推荐四要素）。被推翻的决策标注 SUPERSEDED 与新编号，不删除条目。

## 格式示例

## D<N>: <决策标题> (YYYY-MM-DD)
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

## D8: 宿主嵌入面规范（2026-09-11 批准转正；预登记 2026-09-03）
- **来源**: Easy3D 嵌入域调研 [E3D:C1/C2/C5]（docs/reference/easy3d.md），其 Qt 集成靠 QApplication::notify 私有 hack、ImGui 版丢鼠标坐标、GLFW 指针泄进 public ABI、绑定自动全扫——四坑全为我方反证。
- **预登记内容**: ①引擎零事件 API：宿主统一经 `visiaengine_on_input(ev)` 注入，出向仅 `frame_requested`/`redraw_needed` 回调，永不要求宿主子类化/事件 poll；②渲染契约面（render trait + IR）禁携带任何窗口/surface 后端类型，rwh handle 只在 capi 边界出现（Easy3D renderer 模块 glfw 零引用实证该纯度可达）；③C ABI 首版=6 行 demo 证通最小集（create 即渲染就绪，零多阶段 init），未绑定项入 graveyard 流程，禁 GLOB 式全量扫绑。
- **转正增补（批 2 I0-I4 全清，2026-09-11）**：① 句柄=u64 (slot<<32|gen)、slot 基 1、C 头零位布局外露；② 线程例外集={abi_version,last_error}，destroy 跨线程拒销毁、released 再入 -1（双销毁不吞没）；③ **frame_requested/set_callbacks 被否**（拉模型 ve_render+宿主 rAF/paint 自决；SDL3 双层先例在案，重评触发=Qt 动画轮）；④ attach 原子性（失败保 headless 现目标）+ x11 display 必填/win32 hinstance 0=自动；⑤ **wasm 面 C ABI 整体 cfg 出局**（no_mangle 与 bindgen 导出表 wasm-ld 互斥实测）——JS=独立 crate visiaengine-wasm，C 符号图零侵蚀。
- **验证锚**：gate-abi（14 符号白名单+demo 编译运行）· SMOKE-X11 'OK capi x11' 真窗三帧 · 'OK capi headless input+pick' · CAPI-01..09 十五条款全带测试。

## D9: P2 裁决——样式 spec 采 MapLibre v8 paint 别名子集（2026-09-03，用户裁决 B 案）
- **决策**: 在既有 simplestyle 六键之上，接受 MapLibre v8 style spec 的**静态 paint 布局键别名**：`fill-color`→fill、`line-color`→stroke、`line-width`→stroke-width、`circle-color`→marker-color、`circle-radius`→marker-radius（`fill-opacity` 同名）。**不采** v8 的 expressions / zoom stops / source-layer / data-driven（那是 Visia Studio 商业面 + 完整 spec 轮）。simplestyle 主键优先，别名回退。
- **原因**: GIS 行业现成样式资产多为 v8 格式（MapTiler/OSM thisdir 生态）；别名映射 ~30 LoC 即吃下兼容面，为 SDK 嵌入门槛降低；完整 v8 表达式引擎 3-4 周且侵蚀商业层（Open Core 边界），故取"静态别名子集"这一最小兼容切面。
- **参考**: C1 四案对比（A 维持/B 别名子集/C 完整/D 自有），B 案 = docs/reference/maplibre.md 活标杆生态兼容 + three-js.md 库形态先例；否决 D（自造=零生态+隐性锁定）。
- **影响**: geo::style 增别名表（GEO-19）；值格式复用 GEO-12 色解析（#hex/rgb()）；样式系统商业面（Visia Studio 表达式/图层编排）边界清晰上移至未决 P4 邻近。后续 3b 标量→LUT 建在此样式口上。

## D10: Web/wasm 绑定=路线 A（双面口单源）（2026-09-11 J1-J3 落地转正）
- **预登记内容**: ① 浏览器面=capi crate 的 `wasm` feature，js.rs 与 ffi.rs 双叶共 engine.rs（单 crate 双面，非 per-repo——单厂商全控面下更懒且免疫 MapLibre 式"同名异核"）；② 薄口纪律：js.rs 只限编组/生命周期/常量再导出/错误翻译，语义唯一住 engine.rs；常量 Rust pub const 单源进 .d.ts 与 C 头（镜像 CAPI-09 机器守卫）；③ 无 free 函数纪律维持（borrowed + caller-alloc 两式，SQLite/[MS] 双派实证）；④ emscripten 路线否决（语言匹配律：C/C++ 核心→emscripten，Rust 核心→wasm-bindgen，三例全合）；⑤ u64≡BigInt、异步工厂（web GPU 交互无同步路径，wgpu 30 Future 实证）。
- **转正增补（J1-J3）**：① js 面冻结清单=类 VisiaEngine（fromCanvas 异步工厂/扁平 input/pick→bigint）+11 常量 getter+abiVersion 静态；.d.ts 为锁定档（web-mirror.mjs 编译面断言守卫）；② **修订面（被否路线记录）**：wasm-pack 未死但 D5 无包直用 CLI；`visiaengine_input_struct_size()` 导出否决（14 符号面）；playwright CI 自动化归 M2（本机 timebox 放弃实录）；③ 尺寸实录 **raw 733KB/gz 294KB**（wgpu web 全家，≤10MB 目标的 web 半边提前达标）；④ 隔离终形：capi(native C ABI)+visiaengine-wasm(bindgen) 两 crate，**单源=Engine**，CAPI-09 机器对账。证据链 docs/reference/ffi-patterns.md [FFI-R:BS-5/6/7, v13-FEAS-2/7]。


**D8 触发器①复评（2026-09-14，Qt 轮 Q3 真窗实证）**：frame_requested 帧跳过协商口——QTimer 60Hz 拉模型经 V1 真窗 6 帧无帧率痛点，**维持挂起**；再触发条件=视频/解码帧对齐类宿主（拉模型跟不上外部时钟时才复议）。

## D11: Qt 轮 demo 构建集成=薄自管 CMake（B 案）（2026-09-14）

**决策**：Qt widget 轮的宿主消费面用 iceoryx2 形薄自管 CMake（~60 行）：`add_custom_target` cargo 步 + `RUST_BUILD_ARTIFACT_PATH` 逃生舱 + INTERFACE 伞 `visiaengine::capi`（BUILD/INSTALL 双 genex）+ 树内假 Config 花招（find_package 消费代码树内/装后同文）；`visiaengineConfig.cmake.in` 模板随写但 install 树/.pc/SOVERSION 不做。g++ 脚本案（A）与 corrosion 直上案（C）弃。
**理由**：demo 必被真实宿主以 cmake 消费——现在写一次 vs 打包轮重写+重验证两次；find_package 冒烟由 demo 天然承担（前作头号教训：cmake 包从未被编译过）。依据=团队调研四案对比（docs/reference/cmake-integration-patterns.md，iceoryx2 :153-205/:38-78/install.cmake:17-21、slint :16-26 及其 corrosion 坑 :427-466 实证）。
**影响**：Qt 轮 +1 文件级增量；capi Cargo.toml 顺手加 `links = "visiaengine_c"`（重复静态链防呆）。
**落地续（同日）**：B 案承接地=CMake 工程化层（.omo/plans/visiaengine-cmake-project.md C1-C3 已落，Qt 轮 v1.3 起为纯消费方）；links 账不立（cargo 硬校验无 build script）由 gate-abi nm 集继续覆盖。
**复评触发器（打包轮议题）**：转 corrosion 或维持薄自管——触发=第二 C++ 消费者 / Windows 矩阵 / install-tree 真需求；corrosion 转正需先立「conda 无包→cargo-binstall/GitHub 镜像 vendor」D5 例外账（FetchContent configure 期联网与离线纪律相抵）。伞接口 `visiaengine::capi` 为稳定合同，届时换内核消费面零改动。

## D12: CMake 示例入口工程=stub launcher + 契约表 + ctest 同源（2026-09-15 落地转正）
- **背景**: 11 件 E 系 example 对 CMake 透明，IDE 无运行入口；CWD 数据路径（7 件）与 DISPLAY（5 件）两合同破裂面大。
- **决策**: A=launcher stub（~40 行 C++ 单源）承载 IDE Run 真身：chdir 仓根 + 显示族 DISPLAY 预检 exit 77 + argv 透传；注册=契约表（glob⇄表双向校验，缺项 configure 硬错）；ctest example_* 与 target 同源。弃 B=仅 custom target+ctest（CLion 无 Run 按钮）。
- **原因**: CLion Run 只认常规可执行目标（imported/custom 不出运行配置）；Easy3D 实证「名=目录=标题」零维护（探针 A file:line）；77=ctest SKIP 原生约定（SKIP_RETURN_CODE 需 ≥3.24）。G8 修订：gate-abi/smoke-x11 的手工 cc 路=纯 C 消费者可编译性门禁属性，非冗余平行，保留。
- **影响**: 新增 example 义务三处（源文件入盘 / 契约表登记 / tutorials.md 索引）；examples 族需 CMake ≥3.24；三方锁面扩到 cmake 入口（configure 即门，不另立 gate-docs⑥）。

## D13: ABI 属性读三口 14→17（裁决链闭环，2026-09-15）
- **背景**: 宿主面属性查询缺失（capi 14 口零读口，探针实锤）；缺口表荐「AttrSet 属性例」。
- **决策**: 用户方向批准（问题呈「第 15 口」字样）→ 计划显式摊牌容量差 → C-1 终审=**3 typed 口 14→17**；C-2 str 无探长（cap 不足=-5 零部分写，256B 文档约定）；C-3 E701 追加段不新开 E 号；点云独立例**跳过**（与 E202 重叠且无专有数据源）。
- **原因**: 三型镜像 CORE-12 闭合口，宿主一次学全；单变体型口 C 侧丑（型双关）；键=enc_entity 位形直通（零解码、ABA 免疫=代际在键内）。
- **影响**: D6「14 入口」锁正式改写（CAPI-01/02 条款修订句 + 十二处连带面清单已销）；后续扩口先例=**必须先摊容量差再请批**；wasm 面维持常量镜像（函数不镜像现状）；`attr_of` 保留面为批 7 js 属性口预留同构形状。

## D14: 根 examples 目录法 + c+ 统一例子面 + stub 体系退役（supersede D12，2026-09-16 v1.3 落地）
- **背景**: 用户判 CMake 工程混乱（目录/target/例子三诉）；红队两轮 27+10 项实证：stub launcher=「例子是假的」病根、corrosion 导例进 IDE 主流零先例（slint/iceoryx2 例子皆留 cargo 世界）且自带八税（cargo-build_* 假名/always-dirty #624/双缓存/conda 首蟹/无 CTest #13 拖 2018 至今）。
- **决策**: ①launcher+189 行契约表+旧 ctest example_* 全废；②根 `examples/{rs,c,cpp,qt}/` 按语言分层，Rust 例保 **cargo example 语义**（[[example]] 显式块，不 bin 化——bin 吃不了 dev-deps 的 v1.1 死结随之消解）；③Rust 例进 IDE 经 **c+**：`cargo-build_*/cargo-run_*` 自写步骤（R10 采纳 corrosion 命名文法、零依赖）+ ctest `example_<茎>` 统一清单，pixi smoke 九条降转发壳（argv 单源=R9 注册表）；④corrosion 议题挂打包轮（引入日=自写步骤整批退场日，同名绊线）。
- **原因**: 主流形态+税单实测；c+ 体验 90%/成本 19 行，b 路线招牌被其价目表自贬。逐项裁决全程用户过目（含「说人话」四轮收束）。
- **影响**: cmake-smoke 三态三锚（裸 PATH configure 探+双构建锚——B1「守卫别跟着裁丢」）；D11 复评同步刷新（conda 有 corrosion 包=当初「无包」判据系误，本轮仍弃于税表）；RA Run 降辅助轨。

## D15: bindings 一窝同栖 + SDL3 窗口宿主 + E8xx/孪生编号法（2026-09-16 v1.3 落地）
- **背景**: MediaServo 形采纳裁「全盘一窝同栖」；窗口例宿主需跨平台件；E8xx 新带需编号法。
- **决策**: ①`crates/`=纯核心 5 库；capi→`bindings/c/visiaengine-capi`（src+手写头+tests 同栖，probe 同级），wasm→`bindings/js/rust/`（实测路径引用=0，仅 members+一行 path 地雷），`bindings/cpp`(hpp)/`bindings/qt`(widget.hpp)/`bindings/js/demo`；目录法一句话：examples/=教学例、bindings/=实现+头+验收+包壳。②宿主三分：headless 主力（readback 断言）+ **SDL3**（conda 3.4.16，xid 属性=attach 合同同构，S0 双态实证）+ Qt 专带演示；FetchContent 兜底不建（D5 单源）。③C/C++ SDK 例=**E8xx 带**；孪生共号 `.c/.cpp` 分家（目标名 `_c/_cpp` 尾）；资产逃生口=目录名=文件茎。④hpp 不开 SDD 账（widget.hpp 先例；三壳语义住头注释、活体锚 CAPI-nn；触发器=壳长新行为时建 cpp.md+扩 spec-trace 双正则，30 分钟无利息）。⑤install 树=S-c β：本轮仅 fail-loud 守卫，树+树外演练=打包轮硬债（模板钉 §7）。
- **原因**: 主流六仓实测（iceoryx2/slint/SDL3/Vulkan-Hpp/Sascha/filament）+ 红队 G 组缺口全收编；examples 中心化=6/6 主流一致，「tests 栖绑定价、examples 集中」两分法。
- **影响**: 例子门禁=可 ctest 枚举单源；toolchain 批（compile_commands/.clang-format/.vscode presets always+.gitignore L56 negation 死锁修复）；PIT-19（pixi clobber→conda Xvfb XKB 死）入账。

## D16: 点云带范围裁决（2026-09-17，hyperplan 三回合对抗 + 用户「按推荐」）
- **范围**: M0 ramp 暴露 + M1 add_points 直通 + M2 io-points(PLY ascii/bin_le) 同带；**LAS=二带**（判据：AV/测绘客户先行演示单，本机无证据；las crate 0.11.1 MIT 已核可入）；拾取族整块后置（触发=交互需求 ∧ bench>16ms）；法向/简化/EDL/逐点查询全部非目标。
- **命名**: 新前缀 `IO-*`（io-* 族律=命名跟随 crate），spec-trace **第三检机器锁**同 commit（白名单外前缀=红；正/负例自证在册）——把「扩前缀漏正则双侧静默失明」降维成门禁问题。
- **数据模型**: 云=单实体单 DrawPoints，origin=f64 bbox 中心+local f32（mount_geo 同构，D7 零新数学）；云级 meta 走 attr_* 缀查（零新查询口）；逐点属性/查询口随拾取族一并后置。
- **容量**: cap=4M（声明先拒非半收）；release 实测 1M=upload 127ms/frame 914ms（lavapipe，bench_pcl json 在册），改数窗=一次（常量+条款+测试三处同 commit）。
- **过程账**: 红队攻出的价值——「S 档纯 CPU 预着色=不可交付」并入 M；「ray_aabb 复用云级 pick」被四家打穿后置；bench.sh 恒真绿（PIT-25）由 G3 顺带根修。

## D17: 剖面裁切带八裁决点（2026-09-17，B2 计划轮探针对抗 + 用户「按推荐全带开工」）
- **a 路线**：fs discard 单路——wgpu 30 硬件 CLIP_DISTANCES 实核存在（docs.rs）但
  DX12 后端未在列 + feature-gate 需双路分叉；复评触发=discard 边缘质量真疼（客户投诉锯齿）。
- **b 面数**：≤4 AND（单面/角剖/三轴盒档）；六面盒/OR-union（three.js 双组形）=非目标，
  升 6 仅 MAX_PLANES 一处（定长 Pod 布局锁）。
- **c 盖帽**：不做（剖开见空心如实文档）；stencil cap 触发=BIM 内腔演示单。
- **d 精度**：世界系数 f64（n 指保留侧+面上=保留闭区间），shader 走 model-space 恒等式
  `n_m=Mᵀn, d_m=d+n·(o+t_M)`（f64 相乘后降 f32）——实施中自查根修两处（Mᵀ/平移列），
  教训：列主序下「法向乘矩阵」方向必以恒等式推导，勿凭直觉。
- **e 受裁族**：全绘制族（mesh/inst/stroke/point 逐像素）——「裁世界不裁类型」。
- **f 阴影**：caster 同 discard（剖掉的楼不留整块影；three.js clipShadows 默认 false
  是历史包袱非理由）。
- **g 拾取**：命中点 keeps 负侧→排除重试环（剖开可见者必可拾）；capi pick=positions×
  IDENTITY 既有事实以「两帧合一」注记，不顺手扩 origin 域（独立缺口独立立项）。
- **h 面**：E813 C 单例无头活体门；交互剖切滑杆挂账（E901 键控=演示需求触发）。
- **红利记录**：View 块扩段方案（origin+transform 已在 view_block 签名）使本带
  **零新 binding/绑组/缓冲**——比探针预估的独立 binding9 省一整层施工；
  「先查数据通路再定挂点」纪律（think-before-act）实证。

## D18: S2 文字面带八裁决点（2026-09-18，探针 A/B 对抗 + 用户「档①引擎自管·全带开工」）
- **a 档位/方案**：档①最小可用 + 方案 I（引擎自管 fontdue 栅格）——用户三选一明裁；
  非宿主自管（SDK 价值折半）、非标准档（swash 触 R2）。MSDF 路线否（Rust 生态≈0，msdf-font 1★）。
- **b 依赖锁形**：fontdue `default-features=false, features=["hashbrown"]`——
  关 SIMD（wasm 陷阱 #25/#72）保 hashbrown（no_std 下 std/hashbrown 二选一 [docs 实测，
  探针报告未点出，N0 冒烟补正]）；ttf-parser RUSTSEC-2026-0192 deny-ignore
  （unmaintained 非漏洞；攻击面=宿主注入自有资产+panic 栅栏；ab_glyph 同依=换无效；
  三复评触发入 deny 注释）。
- **c 新 crate 落点**：`crates/visiaengine-io-text`（第 9 crate，纯 CPU：face/cache/layout），
  GPU 态住 render-wgpu；接口隔离使 swash 升级非破坏。
- **d 命名空间**：复用 IO 前缀（IO-07..09，白名单已含=第三检零改动）非新立 TEXT-。
- **e 标签坐标**：世界锚定 + 屏幕恒大小（ortho 精确路 assert_eq 跨 zoom 逐像素等锁）；
  色=线性域直传（sRGB 咽喉 label 表=core::srgb_to_linear，咽喉六）；center 对齐落 dx 域
  （PIT-28：pen 是 px 不可进 anchor 世界坐标）。恒顶：depth write=false/Always + 命令序末位。
- **f atlas**：R8 单槽 512² + shelf 装箱 + 1px 缝（采样渗色防）；满则全清重烘透明降级
  （[ponytail] 逐字 LRU 待 profile）；set_glyph_atlas 全量替换（io-text dirty 驱动）。
- **g 受裁**：标签走 WGPU-21 clipped() 同一判据（锚点 model-space，REND-33 表域）；
  caster 不投（无投影面续不装②）。
- **h 双口镜像**：CAPI-21 load_font + CAPI-22 add_label（26→28，abi minor=7 非 8）；
  三面镜像全带（头 POD 直用/hpp 薄转发/wasm loadFont+addLabel→bigint+MISS）；
  管理域=items 外（add_points 同谱，不入 entity_count/remove）。
- **R2 体积战果**：wasm gz 429401→490967 = **+14.3%（阈 15% 内过线，零余量如实记）**；
  探针预估 +6% 偏乐观（真实=字体栈+shader+label 链）。复评触发：wasm 面真投产 or
  需 CJK 内嵌时重判（退路=feature gate 字体栅格仅 native，R2 纹理退路同款）。

## D19: ⑤a 相机飞行带裁决（2026-09-18，双探针对抗 + Momus OKAY 0B/3A + 用户「全带开工」）
- **范围刀**：本带=flyTo 纯飞行；**多视口分屏=独立带**（四坑清单入非目标：depth_cache 单例升 Map/
  scissor 零实证/输入 px 路由/双 px_world_scale——探针双一致，混带=拖垮验证节奏）。WGPU-26 预留。
- **分层定案（被迫即正解）**：render crate 零 std::time——纯函数 fly_sample(from,to,t,easing) 住 render；
  墙钟推进器住 engine。**wasm 运行时雷主动拆**：std::time::Instant 在 wasm32-unknown-unknown 不可用
  （flyTo 会触 capi_guard -4 假死）→ now_ms() shim 双形（native=Instant 进程锚/wasm=js_sys::Date::now）
  ——Web 是主舞台，编译过≠跑得通，写于踩前。
- **缓动**：Easing{Linear,CubicInOut(smoothstep),CubicOut}；capi 面固定 CubicInOut（加参=YAGNI）。
- **中断=MapLibre A 派**：指针/滚轮（kind 1/2/4）首行 cancel；键(5) 不打断（测试反证在）。
- **done/idle 合并单主**：fly_state 返回 1=不在飞（cancel 后=1 可再飞）；out_t01 仅飞中写、
  done 零写（B1 零部分写谱）；wasm 拆 flyState+flyProgress(done=1.0) 免 JS 双值编码。
- **near/far 不飞行**（pose 无深度域=恒现值，PIT-5/REND-14 连带 [Momus-A1]）；
  **改道 from=当前采样**保连续 [A3]；yaw 最短弧住 fly_sample 构造件——**mix_rig 零触碰**
  =REND-15/16 存量断言零回归（wrap 对照用线性中点反证双向锁）。
- dur_ms=0=瞬移形（非错误值域）；高度弧线/Van Wijk=AV 漫游单触发。

## D20: ⑤b 多视口带裁决（2026-09-18，双探针+Momus OKAY 0B/2A→修入 + 用户「全带开工」）
- **范围刀**：本带=分屏管线+活例（纯 rs）；capi 多视口口/click-to-fly=二波（CAPI-25 预留）；
  共享 viewer 库=另立项（E501/E901/E302/E303 四例同构 ≥3 阈值已破线，账转 D21 候选）。
- **clear 语义本机定论**（裁决门现行锁）：wgpu30 LoadOp::Clear=**整个 attachment**
  （AllClear 形后区吞前区 redA=0；与 Vulkan render-area 定论一致）⇒ **首 pass Clear
  （全幅底色=缝色零新机制）+ 后续 pass Load 色**为唯一安全形；AllClear 枚举位=探针/教学。
- **深度每 pass 必清**（E303 现行抓的现行雷，覆写探针预期）：pass 间 Load 会拿主视
  不透明地面的深度把小窗整幅 occlusion 吞（nonsky_map=0 现行）；scissor 圈地内重清
  零副作用、pass 间深度无消费者 ⇒ Clear+Discard 每 pass（免 Store 带宽）。
- **rect 挂点=不触 Frame**（ViewportRect 独立类型走渲染调用参数，第 44 兜底波免交）；
  full-rect 不发 set_* =旧路逐位等 canary 锁；FOV/px 皆 per-Frame 独立（两投自治构图）。
- **caster 一趟共享 map**（light-space 与主相机无关）；小窗=右上正方（长条=构图错实测）。
- **fov 单位雷清剿**（存量真 bug）：perspective=弧度消费（look_at 默认 π/3），E501/E302
  曾传 46/55/60 度数=投影畸变（E302 窗面 bright 96→1987 实证曾烂）；orbit 文档钉死单位；
  教训=像素门没看过的窗面=没验过的面（T3 人验与 headless 断言双保险的本带分册）。
