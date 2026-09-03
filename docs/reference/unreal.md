# Unreal Engine — 封闭巨兽的镜像教训

**快照 2026-09-03**（Wikipedia rev 1372756723 + shields 当日）：stable **5.8**；**UE6 于 2026-05-24 官宣**，EA "Late 2027-ish"、GA 再 +12–18 月；源码 source-available 非公开 repo（EpicGames/UnrealEngine 需登录关联，公开 star 不可得——形态本身即数据）；许可 = 收入 >$1M 收 5% royalty（EGS 独占豁免）；2024 游戏市场份额 28%（Unity 50%），按销量计 31% 为最大商业引擎。

## 画像
28 年五代的影视/游戏霸主。架构标签：C++ + UObject 反射/GC + Blueprint 可视化脚本 + **Verse**（Haskell 系函数-逻辑语言，Simon Peyton Jones 参与，2023 上线）+ Nanite（虚拟化几何：任意精度网格→自动 LOD→GPU 驱动 cluster 流送）+ Lumen（软硬混合 GI）+ World Partition（大地图流式）。**UE6 官宣：Blueprints/Actors 将被 Verse + Scene Graph 取代**（早期版本共存）。

## 强弱
强：渲染技术天花板（Nanite/Lumen 无开源对位）；影视/汽车/建筑非游戏渗透（StageCraft、Rivian 车机、300+ 虚拟摄影棚）。弱：体积（编辑器 GB 级——**SDK 形态的反面极端**）；许可是租约不是产权（Epic 随时改规则——2024 runtime fee 风波→2025 回归，与 HashiCorp 事件同列 Open Core 信任教材）；嵌入第三方应用=不可能任务。

## 对 VisiaEngine
- **借鉴**：① **Nanite 是 BIM/倾斜摄影海量几何的终局答案**——不必抄实现（人年级），但 visia-render 的渲染指令 IR 要**为 cluster 化几何流送留扩展位**（meshlet/indirect draw 语义在 wgpu 可用，见 `_memo-wgpu-direct`），别让 MVP 架构封死这条路；② UE6 的"场景模型换代成本"警示：**场景图语义是引擎的地基，一次定对**（Actor→SceneGraph 迁移拖了三代）；③ World Partition 的大世界流式分区设计 = 孪生城市尺度的参照。
- **规避**：① 什么都别依赖：source-available 的"可看不可用"边界完全随 Epic 心情；② 反射/GC 全宇宙（UObject）绑死引擎生命周期——正打 Visia"无 GC/可嵌入"两条信条。
- **现状判定**：商业对照组（royalty vs subscription vs Open Core 三形态的最后一块拼图）；技术侧只读设计文档与 GDC talk，不进决策依赖。

## 来源
en.wikipedia.org/wiki/Unreal_Engine（2026-09-03）；UE6 细节引文含 The Verge/Lex Fridman/State of Unreal 2026 转述。
