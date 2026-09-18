# visiaengine-geo 行为契约（SDD）

> 条款标题行 `## GEO-NN:` 为追溯锚点；测试挂 `// spec: GEO-NN`。
> fixture：`resources/data/park.geojson`（本仓自产：2 多边形[其一含双洞]+1 线+1 MultiPoint+1 点=5 features，simplestyle 属性）。
> 投影语义：EPSG:3857 定义即**球面近似**（auxiliary sphere WGS84 a=6378137）；与椭球 Mercator 在 50°N 差约 8.7mm 属标准定义行为，非实现误差。datum 假定 WGS84（RFC7946 强制）。

## GEO-01: parses_feature_count
`load_geojson("resources/data/park.geojson")` → 5 features，name 保序（buildingA..lamp）。

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

## GEO-09: fill_area_matches_polygon
单位方环 [-1,1]² 细分 → 三角形总面积 ≈4 ±5%，重心 (0,0) 落于某三角形内（含边容差）。

## GEO-10: hole_interior_uncovered
方环含中心 0.4² 洞 → 洞内采样点 (0,0) 不在任何输出三角形内部。

## GEO-11: stroke_expands_to_width
(0,0)-(10,0) 线、显式 stroke_width_m=2（默认 3 不断言具体值）→ 输出 y 跨度 2±0.1、x 跨度 10±0.1（butt 端）。

## GEO-12: simplestyle_six_keys_parsed
六键：fill / fill-opacity / stroke / stroke-width / marker-color / marker-radius；色支持 #rrggbb 与 rgb(r,g,b)（解析产物=**sRGB 域原值**，线性化住在后端上传咽喉 [CORE-16]——geo 域零换算）。park 断言：A fill=[1,0,0,1] α=0.8、road stroke=#808080、lamp marker=#228B22。

## GEO-13: missing_props_default_style
无/未知键（如 "color"）→ 默认样式（fill 蓝 [0,0.45,1,1]，stroke 白），不报错。

## GEO-14: tessellate_input_is_origin_local
D7 纪律接口化：`tessellate` 输入为**已减 origin 的 local 坐标**（引擎提供 `GeoKind::shifted` 作 world→local 原语，消费方不得手改顶点）；(1e7,0) 偏移方环 local 化细分 ≡ 原点方环细分（对应顶点 <1e-3 米）。

## GEO-15: 宽松加载=分型丢弃计数
`parse_geojson_lenient(bytes)` → `(doc, LoadReport)`：脏几何按 out_of_bounds/non_finite/unsupported/null_geometry/nested_collection 五类在产生点计数（typed GeoError 变体映射，非 reason 字符串反解）；`total_dropped()`=五类和；文档级错误（语法/IO）仍整文件 Err。`parse_geojson_with(bytes, RepairPolicy)` 为显式策略入口。

## GEO-16: 旧入口 FastFail 语义冻结
`parse_geojson` = `RepairPolicy::FastFail`：首件脏几何即整文档 Err（GEO-08 拒绝面原样保持）；`geometry:null` feature 维持旧静默跳过（不计数）；纯干净输入下双入口 features 结果一致且报告全零。

## GEO-17: feature 属性入列，行与展平件对齐
解析成功的每个 feature 件（GC 展平后按件计）占 `AttrSet` 一行，`doc.attrs().len() == features().len()` 恒成立；`geometry:null`/被丢弃脏件不占行（先于 add_row 失败）。JSON 标量（number/string/bool）入对应类型列；object/array/null 值跳过不入列。GC 子件行复制 feature 级 name/props。宿主查询口 `doc.attr_f64/attr_str/attr_bool(feature_idx, name)`。

## GEO-18: 样式经 typed 列读取
simplestyle 六键判定经 `style_from_attrs(&AttrSet, row)`——`style.rs` 模块零 serde_json 依赖（解析边界固定 lib.rs，机器门禁 `pixi run gate-style`）；六键输出与列化前语义逐键一致（回归网=既有 GEO-11/12/13 断言）。`parse_style(json)` 便利口经一次性 AttrSet 走同一实现。

## GEO-19: v8 paint 别名回退（D9 裁决实施）
每个样式键在 simplestyle 主键缺席时接受 MapLibre v8 静态 paint 别名回退：fill→fill-color、stroke→line-color、stroke-width→line-width、marker-color→circle-color、marker-radius→circle-radius（fill-opacity 同名直读）。**主键优先于别名**（并存时主键胜，规则定死）。值格式复用 GEO-12 色解析与 GEO-13 数值语义；v8 表达式/stops/data-driven 明确不在本条覆盖面（D9 边界）。

## GEO-20: 标量→色带扩展键（解析期物化）
`visia:color-column`（属性列名）+ `visia:color-lo/hi`（数值区间）+ `visia:color-low/high`（端色，GEO-12 格式）齐备且区间有效时：列值线性插值 t=clamp((v−lo)/(hi−lo)) 物化写入 `style.fill`/`style.marker_color`（渲染零改动——per-feature 常量色形态）。任一要素缺失/`hi<=lo`/引用列不存在或非数值 → 静默禁用（六键结果保持）。`visia:` 前缀为 D9 边界外自有扩展命名空间；逐顶点 GPU LUT（DEM/点云场）不在本条，留 backlog。

## GEO-21: 折线平面距离
`planar_distance(&[Vec3]) -> f64`：3857 米平面折线长（相邻有限点段和）。非有限点跳过不成段（大坐标 NaN 传播防御）；空/单点=0。**语义冻结**：墨卡托平面距离随纬度含 cos φ 畸变——大地学语义属"坐标系完善"（Alpha roadmap），本条不充胖子。

## GEO-22: 环平面面积
`ring_area(&[Vec3]) -> f64`：shoelace 绝对值、方向无关（GeoJSON 顺/逆时针双俗同值）、闭合重复点与否同值。顶点 <3 或任一非有限点 → 0（不猜测残缺多边形）。

## GEO-23: 解析域全输入无 panic（属性面）
`parse_geojson`（FastFail）与 `parse_geojson_lenient` 对**任意字节流与任意 f64 坐标域**（含 NaN/inf/越界/非法 JSON 结构）零 panic——Err/丢弃为合法出口，panic 为契约违反。proptest 512 例×3 性质覆盖（随机流/全 f64 坐标/干净域双策略一致性：无脏件时 total_dropped==0 且件数同）。

## GEO-24: GeoPart 输出切换（4de）
`tessellate → Vec<GeoPart>`：`{Fill(TessPart), Strokes(Vec<LineStrip>), Markers(Vec<Marker>)}`——线/点**不再 CPU 扩条带/方块**（`tess_stroke`/`quad` 退役，`PartKind` 缩至 `{Fill}`）。中立类型（geo 零 render-IR 依赖，分层不变式；扩片转换住消费者=三处同形注记 pending 4de 后提取）。`LineStrip{pts(相邻点=段，环输出闭合首尾同点), color:[f32;3], width_px}` / `Marker{pos, color, radius_px}`——**px 单位语义 [裁决点 a 纠偏]**：MapLibre stroke-width/circle-radius 本义像素，旧 `_m` 字段系误释；样式键名（含 D9 别名）不动，`stroke_width_px` 默认 1.5（原 3.0 世界单位随单位重释归正）、`radius_px` 默认 4.0，stroke-width 值 floor 0.5px。`stroke_width_px==0` → 不出 Strokes part（关描边语义保真）。

## GEO-25: 文本样式三键（S2/B2后）
`StyleRecord +{ text_field: Option<Box<str>>, text_size_px(默认 14), text_color(宿主 sRGB 面=GEO-12 同制，线性化住消费端咽喉 CORE-16) }`（**Copy→Clone**，零涟漪实测）。解析（style_from_attrs，v8 layout 域正本无 simplestyle 对应=主键即正本）：`text-field` 值花括号剥壳 `"{prop}"`→列名；**裸串=常量文本**；分辨律住消费端（CAPI-22/引擎挂载）：**属性列存在（`attrs.names()` 判）→列语义**（该列存在而行值缺=**skip 不落字面量**，杜绝 `"{name}"` 漏成文本）；列不存在→字面量。`text-size` clamp [1,256]；`text-color` 复用 parse_color。输出形：装载端产 `LabelMark` 合**一 doc 一表一 DrawLabels**（origin=层中心，锚=bbox 中心+**center 对齐**（MapLibre 默认），z=0.1 恒顶层）。无字体装载=整段零收集零输出不报错（休眠形，CAPI-22 时序门配套）。
