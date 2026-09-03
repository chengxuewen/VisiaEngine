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
