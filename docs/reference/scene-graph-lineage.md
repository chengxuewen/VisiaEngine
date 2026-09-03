# 场景图血统 — OSG / OpenSG / Coin / VSG / osgEarth

**快照 2026-09-03 实测**：OpenSceneGraph 3.6k★，**last-commit = 2022-12（维护崩溃标本）**；coin3d/coin 376★（昨日活跃——Inventor 血统靠 SoScenegraph/TEAM3D 续命，*许可证曾双 GPL/商业，复核*）；vsg-dev/VulkanSceneGraph 1.8k★（VSG 正统 org，作者 Robert Osfield 同一人=OSG 精神续作）；osgEarth（仓库迁移，star 未测出——活跃度 *UNCERTAIN*）；OpenSG 已迁 GitLab（GitHub 镜像失效）。

## 画像
- **OSG**：C++ 场景图+渲染抽象事实词典（Visitor/LOD/PagedLOD/ osgDB 插件/ osgText）。**教训**：核心单点维护者 → 活跃十年→骤然停摆三年。
- **VSG**：OSG 作者的 Vulkan 原生重生——薄抽象（vk 对象包装）+ 并行 record + 场景图与渲染分离。
- **osgEarth**：OSG 之上的地理引擎（地形瓦片/影像投影/高程），与 Visia 目标最像的 C++ 先例。
- **Coin**：Open Inventor 开源克隆（节点缓存/延迟求值状态机的鼻祖）。

## 对 VisiaEngine
- **借鉴（密度高）**：① **PagedLOD/ osgDB 分页模型 = 空间数据流式加载的设计词典**（LOD levels + 外部文件引用 + 帧预算加载）——Rust 侧没有等价成熟件，读 OSG 源码是捷径；② Visitor 模式对"遍历+收集渲染命令"的场景图求值（visia-render 指令生成层）；③ VSG 的 thin-over-API 验证了"渲染抽象保持薄、别过度 trait 化"；④ osgEarth 的 地形调度/投影集成 = 地理引擎需求清单第二份（对照 cesium 档）。
- **规避**：① C++ 反射/ intrusive_ptr 时代的 API 债（拷贝语义模糊、线程模型事后补丁）——Rust 原生重设计正好还债；② osgDB 插件格式失控（300+ driver 无测试矩阵）→ Visia IO 层从第一天做 feature-gate 与兼容测试矩阵；③ **维护模型**：域明星项目也会猝死 → 关键解析/调度路径保持可内化 vendoring（small-crate 策略，与 `evidence/2026-09-03-wgpu-direct` R4 一致）。
- **现状判定**：设计参考 >> 依赖候选（C++/LGPL/线程模型三不适配）。

## 来源
img.shields.io 各仓（2026-09-03）；openscenegraph.info/vsgsamples.org 公开。
