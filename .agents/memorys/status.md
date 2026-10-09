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

## capi 属性读轮基线（2026-09-15，D13）

- 团队模式：就绪度探针（点云=PURE-DEMO 但与 E202 重叠→用户裁跳过；属性=分裂判定→capi 版批准）→ 计划 v1.0→Momus OKAY(0B/5A 修入 v1.1)→终审 C-1=3 口/C-2=256B/C-3=E701 追加段
- S1+S2 `a5a830e`：Engine geo_docs+attr_of（**u64 位形直通键**零解码）+ FFI attr_f64/str/bool（1/0/-5 零部分写参表）+ 十二处连带（头/版本 0x00010001/gate-abi 17/CAPI-01·02 修订句/README/AGENTS/wasm 镜像）+ 条款 10/11/12 双靶测
- S3 `17e4e35`：E701 属性闭环段 park 实键 + 故意读缺=missing-intact 活广告；双路（手工 cc/cmake stub）同输出实测
- 新坑：PIT-17（harness 共享 pid tmp 竞态，pid+纳秒双缀）；PIT-7 第 6 见（audit TLS try3 绿）
- 基线：**151 passed（143+4 内联+4 FFI，全和实算自 ci 日志 44 目标；S2 commit message 写 147=局部口径，#17 第六犯如实纠于本笔）· spec-trace 111↔111 · GATE-ABI 17/17 ✓ · ci 十段 ✓ · web_capi_pair_mirror ✓ · ctest example 族不回退**

## CMake 结构轮基线（2026-09-16，v1.3 S0–S5 全落）

- 团队模式两轮审（红队 27 项 + 定稿复审 10 项，逐项用户裁决）→ 计划 `.omo/plans/bindings-restructure-cpp-examples.md` v1.3（corrosion 判退：八税单+conda 首蟹+CTest #13 空转；c+ 自写步骤采纳其命名文法）
- 目录终形：`examples/{rs,c,cpp,qt}`（Rust 例=cargo example [[example]]×9 不 bin 化；C/C++/Qt=cmake 真身）+ `bindings/{c,cpp,qt,js}`（capi/wasm 一窝同栖，crates/=纯核心 5）+ `cmake/VisiaEngineBindings.cmake`（R9 注册表=argv 唯一户口，反-glob 双向硬错）
- 跑面收敛：ctest 统一清单（bare 14 条：rs 转发 9 + native + probe）；pixi smoke 9 条=转发壳（`smoke-rs.sh` 带「≥1 被选」断言堵 ctest 空匹配假绿）；cmake-smoke=三态三锚（裸 PATH {configure 探+cargo-build_capi+cargo-build_examples}）+三负路径（SYSTEM/prebuilt 缺物/install 说谎）+Xvfb:78 display 子态（窗口族首获自动执行轨；conda 无 xvfb-run=直启形，PIT-19 机=PIT-19 clobber note 不假绿）
- C++ 面开张：`visiaengine.hpp`（76 行 RAII，三壳语义住头注释，B9 不开 SDD 账）+ `visiaengine::cpp`；E801(SDL3 x11 三帧)/E802(hpp→readback→PPM) 出生即锁；template.c/.cpp 骨架（B8 不建库保 gate-abi 全眼）
- 工具链：presets 双配 EXPORT_COMPILE + bare testPreset -LE display + `.clang-format`(LLVM+4 随现役) + `.vscode/settings.json`(presets always) + `.gitignore` L56 negation 死锁修复（B4）+ install fail-loud 守卫（S-c β；树=打包轮硬债带验收模板）
- 新坑三件入档：PIT-19（pixi clobber→conda Xvfb XKB 死）/ PIT-20（幽灵 API 族立族）/ PIT-21（编辑残留+grep -E 断言，升 edit-safety ###18）
- 基线：**151 passed · spec-trace 111↔111 · GATE-ABI ✓ · GATE-DOCS ✓（E 14 件三方）· CMAKE-SMOKE ✓ · web ✓ · smoke 11/11 · prebuilt 退场真验 · 工作区 clean**
- 待办：push 候令（本地领先 += 本轮）· Phase B 首带（E8xx 数据带 API 缺口清单→capi 扩口）候令 · RA example Run lens 人验（T3，风险④已降辅助轨）· 打包轮三件（install 树/corrosion 复评/npm-pip）

## B1 数据带基线（2026-09-16，Phase B 首带：CAPI-13..16 四口 + E8xx 例子四件）
- 口面：显隐（严格 0/1、枚举域不变、render/pick 过滤）/ 查询 / add_mesh（VeMeshDesc struct_size 前瞻门、返回码+out 谱、退化零提交）/ remove（双销毁同谱、ABA 隔离）；abi minor=2（0x00010002），17→21 入口，双面镜像（wasm 4 桥 + d.ts 在场断言）+ hpp 4 薄转发
- 例子：E810 attr survey（零新口验收例）/ E811 显隐孪生（c/cpp，目标名带语言尾首用）/ E812 程序化增删；ctest 14→18 条（17 编号）
- 设计反转实录：add_mesh 初版 0=失败哨兵撞 CAPI-01「实体位形 0 合法=有意分工」条款——返回码+out 改谱；测试两处哨兵断言随条款纠偏（幽灵 API 教训的接口设计分册）
- 基线：**160 passed（+7）· 115↔115 · GATE-ABI 21/21 · GATE-DOCS ✓ 17 件 · CMAKE-SMOKE ✓ · web MIRROR ✓ · ci 十段全绿**

## lesson-review 记录（2026-09-16，结构轮+B1 带会话批量回顾）
- 新增：PIT-22（验证通道≠用户通道 DISPLAY 族·PIT-18 泛化）、PIT-23（CMake/ctest 恒真/吞噬三形·E801 死挂案）、edit-safety ###19（管道尾吃 rc·带病入库实锤）、testing.md「视觉例双保险」（像素门+三自证+T3 清单+转发壳断言）、conventions C15（哨兵先对条款分工表/参数不侍二主）、C16（决策呈交=人话+示意+目录+逐项）
- 会话主线：v1.3 结构轮（S0-S5 七笔）+ FOLDER 位置定则 + E201/E301 交互根修 + B1 数据带（RED→GREEN 四口 21 入口 ctest 18 条）；基线 160 passed·115↔115·GATE-ABI 21·ci 十段
- 下一步队列（不变）：push 候令（领先 7+本笔）→ T3 人验五条（E201/E301/E801/测试面板/RA lens）→ B2 剖面裁切计划轮候令

## E202 灰屏根修（2026-09-17，D7 相机位形契约的第二见）
- 根因：resumed() 拟合缺失——DrawMesh 世界位形=origin+local（D7），相机 target 留 [0,0,0] 对 3857 世界系 park 差数百万米 → 全帧清屏色；离屏探针复刻案发参数实证（四变体全空）后修复：target=层 bbox 中心 + dist/zoom 装载期拟合
- 永装门：geo_pipeline::golden_geo_window_fit（spec WGPU-11 多例同条款，spec-trace 115↔115 不动）——谓词三自证齐：canary 案发机位恒 0 px、修复路 23222px 实测阈 12000（−48% 保守位）
- 同族清屏：E901/E401/E201/E301 机位逐例核对无撞（E201/301 数据住局部系 origin=0 自洽）；PIT-22/双保险纪律第二次抓真案（灰屏家族三见：E201/E301/E202）
- 基线：161 passed · 十段全绿 · ctest 18 条不回退

## E402 交互拾取窗带（2026-09-17，空号转正 + unit_box 绕序存量雷根修）
- E402_pick_interactive：hover 橙预览/左键点选黄 toggle 多选/拖轨道/滚轮远近/R 清空/Esc 退；
  placed() 位形单源（pick world=render transform 同一份矩阵=E401 双份烘移 64px 教训构造性根除）
- **存量雷**：unit_box_mesh 出生反绕（几何法向=-声明法向）——GPU cull 默认关三消费者全渲染零
  可见，pick 正面规则（REND-23）一消费即全 miss；根修=索引镜像翻正 + 金训注记，162 passed
  构造证明无像素回归（cull 关下绕序不观测量）
- 门：golden_pick_window_mirror（spec WGPU-12 多例同条款）512×320 窗形镜像 + 中心命中链复用
  + 外角 canary；实测 (35,132,62)→(177,147,51) 黄族翻转锁
- 登记面：Cargo.toml/契约表/tutorials/pixi 四件 + gate-docs E402 预留豁免收回（真件在位）
- 基线：**162 passed · 115↔115 · GATE-DOCS E 18 件三方 · ctest 19 条 · ci 十段 ✓ · SMOKE E402 ✓(真窗)**

## 色彩+事件带（2026-09-17，CORE-16/CAPI-17，计划 color-events-band.md K1-K6）
- CORE-16 sRGB↔线性双口（f64 中间精度，端点逐位）；全链契约=宿主面 sRGB/IR 线性/GPU 两端
  Srgb 格式硬件编码；咽喉五点（material 块/clear/strokes/points/instances 表）+io-gltf factor
  反变换归位；GL 降级档=回退线性直通现形
- 域重钉七处（全部实测先行）：clear [13,18,25] 硬件舍入自标定 canary、shadow 亮暗双峰 95/140、
  纹理棋盘 b 双峰 135（uv 断链语义保持）、E501 分界 128 dark 阈 4000（实测 19642）、
  io-gltf 色钉×2、geo 拟合门容差形
- **挖出并根修**：lavapipe 语义实锤 clear load 值按线性域解释（0.05→63 探针）；渲染测试带
  域脆性教训（编码抬中间调）
- CAPI-17 事件口（21→22 入口，abi 0x00010003）：EvtSink cfg 双形（native C 锚 unsafe-Send/
  wasm Box<dyn FnMut>）、进度单调终态锁/错误同刻/NULL 摘除、三面镜像（头/hpp/wasm+d.ts）
- 例子 E21 件三方：E704_host_callback.c（事件三态）/E502_color_tuning（**WYSIWYG 四色板逐字节
  互逆活证**，采样列公式纠一处）/E503_stroke_points（恒宽秀）+ 永装门 golden_zoom_width_invariant
  （两档 zoom 10px/φ24 跨档锁，列采样=与 zoom 无关相位纪律）
- 基线：**167 passed · 117↔117 · GATE-ABI 22/22 · GATE-DOCS E 21 件 · ctest 22 条 · ci 十段 ✓ ·
  WEB MIRROR ✓**；结构事故自纠：E402 带遗留 _disp 四僵尸行清除（edit-safety #18 族再现一例）

## 双模人验收口（2026-09-17，用户令「例全检 + cmake run target 齐」）
- native run 步全例覆盖：宏 INTERACTIVE 关键字废除，run_<name> 无条件发（DISPLAY 族 run-gui
  包装/:0 回退、headless 族直跑=终端 OK 行人验面）——E702 窗口例首次获得 run 目标；矩阵=
  rs cargo-run_*×12 + native run_*×9（+qt 宏外自注册 E703）
- E502 双模化：色彩标定例从「headless 秒退」升格无参常驻人验窗（四色板 960×600，Esc/关窗退）
  + --frames 自断言路原样保留（ctest）；两路同源 build 数据、config.format 显式 Srgb 优先
- 审计结论（全例逐跑实测）：窗口例无参=常驻 ✓×10；自退例=设计身份 ✓（终端输出/PNG 落盘即
  人验面，拾取替身=E402、色彩=E502 窗、恒宽=E503）——除 E502 外无隐性分叉
- tutorials.md「人验形态」段：双模约定成文（IDE 零参=人验 / ctest argv=自动，C15 单源）
- 基线：167 passed · 117↔117 · GATE-DOCS ✓ · CMAKE-SMOKE ✓ · qt-configure ✓ · ci 十段全绿

## 点云数据族带（2026-09-17，D16/hyperplan 计划 pcl-data-band.md，M0→G 全收）
- 口面：CAPI-18 add_points（23 入口 minor=4）+ CAPI-19 load_pcl（24 入口 minor=5，
  VePclReport 四类导出/meta attr 缀查）；三面镜像全带（头/hpp/wasm loadPclBytes+d.ts/
  web-mirror 钉×2）；C15 值域表先写（attr found=1/none=0 测试纠偏入册）
- io-points 新 crate（裁决 C 授权）：PLY ascii/bin_le 自写 ~420 行零新依赖；四类分型
  「点级触发 FastFail、列/元素级只计数」落点澄清入条款；截断双语义双向断言；
  spec-trace IO 前缀 + **第三检机器锁**（负例 FOO-01→红实测在册）
- 例/门：E204 双模（螺旋自断言阈随点数缩放量纲 + --file 装载路 + 常驻窗）；
  G1 100k 批量完整性 / G2 远原点互逆点版（逐字节等）；G3 --points 支 1M 实测入册
- **工具链根修**：bench.sh 包名 S1 后死链 + 缺席 continue 恒真（PIT-25 入档，MISSING→exit1）；
  rustfmt 误喂 Cargo.toml 事故（edit-safety #20 入档）；测试锚三处 rustfmt 重排教训=锚必现读
- 基线：**181 passed · spec-trace 125↔125 · GATE-ABI 24/24 · GATE-DOCS E 22 件三方 ·
  ctest 23 条 · CMAKE-SMOKE ✓ · WEB ✓ · audit try1 ✓（PIT-7 第 7 见）· 8 crate**
- 余账：LAS 二带（判据=客户演示单）· 拾取族触发制 · cap 改数窗一次 · push 候令

## B2 剖面裁切带基线（2026-09-17，计划 capi-cross-section-band v1.1·Momus OKAY 0B/2A 修入）
- 口面：CAPI-20 set_clips/get_clips（26 入口 minor=6；n=0 唯一清空/界检先于解引用/
  归一化引擎侧读回单位形/cap 截断返真数 [Momus-A2 用例入 RED 谱]）；三面镜像全带
  （头 32B Pod 系数形/hpp 两薄转发/wasm setClips·getClips+d.ts+mirror abi 钉 6）
- 管线：View 块尾缀扩段 192→256B（**零新 binding/绑组/缓冲**=origin+transform 已在
  view_block 签名的设计红利）；clipped() fs discard 全五管线+caster 同裁（fs_shadow
  空函数补体，bgl binding0 +FRAGMENT）；扩片族逐像素=「裁世界不裁类型」；None/EMPTY
  全零恒绑早退=R4 护栏形，golden 全套零重录
- IR：REND-32 世界 f64 系数+ClipSetup::new 退化拒；clip_to_local **model-space 恒等式**
  （n_m=Mᵀn/d_m=d+n·(o+t_M)，列主序 Mᵀ 与 M 平移列两处自查根修如实记）
- 拾取：engine 重试环（命中 keeps 负侧=排除续找，剖开可见者必可拾）；capi pick 域
  =positions×IDENTITY 既有事实以「两帧合一」注记入例体与测试
- 例/门：E813 无头三段活体（基线红 10816/半刀红灭绿存+重试环/角域三面 1764∈[800,4000]
  ——二面形 g2=0 探针战果=前墙同屏遮蔽，PIT-8 三见）；tests/clip.rs 五门（canary 逐
  字节/None≡EMPTY/半切对半/全切清零/角域¼+far_origin 1e7 逐位等+线点分色族+caster 6.5×）
- 新坑：PIT-26（from_raw_parts NULL+0 即 UB，debug 前判实锤——NULL 计数路必 m>0 守卫）；
  PIT-7 第 8 见（audit TLS 抖一次重试绿）
- 基线：**191 passed · spec-trace 130↔130 · GATE-ABI 26/26 · GATE-DOCS ✓ E 23 件三方
  · ctest 24 条 · CMAKE-SMOKE ✓ · WEB MIRROR 3/3 + web_capi_pair_mirror ✓ · ci 十段 ✓ ·
  8 crate 不变**
- 非目标挂账：盖帽 stencil（BIM 内腔演示单）· 六面盒 · OR/union · 硬件 CLIP_DISTANCES
  双路（wgpu30 存在但 DX12 未在列）· 交互剖切滑杆（E901 挂键候需）· LAS（票据制候令）
- 决策入册：**D17 八裁决点**（discard 单路+复评触发/≤4 AND/无盖帽/Mᵀ恒等式/全族受裁/caster 同裁/重试环/E813）

## S2 文字标注带基线（2026-09-18，计划 io-text-labels-band v1.1·Momus OKAY 0B/A-1 修入）
- 带义：档①+方案 I（引擎自管 fontdue 栅格，用户明裁）——标注/图注渲染从「SDK 短板」转正
- 口面：CAPI-21 load_font（替换式；失败保旧）+ CAPI-22 add_label（struct_size 前瞻门+
  时序门「无字体=拒」+值域全谱零提交；26→28 入口 minor=7）；三面镜像全带（头 POD/hpp 薄
  转发/wasm loadFont+addLabel→bigint+MISS/d.ts 钉/mirror 值对）；E701 文字闭环段+gate-abi 28
- 新 crate：visiaengine-io-text 第 9 个（face/GlyphCache R8-shelf-512²-1px 缝-满则重烘/layout
  LTR；IO-07..09 三条款；fontdue 锁 default=false+hashbrown【探针盲点本地源码补正】）
- 管线：REND-33 LabelMark 64B 4×vec4 布局锁+DrawLabels 第 6 变体+create_labels 默认拒；
  WGPU-24 独立 bgl(0+11+12+13)+R8 atlas+ClampToEdge+恒顶(Always/不写深)+SRC_ALPHA；
  WGPU-25 锚判整标同生共死；GEO-25 text-field"{prop}"剥壳/常量二形+列缺值 skip 不落字面量
- 例三门：E205 双模（white=667/warm=348 探针阈；ortho 跨 zoom assert_eq 逐像素等）/
  E814 C 活体（时序门+utf8+white=3917）/labels.rs WGPU 双测（atlas texel/恒大小/锚判双向）
- 教训入档：PIT-28（pen 是 px 不可进 anchor 世界坐标——hist 探针抓出，三口 grep 律）；
  #17 双纠（计划 140=REND-32 双计虚账，实账 139 粘 log；「位形可枚举」断言两次方向错=
  items 外域语义未先对表）；E0449 cfg(test) 跨 crate 不可见教训→自省口转正 API
- 基线：**202 passed · spec-trace 139↔139 · GATE-ABI 28/28 · GATE-DOCS ✓ E 25 件三方 ·
  ctest 26 条 · CMAKE-SMOKE ✓ · WEB MIRROR 3/3（abi 钉 0x…0007）· ci 十段 0 红 · 9 crate**
- R2 复账：wasm gz 429401→490967=+14.3%（阈内过线零余量，探针预估 +6% 偏乐观如实记）
- 余账：push 候令 · billboard/halo/避让/换行富文本/文本更新口=label 后带触发制 ·
  CJK=宿主注字体同口（演示明账拉丁 fixture）

## E901 交互收口带（2026-09-18，D17-h/D18 演示收口，单例文件+账零变动）
- 孪生城常驻窗挂键：**C=剖切刀 ON/OFF（z≤6m 起刀，影随刀塌）· [ / ]=刀高 ±1m（clamp 0.5..15）· L=三塔屋顶标注（Tower-A/B/C，io-text 全链窗形首演）· Esc=退出**；标题实时回显状态
- 构造面教训一则（写后自查根修）：首版「append 再 pop 复原基底」的 swap 逻辑在无字体分支会**掏空全部命令**——正解=基底永不掺标签，redraw 帧按开关拼接（show 域住渲染不住构造）
- 验证：headless 路零回归（bright=3019 dark=2426 原值）· 22s 存活+真窗节点 ✓ · SMOKE ✓ · ctest #14 Passed · ci 十段 0 红 202 passed · 条款/入口/ctest 计数全不动（纯消费者面）

## ⑤a 相机飞行带基线（2026-09-18，计划 camera-flyto-band v1.1·Momus OKAY 0B/3A 修入）
- 口面：CAPI-23 fly_to（pose f64 八位 struct_size 门/瞬移形/改道连续/near-far 恒现值）+
  CAPI-24 fly_state（done 含 idle 单主/out 飞中写）；28→30 入口 minor=8；三面镜像全带
  （wasm flyTo·flyState·flyProgress+d.ts+hpp）；E701 无新段（E815 独立例）
- IR：REND-34 Easing 三族+fly_sample 纯函数（端点恒 from/to 原值快路；yaw 最短弧住采样器
  构造件 mix_rig 零触碰=REND-15/16 存量零回归）；E302 反证锁（线性中点两侧对照）
- **wasm 时钟雷先拆**：std::time 于 wasm32-unknown-unknown 运行时不可用→now_ms() 双形 shim
  （native Instant 进程锚/wasm js_sys::Date::now；js-sys 0.3 target dep）——编译过≠跑得通
- 例：E302_fly_camera 双模（1/2/3 预设互飞/拖滚 cancel/R/Esc；headless 四断言 bright 实测
  96 阈 40）+ E815_fly_camera.c 无头活体（idle 即 done/瞬移/进度单调 14 样本/cancel/域拒×4）
- 教训如实：wasm 桥 pose 扁平 7/8 元手滑自纠；借用闭包三 borrow 错→展开直写；中文锚 heredoc
  绊（#9 跨带复发第 N 见）；T2 漏 gate-docs（E815 tutorials 行）T3 抓回=门禁互保
- 基线：**209 passed · spec-trace 142↔142 · GATE-ABI 30/30 · GATE-DOCS ✓ E 27 件三方 ·
  ctest 28 条 · CMAKE-SMOKE ✓ · WEB MIRROR 3/3（abi 钉 0x…0008）+ pair_mirror ✓ ·
  ci 十段 0 红 · 9 crate 不变**
- 非目标挂账：多视口分屏（四坑清单 D19/触发=分屏演示单）· 高度弧线 · 多关键帧巡览 ·
  惯性阻尼 · fly 事件推送 · CJK 内嵌（S2 余账）

## ⑤b 多视口分屏带基线（2026-09-18，计划 multiview-split-screen-band v1.1·Momus OKAY + 用户「全带开工」）
- 管线：render_view_rects（N Frame×ViewportRect 单 surface/单全幅 depth 多 pass；首 pass
  Clear 全区底色+后区 Load 色+**每 pass 深度 Clear+Discard**）；caster 一趟共享 map；
  full-rect 零 set_*=旧路逐位等 canary；REND-35 ViewportRect 独立类型**零触 Frame**（44 兜底波免交）
- 裁决门定论：wgpu30 LoadOp::Clear=整个 attachment（AllClear redA=0 现行锁，Vulkan 定论一致）——
  两探针分歧由本机像素门裁决（E303 首跑又抓深度 Load 吞图雷→每 pass 清形，探针预期被覆写）
- 存量雷清剿：fov 弧度/度数单位（E501/E302 窗面投影畸变实锤，E302 bright 96→1987）；
  camera.rs orbit 文档钉单位；multiview 四门（裁决/互不侵犯+缝色/深度围栏/共享影 328×328 阈 150）
- 例：E303_split_screen 双模（96×64 四断言+右上正方小窗跟随；常驻窗 ✓）；注册四面+gate 全套
- 教训入册：「像素门没看过的窗面=没验过的面」（D20 末条）；隔离探针法（单投全屏 ISO→多视口
  差分）两分钟定位 pass 间差异；U1 漏写 REND-35 体被 spec-trace 143 现形抓（靶/体成对纪律）
- 基线：**214 passed · spec-trace 144↔144 · GATE-ABI 30（未动）· GATE-DOCS ✓ E 28 件三方 ·
  ctest 29 条 · CMAKE-SMOKE ✓ · ci 十段 0 红 · R2 零触（纯 rs 带）· 9 crate**
- 余账：push 候令（含本带 3 笔）· capi 多视口二波（CAPI-25/click-to-fly）· 共享 viewer 库立项候令 ·
  半透明④带/打包轮/LAS 候令

## ④ 半透明带基线（2026-09-18，计划 translucent-water-band v1.1·Momus OKAY 0B/3A 修入）
- 三无纪律兑现：**零新口/零依赖/零 IR 字段**（入口 30·abi·R2 全静；透明判定=材质 alpha<1
  后端侧表分拣 WGPU-28）；材质 a==1 恒旧路=**golden 全族零重录构造保证**（labels 族两 pass 现行）
- V0 混合域定案：本机 wgpu30=线性域（151 vs 83 棋盘门单值判案，Vulkan spec 一致）；
  预言算术首版漏算 Lambert shade——PIT-8 扩注「预言含全链因子」
- 管线：Variant 扩 FlatT/TexturedT/InstancedT（bgl/layout 全同仅管线态：SRC_ALPHA 标准式+
  不写深+Less 保留遮挡）；两 pass=opaque vec 序 + trans 视深降序（f64 D7 键 origin+M 平移列，
  同键提交序稳定）+ Labels 恒顶；**pass2 depth Load 承遮挡**（与 ⑤b 视口间清深分域）
- E504 双向语义锁：水膜=衰减对比（52→15 对照带 (10,30)），非消灭对比；1/2 键=重传材质
  改 alpha 的宿主路活演示；GEO-26 靶+条款（park α=0.8 WGPU-11 门复测零重录余量内）
- 教训：透明序测试「近远」命名被相机位打脸一次（z50 俯视→z7=近；期望归正引擎零触）；
  amend 连坐三犯（fmt/clippy/Mats 死码——终验前本地全链先行，勿以 amend 当 lint 循环）
- 基线：**220 passed · 148↔148 · GATE-ABI 30/30 · GATE-DOCS ✓ E 29 件三方 · ctest 30 条 ·
  CMAKE-SMOKE ✓ · WEB MIRROR 3/3 · ci 十段 0 红 · D21 入册**
- 余账：push 候令（④ 四笔+前带累计）· set_material_alpha/软影/OIT/逐实例 alpha/逐顶点 alpha
  全挂触发制 · ⑤b capi 二波不变

## ⑤b 二波小地图导航带基线（2026-09-18，计划 capi-minimap-nav-band v1.1·Momus OKAY 0B/2A 修入）
- 口面：CAPI-25 set_map（VeMapView 比例表；NULL=关=旧路；域拒零副作用）+ CAPI-26
  navigate_click（区外/无图拒；实体→地面两阶段；保角保距 [裁决 e]）+ CAPI-27
  get_camera（VeCameraPose 复用双向=入参 fly/出参读回，宿主预置 struct_size 先检后写）；
  30→33 入口 minor=9；三面镜像全带（头/hpp 三薄转发/wasm 4 桥 setMap·clearMap·
  navigateClick·getCameraPose+d.ts 名册+mirror 钉 0x…0009）
- 引擎态：MapView+set_map/map_rect/map_rig（主 target 派生顶视）/navigate_click/
  camera_pose；render 双投分支（⑤b render_view_rects 消费首实证；None=旧路 canary）；
  pick 小图区优先路由；apply_input 小图区 kind1..4=消费 no-op
- 例：E816_minimap_nav.c 五段活体门（canary 逐字节/角区双族 1596g+756r/小图 pick/
  600ms 飞行 257 tick 到站 (3.579,3.714,0)/值域拒+无图拒）；ctest #31
- 教训如实：E816 首版把两水平面当「墙」——顶视=红盖绿 g=42 探针现形（PIT-8 四见，
  场景几何先探针后断言）；段5 终态 canary 比旧 ref=飞行挪相机必破（写后自查根修，
  基线重取形）；ffi_spec use 块漏三函数名（regex 换行锚未命中=edit-safety #9 族）
- 基线：**226 passed · spec-trace 152↔152 · GATE-ABI 33/33 · GATE-DOCS ✓ E 30 件
  三方 · ctest 31 条 · CMAKE-SMOKE ✓ · WEB MIRROR 3/3（abi 钉 0x…0009）· ci 十段
  0 红 · 9 crate · R2 零触（web gz 增量 web-check 打印为账）**

## 超级带五连基线（2026-09-24，计划 super-band-2026-09-24 v1.1·Momus OKAY + 三红队审核 + 用户九卡逐项裁决全 A）

- 源起：team 三路差距审计（`docs/reference/team-gap-analysis-2026-09-24.md` 取代 09-17 版）→ 用户「全部采纳」行动序 1-5 → 四路审核（Momus OKAY 0 阻塞 + feasibility/consistency/risk 三卷 `/tmp` 索引）→ 两 plan-killer（B2 VePickHit 前提崩、B1 缩略图前提崩）+ 一 CRITICAL（B2 API 形态）全部修入 v1.1
- **B4 文档对账**（`51f94bb`）：WMTS"内置"/场景树×3/capability_query 三漂移改词（EN, C17）+ **10MB 首测销账**（cdylib strip 7.25MB ≤10MB 达标 27% 余量，evidence/2026-09-24-native-size.md）+ 差距分析正本落盘
- **B2 点拾取**（`03a864a`）：裁决④A=零 ABI 变更（引擎留存 positions+CAPI-04 修订句+kind 挂账触发制）；REND-38 屏幕空间最近点谓词（**局部点直投 compose_mvp**=origin 已烘矩阵勿双计+RH [0,1] 深度域守卫）；REND-39 mesh 优先 canary；CAPI-13 纠正为 CAPI-15 先例；E204 `--pick-check` argv 单主（C15②）
- **B1 Web 画廊**（`2254bbc`）：裁决⑤执行中反转=工具 bin 重造场景（漂移税）弃用 → **例侧自落盘**（gallery.rs helper ×10 双模例 save_frame，例=卡图真源零漂移）；gen_gallery.py 读唯一注册表（cmake+各目录 CMakeLists+qt 裸 add_executable 三形）+fail-loud 双负路径实测；32 卡/11 真图/build gitignored
- **B3 MVT 瓦片流 Phase 0**（`080e07e`）：第 10 crate io-tiles——353 行手写 protobuf wire 解码器（裁决①A，零依赖；fixture 生成器两处 wire 错靠 Rust/python 交叉对照现形=解码器零错）+slippy 瓦片数学（邻边无缝律）+FileSource/typed HttpSource stub/stdlib LRU+GEO-27 GeoTile 映射（y 翻转+D7 origin 锚）；E206 走通全链；**PIT-8 再兑现**：相机 look_at 世界原点而瓦片住 x=−2e7 m=全画面外+far<eye 距全裁
- **B5 一例四吃政策**（`a58e689`）：gate-docs 六检+⑥画廊 manifest↔磁盘注册集机器锁（负路径实测红）；政策文本三处（tutorials 头/dev-workflow/gate 注释，EN）
- **E206 用户三连实测修**（`3f3fccd`/`3690a34`/`9eceb18`）：①无参即退=C15 双模违约（窗桩）→E204 骨架真窗+Xvfb 存活探针；②拖拽滚轮无效=Redraw 每帧硬编码 tile_rig 重建+rig 非字段 →持久化 rig+orbit_delta/zoom；③单瓦片太简→**3×3 邻域**（每瓦片变化水斑/交错道路/边框描线=缝可见/中心 POI；顶点烘场景中心小值域 f32 单一世界帧）；滚轮根因=**正交缩放唯一定义 half-width 而 rig.zoom 从未被消费**（look_at 默认 zoom=1.0+渲染硬编码 tile_w 双叠加）
- 基线：**259 passed · spec-trace 158↔158 · GATE-ABI 33/33（零 ABI 变变守住）· GATE-DOCS 六检 ✓ 32 卡 · ctest 24（机器实报）· 10 crate · 零新依赖 · ci 十段 rc=0 · 工作区 clean**
- 待办：push（领先 11 笔）· Phase 1 瓦片带（C API+HTTP，白皮书日期承诺 2026-Q4）· 画廊 T3 人验

## ⑤ Phase-1 瓦片流带基线（2026-09-24，计划 tile-streaming-phase1.md P1-S1–S4 全落）

- 口面：CAPI-28 load_mvt_dir（FileSource 扫描 z 层；域 z≤30；返回=挂载瓦片数）+ CAPI-29 set_tile_view（3857 bbox 喂调度器；返=可见数；未挂载=拒）；30→**35 入口 minor=10 (0x0001000A)**；三面镜像全带（头 2 原型/hpp 薄转发/wasm loadMvtDir+setTileView+d.ts 名册/mirror abi 钉 0x…000A）
- 引擎态：TileLayer 结构（set:TileSet + ids + center/wpos/widx/strokes/points + uploaded:Option，render-loop 集成=后带，`#[expect(dead_code)]` 注记）；load_mvt_dir 扫描+ensure+decoded→GeoTile 批建；set_tile_view 仅 ensure 不重建（批=挂载时一次性）
- io-tiles 新件：`scheduler.rs` TileSet{visible 半开枚举+boundary snap/ensure 幂等装载+会计/decoded through-cache 二级}（IO-13/14）；HttpSource::new(root) ureq 实装（IO-15，target-gated：wasm=typed stub，getrandom/js 冲突 root-cause 修）；`tests/tile_set.rs` 6 门；`tests/http_source.rs` 3+1 ignored；source.rs stub 测试升格为 new()+域断言
- 依赖：ureq 3.4.2 (rustls, default-features=false) → webpki-roots (CDLA-Permissive-2.0) → **deny.toml += CDLA-Permissive-2.0**
- E206：load_tile 退役（dead code）+ FileSource/decode_tile 导入清理；load_viewport 经 scheduler 驱动（IO-13/14 消费面）
- gate-abi.sh：33→35 期望数更新；web-mirror.mjs：abiVersion 0x0001000A + 名册 +2 桥
- whitepaper：MVT Phase 1 兑现（从「在交付项」改「已交付」+WMS/WMTS 单独注记）；README 163 条/35 入口/32 卡同步
- 教训入档：PIT-28（pen px vs anchor 世界坐标，三口 grep 律）· **PIT-31**（ffi_spec 域测试 cwd=crate 目录，相对路径陷阱）· **PIT-32**（ureq 全量依赖把 getrandom 拖进 wasm 目标——target-gate 修，wasm HttpSource=typed stub）
- 基线：**270 passed · spec-trace 163↔163 · GATE-ABI 35/35 · GATE-DOCS ✓ E 31 件 · ci 十段 rc=0 · web MIRROR 3/3 · 9 crate · ureq 1 new-dep**
- 余账：push（含本带 P1-S1..4 全笔）· render-loop 集成 TileLayer（E206 prove 形态入引擎主循环）· HTTP 源 C API（**CAPI-35+** set_tile_source_http；CAPI-30..34 已被场景树带占用，2026-09-28 重记账）· CJK 字体 S2 余账 · LAS 票据制候令

## ⑥ 场景树带基线（2026-09-24，计划 scene-tree-band.md S1-S4）

- 口面：CAPI-30 create_group（out_entity 谱）+ CAPI-31 set_parent（0=root，环/深度≤16 检测）+ CAPI-32 get_parent + CAPI-33 set_group_offset（finite 门）+ CAPI-34 get_group_offset；35→**40 入口 minor=11 (0x0001000B)**；三面镜像全带（头 5 原型/hpp 5 薄转发/wasm 5 桥+d.ts 名册/mirror abi 钉 0x…000B）
- 引擎态：Slot 新增 parent:Option<EntityId>+offset:[f64;3]（core scene.rs，Component::Group 纯标记变体，slot_of 零成本）；Engine::new_scene（预占 slot0gen0→handle 永非零）；is_hidden（沿祖先链 walk，树感知显隐）；render 循环 origin=it.origin+scene.effective_offset(entity)（CORE-21 累加 O≤16）
- **设计决策**：旋转/缩放继承=触发制（YAGNI：主流 GIS 引擎=仅显隐+平移）；despawn group→子实体自动摘回根（不级联删除）
- 教训：PIT-31（ffi_spec 测试 cwd=crate 目录，相对路径陷阱）**第二见**（load_mvt_dir + set_group_offset 域测试均触发）· **PIT-33**（场景树 handle=0 语义冲突：slot0gen0 合法 vs parent=0=root 哨兵；修法=new_scene 预占 slot 0 + set_parent parent==0 特判）
- 基线：**287 passed · spec-trace 173↔173 · GATE-ABI 40/40 · GATE-DOCS ✓ E 31 · ctest 33 条 · ci 9/10（audit=PIT-7 第 11 见 TLS 持久故障非瞬抖）· web MIRROR ✓ · 9 crate**
- 余账：push（领先含本带 S1/S2/S3 三笔+⑤ P1 五笔=约 11 笔）· 旋转继承=有需求再开 · 场景树名字查询（node_names 存 engine.rs，无 C 查询口）· CJK 字体 S2 余账 · LAS 票据制

## 差距调查→hyperplan→Session N+1 基线（2026-09-28，八笔收官）

- 团队模式：四路调查（.refinfo 六族 + 自查）→ 对抗规划 4 批手 ×3 轮（C1-C10 全签 + 认输台账）→ Momus OKAY → 四卡裁决（1a 瓦片泵 / 2 金标+EDL / 3 morph 做 / 4 不部署+three.js 对标）→ 执行
- N1.1 `a5d3180`：hpp fly_state 转发 + **hpp⊇头镜像门**（别名表制：destroy→destroy_now、entity_set_visible→set_visible；破坏探针自证）+ CAPI-30→35+ 重记账 + 存量 clippy 清账
- N1.2 `59c772a`：whitepaper L27/L103 + README L11/L38 精确改词（"平滑/无缝/无级"→交付语义；L22 竞品句保留）
- N1.4 `47bbe54`：**CAPI-35 set_tile_source_http**（发现式装载）+ io-tiles 4 态泵（begin/pump(budget)/state）+ HttpSource 5s 超时 + loopback 免代理（PIT-35）+ 渲染环 4 片/帧预算 + 三面镜像 abi 0x0001000C + E206 --source 真跑 9/9（4+4+1 分帧）+ rebuild_tile_batch 抽取
- N1.3 `354a8e7`：**goldens-10 逐属性金标**（好坏对自证形；谓词全探针钉值：opacity 145/207、point 4672/232、clip 上下半、shadow 中带分裂）
- N1.5 `e55dd2f`：**EDL**（Frame.edl None=逐位零回归；中间纹理 + PIT-33 用域法；ON 2790px + 顶行 331.0→322.9 实测；WGPU-33+REND-40 条款；E204 --edl）
- N1.6 `ca6dbee`：画廊 three.js 式详情页 ×32（CARD 4=不部署纯本地）
- 审修 `60a24a3`：#1 详情页全名键（E811_c/_cpp 分页 + 撞号 fail-loud）+ #2 C17 21 行清零
- N2.0 `cd55fc1`：#3 evict_done+decoded_cached 对账（PIT-34）+ #4 center 移位清 uploaded/批（错位根除）
- 教训：PIT-33/34/35/36 + edit-safety #21/22/23 + testing.md HashMap 序纪律（本次入册）
- 基线：**297 passed · spec-trace 177↔177 · ABI 41/41 (0x0001000C) · GATE-DOCS ✓ · ci rc=0 · 领先 8 笔**
- 余账：push 候令 · N+2=morph 带+实体动画 6a+画廊分类扩充 · 择期=property_golden 补锚/镜像门声明形/多视口 EDL assert/point-radius 下限

## Session N+2/N+3 基线（2026-09-28 续，五笔：N2 四带 + 清扫）

- N2.1 `6d65346`：**投影 morph**（M，Momus OKAY + 卡 A 纯 rs）——render::morph 纯函数（端点逐字节律）+ Engine::set_morph_time（滑杆 clamp 语义，与 EdlSetup 拒式有意分叉）+ morph_spec 3 门 + E304 双模例（窗 [/] 滑 t；Xvfb 存活探针）。白皮书 §3.1 兑现，README"切换"→"连续投影 morph"（N1.2 诚实弧闭环）
- N2.2 `2ca8551`：**实体动画 6a**（S，C5 拆分）——render::anim::anim_origin（端点恒等 + f64 远原点精度）+ anim_spec 4 门 + E305 轨迹重放（24 标记像素门）。例面首跑抓归一化 t 误用=采样器时间域契约现形
- N2.3 `f10baff`：画廊 sticky 分类导航（three.js 浏览形，9 域锚点条；cards 键=title 的数字恢复教训）
- N2.4 `8b18937`：**属性热更 CAPI-36..38**（M，44 入口 minor=0x0001000D）——override 层 + override-first 读 + typed-reject（CORE-11/12）+ 三面镜像全带 + attr_diff_spec 2 门（fixture 抄 attr_ffi_spec 先例）。rs-only 计划升格 C 三口=测试可达性裁决（Engine 所有权住句柄表）
- 清扫 `0d430c4`：审 #5-#8 择期清零（多视口 EDL debug_assert / point-radius 双面带 / golden 无锚声明 / 镜像门声明形谓词三连自证；顶格 [[nodiscard]] 教训=自证样本必取真行）
- 基线：**306 passed · spec-trace 182↔182 · ABI 44/44 (0x0001000D) · 画廊 34 卡+34 详情页+导航条 · ci rc=0 · 领先 15 笔**
- 数字孪生三骨：属性热更 ✓ / 动画采样 6a ✓ / 序列化 S3' 挂账（Studio 依赖触发）
- 待办：触发制队列全空（几何 diff/框选/ODR/LAS/7 态机等票候令）
- **T3 人验（2026-09-28 用户实测）**：E304 `[/]` 滑投影 morph ✓ · E305 常驻窗 drone 循环重放 ✓（窗面缺口由用户抓出后补齐=eab5530）——两条销账，数字孪生演示面双验通过

## N4 后处理框架带基线（2026-09-28，计划 n4-post-framework-band.md v1.0·Momus OKAY）
- IR：REND-43 PostEffect{Bloom{strength},Outline{width}} + Frame.post Vec（空=逐位零回归旧路）+ bloom/outline 构造器域拒 Option（EdlSetup::new 同谱）；**46 处 Frame 构造点 sweep**（python 缩进感知插入 + contract.rs 手编）；contract_spec +域拒门
- 管线：WGPU-34 stage 链编排——ensure_stage_color 固定 2 槽（尺寸+格式缓存）+ per-(effect,format) 懒建管线 + bgl=EDL 4 绑定形共享（fullscreen_entries 提取共享描述符）；深度 Store 单旗 `edl_on||post_on`；**效果 i 写 stage[i+1]、末效果写最终 view、stage[0]=链输入永不作写目标**（首版写 stage[i] 撞 wgpu usage-scope RESOURCE+COLOR_TARGET 同 pass 冲突=validation error 抓出）
- 着色器：bloom.wgsl 9-tap luma 阈值加性（THRESHOLD=0.25 线性解码域探针定档——sRGB 编码 138/px≈线性 0.25）；outline.wgsl 深度梯度比（WGPU-33 rel 同族阈 0.08）+selection 橙；vs_main 三形同谱（fullscreen triangle strip from vertex_index）
- 例/门：E506_postprocessing 双模（1/2/3 键+标题回显；--frames off 确定性+bloom/outline 像素断言 probe=1289/1509px 阈 600/700）+ 四方消费全带（cmake 注册表 disp 缺省 headless/tutorials 行/画廊 37 卡/smoke-post）；post_chain.rs 四门（off 逐位/bloom 432/outline 678/链序 1038·1640 全探针在注）
- **web-check 抓真雷**：engine.rs wasm-专属 new_canvas 构造器被 sweep 脚本误插 post 字段（native check 不可达=cfg 门后面）——CI 后必跑 web-check 的活证据（R2 gz=539641B 记账）
- 基线：**311 passed（+5）· spec-trace 184↔184 · GATE-ABI 44/44 · GATE-DOCS ✓ E 36 件三方+37 卡 · ctest 37 条 · CMAKE-SMOKE ✓（Xvfb display ✓）· ci 十段 rc=0 · 9 crate**
- 余账：push 候令 · SSAO/DOF/SSR=框架后触发制 · 多视口 post（EDL 同款挂账）· >2 效果链=非目标

## T3 人验补录（2026-09-29，N3-N6 全带交互面）
- E508 拖拽轨道/滚轮缩放 ✓（rig 硬编码根修=733de61）· 1/2/3 特效键 ✓
- E507 tonemap/bloom 键 ✓ · E403 框选 ✓ · E306 map controls ✓ · E304 morph（此前已验）✓
- 结论：N3-N6 五带全部交互面双验通过，无遗留 T3 项

## V 系列 gap-closure 基线（2026-09-30，hyperplan→全量补全裁决→Block A/B 执行）

- 源起：hyperplan 对抗 5 员×3 轮（研究底料=three.js 608 例普查 + refinfo 13 项目 + self-audit）→ 全量补全裁决 → 计划 v1.1（`.omo/plans/gap-closure-full-2026-09-30.md`，4 员审核 10 修 4 险）→ 执行
- 审计正本（**2026-10-08 校正**）：本行原引「09-30 审计」与 `docs/gap-closure-roadmap.md` 为「双正本」，但前者（team-gap-analysis-2026-**09-30**.md）**从未入库**（git 全历史无此文件），后者当时也不存在。现役在盘正本 = `docs/reference/team-gap-analysis-2026-09-24.md`（S 级缺口表 + 例数普查）+ `docs/reference/team-gap-analysis-2026-09-28b.md`（three.js 608 例对标，取代 09-24 的例数段）+ `docs/gap-closure-roadmap.md`（2026-10-08 实写：逐条重测的覆盖图 / 未做清单 / 触发器 / 反向优势；09-30 当时的综合判断留在 gitignored 计划 `.omo/plans/gap-closure-full-2026-09-30.md`，不作库内引用）
- **W1**（`9fd64a2`）：REND-46 stroke 屏幕空间拾取谓词（+5 纯数学测；引擎接线诚实挂账——C pick 面 EntityId 与 seg 无 entity 表冲突 CAPI-01，伪装即违纪）
- **W2**（`e24b308`）：WGPU-38 depth haze——**raw depth 透视压缩雷探针现形**（16m≈0.994），SSAO view_z 米域线性化；三带几何感知双面锁（bg 12114 全雾/near 0 触碰）；E506 键 4
- **D**（`13035eb`）：四漂移对账（MVT/morph/场景树/Flutter-C#）+ **check-promises lint**（proximity 匹配器；self-test 破探针抓到第一版行级 allowlist 太粗）+ llms.txt/docs-gen；**ci 十段→十一段**
- **R**（`54f4e26`）：**白皮书 L53 WMS/WMTS 承诺兑现**（栅格面）——IO-18 `TilePayload{Mvt,Raster}` 枚举（反陷阱裁定=单车道泵/LRU 零分叉）+ CAPI-39/40 平铺 basemap（N1.2 纪律：NOT terrain）；五面镜像 abi 0x0001000E
- **I**（`49c2a95`）：**WGPU-39 SH-9 环境光照**（demo 级声明收窄；View 块 256→416B 尾缀方案零新 binding；PIT-41/42/43 三雷全踩全拆）；CAPI-41；E507 env probe
- **V2.1**（`88e2e90`）：REND-47 Bezier/CatmullRom+tube/ribbon（+0 ABI 消费带）——**E510 端点律断言首跑抓采样器真 bug**（末 knot 永不落地）；E510 四消费者+常驻窗补壳（`5e34bc0`，C15 第四例新变体=无窗；destroy_points 引擎口前置落地）
- **V2.2**（`68cc205`）：**CAPI-42/43 节点局部变换 + 存量根修**——glTF entity.world 层级矩阵 mount 期被丢弃（多节点层级从未正确渲染，E201 单层掩盖）；DrawItem.transform 全链贯通+刚性父链；五面镜像 abi 0x00010010（49 入口）；E511 旋转像素证明
- **V2.3 LAS=条件制未开**（真实 .las fixture 未落库）；**V2.4 蒙皮+morph=L 级候立项**（F1 提级在册，须独立 plan→Momus→批准）
- 基线：**343 passed · spec-trace 199↔199 · ABI 49/49 (0x00010010) · ci 十一段 · 画廊 43 卡 · 领先 origin 3 笔（gitee 鉴权失败待用户侧刷新）**
- 教训入册：PIT-41（多块 patch 截断雷，io-gltf 清空事故）/ PIT-42（WGSL uniform 三雷：闭包/let-if/array stride）/ PIT-43（flag 偏移漂移=探针职责）；后台代理五连败复盘（rpm 限流×3+stale-timeout×2，主会话直做 W1 40 分钟 vs 代理 3 轮全灭——小带直做优于派发）

## Install-tree band baseline (2026-10-08, packaging-round v1.3 P1-P4)

- Scope as user cut it: install tree only (npm/pip K0/K3/K4 out). Nine adjudication
  cards walked one at a time with the new adjudication-walkthrough skill; Momus
  re-reviewed v1.3 -> OKAY (0 blocking, 1 non-blocking dangling probe-B citation,
  fixed). Decisions in **D23**.
- P1 `c2fab6d`: SONAME stamped by the repo's first `build.rs`; gate-abi asserts it
  (expected name derived live from `[lib] name`). Measured payoff: absolute-path
  linking now yields `NEEDED=libvisiaengine.so` (was the full build path).
- P2a `af2608f`: `cmake/VisiaEngineInstall.cmake` + `visiaengineConfig.cmake.in`;
  S-c beta fail-loud guard retired in the facade (43 lines, gate-docs 5 budget OK);
  `cmake-smoke` negative state inverted into a positive install state.
- P2b `2d5fb2d`: pkg-config face; `.pc` URL read from Cargo.toml repository
  (missing line = configure FATAL, probed); prefix uses the `pcfiledir` form after
  a real bug (configure-time bake reported /usr/local) was caught and turned into
  three text assertions.
- P3 `8da46b3`: `scripts/gate-pack.sh` + `pixi run pack-check` (out of ci). All
  three assertions seen red by break probes; RUNPATH reported not judged because
  the leak was traced to this env's relocated conda `.pc` files (tinfo.pc naming a
  sibling checkout prefix) feeding a `-sys` link line -- a publish-band assembly
  concern, not a repo defect (user adjudicated P3=A against my written rec).
- P4 (this commit): README install wording + numbers 40->43 with **two new locks**
  (gallery cards in gate-docs 7, ctest count in cmake-smoke; both probe-red);
  `docs/tutorials.md` installed-consumption section; D23; AGENTS COMMANDS line.
- Incidental root-fix caught by the band's own web channel (`PIT-44` + the
  `web-mirror.mjs` staleness guard): the V2.2 wasm bridge `get_node_transform(&mut
  Vec<f64>)` never compiled for wasm32 -- host tests and the artifact-reading
  mirror script both missed it for two bands.
- Baseline: **ci rc=0, 343 passed, 11 segments; spec-trace 199<->199; GATE-ABI
  (49 entries + SONAME) OK; GATE-DOCS 7 checks OK; CMAKE-SMOKE four states OK
  (install tree 9 files, .pc text checks, ctest count lock); pack-check OK;
  pixi run web-check rc=0 + MIRROR 3/3 same-terminal; 9 crates unchanged;
  zero new dependencies (patchelf deliberately NOT added).**
- Open (declared, not swallowed): npm/pip packaging, staticlib consumer day,
  win/mac install forms, publish-band RUNPATH cleaning + CPack tarball,
  T3 human verification for E510/E511/E506 key4/E507.

## Band L baseline (ledger reconciliation, 2026-10-08, plan ledger-tiles-keys v1.1)

- Scope as adjudicated one card at a time (7/7, all to the lazy option; protocol
  `adjudication-walkthrough`). Plan `.omo/plans/ledger-tiles-keys-2026-10-08.md`; Momus OKAY
  (0 blocking, 1 non-blocking path-name nit, fixed in-plan).
- `e8ddfb5`: `docs/gap-closure-roadmap.md` written for real (census + refreshed coverage map +
  open gaps with triggers + reverse advantages) and the two dangling audit citations repaired.
- `0ef26fc`: haze gates' background reference self-calibrated from the frame's own corner pixel,
  with a >2% frequency guard; corner read back [13,18,25] = bit-identical behaviour, now proven.
- `4618896`: three existence locks — cited `docs/**` paths must exist (check-promises), README
  package count must equal `cargo metadata` (gate-docs 8, README fixed 10→11 crates/12 packages),
  no `let clear: [u8;3] = [` literals (gate-docs 9). Each lock shown red by a planted probe.
- Ledger truth today: **199 clauses / 49 C entries / abi (1<<16)|16 / 12 packages / 43 examples
  = ctest 43 = gallery 43 cards / ci 11 segments**, all four recomputed by the gates.
- Found while planning band T: the tile gap is the **single layer slot** in the capi engine
  (basemap XOR vector), not scheduling — see PIT-46. Band T proceeds with zero ABI change
  (Adjudication 2 = A: per-kind addressing).

## Band T + Band K baseline (2026-10-08, plan ledger-tiles-keys v1.1, all 7 cards = lazy option)

- **Band T (two-layer map)**: capi Engine went from one tile slot to `layers: Vec<TileLayer>`
  with an explicit `LayerKind`, per-kind view addressing, first-mount-only camera fit and
  `RASTER_UNDERLAY_Z` applied only when both kinds are mounted. **Zero ABI change held**
  (49 entries, abi `(1<<16)|16` untouched, no wasm/hpp work). Clauses CAPI-28/29/39/40 each
  carry a Band T revision sentence; spec-trace stayed 199<->199 (no new clause ids).
  - `d79bf78` engine + `tests/tile_layers.rs` (2 tests). Discrimination: single-slot revert
    -> `COEXIST both=0`, `set_raster_view=-1`, red at `tile_layers.rs:90`.
  - `23abf75` **E817_two_layer_map.c** — the visual witness: 5-segment live gate, families
    measured `raster_only basemap=69120 road=0 poi=0` / `vector_only road=10739 poi=1314
    basemap=0` / stacked v-then-r `9782/1314/26030` / stacked r-then-v `5343/684/60322` /
    re-mount memcmp == 0. Break probe: single-slot semantics -> `road=0 poi=0` FAIL rc=1.
  - Two plan premises died on contact: the proof cannot live in examples/rs (Engine is a
    private `mod`), and the two fixtures sit on opposite sides of the planet, so stacking
    them never shared a frame -> added `resources/data/raster_over` (the same four 79-byte
    PNGs re-keyed into the vector region, 316 bytes) instead of claiming a gate that cannot
    pass. Basemap witness = the EXCLUSION bucket, because the basemap's own colours satisfy
    E206's water/seam predicates (PIT-8 fifth sighting).
  - README: stacking claim added + its two numbers 43->44 measured from the same terminal
    (gallery "44 cards (17 thumbnails, 44 detail pages)", cmake-smoke count lock 44==44).
- **Band K (input probe becomes a repo tool)**: `scripts/xinject.c` + `scripts/keys-probe.py`
  + `pixi run keys-probe` (out of the ci chain, pack-check policy) and the previously
  caller-less `scripts/window-probe-all.sh` wired into `.github/workflows/ci.yml` L2.
  Measured: E506 key4 23.90%, E507 keys 2/3/1 = 5.87/29.85/29.85%, E510 drag 8.95% +
  wheel 9.18% (title `phase=43.0`->`56.9`), E511 clock 1.79% at a 4s gap, Escape exits all
  four. Break probe: stubbed `self.haze = !self.haze` -> `changed=0.00%` + rc=1.
  Two port bugs found by running it for real: the X needle is the E number (titles use a
  space, not the file stem), and XWD `header_size` includes the variable-length name so no
  constant guards it -- the length identity `off + h*bpl == len` does.
- Close-out `7e139f0` (PIT-47): the two new ci.yml L2 lines could only ever SKIP -- `xwd` ships in the system package `x11-apps` (not pixi) and the runner installed only `xvfb`, so the probes' own skip-on-missing-toolchain convention = a green not earned (PIT-25's CI-wiring variant). Fixed: apt installs x11-apps with a do-not-remove comment, keys-probe gained `--strict` (skip -> exit 1 naming the missing tool; ci.yml uses it, local default still skips), xwd found via PATH instead of a hardcoded /usr/bin. Branch tests: stub-only-xwd-missing + --strict -> rc=1 "missing toolchain: xwd"; default -> SKIP rc=0; real run --strict -> 4/4 rc=0. And window-probe-all.sh, wired a session ago and never run here, now has: rc=0, 20 resident windows, E509 lit=100.0% vs a 5.0% floor.
- Ledger now: **199 clauses / 49 C entries / abi (1<<16)|16 / 12 packages / 44 examples =
  ctest 44 = gallery 44 cards / ci 11 segments / 9 crates**.

## Band V baseline (shared viewer bootstrap, 2026-10-08, plan viewer-shared-band)

- Adjudicated 5/5 = A/A/B/C/A (one card at a time, protocol adjudication-walkthrough).
  Momus REJECTed v1.0 on one blocking item (canary E101 collided with C3b's protected
  specimen) and 6 non-blocking nits; all fixed in v1.1, then executed.
- New shared piece `examples/rs/src/viewer.rs` (206 non-blank lines): `argv_frames` /
  `argv_frames_and_rest`, `Ctx::new(title,w,h,FormatPolicy)` (window+adapter+device+
  surface+config+MeshCore), `on_resize`, `present`. Scope by ruling: **G1+G2 only**
  (C2=A). FormatPolicy is a parameter because 4 examples (E204/E206/E502/E503) prefer an
  Srgb format while 22 take the surface default -- collapsing that is how PIT-9 was born.
- Migrated **24/29** rs examples: E502(canary) E205 E501 E504 E901 E202 E402 E403 E201
  E301 E204 E206 E302 E304 E305 E510 E511 E303 E306 E505 (the last three of those
  five in `--bootstrap-only` mode: their `render_view_rects` present is the ⑤b lesson).
  Example bodies: 11,480 -> 10,104 non-blank lines = **-1,376** (883 of it moved into
  the 206-line shared piece, the rest is duplicated blocks collapsed to one call).
- Deliberately NOT shared: interaction semantics (drag means pick in E402/E403, phase in
  E510, per-viewport follow in E303) and the `render_view_rects` present of the ⑤b family
  (that call IS those examples' lesson). Protected manual specimens: **E101 + E503**
  (measured: only 9 of 26 window examples were fully unshared today; E201 could not be a
  specimen because it already consumed `examples::BBox`).
- V3 queue is closed: the five non-migrated files are excluded by evidence, not by
  preference -- E101/E503 (specimens, C3=B), E203 (positional CLI, never in scope),
  E401/E601 (no G1 bootstrap to share: zero Instance/adapter/Surface/winit; they already
  use the production `HeadlessBackend`, shared one level above the viewer shell; E601's
  lone `Instance::new` is the IR struct, not a wgpu Instance). The codemod refuses them
  with `!! no ApplicationHandler impl` **before writing** -- both files byte-identical
  to backups afterwards, which is edit-safety #26 item 3 executing, not being described.
  199 clauses, 49 C entries, 12 packages, 44 examples all unchanged.
- Baseline at close: `pixi run ci` rc=0 (11 segments) 345 passed · `keys-probe` 4/4 ·
  GATE-DOCS ok (44 cards) · CMAKE-SMOKE ok · spec-trace 199<->199 · worktree clean.
- Method worth reusing: per batch = determinism pre-check (two back-to-back captures) ->
  static scenes may then use **live-window sha equality** as the gate; animated examples
  must use keys-probe percentages / time-progress + interaction tri-state instead.

## lesson-review 记录（2026-10-08，band V 批 12–14 + V3/V4 收口会话）
- 新增：PIT-48（harness 调用错误被 `2>/dev/null` 吞 = 判成被测物死；rc 必先断言）、
  C19（自检/剪枝判据必须 token 级 + 上线前给检查本身种破坏探针；含如实注记：母本 codemod
  仍是粗判据，仓内真判据是 `lint -D warnings`）、D24（共享件价值以站点数计不以行数计；
  触发器实测覆盖计划存档步骤；V4 净额行数=零）。
- 扩写：testing.md harness-truth 第 (2) 条补**第四见**（E201 argv 位置资产打头，ad-hoc
  `--frames 1` 掉进开窗路，winit 报错长得像渲染 bug 实为跑道选择 bug）。
- 本轮回顾自己抓到的一条：新写的 C19 检查命令**首版是假门**（谓词连正例都不命中，
  且依赖会话级 /tmp 文件）；已改为自带破坏探针的可证伪形并实跑（种植样本=1，仓内=0）。
- 待办队列不变：push（V4 一笔）· T3 人验五条 · npm/pip 打包轮 · LAS/蒙皮票据制候令。

## Publish band ledger note (2026-10-09, plan next-band-queue todo 3)
- Live counters moved by this band, measured same-terminal: ci depends-on = **12**
  (`pack-check` promoted into the chain); install tree = **9 files**, unchanged --
  no new install rules; the CPack TGZ packages exactly what `cmake --install`
  produces (file-for-file diff recorded in `.omo/evidence/todo3-tarball-list.txt`,
  LOCAL artifact only, nothing published). Packaged lib: SONAME bare name +
  system-only NEEDED; RUNPATH names this env's sibling conda prefix = the D23 P3
  assembly concern, adjudicated report-only (no patchelf), decision note in
  `cmake/VisiaEnginePackaging.cmake`. Dated baselines above keep their
  as-of-close values (append-only history discipline).

## Next-band-queue baseline (2026-10-09, plan next-band-queue, todos 1-5)
- Four commits, one per todo, each leaving `pixi run ci` rc=0: `98bdbbe` census ·
  `0682fe3` keys-probe CASES 4→8 · `10c7dc7` CPack tarball + pack-check into ci ·
  `81d89d0` npm local artifact + gate-docs check 4 extended. Todo 5 = this write-back.
- **Todo 1 census** (`docs/reference/c-example-witness-census-2026-10-09.md`): the predicate
  was written into the doc *before* any counting, then applied to 17 tracked files =
  **6 WITNESS / 11 NO-witness** (`grep -c '| WIT[N]ESS |'` → 6, `'| N[O]-witness |'` → 11,
  sum 17). Structural claim: a pixel witness **requires a `readback` call**, and all 11
  NO-witness files have readback n=0. The WITNESS six: E701, E802, E813, E814, E816, E817.
  Of the 11, six still claim visible output (E702/E704/E801/E812/E815 call render, E703
  presents through the Qt pump), so a blanking or distortion regression there stays green.
- **Todo 2 keys-probe 4→8 rows** (`scripts/keys-probe.py:63`): E303/E306/E504/E901 added,
  each drag + wheel + one key. Floors = 0.4 × the row's weakest *measured* action min,
  floored (E303 15.46→6.0, E306 25.07→10.0, E504 13.24→5.0, E901 5.96→2.3); the floor is a
  per-action contract, not a row average. 8/8 green over 5 consecutive `--all` runs; each
  new row shown red by temporarily replacing its actions with `noop` (restore proven).
- **Keys-probe exclusions**, deliberate and recorded: E201_load_gltf, where Escape is a zoom
  key (`examples/rs/E201_load_gltf.rs:52`, `Escape => dist *= 0.9`), and E501_shadow_demo,
  which has zero keyboard handling (only a MouseWheel arm) -- the probe asserts Escape closes
  within 6s, so both would be permanently red. **Noop-ban rationale**: those four window rows
  must never use `noop`, because their idle frame is static, so a noop gate would print 0.00%
  and test the harness instead of the app. `noop` survives only on E511, whose scene clock is
  the thing under test.
- **Todo 3 publish band**: CPack TGZ packages the install tree **file-for-file identical to
  `cmake --install`, 9 files**, with no new install rules (diff recorded in
  `.omo/evidence/todo3-tarball-list.txt`; LOCAL artifact, nothing published or pushed).
  SONAME bare-name consumer witness: E701's `NEEDED` line reads `libvisiaengine.so`. RUNPATH
  naming this env's sibling conda prefix = the D23 P3 assembly concern, documented in
  `cmake/VisiaEnginePackaging.cmake` and adjudicated report-only -- **no patchelf**, no new
  tool dependency.
- **Todo 4 npm**: `build/npm/visiaengine-0.1.0.tgz` = **11 files**, version read from
  `Cargo.toml` `[workspace.package]` by `scripts/gen-npm-package.sh` (never hand-typed). The
  builder is a pixi task **out of ci** (same policy as gallery / keys-probe), and the
  internal-link gate was folded into **gate-docs check 4 in place** (`.md-only` → every
  markdown link in README / docs / llms.txt). So **ci 11→12 came from pack-check alone**;
  this todo added no segment.
- Ledger at close, every value recomputed same-terminal: **ci 12 segments · spec-trace
  199↔199 · ctest 44 · gallery 44 cards · install tree 9 files · crates/ 9 (workspace 12
  packages)**. Only the ci segment count moved this band.
- **Open items (registered, not fixed)**: `examples/c/E814_labels_headless.c:48` --
  `if (white < 60) fail(...)` drops the return, so that census row's pixel bound cannot
  redden ctest; keys-probe's single-shot `noop` invocation prints a cosmetically wrong
  usage/error line (no gate impact); an exit-key convention band for E201/E501 would unlock
  two more probe rows; the npm package name was never checked against a registry (deliberate
  offline stance, nothing pushed); the npm `exports` map is deferred.
