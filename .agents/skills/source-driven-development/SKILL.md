---
name: source-driven-development
description: "Ground every external dependency decision in official docs. Use when adding or upgrading any crate/framework/library, calling unfamiliar external APIs, or about to write framework-specific code from memory. Verifies manifest deps against upstream docs. Triggers: 'check the docs', 'is this API correct', 'what does the spec say', 'verify against upstream', dependency upgrade, new dependency."
---

# Source-Driven Development

## Overview

Every external dependency decision must be backed by official documentation.
Training data goes stale — documentation doesn't lie. Verify every API call,
every feature flag, every build step against upstream docs before writing them.

## When to Use

- Adding or upgrading a dependency (check changelog, migration guide, MSRV/min-version)
- Implementing calls against any non-trivial external API (especially FFI bindings)
- Writing Docker/CI configs (Dockerfile, docker-compose, GitHub Actions)
- Building UI with a framework whose idioms change between versions
- Any time you're about to write framework-specific code from memory

**When NOT to use:**
- Pure language logic (ownership, iterators, error handling) — stdlib patterns don't change
- Renaming variables, fixing typos, moving files
- Tests that exercise project-internal APIs (no external dep involved)

## The Process

```
DETECT ──→ FETCH ──→ IMPLEMENT ──→ CITE
  │          │           │            │
  ▼          ▼           ▼            ▼
 Manifest    Upstream    Follow the   Show your
 version     docs        documented   sources
                         patterns
```

### Step 1: Detect Dependency Versions

From the manifest — `Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, etc.:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", optional = true }
```

State what you found:

```
DEPENDENCIES DETECTED:
- serde 1.x (from <path>/Cargo.toml)
- tokio 1.x, optional (from <path>/Cargo.toml)
→ Fetching upstream docs for the relevant APIs.
```

For Docker/CI: check Dockerfile base image tags, docker-compose service configs,
GitHub Actions `runs-on`. For TS: check `package.json` for framework deps.

### Step 2: Fetch Official Documentation

**Source ladder (in order of authority):**

| Priority | Source | Example |
|----------|--------|---------|
| 1 | Upstream API reference for the pinned version | docs.rs/<crate>/<version>, pkg.go.dev, MDN |
| 2 | Upstream repo README + CHANGELOG | github.com/<org>/<project>/CHANGELOG.md |
| 3 | Official guides / migration docs | project books, version migration guides |
| 4 | Language/framework official docs | rust-lang.org, react.dev, docs.docker.com |
| 5 | Web/protocol standards | w3.org/TR, datatracker.ietf.org |

The ladder is the whole method: **official docs → registry metadata → upstream
source**. Stop at the first rung that answers the question.

### Using Context7 MCP for Documentation

Context7 provides a fast MCP-based query interface to official library
documentation. Use it as a caching layer over rungs 1-4 above.

**Context7 is already configured** in `.opencode/opencode.json` and available via two tools:

**Step 2a: Resolve library name to ID**
```
# Tool: context7_resolve-library-id
# Search for a library; select the best match by snippet count + source reputation + benchmark score.
context7_resolve-library-id(libraryName: "<library>", query: "<specific API question>")
# → Returns libraryId: "/<org>/<project>" (prefer the version-specific ID if docs changed)
```

**Step 2b: Query the library's documentation**
```
# Tool: context7_query-docs
# Pass the exact libraryId from step 2a. Be specific — one concept per query.
context7_query-docs(libraryId: "/<org>/<project>", query: "<specific API question>")
# → Returns code snippets ranked by relevance + benchmark score
```

**When to use Context7 vs. direct docs:**

| Scenario | Use |
|----------|-----|
| Quick API signature lookup | Context7 (faster, code-snippet ranked) |
| Full method docs with examples | Context7 first, fall back to the API reference if incomplete |
| Version-specific migration guides | Direct upstream repo + Context7 |
| First-time library exploration | Context7 (gets you oriented fast) |
| API that changed recently (last 6 months) | Context7 (indexed from live docs, not training data) |

**Workflow:**
```
1. context7_resolve-library-id → get libraryId
2. context7_query-docs(libraryId, "specific API question") → code examples
3. If answer is incomplete, fall back to the API reference / upstream repo (rungs 1-2)
4. Cite: "Source: Context7 /{org}/{project} + docs.rs/{crate}/{version}"
```

**Not authoritative:**
- Stack Overflow, blog posts, tutorials
- AI training data (that's what we're verifying)
- Random GitHub issues without upstream confirmation

### Step 3: Implement Following Documented Patterns

For library APIs, pin the call to the detected version's docs:

```rust
// <crate> <version>: <Type>::<method>() signature
// Source: https://docs.rs/<crate>/<version>/...
let result = obj.method(&arg)?;
```

For C/C++-backed bindings, verify the binding signature against the native API:

```
VERIFICATION: <native lib>.<API surface>
Source: <official native docs URL>
Required fields: <list>
Optional: <list>
→ Confirmed against <binding crate> <version> signatures
```

### Step 4: Cite Your Sources

Every non-trivial external dependency usage gets a citation:

```rust
// Source: https://docs.rs/<crate>/<version>/<path to item>
// Verified against pinned manifest version <x.y>
let handle = Resource::open(&config)?;
```

In conversation:

```
Using <Type>::<method>() from <crate> <version>.
Source: https://docs.rs/<crate>/<version>/...
This replaces the previous assumption that <what memory/training data said>
(verified against the docs above).
```

## Project-Level Verification Recipes

### Rust crate dependencies

Before adding/changing a dep:

```bash
# 1. Check if the crate is already in the workspace
grep -r "<crate_name>" Cargo.toml crates/*/Cargo.toml

# 2. Verify version compatibility
cargo tree -i <crate_name>

# 3. Check MSRV against rust-toolchain.toml / manifest rust-version
cat rust-toolchain.toml 2>/dev/null; grep '^rust-version' Cargo.toml

# 4. Verify license + advisories
cargo deny check   # or cargo audit
```

### Feature flag combinations

Feature flags are interdependent. Each meaningful combination must at least
type-check — don't discover a broken combination in CI:

```bash
cargo check -p <crate> --no-default-features
cargo check -p <crate>                       # defaults
cargo check -p <crate> --features <flag>     # each optional backend/feature
cargo check -p <crate> --all-features        # where combinations are legal
```

### Native binding version constraints

When a crate binds a native library (FFI), the binding pins an upstream range.
Verify binding version ↔ native version compatibility from the binding's
README/CHANGELOG, and record hard constraints explicitly:

```
<binding> <ver> binds <native lib> <major>.<minor>. Constraints:
- platform support (OS/arch)
- build toolchain requirements (meson/cmake/pkg-config equivalents)
- recommended base image for containers
```

### Docker/CI verification

```bash
# Dockerfile base image should match CI runners
grep "FROM" docker/Dockerfile
grep "runs-on" .github/workflows/ci.yml
# Same OS/version family in both, or document why they differ
```

## Common Conflict Shapes

### Conflict: manifest version != what the docs describe

```
CONFLICT: Cargo.toml pins <binding-crate> = "0.13" but the upstream native docs
being read describe APIs introduced in 0.14.x.
→ Check the binding crate's CHANGELOG for which native version 0.13 targets.
→ Fetch docs for the pinned version, not latest.
```

### Conflict: mutually exclusive features

```
CONFLICT: Both <feature-a> and <feature-b> are enabled; each selects a different
backend for the same subsystem.
→ compile_error! is expected. Only one backend per build.
```

### Conflict: platform constraint

```
CONFLICT: This dep only builds on Linux, dev host is macOS.
Options:
A) cargo check only (type-check, no link/build of native parts)
B) Build + run inside the project's Docker service
C) Full build/test on Linux CI
→ Choose based on what you're verifying.
```

## Verification Checklist

- [ ] Dependency version identified from manifest (Cargo.toml / package.json / ...)
- [ ] Official docs fetched for any new/modified external API usage
- [ ] API signatures match the detected version (not training data)
- [ ] Feature flag combinations all `cargo check` clean
- [ ] Platform constraints documented (Linux-only, macOS-only, arch pins)
- [ ] Non-trivial decisions cite upstream docs (+ local decision record if one exists)
- [ ] Deprecated APIs not used (checked migration guides)
- [ ] Conflicts between docs and existing code surfaced
- [ ] Anything unverified is explicitly flagged

## Red Flags

- Writing an external API call from memory without checking the pinned version's docs
- Using a library's APIs across a major-version boundary without reading that version's migration guide
- Adding a manifest dep without checking if it's already in the workspace
- Enabling two mutually exclusive features
- Docker base image family mismatch with CI runners
- Not running the dependency audit/license check after adding a new dependency

## See Also

- `.agents/memorys/pitfalls.md` — dependency/build conflicts accumulate here
- `.agents/memorys/decisions.md` — record version & API decisions with source links
- `.agents/rules/common/docker.md` — container & network constraints
