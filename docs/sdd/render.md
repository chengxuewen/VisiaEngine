# visiaengine-render 行为契约（SDD）

> 条款标题行 `## REND-NN:` 为追溯锚点；测试挂 `// spec: REND-NN`。

## REND-01: backend_trait_object_safe
`RenderBackend` 必须 object-safe：`Box<dyn RenderBackend>` 在 crate 测试中可构造（多后端分发/宿主持有的前提）。

## REND-02: stub_impl_without_wgpu
本 crate 测试模块内以纯 CPU stub 实现 trait（本 crate 无 wgpu 依赖=构造事实）——证明**契约面零后端类型**（不变式②）；stub 处理 ClearColor 帧不 panic。

## REND-03: ir_variants_exhaustive_construct
IR `DrawCommand` 全部变体可构造，`kind()` 一一可辨（v0：ClearColor / DrawMesh）。

## REND-04: viewport_roundtrip
`Viewport::new(w, h, scale_factor)` 访问器无损往返；`logical_size()` = 物理尺寸/缩放因子。

## REND-05: camera_projection_variants
`Camera::Ortho/Perspective` 构造后 `is_orthographic()` 判别正确（2D/3D 统一入口的投影侧地基；切换动画属后续片）。
