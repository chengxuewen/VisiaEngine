# Flutter Embedder — C API 宿主嵌入的黄金标准

**快照 2026-09-03 实测**：flutter/flutter 179k★，当日活跃。BSD-3 + 专利授权。

## 画像
移动/桌面跨平台 UI 框架的**引擎-宿主契约层**：`flutter::FlutterEngineCreate/Awake/Shutdown`（版本化 C API，embedder.h），宿主窗口/平台线程/渲染线程三方解耦（platform thread 收事件、UI thread 跑 Dart、raster thread 上屏）。Windows 桌面嵌入历史跑 ANGLE(D3D11)（*待 ANGLE 线引用复核*）；渲染核 Skia→Impeller 迁移（Vulkan/Metal 直写，甩掉 GLES 翻译层——**反向路径：新引擎主动弃 GLES**）。

## 对 VisiaEngine
- **借鉴（C API 纪律的教科书）**：① **版本化 embedder ABI**（struct_size + API 版本字段握手）——visia-c-api 应直接采纳"首字段 size + version"模式，未来不兼容可探测；② 生命周期三态（Create/Awake/Shutdown + 宿主决定何时 pump）= "不绑架主循环"的接口语言范本；③ 渲染进外部 texture 的路径（FlutterTexture 回调）= Qt/CEF/游戏宿主嵌入的同构方案；④ Impeller 弃 ANGLE 的决策链值得读：为 GLES 翻译层付出的启动时 shader 预编译/功能子集代价——**legacy 兼容是税，收多久要主动定**。
- **规避**：① embedder.h 的平台碎片化（iOS/Android 定制层各自长肉）——Visia 的 C API 平台差异集中在一处 capability 查询；② 引擎版本与宿主 SDK 强耦合（Flutter SDK 全家桶锁版本）= SDK 分发反面教训，Visia 核心应可独立升级。
- **现状判定**：写 visia-c-api 设计文档前必读 embedder.h 注释；"宿主优先级"（先桌面 Qt/C# 后移动端）与 Flutter 演进方向（弃 GLES）共同暗示：**老 GLES 设备市场可能正在蒸发**——legacy 调研回来后作为背景权重。

## 来源
shields（2026-09-03）；docs.flutter.dev/embedded/* 公开。
