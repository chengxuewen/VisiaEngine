# deck.gl / kepler.gl — "数据可视化图层"形态基准

**快照 2026-09-03 实测**：visgl/deck.gl 15k★（当日活跃）；npm 9.3.11（2026-08-28）；keplergl/kepler.gl 12k★（3.3.0-alpha.10，2026-09-02）。MIT。OpenJS 基金会治理。

## 画像
WebGL/WebGPU 数据可视化图层库（Uber 起源）：~20 种 GPU 图层（Scatterplot3D/Arc/HexBin/S3/TextLayer/GeoJson…），数据→GPU attribute 管线 + 轻量 shader 注入 + JSON 声明 props。**自身不做地图**：骑在 maplibre/google-maps 上，补"数据驱动 2.5D 图层"半区。kepler.gl = 其上的分析应用（React+redux UI 词典）。

## 对 VisiaEngine
- **借鉴（重）**：① 图层即 GPU 管线模块的抽象（uniform 注入 + 扩展层注册），Visia 的"空间实体/标注/IoT 动效"图层族可拿它的清单当需求文档；② **聚合/密度层家族**（hexbin/grid/clustering）= 数字孪生与 AGV 热力图的标准形态；③ picking（颜色编码 ID 拾取 + collision）交互实现；④ luma.gl 的 WebGPU 迁移路径（GL/GPU 双后端生命周期管理）= 多后端过渡工程样本；⑤ WebGPU backend 2024+ 落地经验 = 白送的 wgpu 兼容面参考。
- **规避**：① JS 胶水：数据须全量上传 GPU、无持久空间索引 → 百万级动数据帧卡顿，Visia 用原生内核正好打这个痛点（差异化论据）；② 20 个图层各带参数组合爆炸——抽象层别照抄其广度，抄语义分类；③ kepler 的 alpha 拖了数年（3.3 还在 -alpha）：分析型应用产品化成本信号。
- **现状判定**：Visia §5.2/§5.4（孪生/AGV）的功能规格书；MVP 的 2.5D 图层最小集 = GeoJson + Scatterplot + Text + Path/Arc + Heatmap。

## 来源
shields/npm（2026-09-03）；公开文档。
