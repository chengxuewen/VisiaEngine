# 教程索引（E 编号系列）

> 单源纪律 [E3D:D1]：**文件名 = 头注释 = 本索引** 三方一致，机器锁在 `scripts/gate-docs.sh`。
> 空号 = 预留插位。跑法均可 `source pixi.sh` 后直接执行；「相关条款」指向 `docs/sdd/`。

| 编号 | 主题 | 文件 | 相关条款 | 跑法 |
|---|---|---|---|---|
| E101 | 窗口与首帧（winit×wgpu 清色循环） | `crates/visiaengine-render-wgpu/examples/E101_clear.rs` | REND-01..08 / WGPU-01.. | `pixi run smoke-clear` |
| E201 | 数据装载·glTF（装配+轨道相机；4 连招首件） | `crates/visiaengine-render-wgpu/examples/E201_load_gltf.rs` | GLTF-01..11 / WGPU-14 | `pixi run smoke-load-gltf` |
| E202 | 数据装载·GeoJSON（解析/投影/细分/样式合流） | `crates/visiaengine-render-wgpu/examples/E202_geo_viewer.rs` | GEO-01..24 / D7 | `pixi run smoke-geo-viewer` |
| E203 | 属性与量测（平面 3857 距离/面积 CLI） | `crates/visiaengine-geo/examples/E203_measure_cli.rs` | GEO-21/22 | `pixi run smoke-measure` |
| E301 | 相机与视图（透视↔正交无级切换） | `crates/visiaengine-render-wgpu/examples/E301_switch_camera.rs` | REND-10..17 | `pixi run smoke-switch-camera` |
| E401 | 交互·拾取高亮（射线→pick→CPU 覆写闭环） | `crates/visiaengine-render-wgpu/examples/E401_pick_demo.rs` | REND-21..24 / WGPU-12/13 | `pixi run smoke-pick` |
| E402 | 交互·hover/多选 UI —— **空号预留** | — | — | — |
| E501 | 材质与光影（instanced 城 + PCSS 软影） | `crates/visiaengine-render-wgpu/examples/E501_shadow_demo.rs` | REND-31 / WGPU-14..20 | `pixi run smoke-shadow-demo` |
| E601 | 规模与性能（10 万楼块单 draw 压力例） | `crates/visiaengine-render-wgpu/examples/E601_bench_twin.rs` | REND-27/28 / WGPU-16 / [6b] | `pixi run smoke-bench-twin`（`pixi run bench` 出制品） |
| E701 | 绑定镜像·C headless（嵌入样板+属性读闭环 name/opacity/missing≠0；跑不通=API 未完成 [E3D:D7]） | `examples/c/E701_demo_headless.c` | CAPI-01..08 | `bash scripts/gate-abi.sh` |
| E702 | 绑定镜像·C X11（attach 真窗口宿主骨架） | `examples/c/E702_demo_x11.c` | CAPI-06 | `bash scripts/smoke-x11.sh` |
| E901 | 垂直切片 seed·孪生城（geo 底图×instanced×PCSS×PNG） | `crates/visiaengine-render-wgpu/examples/E901_twin_city.rs` | 批 4 全成果面 | `pixi run smoke-twin-city`（T2 新增） |

## IDE 运行（CMake target 面）

`pixi run cmake --preset bare`（或 IDE 直接打开工程选 bare/qt-pixi 预设）后，
每件 E 系例同时是：

- **可运行 target**（FOLDER `examples/<带>` 分组）：launcher stub 自动 chdir 仓根
  （数据路径零改动）、转发参数（例表默认值即入口，也可自行加 `--frames N`）；
  构建 target 即触发对应 crate 的 cargo example 增量编译（cargo 调用环境自足：裸 PATH 下 rustc 可达已由 cmake-smoke 第四态锁死——IDE 直调无需 pixi 激活）
- **ctest 一条** `example_E*`：headless 族恒真跑；显示族无 DISPLAY 时预检
  exit 77 → ctest 记 Skipped（`ctest --preset bare` 全族 / `-L example -LE display`
  headless 子集；qt-pixi 预设 testPreset 已排除 display 族，E703 真窗件由
  smoke-qt 三态门专职）

E703（Qt 宿主带）仅 qt-pixi 预设下存在；新增例必须同轮入
`cmake/VisiaEngineExamples.cmake` 契约表（configure 硬错兜底，禁注释式禁用）。
