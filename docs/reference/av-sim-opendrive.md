# CARLA / esmini — ODR/OSC 可视化的直接参考

**快照 2026-09-03 实测**：CARLA 14k★（昨日活跃，MIT）；**esmini/esmini 942★**（注意 org 拼写）。

## 画像
- **CARLA**：UE4 基 AV 仿真器——OpenDRIVE **消费者**，证明需求体量大，形态（游戏引擎+重 sim）与 Visia 相反。
- **esmini**：ASAM 官方生态位的独立 C++ 工具（OpenDRIVE/OpenSCENARIO 读取、道路 mesh 生成、scenario 播放、OSI 接口，**渲染层就是 OSG**——又见 OSG）。**体量小、聚焦、可通读**（BSD-2 *复核*）。

## 对 VisiaEngine（Beta 期 ODR/OSC 插件 = 本档唯一目的）
- **借鉴（高）**：① esmini 的 **road→mesh 生成逻辑**（参考线+车道宽/超高/纵横断面→三角带，junction 连接）就是 Visia "自动生成道路"的算法需求规格，BSD 可内化参考；② OpenDRIVE XML 的解析分层（先 geometry 段再 lane 再 signal 的渐进加载）；③ scenario 播放器（OSC 事件→实体状态时间线）= 孪生时间动态模型（对表 cesium SampledProperty）在 AV 域的等价物；④ CARLA 的存在证明：客户拿 Visia 做"仿真可视化基底"时心里参照系是 CARLA 的观感。
- **规避**：① esmini 的 OSG 渲染（老）与 API 形状（C++ 单例横飞）——只读算法层；② 规范完整性陷阱（ODR 300+ 页，先支持 <20% 核心元素：planView/elevation/laneSection/link/signal，其余进企业插件付费墙候选）；③ 传感器仿真别碰（白纸上 sim 承诺的膨胀路径）。
- **现状判定**：MVP 零相关（白皮书已把 ODR/OSC 降为 Beta 插件——见 D3 修订②）；Beta 启动时本档升为设计输入，届时 esmini 源码走读 + ODR 解析 crate 调研（`路` 纯 Rust 现状未核）。

## 来源
shields（2026-09-03）；github.com/esmini/esmini、CARLA 文档公开。
