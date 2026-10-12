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

## Hyrum's Law

> With a sufficient number of users of an API, it does not matter what you promise in the contract: all observable behaviors of your system will be depended on by somebody.
>
> — Hyrum Wright, Google

**Every observable behavior becomes a de-facto contract.**

| Boundary | Observable behavior (implicit contract) | Defense |
|------|----------------------|------|
| WS protocol | message ordering, when fields appear, connection-retry interval | E2E scripts freeze the behavior → change it and they fail |
| Trait/interface | method call order, thread-safety markers, error types | `#[non_exhaustive]` + default implementations |
| REST API | response time (~P50), JSON key ordering, empty array vs null | OpenAPI spec locks the contract |
| build features | which modules depend on which feature combinations | CI matrix tests all combinations |

**Principle**: change the API and users break — whether or not you call it an "internal detail". Make additions, not subtractions.

---

## One-Version Rule

> In a package workspace, every dependency should resolve to exactly one semver-incompatible version. Never allow diamond dependencies.

### Workspace constraints

Rules (apply to any multi-package workspace, e.g. a Cargo workspace):

| Rule | Reason |
|------|------|
| all workspace members use one shared `[workspace.dependencies]` version | prevents two majors of the same library being linked together |
| members' `[dependencies]` must not pin concrete version numbers | versions are defined only at the workspace root |
| check the duplicate-dependency tree before adding an external dep | confirm no duplicate dependency is introduced |
| third-party dependency upgrades must be workspace-wide | if a runtime library is upgraded, the whole workspace upgrades together |

**Check command** (Rust): `cargo tree -d --workspace` — empty output = pass.

---

## Branded Type IDs

> Use wrapper types instead of raw `String`/`u64` for domain identifiers to prevent accidental misuse at the type level.

### Pattern

```rust
// ✅ CORRECT: branded types — no accidental RoomId → PeerId swap
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransportId(String);

fn connect(room: RoomId, peer: PeerId, transport: TransportId) { /* ... */ }
// RoomId, PeerId, TransportId cannot be swapped — the compiler blocks misuse
```

```rust
// ❌ WRONG: all String, easy to swap arguments
fn connect(room_id: String, peer_id: String, transport_id: String) { /* ... */ }
```

### Benefits

| No branding | Branded |
|----------|--------|
| `fn route(room_id: String, peer_id: String)` — reversed arguments fail silently | `fn route(room_id: RoomId, peer_id: PeerId)` — the compiler rejects it |
| `HashMap<String, Conn>` — ambiguous key | `HashMap<TransportId, Conn>` — explicit key |
| ID semantics lost at serialization | Display/Serialize retain the type semantics |

### Application convention

When a project defines its own ID set, keep the branded types centralized in the protocol/contract leaf module; any ID passed across modules must be branded.

> **ponytail**: brand only the IDs that travel across modules. Internal one-shot local variables don't need it.
