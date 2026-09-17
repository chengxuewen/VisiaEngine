# IO-*：点云装载面（visiaengine-io-points）行为契约

> fixture 真源：`resources/data/pcl_*.ply` = io-points 测试 builder `#[ignore]` emit（texquad.glb 同谱，可再生）。
> 值域注记：本文件色=**sRGB 宿主面**（CORE-16）；坐标产物=**origin-local f32 + origin f64**（D7/IO-06）。

## IO-01: PLY ascii 头/属性解析
`parse_pcl(bytes, lenient)`：magic `ply` + 头行集（`format ascii|binary_little_endian|binary_big_endian 1.0`）；`element vertex N` 必需；属性集：`x/y/z`（float|double，缺一=整文件 Err）；`red/green/blue`（uchar→/255 归一，或 float 直传=宿主 sRGB 面）可选，缺=整云默认浅灰 [0.7,0.75,0.8]。未知**标量**属性列=列级丢弃（dropped_unsupported 计数，不丢点）；`list` 型属性/`face`/`camera` 等其余元素=元素级丢弃计数。头非法（无 ply 头/end_header 缺失/vertex 缺失）=整文件 Err 不分 lenient（文件级命运）。

## IO-02: binary_little_endian payload 解析
头属性序驱动逐点 stride（`from_le_bytes` 显式端序——禁宿主序依赖，x86 恒 LE 的「假通过」防线）；类型支持面：float(4)/double(8)/uchar(1)（含 int8/uint8/int16/uint16/int32/uint32 标量宽度归并读取）；`binary_big_endian`=整文件 UnsupportedFormat Err（v0 不提供端序翻转——出现证据再加）；payload 长度 ≠ N×stride → IO-04 截断语义接管。

## IO-03: 点云脏数据四类分型（域重定义，不继承 GEO-15）
`PclReport { dropped_non_finite, dropped_out_of_domain, dropped_unsupported, truncated_points, kept }`：
1. `dropped_non_finite`——点级：任一坐标 NaN/±Inf；
2. `dropped_out_of_domain`——点级：double 坐标超 f32 承载（±>3.4e38）；1e38/FLT_MAX 哨兵判定**预留 LAS 侧本带不启用**（PLY 面仅真溢出）；
3. `dropped_unsupported`——列/元素级（IO-01 定义），非点级；
4. `truncated_points`——文件级（IO-04）。
**重复点不成类**：多回波/高密度近重复是合法物理数据（LAS `number_of_returns` 一等语义），去重=下游策略非修复；本带零去重 API，此注记焊死防再议。FastFail=lenient=false：任一丢弃类>0 → 整文件 Err（GEO-16 双策略同谱）；Lenient 收点继续。

## IO-04: 截断双语义（本仓自钉，无 GEO-15 先例）
声明 N > 实际完整点数（ascii 残行/binary 尾字节不足一 stride）：`FastFail` → `Err(Truncated)`（GLTF-06 截断必 Err 谱系）；`Lenient` → 收**完整前缀**（残点不收半行）、`truncated_points = N - kept > 0`、报告如实。正例断言义务：程序化字节级截尾的测试必须同时钉「Lenient 前缀=完整点数」与「FastFail=Err」两向（语义自钉=测试自带定义）。

## IO-05: 容量守卫（本带唯一点数门）
`MAX_POINTS_CAP = 4_000_000`（命名常量；初值 release bench 校准后允许一次性改数：常量+本条款数字+测试域值三处同 commit）。声明点数或实收超帽 → `Err(OverCapacity { declared, cap })`，**非**截断收取（全收半吞=伪省内存）。C 口/wasm 共享此门（引擎侧装载路径唯一）。add_points（CAPI-18）不受本门——raw 直通=宿主自带账。

## IO-06: 产物数据模型（零渲染依赖，io-* 族纪律）
`PclCloud { positions: Vec<[f32;3]>(origin-local), colors: Vec<[f32;3]>(sRGB 面), radius_px: f32(整云统一，mount 消费), origin: [f64;3](bbox 中心，D7), meta: AttrSet(单行云级：point_count/format/bbox_min_{x,y,z}/bbox_max_{x,y,z} 共 8 列), report: PclReport }`。f64→f32 转换发生在装载层（先求差后转，f64 中间精度）；渲染/拾取零依赖本 crate（依赖箭头 io→core 单向）。整云=单实体单 DrawPoints（引擎侧 mount 义务，管理域=items 外 CAPI-18 同谱）。
