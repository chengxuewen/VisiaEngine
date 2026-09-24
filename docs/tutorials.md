# 教程索引（E 编号系列）

> 单源纪律 [E3D:D1]：**文件名 = 头注释 = 本索引** 三方一致，机器锁在 `scripts/gate-docs.sh`。
> 空号 = 预留插位。跑法均可 `source pixi.sh` 后直接执行；「相关条款」指向 `docs/sdd/`。

| 编号 | 主题 | 文件 | 相关条款 | 跑法 |
|---|---|---|---|---|
| E101 | 窗口与首帧（winit×wgpu 清色循环） | `examples/rs/E101_clear.rs` | REND-01..08 / WGPU-01.. | `pixi run smoke-clear` |
| E201 | 数据装载·glTF（装配+轨道相机；4 连招首件） | `examples/rs/E201_load_gltf.rs` | GLTF-01..11 / WGPU-14 | `pixi run smoke-load-gltf` |
| E202 | 数据装载·GeoJSON（解析/投影/细分/样式合流） | `examples/rs/E202_geo_viewer.rs` | GEO-01..24 / D7 | `pixi run smoke-geo-viewer` |
| E203 | 属性与量测（平面 3857 距离/面积 CLI） | `examples/rs/E203_measure_cli.rs` | GEO-21/22 | `pixi run smoke-measure` |
| E204 | 数据装载·点云（螺旋自断言双模 + `--file` PLY 装载路 + `--pick-check` 点拾取三断言 B2） | `examples/rs/E204_pcl_viewer.rs` | IO-01..06 / CAPI-18/19 / WGPU-18 / REND-38/39 | `pixi run smoke-pcl-viewer` |
| E205 | 图注文字·标注双模（io-text 全链活例；无参常驻窗/`--frames` 白墨断言） | `examples/rs/E205_text_labels.rs` | IO-07..09 / REND-33 / WGPU-24/25 | `pixi run smoke-text-labels` |
| E206 | 数据装载·矢量瓦片（MVT 解码→GeoTile→三族渲染；FileSource 装载捆绑合成瓦片） | `examples/rs/E206_tile_viewer.rs` | IO-10..12 / GEO-27 | `pixi run smoke-tile-viewer` |
| E301 | 相机与视图（透视↔正交无级切换） | `examples/rs/E301_switch_camera.rs` | REND-10..17 | `pixi run smoke-switch-camera` |
| E302 | 相机飞行·预设巡览（双模：1/2/3 视角互飞/拖滚中断=飞行 cancel；`--frames` 端点+最短弧+渲染三断言） | `examples/rs/E302_fly_camera.rs` | REND-34 / CAPI-23 | `pixi run smoke-fly-camera` |
| E303 | 分屏驾驶舱·主透视+顶视小地图（双模：1/2/3 飞行小窗跟随/拖滚接管；`--frames` 分区四断言） | `examples/rs/E303_split_screen.rs` | REND-35 / WGPU-26 | `pixi run smoke-split-screen` |
| E401 | 交互·拾取高亮（射线→pick→CPU 覆写闭环） | `examples/rs/E401_pick_demo.rs` | REND-21..24 / WGPU-12/13 | `pixi run smoke-pick` |
| E402 | 交互·hover/多选窗（悬停=橙预览 点选=黄多选 toggle 拖=轨道 R=清空） | `examples/rs/E402_pick_interactive.rs` | REND-21/23 / WGPU-12 | `pixi run smoke-pick-hover` |
| E501 | 材质与光影（instanced 城 + PCSS 软影；双模式=无参交互窗/`--frames` headless） | `examples/rs/E501_shadow_demo.rs` | REND-31 / WGPU-14..20 | `pixi run smoke-shadow-demo` |
| E502 | 材质与光影·色彩标定（sRGB 全链往返「所见即所得」活证；四色板 headless） | `examples/rs/E502_color_tuning.rs` | CORE-16 / WGPU-14 | `pixi run smoke-color-tuning`（例内自断言） |
| E503 | 材质与光影·扩片族独立课（屏幕恒线宽/真圆点·缩放秀） | `examples/rs/E503_stroke_points.rs` | REND-29/30 / WGPU-17/18 | `ctest -L display`（cmake-smoke xvfb 子态） |
| E504 | 玻璃与水体·透明活例（双模：1/2=水 alpha 重传材质；`--frames` 水膜衰减带+恒顶标签断言，含 a=1.0 对照语义锁） | `examples/rs/E504_glass_water.rs` | REND-36 / WGPU-27/28 | `pixi run smoke-glass-water` |
| E601 | 规模与性能（10 万楼块单 draw 压力例） | `examples/rs/E601_bench_twin.rs` | REND-27/28 / WGPU-16 / [6b] | `pixi run smoke-bench-twin`（`pixi run bench` 出制品） |
| E701 | 绑定镜像·C headless（嵌入样板+属性读闭环 name/opacity/missing≠0；跑不通=API 未完成 [E3D:D7]） | `examples/c/E701_demo_headless.c` | CAPI-01..08 | `bash scripts/gate-abi.sh` |
| E702 | 绑定镜像·C X11（attach 真窗双模=无参常驻/`--frames` 快退） | `examples/c/E702_demo_x11.c` | CAPI-06 | `bash scripts/smoke-x11.sh` |
| E703 | 绑定镜像·Qt6 真窗（widget 宿主带；texquad+park 双族） | `examples/qt/E703_qt_viewer.cpp` | CAPI-06 | `pixi run smoke-qt` |
| E704 | 绑定镜像·C 事件回调（CAPI-17 推送口三态：进度单调/错误同刻/NULL 摘除） | `examples/c/E704_host_callback.c` | CAPI-17 | `ctest`（native headless 族） |
| E801 | SDK 消费·C+SDL3 真窗（双模式=无参交互窗含 resize 联动/`--frames` 快退） | `examples/c/E801_sdl_window.c` | CAPI-06 | `ctest -L display`（cmake-smoke xvfb 子态） |
| E810 | SDK 消费·C 属性遍历 survey（name/opacity/缺失≠零值现场） | `examples/c/E810_attr_survey.c` | CAPI-04/10/11 | `ctest -R example_E810` |
| E811 | SDK 消费·显隐孪生（C 面；枚举域不变/查询往返/越值拒） | `examples/c/E811_entity_hide_c.c` | CAPI-13/14 | `ctest -R example_E811_entity_hide_c` |
| E811 | SDK 消费·显隐孪生（C++ 面；hpp 转发验收） | `examples/cpp/E811_entity_hide_cpp.cpp` | CAPI-13/14 | `ctest -R example_E811_entity_hide_cpp` |
| E812 | SDK 消费·程序化增删（位形互异/退化零提交/再入拒） | `examples/cpp/E812_mesh_add.cpp` | CAPI-15/16 | `ctest -R example_E812` |
| E802 | SDK 消费·C++ headless（hpp 门面活体验收；readback→PPM 落盘） | `examples/cpp/E802_offscreen.cpp` | CAPI-04/05 | `ctest -R example_E802` |
| E814 | SDK 消费·C 文字标注（时序门/utf8 多字节/白墨活体门） | `examples/c/E814_labels_headless.c` | CAPI-21/22 | `ctest -R example_E814_labels_headless` |
| E815 | SDK 消费·C 相机飞行（idle 即 done/瞬移/进度单调/cancel/域拒 活体门） | `examples/c/E815_fly_camera.c` | CAPI-23/24 | `ctest -R example_E815_fly_camera` |
| E816 | SDK 消费·C 小地图导航（canary 逐字节/角区顶视族/小图 pick/导航到站保角保距/值域拒 五段活体门） | `examples/c/E816_minimap_nav.c` | CAPI-25..27 | `ctest -R example_E816_minimap_nav` |
| E813 | SDK 消费·剖面裁切（半刀/角域三面 AND/值域拒/清空复原，readback 三段活体门） | `examples/c/E813_section_clip.c` | CAPI-20 | `ctest -R example_E813_section_clip` |
| E901 | 垂直切片 seed·孪生城（geo 底图×instanced×PCSS；交互窗带 C 剖切刀/[/] 刀高/L 标签；`--frames` 离屏 PNG+断言） | `examples/rs/E901_twin_city.rs` | 批 4 全成果面 + REND-32/33 活键演示 | `pixi run smoke-twin-city`（T2 新增） |

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

## 人验形态（双模约定 · IDE 手跑与自动测试同一入口）

所有例子经统一 argv 双模分路：IDE 的 `cargo-run_*`（rs）/`run_*`（native）目标**零参启动 = 人验形态**；
ctest / `--frames N` = 自动化短退形态（argv 单源纪律：注册表参数只服 ctest，run 步零参——C15）。

| 人验形态 | 例 | 无参启动行为 | 看什么 |
|---|---|---|---|
| **窗口常驻** | E101 / E201 / E202 / E205 / E301 / E302 / E303 / E402 / E501 / E502 / E503 / E504 / E702 / E801 / E901 | 开交互窗、不自动退（Esc/关窗退出） | 肉眼验画面：E502 四色板=色彩链，余者标题即操作提示 |
| **自退自证** | E203 / E401 / E601 / E701 / E704 / E802 / E810 / E811_c / E811_cpp / E812 / E813 / E814 / E815 / E816 | 跑完打印 `OK …` 行退出 | 终端输出即验收面 |

**无头证据例“秒退=设计”**：其产物是终端断言行或 PNG/PPM 落盘，非交互窗；要交互验收走同能力域窗口例
（拾取→E402、色彩→E502 无参窗、光影→E501 无参窗、恒宽→E503）。native run 步全例覆盖（DISPLAY 族经
run-gui 探测 `:0` 回退；headless 族直跑，IDE 无 DISPLAY 亦可）。

native 例窗况定性（2026-09-17 E702 修案同轮盘点）：真窗口例=E702/E801/E703 三枚（无参全常驻）；
其余 C/C++ 例（E701/E704/E802/E810/E811×2/E812/E813/E814）=无头证据例，终端 `OK …` 行/PPM 落盘
即其人验面，秒退=设计身份。
