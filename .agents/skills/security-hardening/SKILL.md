---
name: security-hardening
description: "Generic security audit and hardening: OWASP Top 10 checks, hardcoded secrets/ports/URLs scan (merged review-hardcode), PSK/JWT auth flow review, WebSocket security, secrets management, media transport security. Use before release, after auth changes, or when rotating pre-shared keys. Also accessible via /review-hardcode."
---

# security-hardening — Security Hardening

> OWASP Top 10 + PSK/JWT auth + WebSocket security + secrets management.
> Every rule carries a check command. Every check must pass.
> `<src>` = the project's source directory; substitute per actual layout (crates/, src/, app/ …).

## Trigger Conditions

- Before release (release candidate)
- After changes to the Auth module
- New WebSocket endpoint
- New PSK key / JWT secret
- The user says "security review" / "安全审计"
- New FFI boundary (C/ObjC bridge)

## Mode A: audit mode

Full security audit, manually triggered. Runs all 6 Phases and produces an audit report.

## Mode B: Guard-while-building (PreToolUse)

> Auto-triggers on code changes, blocking unsafe commits. Lightweight, target <5s.
> Reference: JoeyPatricio/security-hardening-skill — PreToolUse guard pattern

### B1: dependency-vulnerability quick scan

```bash
# only check newly-appearing vulnerabilities (not a full audit). Rust as example; use each stack's equivalent tool.
cargo audit --quiet --deny unsound 2>&1 | head -20
```

| Check | Command | Pass criteria |
|--------|------|---------|
| known vulnerabilities | `cargo audit --quiet` | 0 vulnerability alerts |
| deny unsound | `cargo audit --deny unsound` | exit 0 |

### B2: unsafe block scan

```bash
# every unsafe block must carry a // SAFETY: comment (other stacks: cgo / FFI / #pragma likewise)
UNSAFE_NO_COMMENT=$(grep -rn 'unsafe' <src>/ --include='*.rs' | grep -v '// SAFETY:' || true)
if [ -n "$UNSAFE_NO_COMMENT" ]; then
  echo "ERROR: unsafe block without SAFETY comment:"
  echo "$UNSAFE_NO_COMMENT"
  exit 1
fi
```

### B3: gitleaks secret detection

```bash
# install (one-time)
# brew install gitleaks  # macOS
# or: go install github.com/gitleaks/gitleaks/v8@latest

# PreToolUse check: scan staged changes
gitleaks detect --source . --no-git --redact --verbose 2>&1 | head -30
```

### B4: hardcode quick scan (lightweight)

```bash
# scan only changed files (not the whole repo)
git diff --cached --name-only -- '*.rs' | xargs -I{} grep -n 'token.*=\|password.*=\|api[_-]key' {} 2>/dev/null || true
```

### Guard execution order

```text
PreToolUse (on edit of source/manifest files):
  1. unsafe-block comment check  (<0.5s)
  2. hardcode quick scan         (<1s)
  3. gitleaks detect             (<2s)
  4. dependency-vulnerability scan (<3s)

any failure → block the operation
```

### gitleaks config (.gitleaks.toml)

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

> merge with the project's existing fmt/clippy quality gates into one hook:

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit — quality gates + gitleaks secret detection
set -euo pipefail

STAGED_RS=$(git diff --cached --name-only --diff-filter=ACM -- '*.rs' || true)
STAGED_TOML=$(git diff --cached --name-only --diff-filter=ACM -- '*.toml' || true)

# ---- gitleaks secret detection (whole repo, only when source/manifest files change) ----
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

# ---- quality gates (only on .rs changes; substitute per stack) ----
if [ -n "$STAGED_RS" ]; then
  echo "→ cargo fmt: checking..."
  cargo fmt --check
  echo "→ cargo clippy: checking..."
  cargo clippy -- -D warnings
fi

echo "✅ pre-commit checks passed"
```

### gitleaks install

| Platform | Command |
|------|------|
| macOS | `brew install gitleaks` |
| Linux | `go install github.com/gitleaks/gitleaks/v8@latest` or download a [release binary](https://github.com/gitleaks/gitleaks/releases) |
| Docker | `docker run -v $(pwd):/path zricethezav/gitleaks detect --source /path` |

## Phase 1: Secret scanning

> Core lesson: hardcoded API keys in global config/source carry leak risk.
> Always use environment-variable interpolation (e.g. `"apiKey": "{env:NEW_API_KEY}"`) or a secrets manager.

```bash
# manual grep checks
grep -rn 'api[_-]key\|api[_-]secret\|token.*=\|password.*="' <src>/ --include='*.rs' | grep -v '//.*TODO' | grep -v 'env::var'
grep -rn 'sk-\|pk-\|AKID\|SecretId' <src>/ --include='*.rs'
grep -rn 'apiKey\|api_key\|API_KEY' .opencode/ --include='*.json' --include='*.jsonc'
```

| Check | Command | Pass criteria |
|--------|------|---------|
| no hardcoded secrets | the greps above + gitleaks | 0 CRITICAL |
| env-var interpolation | `grep -rn '{env:' <config dir>` | every key reference uses interpolation |
| .gitignore coverage | `grep '\.env' .gitignore` | includes .env, .env.local |
| example-file placeholders | `grep 'EXAMPLE_KEY\|your-key-here' .env.example` | no real secrets |

### Hardcoded-value severity (formerly review-hardcode)

This skill absorbed `review-hardcode`'s full scanning capability. The `/review-hardcode` command points here
(quick scan = this section's Phase 1; full audit = all 6 Phases).

| Pattern | Severity | Note |
|------|--------|------|
| `token="..."` / `password="..."` / `secret="..."` / `api_key="..."` | 🔴 CRITICAL | hardcoded key/token |
| numeric port literals (e.g. `:<port>` in source constants/URLs) | 🟠 HIGH | ports belong in config; production ports must not be hardcoded |
| `localhost:PORT` / `127.0.0.1:PORT` | 🟠 HIGH | address+port belong in config |
| `http://IP` hardcoded-IP URLs | 🟡 MEDIUM | use config or DNS |

### Exclusion rules

Scans automatically exclude: `target/`, `node_modules/`, `.git/`, package-manager cache dirs.
Hardcoded values marked `TODO:` (allowed temporarily) need manual review to decide on ignoring.

## Phase 2: Auth-flow audit

### PSK (pre-shared key) authentication

```
// generic PSK flow:
// Client ──[PSK in handshake/header]──> Server ──[validate]──> Session token

// checkpoints:
// 1. is the PSK injected via env var / secrets manager? (not plaintext in a config file)
// 2. is the PSK ≥32 bytes?
// 3. does the Server rate-limit PSK validation? (brute-force defense)
// 4. does the Session token have an expiry?
// 5. are PSK-error responses vague? (never leak "user exists" or "key nearly correct")
```

```bash
grep -rn 'pre_shared_key\|psk' <src>/ --include='*.rs'
grep -rn 'env::var.*PSK\|env::var.*SECRET' <src>/ --include='*.rs'
grep -rn 'session.*ttl\|token.*expir\|jwt.*exp' <src>/ --include='*.rs'
grep -rn 'rate.limit\|429\|too.many' <src>/ --include='*.rs'
```

### JWT authentication

```bash
# 1. is the JWT secret ≥256 bit and sourced from an env var?
grep -rn 'jwt.*secret\|JWT_SECRET' <src>/ --include='*.rs' | grep -v env::var

# 2. confirm the algorithm is explicit (HS256/RS256 etc.) — reject none
grep -rn 'Algorithm\|alg.*HS\|alg.*RS' <src>/ --include='*.rs'

# 3. confirm an exp claim exists
grep -rn 'exp\|expir' <src>/ --include='*.rs'

# 4. confirm refresh-token rotation exists
grep -rn 'refresh_token\|refresh' <src>/ --include='*.rs'
```

## Phase 3: WebSocket security

```bash
# 1. message-size limits (OOM defense)
grep -rn 'max.*message\|max.*frame\|message.*size' <src>/ --include='*.rs'

# 2. connection rate limiting
grep -rn 'connection.*limit\|max_connections\|concurrent' <src>/ --include='*.rs'

# 3. Origin validation (CSWSH defense)
grep -rn 'origin\|allowed_origin\|verify_origin' <src>/ --include='*.rs'

# 4. TLS (production)
grep -rn 'wss://\|tls\|ssl_config' <src>/ --include='*.rs'
```

### Message-injection defense

```rust
// all WS message deserialization must use strict mode
// forbidden: lenient deserialization that never validates extra fields
// correct: deny_unknown_fields / schema validation before consumption
```

```bash
grep -rn '#\[serde(deny_unknown_fields)\|unknown_fields\|additional_properties' <src>/ --include='*.rs'
grep -rn 'from_slice\|from_str\|unmarshal\|json.loads' <src>/ --include='*.rs'
```

## Phase 4: Media transport security (if the project involves it)

For real-time media / port-heavy transports (WebRTC-class systems are typical) — the method is generic:

```bash
# 1. is the media port range controlled (not 0-65535 wide open)
grep -rn 'min_port\|max_port\|port_range\|rtp' <src>/ --include='*.rs'

# 2. does transport establishment require auth (token/handshake validation)
grep -rn 'transport.*auth\|token\|dtls\|ice' <src>/ --include='*.rs'

# 3. does session/room creation require permission
grep -rn 'create.*session\|create.*room' <src>/ --include='*.rs'

# 4. publisher/subscriber permission isolation (is reading another's stream authorized?)
grep -rn 'peer_id\|publisher\|subscriber\|producer\|consumer' <src>/ --include='*.rs'
```

## Phase 5: Dependency audit

```bash
# known vulnerabilities + licenses (Rust as example)
cargo deny check advisories
cargo audit

# check unsafe usage
grep -rn 'unsafe' <src>/ --include='*.rs' | grep -v '// SAFETY:'
# rule: every unsafe block must carry a // SAFETY: comment explaining why it is safe
```

## Phase 6: Web UI / Dashboard security

```bash
# 1. does the Dashboard require auth
grep -rn 'auth\|login\|redirect.*login' <src>/ --include='*.rs'

# 2. is there CSRF protection
grep -rn 'csrf\|xsrf\|same_site' <src>/ --include='*.rs'

# 3. is CORS strict (never * in production)
grep -rn 'access-control\|allow_origin\|cors' <src>/ --include='*.rs'

# 4. Content Security Policy
grep -rn 'content-security\|CSP\|frame-ancestors' <src>/ --include='*.rs'
```

## Security checklist (OWASP Top 10 aligned)

| # | Check | Typical implementation | Command | Mandatory |
|---|--------|---------|------|:---:|
| A01 | broken access control | auth middleware covers every protected entry | `grep -rn 'TODO\|FIXME' <src>/` | ✅ |
| A02 | cryptographic failure | key strength meets bar + transport over TLS | `grep -rn 'wss://\|https://' <src>/` (production check) | ✅ |
| A03 | injection | strict message deserialization | `grep -rn 'deny_unknown' <src>/` | ✅ |
| A04 | insecure design | rate limiting + timeouts | `grep -rn 'rate.limit\|timeout' <src>/` | ✅ |
| A05 | security misconfiguration | debug mode disabled in production | `grep -rn 'debug_assert\|cfg(debug)' <src>/` | ✅ |
| A06 | vulnerable components | dependency audit passes | `cargo audit` / stack equivalent | ✅ |
| A07 | authentication failure | PSK/JWT flows complete | all Phase 2 checks | ✅ |
| A08 | software/data integrity | serialization format validation + signed provenance | dedicated verification per format | ✅ |
| A09 | logging/monitoring failure | security-event audit log | `grep -rn 'audit\|security.log' <src>/` | ⚠️ |
| A10 | SSRF | server-side HTTP fetches go through an allowlist | `grep -rn 'reqwest\|hyper::Client\|fetch(' <src>/` | ✅ |

## Report format

```
## Security audit report — [date]

### Phase 1: secret scanning
✅ scan passed: 0 CRITICAL, 0 HIGH, 0 MEDIUM

### Phase 2: auth flow
✅ PSK from environment variable
✅ PSK length: 64 bytes
⚠️ Session TTL: 24h (suggest 2h)
❌ no rate limiting (P0)

### Phase 3: WebSocket
✅ message size limit: 1MB
❌ no Origin validation (CSWSH-attackable)
⚠️ TLS enabled only in some environments

### Phase 4: media transport
✅ port range controlled
✅ session creation requires auth
⚠️ publishers have no bandwidth limit

### Phase 5: dependencies
✅ cargo-audit: 0 vulnerabilities
✅ cargo-deny: 0 unlicensed

### Summary
CRITICAL: 0 | HIGH: 1 | MEDIUM: 2
Fix recommendations: [sorted by severity]
```

## Forbidden

- hardcoded secrets: never allowed — keys go only through env vars/secrets managers; any leaked key is rotated immediately
- silently swallowed errors: auth failures must be logged but never leak the key
- ignoring dependency-audit warnings: every vulnerability alert needs a recorded decision
- production running debug config: `debug_assert!` doesn't execute in release
- plaintext-HTTP WebSocket: production must use wss:// or a controlled internal network
