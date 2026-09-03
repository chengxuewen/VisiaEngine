# wgpu + Rust GPU 栈 — 生态体检

**快照 2026-09-03**（shields/crates.io 实测）：wgpu 18k★，v30.0.1（2026-08-22），33.6M dl，Apache-2.0，主分支活跃。vello 4.3k★（0.10.0，2026-08-14）。glow 0.18.0（2026-07-09，**39.1M dl——比 wgpu 还高**）。three-d 0.19.0。fyrox 1.0.1（2026-03-28）。crates.io 上 `angle` crate：0.5.0，**2022-03 停更 = 死**（ANGLE 绑定无现成维护件，集成须自建 bindgen）。

## 画像
- **wgpu**：Rust 原生 WebGPU API 实现 + WGSL(naga)，后端 Vulkan/Metal/DX12 一等、WebGL2 best-effort、wasm→浏览器 WebGPU。Firefox/Deno 的 WebGPU 即 wgpu（见 WebGPU 档）。治理：gfx-rs 组织，季度发布带破坏性变更（v29→v30 间隔 ~2 月）。
- **linebender 系**：vello（GPU compute 2D 矢量，Vello Hybrid 为生产形态）+ lyon（GPU 路径细分事实标准）+ cosmic-text/parley/swash（文本）+ fontique/color。Raph Levien 主导。
- **glow**：GL/GLES/WebGL 统一加载器——legacy 渲染路径（若走 GLES/ANGLE）的 Rust 标准入口。

## 对 VisiaEngine
- **借鉴**：wgpu 的 API 面 = `visia-render` trait 设计的语义词汇表（texture binding/limits/adapter 选择逻辑直接映射）；WebGL2 兜底与 downlevel capability flag 的处理是 SDK 兼容矩阵教材；linebender 的 `color`（色彩管理）/`fontique`（字体发现）是"把基础设施做成独立小 crate"的模块化示范。
- **规避**：① 把 vello 当现成 2D 引擎——6 年未 1.0，它是组件不是产品；② 季度破坏性节奏会打穿 SDK 稳定承诺 → **必须 pin 主版本 + 抽象层隔离 wgpu 类型出不了 visia-render**（这正是 trait 抽象存在的理由）；③ glow 39M dl 提醒：Rust 世界 GPU 负载仍大量在 GL 路径上，legacy tier 问题不是伪需求。
- **现状判定**：wgpu 作为主栈无对手；Rust 2D 矢量渲染没有"地图级"成品（vello 无瓦片/样式概念）→ 自研 2D 样式层是必然工作量，与 `evidence/2026-09-03-wgpu-direct` §2 缺口清单一致。

## 来源
img.shields.io + crates.io（2026-09-03）；`evidence/2026-09-03-wgpu-direct` §2/§4/§5。
