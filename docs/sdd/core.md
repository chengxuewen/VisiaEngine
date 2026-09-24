# visiaengine-core 行为契约（SDD）

> 测试注释携带 `// spec: CORE-NN`；双向追溯门禁 `scripts/spec-trace.sh`。
> 坐标重基（RTC）测试留 P1 裁定后追加（architecture.md 未决点），不预写。

## CORE-01: create_get_roundtrip
`spawn()` 分配实体，`insert(id, Component)` 挂载，`get(id)` 返回同值组件；同帧内幂等。

## CORE-02: delete_invalidates_handle
`despawn(id)` 后该 id 的 `get`/`insert`/`mark_dirty` 一律返回 `CoreError::NotFound`，`is_alive` 为 false。

## CORE-03: stale_generation_rejected
槽位回收后旧 handle 携带过期代际，任何访问返回 `NotFound`（不得触到新主数据）。

## CORE-04: slot_reuse_after_delete
despawn 腾出的槽位由后续 spawn 复用：`slot` 相同、`generation` 严格 +1。

## CORE-05: iteration_yields_alive_only
`alive_ids()` 恰产出全部存活实体，无空槽、无已删实体。

## CORE-06: dirty_flag_coalesces
同一实体同帧多次 `mark_dirty` 只记一次脏（`take_dirty` 结果去重为单条）。

## CORE-07: take_dirty_clears
`take_dirty()` 产出当前脏清单并清空标记；紧接第二次取为空。

## CORE-08: f64_position_preserved
`Vec3` 全 f64：≥1e7 量级坐标与任意二进制分数往返零损耗（不丢精度、不转 f32）。

## CORE-09: component_missing_err
存活但无组件的实体 `get` 返回类型化 `CoreError::MissingComponent{ component: "Transform" }`，不 panic。

## CORE-10: insert_replaces_component
同实体重复 `insert` 同型组件为替换语义（get 恒得最后值），非多值叠加。

## CORE-11: AttrSet 三型 typed 列
`AttrSet` 提供 `add_row()->行号` + `set_f64/set_str/set_bool(row,name,v)->bool` 与 `f64/str_value/bool(row,name)->Option<T>` 读口。缺失（列不存在/异型读/行越界/空格）一律 `None` 不 panic——"属性缺席"与"值为 0/false"语义分离（样式键判定依赖）。

## CORE-12: 行对齐是存储不变式
列行数恒等于 `AttrSet::len()`；`set_*` 到越界行号隐式扩行（所有列同步 resize）；同行同名重写=覆盖。行号分配与元素↔行 1:1 由宿主维持（geo 侧见 GEO-17）。

## CORE-13: 同名首写定型
列一经创建类型锁定；后续异型 `set_*` 返回 `false` 且不改动数据（渲染/样式端可按列名稳定依赖类型）。

## CORE-14: 射线×三角形（Möller–Trumbore）
`ray_triangle(Ray{origin:Vec3,dir}, a,b,c) -> Option<t>`：CCW 正面命中（背向剔除，`ray_triangle_double` 双面口），返回距离参数 t（命中点=origin+t·dir）。背后（t≤0）、平行、出界（u/v/u+v）、退化（det≈0 零面积）一律 None 不 panic。全 f64（D7 远原点安全）。

## CORE-15: 射线×AABB 剪除口
`ray_aabb(Ray, min, max) -> bool`（slab 法）：框内起点=true；平行轴越界=false；全区间背后=false。供实体级即算剪除（bbox 不常存，[E3D:A5] 懒算语义；升级位=BVH，触发=bench 证据，CORE-12 行对齐纪律同源）。

## CORE-16: 色彩空间换算口（sRGB↔线性）
`srgb_to_linear/linear_to_srgb([f32;4]) -> [f32;4]`（IEC 61966-2-1 分段精确式：阈值 0.04045、线性段 /12.92、幂段 ((c+0.055)/1.055)^2.4；端点 0/1 逐位精确、alpha 恒直通）。契约分工：宿主/CSS 惯例值=sRGB 编码，引擎光照域=线性；本口管标量半边，纹理/帧目标由 Srgb 格式硬件半边（render-wgpu）。往返 ≤2e-7、256 级单调。

## CORE-17: 组节点创建（spawn_group）
`Scene::spawn_group() → EntityId`：分配新槽位并写入 `Component::Group`。组实体无网格、不参与渲染/拾取，仅作为树容器承载 offset 继承与显隐聚合。`alive_ids()` 返回的列表中组实体与普通实体同列（消费方按 `Component::Group` 过滤）。

## CORE-18: 父子关系设置（set_parent）
`Scene::set_parent(child, parent: Option<EntityId>) → Result<(), CoreError>`：
- parent=Some(g) 要求 g 携带 `Component::Group`（非 Group=MissingComponent 拒绝，零部分写）；
- **环检测**：从 g 向上走祖先链，若遇到 child=拒绝（NotFound）；链长 > MAX_TREE_DEPTH(=16)=拒绝（防死循环）；
- parent=None=摘离至根（Slot.parent 归 None，保留 child 自身 offset）；
- child 已死亡=NotFound；child 存活但无组件=合法（纯容器可挂任何实体，包括子 Group）。
**despawn 语义**：删除 g 后，其直接子实体 parent 自动重置 None（摘到根，不级联删除）。

## CORE-19: 父查询（get_parent）
`Scene::get_parent(id) → Result<Option<EntityId>, CoreError>`：id 死亡=NotFound；存活=Some(parent) 或 None（根）。

## CORE-20: 组偏移读写（group_offset / set_group_offset）
`Scene::set_group_offset(g, offset: [f64; 3]) → Result<(), CoreError>`：g 必须携带 Component::Group（否则 MissingComponent 拒绝）；offset 为世界米平移量，不触发子实体脏标记（渲染期实时累加，见 CORE-21）。
`Scene::group_offset(g) → Result<[f64; 3], CoreError>`：读 g 自身偏移（非累加）。

## CORE-21: 有效偏移累加（effective_offset）
`Scene::effective_offset(id) → [f64; 3]`：沿 parent 链从 id 累加各级 `Slot.offset`，至根（parent=None）或 MAX_TREE_DEPTH 截断。O(depth ≤16)。死链中间段跳过（slot_of 失败=break，不报错——容许宿主并发修改边缘）。
消费方（render 循环 / pick / is_hidden）用 `draw_origin = it.origin + scene.effective_offset(it.entity)` 代替 `it.origin`。
`Scene::is_descendant_of(d, a) → bool`：d 或 d 的某祖先 = a（d==a 返回 true，自反性）。
