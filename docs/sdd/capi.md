# visiaengine-capi 行为契约（CAPI-NN）

C ABI 面条款。**位布局与编码规则正本住本档（C 头不外露 [FFI-R:CS-R1]）**。

## CAPI-01: 句柄编码与世代（slot 基 1）
引擎句柄=`u64 (slot<<32 | generation)`；**slot 自 1 起分配，slot 0 永不占用**（0=失败/无效专用哨兵）；destroy 槽回收（**released 句柄再入含 destroy 本身一律 -1**——双销毁=宿主 bug 信号不吞没）、槽位复用世代前进（stale=旧句柄进门→`VE_ERR_ARG`）；表外 slot（foreign）同 -1。全部带 ve 入口校验先行于状态机校验。实体句柄同形**不同空间**（引擎侧 slot 基 1 偏置不适用于实体：slot0gen0=0 合法；miss/越界哨兵=`UINT64_MAX`，与引擎侧 0 哨兵不对称是有意分工）。abi_version=`(major<<16)|minor`，宿主校验面恒 `>>16==1`（MAJOR 内只增承诺的机器出口）；0x00010000 初版，2026-09-15 属性三口（CAPI-10..12）追加→minor=1。

## CAPI-02: panic 栅栏（VE_ERR_PANIC 不外溢）
全部 17 extern 入口体经 `capi_guard` 单宏（=catch_unwind(AssertUnwindSafe)+TLS 写诊断+返回 -4）；grep 门：`cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))` 行首式计数==17（v1.3 形态；2026-09-15 属性面 14→17 修订）且 `pub unsafe extern` 签名零命中（安全签名+内部校验，[FFI-R:FC-5]）。栅栏后同 handle 后续入口行为不受污染。

## CAPI-03: 线程亲和与错误串协议
create 记录 owner ThreadId；**例外集={abi_version, last_error}** 外全部入口非 owner 调用→`VE_ERR_STATE`（destroy 跨线程**不销毁**）并在调用方线程 TLS 记含"thread"诊断。错误串协议：`last_error(ve)` 读**调用线程 TLS**，线程绑定、**至该线程下次错误写入前有效**；**仅返回值 <0 分支可写 TLS——任何返回 0/正值入口（含 destroy 幂等 0、on_input 未消费 0）不得触碰**（SDL 警告面：串禁作分支判据）。

## CAPI-06: attach 与窗口生命周期（I3 实装）
`attach(ve, win, display, kind)`：kind 0=x11（**display 必填**——Xlib 无 display 不可构面）1=win32（display 槽=hinstance，0=自动）2=appkit；非法 kind/win=0/必需槽空 → `VE_ERR_ARG`。**原子性**：构面/配置失败保全原 headless 目标（半途不换面，错误走 -2+last_error）。**宿主义务**：窗口与 display 必须存活至 swapchain 析构（destroy 即释放；跨重启持久化句柄被 CAPI-01 世代门拒）。resize=下帧自动重配；Outdated/Lost 帧内消化；Timeout/Occluded=跳帧不计错（Filament beginFrame 语义的 wgpu 对应物，D8 触发器①就此兑现）。管线按 surface caps 格式惰性建（Rgba8Unorm 优先，Bgra* 合法——X11 面格式多样，I3 smoke 实锤）。

## CAPI-05: 输入映射与错误归因（前段，I1 生效子集）
`VeInput.struct_size` 过小→`VE_ERR_ARG`（-1 入口参数家族；-5 专属缓冲/维度运行时量值——归因表）。kind 口径：PTR_DOWN 消费=1、其后 MOVE=orbit 消费=1、未按下的 MOVE=0（非本引擎事件不消费）、WHEEL=zoom 乘性（透视/正交共享 [E3D:B6]）、未登记 kind no-op=0。输入→相机=引擎策略非事件透传。

## CAPI-04: headless 出图字节链
`create_headless→load_*→render→readback` 全链经 C ABI：装载成功=0 且 entity_count 增长、render=0、readback 缓冲不足→-5/空指针→-1、像素回读非背景。viewport(w,h) 0 维→-5 族（w==0 归 -5 量值口径，attach kind 越界归 -1 参数口径）。pick 命中句柄 ∈ {entity_at(i)}（代际稳定），装载件的世界几何同一来源（REND-23/24 复用）。

## CAPI-09: 双面镜像一致性（web/C 常量同一性，批 7 J2′）
js 胶水面（visiaengine-wasm crate）的 `abiVersion` + 11 常量 getter 与 capi C ABI 常量逐项等价；`pick/entityAt` 在 .d.ts 编译面为 `bigint`（u64≡BigInt [FFI-R:BS-6/7]）。Rust 侧同一 `pub const` 源引用=恒等按构造（不设运行时断言）；跨语言面由 `scripts/web-mirror.mjs` 机器对账（node 载 pkg-node 产物：值对表 12 项 + d.ts bigint 编译断言 + 产物存在性），执行通道=`web-check` 任务（build→test，`#[ignore]` 显式链 T2——无产物即红，禁假绿）。

## CAPI-10: 宿主属性读（entity 键控三型，缺失≠零值）
`attr_f64/attr_str/attr_bool(entity_bits, key)`：entity=pick/entity_at 的 u64 编码，**键含 generation=ABA 免疫**（旧代句柄永不撞新代行）。缺失四径（无行/无列/异型/空格）统一「无值」：engine 层 `Option::None`、FFI 层返回 0 且 out 不写——**缺失≠零值**（CORE-11/12 的宿主投影）。拾取面=网格件：Strokes/Markers part 现行不带 entity（纯线/点 feature 不可 pick，`entity_at` 下标仍可达其属性行）——宿主禁按 pick-only 心智建模。

## CAPI-11: attr_str 缓冲契约
engine 层 `Option<&str>`（借用引擎）；FFI 层 `buf[cap]` 写 NUL 终止：命中=返回 1；cap 不足=`VE_ERR_SIZE`(-5) **零部分写，无探长子模式**（buf=NULL 不特判=裁决 C-2）；null buf/key/out=`VE_ERR_ARG`；宿主习惯约定 256B（demo/文档面）。错误串 TLS 纪律承 CAPI-03（仅 <0 分支可写）。

## CAPI-12: 属性保留与加性复载行
`load_geojson*` 成功即 move `GeoDocument` 入保留面（`geo_docs` 追加，与 items 加性语义同构）；entity→(装载序号, feature 行) mount 期反标，行=GEO-17 展平下标；**mount 失败=零发布**（doc 不保留、反标不插入，中途 spawn 随现行孤儿语义）。复载隔离：旧 entity 读不漂不乱，跨 doc 列空间独立。

## CAPI-13: 实体显隐（render/pick 过滤，枚举域不变）
`entity_set_visible(ve, entity_bits, visible)`：visible=严格 0/1（其余取值=VE_ERR_ARG）；未知位形（枚举域外/旧代际）=VE_ERR_ARG；成功=0、重复设同值幂等=0。隐藏语义三契约：**render 命令排除、pick 不命中、属性读与 entity_count/entity_at 枚举域不变**（宿主图层开关依赖索引稳定；隐藏件的旧句柄命中面=MISS）。线/点扩片表等不带 entity 位形的附件不在本口管辖（CAPI-10 域划分沿用）。

## CAPI-14: 显隐查询
`entity_visible(ve, entity_bits) -> int32`：可见=1、隐藏=0、未知位形=-1（编码含代际=ABA 免疫，与 CAPI-13 setter 同域同谱；位形 0 合法可查——禁按值判存在，存在性=本口/枚举域回执）。

## CAPI-15: 程序化加网格（VeMeshDescC 值结构，调用期拷贝）
`add_mesh(ve, *const VeMeshDescC) -> uint64`：首字段 struct_size（VeInput 族前瞻门，过小=拒→返回 0 并写 last_error）；positions/normals(可 NULL=引擎合成 +Z，Flat 族同源；长度失配=退化拒绝)/indices/base_color[4]/origin[3] **调用期读拷贝**，返回后宿主即释放合法（load_* bytes 同构）。退化输入（空 positions/空 indices/索引越界/normals 长度与 positions 不一致）=返回 0（0 非常成柄），无部分状态提交。返回码制（attr 族同谱）：0=成功写 *out_entity；退化/NULL/struct_size 门/坏 out=-1 且 **out 零部分写**。成功位形**可为 0**（slot0gen0 合法=CAPI-01 有意分工，宿主禁按 0 判有效性——存在性判据=枚举域/pick 出口）；枚举域即时 +1、render 即时生效、无属性行（attr 读=CAPI-10 缺失路）。

## CAPI-16: 实体删除（代际再入，双销毁同谱）
`remove_entity(ve, entity_bits) -> int32`：成功=0——items 摘除、属性反标表清理、显隐表清理、core `despawn` 槽位代际 +1；未知位形/旧句柄再入=VE_ERR_ARG。ABA 免疫显式化：删除→再增加同槽→**旧句柄仍死**（新实体新位形不撞）。`entity_at` 下标域压缩重排：删除后宿主必须重枚举（本条显式警示，索引不承诺稳定仅对显隐承诺，CAPI-13 分野）。

## CAPI-17: 事件推送口（set_event_callback）
`visiaengine_set_event_callback(ve, cb, user)`：cb=`void(*)(void* user, uint32_t event, uint64_t a, uint64_t b)`；cb=NULL 摘除（重复注册=替换不叠加）。事件域：`VE_EVT_LOAD_PROGRESS=1`（load_gltf/load_geojson 循环体内逐要素触发，a=done b=total，单调且终态 done==total==实体数）；`VE_EVT_LOAD_ERROR=2`（装载失败出口与返回值同刻触发，a=错误码 b=0）。触发线程=调用线程（同步语义）；异步资源管线落地后触发线程改 loader 线程、事件 id/序不变（前向兼容声明）。回调内再入 visiaengine_* = 未定义行为（禁）；panic/句柄门同 CAPI-03 族。

## CAPI-18: 点云直通装载（add_points）
`visiaengine_add_points(ve, const VePointsDesc*, uint64_t* out_entity) -> int32`：raw 数组直通（复用引擎内部 create_points→DrawPoints 路，渲染 IR 零改动）。值域分工表（C15，对 CAPI-01/15 既有行零撞位）：返回码 0=成功唯一形、非 0=既有 VE_ERR_* 谱；out_entity **仅成功时有效**、实体位形 0 合法禁当失败哨兵（CAPI-01 分工）；struct_size 前瞻门照 CAPI-15（小于所需=拒）；count=0/NULL marks/退化=**零提交**+VE_ERR_ARG（CAPI-15 同谱）。域语义：VePointMark{pos f32×3(宿主系局部坐标，origin=[0,0,0])，radius_px 屏幕像素(REND-30)，color sRGB×3(CORE-16 宿主面)}；**非有限坐标照收**——本口=宿主责任，脏数据四分类在 load 侧（IO-*，明写分工）；点数本口不设帽（唯一点数守卫在装载侧）。云=单实体单 DrawPoints（共识 5）；拾取域不含本口产物（CAPI-04 mesh-only 既成口径，本条不改写）。

## CAPI-19: 点云文件装载（load_pcl）
`visiaengine_load_pcl(ve, path, policy, uint64_t* out_entity, VePclReport* out_report) -> int32`：policy=VE_PCL_FASTFAIL(0)/VE_PCL_LENIENT(1)，越值=VE_ERR_ARG；out 双非空必需（NULL=ARG，成功唯一写点、失败零部分写——attr_str 纪律同谱）。成功=0 且 云=单实体（IO-06）；report 逐类导出（struct_size 前瞻门）。云级 meta（point_count/format/bbox 8 列）注册进属性域：**pcl 位形 attr_f64/str 可查**（geo attr_of 之后缀查，键不撞——位形编码全局唯一）。非有限/溢出列的丢弃语义归 io-points 四类（本口零翻译）；管理域（枚举/显隐/删除）=items 外与 CAPI-18 同谱。容量门在 IO-05（Err→VE_ERR_IO+错误串，超帽非半收）。path 不存在/解析失败=VE_ERR_IO；句柄门 CAPI-03 既成。

## CAPI-20: 剖面裁切双口（B2）
`visiaengine_set_clips(ve, const VeClipPlane *planes, size_t n)` / `visiaengine_get_clips(ve, VeClipPlane *buf, size_t cap)`。`VeClipPlane{nx,ny,nz,d}`=世界系数 32B Pod（法向指**保留侧**，判据 `dot(n,P)+d ≥ 0`，面上=保留）。**值域表 [C15]**：`n∈[0,4]`（MAX 与 REND-32 同源），**n=0=唯一清空形**（planes NULL+0 合法）；`NULL∧n>0`→-1；`n>4`→-1（**界检先于解引用**，越界零读）；零法向/非有限→-1+错误串（ClipSetup::new 同源门，不截断不吞）。归一化引擎侧做：读回恒单位形（(0,2,0,2)→(0,1,0,1)，w 同步除）。get **返回≥0=当前面数**（<0 专属错误谱，返回值不兼二主）；buf NULL=仅计数；`cap<面数` 截断写=写 min 返真数（n=3,cap=2→返 3 写 2）。owner 线程门=显隐口同谱；例外集不入。行为面：render 管线 fs discard（REND-32 换算律；WGPU-21/22/23 含 caster/扩片族同裁）+ **pick 命中点 keeps 谓词负侧→排除重试环**（剖开可见者必可拾；与 hidden 过滤正交叠加）。wasm 镜像=setClips(扁平 4n)/getClips→Float64 面；abi minor=6。

## CAPI-21: 标注字体装载（S2）
`visiaengine_load_font(ve, const uint8_t *data, size_t len)`：**替换式**装载/更新字体（TTF/OTF 全式字节；宿主注入=唯一源，**无默认字体**，CJK 15MB 级不可内嵌的定案面）。NULL/len=0/非字体=**-1+错误串**且**不动现存字体**（失败保旧=装载器语义，io-points FastFail 同谱）；成功=0。换字体=GlyphCache 作废重建（dirty→下帧 render 前 atlas 全量重传 [WGPU-24]）。wasm 面 `loadFont(Uint8Array)`。时序前提：CAPI-22 add_label 与 GEO-25 样式产标皆依赖已载字体。

## CAPI-22: 世界锚标签装载口（S2）
`visiaengine_add_label(ve, const VeLabelSpec{struct_size,pos[f64;3],color[sRGB 4],size_px,text=*const char NUL UTF-8}, uint64_t *out_entity)`。**成败=返回码**（0 成功写 out，位形 0 合法 [CAPI-01]——B1 add_mesh 教训的第二消费者）；**结构门**：NULL spec/out、struct_size<sizeof、NULL text、坏 UTF-8（to_str 失败）、pos 非有限、color∉[0,1]⁴、size_px≤0/非有限、空文本=**全部 -1 零提交**（domain 表先于副作用）。**时序门：字体未载=显式拒**（与 GEO-25 样式休眠成对：休眠只属样式域，数据口无休眠义）。管理域=items 外（CAPI-18 同谱：不入 entity_count/remove——v0 明账，删除路=全清重建触发制）。center 对齐（MapLibre 默认锚，pen/2 平移）；色经 `core::srgb_to_linear` 咽喉（宿主 sRGB 面 [CORE-16 咽喉六：label 表]）。wasm `addLabel(pos3,color4,text,size_px)→bigint|MISS`；hpp 薄转发。abi minor=7。
