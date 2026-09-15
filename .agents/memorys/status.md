# VisiaEngine Status

**生成**: 2026-09-03 | Phase: 项目初始化 | 分支: main（首 6 提交见 git log，工作区 clean 为常态）

> 前身项目 MediaServo 的全部历史（D/PIT/C 编号体系、crate 矩阵、Phase 记录）见本机归档 `.refinfo/MediaServo/.agents/memorys/`（已被 gitignore，不随 clone 分发）。本目录自 2026-09-03 起为 VisiaEngine 从零积累。

## 概览

| 项 | 状态 |
|----|------|
| 技术栈 | ✅ Rust 核心 + wgpu 渲染（白皮书 v0.1.0，2026-09-03 定）；**后端 = D4 终审定案：wgpu 直用自研管线 `visiaengine-render-wgpu`（不采用 Bevy）**；SDK 形态（C API FFI）；Open Core 商业模型 |
| 许可证 | ✅ 已落地：MIT OR Apache-2.0 双许可正本文件（LICENSE-MIT/LICENSE-APACHE），不可撤销承诺入 README |
| 源码 | crates/ **5** crate（core/render/render-wgpu/io-gltf/geo），~4.7k 行，**68 cargo 测试全绿** + 4×L2 smoke 接线；离屏 golden 9/9 本机 lavapipe 无 SKIP；fixture 已迁 `resources/data/`（1b） |
| 项目定位 | ✅ 多维空间可视化引擎（2D/2.5D/3D 统一，GIS/数字孪生/AV 仿真/BIM），非游戏引擎 |
| Agent 工具链 | ✅ 中性化+注册表+双层 AGENTS；Rust 规则已入 instructions（16 条）；MCP 修复轮：nodejs 入 pixi 环境+with-node 包装器，codegraph/github 通路绿，openspace 归按需族（PIT-6） |

## Phase 状态

| Phase | 状态 |
|-------|:----:|
| 0 项目初始化（配置中性化/白皮书/架构基线/参考库/许可证） | ✅ |
| 1 MVP | 🔨 内容轮+GeoJSON 片完成（G1-G4 ✓；H1-H4 ✓：geo 解析/细分样式/D7 落地/geo_viewer）；宿主嵌入/capi = 后续片） |
| 2 Alpha / 3 Beta / 4 1.0 | — 白皮书路线图 |

## 下一步

1. **push gitee**（远端由用户侧同步推进中，本地领先笔数随轮变动）+ GitHub 镜像决策（ci.yml 五路 L2 smoke+gate 即转现役）——**用户侧另需 GITHUB_TOKEN+重启验 MCP**
2. ~~P1 裁决~~ ✅ D7 已裁 + **H3 已实施**（compose_mvp 落地，WGPU-10 远原点像素一致实证）
3. ~~宿主嵌入片候令~~ ✅ **批准轮（2026-09-11）**：批 2 v1.4 + 批 7 v1.4 同批批准；I0/J0 环境轮开工；Qt demo 仍单独立项（qt-main 扩张随该轮）
4. capi 片（ABI 面 D6 已锁：visiaengine_*/visiaengine.h）；纹理/材质 PBR、屏幕空间线宽、MVT/tile 流式（⑦）= Alpha 档
5. P2 样式 spec 兼容性裁决（simplestyle 六键之上：MapLibre v8 子集？）——样式系统设计轮
4. CI 激活：GitHub 镜像仓决策日（ci.yml 已三连 smoke）；Gitee push 待指令
5. wgpu 升级窗口（季度）：重跑 PIT-3/PIT-5 破坏面清单
6. 环境/许可证条款不变（D5/D6）；镜像 CI 待命期本机 `pixi run ci` 为唯一门禁（G 轮实证）

## MVP 内容轮基线（2026-09-03，G1-G4）

`pixi run ci` 全绿 · spec-trace **66↔66**（CORE/REND/WGPU/GLTF/GEO 五命名空间）· 立方 golden 7/7 本机 lavapipe 真机 · L2 三 smoke 接线（运行绿地点=CI 待命）· 提交面 11 笔（计划 9 + S2/G1 补采 + G4 拆 b，K5 弹性记账）· PIT-5 入档（wgpu [0,1] 静默裁剪）

## GeoJSON 片基线（2026-09-03，H1-H4）

geo crate 16 测试（GEO-01..14）· D7 全链：`tessellate` 输入 local（`GeoKind::shifted` 原语）+ `rebase::compose_mvp` f64 相减 + 远原点立方/geo 全链路像素级验证 · Web Mercator 球面式（EPSG:3857 定义语义入 SDD）· geo_viewer example + 第 4 路 smoke · 提交 12 笔（计划 8+RED 补采/H1 收口各 1，弹性条款覆盖）· 事故三笔如实记于 message（双 view_rotation 并行编辑 / contract 批内未写盘 / 51 声称口径）

## 计划 v1.1 批次 0/1 基线（2026-09-03，Easy3D 借鉴移植开工轮）

- 批次 0：测试三层法(T1/T2/T3)+示例门禁+D8 预登记入 rules/decisions（`0464fd2`）
- 批次 1a：LoadReport+RepairPolicy 五类分型（GEO-15/16）+ GLTF-09 模式过滤（**顺带修 POINTS 假三角存量 bug**）（RED `6892b81`+GREEN `07999cd`）
- 批次 1b：testdata→resources/data 迁移 14 处同步，grep 清零（`229f77f`）
- 基线：66↔66 · 68 测试 · ci exit=0 · golden 无 SKIP；边界事实：JSON 数字超域在文档层拒=双策略 Err（条款注记）；非有限坐标 parse 入口不可达（字段留 web_mercator 直调路径）
- 计划文档 `.omo/plans/easy3d-adoption-plan.md`（v1.1 已批准）；下一步=批 2 计划轮（宿主嵌入，含 D8 转正）或批 3（∥3c→3b）+ 批 3.5 交互片

## 批次 3a 基线（2026-09-03，AttrSet 属性列化轮）

- core::AttrSet（CORE-11/12/13）：三型闭合列存、行对齐不变式、缺失≠零值、首写定型
- geo（GEO-17/18）：props 解析后不再丢弃（宿主查询口打开）；样式经 typed 列读，style.rs 零 JSON 依赖（gate-style 任务入 ci 链）
- 基线：**71↔71 · 74 passed · ci exit=0 · golden 9/9 无 SKIP**；RED `070ce59` GREEN `28c250f`
- 教训两处如实：GREEN 回归网抓到 feed 普通件分支绕过 accept 分类器；grep 门禁谓词首版被自家注释文本绊倒（谓词收窄至代码形态）
- 3c 已裁：**D9=B**（v8 paint 别名子集，主键优先/表达式边界冻结）→ GEO-19 实施（`str_first/f64_first` 回退映射）
- 基线刷新：**72↔72 · 76 passed · ci exit=0（含 gate-style 段）· golden 9/9 无 SKIP**
- 3b 已实施（修订降档 CPU lerp，GEO-20+渲染三色族 golden）：**73↔73 · 80 passed · ci exit=0 · SKIP 0**；批次 3 全清
- 教训追加：grep -c 零匹配=exit 1 断 && 链（3b commit 链破一次，补跑坐实）
- 3.5a/b 已实施（CORE-14/15+REND-21/22 射线族，MT 求交+屏幕口）：**77↔77 · 84 passed**；RED `929b3b6` GREEN `0db9d87`
- GREEN 战果：slab tmax 变量误用真 bug + 测试侧两处（zoom 默认值/f32 aspect 噪声）；PIT-7 入档（deny 联网抖动假红）
- 3.5d 已实施（GEO-21/22 平面量测，amend 收编 lint）；3.5c/e 已实施（REND-23/24 pick_meshes + WGPU-12/13 高亮+**深度面补齐** + E401/E403 + smoke 双路）
- 基线刷新：**83↔83 · 90 passed · ci exit=0 · golden 无 SKIP · smoke-pick/measure 本机真跑绿**
- WGPU-13 存量修复：mesh 管线此前无深度缓冲（单对象 golden 掩盖）——多实体遮挡自此正确
- 交互片（3.5 全 5 项）收口
- **批 2 批准轮执行中（2026-09-11）**：I0+J0 环境轮 `a163d0d`（host/wasm 双 spike 环境、wasm 全树绿零 cfg 修复、PROBE-OK）→ I1 `c68e57c`（句柄/栅栏/线程/错误串/输入口 六条款测试）→ I2 `6bdd7e7`（出图链真身 + **headless readback 行对齐存量 bug 修复** + gate-abi 14/14 + demo 'OK capi headless'）；基线 **88↔88 · 98 passed · ci=0**
- #17 纪律四犯四纠（本轮两次 amend：范围混提交/clippy 红时称绿/87 笔误）——commit message 数字必须粘贴同终端实测输出
- 下一步：I3 attach spike（x11 SurfaceTarget）∥ I4 输入/pick demo → 批 7 J1；push 待令

## Qt widget 轮基线（2026-09-14，v1.3 Q1-Q3 全落）

- Q1（`render_spec` 锁）：viewport 换 dims 三面断言直绿——「行为既有、锁缺席」如实记
- Q2/Q3（本笔）：widget.hpp（header-only 零 moc/PaintOnScreen 三件套）+ qt_app 真窗（texquad+park）+ smoke-qt 三态；QTimer 60Hz 拉泵跑通=**D8 触发器① 复评：维持挂起**（无帧跳过协商需求；再触发=视频解码对齐类宿主）
- conda Qt 双坑入档：qt-main=Qt5 正名 qt6-main；QX11Application 未打包→Xlib 自持 Display*（跨连接 xid 合同兑现）
- 基线刷新：**143 passed · ci 十段 ✓ · SMOKE-QT ✓(真窗@:0) · 108↔108 · 14/14**
- Alpha 余账：npm/pip 打包（含 cmake install 树/corrosion 复评 D-11）、真 PBR、站点、E402、wayland/Win/mac 矩阵

## CMake 工程化层基线（2026-09-14，C1-C3；Qt 轮 v1.3 消费面待跑）

- 计划 `.omo/plans/visiaengine-cmake-project.md` v1.0（Momus OKAY 0B；六→十段口径修）+ Qt 轮 v1.3（构建面交公）
- C1：根门面（版本单源 Cargo.toml 硬错锁）+ SDK 三级解析（AUTO/PIXI/SYSTEM/路径 × Rust/Qt 双通道）+ presets；**审核期实测语义洞**：pixi 激活 PATH 使 SYSTEM 假通过→反污染双滤（拒收报文指名）；tools+cmake,ninja,gxx_linux-64（R1 实测 add 2.6s/首装 ~3min）
- C2：cargo-step（仓级共享 target-dir，复跑 0.16s）+ 伞 `visiaengine::capi`（rpath 注入免 LD_LIBRARY_PATH）+ 树内假 Config + ctest 探针——**首跑抓到真缺口：手写头零 extern "C" 护栏（纯 C demos 三轮全绿掩盖），补后 Qt 面预埋清账**；links 账不立实证销（cargo 须配 build script）
- C3：cmake-smoke 三态入 ci（第 10 段；无 cmake 机 SKIP）+ gate-docs ⑤门面纯度（≤60 行/禁编译规则字样）+ README 双入口
- 基线：**142 passed · 108↔108 零触 · ci 十段 ✓ · GATE-ABI 14/14 · ctest 1/1 · 双负路径报文 ✓**
- 插曲如实：cmake 报文折行致全句 grep 断言漏失（短语断言修正）；预设 schema 字段 `output.outputOnFailure` 两连错；未激活 shell 直调 conda cargo 假 rustc-missing（PATH 语义，教训在 pytest 族同款）

## 批 5 文档/教程归位基线（2026-09-14 收官）

- 计划 `.omo/plans/visiaengine-docs-tutorials.md` v1.0（Momus OKAY→批准→T1-T4 全落；纠偏注记：cbindgen→手写头+签名门禁、mdbook 不做=站点轮）
- T1（`1031eef`）：E 编号改名 11 件（E402 空号预留；measure E403→E203 归带）+ E301 头注粘贴残留修 + tutorials.md
- T2（`bfd2f57`）：**E901_twin_city** 垂直切片 seed（park 实尺 93×31m×256 塔群×PCSS×PNG 人检；九路 smoke +1）；caster slope_bias→0 硬化（理论自影面拆除，实测本景无差）
- T3：gate-docs 四检入 ci（上线即抓 README 83≠108 实证门禁价值）+ README/architecture v0.2/AGENTS 零漂移
- 基线：**142 passed · spec-trace 108↔108 · ci exit=0（四 gate）· smoke 九路 ✓ · SKIP 0 · 漂移字样 grep=0**
- 教训：PIT-10 入档（场景人检三探针教训：像素谓词正确但受光区位置假设错——低日角×楼高→前景合法长影带）
- **批 4/5 之后队列**：4g/4h SSAO/DDP/大图分条/热重载=backlog（排序器不承诺）；Alpha 档=Qt widget 轮/npm-pip 打包/真 PBR(GGX)/站点基建/E402 交互件——均候用户裁决；R2 已裁=A 接受现状（2026-09-15 用户令，包体增量=纹理面合理代价，退路口维持不开）；push 候令

## 批 4f Shadow/PCSS 基线 + 批次 4 收口（2026-09-14）

- 计划 `.omo/plans/visiaengine-shadow-pcss.md` v1.0（Momus OKAY 0B 13 引用实测命中→批准→P1-P4 全落）
- P1（`73c9c39..`→`7845601`）：REND-31 ShadowSetup/Frame.shadow Option（33 构造点兜底 21 实插）+ PIT-5 深度域教训升契约（rig 三元组专用路）
- P2（`f7e8..`→`3361392`）：caster pre-pass（无色彩目标双管线 ShadowMesh/ShadowInst、bias{-1,-1.5}、1024² map 懒建）；View 块 176B（light_view_proj per-draw compose_mvp 复用 [裁决点 a]）；接收三 bgl 恒绑 params/map/cmp-sampler（dummy 常驻=动态分支非布局分支）；params-off 载旧 LIGHT 同位型=sp.a.rgb 退役路逐位不变（R4 护栏 golden 全绿自证）
- P3（`f91f759`）：PCSS 三阶段 [E3D:B3 单 pass 移植]（8 环 blocker textureLoad→半影 clamp[texel,0.06]→16-tap 泊松比较）；单调性双边锁 soft>2×hard ∧ hard<400 ∧ soft<6000
- P4（本笔）：shadow_demo 例（16×16 城 dark=22647 断言）+ GL 降级守卫（wgpu-30 实况：GLES=Gl 子版本单变体）+ smoke 第八路
- **批次 4 全收口**：4ab✓ 4c✓ 4de✓ 4f✓ → 批 5（文档归位）强制触发成立；SSAO/DDP/大图分条/热重载=4g/4h backlog（排序器不承诺）
- 事故如实：naga 两实锤入档注记（`0u32` 字面量后缀拒收→i32 推断+u32() 索引；重复 match 臂遮蔽=三元组未推静默失效，binding-missing 报错定位）；splice 切位吃正则前瞻留孤儿函数（切除）；PIT-7 第 3 见（audit 重试 2 绿）；clippy 二清（needless_borrow/collapsible_if/chunks_exact 三度同款）
- 基线：**142 passed · spec-trace 108↔108 · ci exit=0 · GATE-ABI ✓ · smoke 八路 ✓ · SKIP 0 · web-check ✓（1069627B/429401B 同量级）**

## 批 4de VS 扩片族基线（2026-09-14 收官）

- 计划 `.omo/plans/visiaengine-vs-expansion.md` v1.0（Momus OKAY 0B→批准→N1-N4 全落）
- N1 IR（`73c9c39..`→`213edbc`）：REND-29 Frame.px_world_scale（26+ 构造点编译器兜底）/REND-30 StrokeSeg 48B+PointMark 32B Pod 布局锁+双命令+默认拒
- N2/N3（`6d8559c→8ac0ffb`、`aee0bc4→20edd3c`）：View 块 128B（mat+right+up+px_scale+eye_local，三角系只读前 64B=零回归构造证明：golden 零重录）；vs_stroke perp 扩片 [E3D:B4] + 退化兜底 right + bias{−1,−1} 盖面 [E3D:B7③]；vs_point/fs_point 屏幕基 splat+discard 圆盘=真圆点
- N4（`40115b3`）：geo GeoPart 输出切换（GEO-24：tess_stroke/quad 退役，px 单位纠偏 stroke_width_px=1.5/radius_px=4，键名/D9 别名不动）；capi extra_cmds+ortho px_scale=2·hw/W 精确路；geo_viewer 同款
- 视觉战果：描边不再被同深度 fill 吞噬；点从方块→圆；线宽缩放恒定不重建（Frame 单字段）
- 事故如实：#17 第五犯拦下（N3 首版未 fmt 即提交+空值假声称→二 amend 收编，消息内注记）；PIT-8 第 3 次踩中（strokes 位置窗口凭直觉→probe 修正；points 76° 俯角一次对）
- 基线：**136 passed · spec-trace 105↔105 · ci exit=0 · GATE-ABI ✓ · smoke 七路 ✓ · SKIP 0 · web-check ✓（1053859B/424099B 同量级）**
- 批次 4 队列：**4de ✓ → 4f Shadow/PCSS（[E3D:B3]，PIT-5 深度域换算写成条款）候计划轮**；批 5 触发早已满足（examples=7+4de 门禁例）
- 提取位注记：LineStrip→StrokeSeg 三处同形（engine/geo_viewer/geo_pipeline）——第四消费者出现时抽 helper（tess.rs 内注释锚）

## 批 4c instancing+bench 基线（2026-09-14 收官）

- 计划 `.omo/plans/visiaengine-instancing-bench.md` v1.0（Momus 0B→批准→K1-K3 全落）
- K1 IR（`73c9c39`→`3fe7ccf`）：Instance 32B Pod（offset/height/color+pad，布局三重锁）/DrawInstances D7 同款/create_instances 默认拒
- K2 管线（`3f08d2d`→`d5062a5`）：Variant{Flat,Textured,Instanced} 三 layout 族（WGPU-14 编号零破坏）；storage binding5；vs_inst 底对齐挤出（轴对齐法向零误差论证）；静默跳过封堵
- K3 制品（`4d34bd7`+`aeb5a37`）：bench_twin headless 100k（release 本机 lavapipe：upload 3.8ms/frame 217ms/单 draw）；pick_demo --bench 结 3.5b 欠账；scripts/bench.sh→resources/bench/*.json（劣化>20% 红字非门禁）；unit_box_mesh 公共化
- 存量战果：三窗口 example Bgra/Rgba 错配根修（I3 漏同步，跨 3 轮存活——PIT-9 入档：smoke 七路合并前必跑）
- 基线：**124 passed · spec-trace 100↔100 · ci ✓（audit=PIT-7 重试 1 次后绿）· GATE-ABI ✓ · 七路 smoke 真跑 ✓ · golden SKIP 0**
- 批次 4 队列态：4ab ✓ 4c ✓ → **4de（VS 扩片族：线宽+点 splat+polygon-offset+真点）候计划轮**；4f Shadow/PCSS 其后；批 5 文档归位触发已满足（examples≥6：现 7 个）——最迟批 4 完强制归位

## 批 4ab 材质纹理片基线（2026-09-12 收官）

- M1 io-gltf（`2f293fe`）· M2 RED→GREEN（`4449fb3`→`c9d80f3`）· M3（`434e56e`+`7b92676` 条款体同步）
- 管线：32B 材质块双 layout（Flat 逐像素零回归=构造保证）+ upload_texture(256 行距补零)+textured 双管线 (format,textured) 键；REND-25/26+WGPU-14/15 条款体入册（specular=mock-up Lambert 系数，非 GGX）
- capi/example 纹理链路实装；texquad.glb=io-gltf builder #[ignore] emit 真源可再生；gate-abi 双 demo 路
- 基线：**117 passed · ci exit=0 · spec-trace 97↔97 · golden SKIP 0 · GATE-ABI 14/14 · web-check ✓**
- **R2 超阈待裁决**：WEB-SIZE raw=1062807B gz=431472B（基线 733KB/294KB gz **+47% > 15%**）；退路=「纹理解码仅 native，wasm 收 RGBA 裸字节口」**未擅自启用**，数字已记录。**2026-09-15 用户裁决=A 接受**（退路「纹理解码仅 native」维持不开）
- 教训：PIT-8 入档（像素断言首版谓词=几何覆盖×滤波×通道乘法链，先探针实测再写断言；两轮各 1 次踩中）

## 批 2+批 7 落地基线（2026-09-11 收官）

- 批 2 I0-I4 全清：capi 14 入口（句柄/栅栏/线程/错误协议/输入/attach）+ 双 C demo 真跑 + gate-abi/gate-trace 入 ci
- 批 7 J0-J3 全清：visiaengine-wasm 独立 crate（隔离裁决：no_mangle×bindgen wasm-ld 互斥实锤）· WEB-BUILD 双胶水 · CAPI-09 镜像 · demo 页+T3 · **733KB raw/294KB gz**
- 基线：**90↔90 · 100 passed · ci exit=0 · SMOKE-X11 ✓ web-check ✓**
- D8/D10 已转正（修订面记录齐）；README MVP 四件套销账完成
- 余账：push 待令（本地领先持续增长）· 批 4 渲染强化/批 5 文档/批 6 基建候令 · smoke-x11/web-check 的 CI 现役=镜像仓日

## CMake 示例入口轮基线（2026-09-15，S1-S5 全落）

- 团队模式：探针 A（Easy3D 解剖，采纳 6 拒 5）+ 探针 B（gap 矩阵 G1-G8）→ 计划 v1.0→Momus OKAY(0B/3A 修入 v1.1)→用户批准（C-1 stub/C-2 E703/C-3 存档）
- S2 `be4f970`：VisiaEngineExamples.cmake 契约表+glob⇄表双验（负路径双向实测）+ ve_example_launcher.cpp（chdir/DISPLAY-77/argv 透传）；S1 spike 实证 cargo unix 无哈希硬链同 inode
- S3 `59f5b61`：E701/702 原生 cmake 化（ve_real_+OUTPUT_NAME 规范名+伞 rpath）；qt_app→**E703_qt_viewer** 归 7x 带；tools +xorg-libx11/xproto；G8 修订如实记（cc 路=gate 属性保留）
- S4 `ac60a07`：cmake-smoke `-LE display`（ci 不弹真窗）；presets v7+qt-pixi testPreset（filter.exclude.label=字符串正则）；**真缺口=enable_testing 序**修复（PIT-15）；tutorials.md IDE 运行节
- 基线：**143 passed · 108↔108 · ci 十段 ✓（PIT-7 第 5 见：audit TLS 抖 3 红退避绿）· ctest bare 12 注册（@:0 11+probe 全 Passed 2.4s；无 X 6P/5S）· qt-build 13 注册 preset 跑 7 · E703 ctest 真窗 ✓ · SMOKE-QT ✓**
- 新坑入档：PIT-15（enable_testing 序静默丢）、PIT-16（function 内 enable_language 不外传；presets 权威=随包 schema.yaml 离线正本）；D-12 转正
- 缺口表存档 docs/reference/easy3d-tutorial-gap.md（7 带映射+立项建议序 5 项）
