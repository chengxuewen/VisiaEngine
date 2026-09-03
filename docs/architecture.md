# VisiaEngine 架构设计

**v0.1 | 2026-09-03** | 输入：D2（栈=Rust/wgpu/SDK/Open Core）+ D4（终裁：wgpu 直用自研，不采用 Bevy）+ 证据链（`reference/evidence/` 4 份 + 参考项目库 19 篇）
状态：**设计基线，未动工**——本文是 MVP 开工的设计入口，决策点见文末表格。

## 设计不变式（优先级高于一切图示）

1. **`visia-core` 永不依赖 `visia-render`**：数据模型可无头运行（headless 出图/测试/Studio 复用同源）。
2. **C ABI 是唯一稳定边界**：wgpu/bevy 级类型止步于 `visia-render-wgpu`，永不泄漏到 trait 之外（D4 否决 Bevy 的第一主因即此）。
3. **单一语义源，后端只增不分叉**（Unity 三管线互斥学费）：任何后端不得引入新的场景/材质语义。
4. **主循环归属宿主**：引擎只被 pump，不自建事件循环（rerun spawn/connect 模型为参照，Flutter embedder 为 ABI 纪律范本）。
5. **兼容 = tier 参数化，非编译期分叉**：能力运行时查询（§图⑥）。
6. **copyleft 依赖（PROJ/GEOS/OSG 系/LGPL 件）一律 feature-gate 到扩展层**，核心包静态链零污染。

## ① 全景分层

```
┌─────────────────────────────────────────────────────────────┐
│  宿主应用层        Qt/C++   C# (WPF/Unity)   Flutter   Web    │
└───────────────┬─────────────────────────────────────────────┘
                │  C ABI（唯一稳定边界，版本化握手）
┌───────────────▼─────────────────────────────────────────────┐
│  绑定生成层     visia-capi(头文件) → bindgen/csbindgen/pub   │
├─────────────────────────────────────────────────────────────┤
│  引擎核心层     visia-core ── visia-geo ── visia-style       │
│  (纯 Rust)     场景图/坐标  MVT/投影调度  图层/样式/表达式     │
├─────────────────────────────────────────────────────────────┤
│  渲染抽象层     visia-render：Trait + 渲染指令(IR)，无 GPU 类型 │
├─────────────────────────────────────────────────────────────┤
│  后端层         visia-render-wgpu（默认·自研管线，D4）        │
├─────────────────────────────────────────────────────────────┤
│  平台层         wgpu v30                                     │
│   T1: Vulkan│Metal│DX12│WebGPU   T2: GL3.3+/GLES3.0(±ANGLE)  │
└─────────────────────────────────────────────────────────────┘
```

## ② Crate 依赖方向

```
                 visia-capi ─(cdylib/staticlib + cbindgen 头)
                     │
   ┌───────┬────────┼─────────┬──────────┬──────────┐
visia-core  visia-geo  visia-style   visia-io-*   visia-proj
 (无 wgpu    (瓦片/     (样式spec,     (gltf/       (投影: feature-gate
  依赖)      坐标/      表达式)        geojson/      纯Rust | C绑定二选一)
             LOD调度)                  mvt)
   └───────┴────────┴─────┬────┴──────────┴──────────┘
                    visia-render (纯 IR/Trait)
                          │
                  visia-render-wgpu ──→ wgpu, lyon, cosmic-text, rstar
 [扩展档，核心零依赖] visia-3dtiles │ visia-odr(Beta) │ visia-ifc(Beta+)
```
铁律：依赖箭头只向下；`visia-style`/`visia-geo` 独立性 = Open Core 插件边界的代码化。

## ③ 一帧数据流

```
host 事件 ──▶ 输入泵 ──▶ 交互/相机状态 (主世界)
                          │
        ┌─────────────────▼──────────────────┐
        │ 1 CULL     R-tree/quadtree + SSE 选 LOD │
        │ 2 STREAM   瓦片调度器(需求−缓存→合并→限并发) │──▶ 后台线程 IO
        │ 3 PREPARE  脏标记 → 渲染条目抽取(视口快照)    │
        │ 4 BUILD    frame graph 记录(自研薄层)         │
        │            pass: shadow?→tile-2D→3D→text     │
        │ 5 ENCODE   → wgpu command buffer              │
        │ 6 SUBMIT/PRESENT (非阻塞 poll 语义)           │
        └────────────────────────────────────┘
```
- **场景存储定调（本轮 Unity-DOTS/UE6 证据收敛）**：slab-handle 索引 + 脏标记起步，**不上 ECS**；若未来性能证据推翻，替换限 `visia-core` 内部，不破 C ABI。
- **渲染条目 IR 标注 cluster-ready**（UE Nanite 启示）：数据结构预留 meshlet/indirect-draw 扩展位，MVP 不实现、不封死。

## ④ 2D/2.5D/3D 统一与坐标精度

```
     统一投影 P = mix(ortho, perspective, t)   t∈[0,1]
  t=0 正交/鸟瞰(地图)   t=0.5 倾斜(2.5D)   t=1 透视(3D)
     ▲ 插值的是视锥参数，单一相机模型

  坐标精度：世界 f64(ECEF/投影域) ──RTC 分块──▶ 渲染 chunk 局部 f32
     (大坐标偏移以相机/瓦片中心为原点；不这么做 = 地图引擎第一杀手)
```
悬题（进 core 设计文档）：RTC 粒度 per-tile vs per-viewer rebase 阈值——**一次定对**（UE 场景模型换代拖三代的教训）。

## ⑤ 宿主嵌入模型

```
路径 A（自有 surface）: host 原生窗口句柄 ──raw-window-handle──▶ wgpu surface
路径 B（纹理互操作，Qt/Unity 优先）: visia 离屏渲染 → 纹理回调 → 宿主合成

线程契约（仿 Flutter embedder）:
  platform thread = 宿主主线程（visia 绝不自建事件循环、绝不抢占）
  engine work     = 内部任务池（C API 投递/回调）
  render work     = 宿主调 visia_frame() 驱动；专职渲染线程可选开关
ABI: 结构体首字段 {size, api_version}；只导出 opaque 句柄；
     错误 = int code + visia_last_error()
```

## ⑥ 兼容 Tier 能力矩阵

```
            T1 一等            T2 降级              T3 Web
API面        全部               全部                全部     ← "一致"承诺仅此义
compute      ✅                 GLES3.1+/缺失        ✅(WebGPU)/—(WebGL2)
indirect     ✅                 GL 路径模拟          ~
阴影/MSAA    全配               半配                 半配
纹理压缩     BC/ETC2/ASTC       按驱动               按浏览器
→ visia_capability_query() 返回 tier+flags；样式/图层按 flags 自动降画质
```
本轮再验证：Godot 4.7（Vulkan+D3D12+Metal+GL+ANGLE 选项）与 wgpu **独立收敛到同构矩阵**；three.js WebGL/WebGPU 双 renderer 十年并存证明"运行时选档"路线可行。

## ⑦ 数据源流式调度

```
sources: MVT(http) │ WMTS │ GeoJSON文件 │ glTF │ [扩展: 3D Tiles │ ODR │ IFC]
         └──────┬──────┘
   tile scheduler: (视口×LOD 需求) − 缓存 → 合并去重 → 限并发
   cache: LRU(内存预算) + 可选磁盘层；淘汰 → GPU buffer 池回收
   解码: 后台线程(geozero/open-vector-tile) → 网格化(lyon) → 上传(渲染线程)
```
已知风险（evidence 背书）：MVT 解析件薄（open-vector-tile 10★→可 vendoring）；3D Tiles 流式无成熟件→**自研归 Beta**；draco-core 新且薄。

## ⑧ 图层/样式模型

```
Scene ─ Layer*(有序)     Layer = Source + StyleSpec + 交互开关
StyleSpec(声明式, 可 diff): type(fill|line|symbol|heatmap|model|volume)
    ─ paint/layout 分离 ─ zoom/tier 函数 ─ 过滤表达式子集
渲染映射: fill/line→tile-2D pass; model→3D; symbol→text pass
```
悬题：spec 兼容 MapLibre v8 子集（迁移成本换生态好感）vs 自定义——样式系统设计文档时裁。

## ⑨ 构建交付矩阵

```
cargo ──▶ visia-capi: cdylib(.dll/.dylib/.so) + staticlib + visia.h(cbindgen)
      ──▶ wasm: npm 包（webgpu | webgl2 双 feature）
打包: vcpkg/NuGet/pub 镜像；体积预算核心 .so ≤6MB，总量标 ≤10MB（MVP 实测复核）
CI 矩阵: 全量测试跑 T1；T2 在 LLVMPipe/Mesa 软渲 + Android 模拟器抽查；WebGL2 浏览器
pin 纪律: wgpu 季度破坏 → 主版本 pin + 每季度升级窗口（全 tier re-verify）
```

## 未决决策点

| # | 悬题 | 影响面 | 裁定期 |
|---|------|--------|--------|
| P1 | RTC 粒度（per-tile vs viewer-origin rebase） | core 地基，一次定对 | scaffold 后的 core 设计文档 |
| P2 | 样式 spec 兼容性（MapLibre v8 子集?） | visia-style + 迁移工具 | 样式系统设计时 |
| P3 | RK3588 驱动栈（厂商 blob/BSP vs 主线 Panvk） | 发布矩阵/QA 成本 | **商务输入**（客户画像），不阻塞代码 |
| P4 | 材质表达式层（TSL 式）| 渲染远期 | post-MVP，不现在设计 |

## 开工 spike 清单（工具链就绪后 ≤1 天）

- [ ] 裸 wgpu triangle vs bevy-minimal 三平台 release 体积对照（补 evidence 数据缺口，仅归档用）
- [ ] 10 万实体 slab 遍历+抽取帧预算（验证 P0 保守选择的量级）
- [ ] raw-window-handle 入 Qt QWidget 最小 demo（图⑤路径 A 可行性）
- [ ] cosmic-text CJK 字形图集上 GPU 冒烟（R7 风险）
