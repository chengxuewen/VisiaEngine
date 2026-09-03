# GDAL / PROJ / GEOS — 地理数据三件套（链接 C 还是纯 Rust）

**快照 2026-09-03 实测**：OSGeo/gdal 6k★（昨日活跃）；OSGeo/PROJ 2k★；libgeos/geos 1.5k★（注：geos 主开发在 OSGeo git，GitHub 镜像）。许可：**GDAL MIT、PROJ MIT/Centers for OGC、GEOS LGPL-2.1**——LGPL 静态链接对 SDK 分发有义务（动态链接或规避），**GEOS 进核心包是许可证雷区**。

## 画像
- **GDAL**：栅格/矢量统一 IO（200+ driver）+ 坐标重投影管线 + WMS/WMTS 客户端 + 虚拟化（COG/GPKG/Parquet 地理云格式）——30 年的格式沼泽抽水机。
- **PROJ**：坐标引擎（EPSG 码全库、pipeline 表达式、时间相关基准）。**GEOS**：JTS 移植的拓扑运算。
- 生态位事实：PostGIS/QGIS/mapserver 乃至 Cesium 管线全压在这三件上。

## 对 VisiaEngine
- **借鉴**：① GDAL 的 dataset/driver/虚拟子系统 API 形状 = "可扩展 IO 层"的最成熟设计（Visia IO 注册表直接借鉴，抄语义不链 C）；② PROJ pipeline（+proj=pipeline 表达式）= 坐标变换的**声明式组合**设计；③ COG（Cloud Optimized GeoTIFF）+ HTTP range = 栅格流式的云端格式标准，Visia 影像层直接支持。
- **规避/决策**：① 纯 Rust 路径已体检（`evidence/2026-09-03-wgpu-direct` §2）：投影=proj 绑定（C dep，维护 OK 185★ georust/proj 0.31，2025-08）**或** proj4rs（弱许可证声明）——**没有可信纯 Rust 全量 EPSG 件**，Web Mercator+UTM 纯 Rust（geozero/wgs84 类）起步可行、全量投影终要 C 依赖；② **RK3588/Win7 交叉注意**（legacy 线）：C 依赖在每个 legacy 平台加编译/打包成本——SDK 的投影能力应 feature-gate，核心包不强链 PROJ/GEOS；③ WMS/WMTS 纯 Rust 无成熟件（geoserver 生态=服务端，客户端自写 HTTP 即可，KISS）。
- **现状判定**：格式覆盖度上 MVP 只承诺 GeoJSON/glTF/(MVT decode via open-vector-tile，见 `evidence/2026-09-03-wgpu-direct` R4 thin 风险)；GDAL 全量能力留企业插件档（Open Core 边界的一条天然线——"重型格式支持"恰是付费插件候选）。

## 来源
shields（2026-09-03）；OSGeo 官方；georust crates 元数据。
