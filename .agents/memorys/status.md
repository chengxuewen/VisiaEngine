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
