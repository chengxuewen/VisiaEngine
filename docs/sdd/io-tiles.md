# IO-*（tiles 面）+ GEO-27：矢量瓦片流 Phase 0（visiaengine-io-tiles，super-band B3）

> Phase 0 范围：Rust-native 基础层（tile math / MVT 解码 / FileSource / LRU / 本地几何映射）。
> HTTP 源与 C API 面 = Phase 1（白皮书已带日期承诺 2026-Q4，B4 改词在案）。
> fixture 真源：`resources/data/synthetic_tile_z10.mvt` + `resources/data/tiles/10/0/0.mvt`
> = 全合成几何（测试旁的生成器手造 protobuf wire 字节），**零第三方许可面**。

## IO-10: MVT（Mapbox Vector Tile）解码
`decode_tile(bytes)→MvtTile`：protobuf wire 最小读取器手写（零 protobuf 依赖；D5 锁定纪律）。层→要素→keys/values→geometry commands 四级解码；error 分型：`Truncated`（buffer 截断，含空输入）/`UnsupportedVersion(v)`（spec v2.1=version 2，v1 显式拒）/`BadWire`（wire 记录畸形：varint>10B、未知 wire 型）/`BadGeometry`（命令字 id∉{1,2,7}、count=0、坐标溢出 i32）。几何命令状态机：MoveTo=1/LineTo=2/ClosePath=7，命令字=(id&7)|(count<<3)；参数=zigzag **delta**（cursor 起点为 0，MoveTo 亦 delta 语义——spec 原文）；ClosePath 隐式回环起点=**不产生坐标字**（points 数=显式点数，closed 标志独立）。values 七型映射：str/float(→Double)/double/int/uint/sint(zigzag)/bool。tags=packed varint 对（key_idx,value_idx），`tags(layer)` 惰性解析成 name→value。

## IO-11: slippy 瓦片数学
`TileId::new(z,x,y)`：域校验构造——x/y∈[0,2^z)，越界=None（wrap 是调用方策略，本层永不静默回绕）。`TileId::bbox()→(min_x,min_y,max_x,max_y)`（EPSG:3857 米）：z=0=全球全域（±半周长 20037508.34…）；slippy y=0=最北行（3857 y 北大，bbox y 反转映射）；**邻边无缝律**：相邻瓦片共享边坐标逐位相等（bbox 纯函数构造保证，geometry_map/邻接测试在锁）。`WORLD_EXTENT` 常量=2πR（R=6378137）。

## IO-12: 瓦片源抽象 + LRU
`TileSource::load(z,x,y)→Result<Vec<u8>,SourceError>`（同步门面=Phase 0 定案[裁决②]：拉模型引擎宿主驱动帧，文件源毫秒级线程零收益；trait=Phase 1 异步 HTTP 的接缝，调度器零触）。`FileSource`：目录树 `{root}/{z}/{x}/{y}.mvt`；缺失=NotFound 带路径、IO 错误带上下文。`HttpSource`：**typed stub**——每 load 返回 `Unsupported`（无静默死端，调度器可探测能力面）。`LruCache`：stdlib-only（HashMap+单调 stamp），溢出逐出最小 stamp（LRU 语义）；touch on get。ponytail 注记：O(n) 逐出扫描在瓦片量级（百）无感，万级换序字典。

## GEO-27: MVT 要素 → 本地几何映射
`GeoTile::from_layers(id, layers)`：tile-local 整数坐标（extent 单位，y-down）→ 3857 米世界系：scale=tile_size/extent、**y 翻转**（MVT y-down→3857 y-up：px_y=0=瓦片北缘=max_y）；`GeoTile.origin`=bbox 西南角 [min_x,min_y,0]（f64，D7 锚）；类型映射 POINT(1)→Point/MultiPoint、LINESTRING(2)→Line、POLYGON(3)→Poly（**Phase 0 环=独立 Poly，不辨洞**——绕序跨生产者不可靠，ponytail 注记=升级位）。attrs 七型→String 归一（样式消费面）。`GeoTile::shifted()`：D7 重基视图（world−origin=小值 local，顶点永不烘世界大数——geo crate GeoKind::shifted 同制）。E206 消费链=decode→map→tessellate 同构（Poly fan/StrokeSeg/PointMark 三渲染路）。
