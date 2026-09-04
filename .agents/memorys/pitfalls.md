# VisiaEngine Pitfalls & Gotchas

> 每条 PIT 五段式，症状/根因/解法/验证缺一不可（沉淀职责见 C9）。重复 ≥2 次或耗费 >3 轮定位的教训，同步升级为 `rules/common/edit-safety.md` 的可执行规则。

## 格式示例

## PIT-{n}: <一句话标题> (YYYY-MM-DD)
- **症状**: <现象描述，含报错原文/可观察证据>
- **根因**: <为什么会发生，非表象复述>
- **解法**: <正确做法，最小可执行步骤>
- **验证**: <能钉住该修复的检查命令>

<!-- 自此往下追加真实条目。通用工具教训（edit 重复插入、pkill 自杀、set -e 陷阱等）已沉淀于 rules/common/edit-safety.md，勿在此重复。前项目 MediaServo 踩坑史见 .refinfo 归档。 -->

## PIT-1: 调研 prompt 里的"预设事实"会污染交付，必须标注为待验证假设 (2026-09-03)
- **症状**: ANGLE 调研 prompt 写"wgpu 无原生 GL 后端（verify）"——若代理不核查直接沿用，整个 legacy 方案会错判为"须自建 GLES 渲染器"（成本差一个后端的工作量）。实际 wgpu v30 README：原生 GL/GLES 后端 + `cfg(windows_angle)` 内置 ANGLE。同期两处自写文档把"Qt6 用 ANGLE"当事实传播（后证伪：Qt6 RHI 原生 D3D11）。
- **根因**: 把"我的推测"与"要求验证的点"混写在同一句里；且已入库文档中的行业断言未全部带当日引用。
- **解法**: 调研任务书中一切预设以"⚠ 原假设待证"独立成条，禁止内嵌为事实从句；文档内行业断言必须带"快照日期+来源"或 *UNCERTAIN* 标记（本仓 docs/reference 模板已含此纪律，本次执行到位——三处错误全部被复核线抓回）。
- **验证**: 交付前 grep 文档中"Qt6.*ANGLE|无原生 GL"类句式并对照 `evidence/2026-09-03-angle-integration` 的证伪条目；引用备忘录结论与一手 README 逐字比对。

## PIT-2: .gitignore 取反规则次序 bug + check-ignore -q 退出码误导 (2026-09-03)
- **症状**: `!.omo/omo.jsonc` 位于 `.omo/*` 之前，`git add .omo/omo.jsonc` 报"被忽略"；且修复前 `git check-ignore -q` 对含取反匹配的路径返回 0，易误判"仍被忽略/已被忽略"。注释文档（AGENTS.md"仅 omo.jsonc 入库"）长期是空头支票未被发现。
- **根因**: gitignore 语义 = 后匹配规则覆盖前者，目录排除 `dir/*` 必须写在取反 `!dir/file` **之前**；`check-ignore` 默认模式对"被排除规则匹配但与最终忽略状态矛盾"的路径退出码语义与直觉不符（-v 显示的最终匹配行才是事实源）。
- **解法**: 调整次序 `.omo/*` → `!.omo/omo.jsonc`；验证以 `git check-ignore -v`（显示最终命中行）和**实际 `git add` 成功与否**为准，不信 `-q` 退出码。
- **验证**: `git check-ignore -v .omo/omo.jsonc` 命中行必须是取反规则；`git ls-files .omo/` 有输出。任何新增 `!` 取反规则提交前跑一次实 add 演练。

## PIT-3: wgpu v30 升级破坏面——flags 默认值/Color 语义/draw 迁移三连（2026-09-03, S3/S4 实测）
- **症状**: ①debug profile 下 `cargo test` 在 lavapipe 上 panic `Unable to load cmd_begin_debug_utils_label_ext`（ash, panic-in-cannot-unwind）；②清屏色传 `0.05*255` 期望深蓝实得纯白 (255,255,255)；③`RenderPassColorAttachment` 编译报缺 `depth_slice`、`InstanceDescriptor` 无 `Default`、`enumerate_adapters` 变 async、`encoder.set_pipeline/draw` 不存在、`multiview`→`multiview_mask`、`create_slice`→`slice`、`map_async` 改 (mode, bounds, cb) 回调式、`Maintain`→`PollType::wait_indefinitely()`。另：winit `default-features=false` 裁特征时误删 `rwh_06` → `Window: HasWindowHandle` 不成立，create_surface 类型错。
- **根因**: ①`InstanceFlags::default()=from_build_config()`，debug 构建自动含 VALIDATION → 强制加载 VK_EXT_debug_utils，软渲染/旧 loader 无此符号；②v30 `Color` 分量语义 0-255→0-1（越界 clamp 成白）；③v30 与 WebGPU 规范对齐的大版本破坏（季度 pin 纪律的预期成本）。
- **解法**: `create_instance` 显式 `flags = InstanceFlags::empty()`（诊断校验留专项片）；清色按 0-1 传值；winit features 显式含 `rwh_06`；API 差异以**本地 registry 源码**为准（`~/.cargo/registry/src/.../wgpu-30.0.1`，比 docs.rs 快且真）。
- **验证**: `pixi run ci` 绿 + offscreen golden 3 测真机绿；**任何 wgpu 版本升级日 = 先 grep 本条 + 重跑破坏面清单**；实验定标优先于文档采信（Color 语义即实验确认）。

## PIT-4: cargo 集成测试 CWD=crate 目录 + Iter 构造探测法触发库内 debug_assert (2026-09-03, G1)
- **症状**: ①测试引用仓根 `testdata/x.glb` 相对路径全 NotFound（本地手跑 `pixi run cargo test` 时 CWD=workspace 根，单测过滤跑也同——但**集成测试二进制由 cargo 以 crate root 为 CWD 启动**，两种跑法路径语义不同曾给出假绿/假红组合）；②`Iter::<T>::new` 依次试类型当探测（返回 Option 以为安全）在 debug 构建直接 panic（库内 `debug_assert_eq!(size_of::<T>(), accessor.size())` 先行）。
- **根因**: ①cargo 对 tests/ 的运行目录是包目录，非执行 shell 目录；②Option 返回面之外库还带 debug_assert，"能返回 Option"≠"可安全探测"。
- **解法**: ①fixture 路径一律 `env!("CARGO_MANIFEST_DIR")` 拼接；②读 accessor 前用 `data_type()/dimensions()` 预检精确分派，类型不合不进构造函数。
- **验证**: `grep -rn '"testdata/' crates/*/tests/ ` 必须 0 命中（只许经 fixture() 助手）；降级链只许 match 分派形态。
- **另**: tests/ 目录 redirect 先行 `mkdir -p`（本轮第三次同族漏采，机械前置而非事后补）。

## PIT-5: wgpu 投影深度必须 [0,1]——GL 惯例矩阵 = 静默整帧裁剪（2026-09-03, G2 golden）
- **症状**: HeadlessBackend 画立方，vp 矩阵逐点验证全在 ±1 内、**零验证报错**、三角旧路全绿——但立方就是不出现（golden 断言中心=清屏色）。二元诊断（手工 identity 投影塞近处）才把问题锁到深度域。
- **根因**: wgpu clip 空间深度 **[0,1]**（WebGPU 原生约定），不是 GL 的 [-1,1]。照 GL 公式建的 ortho/persp 在近处产生负 z_ndc → 硬件整体裁剪。深度范围错在管线里**不报任何错**——几何死亡无声。
- **解法**: 投影矩阵一律建/校到 [0,1] 变体（glam 旧 `*_rh_gl` 是 [-1,1]，`perspective_rh`/手写 ortho 才是 wgpu 向）；camera.rs 顶注已锁死变体声明，REND-11/12 用**行为断言**（near→0/far→1）而非公式抄写。
- **验证**: 离屏 golden 家族存在即预防——任何矩阵/管线改动后 `pixi run cargo test -p visiaengine-render-wgpu` 立方 7 测必真绿（非 SKIP 路径；验证地点如实记录）。同类：`x+(y-x)*t` 在 t=1 不精确等于 y（lerp 端点快路，REND-16）；clippy 偏序取反禁令用 `partial_cmp` 正解。
