---
name: shipping-and-launch
description: "Release and launch checklist for this engine: full ci green, pack-check, version/tag policy, changelog decision, rollback, and docs. Use before a release, a publish band, or any 'ship it' request."
---

# Shipping & Launch (this workspace)

## Overview

- The release gate is `pixi run ci` (16 gates) plus `pixi run pack-check` (`scripts/gate-pack.sh`).
- Version lives in the workspace `Cargo.toml` (currently `0.1.0`); **version files are user-managed** — never bump them without an explicit instruction.
- There is **no `CHANGELOG` yet**; creating one is a decision, not an automatic step.
- Docs to keep in sync: `docs/architecture.md`, `docs/env.md`, `AGENTS.md`, `CONSTRAINTS.md`.

## When to Use

- Before tagging/releasing, or when the user says "ship it / release / publish".
- After a publish-band change (`pack-check`, packaging, workspace layout).
- When deciding whether a change is release-worthy.

## Process (steps + checkpoints + exit)

1. **Full gate**: `pixi run ci` must be green (fmt/lint/check/test/audit + all `gate-*` + `pack-check`).
2. **Pack**: `pixi run pack-check` green (`scripts/gate-pack.sh`); confirm the tarball/assembly is current.
3. **Version**: read `Cargo.toml`; propose the bump (semver) and **ask** — do not edit version files unilaterally.
4. **Changelog**: if none exists, propose creating one (a decision); otherwise update it from the commits since the last tag.
5. **Docs**: update `docs/architecture.md` / `docs/env.md` / `AGENTS.md` / `CONSTRAINTS.md` where the release changed behavior.
6. **Tag**: annotated tag on the release commit; the tag message summarizes the change set.
7. **Rollback plan**: state the previous tag and the revert path before announcing.
8. **Exit**: `ci` + `pack-check` green, version/tag set, docs synced, rollback named — with the evidence quoted.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "CI was green yesterday" | Green is per-commit; run it on the release commit. |
| "bump the version, it's obvious" | Version is user-managed; propose, don't decide. |
| "skip the changelog" | Users need to know what changed; a missing changelog is a debt. |
| "rollback is unlikely" | The plan costs one sentence; not having it costs an outage. |
| "pack-check is packaging detail" | It is in the `ci` chain for a reason — a broken pack ships broken. |

## Red Flags

- Release with a red `ci` or `pack-check`.
- A version bump without the user's instruction.
- A tag on a commit that is not the tested one.
- Docs describing behavior that the release no longer has.
- No rollback tag named.

## Verification (evidence)

- `pixi run ci` tail (16 gates green) and `pixi run pack-check` output.
- The tag object (`git tag -n`) and the commit it points to.
- The docs diff for the release.
- The named previous tag for rollback.

## References

- `pixi.toml` — `ci` depends-on + `pack-check`.
- `docs/env.md` — toolchain and task reference.
- `constraint-driven-development` — why a red gate is not bypassed.
