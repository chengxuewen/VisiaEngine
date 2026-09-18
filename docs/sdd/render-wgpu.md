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
材质绑定 32B 块 `[base_color, repeat, specular, pad]`（**值域契约 CORE-16**：base_color 宿主面=sRGB/CSS 惯例，后端写 uniform 前转线性——光照全链线性域，srgb 目标 store 硬件编码回人眼域）（binding2 与 Flat 同布局同尺寸——Flat 侧 shader 忽略后部字段=逐像素零回归的构造保证 [Momus-B1]）。textured 变体（独立 bgl/pipeline，mega-bool 分支否决 [E3D:B2]）：fragment `out = base·shade ⊗ texel.rgb`，`a = base.a·texel.a`；uv 顶点属性 @location(2)（缺失=零填充，全采 texel(0,0)）。`upload_texture`：RGBA8（**字节=sRGB 编码**，Rgba8UnormSrgb 格式=采样硬件解码 [CORE-16]），行距补零至 256B（write_texture 约束），零尺寸/数据短=w·h·4 → Err；材质引用未知纹理 id → Err（不静默降 Flat）。specular mock-up [4ab①]：参与既有 Lambert 亮度系数（非 GGX 项——真 PBR=独立轮，诚实注记）；io 层因子零换算，`(1-metallic)*roughness` 反向映射住在消费者侧（capi/example）。

## WGPU-15: repeat=采样相位倍率
顶点 `out.uv = uv·mat.repeat`，sampler 全局单例 REPEAT/Linear：repeat=k → 视口内采样频率 ×k。机器 oracle：水平渐变纹理（R=4i）单行回绕断崖数 repeat=1 为 0、repeat=2 恰 1（棋盘奇偶对照在线性滤波下不成立——弃）。

## WGPU-16: Instanced 管线（4c）
`create_instances`：32B/条 storage 表（`STORAGE|COPY_DST`，REND-27 Pod 同形）；**空表建期即拒**；id 与 mesh/material 同计数域。`Variant::Instanced` 独立 bgl/pipeline（binding 0/2/**5 storage**，vs_inst 入口，组合 Textured 不装——材质 texture 位忽略）：`p=(x, y, z·height)+offset` 底对齐挤出、法向直传零误差（轴对齐盒 z 缩放不变向）、链 `out = base·inst.color·shade`（inst.color 上传期线性化副本入表 [CORE-16 咽喉五]）、alpha=材质。消费面 match 三分支显式处理 DrawInstances（let-else 静默跳过=已封堵）；缺表=skip（mesh 缺失同纪律）。像素证据：三实例三色族 + 列高单调（挤出语义）+ 空表/缺表两语义锁。

## WGPU-17: Strokes 扩片管线（4de）
线段表 48B/条（`StrokeSeg` Pod，REND-30）入 binding5 storage；共享单位四边形（side∈±1，2 三角）常设顶点源 [E3D:B4 GS→VS 移植]。VS：`perp = normalize(cross(视向, 轴))`（视向=eye_local−mid，**eye_local 与 right/up/px_scale 同住 View 块 @binding0 128B——REND-29 兑现，三角系入口只读前 64B=零回归构造保证**）；退化 `|dot|>0.999 → perp=right`（NaN 不得污染同批 [R2 实义：视向平行段投影=点属几何事实，测面锁"整批存活"）；`halfw = width_px·px_scale·0.5`。FS 色直出（光照链不参与）。polygon-offset `bias{constant:-1, slope:-1}` 伴生件 [E3D:B7③]——盖同深度 fill 不 z-fight。缺表=skip；空表建期拒。像素证据四锁：族位（right/up 基符号）、宽度阶梯、offset 盖面、退化污染控制组。

## WGPU-18: Points splat 管线（4de）
点表 32B/条（`PointMark` Pod）入 binding6 storage；四边形常设顶点源屏幕基展开 `p = pos + right·(sx·r) + up·(sy·r)`，`r = radius_px·px_scale`；**FS 单位盘 mask `length(local)>1 → discard`=真圆点**（纠 geo 方块存量）。色直出 alpha=1（同 WGPU-17 链）；depth-bias 伴生同 Strokes（盖面）。空表建期拒/缺表 skip/族位右上一致性锁（right/up 符号面）。

## WGPU-19: shadow pre-pass 与接收 PCF（4f）
caster pass：无色彩目标 RenderPass（Depth32Float 1024² 懒建常驻，store=Store）+ 两 caster 管线（`ShadowMesh{vs_shadow}`/`ShadowInst{vs_shadow_inst}`，复用 mesh 顶点布局与 binding0；**depth bias 住 caster** `{-1,-1.5}` 与接收端常数偏置 `z−0.0015` 双保险 [R2]）。光源 mvp=**per-draw `compose_mvp(setup 三元组, draw.origin, draw.transform)`**（D7 全链零特例 [裁决点 a]），住 View 块 `light_view_proj`（176B，`[f32;48]`——前 128B 位序不变=三角系零回归续存）。接收端：三主 bgl **恒绑三元**（binding1 params/binding7 map/binding8 comparison sampler）——None=dummy 1×1+params-off（enabled=0 且 light_dir 载**旧 LIGHT 常数同位型** → Lambert `normalize(sp.a.rgb)` 逐位不变，R4 护栏）；fs `× mix(0.25,1.0,vis)`（vis=1 时 ×1.0 恒等）。光锥外=判亮（无级联 [不装④]）。shadow_vis 本体=WGPU-20 PCSS（本条款锁 pass/资源面）；map 边长 1024 与 shader 常量成对耦合。像素四锁：亮暗两态（PIT-5 症状级死锁）/质心反侧/地面 patch 平滑/None 无暗斑。


## WGPU-20: PCSS 三阶段（4f）
接收端 `shadow_vis` [E3D:B3 移植，单 pass]：① blocker 搜索 8 环（半径 `max(1.5, light_size·220·(1−z))` texel，`textureLoad` 直读深度——非比较路）；② 半影估计 `pen = size·(z−blocker̄)/blocker̄·2.5` clamp `[texel, 0.06]`（爆炸护栏）；③ 16-tap 泊松 `textureSampleCompareLevel` 均值。返回可见度 ∈[0,1]，fs 乘链 `×mix(0.25,1.0,vis)`（vis=1 逐位恒等=None 零回归路不变）。无遮挡者=判亮（早退③）；光锥外=判亮（无级联 [不装④]）。**单调性契约**：光源角尺寸 size↑ → 地面半影中间灰严格 >2×（近硬影 <400px 与有界 <6000px 双边锁）。

## WGPU-21: 裁切管线·mesh 族（B2）
View 块尾缀 80B 段（float 44..64：`planes: array<vec4,4>` + `clip_count` + 3 pad，总 256B）——**无新 binding/绑组/缓冲构造**（view_block per-draw 已握 origin+transform，设计红利）；前 176B 位序不变=存量构造保证续存（WGPU-19 同构）。`clipped(p)` 判据 `dot(n,q)+w<0` 任一负侧=discard（AND；零平面 dot=0 恒不触发=padding 天然无害）；`clip_count<0.5` 早退=None/EMPTY **逐位零回归** [R4 恒绑护栏形，golden 全套零重录自证]。fs/fs_textured 头部弃片；varying `@location(3) wpos` 载原始 model-local 顶点（与 REND-32 `clip_to_local` 恒等式同解：world=M·q+o ⇔ dot(Mᵀn,q)+d_m）。远相机/园区坐标零 f32 抖动（far_origin 逐位等锁在 tests/clip.rs）。

## WGPU-22: caster 同裁（B2）
`fs_shadow` 补 FsIn 入参与同 discard、`vs_shadow/vs_shadow_inst` 写 wpos；caster bgl binding0 可见性 +FRAGMENT（读 view.planes），尺寸 192→256。语义=**剖掉的投影物不得留影**（接收端被裁区清深度=判亮，方向正确）；与 E501/批 4f 影斑谓词族同制（探针实测定阈 [PIT-8]：城影基线/崩塌比 3:1/地面中性族存活对照三断言）。

## WGPU-23: 扩片族同裁（B2）
`fs_stroke`/`fs_point` 头部同判据（wpos=扩片插值 model 位=**逐像素**粒度，「裁世界不裁类型」的族一致性）。点盘跨面=半盘（几何正解非缺陷）；侧内盘逐位不动（tests 分色族断言：黄线半裁/绿盘不动/蓝盘清零）。线表/点表坐标=entity-local（REND-20 同款），换算共用 view_block 一条路。

## WGPU-24: 标签管线（S2）
`Variant::Labels` 独立 bgl（binding0 view + 11 表 storage 64B stride + 12 atlas `texture_2d<R8Unorm>` + 13 sampler **ClampToEdge**+Linear——atlas 禁 Repeat 越界采样）。`set_glyph_atlas(r8,w,h)`：len≠w·h 拒、全量替换（io-text dirty 驱动，v0 单槽）；`create_labels` 空表建期拒（族纪律）。**色=线性直传**（sRGB 咽喉在生产者面，与 point/stroke 的 CPU 副本转换分域——label 表由 io 上游给线性值）。vs_label：锚点投影后沿 `view.right/up ×(px·px_world_scale)` 展开（**屏幕恒大小**：ortho 精确路 `px_scale=2·zoom/W` 下跨 zoom 逐像素等，tests 锁 assert_eq；透视=近似随深度，E202 同谱分工）。恒顶 [裁决点 e]：`depth_write=false + CompareFunction::Always`，命令序末位天然叠放。**atlas 缺位=族 skip+warn-once**（表在场也不瞎画——WGPU-18 缺表纪律同谱）。fs_label：`cov=atlas.r`，`cov<0.02 discard`，色×cov 直出 + SRC_ALPHA 混合。

## WGPU-25: 标签受裁=锚判（S2）
`out.wpos = 锚点 l.pos.xyz`（非展开顶点）→ fs `clipped(in.wpos)` 整标同生共死——**半截标签不是标注**。判据与 WGPU-21 同一 `clipped()`（AND 世界面经 REND-33 表域为 entity-local 锚点，view_block 不涉）。labels 不入 caster（无投影面 [不装② 续]）。

## WGPU-26: 多视口分屏管线（⑤b）
`render_view_rects(passes: &[(Frame, ViewportRect)], view, w, h, fmt, policy)`：单 surface **单全幅 depth**（键=surface 尺寸，rect 不参键——视口同尺寸零重建）上 N 个 scissor 圈地 pass，各携独立 Frame（相机/px_world_scale 自治，命令与资源表共享）。**clear 语义本机定论（裁决门 2026-09-18 实测）**：wgpu30 `LoadOp::Clear` 作用**整个 attachment**（AllClear 形下后区吞前区=现行；与 Vulkan render-area 定论一致）⇒ **安全形=首 pass Clear（全幅底色，缝隙色顺带）+ 后续 pass Load**，为唯一公开形（`MultiClearPolicy::AllClear` 仅探针/教学位）。depth：**每 pass 必 Clear(1.0)+Discard**——首 pass 全幅 clear 已覆盖全区、后续 Load 会拿主视不透明地面的深度把小图整幅 occlusion 吞掉（2026-09-18 像素门现行抓得）；pass 间深度无消费者故免 Store 带宽；**scissor 圈栅格**令跨区深度互不可见（色/深写入围栏，共享深度零串扰=门③锁）。caster pre-pass **一趟**（frame0 配置；light-space 数学与主相机无关 → 双投共享 shadow map [裁决 d]，门④双区影现行锁）。**canary 纪律**：`ViewportRect::full` 或等 full 的 rect **不发 set_viewport/set_scissor 调用**（默认全架=旧路**逐字节等**，门②锁）；`render_view_format` 旧口零触。色 clear 单主=首 Frame 的 ClearColor（后续 Frame 之 ClearColor 不消费，与 find_map 律同形）。
