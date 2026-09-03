# visia-core 行为契约（SDD）

> 测试注释携带 `// spec: CORE-NN`；双向追溯门禁 `scripts/spec-trace.sh`。
> 坐标重基（RTC）测试留 P1 裁定后追加（architecture.md 未决点），不预写。

## CORE-01: create_get_roundtrip
`spawn()` 分配实体，`insert(id, Component)` 挂载，`get(id)` 返回同值组件；同帧内幂等。

## CORE-02: delete_invalidates_handle
`despawn(id)` 后该 id 的 `get`/`insert`/`mark_dirty` 一律返回 `CoreError::NotFound`，`is_alive` 为 false。

## CORE-03: stale_generation_rejected
槽位回收后旧 handle 携带过期代际，任何访问返回 `NotFound`（不得触到新主数据）。

## CORE-04: slot_reuse_after_delete
despawn 腾出的槽位由后续 spawn 复用：`slot` 相同、`generation` 严格 +1。

## CORE-05: iteration_yields_alive_only
`alive_ids()` 恰产出全部存活实体，无空槽、无已删实体。

## CORE-06: dirty_flag_coalesces
同一实体同帧多次 `mark_dirty` 只记一次脏（`take_dirty` 结果去重为单条）。

## CORE-07: take_dirty_clears
`take_dirty()` 产出当前脏清单并清空标记；紧接第二次取为空。

## CORE-08: f64_position_preserved
`Vec3` 全 f64：≥1e7 量级坐标与任意二进制分数往返零损耗（不丢精度、不转 f32）。

## CORE-09: component_missing_err
存活但无组件的实体 `get` 返回类型化 `CoreError::MissingComponent{ component: "Transform" }`，不 panic。

## CORE-10: insert_replaces_component
同实体重复 `insert` 同型组件为替换语义（get 恒得最后值），非多值叠加。
