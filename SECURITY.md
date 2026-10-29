# Security policy

## Reporting a vulnerability

Do **not** open a public issue for a suspected vulnerability. Report it privately
to the maintainers through the repository owner contact on the hosting platform
(Gitee `chengxuewen/VisiaEngine`), including:

- the affected surface: Rust crate, the C ABI (`visiaengine-ffi`), the wasm
  package, or an environment/script layer;
- a minimal reproduction; and
- the impact you believe it has.

You will get an acknowledgement, then a fix or a written decision on scope.

## Scope and expectations

- The engine parses **untrusted input** through the IO crates (glTF, tiles,
  GeoJSON, font bytes) and exposes a C ABI; `unsafe` blocks and the FFI boundary
  are the highest-value targets. Every `unsafe` block is expected to carry a
  `// SAFETY:` rationale (see `CONSTRAINTS.md`).
- Dependency advisories are enforced in CI: `pixi run audit` runs cargo-deny with
  the live RUSTSEC advisory database. A new advisory fails the gate until it is
  either fixed or explicitly accepted with a reason.
- Secrets must never enter the tree; host/agent configuration layers are
  gitignored and generated (`.codebuddy/`, `.trae/`, `.mcp.json`).
