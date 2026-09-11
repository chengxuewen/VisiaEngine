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
