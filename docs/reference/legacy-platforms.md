# 老旧与嵌入式平台支持矩阵 — wgpu/ANGLE/工具链三方交叉

**快照 2026-09-03**。证据链：wgpu v30 README（本地抓取件）+ rustc book 平台页（当日直采）+ `evidence/2026-09-03-angle-integration`（ANGLE 后端/移除史，grep.app 代码证据）+ Wikipedia Mali 架构表（当日）。本文与 angle 备忘录重叠处不重复展开，只给判定。

## 核心纠偏（相对调研前的直觉假设）
1. **wgpu 有原生 GL/GLES 后端**（README 首句 + 平台表）：Windows 🆗 GL 3.3+（默认 WGL，`cfg(windows_angle)` 切 ANGLE）；Linux/Android 🆗 **GLES 3.0+**；Web 🆗 WebGL2。"wgpu 无 GL 路" ❌。
2. **ANGLE 已是 wgpu 的构建选项**，不是需要独立集成的第二后端。`cargo add angle` 这类需求不存在（维护件 `angle` crate 已死，但上游 wgpu 路线已通）。
3. **ANGLE 的 D3D9 后端已从 main 移除**（2026-04~07 系列 commit，angle 备忘录 §2）→ **XP 经 ANGLE 无解**；且 **Rust MSVC 目标官方要求 Win10+**（rustc book 原文当日核验）→ XP/Win7 在工具链层就断。
4. **Qt6 不用 ANGLE**（其调研已证伪：Qt6 RHI = 原生 D3D11/D3D12/Vulkan/Metal/GL；qtbase 3rdparty 无 angle）。Flutter Windows = 真 ANGLE 用户（BUILD.gn 链接证据）。
5. **RK3588 GPU = Mali-G610 = Valhall 3rd gen**（Wikipedia 变体表）→ 开源 Vulkan 路径是 **Panvk**，不是 Panthor（后者管 G615/Immortalis 起）。

## 平台 × 路径矩阵（判定版）

| 平台 | 可行渲染路径 | 判定 |
|------|-------------|------|
| Win10/11 | wgpu DX12 或 Vulkan（一等）；GL 兜底 | ✅ T1 主战场 |
| Win7 SP1 | 技术上 wgpu-GL(ANGLE-D3D11, FL10_0) 可通；**Rust MSVC 官方线已断**（需钉旧 rustc/GNU 目标 + 自建管线） | ⚠ **非官方 best-effort tier**，除非客户显式付费要求，不进承诺 |
| XP/Vista | Rust 1.27 起无目标支持；ANGLE D3D9 已删；Chromium 2016 停 | ❌ **不可能**，从任何承诺文本中移除 |
| Android API 24+ | 系统 GLES（CDD 强制）或 Vulkan（新 SoC）→ wgpu GL/Vulkan 双路，无需 ANGLE | ✅ 移动端主战场（API24 覆盖 ~96.6%，与 Flutter 底线对齐） |
| Android API 21–25 | GLES 3.0 子集 + wgpu downlevel（GL 后端） | 🆗 可跑，功能子集降级；3.4% 份额按产品决策 |
| RK3588 (G610/Valhall) | **两条**：① Rockchip/ARM 闭源 libmali blob（Android GKI/厂商 BSP 5.10，Vulkan 1.2+，但 blob 与内核/发行版强耦合）② 主线 **Panvk**（Mesa 24.x+ Valhall 支持，成熟度 2025-26 快速演进中，*版本细节 UNCERTAIN*） | ✅ 可行，但**发布二进制锁一种驱动栈**（选①跟厂商走，选②跟 distro 走），QA 矩阵砍半否则成本翻倍 |
| RK3566/8 级 G52 (Bifrost) | libmali 或 Panvk（Bifrost 支持更早熟）/ panfrost GLES | ✅ 同上策略 |
| Web（宿主浏览器） | wgpu→WebGPU（Chrome/Edge 113+、Safari 26、Firefox 141 全谱）；webgl feature 兜 GLSL 老浏览器 | ✅ Web 一翼成立；**但 Win7 上 Chrome 119+ 已不可用** → Web 路不救 legacy Windows |

## 对后端终裁与白皮书的含义
1. 白皮书 §3.3 可诚实升级为：**"Vulkan/Metal/DX12 一等 + GLES 3.0+/GL 3.3+ 降级档 + WebGL2 兜底"** ——这比原文"适配 Vulkan、Metal、DX12 及 WebGPU"更强且全部有上游文档背书。
2. "跨平台一致"加限定语：**一致的是 API 面与功能 tier 语义，不是一致画质/帧率**（downlevel = 特性子集，rerun 的 WebGL 兼容档是同构先例）。
3. XP 从一切文本中消失；Win7 = "无官方承诺的 best-effort"（若销售侧有 Win7 大单，那是企业插件+定制构建的商务题，不是架构题）。
4. Android 底线定 **API 24**（对齐 Flutter 生态，覆盖 96.6%）；老设备靠 wgpu GL downlevel 而非 ANGLE。
5. RK3588 的真正成本在**驱动栈选择与 QA 矩阵**（blob vs 主线 distro 二选一先定客户画像），不在 API 可行性。

## 来源
本地 `wgpu_README.md`（trunk，2026-09-03 抓）；doc.rust-lang.org windows-msvc 页（当日）；`evidence/2026-09-03-angle-integration` §2-§9（含 grep.app/commit 证据）；en.wikipedia.org/wiki/Mali_(processor)（当日，rev 1369060869）；docs.rs wgpu platforms/angle 路由存在性（README 内链）。*UNCERTAIN：Panvk Valhall 精确 Mesa 版本线（gitlab 反爬未取一手）；libmali blob 具体 Vulkan 版本矩阵。*
