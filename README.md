# VisiaEngine（维视引擎）

> **多维空间可视化引擎** — 统一的 2D / 2.5D / 3D 渲染管线，为"看见数据的本质"而生。

**状态：白皮书 v0.1.0 定稿 + Phase 1 MVP 完成 + 批 4 渲染强化 + 批 5 文档归位（2026-09）。** 完整定位见 [docs/whitepaper.md](docs/whitepaper.md)。

## 它是什么

一个开源、轻量、可嵌入的空间可视化**引擎内核**（非游戏引擎）：

- **同一场景树**内鸟瞰地图（2D）、倾斜视角（2.5D）、沉浸 3D 无级切换，图层/实体/标注交互语义一致
- **空间数据一等公民**：矢量瓦片、GeoJSON、WMS/WMTS、主流坐标投影；glTF/点云/BIM 轻量化展示；ODR/OSC 经官方仿真插件支持
- **Rust 内核 + wgpu 渲染**：内存安全、无 GC；一等 Vulkan / Metal / DX12 / WebGPU，GL 3.3+ / GLES 3.0+ / WebGL2 降级档
- **SDK 形态**：经 C API 嵌入 Qt / Flutter / C# (WPF/Unity) / Web，不绑架宿主主循环；启动体积目标 ≤10 MB

## 架构（一句话）

`visiaengine-core`（数据模型/空间索引/坐标系/场景图）→ `visiaengine-render`（渲染抽象 Trait）→ `visiaengine-render-wgpu`（默认后端：基于 wgpu 的自研渲染管线，D4 终审定案；后端抽象保留可插拔）。宿主侧只依赖 C API，后端选型不破坏兼容承诺。

## 仓库结构

```
├── AGENTS.md / SKILL.md   # 代理知识库与技能注册表
├── crates/                # 纯核心 5：core / render / render-wgpu / io-gltf / geo（D6 命名）
├── examples/              # 例子按语言分层：rs（Rust 教程系）/ c / cpp / qt——ctest 统一清单一键跑
├── bindings/              # 对外实现面：c（capi crate+手写头+探针）/ cpp（RAII 头）/ qt（widget）/ js（wasm+demo）
├── pixi.toml / pixi.lock  # 开发环境单源（D5：conda-forge，含 rust 工具链）
├── docs/sdd/              # 行为契约条款（与测试双向追溯）
├── docs/
│   ├── whitepaper.md      # 白皮书 v0.1.0（定位与商业策略一手源）
│   ├── architecture.md    # 架构设计基线 v0.1（D4 定案后的设计入口）
│   └── reference/         # 开源参考项目库 + 决策证据快照（evidence/）
└── .agents/               # 项目记忆 / 规则 / 技能（agent 工具链）
```

## 开发状态

**Phase 1 现状（数字以门禁实报为准）**：7 crate workspace（核心 5 + bindings 面 2）、**116 条** SDD 行为契约全绿（`scripts/spec-trace.sh` 双向追溯，`gate-docs` 锁本文数字与实报一致）。渲染管线：glTF/GeoJSON 装载 + 材质纹理（PBR mock-up）+ GPU instancing（10 万楼块单 draw）+ 屏幕空间线宽/真圆点扩片 + 方向光 PCSS 软影 + 2D↔3D 无级切换 + 拾取/量测闭环 + D7 远坐标重基（像素级验证）。离屏 golden 真机无 SKIP；**ctest 统一例子清单（19 条，含 SDL3 窗口族 xvfb 子态）+ 三 gate（abi/trace/docs）+ cmake-smoke 三态三锚 + bench 制品链**机器门禁；教程 E 编号系见 [docs/tutorials.md](docs/tutorials.md)（文件名=头注=索引单源）。**宿主嵌入**：C ABI 21 入口（人审手写头 + 双 C demo 真跑；属性读三口 + B1 数据带显隐/程序化增删）+ Web 面（visiaengine-wasm 双面镜像，demo 页可跑）——Qt 宿主实证 ✅（header-only widget+真窗 smoke 三态）；npm/pip 打包挂 Alpha。

```bash
```bash
bash bootstrap.sh && source pixi.sh && pixi run ci   # 环境三步
pixi run cmake --preset bare && pixi run cmake --build --preset bare   # C++/IDE 入口（根 CMake 门面）
# IDE 打开工程即得：测试面板=全语言例子统一清单（ctest --preset bare）；构建菜单含逐例 cargo-build_*/cargo-run_* 步骤；Rust 例另有 rust-analyzer Run/Debug
```

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
