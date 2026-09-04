# visiaengine-render-wgpu 行为契约（SDD）

> 条款格式：`## WGPU-NN: 名称`。双向追溯：`scripts/spec-trace.sh`。

## WGPU-01: instance_create_headless
`create_instance()` 以 `Backends::PRIMARY` 构造 wgpu Instance：不 panic、不依赖 DISPLAY/表面即为通过（本机无 GPU 时构造本身仍合法）。

## WGPU-02: adapter_enumeration_typed
`available_adapters()` 返回 `Vec<wgpu::AdapterInfo>` 类型面：无适配器时空 vec 合法，panic 非法（驱动缺失是可报告状态，不是崩溃理由）。

## WGPU-03: golden_center_pixel
`render_offscreen_triangle()`（640×480 RGBA8）中心像素为红（R≥200，G/B≤60，A=255，容差 16）；无可用适配器时打印 SKIP 并合法返回（验证地点义务由调用方记录）。

## WGPU-04: golden_frame_dimensions
返回帧的 `rgba.len()` 恰为 `width*height*4`（行距 256 对齐由尺寸选取保证，640 天然满足）。

## WGPU-05: golden_corner_clear_color
四角像素等于清屏色 (13,18,26)±16——全屏污染的反证。

## L2 窗口 smoke（叙述性条款，**不占编号、不入双向 grep**）

`examples/clear.rs` 接受 `--frames N` 自动退出；CI 以 `xvfb-run -a pixi run smoke-clear`
（N=3）断言 exit 0 = winit→surface→帧循环全链路 E2E 通。本机无 DISPLAY 时该层仅在 CI 验证。

## WGPU-06: golden_cube_center_hit
`render_offscreen_cube()`（程序化 8 顶点立方 MeshDesc，零资产依赖）中心像素红主导（r≥g+40 且 r≥b+40）——着色非清屏。

## WGPU-07: golden_cube_corner_clear_color
四角=清屏色 ±16（立方不污染全帧的反证）。

## WGPU-08: golden_cube_silhouette_row
中线行非清屏段：左右恰 2 次清↔染跳变 + 连续段宽 >100px（正交正面投影的尺寸正确性）。

## WGPU-09: material_uniform_path_live
同一 offscreen 管线换 material（绿）→ 中心像素绿主导（uniform 绑定路径实跑证据，非 SKIP 假绿）。

## WGPU-10: golden_far_origin_pixel_identity
`render_offscreen_cube_at(offset)`：立方 local 网格 + origin=(1e7,0,0) + 相机随之 10m——中心红/四角清 与 origin=0 版**像素一致**（D7 rebase 在真管线的端到端证明；无适配器 SKIP）。

## WGPU-11: golden_geo_fill_hit
park.geojson 全链路（解析→细分→D7 上传→正交鸟瞰离屏）：buildingB 质心投影像素邻域命中默认蓝填充——geo×管线合流的存在性证明（无适配器 SKIP）。
