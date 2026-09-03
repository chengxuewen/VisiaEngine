---
name: api-interface-design
description: "Contract-first interface design for polyglot projects: trait/interface boundaries, WebSocket signaling protocols (tagged serde enums), and REST APIs (OpenAPI spec). Enforces protocol backward compatibility, module-boundary contracts, and wire-format discipline. Use BEFORE adding new API endpoints, WS message types, or cross-module trait changes."
---

# api-interface-design — Contract-First API Design

> Define the contract BEFORE the implementation. Traits, messages, and endpoints are the architecture — code is decoration.

## API Boundary Types

Most projects have three distinct API contract surfaces:

| Boundary | Form | Typical Location | Contract |
|----------|------|----------|----------|
| **Internal module traits/interfaces** | Language-level interfaces | Core library modules | Signature stability |
| **WebSocket signaling** | JSON enum | Shared protocol module | `#[serde(tag = "type")]` discipline |
| **REST API** | OpenAPI spec | `docs/openapi.yaml` | Schema + validation |

## Design Protocol

### Phase 1: Scope the Change

Which boundary is affected?

```
Internal trait change  → core library traits
WS protocol change     → shared protocol module
REST endpoint change   → OpenAPI spec + server routes
Cross-boundary         → Draft all contracts FIRST
```

### Phase 2: Write the Contract

**Trait/Interface Contract:**

```rust
// ✅ CORRECT: minimal, composable trait
pub trait Component: Send + Sync {
    fn name(&self) -> &'static str;
    fn start(&mut self) -> Result<(), ComponentError>;
    fn stop(&mut self) -> Result<(), ComponentError>;
}

// ❌ WRONG: implementation leaks, too many methods
pub trait Component: Send + Sync + Debug + Clone { /* 15 methods */ }
```

Rules:
- One trait = one responsibility
- Use `thiserror` for error types (library crates)
- `&str` over `String` for params (borrowing)
- `&[T]` over `Vec<T>` for collections
- Never expose internal types in public API
- `#[non_exhaustive]` on enums meant for extension

**WebSocket Protocol Contract:**

```rust
// ✅ CORRECT: tagged enum, snake_case, backward-compatible
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalMessage {
    // Existing variants unchanged
    Join { room_id: String, role: Role },

    // NEW: additive only, no field renames
    Stats { room_id: String, fps: f64, bitrate_kbps: u64 },
}

// ❌ WRONG: camelCase tags (browser mismatch), removed fields
#[serde(tag = "type")]  // missing rename_all → camelCase default
pub enum SignalMessage {
    Join { room_id: String },  // role removed = breaking
}
```

Rules:
- **ALWAYS** `#[serde(tag = "type", rename_all = "snake_case")]`
- Browser clients MUST send snake_case type tags — the wire format is a cross-language contract, both sides must agree
- Additive changes only — never rename/remove fields
- New variants append to the END of the enum
- `Option<T>` for new fields on existing variants
- Protocol changes → update E2E test scripts

**REST API Contract (OpenAPI 3.0.3):**

```yaml
# docs/openapi.yaml
paths:
  /api/health:
    get:
      summary: Server health check
      responses:
        '200':
          description: OK
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HealthStatus'

components:
  schemas:
    HealthStatus:
      type: object
      required: [status, uptime_seconds]
      properties:
        status:
          type: string
          enum: [ok, degraded, down]
        uptime_seconds:
          type: integer
```

Rules:
- OpenAPI spec is the source of truth for REST
- CI validates spec (dedicated validation job)
- Every response has a schema
- Every endpoint has error responses (4xx, 5xx)
- Server routes mirror spec paths exactly

### Phase 3: Backward Compatibility Check

| Change | WS Protocol | REST API | Trait/Interface |
|--------|:-----------:|:--------:|:----------:|
| Add new enum variant | ✅ Safe | — | — |
| Add new field (optional) | ✅ Safe | ✅ Safe | — |
| Add new trait method (default impl) | — | — | ✅ Safe |
| Rename existing field | ❌ BREAKING | ❌ BREAKING | — |
| Remove enum variant | ❌ BREAKING | — | — |
| Change field type | ❌ BREAKING | ❌ BREAKING | ❌ BREAKING |
| Remove trait method | — | — | ❌ BREAKING |

## Cross-Boundary Rules

### Internal Errors → WS Protocol
- Internal errors must not leak into WS messages
- Map internal errors to `Error { code, message }` variants
- Reserve disjoint error-code ranges per error source (e.g. 1xxx = client, 2xxx = server, 3xxx = backend service)

### WS Protocol → REST
- WS message types and REST response schemas are distinct
- Don't reuse WS enum variants as REST response types
- REST uses snake_case JSON keys by default (like WS)

### Module Dependency Direction
```
protocol/contract types  ← leaf dependency (zero internal deps)
        ↑
core traits/interfaces   ← depends only on protocol types
        ↑
services / adapters      ← implement traits, speak WS/REST
```

No circular dependencies. The protocol module is the leaf every other module may depend on.

## Verification Gates

### Per Boundary

```
[ ] Trait change:    cargo doc --no-deps -p <crate>   (docs compile, signatures reviewed)
[ ] WS protocol:     cargo test -p <protocol-crate>   (serde roundtrip tests)
[ ] REST API:        python3 -c "import yaml; yaml.safe_load(open('docs/openapi.yaml'))"
[ ] E2E:             client → WS → server → WS → client round-trip scripts pass
```

### Additive Change Checklist

```
[ ] No existing fields renamed or removed
[ ] New fields use Option<T> when adding to existing variants
[ ] New variants appended to end of enum
[ ] serde tag/rename_all unchanged
[ ] E2E test scripts updated for new messages
[ ] Backward compat: old client ignores unknown variants (serde default)
[ ] Error codes assigned from correct range
```

### Breaking Change Protocol (IF UNAVOIDABLE)

1. Draft the breaking change in the protocol docs directory
2. Version the WS endpoint: `/ws/v2` alongside `/ws`
3. Deprecation window: 1 release cycle
4. Migration guide alongside the spec

## Common Pitfalls

| Pitfall | Symptom | Fix |
|---------|---------|-----|
| camelCase WS tags | Server silently ignores the message | snake_case only; wire tags are a cross-language contract |
| Missing routing ID in messages | Response cannot be routed to the right peer/session | Every message that targets a resource must carry its ID |
| Internal error in WS message | Crash, not Error response | Always map to `Error { code, message }` |
| Renaming protocol field | Old clients break silently | Never rename; add new field instead |
| Trait method without default | All implementors break | Add default impl when possible |

## Related Skills

| Skill | Relationship |
|-------|-------------|
| `context-engineering` | Which module gets which protocol change |
| `test-harness` | Generate serde roundtrip + E2E test skeletons |
| `security-hardening` | Scan for hardcoded ports/URLs in new endpoints |
| `think-before-act` | Contract review BEFORE implementation |

---

## Hyrum's Law（海勒姆定律）

> With a sufficient number of users of an API, it does not matter what you promise in the contract: all observable behaviors of your system will be depended on by somebody.
>
> — Hyrum Wright, Google

**每一个可观察行为都会成为事实契约。**

| 边界 | 可观察行为（隐性契约） | 防护 |
|------|----------------------|------|
| WS 协议 | 消息顺序、字段出现时机、连接重试间隔 | E2E 脚本固化行为 → 改了就跑不过 |
| Trait/接口 | 方法调用顺序、线程安全标记、错误类型 | `#[non_exhaustive]` + 默认实现 |
| REST API | 响应时间（~P50）、JSON key 排序、空数组 vs null | OpenAPI spec 锁定 contract |
| 构建 features | 哪些模块依赖哪些 feature 组合 | CI matrix 测试所有组合 |

**原则**: API 改了用户就会断 — 无论你是否认为那是"内部细节"。做加法，不做减法。

---

## One-Version Rule（唯一版本规则）

> In a package workspace, every dependency should resolve to exactly one semver-incompatible version. Never allow diamond dependencies.

### 工作区约束

规则（适用于任何多包 workspace，如 Cargo workspace）：

| 规则 | 原因 |
|------|------|
| 所有 workspace 成员使用同一个 `[workspace.dependencies]` 版本 | 避免同一库的两个大版本同时链接 |
| 禁止成员 `[dependencies]` 中写具体版本号 | 版本号只在 workspace root 定义 |
| 新增外部依赖前检查重复依赖树 | 确认不引入 duplicate 依赖 |
| 第三方依赖版本升级必须全 workspace 一致 | 升级运行时库就全 workspace 一起升 |

**检查命令**（Rust）: `cargo tree -d --workspace` — 空输出 = 通过。

---

## Branded Type IDs（品牌化类型 ID）

> Use wrapper types instead of raw `String`/`u64` for domain identifiers to prevent accidental misuse at the type level.

### 模式

```rust
// ✅ CORRECT: branded types — no accidental RoomId → PeerId swap
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransportId(String);

fn connect(room: RoomId, peer: PeerId, transport: TransportId) { /* ... */ }
// RoomId, PeerId, TransportId 不能互换 — 编译器阻止误用
```

```rust
// ❌ WRONG: all String, easy to swap arguments
fn connect(room_id: String, peer_id: String, transport_id: String) { /* ... */ }
```

### 收益

| 无品牌化 | 品牌化 |
|----------|--------|
| `fn route(room_id: String, peer_id: String)` — 传反静默出错 | `fn route(room_id: RoomId, peer_id: PeerId)` — 编译器拒绝 |
| `HashMap<String, Conn>` — key 模糊 | `HashMap<TransportId, Conn>` — key 明确 |
| 序列化时 ID 语义丢失 | Display/Serialize 保留类型语义 |

### 应用约定

项目定义自己的 ID 集合时，把品牌化类型集中放在协议/契约 leaf 模块里，跨模块传递的 ID 一律品牌化。

> **ponytail**: 仅对跨模块传递的 ID 做品牌化。内部一次性局部变量不需要。
