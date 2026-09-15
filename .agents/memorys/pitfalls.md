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

## PIT-6: MCP 桥裸 PATH 依赖 + opencode 配置键名静默忽略（2026-09-03, MCP 修复轮）
- **症状**: 三台 MCP 报 `Executable not found in $PATH: node/npx`；local-github 的 token 从未注入过（无报错，纯静默）。
- **根因**: ①桥配置假定 node 在 PATH，而本机遵 D5 纪律不装系统 node——环境里根本没注册 nodejs；②配置键写作 `env`，opencode schema（McpLocalConfig）正名 `environment`，未知键不报错只忽略；③桥脚本子进程还要 `npm`/`pixi`——绝对路径直启 node 也救不了这层。
- **解法**: nodejs 纳入 pixi 默认环境（单源不破）；`with-node.sh` 包装器统一注入 node 目录+~/.pixi/bin 后透传 exec；配置键改正。
- **验证**: `bash .opencode/with-node.sh node --version` 出号 + 桥拉起 timeout 存活；**任何 opencode 配置字段改动以 https://opencode.ai/config.json schema 为准，不凭记忆**；配置类修改需重启 opencode 才热载（运行会话用旧配置）。

## PIT-7: cargo-deny audit 实时联网拉库=本机网络抖动的假失败源（2026-09-11）
- **症状**: `pixi run ci` 时绿时红；红时全量测试/lint/clippy 皆绿，唯 audit 段 exit=1。
- **根因**: `cargo deny check`（advisories 部分）**每次实时 git-fetch** RustSec advisory-db；本机对 github.com 的 TLS 出网（gnutls_handshake）间歇失败 → fetch 失败即红，与代码/依赖变更零相关。
- **解法**: 认定"audit 红"前先 `grep 'failed to fetch advisory' ` 看日志；网络性失败重跑即可；真 advisory 才会输出 RUSTSEC-id。镜像 GitHub CI 端拉库成功率高，本机该症状会随网络环境波动。
- **验证**: `pixi run cargo deny check` 连跑两次结果一致才算依赖面定论；不一致=网络层。

## PIT-8: T2 像素断言首版谓词必翻车——几何覆盖/通道乘法链未先实测（2026-09-12, 批 4ab M2/M3 两轮）
- **症状**: M2 texture_render 首版：采样窗 (8,8) 全黑（脱四边形）+ 棋盘单像素族断言被线性滤波混合带打成橙灰；M3 capi 首版：`B<20` 谓词恒假（baseColorFactor 的 B=0.5 与红 texel 相乘→品红）且 ±1 四边形 21px 覆盖下红族核心仅 20 px。
- **根因**: RED 写断言时凭 uv/投影直觉推像素，未先测「四边形屏幕 bbox」「texel×base×shade 通道乘积」。像素测试的谓词是**三条乘法链的交点**，任一链猜错即假红。
- **解法**: 先跑一次性 dump 探针（区域步进打印 R/G 值或 bbox 统计），从实测值反推谓词阈值与窗口；探针删除、断言改写为**区域族统计**（count≥N 双族并存）而非单像素；fixture 尺寸让 texel≥16px 屏宽（滤波混合带吞 <8px 纯度核心）。
- **验证**: 新像素断言首次运行前先 `-- --nocapture` 看数值带；计划期 [四个不装/诚实面] 禁止用调阈值掩盖断链——判据须保留判别力（uv 断链→某族恒 0 的不变式，M3 红=20/紫=304 即链路未通的证据形态）。

## PIT-9: L2 smoke 段不入本机 ci 链=窗口格式类回归的假绿窗口（2026-09-14, 4c 七路首爆）
- **症状**: 三窗口 example 在 I3 表面格式对齐后仍直挂 `render_view`（内部写死 Rgba8Unorm），x11 实配 Bgra8UnormSrgb → wgpu validation panic；该 bug 跨 3 轮 T1 全绿存活（ci 链不含 smoke）。
- **根因**: 本机门禁=`pixi run ci`（fmt/lint/test/audit/gates），smoke-* 七路=手动段——镜像日现役化前无人跑；「六路 smoke ✓」基线声明=上次手跑日期的快照，非持续断言。
- **解法**: ① 任何触 render 路径的轮次，合并前必须实跑七路（DISPLAY=:0 直跑，勿套 timeout——timeout 不解析 shell 函数/alias 版 pixi，用绝对路径或裸跑）；② example/attach 消费格式一律 `config.format` 直传 render_view_format，禁走写死便捷口。
- **验证**: `for t in smoke-*; do pixi run $t; done`（真执行≠编译，E3D:D2 纪律）；镜像日 ci.yml L2 段接线后本条 ① 转机器门禁。

**来源**: 4c K3 smoke-bench-twin 接入时全量复跑首爆。


## PIT-10: 场景人检断言「前景该是受光地」=几何想当然（2026-09-14, 批 5 E901）
- **症状**: E901 twin_city 首版前景地面整片半影调（obs 0.58-0.63 vs 预期 0.86），判为 shadow 数学 bug → 三轮探针（改 dir/slope/None 对照）都指向「参数没错」。
- **根因**: 场景本身合法——13m 楼 × 低日角（ld 水平分量 0.5/0.79≈0.63）→ 影长 ≈10m；16×16 密排城的影带首尾相接，**前景本就是受影区**。「受光地应在此」是屏幕直觉，不是光线-几何推演。
- **解法**: 像素断言失配时，先做光源几何计算（影长=Σh·|l.xy|/l.z 与城占位关系）再进参数二分；场景演示例的影子用**族计数**（本例 dark=影地+楼暗面族）不做位置点断言。
- **验证**: 任何新增渲染 example 的人检图跑 `look_at` 式视觉复核（本次靠它发现真缺陷=底图被城压住→右偏置修复）；纯数值断言过≠图对。

## PIT-11: API 台账以名判义不读实现=计划前提整块塌方（2026-09-14, Qt 轮 v1.0 REJECT）
- **症状**: 计划断言「14 入口零 resize 通道，需新设第 15 符号 CAPI-10」；Momus 审核读实现推翻——`visiaengine_viewport(w,h)`（第 14 入口）即完整 resize 通道（ffi.rs:441 0 维拒 → engine.rs:405 w/h+backend.resize+Window sw.resize）。若按假前提开工=多一冗余符号+三锁返工。
- **根因**: 取证只 grep 了**入口名集**，对 `viewport` 按字面名判为「视口查询」——名≠义；能力面台账的正确构建=读实现调用链，不是读函数名。
- **解法**: 任何「能力缺失/需要新入口」断言，必须先 grep 该能力的**动词实现**（resize/reconfigure/set_size 字样）+ 读一条从入口到效果体的调用链再下结论。
- **验证**: 计划 §0 锚点必须含文件:行到实现体（非仅符号表）；Momus 审核负责任的点名为「引用实测」正是本条的机器面。

## PIT-12: conda-forge Qt6 双坑——包名正字法与 QX11Application 缺物（2026-09-14, Qt 轮 Q2）
- **症状**: ①`qt-main` 装出来是 **Qt5**（5.15.15，lib/cmake 只有 Qt5*）；②改 `qt6-main` 后 Qt6Config/xcb 插件俱在，但 `<QNativeInterface/QX11Application>` 头**不存在**（x11extras 未打包）——widget 取 Display* 的公开 API 路线断头。
- **根因**: conda-forge Qt 系包名历史分裂（qt-main=Qt5 遗产名，Qt6=qt6-main 前缀式）；「pixi search 模糊命中」包名坑族第二例（第一例 xorg-xvfb-server）。QX11Application 属 qtx11extras 模块，qt6-main 裁剪不含。
- **解法**: 宿主 Xlib 自持 `Display*`（XOpenDisplay/XSync/CloseDisplay 三调用面）+ winId() 的 xid **跨 X 连接共享**（attach 合同 demo_x11 同机件实证）；QT_QPA_PLATFORM_PLUGIN_PATH 显式钉 `$CONDA_PREFIX/lib/qt6/plugins`。
- **验证**: `ls $CONDA_PREFIX/lib/cmake/Qt6/Qt6Config.cmake && find $CONDA_PREFIX/include -name QX11Application`（后者应空→触发 Xlib 路线）；`pixi list -e qt-spike | grep qt6-main` 确认非 qt-main。

## PIT-13: C ABI 手写头缺 extern "C" 护栏——纯 C 测试全绿掩盖 C++ 必炸（2026-09-14, CMake 层 C2）
- **症状**: `visiaengine.h` 零 `extern "C"` 包裹；C demos（E701/702）与 14 入口 nm 门禁三轮全绿；C++ 消费者（ctest 探针 consume_capi.cpp）首链即 `undefined reference to 'visiaengine_abi_version()'`（C++ mangling）。
- **根因**: 消费者测试面与 ABI 声明语言**同构偏差**——C 头配 C 测试=自证循环；C++/Rust FFI 才是真实宿主语言分布（Qt/Flutter/C# 全走 C++ 编译单元）。
- **解法**: 手写头首行区加 `#ifdef __cplusplus extern "C" {` 护栏；消费者探针矩阵必须含 **≥1 个 C++ TU**（本仓=cmake/probe ctest 常驻）；新 C ABI 面的验收锚从「C demo 跑通」升「C+C++ 双语言跑通」。
- **验证**: `grep -c 'extern "C"' crates/visiaengine-capi/include/visiaengine.h` ≥2；`pixi run ctest --test-dir target/cmake-bare` 1/1。

## PIT-14: 环境态污染门禁断言——激活壳 PATH 让「强制模式」假通过 + 报文折行让全句 grep 漏失（2026-09-14, C1 审核期/C3）
- **症状**: ①`-DRUST_SDK=SYSTEM` 在 pixi 激活 shell 下 find_program 命中 conda 树内 cargo，判成 [system] 通过——「强制系统」语义在该环境不可能为真却无报错；②逃生舱 FATAL 断言 grep 全句「无 capi 产物」漏失——cmake 把报文折成两行「无 capi\n产物」。
- **根因**: ①解析器的查找域=当前环境，而环境的 PATH/PREFIX 受激活态污染——「强制 X」若不显式排除非 X 来源即名存实亡；②终端/日志折行对全文匹配是破坏性的。
- **解法**: ①强制模式加**反污染双滤**（find_program 结果前缀比对 CONDA_PREFIX/.pixi、Qt6_DIR 同检，命中=拒收报文指名）；②门禁断言一律用**跨折行稳定短语**（"无 capi" 级），负路径断言同时接受 rc 与报文双条件。
- **验证**: `pixi run bash scripts/cmake-smoke.sh`（双负路径报文在断言列）；SYSTEM 拒收报文在本机激活态必现。

## PIT-15: enable_testing() 晚于 add_subdirectory=该目录 add_test 静默丢（2026-09-15, S4 真机实锤）
- **症状**: platform/qt 的 example_E703 add_test 注册后 ctest -N 永不见之，亦不产 platform/qt/CTestTestfile.cmake；零报错零提示。
- **根因**: 根门面 enable_testing() 排在 add_subdirectory(platform/qt) 之后（C2 期 probe 恰在其后故未暴露）；CMake 对该顺序问题不告警。
- **解法**: enable_testing() 必置于**所有** add_subdirectory 之前（已前移+注释钉死）。
- **验证**: `ctest --test-dir target/qt-build -N | grep -c "Test #"` == 注册数（13）；`ls target/qt-build/platform/qt/CTestTestfile.cmake` 存在。

## PIT-16: 两个「作用域/形状」坑——function 内 enable_language 不外传；presets 字段以随包 schema 为权威（2026-09-15）
- **症状**: ①function() 内 enable_language(C) 后，函数里 add_executable(.c) 报「can not determine linker language」；②CMakePresets testPreset 标签过滤字段四连猜全拒（excludeLabels/labelExclude/exclude.label 数组/filter.exclude.labels）。
- **根因**: ①enable_language 的编译器变量落 function 作用域，出函数即丢——**语言启用必须文件/目录作用域**；②presets v6 无标签过滤、v7 正形= `filter.exclude.label`（**字符串正则**）；记忆/搜索均不可靠。
- **解法**: ①enable_language(C) 上提至 .cmake 文件顶层 if(VISIAENGINE_EXAMPLES) 块；②离线正本读 `$CMAKE_HOME/share/cmake-*/Help/manual/presets/schema.yaml`（或同目录 exclude-properties.rst）；外网断联（context7/webfetch 双失）时此路 0 成本。
- **验证**: configure 过 + `ctest --preset qt-pixi` 计数=排除 display 后 7；schema.yaml grep "``label``"。

## PIT-17: cargo harness 多线程共享 pid——tmp 路径唯一性 pid 不够须 pid+纳秒（2026-09-15, S1 闪断）
- **症状**: attr_three_types 首跑 FAILED、单跑/复跑绿（flaky 假象）；同 tmp 前缀的 3 测试并行。
- **根因**: `format!("ve-attr-{tag}-{}", process::id())` 在**同一测试二进制**内所有线程 pid 相同 → 多线程并行 `fs::write`（truncate-then-write 非原子）互踩，读方可见半空文件。
- **解法**: 唯一性 = pid + SystemTime 纳秒双缀（engine 内联与 attr_ffi_spec 两处同款）；或每测试专属 tag。
- **验证**: `for i in $(seq 6); do cargo test -p visiaengine-capi | grep -c 'test result: ok'; done` 全等（6×6 实测绿）。

## PIT-18: `pixi run` 包装验证 ≠ 用户面验证——IDE 直调构建首爆 rustc-missing（2026-09-15, 用户实锤）
- **症状**: 全部经 `pixi run cmake --build` + ctest 验证「13 条全绿」后，用户 VS Code（CMake Tools 直调 /usr/bin/cmake，无 pixi PATH）构建 E301 → `error: could not execute process rustc -vV (never executed)`。
- **根因**: conda cargo 按 PATH 找兄弟 rustc；`pixi run` 激活恰好把 rustc 塞进 PATH=每步都绿。构建期 COMMAND 的运行时环境与配置期解析是两回事——**验证通道与被验证据通道重合时，环境差异盲区全掩盖**（verification-honesty 的构建面投影；与「未激活 shell 直调 conda cargo 假 rustc-missing」旧案同源反向）。
- **解法**: cargo 调用点 `${CMAKE_COMMAND} -E env "PATH=<cargo 同目录>:$ENV{PATH}"` 前缀（cargo-step + examples-step，SYSTEM 真系统件无害）；IDE 用户面=裸环境，一切构建门禁必须以裸 PATH 复测。
- **验证**: `env PATH=/usr/bin:/bin cmake --build target/cmake-bare --target visiaengine-examples-step` 过；已固化为 cmake-smoke 第四态双 target（守卫回归锁）。
