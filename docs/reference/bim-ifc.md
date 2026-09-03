# BIM/IFC — 显示层边界与许可证雷区

**快照 2026-09-03 实测**：IfcOpenShell 2.7k★（昨日活跃，**LGPL**）；ThatOpen/web-ifc 1k★（上周五，MPL *复核*）；FreeCAD 33k★；OCCT 2.8k★（**LGPL-2.1+例外**）。Bonsai（前 BlenderBIM）star 未测（repo 迁移中）*UNCERTAIN*。

## 画像
IFC = BIM 数据交换标准（buildingSMART，STEP 物理格式）。IfcOpenShell = 解析几何内核事实标准（C++ 核+Python API，BlenderBIM/Bonsai 建其上）；web-ifc = WASM 解析器（Web 端 BIM 查看器底）；OCCT = B-rep CAD 几何内核（FreeCAD）——**IFC 几何（ CSG/B-rep 混合）→ 网格化(tessellation) 才是可视化的正确落点**。

## 对 VisiaEngine（"高保真轻量化 BIM 展示"承诺的边界文件）
- **借鉴**：① **显示层立场**：IFC 语义树（IfcProject→Site→Building→Storey→Element）+ 预网格化几何 = "剖切/测量/标注"所需的最小数据面（测量=mesh 射线拾取；语义导航=树过滤）——**别进参数化几何重放（那是 OCCT 的 20 年）**；② web-ifc 的 wasm 流式策略（按需 element 解析）= 大模型轻量化的 Web 侧先例；③ IfcOpenShell 的 geom 处理层（要素→triangulated item）接口形状。
- **规避（许可证）**：IfcOpenShell **LGPL** → Visia 静态打包 SDK 有义务边界；**正确姿势 = IFC 解析进可选插件 crate（dlopen/动态链接隔离）而非核心**，与 gdal-proj 档同一原则：copyleft 件全部 feature-gate 到企业/扩展层（白送 Open Core 付费插件候选边界）。
- **现状判定**：MVP 只承诺 glTF（BIM 工具导出 glTF 的路径存在且无许可证问题——引导用户走这条），IFC 原生支持列 Beta+ 且按上面边界设计。点云另见 `evidence/2026-09-03-wgpu-direct` R1（las 有维护、pcd 死、draco-core 新且薄）。

## 来源
shields（2026-09-03）；docs.buildingsmart.org/ifcopenshell/readthedocs 公开。
