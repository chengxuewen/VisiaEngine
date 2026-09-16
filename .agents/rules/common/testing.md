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
