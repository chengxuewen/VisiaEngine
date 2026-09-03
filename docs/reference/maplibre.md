# MapLibre（Native + GL JS）— 地图赛道活标杆

**快照 2026-09-03 实测**：maplibre-native 2.2k★（last-commit ≈09-02）；maplibre-gl-js 12k★，npm v6.7.0（2026-09-02，发布勤）；对照 mapbox-gl-js 12k★。BSD 系。fork 自 2020-11（Mapbox GL 闭源改许可证事件）——**Open Core 信任叙事的活教材**。

## 画像
- **Native**：C++ 自研渲染核（矢量瓦片管线），移动端一等（iOS Metal flag `--//:renderer=metal`，Android GLES；Vulkan *UNCERTAIN 状态未核*），桌面靠社区绑定（maplibre-native-qt 136★ 活跃——Qt 宿主嵌入的直接先例）。
- **GL JS**：浏览器标准件，Style Spec v8 = 声明式地图样式 DSL **事实标准**；CustomLayerInterface 开放 GPU 管线给第三方图层。

## 对 VisiaEngine
- **借鉴（重）**：① Style Spec v8 的 JSON 语义（layers/sources/paint/layout 分离、表达式语言、zoom 函数）——Visia "图层+样式"子系统的成熟答案已被验证过一遍，抄结构不抄包袱；② MVT 解码管线与 glyph/sprite 图集管理 = 2D 地图渲染的完整需求清单；③ GL JS custom layer = 宿主可扩展渲染的正确姿势（对齐 deck.gl 档）；④ 双核（Native/JS）不同源 = Visia "一份内核多宿主"的反面教材，差异化论点在这。
- **规避**：① 地图专才：无真 3D 场景语义（extrusion 是瓦片属性不是空间实体）——正是 whitepaper 1.1 的空档；② Native 桌面端二等公民（Qt 绑定薄、Windows 边缘）；③ 上游治理为 Meta 时代遗留的 fork 社区，roadmap 被动。
- **现状判定**：2D/2.5D 地图能力基线参照物 = maplibre（性能、样式覆盖度），MVP 验收可拿它对表。

## 来源
img.shields.io / npm registry（2026-09-03）；README/FORK.md 见 `evidence/2026-09-03-wgpu-direct` §1c（当日抓取）。
