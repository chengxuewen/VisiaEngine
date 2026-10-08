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

## PIT-19: pixi 文件 clobber 致 conda 版 Xvfb 恒炸 XKB（2026-09-16，S0 spike 实锤）
- **症状**: `pixi add` 后 default 环境 `.pixi/envs/default/bin/Xvfb :77 …` 起服即
  `XKB: Failed to compile keymap` → `Failed to activate virtual core keyboard: 2`；
  `share/X11/xkb/` 目录仅剩 `compiled/`，rules/keycodes/symbols 全失踪。
- **根因**: `xorg-xvfb-server` 包携文件 `share/X11/xkb/compiled/.keep` 与 `xkeyboard-config`
  的树路径冲突 → pixi 0.78 clobber 处置=把 xkeyboard-config **整树重定向**至
  `share/xkeyboard-config-2/`（conda-meta 出现 `__clobbers__/xkeyboard-config/share/X11/xkb` 标记），
  Xvfb 按编译期正位找不到 keymap。host-spike 环境同款；历史未暴露=本机 smoke-x11 恒走 :0 分支。
- **解法**: 环境为本机态，重建后重放一次合并：
  `cp -a .pixi/envs/<env>/share/xkeyboard-config-2/. .pixi/envs/<env>/share/X11/xkb/`。
  display 测试子态优先级=系统 `command -v Xvfb`（apt CI 免疫）→ conda 兜底（须随附修树重放）→ 皆无=SKIP note。
- **验证**: `Xvfb :77 -screen 0 1024x768x24` 起服 2s 后 `kill -0` 存活；`DISPLAY=:77` 跑 SDL3×attach spike 绿。
  防复发检查: `ls .pixi/envs/default/share/X11/xkb | grep -q rules || cp -a …xkeyboard-config-2/. …X11/xkb/`。

## PIT-20: 「幽灵 API」族——凭记忆书写的接口名/参数名 ≥3 例（2026-09-16 立族）
- **症状**: 计划/代码引用不存在的外部接口：`corrosion_add_test_crate`（上游 #13 自 2018 从未实现）、`CORROSION_CARGO` 用户缓存（0.3 起即移除）、`crates_only`（真名 CRATES）、conda 无 corrosion 包（实有 0.6.1）——每次都在执行带撞墙或静默绕路。
- **根因**: 把「记得它存在」当「验证过名字」。
- **解法**: 凡外部 API 句柄（函数/参数/包名/文件路径）落纸当场抓正本（docs/源码 grep/API 实测），引用带 verbatim 片段；无锚=标 [UNCERTAIN] 不许进计划正文。
- **验证**: 本轮教训源自动画：写计划前 context7/源码/`pixi search` 三查已在 Momus 两轮救回 4 错；残留错=未查句。

## PIT-21: 编辑工具插入式改写的「新旧并存」残留 + grep 断言的 BRE 假绿（2026-09-16 四连实锤）
- **症状**: 同文件批量 replace 只吞首锚行，被替换块「第二行旧内容」存活 → 僵尸行×4（IDE 终态双行/门禁双行/if 行被吞致 || 悬空/示例双锚）；同族：`grep 'a|b'` 无 -E 时交替失效=恒零匹配=「清零断言」假绿。
- **根因**: 插入式改写未声明 end 范围；断言谓词用 BRE 裸 |。
- **解法**: ①插入式改写必 `pos+end` 吞全旧范围，改毕 `sort|uniq -d` 查重 + 旧名 grep=0；②交替断言一律 `grep -E`/`grep -c` 先证「能命中正例」再断零；③守卫「如果报错再补」句式=条件分支悬念，改「出生预置」消灭分支。
- **验证**: `bash -n` + 手工同参复跑当带；S3/S4 各撞一次全部当带收（未过夜=合格，教训=模式已在 edit-safety #18 固化）。

## PIT-22: 「验证通道≠用户通道」环境盲区第二见——DISPLAY 族（2026-09-16，用户实锤）
- **症状**: 全部真窗验证（S0 spike / smoke-qt / cargo-run 抽验）在 agent 工具 shell 里 `export DISPLAY=:0` 跑绿；用户 VS Code（vscode-server 进程树）点 cargo-run_* 即 winit `neither WAYLAND_DISPLAY nor DISPLAY is set` rc=1。
- **根因**: PIT-18 同族泛化——工具 shell 与用户 IDE shell 是两套环境；手工 export = 给验证通道私加用户没有的补给（PATH 族当时是 conda 激活，本次是显示变量）。窗口类断言的绿只有在**用户环境形**里取得才算数。
- **解法**: ①环境敏感步骤（GUI/剪贴板/音频/网络代理面）一律走「启动器包装 + 探测回退 + stderr 播报走了哪条路」（run-gui.sh 形）；②交付窗口功能前须在**未加料** shell（`env -u DISPLAY` / `env PATH=/usr/bin:/bin`）复跑一次；③IDE 侧承诺面（测试面板/Run 按钮）属 T3，机器不可达就明说等人验。
- **验证**: `env -u DISPLAY -u WAYLAND_DISPLAY cmake --build target/cmake-bare --target cargo-run_E501_shadow_demo` 存活至 timeout(124)=回退链生效；裸 cargo run（无包装）必须仍报清晰错=哨兵面未坏。

## PIT-23: CMake/ctest 参数与断言的三副「恒真/吞噬」形（2026-09-16，E801 死挂案）
- **症状**: ①`cmake_parse_arguments` 单值关键字收 `"--frames;3"` 实收仅 `--frames`（列表在 ${ARGN} 拍扁成双元素，单值口吞首丢余）→ ctest 例子落入交互态，cmake-smoke 死挂 15 分钟；②`ctest -R <不匹配>` = "No tests were found" **exit 0**，转发壳恒绿；③（PIT-21 已录 BRE 裸 `|` 恒假，同族互引）。
- **根因**: 三形共性=「看起来在传/在断，实际静默丢/恒过」——CMake 列表语义与 ctest 空集哲学都和直觉相反。
- **解法**: ①列表参数一律 multivalue 口（`cmake_parse_arguments(prefix options oneValue multiValue)` 第四席），生成物以 `CTestTestfile.cmake` 实行为准回看；②ctest 包装器必带「≥1 条被选」断言（`--show-only` 预计数或输出 `out of [1-9]` grep，smoke-rs.sh 先例）+ `--timeout` 永装（挂死=红而非黑洞）；③任何「=0 即绿」的门禁先做一次**能命中正例/能触发负例**的自证。
- **验证**: `grep -o 'E801_sdl_window" "[^)]*' target/*/examples/c/CTestTestfile.cmake` 见 `"--frames" "3"` 两位齐；注入不存在的 -R 转发一次必红。

## PIT-24: 共享几何件的绕序哑雷——cull 关使 winding 成为只被拾取读的暗观测（2026-09-17，E402 带）
- **症状**: `unit_box_mesh` 三消费者（instances/bench/twin_city）渲染全正常、golden 全绿；
  E402 拾取链第一次消费即中心射线必 miss——竖直向下射线打盒顶不中、自下向上反中。
- **根因**: 出生绕序与声明法向**全局相反**（几何 cross=−nrm）。GPU 路 PrimitiveState 未设
  cull（wgpu 默认 None）→ 双面都画 → 渲染零症状；`pick_meshes` 正面规则（REND-23）读的
  恰是 winding——同一份几何在两条消费路上语义相反。凡"渲染件复用进拾取/gpu-skinning/
  装 cull 后的管线"都是此雷的引信。
- **解法**: 索引镜像翻正 `[b,b+1,b+2][b,b+2,b+3]`→`[b,b+2,b+1][b,b+3,b+2]`（一次改，
  全消费者受益；cull 关下像素逐位不变=构造证明，162 passed 零回归实锤）。公共几何件定义
  处必须写明 **绕序=外法向 CCW** 契约（已入金训注）。
- **验证**: 新共享几何出生即跑一条「正面射线必中/背面射线必不中」双向断言（E402 门的
  corner-canary 形）；`grep -n 'CCW' crates/visiaengine-render-wgpu/src/offscreen.rs` 契约行在位。

## PIT-25: 工具链静默断链——包名漂移 + 缺席 continue 恒真（2026-09-17，点云带 G3 挖出）
- **症状**: `bash scripts/bench.sh` 常年 `BENCH ✓/exit 0`，但 resources/bench/*.json 自 S1 迁移后两轮未更新（时间戳停在 9-14）。
- **根因**: 双重静默——①spec 里 `-p visiaengine-render-wgpu --example E601…` 在例子迁 `examples/rs`（包名 examples）后 cargo 直接报错；②脚本对"无 RESULT 行"只 `continue` 不置错，管道+`|| true` 吞 rc（edit-safety #19 同族）→ 死链被恒真绿掩盖。
- **解法**: 包名改 `-p examples`；缺席计数 `MISSING>0 → exit 1`（工具链故障与性能劣化分离：后者红字 exit 0 维持观测档）；新例 bench_pcl 挂同链。
- **验证**: `bash scripts/bench.sh` 三 json 时间戳当日 + 故意 `sed` 错包名一次必 exit 1（正/负例自证，PIT-21 纪律）。

## PIT-26: from_raw_parts(_mut)(NULL, 0) 即 UB——debug 前判实锤 (2026-09-17)
- **症状**: capi get_clips 的 `buf=NULL` 纯计数路在 debug 测试中 panic「unsafe precondition(s) violated: slice::from_raw_parts_mut requires … non-null」（release 静默）。
- **根因**: Rust 新版 core 对 from_raw_parts 加了 debug 前判（null/对齐/isize::MAX）；`m=0` 也算构造空切片的非法指针来源。既有 add_points/add_mesh 未踩中是因为 NULL 检查在构造切片**之前**就早退了。
- **解法**: 构造前 `if m > 0 {}` 守卫（NULL∧任意长、任意∧0 长全躲开）；或先 is_null 早退再构造。
- **验证**: ffi_spec `set_get_clips_full_domain_roundtrip_truncation` 的 NULL-buf 段 + `cargo test -p visiaengine-capi`（debug 档即炸回归）。
- **禁止**: 任何 FFI 口对可能为 NULL 的 buf 无条件 from_raw_parts（哪怕长度为 0）。

## PIT-28: 标签居中的 px 平移只能进 dx 域——anchor 是世界坐标 (2026-09-18)
- **症状**: E205 三段标签金色副标完全不渲染（hist 无黄色系），白/青在但偏移异常。
- **根因**: `anchor_x = cx - pen/2` 把**像素** pen 从**世界**锚点减（-4−50 世界米=出屏）。capi `add_label` 正本式=center 对齐全部落在 dx(px) 分量：`dx = top_left_px - pen/2`，anchor 原样世界位。
- **解法**: anchor 只进世界坐标，一切 px 语义（含对齐平移）住 metrics/dx 域（vs 统一乘 px_scale）。
- **验证**: 三口互证（capi engine.rs / hpp 转发展 / E205 例）grep `pen / 2.0` 必须全部出现在 LabelMark 第 4 参数组内，第 1 参数组内出现=红。

## PIT-29: orbit 构造传度数=perspective 吃弧度——三例带病现行 (2026-09-18)
- **症状**: E303 顶视 OK 但主视地面仅占 51%/构图怪；E302 headless bright=96（阈 40 擦过）；E501 窗面同雷（无人细看）。
- **根因**: `CameraRig::perspective`/look_at 默认 `FRAC_PI_3`=**弧度**制；`orbit(...)` 调用位曾传 46/55/60（当度数）→ tan(23 rad) 负值投影翻转/压扁。lavapipe 不报错=静默烂画面。
- **解法**: 全用弧度（0.802_851=46°、FRAC_PI_3=60°、0.959_931=55°）；`orbit` 文档钉死单位并注前科。修后 E302 bright 96→1987、E501 dark=19642 精确、E303 构图成立。
- **验证**: 例面 grep `orbit\(.*[0-9]\.[0-9], [0-9]{2}\.0` 出度数量级（>7 的 fov 位）=红；窗例像素门（E302/E303 headless 断言族）跑绿。
- **教训**: 像素门没看过的窗面=没验过的面——T3 人验不可达时，headless 结构断言必须覆盖窗例的同一帧形（本带双保险成例）。

## PIT-29: 正交相机的缩放唯一定义=half-width，dist 字段是摆设（2026-09-24 E206 三修第二见）
- **症状**: 滚轮缩放"无效"——rig.zoom 与 rig.dist 双双更新但画面纹丝不动。
- **根因**: 双叠加。①`look_at()` 构造默认 zoom=1.0，而渲染的 ortho half-width 消费的是硬编码 tile_w 常量（rig.zoom 从未被读）；②正交投影下 dist 不影响画面大小（透视才有"离远变小"），改 dist=空操作。
- **解法**: 滚轮→rig.zoom（正交唯一缩放语义），渲染 half-width 消费 rig.zoom；clamp 域按场景量纲重算（3857 米世界=8km..500km，非 E204 米级的 2..60）。
- **验证**: `grep -n 'rig.zoom' examples/rs/E206_tile_viewer.rs`（消费点在场）；滚轮实测画面变化。
- **禁止**: 正交例把 zoom/zoom_scale 写到 dist 上；从 look_at 构造后假设 zoom 已是可用视野。

## PIT-30: 窗例"交互=后 Phase"砍窗面/砍交互=C15 双模违约三连（2026-09-24 E206 实测三案）
- **症状**: 用户跑 cargo-run_E206：①无参立即退出（窗是 println 桩）→②拖拽滚轮无反应（Redraw 每帧硬编码重建机位，事件改的字段不在结构体/不被消费）→③单瓦片内容过简（"最小可跑"≠"可演示"）。
- **根因**: 交付带把"静态/最小"当收口标准，而双模纪律（C15）的零参窗=**常驻+可交互**是独立验收面——smoke/ctest 绿救不了人验红；且"Phase 1 再接"被当成了砍实现的许可而非分期计划。
- **解法**: 窗例出生带真窗+真交互（E204 骨架照搬 ~150 行）；交互相机=TileApp 持久化 rig 字段（Redraw 消费 self.rig，禁每帧重建）；演示内容按"可演示"标准（3×3 邻域+瓦片缝描线）非"可运行"标准（单瓦片）。
- **验证**: Xvfb 存活探针（timeout 22s 击杀=常驻）+ 人验拖拽/滚轮实际生效。
- **禁止**: 窗例无参路 println 后 Ok(());Redraw 路径内 new 硬编码 rig 覆盖交互态;把交互承诺写进注释当已交付。

## PIT-31: ffi_spec 集成测试 cwd=crate 目录（2026-09-24）
- **症状**: capi 新口 load_mvt_dir 集成测试返回 -1（VE_ERR_ARG），期望 9；同测试本地 `cargo test -p visiaengine-capi --test ffi_spec` 也红。
- **根因**: cargo 集成测试（tests/ 目录）执行时 CWD=`bindings/c/visiaengine-capi/`（crate 根），相对路径 `resources/data/tiles` 解析到 `bindings/c/visiaengine-capi/resources/data/tiles`（不存在）。FileSource 路径参数是运行时字符串，非编译期宏。
- **解法**: 测试内路径统一用 `env!("CARGO_MANIFEST_DIR")` 上溯仓库根：`let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR")); while !p.join("Cargo.lock").exists() { p.pop(); }`（ffi_spec.rs 内已有此模式可照抄）。
- **验证**: `cargo test -p visiaengine-capi --test ffi_spec` 19/19 passed。
- **禁止**: 测试内用相对路径 `resources/...` 指向仓库根 fixture（cwd 不确定）。

## PIT-32: ureq 全量依赖把 getrandom 拖进 wasm32 目标（2026-09-24）
- **症状**: 给 io-tiles 加 `ureq`（native-only HTTP 源）后，wasm32-unknown-unknown 目标编译失败：`the wasm*-unknown-unknown targets are not supported by getrandom, you may need to enable the "js" feature`——wasm 面本不该有 ureq，但 Cargo.toml 无条件依赖 = wasm 也编译它。
- **根因**: ureq → rustls → ring → getrandom。getrandom 0.2/0.3 wasm32-unknown-unknown 需显式 `js` feature；rustls 树里没加，io-tiles wasm 消费者也没加 → 编译期 compile_error! 宏触发。
- **解法**: `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` 把 ureq 隔离到 native 段；wasm 侧给 `HttpSource` 加 typed stub（`load()` 返回 `Err(Unsupported)`）。wasm 真实 HTTP 需宿主 JS fetch + bytes 注入口（Phase 2 wasm demo 带）。
- **验证**: `pixi run web-check rc=0`（MIRROR 3/3）。
- **禁止**: 向 wasm 可见 crate 直接添加含 getrandom/ring/rustls 的无 target-gate 依赖。

## PIT-33: 全屏 pass 后 depth 生命周期与 wgpu30 用域法（2026-09-28，EDL 带）
- **症状**: EDL 后处理需在同一 encoder 内采样主 pass 的 color+depth；直接把 swapchain view 当采样源（无 TEXTURE_BINDING）或同 pass 内"写 color 又采 color"即翻车。
- **根因**: wgpu 30 用域法：①attachment 与 RESOURCE 在同一 pass 互斥（color 同时作为 write target 与 sample source 非法）；②swapchain 视图不带 TEXTURE_BINDING；③depth 默认 StoreOp::Discard，后 pass 无从读。
- **解法**: 中间纹理缓存（键=(w,h,format)，resize 重建）承主 pass 输出 + depth 改 Store；EDL pass 独立成 pass（采 color+depth、写最终 view）；OFF 路径整段跳过=逐位零回归。多视口路径本带豁免（条款 WGPU-33 记录）。
- **验证**: `cargo test -p visiaengine-render-wgpu --test edl`（OFF 复渲逐字节等 + ON 像素差异 + OFF-after-ON 再等）。

## PIT-34: LRU/状态双账本失同步——state() 说谎 Done 而字节已被逐出 (2026-09-28，审修 #3)
- **症状**: pump() 先 cache.insert 后标 Done；LRU 逐出字节后 state() 仍报 Done → decoded() None → 批重建静默 continue = 永久渲染洞，且 begin() 跳过 Done 永不再武装。
- **根因**: 双账本（pump_state map + LRU bytes）无一致性维护；Done 是稳定态、无人复查字节还在不在。
- **解法**: 显式 `evict_done(ids)`（清 state+bytes+decoded 三账）+ 只读探测 `decoded_cached(id)`；消费环（engine render loop）每帧对账 `state()==Done && !decoded_cached()` → evict_done 重武装。单点真相原则：发现双账本即配对账函数。
- **验证**: `cargo test -p visiaengine-io-tiles --test tile_pump lru_eviction`；引擎侧由批重建路径消费（decoded==None 不得渲染）。

## PIT-35: env 代理在库层也吃 loopback——ureq 跟随 http_proxy 把 127.0.0.1 转 502 (2026-09-28)
- **症状**: E206 --source 泵 0/9 Done；同 URL curl 直连 200、走 env proxy 502（10.144.0.1:7897）。
- **根因**: 既有教训只覆盖 curl（edit-safety「curl 必须 --noproxy」）；HTTP 客户端库（ureq 3.x）默认 Proxy::try_from_env 同样中招——修调用方 curl 救不了库。
- **解法**: **修在源头**：HttpSource::load 内 `uri_host_is_loopback(root)` → proxy=None，否则 try_from_env；5s 全局超时同点设。库层网络代码一律自带 loopback 免代理 + 超时，宿主环境不背锅。
- **验证**: `cargo run --example E206_tile_viewer -p examples -- --source http://127.0.0.1:8932/data/tiles`（配 python http.server）→ 9/9 Done。

## PIT-36: python 模块级定义顺序——main() 执行时 TEMPLATE 尚未定义 (2026-09-28，画廊带)
- **症状**: gen_gallery.py 尾部 append 的 DETAIL_PAGE 模板在 `if __name__ == "__main__":` 之后，pixi run gallery 报 NameError。
- **根因**: 追加式编辑把常量放在了入口守卫后面；模块自上而下执行，main() 调用时 TEMPLATE 未绑定。
- **解法**: 常量/模板一律置于入口守卫之前；追加编辑后先 `python3 -c "import ast; ast.parse(open(f).read())"` 再跑。
- **验证**: `python3 -c "import ast; ast.parse(open('scripts/gen_gallery.py').read())" && pixi run gallery`

## PIT-37: 例子重构丢 origin——静态表重构时逐 marker 数据静默归零 (2026-09-28，E305 窗面带)
- **症状**: E305 窗面重构（build 表 + group 分离）后 cyan 像素门 2 vs 阈 20——轨迹 24 枚标记全画在原点。
- **根因**: build() 里 `let _o = anim_origin(...)` 算了 origin 但存表时只存 (mesh,material) 二元组，group() 统一 origin:[0;3]——数据在"算了"与"用了"之间断链。
- **解法**: 表结构带数据（`Vec<((id,id), origin)>`），group() 逐项消费；**像素门当场拦截**（这正是门禁存在的意义——第 N 次兑现）。
- **验证**: `cargo run --example E305_entity_anim -p examples -- --frames 24`（cyan>20 断言）。

## PIT-38: winit match 臂内 if = collapsible_match clippy 拒式——用 match guard (2026-09-28)
- **症状**: 窗例 Esc 处理 `WindowEvent::KeyboardInput{..} => { if pressed && key==Esc {exit} }` 触发 clippy collapsible_match 三连红。
- **根因**: match 臂内首行 if 合并形态=match guard（`arm if cond =>`），clippy 1.98 按可读性建议直接红。
- **解法**: 抄 guard 形（`KeyboardInput{..} if state==Pressed && key==Escape => exit`）+ 保留 `_ => {}` 全匹配臂。附带：winit 例**每次新建必带 imports 清单**（ApplicationHandler/WindowEvent/EventLoop/Window/WindowId——E304/E305 两次各漏一次浪费两轮）。
- **验证**: `cargo clippy --example <name> -p examples --all-targets -- -D warnings`。

## PIT-39: 窗例"第一帧"无机器守卫——三连黑屏全靠人工（2026-09-29，E305/E508/E505 家族）
- **症状**: 三个窗例先后黑屏/空壳（E305 无循环、E508 窗面无键、E505 无 redraw），全部 cargo test/ctest/ci 绿，全靠用户人验抓出。
- **根因**: 机器面（assert 路）与人验面（窗路）生命周期完全不同——窗例的"swapchain 有没有第一帧"无任何 CI 断言；Wait 模式 + resumed 不请求重绘 = 永黑。
- **解法**: scripts/window-probe.sh（零参跑例→Xvfb 截屏→亮度比断言）+ window-probe-all.sh 批量（20 常驻窗例）+ **先截屏后杀 app**（root 截屏依赖窗口存活——首版顺序反了截到全黑）。自退例 skip（golden 守它们）。
- **验证**: `bash scripts/window-probe-all.sh`（20 例全 ✓）；破坏探针实测（注释首帧请求→红 rc=1）。

## PIT-40: 注册表 disp/args 必须成对——半边缺失两连（2026-09-28/29）
- **症状**: E403 漏 _args（display 例零参常驻→ctest 挂死 10 分钟）；E508 漏 _args（headless 例误注册→DISPLAY 死）；两次均 ci/ctest 才现形。
- **根因**: cmake 注册表 _disp_X ON 与 _args_X "--frames;N" 是一对语义绑定（disp=窗形态 args=自动化形态），单边注册=例面形态错位。
- **解法**: 新例注册时同 commit 内两行一起写；验证=`ctest -LE display -N | grep <名>` 选中且 `--frames` 在 argv（冒烟跑通即证）。门禁化候选：R9 双向校验扩展 args 检查。
- **验证**: `grep -A2 "_disp_${NAME}" cmake/VisiaEngineBindings.cmake | grep _args`（成对在场）。

## PIT-41: 多块 patch 脚本=截断雷区——本轮四连（2026-09-30，io-gltf 清空事故）
- **症状**: 多 patch python 脚本中途 assert/NameError 死亡时，已写盘内容不一而足（有的块落了有的没落），最恶性一例 io-gltf lib.rs 被清空 0 字节（open(p,'w') 截断时机踩中）。
- **根因**: 「read→replace→write」与「write 在最后」混用；同一脚本内多个 patch 共享外层 `s` 变量时中途死亡=半新半旧状态不明；`open(p,'w')` 的截断发生在 write 前一瞬但变量链断裂时结果不可预测。
- **解法**: ① patch 函数体**每次独立 read→write**（无外层共享 s）；② replace 结果先存 `ns` 变量再 write；③ 超过 2 块的改动**拆成多次工具调用**（每调用 1-2 patch）；④ 任何 patch 后立即 `grep` 验证关键字段在场。
- **验证**: `wc -l` 对比改动前后 + `grep -c` 关键锚在场 + cargo check。
- **阻塞条件**: 同一脚本 >2 patch 未拆分；patch 后未 grep 验证即继续。

## PIT-42: WGSL uniform 布局三雷——闭包/let-if/array stride（2026-09-30，I 带三连）
- **症状**: mesh.wgsl 加 env_sh 段三连崩：①`|i: u32| -> ...` 闭包语法 naga 拒（WGSL 无闭包）；②`let x = if cond {...}` 表达式形拒（WGSL let-if 不存在，需 var+语句）；③`array<f32,27>` uniform 校验拒（uniform 地址空间 array stride 必须 16 对齐）。
- **根因**: Rust 心智直译 WGSL——三者都是 Rust 有而 WGSL 无的形态；stride 规则是 GPU uniform 布局的硬约束（vec3=16B、array<f32,N> 也要 16 对齐）。
- **解法**: 闭包→模块级 fn；let-if→`var x = ...; if cond { x = ...; }`；数据数组→`array<vec4<f32>,N>`（RGB+pad）或手工 16B 对齐排布。View 块尾缀扩展后总 size 必须 16 倍数。
- **验证**: 改动后先 `cargo run --example <nearest>` 看 shader parsing/validation 错误（lavapipe 报文行号精确），再谈像素断言。
- **阻塞条件**: WGSL 新增 uniform 段未验证 16 对齐即绑定 binding size。

## PIT-43: flag 偏移雷=探针职责非 review 职责（2026-09-30，I 带第二连）
- **症状**: env_on flag 写在 float 92（旧 27-float 布局残迹），vec4 布局后真位=float 100——sh-diff=0 假装生效，review 三人无一发现，探针一次现形。
- **根因**: CPU/GPU 两端各自演化后偏移表漂移；review 看代码对不出「92 vs 100」这种数值错位。
- **解法**: CPU/GPU 共享布局段一律在两处注释里**同写字节偏移表**（「flag=float 100=byte 400」双边锚）；布局改动的验证谓词必须包含**端到端值差**（sh-diff>阈值），禁止只测「不崩」。
- **验证**: 端到端像素差断言在场（E507 env probe 形）。
- **阻塞条件**: 新增 CPU→GPU 段无端到端值差断言即交付。

## PIT-44: web 通道是 ci 之外的独立面 + 名册脚本读产物不辨新旧（2026-10-08，V2.2 雷两带后才现形）
- **症状**: `pixi run web-check` 报 `the trait bound Vec<f64>: RefMutFromWasmAbi is not satisfied`，
  源头是 V2.2 带写的 wasm 桥 `get_node_transform(&mut self, entity, out: &mut Vec<f64>)`。
  该带当时在基线里写了「WEB MIRROR 3/3」，雷一路活到下一个带才炸。
- **根因**: 三条叠加。① `pixi.toml` 的 `ci` = 11 段，**不含 web-check**（`web-check` 是独立任务）——
  任何「WEB ✓」字样都不由 ci 背书。② 宿主通道（`cargo test -p visiaengine-wasm`，target=host）**不编
  wasm32 代码生成路径**：wasm-bindgen 只在 wasm32 目标要求参数实现 `RefMutFromWasmAbi`，
  于是签名在宿主绿、在真目标红（PIT-22「验证通道≠用户通道」的编译面变体）。
  ③ `scripts/web-mirror.mjs` 断言的是 `target/web/` 里的**构建产物**（d.ts 名册/值对表），
  无新鲜度校验——产物是昨天的，源码编不过照样打印 MIRROR 3/3。
- **解法**: ① 签名按仓内先例归位（`getCameraPose -> Vec<f64>` 同形）：wasm 无出参通道，
  C 面「负码 + out 不触」在 wasm 侧映射为**空数组返回**；② 名册脚本加**陈旧护栏**（源目录 max-mtime
  晚于 d.ts → 红，报文指名目录与补救命令），把「对旧产物报绿」这一类直接堵死；
  ③ 声明口径：**「WEB ✓」只允许来自同终端 `pixi run web-check` 的输出**（#17 的 web 面投影）。
- **验证**: 破坏探针实测——`touch bindings/js/rust/visiaengine-wasm/src/lib.rs` 后单跑名册脚本
  必红（报文=「产物陈旧：…先跑 pixi run web-check」，rc=1）；重跑 web-check 后复绿 MIRROR 3/3。
  `grep -c "陈旧" scripts/web-mirror.mjs` ≥1。
- **禁止**: 用宿主 `cargo test -p visiaengine-wasm` 结果指代 wasm32 可用性；
  未跑 `pixi run web-check` 而在基线/commit message 里写 WEB 字样。

## PIT-45: "window looks lit" is not "window looks right" — two false-evidence channels (2026-10-08)
- **Symptoms**: E510's and E511's resident windows passed every automated gate (first-frame
  lit-ratio 82.3%, pixel-delta assertions, ctest, Xvfb display group) yet both rendered the
  wrong thing: E510 showed yellow discs ~1/4 frame height swallowing the road ribbon; E511
  showed a flat plate that "swung" about the wrong axis (it was built in the x-y plane in a
  Z-up world and rotated about Y — a floor panel, not a gate).
- **Root causes** (three, all independent):
  1. `px_world_scale: 1.0` in a window/assertion lane means "1 pixel = 1 world unit". Point
     radii and stroke widths are declared in **pixels** (`PointMark.radius_px`, GEO-24), so a
     6 px marker became a 6-world-unit disc. The geo pipeline computes the scale from the
     camera; hand-written frames must too.
  2. A hardcoded clear-colour triple in a pixel assertion (`[16,21,29]`) missed every pixel
     (actual `[15,20,28]`), so "geometry px" counted 153600/153600 = the assertion was
     constant-true and had been for a whole band.
  3. Lit-ratio and pixel-delta are *differential* measures: neither has any notion of whether
     the subject resembles what the example claims to show.
- **Fixes**:
  1. derive `px_world_scale = 2·dist·tan(fov_y/2)/height` from the rig (helper in both examples);
  2. self-calibrate the clear reference from the render's own corner pixel instead of a literal;
  3. added a determinism canary (same pose renders byte-identically) so a pixel-delta claim
     cannot be propped up by nondeterministic background;
  4. review the actual picture: `xwd -> PNG` and look (or measure per-band delta). Machine
     harness used: XTEST key/wheel/drag injection + screenshot diff (drives the "only a human
     can press this" cases; angle/title echo read back through the window manager).
- **Verification**: after fixes — E511 diff 1222 -> 5848 (measured, threshold 3500) and the
  picture reads as two hinged leaves between posts on a ground plane; E510 geometry count
  dropped from the vacuous 153600 to 16457 and the ribbon/markers are legible; harness reports
  E506/E507/E510/E511 all OK with Esc exiting each through the real X input path.
- **Prohibited**: hand-writing `px_world_scale: 1.0` in a frame that draws px-sized points or
  strokes; hardcoding a clear-colour literal in a pixel assertion; treating lit-ratio as
  content verification for an example whose claim is about *what* is drawn.

**Pointer (band K, 2026-10-08)**: the hand-built harness this entry describes is now
a repo tool -- `pixi run keys-probe` (scripts/keys-probe.py + scripts/xinject.c).
Rebuild it by hand only if the tool is skipped for a missing dependency.

## PIT-46: a stale ledger line generated a wrong plan (2026-10-08)
- **Symptoms**: the recommended next-step order handed to the user led with "integrate the tile
  scheduler into the render loop" as the highest-value item. That work had already shipped
  (N1.4 / CAPI-35): the engine pumps the layer every frame.
- **Root cause**: the item was copied from a `余账` line in `status.md` without re-deriving it
  from code. Ledger lines are cheap to write and never announce when they become false; the
  install-tree band's own `余账` list had inherited one from the P1 band verbatim.
- **Fix**: before a ledger item becomes a plan item, open its anchor. Here that took one grep and
  produced the *real* gap instead: the capi engine keeps a **single** layer slot
  (`engine.rs:1196/1991/2021` all assign `self.tiles = Some(..)`, no `None`), so a raster
  basemap and vector tiles are mutually exclusive. Band T of the new plan is that, not scheduling.
- **Verification**: `grep -n "self.tiles" bindings/c/visiaengine-capi/src/engine.rs` before
  claiming anything about tile-layer capability; `sed -n '765,805p'` before claiming the pump is
  missing. Cross-check any "余账/待办" line the same way when promoting it.
- **Prohibited**: turning a ledger line into a plan item without re-measuring it; "we should
  integrate X" where X exists.

## PIT-47: 门禁接进 CI 前没问"它的工具由谁提供"——永久静默 SKIP 的绿 (2026-10-08)
- **症状**: band K 把 `keys-probe` / `window-probe-all.sh` 写进 `.github/workflows/ci.yml`，
  本机全绿；但镜像 runner 上这两行**永远不会执行任何断言**——它们找不到 `xwd` 就按设计
  `SKIP exit 0`，CI 只打印绿。步骤存在≠步骤在跑。
- **根因**: 两重。① 依赖来源错判：`xwd` 属**系统包 x11-apps**（`dpkg -S /usr/bin/xwd` 实测），
  不在 pixi 环境里，而 workflow 的 apt 只装 `mesa-vulkan-drivers xvfb`；`xorg-libxtst`/XTest.h
  反而在 pixi.lock 里（所以那部分没风险——逐件查来源才判得准）。② 门禁惯例本身是
  「缺件=SKIP exit 0」（clone 不误红），这条惯例进了 CI 就变成「缺件=不跑也算过」，
  即 PIT-25（缺席恒真）的 CI 接线变体。
- **解法**: ① apt 补 `x11-apps` 并写注释说明它不是可选（防将来"整理依赖"删掉）；
  ② 探针加 `--strict`：SKIP 翻成 exit 1 且**指名缺哪件**，CI 用 `--strict`，本机默认仍可跳过；
  ③ 工具检索走 PATH（旧代码硬写 `/usr/bin/xwd`，别的容器布局既找不到也报不出原因）；
  ④ 接门前先在本机把那条步骤**真跑一遍**（`window-probe-all.sh` 落地一整个 session 没跑过，
  本条补跑：rc=0 / 20 常驻窗 / E509 lit=7500/7500 100.0% vs 5.0% 阈）。
- **验证**:
  ```bash
  # 分支可达性（--strict 的意义就在于它能被触发）
  python3 - <<'PY'   # 只让 xwd 缺失，注入式打 main() 的缺件分支
  #   --strict --all -> rc=1 且报文 "missing toolchain: xwd"
  #   默认           -> SKIP rc=0
  PY
  dpkg -S "$(command -v xwd)"                    # 期望点名系统包，别假设来自 pixi
  grep -n "x11-apps\|--strict" .github/workflows/ci.yml
  bash scripts/window-probe-all.sh; echo rc=$?   # 新接的门禁必须本机先真跑
  ```
- **禁止**: 把带 SKIP 语义的门禁接进 CI 而不给它一条"缺件即红"的调用形；
  在提交 CI 变更前跳过本机实跑该步骤。
