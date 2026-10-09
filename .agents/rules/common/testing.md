# Testing Requirements

## Minimum Test Coverage: 80%

Test Types (ALL required):
1. **Unit Tests** - Individual functions, utilities, components
2. **Integration Tests** - API endpoints, database operations
3. **E2E Tests** - Critical user flows (framework chosen per language)

## Test-Driven Development

MANDATORY workflow:
1. Write test first (RED)
2. Run test - it should FAIL
3. Write minimal implementation (GREEN)
4. Run test - it should PASS
5. Refactor (IMPROVE)
6. Verify coverage (80%+)

## Troubleshooting Test Failures

1. Use **tdd-guide** agent
2. Check test isolation
3. Verify mocks are correct
4. Fix implementation, not tests (unless tests are wrong)

## Agent Support

- **tdd-guide** - Use PROACTIVELY for new features, enforces write-tests-first

## Test Structure (AAA Pattern)

Prefer Arrange-Act-Assert structure for tests:

```typescript
test('calculates similarity correctly', () => {
  // Arrange
  const vector1 = [1, 0, 0]
  const vector2 = [0, 1, 0]

  // Act
  const similarity = calculateCosineSimilarity(vector1, vector2)

  // Assert
  expect(similarity).toBe(0)
})
```

### Test Naming

Use descriptive names that explain the behavior under test:

```typescript
test('returns empty array when no markets match query', () => {})
test('throws error when API key is missing', () => {})
test('falls back to substring search when Redis is unavailable', () => {})
```

## 可执行检查

```bash
# 验证：测试通过
cargo test --workspace
# 验证：覆盖率
cargo tarpaulin --workspace 2>/dev/null || echo "tarpaulin not installed"
```

## 本仓测试三层法（E3D:D5 纪律，2026-09-03 计划 v1.1 批次 0a）

| 层 | 定义 | 命令 | 门禁位置 |
|----|------|------|---------|
| **T1 纯单元** | 无 GPU/窗口的逻辑断言（解析/数学/场景）| `cargo test --workspace` | `pixi run ci` |
| **T2 帧预算 auto-smoke** | `--frames N` 离屏/窗口示例真执行 | `xvfb-run -a pixi run smoke-clear`（smoke-* 全族） | `pixi run ci`（L2 段）|
| **T3 人工交互（诚实层）** | 无法自动化的交互验证（鼠标拾取、飞行相机等）——测试体 `#[ignore]` 且**运行时必须打印人检步骤清单**，报告时明说"T3 未跑/已人验" | `cargo test --workspace -- --ignored`（当前允许空集，命令合法空跑=通过） | 合并前人检 |

原则：任何"测试通过"声明必须指明层级；T3 项禁止被 T1/T2 绿灯冒充（verification-honesty 条款的测试面投影）。

## 视觉例双保险（2026-09-16 教训升级：交互窗从未被像素验证案族）

窗口/画面类 example 的 exit-code+计数门（`loaded N entities`/rc=0）**不是画面证据**——本会话三案：E201 相机焊错机位全灰屏、E301 键鼠写死变量、golden 族中心亮度 >40 被暗背景自身 57 分骗过。纪律：

1. **headless 像素门**：每个视觉例至少一条离屏镜像断言（颜色/计数级、保守阈=实测 −40%，专捕回归不追审美）；先例 `crates/visiaengine-render-wgpu/tests/gltf_scene.rs`（装载→拟合→红绿计数）。新例出生带同 commit 落门。
2. **断言谓词三自证**：能命中正例（旧机位必红一次）；不吃背景值（阈值贴背景之上留距）；亮度带族慎用「非背景即过」形（本案骗过例）。
3. **T3 人验清单**：交互面（拖/滚/关窗/resize）机器不可达，交付文案带人验步骤清单（既有 `#[ignore]` 运行时打印纪律的窗口例投影）；验证环境=用户通道形（PIT-22），非工具 shell 手加 export。
4. **ctest 转发壳**：`-R` 必配「≥1 被选」断言 + `--timeout`（PIT-23 互引），空匹配 exit-0 = 恒假绿。

```bash
# 验证：视觉例像素门在场（例名→测试映射人工核 + 转发壳断言 grep）
grep -rn 'out of [1-9]' scripts/smoke-rs.sh        # 转发壳 ≥1 断言在位
grep -rn 'lum >\|red >\|green >' crates/*/tests/*.rs | wc -l   # 像素门计数（增删随带对账）
```

### HashMap 迭代序测试纪律（2026-09-28 N1.4 带实锤）
涉及 HashMap 驱动的多对象断言（如 pump 逐出顺序）：单测内**不得**对迭代序做可观测断言——本机单跑绿、CI 全量跑翻车（PIT-7 族假红的近亲）。确定性语义（budget=每次 I/O 数）用**脚本化 source 计数**锁（attempts.len()==budget），顺序本身不锁。
**验证**: 同测试连跑 `for i in 1 2 3 4 5; do cargo test ... ; done` 全绿再提交。
**阻塞条件**: 修复 HashMap 序型偶发后未 5 连跑。

## Web/wasm 验证通道（PIT-44 入册，2026-10-08）

`pixi run ci`（11 段）**不含** web 面。wasm 相关的三条硬规矩：

1. **声明口径**：任何「WEB ✓ / MIRROR ✓」字样只能来自**同终端** `pixi run web-check`
   （+ 需要名册时 `node scripts/web-mirror.mjs`）的实际输出。宿主 cargo test 不算。
2. **编译面不可达性**：`&mut Vec<T>` / `Vec<T>` 出参这类 wasm-bindgen -only 约束，
   host target 编得过、wasm32 编不过。凡动 `bindings/js/rust/visiaengine-wasm/src/`
   的签名，切片内必须跑一次 `pixi run web-check`（不是 cargo check）。
3. **陈旧护栏在场**：`scripts/web-mirror.mjs` 会比对源目录与 `pkg-node/*.d.ts` 的 mtime，
   源更新即红并指名补救。该检红 ≠ 门禁坏，是「你在拿昨天的产物报今天的绿」。

```bash
# 验证：护栏在位 + 声明来源可查
grep -c "陈旧" scripts/web-mirror.mjs          # 期望 ≥1
grep -n "^ci = " pixi.toml                     # 期望：无 web-check（本条存在的原因）
```

## XTEST input probe = T3-input promoted to T2 (band K, 2026-10-08)

`pixi run keys-probe` (scripts/keys-probe.py + scripts/xinject.c) injects real
key/wheel/drag events into a live window on a **private** Xvfb display, then fails
when the window stays alive but the picture never changes. Measured output of the
committed set: E506 key 4 = 23.90%, E507 keys 2/3/1 = 5.87/29.85/29.85%,
E510 drag = 8.95% (+ title `phase=43.0`), E511 clock at a 4s gap = 1.79%, and
Escape closes all four through the same input path. Break probe recorded: stubbing
`self.haze = !self.haze` in E506 printed `changed=0.00%` and exited 1.

| Now machine-checked (T2) | Still human (T3) |
|---|---|
| a key/handler is wired to something the frame reads | does the result *look right* (aesthetics, colour taste) |
| the title/state echo updates | is the motion smooth (frame pacing, not pixel deltas) |
| drag/wheel change the view at all | does it read as the thing it claims to be |
| Escape closes the window | resize-while-dragging, multi-monitor, WM oddities |
| zero-arg run is resident (C15 breach detector built in) | anything needing a second app or a real GPU feel |

Harness-truth rule (band V, three false alarms in one session, each from MY run and
not from the code): before calling a gate red, confirm the gate was pointed at the
thing the example claims -- (1) display-family examples must run under Xvfb (their
`--frames N` is a window frame counter, not a headless lane: batch 4 read 3/3 FAILED
with no DISPLAY); (2) run the lane with the **registry argv** (`_args_<stem>` in
cmake/VisiaEngineBindings.cmake), not an ad-hoc number -- E305's marker threshold is
calibrated for 24 frames and panics at 4 on both the migrated and the pre-migration
build (batch 11); **fourth sighting (V4): E201's argv is
`resources/data/twoprim.glb --frames 3` -- a POSITIONAL ASSET FIRST. My ad-hoc
`--frames 1` dropped the model path, which silently switched that example into its
window lane and produced winit's "neither WAYLAND_DISPLAY nor DISPLAY is set" --
an error that reads like a rendering bug and is actually a lane-selection bug. Rule:
copy the registry argv VERBATIM, argument order included.**; (3) X keysym names are lowercase (`bracketright`), winit-style
`BracketRight` silently does nothing while the injector prints ok.

Rules baked into the tool (keep them if you port it): private display, never
`$DISPLAY` (PIT-22); assertions happen before cleanup; per-case settle/gap so a slow
clock is not read as "nothing changed"; SKIP exit 0 only for a missing toolchain
(Xvfb / xwd / cc / `XTest.h`) -- and `--strict` turns that skip into exit 1, which is
how ci.yml calls it: `xwd` ships in the SYSTEM package `x11-apps`, not in pixi, so a
runner that installs only `xvfb` would otherwise skip the whole probe and still print
green (PIT-25). A live window with no delta is exit 1.
It stays **out of `pixi run ci`** (same policy as `pack-check`) and is wired into
`.github/workflows/ci.yml` next to `scripts/window-probe-all.sh`, which had zero
callers until this band.

## Batch-migration gates (band V, 2026-10-08)

Mechanical edits across many examples are only as safe as the pixel gate attached to
them. The order that worked (and caught every real defect this band):

1. **Pre-check determinism before using it as evidence**: capture the same window twice
   back-to-back on the *unmodified* build. Equal -> a byte-identity gate is legal.
   Unequal (E305/E510/E511: the scene is animated) -> byte gates are BANNED for that
   example; use keys-probe percentages, time-progress ("the frame must change"), or the
   interaction tri-state (idle / drag / wheel must be three distinct hashes).
2. Capture the before-frame BEFORE editing. If you forgot, rebuild from git (checkout ->
   build -> capture -> restore), so the comparison is still pre-vs-post, not post-vs-post.
3. Card `cmp` only proves the headless lane; the migrated code is usually the *window*
   lane. A static sha match does not prove input still works (a shared bootstrap can pass
   frames while killing handlers) -- hence the interaction tri-state in every batch.
4. Run display-family examples under a display and with the **registry argv**
   (`_args_<stem>` in cmake/VisiaEngineBindings.cmake). Both mistakes happened this band
   and each produced a red that was about my harness, not the code.
5. For import-pruning steps inside a codemod, the real judge is `pixi run lint`
   (`cargo clippy --workspace --all-targets -- -D warnings`, pixi.toml L76) -- a symbol
   kept alive only by a comment mention becomes an unused-import **error** there. Prove
   the prune predicate bites before trusting it: plant a sample, the check must hit it
   (conventions.md C19 addendum).
