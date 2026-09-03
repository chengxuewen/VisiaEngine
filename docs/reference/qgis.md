# QGIS — 图层语义 / 插件经济 / 双语言绑定的 UX 词典

**快照 2026-09-03 实测**：qgis/qgis 14k★，当日活跃。GPL-2.0+。QGIS.org 社区 + 大量商业用户（政府/公用事业）。

## 画像
桌面 GIS 应用（非引擎）：C++ 核心 + PyQGIS 自动绑定（sip）；QgsProject 文档模型、**图层树+图层类型族**（vector/raster/mesh/point-cloud/temporal 统一接口）、渲染器/符号系统（SVG marker、分级渲染器）、表达式引擎（过滤/标注语法）、**插件仓库**（Python 插件免费+个别商业支持=社区经济样本）、QGIS Server（FWS/WMS/WFS）、3D 视图建立在 **Qt3D** 上（→ qt-3d 档冷场链）。

## 对 VisiaEngine
- **借鉴（重，但按层取）**：① **图层类型统一接口 + 渲染器/符号系统分离**（同一 vector 层挂不同 renderer）= Visia 样式系统的语义基准，比 deck.gl 更 GIS 正统；② 表达式语言（字段过滤/标注计算）——孪生数据绑定的最小可用形态；③ temporal 控制器（时间轴→数据刷新）= 数字孪生回放的产品形态；④ **C++ 核 + Python 薄绑定**的双层 API 纪律 = Visia Rust 核 → C API → TS/Py/C# 绑定生成器的路线验证（绑定生成工具的选型参考）；⑤ 项目文件格式（.qgz XML zip，含样式/图层树）= Visia Studio 保存格式的语义清单。
- **规避**：① GPL 核 + 闭源插件的灰色边界（Visia MIT 核心 + 商业插件反而更干净，写明兼容性）；② CPU QPainter 渲染路线在海量数据下的天花板（3D 弱 = Qt3D 拖累，正是 Visia 的机会而非学习对象）；③ UI 功能堆积（每版 release notes 数百条目——功能引力失控样本，Visia 内核按 SDK 边界克制）。
- **现状判定**：不碰其代码基（GPL），但 MVP 的"用户看得懂 GIS 语义"验收清单（图层开关/属性表/标注/符号分级）源自 QGIS 习惯——目标用户的肌肉记忆就是它。

## 来源
shields（2026-09-03）；docs.qgis.org 公开。
