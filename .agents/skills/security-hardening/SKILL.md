---
name: security-hardening
description: "Generic security audit and hardening: OWASP Top 10 checks, hardcoded secrets/ports/URLs scan (merged review-hardcode), PSK/JWT auth flow review, WebSocket security, secrets management, media transport security. Use before release, after auth changes, or when rotating pre-shared keys. Also accessible via /review-hardcode."
---

# security-hardening — 安全加固

> OWASP Top 10 + PSK/JWT auth + WebSocket security + secrets management.
> 每一条规则都有检查命令。每个检查都必须通过。
> `<src>` = 项目源码目录，按实际布局替换（crates/、src/、app/ …）。

## 触发条件

- 发布前（release candidate）
- Auth 模块修改后
- 新增 WebSocket endpoint
- 新增 PSK key / JWT secret
- 用户说 "security review" / "安全审计"
- 新增 FFI 边界（C/ObjC bridge）

## Mode A: 审计模式 (audit)

完整安全审计，手动触发。运行所有 6 个 Phase，生成审计报告。

## Mode B: Guard-while-building (PreToolUse)

> 代码变更时自动触发，阻断不安全提交。轻量级，目标 <5s。
> 参考: JoeyPatricio/security-hardening-skill — PreToolUse guard pattern

### B1: 依赖漏洞快速扫描

```bash
# 仅检查新出现的漏洞（非完整审计）。Rust 为例，各栈用等价工具。
cargo audit --quiet --deny unsound 2>&1 | head -20
```

| 检查项 | 命令 | 通过标准 |
|--------|------|---------|
| 已知漏洞 | `cargo audit --quiet` | 0 漏洞告警 |
| 拒绝 unsound | `cargo audit --deny unsound` | exit 0 |

### B2: unsafe 块扫描

```bash
# 每个 unsafe 块必须有 // SAFETY: 注释（其他栈：cgo / FFI / #pragma 同理）
UNSAFE_NO_COMMENT=$(grep -rn 'unsafe' <src>/ --include='*.rs' | grep -v '// SAFETY:' || true)
if [ -n "$UNSAFE_NO_COMMENT" ]; then
  echo "ERROR: unsafe block without SAFETY comment:"
  echo "$UNSAFE_NO_COMMENT"
  exit 1
fi
```

### B3: gitleaks 密钥检测

```bash
# 安装 (one-time)
# brew install gitleaks  # macOS
# or: go install github.com/gitleaks/gitleaks/v8@latest

# PreToolUse 检查：扫描暂存区变更
gitleaks detect --source . --no-git --redact --verbose 2>&1 | head -30
```

### B4: 硬编码快速扫描 (lightweight)

```bash
# 仅扫描变更文件（非全仓库）
git diff --cached --name-only -- '*.rs' | xargs -I{} grep -n 'token.*=\|password.*=\|api[_-]key' {} 2>/dev/null || true
```

### Guard 执行顺序

```text
PreToolUse (on edit 源文件/清单文件):
  1. unsafe 块注释检查  (<0.5s)
  2. 硬编码快速扫描     (<1s)
  3. gitleaks 检测     (<2s)
  4. 依赖漏洞扫描       (<3s)

任一失败 → 阻断操作
```

### gitleaks 配置 (.gitleaks.toml)

```toml
# .gitleaks.toml — project-level config
title = "Project gitleaks config"

[extend]
useDefault = true

[allowlist]
description = "Known false positives"
paths = [
  ".env.example",               # placeholder values
]

[[rules]]
id = "custom-psk-pattern"
description = "Pre-shared key literals"
regex = '''(?i)(psk|pre_shared_key)\s*=\s*["'][A-Za-z0-9+/]{32,}["']'''
```

### gitleaks pre-commit hook (.git/hooks/pre-commit)

> 与项目现有 fmt/clippy 质量门禁合并到同一 hook：

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit — 质量门禁 + gitleaks 密钥检测
set -euo pipefail

STAGED_RS=$(git diff --cached --name-only --diff-filter=ACM -- '*.rs' || true)
STAGED_TOML=$(git diff --cached --name-only --diff-filter=ACM -- '*.toml' || true)

# ---- gitleaks 密钥检测 (全仓库，仅当源/清单文件变更时) ----
if [ -n "$STAGED_RS" ] || [ -n "$STAGED_TOML" ]; then
  if command -v gitleaks &>/dev/null; then
    echo "→ gitleaks: scanning staged changes..."
    { gitleaks detect --source . --no-git --redact --verbose 2>&1; } || {
      echo ""
      echo "ERROR: gitleaks detected secrets in staged files."
      echo "  Review findings above. False positive? Add to .gitleaks.toml allowlist."
      exit 1
    }
  else
    echo "⚠ gitleaks not installed. Install: brew install gitleaks"
  fi
fi

# ---- 质量门禁 (仅 .rs 变更时，按栈替换) ----
if [ -n "$STAGED_RS" ]; then
  echo "→ cargo fmt: checking..."
  cargo fmt --check
  echo "→ cargo clippy: checking..."
  cargo clippy -- -D warnings
fi

echo "✅ pre-commit checks passed"
```

### gitleaks 安装

| 平台 | 命令 |
|------|------|
| macOS | `brew install gitleaks` |
| Linux | `go install github.com/gitleaks/gitleaks/v8@latest` 或下载 [release binary](https://github.com/gitleaks/gitleaks/releases) |
| Docker | `docker run -v $(pwd):/path zricethezav/gitleaks detect --source /path` |

## Phase 1: 密钥扫描

> 核心教训：全局配置/源码中的硬编码 API Key 存在泄露风险。
> 一律使用环境变量插值（如 `"apiKey": "{env:NEW_API_KEY}"`）或密钥管理器。

```bash
# 手动 grep 检查
grep -rn 'api[_-]key\|api[_-]secret\|token.*=\|password.*="' <src>/ --include='*.rs' | grep -v '//.*TODO' | grep -v 'env::var'
grep -rn 'sk-\|pk-\|AKID\|SecretId' <src>/ --include='*.rs'
grep -rn 'apiKey\|api_key\|API_KEY' .opencode/ --include='*.json' --include='*.jsonc'
```

| 检查项 | 命令 | 通过标准 |
|--------|------|---------|
| 无硬编码密钥 | 上述 grep + gitleaks | 0 CRITICAL |
| env var 插值 | `grep -rn '{env:' <配置目录>` | 所有 key 引用使用插值 |
| .gitignore 覆盖 | `grep '\.env' .gitignore` | 包含 .env, .env.local |
| 示例文件占位 | `grep 'EXAMPLE_KEY\|your-key-here' .env.example` | 无真实密钥 |

### 硬编码值严重性（原 review-hardcode）

本技能吸收了 `review-hardcode` 的完整扫描能力。`/review-hardcode` 命令指向此处
（快速扫描 = 本节 Phase 1，完整审计 = 全部 6 Phase）。

| 模式 | 严重性 | 说明 |
|------|--------|------|
| `token="..."` / `password="..."` / `secret="..."` / `api_key="..."` | 🔴 CRITICAL | 硬编码密钥/令牌 |
| 数字端口字面量（如 `:<端口>` 出现在源码常量/URL 中） | 🟠 HIGH | 端口应配置化，生产端口不应硬编码 |
| `localhost:PORT` / `127.0.0.1:PORT` | 🟠 HIGH | 地址+端口应配置化 |
| `http://IP` 硬编码 IP URL | 🟡 MEDIUM | 应使用配置或 DNS |

### 排除规则

扫描自动排除: `target/`, `node_modules/`, `.git/`, 包管理器缓存目录。
标记 `TODO:` 的硬编码值（允许临时存在）应手动审核后决定是否忽略。

## Phase 2: Auth 流审计

### PSK（预共享密钥）认证

```
// 通用 PSK 流程:
// Client ──[PSK in handshake/header]──> Server ──[validate]──> Session token

// 检查点:
// 1. PSK 是否通过环境变量/密钥管理器注入？(不是 config file 明文)
// 2. PSK 是否 ≥32 字节？
// 3. Server 是否限速 PSK 验证？(防暴力破解)
// 4. Session token 是否有过期时间？
// 5. PSK 错误响应是否模糊？(不泄露"用户存在"或"密钥接近")
```

```bash
grep -rn 'pre_shared_key\|psk' <src>/ --include='*.rs'
grep -rn 'env::var.*PSK\|env::var.*SECRET' <src>/ --include='*.rs'
grep -rn 'session.*ttl\|token.*expir\|jwt.*exp' <src>/ --include='*.rs'
grep -rn 'rate.limit\|429\|too.many' <src>/ --include='*.rs'
```

### JWT 认证

```bash
# 1. JWT secret 是否 ≥256 bit 且来自环境变量？
grep -rn 'jwt.*secret\|JWT_SECRET' <src>/ --include='*.rs' | grep -v env::var

# 2. 确认算法是 HS256/RS256 等显式算法 (拒绝 none)
grep -rn 'Algorithm\|alg.*HS\|alg.*RS' <src>/ --include='*.rs'

# 3. 确认有 exp 声明
grep -rn 'exp\|expir' <src>/ --include='*.rs'

# 4. 确认有 refresh token 轮换
grep -rn 'refresh_token\|refresh' <src>/ --include='*.rs'
```

## Phase 3: WebSocket 安全

```bash
# 1. 消息大小限制 (防 OOM)
grep -rn 'max.*message\|max.*frame\|message.*size' <src>/ --include='*.rs'

# 2. 连接速率限制
grep -rn 'connection.*limit\|max_connections\|concurrent' <src>/ --include='*.rs'

# 3. Origin 验证 (防 CSWSH)
grep -rn 'origin\|allowed_origin\|verify_origin' <src>/ --include='*.rs'

# 4. TLS (生产环境)
grep -rn 'wss://\|tls\|ssl_config' <src>/ --include='*.rs'
```

### 消息注入防护

```rust
// 所有 WS 消息反序列化必须使用严格模式
// 禁止: 未验证额外字段的宽松反序列化
// 正确: deny_unknown_fields / schema 验证后再消费
```

```bash
grep -rn '#\[serde(deny_unknown_fields)\|unknown_fields\|additional_properties' <src>/ --include='*.rs'
grep -rn 'from_slice\|from_str\|unmarshal\|json.loads' <src>/ --include='*.rs'
```

## Phase 4: 媒体传输安全（如项目涉及）

对实时媒体/端口密集型传输（WebRTC 类系统为典型），方法通用：

```bash
# 1. 媒体端口范围是否受控（非 0-65535 全开）
grep -rn 'min_port\|max_port\|port_range\|rtp' <src>/ --include='*.rs'

# 2. transport 建立是否需要鉴权（token/握手验证）
grep -rn 'transport.*auth\|token\|dtls\|ice' <src>/ --include='*.rs'

# 3. 会话/房间创建是否需要权限
grep -rn 'create.*session\|create.*room' <src>/ --include='*.rs'

# 4. 发布者/订阅者权限隔离（读他人流需要授权？）
grep -rn 'peer_id\|publisher\|subscriber\|producer\|consumer' <src>/ --include='*.rs'
```

## Phase 5: 依赖审计

```bash
# 已知漏洞 + 许可证（Rust 为例）
cargo deny check advisories
cargo audit

# 检查 unsafe 使用
grep -rn 'unsafe' <src>/ --include='*.rs' | grep -v '// SAFETY:'
# 规则: 每个 unsafe 块必须有 // SAFETY: 注释说明为何安全
```

## Phase 6: Web UI / Dashboard 安全

```bash
# 1. Dashboard 是否要求认证
grep -rn 'auth\|login\|redirect.*login' <src>/ --include='*.rs'

# 2. 是否有 CSRF 防护
grep -rn 'csrf\|xsrf\|same_site' <src>/ --include='*.rs'

# 3. CORS 是否严格（生产不用 *）
grep -rn 'access-control\|allow_origin\|cors' <src>/ --include='*.rs'

# 4. Content Security Policy
grep -rn 'content-security\|CSP\|frame-ancestors' <src>/ --include='*.rs'
```

## 安全清单 (OWASP Top 10 aligned)

| # | 检查项 | 典型落实 | 命令 | 必须 |
|---|--------|---------|------|:---:|
| A01 | 访问控制失效 | auth 中间件覆盖全部受保护入口 | `grep -rn 'TODO\|FIXME' <src>/` | ✅ |
| A02 | 加密失败 | 密钥强度达标 + 传输走 TLS | `grep -rn 'wss://\|https://' <src>/` (生产检查) | ✅ |
| A03 | 注入 | 消息严格反序列化 | `grep -rn 'deny_unknown' <src>/` | ✅ |
| A04 | 不安全设计 | 速率限制 + 超时 | `grep -rn 'rate.limit\|timeout' <src>/` | ✅ |
| A05 | 安全配置错误 | 生产禁用 debug 模式 | `grep -rn 'debug_assert\|cfg(debug)' <src>/` | ✅ |
| A06 | 脆弱组件 | 依赖审计通过 | `cargo audit` / 栈等价 | ✅ |
| A07 | 认证失败 | PSK/JWT 流程完备 | Phase 2 全部检查 | ✅ |
| A08 | 软件数据完整性 | 序列化格式校验 + 签名来源 | 按所用格式专项验证 | ✅ |
| A09 | 日志监控失败 | 安全事件审计日志 | `grep -rn 'audit\|security.log' <src>/` | ⚠️ |
| A10 | SSRF | 服务端 HTTP 拉取走白名单 | `grep -rn 'reqwest\|hyper::Client\|fetch(' <src>/` | ✅ |

## 报告格式

```
## 安全审计报告 — [日期]

### Phase 1: 密钥扫描
✅ 扫描通过: 0 CRITICAL, 0 HIGH, 0 MEDIUM

### Phase 2: Auth 流
✅ PSK 来自环境变量
✅ PSK 长度: 64 字节
⚠️ Session TTL: 24h (建议 2h)
❌ 无速率限制 (P0)

### Phase 3: WebSocket
✅ 消息大小限制: 1MB
❌ 无 Origin 验证 (可被 CSWSH 攻击)
⚠️ TLS 仅部分环境启用

### Phase 4: 媒体传输
✅ 端口范围受控
✅ 会话创建需 auth
⚠️ 发布者无带宽限制

### Phase 5: 依赖
✅ cargo-audit: 0 vulnerabilities
✅ cargo-deny: 0 unlicensed

### 总结
CRITICAL: 0 | HIGH: 1 | MEDIUM: 2
修复建议: [按严重性排序]
```

## 禁止

- 硬编码密钥：决不允许——密钥只走环境变量/密钥管理器，泄露的立即轮换
- 静默吞错误：auth 失败必须日志但不泄露密钥
- 忽略依赖审计警告：任何漏洞告警必须有决策记录
- 生产环境使用 debug 配置：debug_assert! 在 release 中不执行
- HTTP 明文 WS：生产必须 wss:// 或受控内网
