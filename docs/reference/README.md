# VisiaEngine 参考项目库

**数据快照：2026-09-03 实测**（GitHub stars/last-commit 经 shields 端点、版本经 crates.io/npm registry；定性描述来自模型知识与当日已验证备忘录，标 *UNCERTAIN* 处使用前须复核）。
用途：渲染后端终裁（D4）、白皮书 §4、MVP 架构设计的证据底座。**每篇只保留"对 VisiaEngine 有用"的部分，不是百科。**

## 索引

| 文档 | 对象 | 一句话 | 相关度 |
|------|------|--------|:-----:|
| [wgpu-rust-gpu-stack.md](wgpu-rust-gpu-stack.md) | wgpu/linebender 系/glow | 主依赖候选的生态体检 | ★★★ |
| [angle-dawn.md](angle-dawn.md) | ANGLE, Dawn | legacy 平台翻译层的工业底 | ★★★ |
| [bevy.md](bevy.md) | Bevy 0.19 | 默认后端候选 A（调研备忘录级） | ★★★ |
| [maplibre.md](maplibre.md) | MapLibre Native+GL JS | 同赛道（地图）活标杆 + 双许可教训 | ★★★ |
| [cesium.md](cesium.md) | CesiumJS/3D Tiles | 统一 2D/3D + 流式 + Open Core 结构范本 | ★★★ |
| [deck-gl.md](deck-gl.md) | deck.gl/kepler.gl | "数据可视化图层"形态基准（2.5D 半边天） | ★★★ |
| [scene-graph-lineage.md](scene-graph-lineage.md) | OSG/OpenSG/Coin/VSG/osgEarth | 场景图流派词典 + 维护崩溃标本 | ★★☆ |
| [godot.md](godot.md) | Godot 4.7 | 编辑器架构/后端策略同构验证/「全免费对照组」 | ★★☆ |
| [unreal.md](unreal.md) | UE 5.8/UE6 | Nanite 几何流送启示 + 许可信任教材 | ★★☆ |
| [unity.md](unity.md) | Unity 6 | 定价崩塌样本 + 管线分叉禁令来源 | ★☆☆ |
| [three-js.md](three-js.md) | three.js r185 | 与 Visia 同构性最高的库形态先例 | ★★★ |
| [qt-3d.md](qt-3d.md) | Qt3D/Quick3D/QRhi | 官方厂做域 3D 的冷场实证 + RHI 多后端工程 | ★★☆ |
| [flutter-embedder.md](flutter-embedder.md) | Flutter embedder | C API 宿主嵌入的黄金标准 | ★★☆ |
| [gdal-proj-geos.md](gdal-proj-geos.md) | GDAL/PROJ/GEOS | 地理数据三件套：链接 C 还是纯 Rust | ★★☆ |
| [qgis.md](qgis.md) | QGIS | 图层树/插件生态/双语言绑定 UX 词典 | ★★☆ |
| [openusd.md](openusd.md) | OpenUSD | 数字孪生场景组织与交换标准 | ★☆☆ |
| [av-sim-opendrive.md](av-sim-opendrive.md) | CARLA/esmini | ODR/OSC 插件的直接参考实现 | ★☆☆ |
| [bim-ifc.md](bim-ifc.md) | IfcOpenShell/web-ifc/OCCT | BIM 线：显示层边界与许可证雷区 | ★☆☆ |

## 证据备忘录（evidence/，日期快照 **永不更新**；摘要档与 D4 的引用链落点，勿删勿改名）

- [2026-09-03-wgpu-direct.md](evidence/2026-09-03-wgpu-direct.md) — wgpu 自研路线证据备忘录（积木/先例/人月，已 6/6 抽查通过）
- [2026-09-03-bevy-embed.md](evidence/2026-09-03-bevy-embed.md) — Bevy 嵌入式可行性全证据（已 2/2 抽查通过）
- [2026-09-03-angle-integration.md](evidence/2026-09-03-angle-integration.md) — ANGLE 集成全证据（commit 级 + Qt6 证伪 + XP 判词）
- [legacy-platforms.md](legacy-platforms.md) — 老旧/嵌入式判定矩阵（主会话合成：wgpu 原生 GL 纠偏 / Rust Win10 底线 / G610=Valhall→Panvk）

## 写作纪律

新增项目档：模板 = 画像(谱系/治理/资金)/架构要点(只写相关)/强弱/借鉴与规避/来源+快照日期。数字必须当日实测；许可证影响 Open Core 边界的一律标粗。
