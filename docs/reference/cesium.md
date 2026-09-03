# Cesium — 统一 2D/3D + 流式 + Open Core 结构范本

**快照 2026-09-03 实测**：cesium 16k★（≈09-02 活跃）；npm 1.145.0（2026-09-01，~周更）；3d-tiles 规范仓库 2.6k★（8 月）。Apache-2.0。repo 创建 2012-03 —— **14 年引擎连续投入**。

## 画像
自研 WebGL 地理 3D 引擎（球/地形/影像/3D Tiles/时间动态实体）。商业 = Cesium ion（闭源云服务：转换/托管/资产）+ 开源运行时——**与 Visia 计划同构的 Open Core**：软件栈全开源，钱在数据管线服务。Cesium for Unreal/Unity = 把自有流式渲染**插件化嵌进游戏引擎**——"不绑架宿主"的另一种解法（库进宿主，渲染走宿主管线）。

## 对 VisiaEngine
- **借鉴（重）**：① **3D Tiles = OGC 标准 + 流式 LOD（SSE 驱动）+ b3dm/pnts 演进（1.1 GLB 化）**——Visia 点云/倾斜摄影/BIM 流式直接采纳规范而非自造；② 地形/影像的 Quadtree 调度与请求合并、`EllipsoidTerrainProvider` 抽象；③ SampledProperty/时间动态模型 = 数字孪生 IoT 绑定的数据形态答案；④ 2D/2.5D/3D **SceneMode 切换**的产品化语义（morph 动画）= Visia §3.1 承诺的交互规格模板；⑤ ion 的"免费层+按量"定价骨架（商业化期参照，*数据点来自公开页面，未逐项复核*）。
- **规避**：① JS 内核 → 原生嵌入几乎不可能（正打 Visia 卖点，见 whitepaper 1.1 修正句）；② API 面历史肥厚（14 年包袱：deprecated 层叠）；③ ion 转换管线闭源 = 数据格式覆盖度的隐性锁定，Visia 若学其形须避其锁。
- **现状判定**：最完整的"空间数据可视化引擎"需求清单来源；MVP 的 GeoJSON/矢量流式对表 Cesium 同类功能行为（许可 Apache 允许读码学习）。

## 来源
shields/npm 实测（2026-09-03）；cesium README / 3d-tiles spec 公开。
