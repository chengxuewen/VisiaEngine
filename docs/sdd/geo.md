# visiaengine-geo 行为契约（SDD）

> 条款标题行 `## GEO-NN:` 为追溯锚点；测试挂 `// spec: GEO-NN`。
> fixture：`testdata/park.geojson`（本仓自产：2 多边形[其一含双洞]+1 线+1 MultiPoint+1 点=5 features，simplestyle 属性）。
> 投影语义：EPSG:3857 定义即**球面近似**（auxiliary sphere WGS84 a=6378137）；与椭球 Mercator 在 50°N 差约 8.7mm 属标准定义行为，非实现误差。datum 假定 WGS84（RFC7946 强制）。

## GEO-01: parses_feature_count
`load_geojson("testdata/park.geojson")` → 5 features，name 保序（buildingA..lamp）。

## GEO-02: webmercator_known_points
球面式正反参考：(0,0)→(0,0)；(180°,0)→x=20037508.342789244；(10°,50°)→(1113194.9079327357, 6446275.841017158)±0.01m。

## GEO-03: polygon_exterior_holes_preserved
buildingA → 外环 5 点（闭合）+ 2 洞保序保点数，环不合并。

## GEO-04: line_and_multipoint_kinds
road → Line(len 4)；grove → MultiPoint(len 2)；lamp → Point；类型不混。

## GEO-05: layer_bbox_union
park 层 bbox = 全 feature 3857 并集：x∈[261658.6,261704.2]±1m，y∈(5.65879e6,5.65891e6) 量级带（fixture 构造值）。

## GEO-06: malformed_err_not_panic
park 字节截半 → GeoError::Parse，零 panic。

## GEO-07: geometrycollection_flattened
顶层 GeometryCollection{Polygon,Point} → 2 features（Poly+Point），GC 自身不成 feature。

## GEO-08: extreme_latitude_rejected
lat=89.0 → GeoError::InvalidCoord（|φ|>85.0511° 3857 发散域）。
