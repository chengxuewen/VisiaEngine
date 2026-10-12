# Unity — the full-cycle sample of trust collapse and recovery

**Snapshot 2026-09-03** (Wikipedia rev 1371602628): stable **6000.4.0f1 Mainline / 6000.3.x LTS / 6000.0.x LTS** (2026-03); beta 6000.5; proprietary, C++ runtime + C# API; **the runtime fee announced 2023 -> rescinded 2024-09** (CEO change); 19+ platforms incl. WebGL and WebGPU build targets; ~50% mobile-game share.

## Profile
21 years of a "democratization" engine: editor + managed experience as the core, DOTS/ECS (Entities+Burst+JobSystem) as the performance side-track. **Rendering-architecture label: three pipelines (Built-in / URP / HDRP) mutually incompatible** — three render semantics coexisting inside one engine (its own docs' original wording "incompatible with each other", quoted the same day). C# source is reference-only (not modifiable, since 2018).

## Strengths/weaknesses
Strong: install-base dominance on mobile; the Asset Store ecosystem (40M+ download history); non-game penetration (automotive/architecture/US-military simulation). Weak: one pricing event's burned trust took 2+ years plus a CEO swap to barely backfill; three mutually-exclusive pipelines = users forced to pick sides at the "render stack" level; black-box debugging of closed-source dependencies.

## For VisiaEngine
- **Adopt (all three are lessons bought with money)**: ① **a pricing/license sudden-change = survival-level risk** — the Visia whitepaper's "licenses of released versions are irrevocable" promise (written into D3) answers the Unity event head-on; this history is the most persuasive control case for outward communication; ② **the pipeline-fork ban**: the visia-render abstraction permits only "one semantic source + tier parameterization" (figure ⑥ capability matrix), never the "URP or HDRP?" style of user side-picking; ③ DOTS failed to eat the main scene model over ten years (managed/native dual-world friction) -> for "should the core go ECS" (figure ③ decision point) this is heavyweight conservative evidence: **a simple direct data model beats paradigm purity**.
- **Avoid**: the strong editor-runtime coupling (Unity's engine cannot leave the Unity product as a library) — throwing into relief that Visia's "SDK-first" vacancy is exactly the market Unity cannot reach (industrial HMI / embedded twin screens).
- **Current-state ruling**: business-narrative material >> technical reference; the existence of WebGPU build targets (quoted same day) shows "native engine on the Web" is already the industry baseline, Visia's wasm route (figure ⑨) is not a radical choice.

## Sources
en.wikipedia.org/wiki/Unity_(game_engine) (2026-09-03).
