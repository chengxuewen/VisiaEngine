# Qt3D / Qt Quick 3D / QRhi — 官方厂做域 3D 的冷场实证

**快照 2026-09-03 实测**：Qt/qt3d 229★；Qt/qtquick3d **74★**（last-commit 7 月）。商业 Qt + GPL/LGPL 双。

## 画像
Qt 官方 3D 两代：Qt3D（ECS 式，Qt Widgets 系，维护向）→ Quick3D（QML 声明式，USD/glTF 导入，多年 Technology Preview）。底层骑在 **QRhi** 上：Qt 自研 GPU 抽象层，统一 Vulkan/Metal/D3D11/D3D12/OpenGL ES（注：**Qt6 的 D3D11 为原生后端非 ANGLE**——ANGLE 调研证伪，Qt5 `-opengl es2 -angle` 时代才是翻译层用法）。QGIS 3D 建立在 Qt3D 上（连锁证据：官方组件也只是"可用"级）。

## 对 VisiaEngine
- **借鉴**：① QRhi = 多后端兼容差异的 C++ 工程样本（后端能力探测、shader 编译管线 glslang→各目标、纹理/资源抽象），与 wgpu 互为镜像对照；② Quick3D 的 QML 属性绑定↔引擎数据桥接形态 = "宿主 UI 框架与渲染核同步状态"的问题清单；③ USD 导入器（Quick3D 做了）= 孪生交换格式接入先例。
- **规避（教训为主）**：① **stars 数 = 生态热度真相**：Qt 官方+商业推广位才 74 颗——3D 领域没有数据故事就没有开发者引力；Visia 的护城河必须落在地理/孪生数据语义，不是"又一个 3D 渲染 API"；② 两代 API 断层（Qt3D→Quick3D）= 抽象层早期锁 API 的代价，visia-render trait 第一版要做破坏性演进预留（crate 版本号纪律）；③ Technology Preview 多年转正难——对外承诺里警惕"preview"语义。
- **现状判定**：宿主侧知识（未来 Qt 嵌入示例的坑：QRhi 与 wgpu surface 共存 = 同窗口两个 GPU 上下文，交互模式须实测；优先走"渲染进纹理喂 QML/Widget"的无争路径）。

## 来源
shields 实测（2026-09-03）；Qt 文档公开。
