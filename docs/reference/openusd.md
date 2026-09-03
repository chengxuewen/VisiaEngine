# OpenUSD — 数字孪生的场景交换标准（非引擎）

**快照 2026-09-03 实测**：PixarAnimationStudios/OpenUSD 7.5k★，last-commit 8 月。Apache-2.0。AOUSD 联盟（Pixar/ADSK/Bentley/NVIDIA）。

## 画像
场景描述与交换框架：Prim 树 + typed schema + **composition（overlays/inheritance/variants/references——非破坏性组装大型场景）** + 扩展库（usdGeom/usdPhysics/usdRender...）。Hgi = 其内部图形抽象层（又一个"薄后端抽象"工业样本，对照 QRhi/wgpu-hal）。影视/孪生/CAD（ADSK 全线、NVIDIA Omniverse）趋同中。

## 对 VisiaEngine
- **借鉴**：① **composition arcs 的思想**——数字孪生"基础场景 + 各租户覆盖层"的可视化层模型（Visia 场景图若支持引用/覆盖即对齐此语义，哪怕不读 USD）；② schema 可扩展性（强类型 prim 类型注册表）= visia-core 实体模型的可抄设计；③ usdRender 的 render settings/terminals = "场景 ↔ 渲染器"配置面分离的先例。
- **规避**：① **别链接 USD C++**（重、慢装、版本地狱）——做 **USD import/export 适配器**（读 usda 文本子集）即可，孪生客户要的是进出兼容，不是运行时 USD；② spec 广度（USD 是制片管线标准，Visia 只取 geom+xform+material 子集）。
- **现状判定**：非 MVP 相关；Visia Studio 数据交换 + 孪生企业插件（Bentley/NVIDIA 生态对接）阶段回来读。与 bim-ifc 档成对（USD 管组装/展示，IFC 管 BIM 语义）。

## 来源
shields（2026-09-03）；openusd.org/AOUSD 公开。
