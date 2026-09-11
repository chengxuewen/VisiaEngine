# FFI/C-API 绑定模式研究 —— 团队 ffi-best-practices 证据档

**快照 2026-09-11**（团队 4 路 × 28 个一手源：实拉头文件至本机 /tmp 会话档 + ~/.cargo/registry 源码 + `.refinfo/MediaServo/bindings` 自家交付件解剖；行号为实文件行号）。**用途**：批 2/批 7 计划中 `[FFI-R:xx]` 引用的号码正本（防悬空引用）。模式知识非 GPL（全部 MIT/Apache/BSD/公开规范 + 自家 MediaServo）。

## 结论一句话
批 2 §2 十四函数面与三个最成熟同类（**Sokol/Filament/webgpu 标准头**）形状收敛、零结构性错误；两轮 Momus + 本调研前全数未抓的四处真空白已由本调研钉死并落入 v1.3（struct_size / 覆写断言 / gate 工具名 / re-attach 语义）。

## 引用号码表

### CS = capi-surface（9 源：SDL3/Vulkan/godot-headers/llama/sqlite/sokol/webgpu-headers/zlib/ggml）
| 号 | 结论 | 证据 |
|---|---|---|
| CS-M1 | VeInput 演进政策缺位→采「memset 零初始化+尾部追加 0=旧行为」SQLite iVersion 轻档；拒绝 Vulkan pNext 链（v0 单结构 K8） | sqlite.h.in:855；vulkan_core sType 1157 处（重） |
| CS-M3 | last_error 两断言入测试：成功调用不覆写（SDL 原文「勿以错误串判分支」）+ 失效谓词（SQLite「可能被后续调用覆写」原文）| sdl_error.h:31/132；sqlite.h.in:4245 |
| CS-M2 | 每函数 `\threadsafety` 头注行（SDL3 惯例，与「一行用途」同位） | sdl_error.h:151 |
| CS-R1 | **句柄位布局不写 C 头**（sokol 敢写因源码分发；二进制 SDK 写=邀请拆解持久化；Vulkan 从不文档位布局）；只写「值不透明，仅 0/UINT64_MAX 有含义」 | sokol_gfx.h:2131 注释 vs vk_platform.h 沉默 |
| CS-2 | u64 slot+gen 世代句柄：sokol 直接先例（16+16 位、slot 0 保留=我方基 1 同构）+ Vulkan 32 位平台强制 u64 恒宽先例；**世代句柄是少数派但有实效**（除 sokol 外无人外露） | sokol_gfx.h:2131-2147；vk_platform.h:26-58 |
| CS-4 | 错误协议码+串双口拉式 = SQLite/SDL 同派（勿被 webgpu `Success=1` 带跑：其 Success≠0 惯例） | webgpu.h:207 |
| CS-5 | packed u32 版本与 Vulkan/SDL 同族；ELF .symver（zlib.map）=Alpha 可选非 v0 缺口 | vk_platform.h:62；zlib.map |
| CS-9 | 拉模型：sokol sg_commit/webgpu present/SDL/llama 全宿主驱动；SDL3 的 push 层（SDL_AppIterate）恰证「拉核心+推框架」双层先例=我方 frame_requested 缓评同构 | sdl_init.h:89-133 |
| CS-10 | VeInput 平铺 struct 与 SDL_Event/WndProc 同族（定长超集比分派 union 懒且够） | sdl_events.h:1100 |

### FC = rust-ffi-craft（Rust→cdylib 工程；一手=[MS] `.refinfo/MediaServo/bindings` 实交付件 + wgpu-native 抓源）
| 号 | 结论 | 证据 |
|---|---|---|
| FC-M1 | **gate-abi/demo 构建工具名**：本机无裸 nm/gcc（编排者复核 ✓），conda 件带前缀；修法=I0 host feature 扩 binutils+gcc 或前缀件名 | `which nm`=1；`x86_64-conda-linux-gnu-nm` ✓ |
| FC-M2 | 输入线程/paint 线程可异→头文件钉「宿主 marshal 到 owner 线程」义务句 | Qt QWindow 线程语义 |
| FC-M3 | 14 栅栏收单宏 `capi_guard(\|\|…)`；demo 首行 abi_version assert（ABI 门焊进测试） | [MS] 12 extern×13 catch_unwind |
| FC-1 | 手写头定调：wgpu-native（685 函数）手写、上游哲学「勿编辑生成物」；cbindgen 翻案门槛=40 函数 | wgpu-native README；cbindgen README |
| FC-2 | musl/嵌入式默认 crt-static 使 cdylib 不可产（需 `-C target-feature=-crt-static`）→ docker.md 注记 ✓ | cargo-c troubleshooting |
| FC-3 | cdylib 无 LTO 不泄 rust_eh_personality（[MS] 在世 .so 实证）——nm==14 可保；version-script 不做（GNU-ld 单脚本+macOS 不对称） | [MS] build.rs/.so |
| FC-4 | owner-tid 检查派实证（[MS] last_error thread_local）；v0 单线程+`!Send` 文档化，Mutex<Slab> 只管槽，engine 不跨线程共享 | [MS] c/src:109 |
| FC-5 | 风格定版：`#[unsafe(no_mangle)] pub extern "C"` + 安全签名内部校验——校验在函数体内做、返回错误码而非调用点 UB（[MS] 12× 实装同款） | [MS] c/src 全篇 |
| FC-6 | canonical 链接行 + soname defer 有据（[MS] build.rs `-Wl,-soname` 先例挂 release 轮）；abidiff conda 在库（libabigail 2.7）但 14 函数面不配 | [MS] build.rs |

### BS = binding-stack（7 家分层：llama/DuckDB/SQLite/MapLibre/godot/napi-rs/sql.js+duckdb-wasm）
| 号 | 结论 | 证据 |
|---|---|---|
| **BS-1** | **头号采纳：VeInput `struct_size` 首字段**+库校验（MediaServo 血泪写进自家头原文；godot method-info 同法）。与 EP-8 冲突裁决：EP 触发器「动态库二进制分发」**已是现在**（gate-abi 验的 .so 即动态分发物；sokol 无 size 活十年靠源码重编译契约） | [MS] 头文件注释；gdextension_interface.h |
| BS-2 | 兼容方向承诺一行入 CAPI-07（godot 措辞：旧头→新 .so 可跑，永不反向，MAJOR 内只增） | godot 文档 |
| BS-3 | wasm-pack **未死**（repo 2026-08 有 push，本调研纠正任务前提）；直用 CLI 理由改「conda 无包+少活动部件」 | npm/github registry |
| BS-4 | CAPI-09 镜像联动（v13 审核修正）：跨 arch 禁固定数字面（wasm32 size_t=4）——Rust 半区两断言 `struct_size=1→-1`、`真实 sizeof→≠-1`；node 半区 typeof bigint 改**静态 grep 生成物** `.d.ts` 含 `pick(...): bigint`（node 无 GPU 实例不可构造，运行时 typeof 不可行，[FFI-R:v13-FEAS-1/7]） | [MS] lib.rs:257 拒断先例 |
| BS-5 | 路线 A 终审：单 crate 双面非主流（主流=每语言独立 repo），但**不变式=一内核+纯编组薄面**，单厂商 SDK 用双面+镜像守卫更廉且免疫 MapLibre 式漂移；**emscripten 拒绝获语言匹配律**（sql.js=编译 C、duckdb-wasm=编译 C++、wgpu=Rust→bindgen——三例全合） | 三 wasm 项目 README |
| BS-6 | 所有权四模式目录（M1 borrowed/M2 free 派/M3 caller-alloc/M4 by-value）——我方 M1+M3 组合=SQLite+MediaServo 双派；**M2 free fn 禁令维持**（头号 C 内存 bug 农场） | duckdb.h / sqlite / [MS] |
| BS-7 | 「杀绑定的是 ABI 翻搅非年龄」：llama-cpp-node 死（repo 404/停更 2024）+ node-sqlite3 DEPRECATED（原生派停维护→平台内置吸收 node:sqlite）——CAPI-07 冻结的又一实证；node 面停在 abiVersion+常量（无 WebGPU，不建 napi 面）=正解 | 各 repo 实况 |
| BS-8 | proc-address 注册表（godot/DuckDB 插件式）不采：14 符号+静态链接+单厂商；若未来第三方 .so 载我方 ABI，C API 即锚（DuckDB 1.2 先例，形状已对） | gdextension 机制 |

### EP = embed-peer（7 同类嵌入面：wgpu-native/Filament/bgfx/sokol/raylib/Flutter/UaaL）
| 号 | 结论 | 证据 |
|---|---|---|
| EP-1 | attach(ve,win,display,kind) 三标量**CONFIRM**（webgpu.h SurfaceSource 字段同集合；kind 枚举=bgfx NATIVE_WINDOW_TYPE 先例）；RECONSIDER 文本面→**CAPI-06 钉 kind=win32 时 display_ptr=hinstance（0=自动）** | webgpu.h:3180-3350；bgfx.h:784 |
| EP-2 | u64 句柄双实证（sokol 16+16+slot0 保留）；RECONSIDER=类型化 struct 包装编译期防串用——**默认不加**（if(!ve) 惯用法是既裁决），列 D8 知情取舍 | sokol_gfx.h:2125-2150 |
| EP-3 | 输入 struct CONFIRM（sapp_event 平铺同族）；缺 frame_count 时间戳=D8 触发器②（相机惯性时补） | sokol_app.h:1560 |
| EP-4 | 拉模型三先例 CONFIRM（Filament do-loop/bgfx_frame/sg_commit；Unity UaaL 反例=自转+全屏锁死恰证对立面）；beginFrame()->bool 跳帧协商口=D8 触发器① | Engine.h:107；uaal README |
| EP-5 | 单线程 owner CONFIRM 且我方更严（bgfx 暗起 render 线程=反例；Flutter platform-thread 断言=直接先例） | embedder.h 纪律 |
| EP-6 | headless=同门面 CONFIRM（Filament createSwapChain(w,h)/bgfx nwh=NULL 双先例） | SwapChain.h:164 |
| EP-7 | viewport() 单口 CONFIRM（bgfx_reset 打包 present 策略我方 v0 全不暴露=对，真机轮再裁） | bgfx.h:2510 |
| EP-8 | struct_size 触发器论（与 BS-1 冲突，裁决见 BS-1：触发器已满足→采） | — |
| EP-9 | **re-attach 语义六家无先例可抄**→CAPI-06 必须钉死「重复 attach=旧目标释放后重配」否则实现侧留竞态 | 七家扫描 |
| EP-10 | Qt demo 三块拼图一手钉死：`QWidget::winId()`→XId ✓ `QNativeInterface::QX11Application::display()`「for use with Xlib」✓（display_ptr 直接用 Qt 的）；QQuickRenderControl 仅 QML 合成路需要，widget 路不需要（维持 qt-3d.md 判定） | Qt 6.11 docs 三页 |
| EP-附 | demo 即文档（canonical 10 行嵌入样板=Filament 循环 C 化，各家 README 同法）——I2/I3 demo 代码按「可直接贴 README 的文档件」标准写 | 各家 README |

## v13-final-review 增补行（第二批证据，v1.4 引用正本）
| 号 | 结论 | 证据 |
|---|---|---|
| v13-CONS-1 | 审方「13≠14」为**误报**（其行号清单自证 14 项）——编排者重数+feasibility 独立计数双 14，驳回；审方产物亦须 C14 反核之实证 | 代码块 sed 计数 14 |
| v13-FEAS-2 | abi assert `>>16==0` vs v0=0x00010000(major=1) 自相矛盾，coverage #6 独立同证→**==1** | 两审双证 |
| v13-FEAS-4 | canonical 链接行三重死：pixi 0.78 `$$` 硬解析错/裸 cc-nm 无/`-lvisiaengine` 缺 `[lib] name` → 零-$+conda 前缀+LD_LIBRARY_PATH 版 | pixi run 实测 |
| FC-6 | spec-trace 不在本地 ci 链（只在待命 ci.yml L33）→ `gate-trace` 任务入 ci depends-on 本机化；正则实为**三处** alternation（tags 行内 2 处，漏一=静默空集） | ci.yml/pixi.toml 实况 |
| env-W4 | 条件段先例=零（ci.yml L35-42 全无条件）——smoke-web/smoke-x11 条件=步骤级 shell 门自写 | ci.yml |
| v13-LEDG-* | 账本五处 ready text/graveyard 回补/README 43→83 诚实债/status 双 stale | ledger-auditor 表 |

## 落点汇总（v1.3→v1.4）
批 2：VeInput+struct_size / CAPI-06 双钉 / CAPI-07 双承诺 / I1 测试+3 / 风格+工具名定版 / D8 触发器×4 / threadsafety 头注。批 7：wasm-pack 措辞 / CAPI-09 sizeof 联动 /（路线 A、emscripten 拒绝、无 free 纪律获 7 家背书，零改动）。横切：docker.md musl 注记。
**四路零推翻核心设计**（14 签名、拉模型、单入口、世代句柄、无回调、线程亲和、薄口单源全部 CONFIRM 带名有姓）；净增四处「两轮 Momus + 一轮团队评审均漏」的实战缺陷——调研的边际价值实证。
