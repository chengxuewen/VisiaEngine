---
name: performance-optimization
description: "Generic performance profiling and optimization across four dimensions: latency tracing, throughput benchmarking, web UI render profiling, and benchmark regression detection. Use when latency spikes, after core pipeline changes, or before release."
---

# performance-optimization — Performance Optimization

> Latency + throughput + rendering + benchmark regression. Four dimensions, one method.
> Measure first, optimize second. Never optimize guesses.
> `<src>` / `<crate>` / `<binary>` = the project's actual paths and targets; substitute per chosen stack.

## Trigger Conditions

- End-to-end latency exceeds the project SLO
- Throughput drops >10%
- Web UI render frame drops
- After core pipeline code changes
- The user says "performance" / "latency" / "slow" / "优化性能"
- Pre-release benchmark regression

## Golden Rules

```
1. measure the baseline (bench harness / browser trace)
2. locate the bottleneck (flamegraph / perf / DevTools)
3. single-variable optimization (one variable at a time)
4. verify the regression (re-measure, compare against baseline)
5. record the decision (decisions.md)
```

## Phase 1: Benchmarking (backend layer)

```bash
# run all benchmarks (cargo as example; any bench harness works the same way)
cargo bench --workspace

# specific target
cargo bench -p <crate>

# compare pre/post-change
cargo bench -- --save-baseline before
# ... make changes ...
cargo bench -- --baseline before
```

Key baseline setup (criterion style):

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

### Profiling

```bash
cargo flamegraph --bin <binary> -- <args>   # flamegraph (Linux, needs perf)
valgrind --tool=cachegrind <command>        # instruction/cache hotspots
```

Code-level timing — use the project's existing metrics module, or minimal inline timing:

```rust
use std::time::Instant;
let start = Instant::now();
// ... operation ...
// record start.elapsed() under a named metric, e.g. "op_latency_us"
```

## Phase 2: Latency-chain analysis

For any multi-stage pipeline (request→response, input→present): first name every hop, then instrument each hop.

```
[entry] ──t1──> [processing A] ──t2──> [serialize/packetize] ──t3──> [network/IO send]
                                                                │
[exit]  <──t6── [processing B] <──t5── [deserialize/parse] <──t4── [network/IO receive]
```

Method:

```bash
# check whether existing instrumentation covers every hop
grep -rn 'metrics\|latency\|Instant::now' <src>/ --include='*.rs'
```

- a hop without instrumentation → instrument first, optimize later (blind optimization = guessing)
- time each hop separately, find the longest stretch, optimize only that stretch
- treat network/IO boundaries separately: distinguish "our code is slow" from "the environment/peer is slow"

### Optimization pattern (allocation hotspots)

```rust
// BEFORE: alloc on every hot-path call
fn send_item(&mut self, item: &Item) {
    let buf = build_buffer(item);      // alloc per call
    self.transport.send(buf);
}

// AFTER: reuse a pre-allocated buffer
fn send_item(&mut self, item: &Item) {
    self.buf.clear();
    build_buffer_into(item, &mut self.buf);
    self.transport.send(&self.buf);
}
```

Only do this when timing proves allocation is the bottleneck; buffer reuse trades away simplicity and must be backed by data.

## Phase 3: Throughput profiling

- measure under steady load: concurrency, messages/sec, bytes/sec, error rate
- ramp pressure step by step until the knee — record which resource saturates (CPU / memory / bandwidth / fd / locks)
- the server's own stats/metrics endpoint beats external guessing

```bash
# find the project's stats/metrics outlets and key metric instrumentation
grep -rn 'stats\|metrics\|throughput\|bitrate' <src>/ --include='*.rs'
```

- every buffer/pool/queue cap needs a numeric justification + a matching metric (visible when full)
- for bandwidth-cap parameters: state the default and the "lower-latency" value, set the tier after load-testing

## Phase 4: Web UI render optimization

Browser performance tracing (DevTools / Playwright trace):

```
1. navigate → target page
2. performance.mark('start') → interact → performance.measure('render', 'start')
3. watch long tasks (>16ms = one dropped frame @60fps)
```

### Generic checklist (React as example; other frameworks apply the same)

| Problem | Detection | Fix |
|------|------|------|
| recursive re-render | DevTools Profiler → flame chart | `React.memo` + `useCallback` |
| expensive compute in render | wrap the render body with `console.time` | `useMemo` |
| big list without virtualization | Items >100 and not virtualized | virtual-scroll component |
| push-message bombardment | >60 updates/sec | batch updates (requestAnimationFrame throttle) |
| uncleaned subscriptions | no useEffect cleanup | return `() => ws.close()` |
| Context spread | Context value holds frequently-changing objects | split the Context / use a ref |

```bash
# rough memoization ratio (useMemo+useCallback+memo vs useState+useEffect should be >0.5)
grep -rn 'useState\|useEffect' <ui-src>/ --include='*.tsx' | wc -l
grep -rn 'useMemo\|useCallback\|memo' <ui-src>/ --include='*.tsx' | wc -l
```

```javascript
// execute in the browser:
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

## Phase 5: CI regression detection

```yaml
# add to .github/workflows/ci.yml (cargo as example):
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
        alert-threshold: '130%'  # alert on >30% regression
```

Local pre-bench checks:

```bash
cargo check --workspace --all-features                    # compile check
cargo clippy --workspace --all-features -- -D warnings    # no needless alloc/clone introduced
cargo bench --workspace -- --quick                        # quick bench (under-sampled, sanity only)
```

## Report format

```
## Performance analysis report — [date]

### Baseline comparison
| Benchmark | Before | After | Delta |
|------|--------|-------|-------|
| core_encode | 3.2ms | 3.1ms | -3% |
| packetize | 0.8ms | 0.4ms | -50% ✅ |

### Findings
- [hotspot] <item>: <technique>, <gain>
- [regression] none / <item> <magnitude>
- [bottleneck] <location> ~<number> (<known-issue number or deferred note>)

### Recommendations
- [P0] ...
- [P1] ...
```

## Forbidden

- optimizing without measuring (blind optimizations usually introduce new problems)
- optimizing multiple variables at once (attribution becomes impossible)
- sacrificing readability for insignificant gains
- micro-benchmarks detached from real usage scenarios
- ignoring CI performance-regression alerts
- trading `unsafe` for performance without verification
