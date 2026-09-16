# 教程索引（E 编号系列）

> 单源纪律 [E3D:D1]：**文件名 = 头注释 = 本索引** 三方一致，机器锁在 `scripts/gate-docs.sh`。
> 空号 = 预留插位。跑法均可 `source pixi.sh` 后直接执行；「相关条款」指向 `docs/sdd/`。

| 编号 | 主题 | 文件 | 相关条款 | 跑法 |
|---|---|---|---|---|
| E101 | 窗口与首帧（winit×wgpu 清色循环） | `examples/rs/E101_clear.rs` | REND-01..08 / WGPU-01.. | `pixi run smoke-clear` |
| E201 | 数据装载·glTF（装配+轨道相机；4 连招首件） | `examples/rs/E201_load_gltf.rs` | GLTF-01..11 / WGPU-14 | `pixi run smoke-load-gltf` |
| E202 | 数据装载·GeoJSON（解析/投影/细分/样式合流） | `examples/rs/E202_geo_viewer.rs` | GEO-01..24 / D7 | `pixi run smoke-geo-viewer` |
| E203 | 属性与量测（平面 3857 距离/面积 CLI） | `examples/rs/E203_measure_cli.rs` | GEO-21/22 | `pixi run smoke-measure` |
| E301 | 相机与视图（透视↔正交无级切换） | `examples/rs/E301_switch_camera.rs` | REND-10..17 | `pixi run smoke-switch-camera` |
| E401 | 交互·拾取高亮（射线→pick→CPU 覆写闭环） | `examples/rs/E401_pick_demo.rs` | REND-21..24 / WGPU-12/13 | `pixi run smoke-pick` |
| E402 | 交互·hover/多选 UI —— **空号预留** | — | — | — |
| E501 | 材质与光影（instanced 城 + PCSS 软影） | `examples/rs/E501_shadow_demo.rs` | REND-31 / WGPU-14..20 | `pixi run smoke-shadow-demo` |
| E601 | 规模与性能（10 万楼块单 draw 压力例） | `examples/rs/E601_bench_twin.rs` | REND-27/28 / WGPU-16 / [6b] | `pixi run smoke-bench-twin`（`pixi run bench` 出制品） |
| E701 | 绑定镜像·C headless（嵌入样板+属性读闭环 name/opacity/missing≠0；跑不通=API 未完成 [E3D:D7]） | `examples/c/E701_demo_headless.c` | CAPI-01..08 | `bash scripts/gate-abi.sh` |
| E702 | 绑定镜像·C X11（attach 真窗口宿主骨架） | `examples/c/E702_demo_x11.c` | CAPI-06 | `bash scripts/smoke-x11.sh` |
| E703 | 绑定镜像·Qt6 真窗（widget 宿主带；texquad+park 双族） | `examples/qt/E703_qt_viewer.cpp` | CAPI-06 | `pixi run smoke-qt` |
| E801 | SDK 消费·C+SDL3 真窗（8x 带首婴；xid attach 三帧 present） | `examples/c/E801_sdl_window.c` | CAPI-06 | `ctest -L display`（cmake-smoke xvfb 子态） |
| E802 | SDK 消费·C++ headless（hpp 门面活体验收；readback→PPM 落盘） | `examples/cpp/E802_offscreen.cpp` | CAPI-04/05 | `ctest -R example_E802` |
| E901 | 垂直切片 seed·孪生城（geo 底图×instanced×PCSS×PNG） | `examples/rs/E901_twin_city.rs` | 批 4 全成果面 | `pixi run smoke-twin-city`（T2 新增） |

## IDE 运行（CMake target 面）

`pixi run cmake --preset bare`（或 IDE 直接打开工程选 bare/qt-pixi 预设）后：

- **Rust E 系（examples/rs）三件套**：构建步 `cargo-build_<E#>`（单例增量）、
  运行步 `cargo-run_<E#>`（构建+跑，输出终端）、ctest 条目 `example_<E#>`——
  argv 唯一户口在 `cmake/VisiaEngineBindings.cmake` 注册表，pixi `smoke-*` 任务
  均为 `ctest -R` 转发壳。cargo 环境自足由 cmake-smoke 态④三锚锁死（裸 PATH=
  IDE 直调无需 pixi 激活）；逐例断点调试走 rust-analyzer Run/Debug。
- **原生 E 系真身（examples/{c,cpp,qt}）**：add_executable 改一编一，
  ctest 同源条目，FOLDER=磁盘目录镜像（examples/<语言> 自动推导；根级步骤顶层裸列），身份语义归 LABELS。
- **ctest**：headless 族恒真跑；display 族默认 testPreset 已 `-LE display` 全跳，
  真跑经 xvfb 子态（S4 接 cmake-smoke 显示带）。

新增例必须同轮注册：Rust 例=Bindings.cmake 注册表行 + `examples/rs` 的
`[[example]]` 块；C/C++ 例=对应 `examples/<lang>/CMakeLists.txt` 的
`visiaengine_add_example()` 行（反-glob 中央账 configure 硬错，禁注释式禁用）。
E703（Qt 宿主带）仅 qt-pixi 预设下存在。
