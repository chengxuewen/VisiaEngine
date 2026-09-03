---
name: performance-optimization
description: "Generic performance profiling and optimization across four dimensions: latency tracing, throughput benchmarking, web UI render profiling, and benchmark regression detection. Use when latency spikes, after core pipeline changes, or before release."
---

# performance-optimization — 性能优化

> 延迟 + 吞吐 + 渲染 + 基准回归。四个维度，一套方法。
> 先测量，再优化。不优化猜测。
> `<src>` / `<crate>` / `<binary>` = 项目实际路径与目标，按选定栈替换。

## 触发条件

- 端到端延迟超出项目 SLO
- 吞吐下降 >10%
- Web UI 渲染掉帧
- 核心管线代码变更后
- 用户说 "performance" / "latency" / "slow" / "优化性能"
- 发布前基准回归

## 黄金法则

```
1. 测量基线 (bench harness / browser trace)
2. 定位瓶颈 (flamegraph / perf / DevTools)
3. 单变量优化 (一次只改一个变量)
4. 验证回归 (重新测量，对比基线)
5. 记录决策 (decisions.md)
```

## Phase 1: 基准测试（后端层）

```bash
# 运行所有基准 (以 cargo 为例，任何 bench harness 同理)
cargo bench --workspace

# 特定目标
cargo bench -p <crate>

# 对比 pre/post-change
cargo bench -- --save-baseline before
# ... make changes ...
cargo bench -- --baseline before
```

关键基线设置（criterion 风格）：

```rust
// benches/<name>_bench.rs
use criterion::{black_box, Criterion};

fn bench_hot_path(c: &mut Criterion) {
    c.bench_function("core_encode", |b| {
        let input = generate_test_input();
        b.iter(|| op(black_box(&input)))
    });
}
```

### 性能剖析

```bash
cargo flamegraph --bin <binary> -- <args>   # flamegraph (Linux, 需要 perf)
valgrind --tool=cachegrind <command>        # 指令/缓存热点
```

代码级计时 — 使用项目已有的 metrics 模块，或最小内联计时：

```rust
use std::time::Instant;
let start = Instant::now();
// ... operation ...
// record start.elapsed() under a named metric, e.g. "op_latency_us"
```

## Phase 2: 延迟链路分析

对任何多阶段管线（请求→响应、输入→呈现）：先命名每一跳，再逐跳打点。

```
[入口] ──t1──> [处理A] ──t2──> [序列化/打包] ──t3──> [网络/IO 发送]
                                                                │
[出口] <──t6── [处理B] <──t5── [反序列化/解析] <──t4── [网络/IO 接收]
```

方法：

```bash
# 检查现有打点是否覆盖每一跳
grep -rn 'metrics\|latency\|Instant::now' <src>/ --include='*.rs'
```

- 无打点的跳 → 先补打点，再谈优化（盲优化 = 猜）
- 每跳单独计时，找最长的一段，只优化那一段
- 网络/IO 边界单列：区分"我们的代码慢"与"环境/对端慢"

### 优化模式（分配热点）

```rust
// BEFORE: 热路径每次 alloc
fn send_item(&mut self, item: &Item) {
    let buf = build_buffer(item);      // alloc per call
    self.transport.send(buf);
}

// AFTER: 复用预分配 buffer
fn send_item(&mut self, item: &Item) {
    self.buf.clear();
    build_buffer_into(item, &mut self.buf);
    self.transport.send(&self.buf);
}
```

仅当计时证明分配是瓶颈时才做；buffer 复用牺牲简单性，须有数据支撑。

## Phase 3: 吞吐剖析

- 稳态负载下测：并发数、消息/秒、字节/秒、错误率
- 逐步加压直到拐点 —— 记录饱和的资源（CPU / 内存 / 带宽 / fd / 锁）
- 服务端自身的 stats/metrics 出口优先于外部猜测

```bash
# 找到项目的 stats/metrics 出口与关键指标打点
grep -rn 'stats\|metrics\|throughput\|bitrate' <src>/ --include='*.rs'
```

- 每个缓冲/池/队列的上限都要有数值理由 + 对应度量（满了能看到）
- 带宽上限类参数：给出默认值与"降延迟"取值的对照，压测后定档

## Phase 4: Web UI 渲染优化

浏览器性能追踪（DevTools / Playwright trace）：

```
1. navigate → 目标页面
2. performance.mark('start') → 交互操作 → performance.measure('render', 'start')
3. 观察 long task（>16ms = 掉一帧 @60fps）
```

### 通用清单（以 React 为例，其他框架同理）

| 问题 | 检测 | 修复 |
|------|------|------|
| 递归重渲染 | DevTools Profiler → 火焰图 | `React.memo` + `useCallback` |
| 昂贵计算 in render | `console.time` 包裹 render 体 | `useMemo` |
| 大列表无虚拟化 | Items >100 且未虚拟渲染 | 虚拟滚动组件 |
| 推送消息轰炸 | 每秒 >60 条更新 | 批量更新 (requestAnimationFrame throttle) |
| 未清理的订阅 | 无 useEffect cleanup | return `() => ws.close()` |
| Context 扩散 | Context value 含频繁变化对象 | 拆分 Context / 用 ref |

```bash
# memoization 比例粗查（useMemo+useCallback+memo 对 useState+useEffect 应 >0.5）
grep -rn 'useState\|useEffect' <ui-src>/ --include='*.tsx' | wc -l
grep -rn 'useMemo\|useCallback\|memo' <ui-src>/ --include='*.tsx' | wc -l
```

```javascript
// 在浏览器中执行:
const observer = new PerformanceObserver((list) => {
  for (const entry of list.getEntries()) {
    if (entry.duration > 16) { // >1 frame (60fps)
      console.warn('Long task:', entry.duration.toFixed(1) + 'ms', entry.name);
    }
  }
});
observer.observe({ entryTypes: ['measure', 'longtask'] });
performance.mark('monitoring-start');
```

## Phase 5: CI 回归检测

```yaml
# .github/workflows/ci.yml 添加（cargo 为例）:
perf-regression:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - run: cargo bench --workspace -- --output-format bencher | tee bench-output.txt
    - uses: benchmark-action/github-action-benchmark@v1
      with:
        tool: 'cargo'
        output-file-path: bench-output.txt
        github-token: ${{ secrets.GITHUB_TOKEN }}
        auto-push: true
        alert-threshold: '130%'  # >30% 退化则告警
```

本地 pre-bench 检查：

```bash
cargo check --workspace --all-features                    # 编译检查
cargo clippy --workspace --all-features -- -D warnings    # 不引入不必要的 alloc/clone
cargo bench --workspace -- --quick                        # 快速基准 (采样不足，仅快速验证)
```

## 报告格式

```
## 性能分析报告 — [日期]

### 基准对比
| 基准 | Before | After | Delta |
|------|--------|-------|-------|
| core_encode | 3.2ms | 3.1ms | -3% |
| packetize | 0.8ms | 0.4ms | -50% ✅ |

### 发现
- [热点] <项>: <手法>, <收益>
- [回归] 无 / <项> <幅度>
- [瓶颈] <位置> ~<数值> (<已知问题编号或 deferred 标注>)

### 建议
- [P0] ...
- [P1] ...
```

## 禁止

- 不测量就优化 (猜测优化通常引入新问题)
- 多变量同时优化 (无法归因)
- 优化牺牲可读性而无显著收益
- micro-benchmark 脱离真实使用场景
- 忽略 CI 性能退化告警
- `unsafe` 换性能而无验证
