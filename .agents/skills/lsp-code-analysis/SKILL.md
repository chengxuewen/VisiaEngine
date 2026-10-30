---
name: lsp-code-analysis
description: "Semantic code analysis for pre-change impact work in the 10-crate workspace: find trait implementors before editing visiaengine-core, call hierarchies before touching the C ABI (49 entries), and cross-crate reference maps. Falls back to ripgrep + gate-trace when the host has no LSP. Use before refactors, for 'what breaks if I change X' questions, and for C ABI surface reviews."
---

# LSP Code Analysis -- pre-change impact work (thin wrapper)

This is a THIN wrapper: the host's LSP tool does the work; this file says when it
is worth invoking, and what to do when the host lacks one.

## When to invoke (this repo's three high-value queries)

1. **Trait implementors** before editing anything in `visiaengine-core` /
   `visiaengine-render`: find implementations across the 10-crate workspace --
   the dependency direction (core never depends on render) makes the blast radius
   non-obvious from the file tree.
2. **Call hierarchy** before changing the C ABI surface (49 entry points in
   `crates/visiaengine-ffi`): every caller, including the wasm mirror.
3. **Reference maps** before renaming crate-internal items shared through the
   style layer (`crates/visiaengine-geo/src/style.rs`).

## Host availability and degradation

- CodeBuddy: the `lsp` tool (definitions / references / implementations / call
  hierarchy / document symbols).
- Other hosts (trae / opencode): LSP presence varies. If unavailable, degrade to
  ripgrep with symbol-shaped patterns AND state the limitation -- ripgrep hits
  comments and same-name items, so treat its result as a superset to be pruned.
- The safety net is unchanged either way: gate-trace (spec<->code bidirectional)
  and the 17-gate chain catch what the pre-change analysis misses.

## Boundary

- Complements `rust-engineering` (that skill owns change-then-verify; this one
  owns assess-before-change).
- Do not use for docs/book work -- text search is the right tool there.
