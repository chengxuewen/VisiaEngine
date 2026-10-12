# spike-2: 100k-entity slab walk + dirty-drain frame budget (S1 companion)

**Date** 2026-09-03 | **Source** plan visia-scaffold-kickoff §7 | **Status** data point (not a gate)

Machine: development box (linux-64, conda rust 1.98.0, `cargo test --release`).
Implementation: hand-rolled Vec+free-list+generation (no external slab crate), v0 component a single-variant enum.

| operation | 100k entities |
|---|---|
| spawn ×100k (sequential allocation) | **6.6 ms** |
| mark_dirty×100k + take_dirty + alive_ids walk | **5.7 ms** |

Conclusion: full-slate dirty + double walk ≈ 12.3ms, inside the 60fps(16.7ms) frame budget but with no headroom — **validates that "start on a slab, no ECS" is feasible at this magnitude**, and simultaneously confirms the necessity of architecture ③ "dirty-flag-driven partial rebuild": a steady-state frame processes only the truly dirty set (far below full scale), a full walk occurs only on anomaly/first frame. The RTC/index hot-path designs (P1) must baseline against this.
Re-measure: `pixi run cargo test -p visia-core --release -- --ignored --nocapture`
