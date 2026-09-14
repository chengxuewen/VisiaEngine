# visiaengine-render 行为契约（SDD）

> 条款标题行 `## REND-NN:` 为追溯锚点；测试挂 `// spec: REND-NN`。

## REND-01: backend_trait_object_safe
`RenderBackend` 必须 object-safe：`Box<dyn RenderBackend>` 在 crate 测试中可构造（多后端分发/宿主持有的前提）。

## REND-02: stub_impl_without_wgpu
本 crate 测试模块内以纯 CPU stub 实现 trait（本 crate 无 wgpu 依赖=构造事实）——证明**契约面零后端类型**（不变式②）；stub 处理 ClearColor 帧不 panic。

## REND-03: ir_variants_exhaustive_construct
IR `DrawCommand` 全部变体可构造，`kind()` 一一可辨（v0：ClearColor / DrawMesh）。

## REND-04: viewport_roundtrip
`Viewport::new(w, h, scale_factor)` 访问器无损往返；`logical_size()` = 物理尺寸/缩放因子。

## REND-05: camera_projection_variants
`Camera::Ortho/Perspective` 构造后 `is_orthographic()` 判别正确（2D/3D 统一入口的投影侧地基；切换动画属后续片）。

## REND-06: mesh_desc_constructible
`MeshDesc{positions:&[[f32;3]], normals:&[[f32;3]], indices:&[u32]}` 纯借用构造，长度约束由文档担保（positions/normals 等长）。

## REND-07: create_mesh_returns_distinct_ids
trait 法 `create_mesh(&MeshDesc) -> Result<MeshId, BackendError>`；stub 两次调用得不同 MeshId（单调不复用）。

## REND-08: create_material_returns_distinct_ids
trait 法 `create_material([f32;4]) -> Result<MaterialId, BackendError>`；同 REND-07 语义。

## REND-09: frame_view_proj_fields_roundtrip
`Frame` 增 `view/proj: [[f32;4];4]`（列主序裸数据，契约面零矩阵库）；恒等阵往返无损。

## REND-10: camera_rig_view_matrix_lookat
`CameraRig::look_at(eye,target,up)` → view 矩阵：eye 映射到原点、target 的 view 空间 z 分量<0（右手系约定断言）。

## REND-11: persp_proj_matches_glam_std
90°FOV/aspect1/near1/far∞(大) 透视阵对照手算标准形（RH-GL 深度 0..1 变体，实现选型声明于 camera.rs 顶注）。

## REND-12: ortho_zoom_scales_halfextents
`CameraRig::ortho_frame(zoom)` 投影半宽半高随 zoom 线性（x 轴 [1,0,0,0] 列分量=1/zoom）。

## REND-13: orbit_90deg_equivalence
orbit_delta(45°)×2 与 orbit_delta(90°)×1 的 eye 位置 f64 容差 1e-6 等。

## REND-14: degenerate_near_far_rejected
near>=far → `proj_matrix` 返回 None（构造式拒绝，零 panic）。

## REND-15: switch_midpoint_continuity
`mix_rig(a, b, t)` 参数空间逐分量插值：`mix(0.5)` 与 `mix(0.5001)` 逐字段差 <1e-3（t 连续域无跳变）。

## REND-16: switch_endpoints_exact
`mix_rig(a,b,0)==a`、`mix_rig(a,b,1)==b` 逐字段精确（端点恒等）。

> v0 注：yaw 线性插值（demo 角度域 <180°）；wrap 最短弧属后续相机片（ponytail 标记于实现）。

## REND-17: frame_view_rotation_eye_split
`Frame.view` 拆为 `view_rot:[[f32;4];4]`（纯旋转）+ `eye:[f64;3]`（世界相机位）；`camera` 型判别字段保留；裸数据构造往返无损（D7 实施态）。

## REND-18: drawmesh_carries_origin
`DrawMesh +{ origin: [f64;3] }`（layer 级世界 origin，D7 ②）；transform 语义改为 **origin-local** 位姿（f64）；全部构造点编译同步。

## REND-19: rebase_identity_at_zero_origin
`rebase::compose_mvp(proj, view_rot, eye, origin, model_local)`：origin=0、model=世界矩阵时 ≡ P·V·M 旧组合（1e-4 容差）。

## REND-20: far_origin_precision_preserved
origin=(1e7,0,0) 顶点 local 0.5、相机 origin 前 10m：compose 后 clip 域相对误差 <1e-4；对照旧路（world 直 f32 cast）误差 ≥0.25m。

## REND-21: 透视屏幕射线
`screen_to_ray_persp(rig, px, py, w, h) -> Option<Ray>`：左上原点像素系（px=0→NDC −1，py=0 顶缘→+1）；张角 tx=tan(fov_y/2)·aspect、ty=tan(fov_y/2)，dir=归一(fwd+right·tx·nx+up·ty·ny)，origin=eye。视基=与 look_at 同 UP 约定解析构造；退化输入（w/h≤0、fov 非法、极点 fwd∥UP）→ None。

## REND-22: 正交屏幕射线 + D7 远场保持
`screen_to_ray_ortho`：origin=eye+right·(hw·nx)+up·(hh·ny)（hw=rig.zoom、hh=zoom·h/w，与 REND-12 同约定），dir=fwd。两口全程 f64（世界大坐标下起点/方向无 f32 步长灾难——D7 断言面）。

## REND-23: 场景拾取=最近命中+即算剪除
`pick_meshes(Ray, &[MeshCandidate{entity, positions, indices, world}]) -> Option<PickHit{entity, triangle, t, point}>`：逐候选**世界 AABB 即算**（局部 8 角变换，[E3D:A5] 懒算不常存）→ `ray_aabb` 剪除 → 逐三角 `ray_triangle` 正面命中取全局 min t。空几何/索引越界候选跳过不 panic。

## REND-24: 拾取世界合成
候选顶点 = 局部 f32 经 `world`（列主序 f64，`DrawMesh.transform` 同型）变换后再求交——缩放/平移合成命中点与 t 均落世界系（f64，D7 域）。空候选数组=None。

## REND-25: MeshDesc.uv 可选语义
`MeshDesc.uv: &'a [[f32;2]]`：空片=整 mesh 零填充 (0,0)（Flat 存量形态零改动）；非空且 ≠ len(positions) = 后端拒绝（不静默补齐，防错位采样）。

## REND-26: RenderBackend 纹理扩展默认体
`create_material_desc(&MaterialDesc)` 默认 = 转发 `create_material(base_color)`（texture/repeat/specular 丢弃=安全降级）；`upload_texture(&TextureDesc)` 默认 = 显式 Err（无纹理面后端不假装成功）。两者皆默认体=trait 向后兼容，存量 impl 零改动。

## REND-27: 实例表契约（4c）
`create_instances(&InstanceDesc{data:&[Instance]}) -> Result<InstanceId, BackendError>` 默认体=显式 Err（无实例面后端不假装成功，upload_texture 同款协议）。`Instance` = 32B Pod（`repr(C)`：offset[0..12]/height[12..16]/color[16..28]/pad[28..32]，与 WGSL `struct Instance` 逐字节同形，offset_of+bytes 双断言锁）；仅经 `Instance::new` 构造（pad 归零担保）。实例色为 RGB——alpha 恒住材质（裁决点 c）。

## REND-28: DrawInstances 变体（4c）
`DrawCommand::DrawInstances{mesh, material, instances: InstanceId, origin, transform}`——D7 origin 语义与 DrawMesh 同款（f64 远坐标层，实例 offset 为 origin-local f32）；`kind()="draw-instances"`；消费端穷举 match 同步器扩臂（compile-time 逼显式处理，新后端漏臂=编译失败）。v0 无实例级剔除/排序（全量单 draw，DrawMesh 纪律同款）。

## REND-29: Frame.px_world_scale（4de）
`Frame +{ px_world_scale: f32 }`——1 屏幕像素≙多少世界单位 @参考深度（**宿主给**：ortho 顶视=2·zoom/width 精确；透视近似=4de 不装①注记）。扩片族（Strokes/Points）宽度/半径乘子；三角系管线不读=零影响。默认构造点=1.0。

## REND-30: 扩片表与命令族（4de）
`TableId=u64` 别名（与 InstanceId 同计数域，新资源族命名）。`StrokeSeg` 48B Pod（`a[0..16] b[16..32] color[32..44] width_px[44..48]`——vec4 同宽字段消 CPU/GPU 对齐歧义 [R1]）/`PointMark` 32B Pod（pos/radius_px/color/pad）；`create_strokes`/`create_points` 默认体=显式 Err（族协议第 4/5 员）。`DrawStrokes{table,origin,transform}`/`DrawPoints{...}`：**色/宽住表不挂材质** [裁决点 a]（材质对扩片族无 base/shade 语义——光照链不参与，色直出）；D7 origin 同款；kind()="draw-strokes"/"draw-points"。

## REND-31: ShadowSetup 契约（4f）
`Frame +{ shadow: Option<ShadowSetup> }`：`None`=关闭且**逐位零回归**（后端 dummy 早退位，key位 语义入 WGPU-19）；`Some`=方向光。`ShadowSetup{proj, view_rot, eye(f64 D7 同 Frame), light_dir, size, bias}`——**构造契约 [PIT-5 升约]**：三元组只许 `CameraRig::perspective/ortho_frame` 产出（其 wgpu [0,1] 域由 REND-11/12 行为断言锁死）；手工 GL [-1,1] 矩阵=症状级死亡（全黑/全亮），像素死锁住 WGPU-19 面。`size`=光源角尺寸世界单位（0→硬 PCF）；`bias` 默认 `ShadowSetup::DEFAULT_BIAS{-1.2,-1.5}`（调参基线，P2 实测定数后两处同步）。
