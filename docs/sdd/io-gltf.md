# visiaengine-io-gltf 行为契约（SDD）

> 条款标题行 `## GLTF-NN:` 为追溯锚点；测试挂 `// spec: GLTF-NN`。
> 矩阵约定：`world: [[f64;4];4]` **列主序**（glTF/glam 同构），平移即 `m[3][0..3]`。
> 资产纪律：fixture 为程序化生成 glb（resources/data/*.glb，本仓自产零第三方许可面）。
> v0 格式面：仅 GLB；`.gltf`+外链资源 → `UnsupportedFormat`（多文件支持后续片）。

## GLTF-01: parses_glb_fixture
`load_gltf("resources/data/tri-blue.glb")` → 1 实体，positions.len()==4、indices.len()==6，name=Some("Quad")。

## GLTF-02: node_hierarchy_transform_baked
hierarchy.glb 双节点（root translate(1,2,3) × leaf translate(10,0,0)）→ 实体 world 矩阵平移列 == (11,2,3)（f64 累乘烘焙，场景展开不保留节点树）。

## GLTF-03: missing_normals_default_or_generated
twoprim.glb 的 prim1（无 NORMAL 属性）→ normals.len()==positions.len() 且全零（着色端兜底信号，非缺字段）。

## GLTF-04: base_color_from_material
baseColorFactor 原样透传（tri-blue=[0.1,0.2,0.9,1]）；twoprim 两实体分别 [1,0,0,1]/[0,1,0,1]。

## GLTF-05: multi_primitive_counts_consistent
twoprim.glb → 2 实体（prim 拆分独立），positions/indices 计数与 fixture 构造逐项等（3/6 与 3/6）。

## GLTF-06: corrupt_file_err_not_panic
GLB 截断字节流 → `IoError::Parse`，零 panic 零 unwrap。

## GLTF-07: missing_file_io_err
不存在路径 → `IoError::NotFound{}`。

## GLTF-08: y_up_orientation_preserved
loader 零坐标系改写：hierarchy 世界平移 z==3.0 原样出（无隐式 Z-up 翻号——行业转换属 geo 片）。

## GLTF-09: primitive 模式过滤+跳过报告
`load_gltf_with_report(path)` → `(doc, LoadReport)`：mode≠Triangles 的 primitive 跳过并计 `skipped_non_triangle`（不再以假三角混入实体表——正确性修复对旧入口同步生效），POSITION 缺失/不可读/空计 `skipped_unreadable_positions`；`load_gltf` 签名不变 = 丢弃报告的 wrapper。
