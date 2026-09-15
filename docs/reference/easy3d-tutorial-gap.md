# Easy3D tutorials 带缺口分析（对标存档）

> 生成：2026-09-15（cmake-example-targets 轮 S5 存档，C-3 裁决=是）。
> 源：`.refinfo/Easy3D/tutorials/`（52 件，首位带号 1-7）只读解剖 + 本仓 E 系 11 件。
> 用途：后续渲染/功能轮的立项依据。**引用只述机制，禁引 MediaServo/Easy3D 内部编号入新条款**。

## 带映射现状

| Easy3D 带 | 件数 | 我方对应 | 缺口（立项候选择） |
|---|---|---|---|
| 1xx 数据类型（cloud/mesh/graph/poly × base/connectivity/property/IO） | 15 | E201/E202/E203 | **点云例**（point marks 无独立演示）；**AttrSet 属性查询例**（CORE-11..13 有契约无演示面）；GeoJSON 写回 IO |
| 2xx viewer/UI（默认/imgui/wx/Qt/多视口/相机插值/真相机） | 7 | E101/E301/E702/E703 | 多视口（一场景双视图）；相机插值漫游；HUD 叠加层 |
| 3xx drawables（纹理/标量场/向量场/IMG viewer/剖面/动画/文本） | 12 | E201 texquad | **剖面裁切**（clip plane）；标量场着色；文本渲染（依赖字体面，长期） |
| 4xx interaction（picker×2/点选/虚拟扫描/物体操控/碰撞） | 6 | E401/E203（+E402 空号预留） | hover/多选 UI（=E402 既定候补）；物体操控（gizmo→D7 rebase 面）；碰撞查询 |
| 5xx rendering（AO/hard shadow/soft shadow/透明/EDL/depth map） | 6 | E501(PCSS) | 半透明排序（blend 序=管线议题）；EDL 点云光照；SSAO（=批 4g backlog） |
| 6xx 算法-core（细分/凸分割/曲线） | 3 | E202 内嵌 tess | 细分独立演示；曲线带（缓冲车道线类） |
| 7xx 算法-cloud（法向/重建/平面提取） | 3 | E203(平面量测) | 法向估计演示；平面提取演示 |

## Easy3D 结构采纳已落（本仓 cmake 面）

中心函数（`visiaengine_setup_examples`）/ 名=文件=target=ctest（零映射表）/
FOLDER 带分组 / 每例显式契约行 / 双层可选依赖门（Qt+X11）/ 注册即义务
（glob⇄表双向校验=configure 硬错，反其注释式禁用）。

## 已反超项（勿退坡）

- Easy3D 52 例 argv 零消费 → 结构上不可 CI 化；我方 `--frames N` 全族 +
  ctest 同入口 + 无 X 机 77-Skip 合同。
- 其 tutorials 零 add_test；我方 13 条 ctest 注册、ci 段常驻。
- 其资源路径=编译期绝对宏+运行时向上爬树；我方仓根 chdir 合同（stub 单点）。

## 立项建议序（并入 Alpha 队列视角）

1. 点云例 + AttrSet 属性例（数据面已齐=纯演示成本，填 1xx 带空洞）
2. 剖面裁切（2.5D GIS 刚需；管线 depth/stencil 面已具备）
3. E402 hover/多选（交互带收口，空号已留）
4. 半透明排序 + SSAO（=4g backlog 合流）
5. 多视口/相机插值（漫游演示向，批 7 站点素材可复用）
