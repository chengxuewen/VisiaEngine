# spike-2：10 万实体 slab 遍历+抽脏帧预算（S1 附带）

**日期** 2026-09-03 | **来源** 计划 visia-scaffold-kickoff §7 | **状态** 数据点（非门禁）

机器：开发机（linux-64，conda rust 1.98.0，`cargo test --release`）。
实现：手搓 Vec+free-list+代际（无外部 slab crate），组件 v0 单变体 enum。

| 操作 | 10 万实体 |
|---|---|
| spawn ×100k（顺序分配） | **6.6 ms** |
| mark_dirty×100k + take_dirty + alive_ids 遍历 | **5.7 ms** |

结论：全量标脏+双遍历 ≈ 12.3ms，处于 60fps(16.7ms) 帧预算内但无余量——**验证"slab 起步不上 ECS"量级可行**，同时坐实架构③"脏标记驱动局部重建"的必要性：稳态帧只处理真脏集（远低于全量），全量扫描只在异常/首帧发生。RTC/索引等热路径设计（P1）需以此为基线。
复测：`pixi run cargo test -p visia-core --release -- --ignored --nocapture`
