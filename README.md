# VisiaEngine（维视引擎）

> **多维空间可视化引擎** — 统一的 2D / 2.5D / 3D 渲染管线，为"看见数据的本质"而生。

**状态：白皮书 v0.1.0 已定稿方向，工程未动工（无源码）。** 完整定位见 [docs/whitepaper.md](docs/whitepaper.md)。

## 它是什么

一个开源、轻量、可嵌入的空间可视化**引擎内核**（非游戏引擎）：

- **同一场景树**内鸟瞰地图（2D）、倾斜视角（2.5D）、沉浸 3D 无级切换，图层/实体/标注交互语义一致
- **空间数据一等公民**：矢量瓦片、GeoJSON、WMS/WMTS、主流坐标投影；glTF/点云/BIM 轻量化展示；ODR/OSC 经官方仿真插件支持
- **Rust 内核 + wgpu 渲染**：内存安全、无 GC；一等 Vulkan / Metal / DX12 / WebGPU，GL 3.3+ / GLES 3.0+ / WebGL2 降级档
- **SDK 形态**：经 C API 嵌入 Qt / Flutter / C# (WPF/Unity) / Web，不绑架宿主主循环；启动体积目标 ≤10 MB

## 架构（一句话）

`visia-core`（数据模型/空间索引/坐标系/场景图）→ `visia-render`（渲染抽象 Trait）→ `visia-render-wgpu`（默认后端：基于 wgpu 的自研渲染管线，D4 终审定案；后端抽象保留可插拔）。宿主侧只依赖 C API，后端选型不破坏兼容承诺。

## 仓库结构

```
├── AGENTS.md / SKILL.md   # 代理知识库与技能注册表
├── docs/
│   ├── whitepaper.md      # 白皮书 v0.1.0（定位与商业策略一手源）
│   ├── architecture.md    # 架构设计基线 v0.1（D4 定案后的设计入口）
│   └── reference/         # 开源参考项目库 + 决策证据快照（evidence/）
└── .agents/               # 项目记忆 / 规则 / 技能（agent 工具链）
```

## 开发状态

**Phase 0 完成，工程未动工。** 已定案：Rust 核心、wgpu 直用自研管线（后端终裁 D4）、SDK 形态（C API 唯一稳定边界）、Open Core。决策与证据链见 `docs/architecture.md` 与 `docs/reference/`。下一步：Cargo workspace 骨架（visia-core / visia-render / visia-render-wgpu）。

## 商业模型

Open Core：核心运行时 **MIT OR Apache-2.0 永久开源**（已发布版本许可不可撤销）；商业产品为 **Visia Studio** 编辑器与企业级插件/支持。

## 路线图

| 阶段 | 内容 |
|------|------|
| MVP（当前） | 核心库骨架、glTF+GeoJSON 加载、2D↔3D 切换演示、宿主嵌入示例 |
| Alpha | 空间索引/坐标系完善、2.5D 白模、C API 稳定化 + 绑定生成 |
| Beta | Visia Studio 预览、ODR/OSC 插件、孪生数据实时绑定 |
| 1.0 | 编辑器+运行时正式发布、插件市场、商业支持上线 |

## 许可

核心运行时以 **MIT OR Apache-2.0** 双许可开源（文本见 [LICENSE-MIT](LICENSE-MIT) / [LICENSE-APACHE](LICENSE-APACHE)）。已发布版本的许可**不可撤销、不追溯变更**。Visia Studio 与企业插件为独立商业授权。

## 联系方式

- 仓库：gitee.com/chengxuewen/VisiaEngine
- 官网 / 邮箱：待公布
