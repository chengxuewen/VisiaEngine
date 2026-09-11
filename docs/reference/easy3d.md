# Easy3D — C++ OpenGL 全栈参考：宿主嵌入反面教材库 + 数据/渲染设计正例

**快照 2026-09-03 实测**（`.refinfo/Easy3D/` 本机归档，345M / 1419 C++ 文件 / 52 教程）：**GPL-3.0——一切仅作概念参考，零代码/文本搬运**。定位=3D 建模/几何处理/渲染研究库（含 Python 绑定）。团队分析 4 域 × 27 条结论，34 处证据路径 C14 实测 32 真 2 笔误（行号级，机制无损）。

## 画像
数据结构（PointCloud/SurfaceMesh/PolyMesh/Graph，共享列式属性系统）+ OpenGL 高层封装（shader/纹理/pass 效果）+ 自足 GLFW viewer（事件循环内藏）。**非 SDK 形态**：viewer 与 GLFW 深耦合，Qt 集成=在示例层重写整套——这正是 VisiaEngine 反方向的镜子。

## 对 VisiaEngine（三档，编号=域.序；A 核心层 B 渲染层 C 嵌入层 D 流程）

### 借鉴（采纳其概念，我方形态重写）
- **A1/A2 列式命名属性系统**：类型化闭合 enum 列 + 几何/用户属性同命名空间 → 直接解我方 geo `serde_json::Map` 样式热路径字符串化。证据 core/property.h L460/587/628、graph.cpp:37-41。排：样式轮。
- **A4 加载报告+修复分型计数**：builder 保流形、end_surface(log) 报"修了什么多少"；我方 io-gltf/geo 返回 `LoadReport`+`RepairPolicy`。证据 surface_mesh_builder.h:40/122-125、5 loader 全走 builder。GIS 脏数据即此病。排：小片可做。
- **B6 Camera 三增量**：透视/正交共享 zoom 因子（切换连续，GIS 刚需）、pivot turntable、dpi_scaling。证据 camera.h:143/342-354。排：camera 轮。
- **B7 廉价值打包**：标量→色带 LUT+clamp（state.h:58-61 ✓ GeoJSON 热力图现成答案）、纹理 fractional repeat、polygon-offset 线面抗 Z-fighting（viewer.cpp:1805 ✓）、大图分条（texture.h:159）。排：随各属轮。
- **C5 绑定面裁剪法**：39 renderer 头只手绑 4（camera/drawables/renderer/state，drawables 合并），unused/ 隔离带，GLOB 注释防全扫（python/CMakeLists.txt:6 ✓）→ **C ABI 首版=最小可用集+graveyard 流程**。排：capi 设计输入。
- **D1 编号教程系列**：7 主题段+空号插入+每数据类型 4 连招+dir=target=标题=文档 ID 单源（CMakeLists 91 行 ✓/T101 仅 2 文件 ✓）→ examples 教学化蓝本；900s=GIS/孪生/AV/BIM 垂直切片。
- **D3 零漂移文档管线**：Doxyfile EXAMPLE_PATH 收割教程源码（L854 ✓）→ mdbook `{{#include}}`+rustdoc+cbindgen 注释规范。排：文档轮。
- **D5 测试三层法**：单元/帧预算 auto-smoke（duration=1500ms ✓ main.cpp:107）/**显式标注人工交互+诚实横幅**（L133 ✓）。排：即刻采纳（纪律零成本）。

### 移植（机制级照搬，语言重写）
- **A5 渲染资源按实体挂载 + State 按属性名绑色**：换样式不碰几何层（renderer.h:138/154、state.h:49-61 ✓）。另：bbox 懒算（invalidate 后访问时重算，model.h:64-68 ✓——「即算不常存」概念出处，pick 用）。排：PBR 轮所有权重构。
- **B3 pass 三件套**（GIS/孪生相关性排序）：Shadow/PCSS（⚠ **GL[-1,1]→wgpu[0,1] 即 PIT-5 雷区**）、3-pass SSAO+共享几何 FBO（ambient_occlusion.h:106-114 ✓）、DDP 透明（dual_depth_peeling.h:98 ✓；wgpu dual-source 可做，固定 peel 数，与 MSAA 互斥=我方 non-MSAA golden 线天然兼容）。排：Alpha 档。
- **B4 线宽数学**：世界半径×距离相机空间扩片 `cross(view_dir,axes)*radius*0.5`（width_control.geom:67-68 ✓），GS→VS instancing。排：屏幕空间线宽项直接方案。
- **B1 带名管线缓存+负缓存+F5 热重载**（shader_manager.h:119/124 ✓）。排：PBR 轮。
- **D4 中央资源解析器**：compile-time 默认+env 覆盖+fixture <10MB 预算（其 27M 教训）（resource.h ✓）。排：examples 增多前。

### 优化（其做不到/做坏，我方做对即超越）
- **C1/C2 嵌入通路彻底重做**：其 Qt 靠 QApplication::notify 私有 hack（Mapple main.cpp:53 ✓）、ImGui 版丢鼠标坐标（upstream 实锤）、GLFW 指针泄进 public ABI（viewer.h:38 ✓）→ 我方 capi=`visiaengine_on_input()` 单入口+redraw 回调+renderer 层零窗口类型（其 renderer 零 glfw 引用 ✓ 证明该纯度可达）。**嵌入片计划约束**。
- **C3/C4 几行接入配方**：create() 即渲染就绪（零 init 仪式）；Qt 侧=Widget 持引擎对象+父容器管寿命（T204 viewer.h:47 ✓），rwh 同构且免其子类化代价。
- **D2 教程真执行**：其教程仅编译、tests 默认 OFF 从不进 CI（BUILD_TESTS=OFF ✓）——我方 `--frames N`+golden 已反超；取其"编译全覆盖+功能必须带可运行示例"两条纪律。
- **A3 句柄安全**：其 int 裸索引 GC 后全失效；我方 slab+generation 天然抗 FFI ABA——维持并写进 capi 规范。
- **B5/B2 材质系统**：其 #include 机制出厂零使用（全仓 0 命中 ✓）、mega-shader bool 全家桶 → 我方 WGSL 极简 include/const 预处理器（~50 行）+ pipeline key 变体分派；材质 DSL（P4）3+ 形态前不动。
- **D7 绑定教程=ABI 契约测试**：其 python/tutorials 同 NNN 镜像（sphinx_gallery ✓）→ 我方 capi 教程跑不通即 API 未完成。
- **A6/A7 负空间戒律**：不预建场景图（其 flat vector 无 Node 跑 10 年 ✓）；不泛型化 Scalar（其 traits→具体类撤退史 ✓ template 计数 25/30/15/10）。

## 现状判定
非竞品（GPL+几何算法定位），但为**下一里程碑（宿主嵌入）价值最高的反面+正面双料教材**：C 域 8 项发现归档合并为 5 条，全部预写了我方 capi 约束；A/D 域给出属性、加载报告、教程、测试四项即刻可捡的实利。

## 来源
团队 easy3d-analysis（4 员 × 36 轮，2026-09-03）；全部引用路径经编排者 grep/sed 实测。归档只读，`.refinfo/` 永不入库。
