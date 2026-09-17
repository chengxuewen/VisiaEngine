# 四家对标差距审计 · 团队调研合成（2026-09-17）

> 方法：team-mode 四路并行解剖（`.refinfo/three.js` / `.refinfo/Easy3D` / `.refinfo/bevy` + 例子/文档生态四家横比），lead 交叉验证合成。
> 全部结论双侧文件证据；S 级声称经七项 grep 复验（色彩管理/层级/文本/capi 回调/blend/动画/异步=全 0 命中属实）。
> 原始四卷全文存于当次会话（分析员：threejs-gap / easy3d-gap / bevy-gap / examples-gap）；本档为合成正本。
> ⚠ examples 卷三处不实已在合成时修正：E901=twin_city（非 wasm demo）；E403 为在库号（非空号）；E202 是 GeoJSON 点标记域（非点云数据族）。

## 一句话（人话版）

**工程纪律与 SDK 骨架我们领先，画面正确性（色彩）、文字的地图半边、以及"宿主体验 API"（事件推送/异步加载/场景层级）是三大真空**。三家参照物各照出一类问题：three.js 照出"渲染器还幼稚"（色彩/材质/PBR/后期），bevy 照出"宿主还原始"（事件/资源/序列化），Easy3D 照出"GIS 承诺未兑现"（点云/文字/半透明/剖面——其中地形/VSM/GGX 三项其实 Easy3D 自己也没有，属自主立项账，勿挂对标名目）。

## 共识矩阵（≥2 卷独立同判才入表）

### S 级（战略缺口：产品门面或定位承诺）

| # | 缺口 | 提出卷 | 证据锚点 | 本仓现状 |
|---|------|--------|---------|---------|
| S1 | **色彩管理全链**（sRGB→线性→tone map→输出） | threejs | `.refinfo/three.js/src/math/ColorManagement.js` + ToneMapping 族 | 全灰度=线性域当 sRGB 画；golden 在锁错值，越晚改重录越多。成本 M |
| S2 | **文字/标注面**（地图半边） | threejs/easy3d/examples 三卷同判 | text_renderer(T309)/Sprite/23 文字例 | 零 glyph 命中；E202 连一个 POI 标签都画不出。成本 L |
| S3 | **事件推送半边**（loading 态对宿主黑箱） | bevy/threejs | EventDispatcher；bevy 全链 loadable→loaded(AddAsset)→LoadFailed | capi 零回调；on_ready 是拉询形，加载中间态无处安放。成本 S-M，**纯补口不动语义** |
| S4 | **资源管线**（异步/取消/缓存/句柄失效/卸载） | bevy/threejs | asset/LoadState + Guard 防句柄复用 + LoaderCycle 依赖图；LoadingManager+AbortSignal | 同步 load 阻塞宿主线程；GLB-only；无进度/取消。成本 L |
| S5 | **点云数据族**（白皮书一等公民未兑现） | easy3d（含旧表拆账修正） | point_cloud_io_{ply,las,...}+kdtree+normals | 演示层销过账、数据族层从未销：无 IO/类型/法向/空间索引。成本 中-大 |

### A 级（功能债：下阶段主线）

| # | 缺口 | 卷 | 要点 | 成本 |
|---|------|-----|------|------|
| A1 | 场景层级 parent | threejs/easy3d | 孪生"设备→部件"、图层组变换/组显隐需父子链；两级轻量形即可（parent: EntityId，不破 C ABI 扁平句柄面） | M |
| A2 | 真 PBR GGX + glTF 通道 | threejs/bevy/easy3d | 现 4 字段 MaterialDesc；normal/emissive/occlusion/metallic 全弃（装载即哑数据）；Alpha 档在册，通道逐个点亮 | M |
| A3 | 半透明排序（+DDP） | easy3d | blend:None 唯一路实证；白模半透+图层叠加双刚需；先 back-to-front 排序 blend，DDP 升级位 | 中 |
| A4 | 剖面裁切 | easy3d | clip_plane+T307；**已排队 B2 候令**；uniform 平面+fs discard 最短路 | 小-中 |
| A5 | 纹理解码（PNG/JPEG）+点云 PLY 口 | threejs | upload_texture 只吃裸字节；io-gltf 外部图像 URI 不解析（GLB-only 根因半边） | M |
| A6 | 属性推送半边（attr_changed/dirty） | bevy | 现全量重查（E810 路）；变化通知缺 | S |
| A7 | 拾取半边（hover 推送） | bevy | 轮询可行；E402 已证 CPU 射线路 | S |
| A8 | 动画播放器（最小两轨） | threejs/bevy | glTF 动画轨在 io-gltf 直接丢；孪生演示/AV 仿真的时间维=零；只采变换轨+可见性轨（句柄定向，PropertyBinding 反射形在 C ABI 不成立），蒙皮不做 | L |
| A9 | 投影/CRS 坐标系 | 白皮书/geo 域 | web_mercator 单投影；WMS/WMTS 承诺同族；地理定位刚需，**脱对标自主立项** | 中-大 |
| A10 | 场景序列化（轻量 IR：存/恢复/部分更新） | bevy | 宿主状态持久化无口；格式走 glTF 扩展/JSON 自主设计，**不抄 .scene RON** | 中 |
| A11 | LOD 自动切换 | bevy/threejs/易3D 三家 | 有实体级 Instance LOD，无距离切换调度；与瓦片流式同题 | 中 |

### B 级（补强/体验，择机）

几何原语公开口（box/sphere/cylinder 进 capi，孪生造体刚需，成本 S）· BVH 空间索引（触发=bench 证据，picking.rs 注记在册）· 相机阻尼+关键帧漫游（mix_rig 原语已在，残余=编排）· SSAO/EDL（4g backlog；EDL 触发=S5 落地）· rustdoc 自动生成（**成本最低收益立见**——bevy 438 例+docs.rs 全量、three 658 页 vs 本仓 0 页自动参考）· gizmo/碰撞（1.0 编辑器档）· 多视口（随站点轮）· llms.txt（成本极低）· 迁移指南（GA 前记票）。

### C 级（明确不做，防对标焦虑）

ECS 全家桶（拉模型+push patch 是 SDK 正确形，bevy 自证 ECS 不跑在库线程）· 渲染图/TSL 式 DSL（固定管线+变体键+默认拒，2 个 wgsl 够用；唯一前瞻位=样式若需运行时表达式编译再评）· XR（宿主持有）· hot-reload（桌面档）· WebGPU/GL 双后端税（wgpu 原生已支付，D4 同判）· 30-pass 后期框架（挂点枚举位替代）· 蒙皮动画 · KO/SM 私有格式 · GaussianSplat/Water/Sky 网页特效 · 脚本游戏层。

### 对标失效声明（账目卫生，easy3d 卷方法论）

地形/TIN/栅格、VSM/CSSM、GGX 三项 **Easy3D 无证据**（grep 零命中）→ 移入白皮书自主立项轨，严禁挂"Easy3D 借鉴"名目（PIT-20 幽灵 API 纪律的反向适用）。bevy 卷同型修正一处：bevy 并非无视觉回归——Pixel Eagle 截图服务 + 帧事件 RON + 438 例单源元数据，本仓 golden 优势成立但措辞从"独家"收窄为"门禁族独家"。

## 排序建议（裁决点，逐项过）

- **D-1 下一带选谁**：现队列 B2=剖面裁切（A4 已排队）vs 插队候选=点云数据族（S5）/色彩管理（S1）。lead 推荐：**S1 色彩管理先行半拍**——理由：golden 重录成本随每轮单调上涨，且它是 S 级里唯一不动 API 面的纯渲染轮；剖面带维持排队。
- **D-2 事件推送口（S3）**：可与任一带搭车（CAPI minor 版本，纯加法）；建议与色彩轮或剖面轮同批 RED→GREEN。
- **D-3 文字面（S2）**：独立计划轮（字体图集选型先行），不塞进现有队列。
- **D-4 资源管线+异步（S4）**：大工程，建议 Alpha 档单独立项（与 MVT/瓦片流式=同一张设计图）。
- **D-5 点云族（S5）**：插队 or 排队 = 用户商业节奏裁决（白皮书承诺已欠账，easy3d 卷主张插队）。
- **D-6 低成本立见项打包**：rustdoc + llms.txt + 几何原语口 + 属性脏通知——一个"文档与糖"小带可全收。

## 反向优势总账（四卷合并去重，勿退坡清单）

1. D7 远坐标 f64 重基（three=纯 f32 大坐标淘汰级；bevy 也盯了 projection_zoom 同款但无远原点纪律）2. **SDD 115 条↔测试双向锁**=真独家（三家全 0）3. gate-docs/gate-abi 四方一致门禁族（bevy 单源元数据最接近但仍无机器一致校验）4. `--frames`+golden 三自证=例子即 CI（bevy 例数 438 碾压但仅部分自动化；Easy3D 从不跑；本仓 19 条全执行）5. 多语言例子矩阵 5 路（各家仅单语言）6. 纯拉模型 render 时机宿主自决+零主循环绑架 7. 无 aliasing/无 GC/无 data race 构造保证+162 测试秒级 8. 双许可 vs GPL 传染（Easy3D）/纯 JS 运行时（three）——车机/HMI 出局对手 9. 样式 spec 一等公民（D9 v8 子集）10. 修复策略装载 LoadReport（三家脏数据静默进门）11. placed() 位形单源构造锁（交互正确性进机器门禁文化）。

**收口**：以约 1/30 的复杂度，换来无 aliasing、无 panic 穿越、可 FFI 的确定性——这是三家的架构税，我们没交。对标结论=**补 S 级三件（色彩/文字/事件推送），守全部 11 项独门，拒 C 级焦虑**。
