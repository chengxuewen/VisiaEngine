# cmake × Rust 集成形态对比（iceoryx2 / slint / zenoh / MediaServo）

> 来源：团队调研 4 探针（2026-09-14，`.refinfo/` 本机归档只读取证；全部 file:line 均可复核）。
> 服务决策：VisiaEngine 的 CMake 集成选型（Qt widget 轮消费面 + 打包轮 `bindings/` 设计）。

## 谱系（按 cargo↔cmake 耦合度排序）

| 项目 | 机制 | 产物→target 映射 | 头文件 | 核心教训 |
|---|---|---|---|---|
| **zenoh**（core 仓） | **零 cmake**——绑定面全数外置兄弟仓（zenoh-c/-python/…），交接点=crates.io 发布（release.yml:86-94） | n/a | 无 C 头（Rust API+feature 门即合同，`internal` 门=绑定逃生舱 zenoh/Cargo.toml:47） | 「core 仓无 cmake」可以是**设计而非缺失**；仓边界=版本合同，非 path 耦合 |
| **MediaServo**（前作） | 构建期零集成：cargo 由脚本驱动，**打包时渲染** Config.cmake.in 模板（python 替换 @VERSION@ 等） | 每组件一个 INTERFACE IMPORTED，WIN32/APPLE/UNIX 名映射**只活在模板里**（mediaservoConfig.cmake.in:9-18） | 手写（弃 cbindgen 在案）+ abi-drift 外部门 | **最大反面教材：find_package 通路从未被任何测试编译过**——cmake 包=发货的暗代码；SONAME 符号链接 shim、LD_LIBRARY_PATH 手动、.so 覆盖 ETXTBSY |
| **iceoryx2** | CMake 自管 cargo：`add_custom_target(ALL COMMAND cargo build …)`（顶层 CMakeLists:190-201）+ **逃生舱** `-DRUST_BUILD_ARTIFACT_PATH=`（预构建纯消费态 :153） | 非 IMPORTED——INTERFACE 库 + 生成器表达式装绝对路径（iceoryx2-c/CMakeLists.txt:62-78）；注释自白：IMPORTED 方案「全部试过不行」:56-60 | cbindgen 于 build.rs，路径=OUT_DIR 逆向字符串（byproduct-defs:13） | 双态 config 花招（树内假 Config+FORCE _DIR / 装后生成版）让 in-tree 与 installed 消费代码**同文**；弱=排序脆弱、无 SOVERSION、DLL 装进 lib/ |
| **slint** | **Corrosion**（find_package→FetchContent v0.6.1 回退，api/cpp/CMakeLists:16-26）`corrosion_import_crate` 一呼全起 | corrosion 每 crate-type 造 `slint_cpp-shared/-static` IMPORTED，**逐 OS 命名/构建型→profile 映射零自码** | cbindgen 1383 行生成器（大 FFI 面才需要） | 消费面收敛=一个 `Slint::Slint` INTERFACE 伞；示例通式 `if(NOT TARGET …) find_package(…)`；代价=531 行里 250 行是 Android/Yocto/Skia 特管，**别抄** |

## Qt 消费先例（slint 独有，MediaServo 零先例）

- `examples/cpp/platform_qt/main.cpp:44-73`：`winId()` → per-platform `NativeWindowHandle{from_x11_xlib/win32/appkit}`——**与我方 `visiaengine_attach(win,display,kind)` 同构**，证明整型窗柄+kind 参数路线正典。
- 泵：`paintEvent→render()`、`request_redraw→requestUpdate()`（同文件 :202-211）+ QTimer——=我方设计，无帧回调推送。
- `qt_viewer`（widget 嵌入）与 `iot-dashboard`（零 codegen 纯链伞）= demo 消费 CMakeLists 模板两式。

## 对 VisiaEngine 的落位（裁决草案）

1. **仓型**：采 zenoh 论但按我们的规模折中——ABI 实现留 `crates/visiaengine-capi`，分发面 `bindings/` 预留；**pre-1.0 不学它的 crates.io 交接**（workspace path+gate-abi/CAPI-09 镜像门禁=我们的版本合同平替）。
2. **Qt 轮消费集成**：v0 用 iceoryx2 式**薄自管**（`add_custom_target cargo build --package visiaengine-capi --target-dir <cmake-bin>/rust` + `RUST_BUILD_ARTIFACT_PATH` 逃生舱 + INTERFACE 伞 `visiaengine::capi` + BUILD/INSTALL 双 genex），**不引 corrosion**（FetchContent 网络面与 pixi/D5 离线纪律相抵；corrosion 转正=打包轮议题，触发=第二宿主语言出现）。
3. **硬门禁继承**（三案共债）：find_package 通路**必须有真实消费者编译冒烟**（Qt demo 兼任；入 ci 段）——MediaServo 头号反面教训。
4. **零改动即得**：capi Cargo.toml 加 `links = "visiaengine_c"`（重复静态链防呆，slint 一行式）；头文件合同注释（owner 线程/close 失效）保持现手写纪律。
5. **SKIP 清单**：COMPONENTS 多组件（单库不派生）、SOVERSION 装树、版本化 include 前缀、cbindgen、.pc 双发（打包轮再议）、SONAME dev-symlink（v0 直链 target 产物即无此问题）。
