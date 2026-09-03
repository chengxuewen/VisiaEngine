# ANGLE / Dawn — 翻译层双雄

**快照 2026-09-03**（shields 实测）：google/angle 4.1k★，last-commit ≈2026-09-01（活跃，随 Chromium）；google/dawn 1.1k★，当日有推送。BSD-3-Clause（angle）。**crates.io 上无维护中的 ANGLE 绑定件**（`angle` crate 0.5.0，2022 死）。

## 画像
- **ANGLE**：GLES/EGL **前端** → D3D11/D3D11on12/Vulkan/Metal/GL 多后端翻译层。存在的唯一理由 = 在不提供合格 GLES 的平台上稳定供 GLES（Windows→D3D11，AMD/Intel Linux→GL，Chrome OS/Android 的 ANGLE-on-Vulkan）。是**全球部署最广的图形翻译层**（Chrome/Edge/Firefox 的 WebGL 即 ANGLE；Flutter Windows 引擎确认链接 ANGLE（BUILD.gn 证据）；**Qt6 证伪**——RHI 的 D3D11 为原生后端，qtbase 3rdparty 无 angle，Qt5 时代 `-opengl es2 -angle` 才是翻译层用法）。
- **Dawn**：Google 的 WebGPU native（C++），与 wgpu 同 API 不同实现。对照价值：Chromium 的"平台差异吸收层"工程形态。

## 对 VisiaEngine
- **核心命题（已验证，原假设推翻）**：wgpu v30 **有原生 GL/GLES 后端**（Windows GL3.3+ 默认 WGL / Linux-Android GLES3.0+，downlevel 档）且**自带 ANGLE 构建选项**（Windows `cfg(windows_angle)`、macOS `angle` feature，README 📐 注记）→ "独立集成 ANGLE"是伪需求：legacy 成本从"自建第二 GLES 渲染后端"降为"构建开关 + QA 矩阵"。
- **借鉴**：后端能力矩阵 + workaround 机制（ANGLE 对 buggy GL 驱动的逐设备 workaround 表）= SDK 兼容矩阵管理的最佳工业样板；其 "feature level" 探测 → 质量分级（对应 whitepaper 的兼容 tier 语言）。
- **规避**：① GN/ninja C++ 构建链进 cargo 工程的运维税（预编译分发/大小 ~2-4MB *UNCERTAIN*）；② D3D11 翻译层的性能税（buffer upload、无 async compute）——重 3D 场景上只当兜底不当主路；③ 别把 ANGLE-on-Vulkan 和 ANGLE-on-D3D11 混为一个方案（解决的是完全相反的设备问题：前者"有 Vulkan 无好 GLES"，后者"啥都没有有 D3D11"）。
- **XP 判词（已闭环）**：ANGLE D3D9 后端已于 2026-04~07 自 main 移除（commit 级证据）+ Rust MSVC 官方要求 Win10+（rustc book 当日核验）→ XP 无解，从承诺文本移除。Win7 = 非官方 best-effort（技术可通：wgpu-GL/ANGLE-D3D11，但需钉旧工具链），矩阵见 [legacy-platforms.md](legacy-platforms.md)。

## 状态
✅ 调研完成：全证据见 [2026-09-03-angle-integration.md](evidence/2026-09-03-angle-integration.md)（grep.app commit 级；含 Qt6 证伪、ANGLE-on-Vulkan 对 Android 为红鲱鱼的判定、§3.3/§4 建议措辞）。

## 来源
img.shields.io（2026-09-03）；crates.io；待引：ANGLE 调研备忘录。
