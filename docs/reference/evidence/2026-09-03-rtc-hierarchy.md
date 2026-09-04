# P1 证据：RTC 粒度的业界收敛（D7 输入）

**日期** 2026-09-03 | **取证** 本会话 web/源码实拉 + 参考项目库画像

| 系统 | 一手证据 | 机制 | 归属层 |
|---|---|---|---|
| CesiumJS | `packages/engine/Source/Shaders/Builtin/Functions/translateRelativeToEye.glsl`（raw 全文实拉） | 全局绝对坐标拆 high/low 2×f32（EncodedCartesian3）→ shader 平移相对 eye；引 AGI《Precisions, Precisions》 | viewer-origin rebase（存储侧背锅） |
| 3D Tiles | 1.1 规范 tile.transform 4×4 column-major（广知条款） | content 单位/局部 + tile 级 origin | 分层 origin |
| MapLibre GL | 瓦片内 0-8192 extent + 每 tile `u_matrix` | tile-local 存储 + uniform 相对平移 | 分层 origin |
| deck.gl | 官方高精度策略：CPU 公共 origin 相减 + offset 经 uniform（画像+文档站） | layer 级 origin | 分层 origin |
| UE5 | LWC 文档 + 5.0 release notes 方向：per-component origin rebasing **废弃**，全局 double+绘制期 rebase | 过度特化 origin 层的反例（维护性） | 分层 origin（收敛后） |

**四方案对比结论**（甲=分层 origin 胜出）：乙 high/low 仅补"必须存全局"的包袱；丙 shader f64 被 WebGPU 禁杀 Web 路；丁每帧重写顶点 VBO churn 杀大模型。甲=前三者之长：f32 上传面天然 web 兼容、origin 级=实体级（glTF 模型/tile）语义自然、rebase 决策收在 1 个 uniform 路径。
**精度预算**：local≤64km → f32 步长 ≤7.6mm；|origin−eye|≤100km → 同量级。地图级亚毫米远景需求=tile 细分粒度问题（io-geo 自决），非引擎机制问题。
