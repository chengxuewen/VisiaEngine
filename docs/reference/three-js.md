# three.js — 与 VisiaEngine 同构性最高的参考物

**快照 2026-09-03**（shields/npm 实测）：115k★（昨日活跃）；npm **r185**（2026-07-01）；MIT；单核心维护者（mrdoob）+ 稳定贡献层——16 年不断代。

## 画像
"JavaScript 3D Library"——**刻意自称库不叫引擎**：最小场景图（Object3D/Scene/Mesh/Material）+ 可换 renderer（WebGLRenderer 稳态 / WebGPURenderer + TSL 着色语言迁移进行中，2026 完成度 *UNCERTAIN：官网示例前缀仍多 webgl_*，精确状态查 CHANGELOG*）+ 一切扩展（loader/controls/后处理）在 examples/jsm 外围。生态=React-Three-Fiber、model-viewer、potree（点云）、aframe……网页 3D 的事实基础设施（Google/NASA/Shopify 案例墙当日可见）。

## 对 VisiaEngine（同构性：库形态/API 极简/嵌入优先/Web 一翼——四条全对齐）
- **借鉴**：① **API 面的"最小核 + 外围"分层**：核心包只含场景图/材质/相机，40+ loader 全在外围——visia-core vs visia-io-* 的 crate 切分有 16 年存续先例；② **WebGL→WebGPU 的渐进迁移工程**：双 renderer 长期并存、同 API 面换后端——visia-render trait 的"语义源唯一+后端可并存"路线的现实注脚（且证明这条路能走十年）；TSL（把着色器逻辑写成 JS 图，编译到 WGSL/GLSL）= visia 未来材质 DSL 的形态参照；③ 发布节奏（每月 rNNN，date-based）对 SDK 版本策略友好；④ 点云查看器 potree 是 Visia 点云层的用户画像样本。
- **规避**：① 无空间索引/无 LOD 调度/无流式原语——它停在"3D 库"，正停在 Visia 要越过的线上（白皮书 1.1 对 Three.js 的批评句可引用此缺口，措辞已修正过 Cesium 句，此句仍成立）；② 全局状态与隐式生命周期（Material/Texture 手动 dispose，泄漏是用户税）——C API 层用所有权类型替他挡掉；③ 破坏性变更频繁（每 r 可删 API）——16 年靠 Web 生态容忍度，对 SDK 承诺是反面教材（C ABI 版本握手必做）。
- **现状判定**：写 visia-core API 设计文档时人手一份其 Scene API 做语感对照；"库而非引擎"的定位声明句式可直接借进白皮书 FAQ。

## 来源
img.shields.io + registry.npmjs.org（2026-09-03）；threejs.org 首页案例墙（当日）；WebGPU 迁移完成度待 CHANGELOG 复核。
