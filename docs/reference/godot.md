# Godot — 编辑器形态与"全免费对照组"

**快照 2026-09-03 实测**：117k★，昨日活跃；stable **4.7.2（2026-08-18）**（Wikipedia rev 1370124215 当日）。MIT；编辑器产物 28–189MB（OS 差异，官方数值）。商业层样本：**W4 Games**（创始码农创办，做控制台移植/支持服务——引擎本体全 MIT 不动摇）。

## 画像
开源全栈游戏引擎+编辑器同一代码库。渲染（当日核实）：**Vulkan 为主 + D3D12 + Metal 原生 + OpenGL，且 ANGLE 为 Win/macOS 的 GL 替换选项**——与 wgpu 的后端策略同构，两个项目独立收敛到同一设计。2D/3D 为**两套引擎可同屏混用**（Viewport 节点混合）——与 Visia「同一场景树」主张不同路：Visia 的统一内核赌注更大，差异化也更大。GDExtension = C ABI 绑定面（Rust 社区绑定成熟）；物理引擎可换（Jolt 4.6 转正）。**场景 = 文本化节点树（.tscn/.tres），运行时/编辑器共享同一序列化**；GDExtension（C ABI 绑定不重编）、headless editor 跑 CI、Remote Data Source（编辑器当远端数据的可视化壳）。

## 对 VisiaEngine（核心是 Visia Studio 的参考）
- **借鉴**：① **编辑器和运行时同一场景模型**——Visia Studio 架构的第一原则（别做"编辑器格式→运行时格式"双轨）；② 文本可 diff 的场景持久化格式；③ GDExtension 的 C ABI 插件面设计（宿主不重编）= Visia 扩展系统的成熟模板；④ headless 模式 = Studio 可自动化（批量出图/测试）。⑤ 治理：全 MIT 无 Open Core = **对照组**——Visia 决定"编辑器收费"边界时，Godot 证明社区全免费产品存在（以及它为何对游戏成立、对垂直 GIS 工具未必成立）。
- **规避**：① 游戏核的包袱（物理/动画/音频/导航全在 core）——Visia 内核按 whitepaper"数据呈现"窄定位，警惕功能引力；② 4.x 渲染重写带来的迁移痛（3→4 兼容断层）= 大版本策略反面教材（Rust 库可用 feature 隔离缓解）；③ 编辑器 UX 复杂度（节点树对非游戏用户过重——Visia Studio 面向 GIS 人员，交互语言学 QGIS 不学 Godot）。

## 来源
shields 实测（2026-09-03）；docs.godotengine.org 公开。
