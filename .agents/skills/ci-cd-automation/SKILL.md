---
name: ci-cd-automation
description: "Generic CI/CD pipeline management: Docker compose services, GitHub Actions gates (fmt/check/lint/test/audit/benchmark), task-runner mirrors, dependency audits, and platform-specific builds. Use for CI troubleshooting, pipeline changes, or pre-merge verification."
---

# ci-cd-automation — CI/CD Pipeline Management

> The pipeline IS the gate. Every check is a contract. Don't merge red.
> `<src>` below = the project's source directories; adapt commands to the chosen stack.

## Pipeline Architecture

Typical gate set on push/PR to main (name jobs after what they verify):

```
GitHub Actions
│
├── fmt           : format check (e.g. `cargo fmt --all --check`, `prettier --check`)
├── check         : compile / type-check on every supported OS
├── lint          : linter with warnings-as-errors (e.g. `cargo clippy -- -D warnings`)
├── test          : unit + integration tests on every supported OS
├── audit         : dependency advisory / license check
├── benchmark     : perf regression job (only if the project has benches)
└── spec-validate : schema validation (OpenAPI, protobuf, JSON Schema, ...)
```

Rule: every CI job must be reproducible locally with the same command.

## Local Development Workflow

Two tiers — document both, use the fast one in the inner loop:

```bash
# Quick tier: type-check + tests without heavy optional features
cargo check --workspace
cargo test --workspace

# Full tier: anything with OS/hardware-bound deps (native libs, GPU, codecs)
# runs inside the compose service container instead of on the host
docker compose up -d <service>
docker compose exec <service> <build/test command>
```

## Docker Compose Workflow

```bash
docker compose up -d                 # build + start
docker compose logs -f <service>     # tail logs
docker compose exec <service> sh     # debug inside
docker compose down                  # stop
```

### Critical Constraints (verify against your project)

| Constraint | Detail |
|-----------|--------|
| Native deps may be platform-bound | If a dep builds only on Linux, keep other hosts check-only + Docker for build/test |
| Bind mounts on macOS are slow | Keep build artifacts (`target/`, `node_modules/`) in named volumes, mount only source |
| First cold Docker build is long | Native toolchains compile from scratch — cache package-manager registries aggressively |
| UDP / wide port ranges | RTC/media/proxy-style services need explicit port-range mappings in compose + firewall |
| TLS stack choice | Prefer minimal native TLS stacks (e.g. rustls) over pulling full OpenSSL into the dep tree |

## Task Runner Reference

Mirror every CI job as a local task (just / make / npm scripts / equivalent):

| Task | Command | When |
|------|---------|------|
| `check` | workspace compile check, platform-bound service excluded | after any code change |
| `build` | workspace build | before running binaries |
| `lint` | linter with `-D warnings` | before commit |
| `test` | workspace tests | before PR |
| `check-<svc>` | containerized check for the platform-bound service | service changes |
| `audit` | dependency advisory + license check | pre-merge |
| `coverage` | coverage report (target ≥80%) | pre-release |
| `format` / `format-fix` | fmt check / fmt apply | pre-commit |

Rules:
- CI job N == local task N (same command, different runner). Divergence = debugging tax.
- Put the fast variants first in docs — that's the loop people live in.

## Dependency Audit

Run before EVERY merge (Rust example: `cargo deny check` / `cargo audit`; every
ecosystem has an equivalent — npm audit, pip-audit, govulncheck):

| Check | Config | Threshold |
|-------|--------|-----------|
| Security advisories | `[advisories]` | deny yanked / vulnerable |
| License compliance | `[licenses]` | allow-list (Apache-2.0, MIT, BSD-*, ISC, Unicode-3.0, Zlib, ...) |
| Duplicate deps | `[bans]` | warn on multiple versions |
| Source registry | `[sources]` | deny unknown registries / git sources |

### Audit Failure Protocol

1. **Security advisory**: find the fixed version, update the manifest; never ignore without a written decision record.
2. **License violation**: check the new dep's license against the allow-list; if absent, discuss with team before merging.
3. **Duplicate dependency**: `cargo tree -d` (or equivalent) to locate it, deduplicate where possible.
4. **Unknown source**: remove the git dependency or allow-list it explicitly with a reason.

## Pre-Merge Verification Sequence

Run in order. Do NOT skip steps.

```bash
<format>      # 1. style gate (fastest, catches most noise)
<lint>        # 2. lints (finds logic errors)
<check>       # 3. compile / type check
<test>        # 4. unit + integration tests (catches regressions)
<audit>       # 5. dependency security + license
<svc-tests>   # 6. containerized / platform-bound tests, if touched
<coverage>    # 7. coverage vs target
```

## CI Troubleshooting

### Fails on one OS, passes on the other

```bash
# Check for OS-specific code paths
grep -rn "cfg.*target_os" <src>/
```

Common causes:
- Missing `#[cfg(target_os = "...")]` guards (Rust) or equivalent conditional compilation
- Platform-specific API usage without a fallback
- Hardcoded platform paths (`/usr/lib`, `/opt/homebrew`, `/etc`)

### Native / FFI build job fails

- Compare required system packages in the CI job vs the Dockerfile (`apt-get install ...`).
- Build-script env vars must be **absolute paths** (relative `meson`/`cmake` binaries break silently in CI).
- `build.rs`-style script changes need a build-cache clear — package-level clean often doesn't reach generated artifacts:
  `rm -rf target/debug/build/<pkg>-*`
- Check the underlying builder's generated flags for duplicated/conflicting arguments (e.g. buildtype set in two places).

### Schema validation job fails

Validate locally with the same parser CI uses, e.g. for OpenAPI:

```bash
python3 -c "
import yaml
spec = yaml.safe_load(open('docs/openapi.yaml'))
assert spec.get('openapi'), 'Not an OpenAPI spec'
for path, methods in spec['paths'].items():
    for method, detail in methods.items():
        assert 'responses' in detail, f'{path} {method}: missing responses'
print('OK')
"
```

## Adding a New CI Job

Template:

```yaml
new-job-name:
  runs-on: ubuntu-latest   # pin an exact runner version when a native dep needs a specific glibc
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable   # or the language setup equivalent
    - uses: Swatinem/rust-cache@v2          # always cache deps
    - run: <command>
```

Rules:
- All jobs use dependency caching (saves minutes per run).
- Platform-pinned jobs: document WHY the pin exists, in a comment.
- No hardcoded secrets in workflow files — use repository secrets.
- New job → update the pipeline diagram in this skill.

## Common Pitfalls (generic)

| Pitfall | Fix |
|---------|-----|
| CI command not reproducible locally | mirror every job as a local task, same command |
| Cold dependency build on every run | add dep-cache volume/action; persist package-manager caches |
| Platform-bound dep breaks one OS in the matrix | split matrix: check-only native, full build in Docker |
| Stale build-script cache hides a fix | remove the package's build dir under `target/`, don't trust package-level clean |
| Secrets committed in workflow env blocks | move to repo secrets, rotate, audit git history |

## Related Skills

| Skill | Relationship |
|-------|-------------|
| `security-hardening` | Run its secrets/hardcode scan BEFORE audit — catch hardcoded secrets in CI config |
| `test-harness` | Generate test skeletons that CI will run |
| `lesson-memory` | CI failures → write to pitfalls.md |
| `think-before-act` | Check CI status BEFORE implementing a fix |
