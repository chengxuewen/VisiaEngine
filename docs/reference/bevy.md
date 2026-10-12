# Bevy — backend candidate A (**not adopted by the final D4 ruling**, 2026-09-03: option B adopted, wgpu-direct in-house)

**Snapshot 2026-09-03, measured**: 48k★; latest 0.19.1 (2026-08-13); main=0.20.0-dev, MSRV 1.96. MIT OR Apache-2.0.

> Full evidence (item-by-item URLs, migration-guide section counts, issue numbers): [2026-09-03-bevy-embed.md](evidence/2026-09-03-bevy-embed.md)

## Profile
A Rust full-stack ECS game engine (Bevy Funded, cart full-time + volunteers). Rendering is in-house on top of wgpu. **The official README self-reports a breaking cadence of ~3 months; the migration-guide preface self-describes an "experimental phase"; the sitemap's 1374 pages carry no 1.0 timeline.**

## Key measured conclusions (input to the final D4 ruling; every item has a URL, stored in the session memo)
1. **bevy_render is inseparable from App/ECS, and 0.19 went deeper**: #22144 RenderGraph->ECS Schedule (the v0.19.0 source tree no longer has a render_graph/ directory); no functional "device+surface+scene -> one frame" entry.
2. **Host-driven main loop**: the official `externally_driven_headless_renderer` was born 2026-01 and was still getting render-sync panic fixes in 2026-07; **rendering inside a host window (Qt widget / C# container): zero first-party / zero third-party precedent**. godot-bevy (540★) deliberately uses only Bevy ECS and hands rendering to Godot — the strongest side evidence.
3. **Breakage surface**: 0.18->0.19 = 120 sections; ecosystem plugins hard-blocked in lockstep (SpawnForge: five crates in one PR).
4. **Capability positive list**: the `Projection` enum switches projections at runtime (projection_zoom example), custom render-phase hooks, 65,536 instances auto-batched into 1 draw call, the 0.19 theme "Render Bigger Scenes Faster".
5. **Size**: no controlled native comparison data (no cargo on this machine, marked as a must-test spike); the wasm sample jlg.io self-reports 13MB/3.4MB gzip.
6. **wasm**: #4078 threads still open; 0.20 is mid WGSL->WESL (the shader interface changes again).

## For VisiaEngine
- **If chosen**: the trade is ~one release cycle of rendering-API surgery + the isolation discipline of "bevy types fully hidden behind a stable C ABI" + forgoing the plugin-upgrade dividend.
- **Adopt regardless of the choice**: the extract/prepare/queue three-phase dataflow (main world -> render world copy) is architecturally valuable for dynamic data sources (twin IoT); the auto instance batching design; the feature sets (2d_api/3d_api excluding the render backend = the posture of "custom renderer" as a first-class citizen).
- **Avoid**: Reflection being non-prunable (#20337) is a headwind for ≤10MB; the rendering-architecture docs chapter is "hidden" (unfinished) — read documentation instability as a signal.

## Sources
all URLs inline in bevy.md: github releases API / bevy.org migration guides and book / docs.rs/bevy_render/0.19.1 / project-forge #8887 / jlg.io PR #550.
