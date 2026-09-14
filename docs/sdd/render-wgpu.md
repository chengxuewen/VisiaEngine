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

## WGPU-13: 深度遮挡正确性
mesh 管线挂 Depth32Float 面（pipeline `Less`+write on；pass 每帧 Clear(1.0)/store Discard；尺寸驱动 MeshCore 内部缓存重建）：多实体场景**近侧覆盖与 draw 顺序无关**（拾取命中件与渲染可见件一致——WGPU-12 双箱堆叠断言锁行为）。窗口/surface 路径与 headless 同深度配置（`render_view` 尺寸参数化）。

## WGPU-12: 选择高亮渲染面
拾取链闭环可渲染：屏幕射线（REND-21/22）→ `pick_meshes`（REND-23/24）→ 选中实体 material 以高亮纯色重建（**CPU 侧覆写，[E3D:A5] by-name 的渲染侧最小子集**）→ 中心像素=高亮色族断言（双箱堆叠场景，pick_highlight.rs）。选择态本体=宿主/example 层 `Option<EntityId>` 数据（引擎零新增状态）。

## WGPU-14: Textured 管线采样链
材质绑定 32B 块 `[base_color, repeat, specular, pad]`（binding2 与 Flat 同布局同尺寸——Flat 侧 shader 忽略后部字段=逐像素零回归的构造保证 [Momus-B1]）。textured 变体（独立 bgl/pipeline，mega-bool 分支否决 [E3D:B2]）：fragment `out = base·shade ⊗ texel.rgb`，`a = base.a·texel.a`；uv 顶点属性 @location(2)（缺失=零填充，全采 texel(0,0)）。`upload_texture`：RGBA8，行距补零至 256B（write_texture 约束），零尺寸/数据短=w·h·4 → Err；材质引用未知纹理 id → Err（不静默降 Flat）。specular mock-up [4ab①]：参与既有 Lambert 亮度系数（非 GGX 项——真 PBR=独立轮，诚实注记）；io 层因子零换算，`(1-metallic)*roughness` 反向映射住在消费者侧（capi/example）。

## WGPU-15: repeat=采样相位倍率
顶点 `out.uv = uv·mat.repeat`，sampler 全局单例 REPEAT/Linear：repeat=k → 视口内采样频率 ×k。机器 oracle：水平渐变纹理（R=4i）单行回绕断崖数 repeat=1 为 0、repeat=2 恰 1（棋盘奇偶对照在线性滤波下不成立——弃）。

## WGPU-16: Instanced 管线（4c）
`create_instances`：32B/条 storage 表（`STORAGE|COPY_DST`，REND-27 Pod 同形）；**空表建期即拒**；id 与 mesh/material 同计数域。`Variant::Instanced` 独立 bgl/pipeline（binding 0/2/**5 storage**，vs_inst 入口，组合 Textured 不装——材质 texture 位忽略）：`p=(x, y, z·height)+offset` 底对齐挤出、法向直传零误差（轴对齐盒 z 缩放不变向）、链 `out = base·inst.color·shade`、alpha=材质。消费面 match 三分支显式处理 DrawInstances（let-else 静默跳过=已封堵）；缺表=skip（mesh 缺失同纪律）。像素证据：三实例三色族 + 列高单调（挤出语义）+ 空表/缺表两语义锁。

## WGPU-17: Strokes 扩片管线（4de）
线段表 48B/条（`StrokeSeg` Pod，REND-30）入 binding5 storage；共享单位四边形（side∈±1，2 三角）常设顶点源 [E3D:B4 GS→VS 移植]。VS：`perp = normalize(cross(视向, 轴))`（视向=eye_local−mid，**eye_local 与 right/up/px_scale 同住 View 块 @binding0 128B——REND-29 兑现，三角系入口只读前 64B=零回归构造保证**）；退化 `|dot|>0.999 → perp=right`（NaN 不得污染同批 [R2 实义：视向平行段投影=点属几何事实，测面锁"整批存活"）；`halfw = width_px·px_scale·0.5`。FS 色直出（光照链不参与）。polygon-offset `bias{constant:-1, slope:-1}` 伴生件 [E3D:B7③]——盖同深度 fill 不 z-fight。缺表=skip；空表建期拒。像素证据四锁：族位（right/up 基符号）、宽度阶梯、offset 盖面、退化污染控制组。
