# Unreal Engine — the mirror lessons of a closed leviathan

**Snapshot 2026-09-03** (Wikipedia rev 1372756723 + shields same day): stable **5.8**; **UE6 announced 2026-05-24**, EA "Late 2027-ish", GA another +12–18 months; source is source-available in a non-public repo (EpicGames/UnrealEngine requires linked login, public stars unobtainable — the form itself is data); license = 5% royalty on revenue >$1M (EGS-exclusive exempt); 28% share of the 2024 game market (Unity 50%), 31% by unit sales = the largest commercial engine.

## Profile
28 years, five generations, the film/game overlord. Architecture labels: C++ + UObject reflection/GC + Blueprint visual scripting + **Verse** (a Haskell-line functional-logic language, Simon Peyton Jones involved, launched 2023) + Nanite (virtualized geometry: arbitrary-precision meshes -> automatic LOD -> GPU-driven cluster streaming) + Lumen (hybrid software/hardware GI) + World Partition (large-map streaming). **The UE6 announcement: Blueprints/Actors will be replaced by Verse + Scene Graph** (early versions coexist).

## Strengths/weaknesses
Strong: the rendering-technology ceiling (Nanite/Lumen have no open-source counterpart); non-game penetration in film/automotive/architecture (StageCraft, Rivian's vehicle HMI, 300+ virtual production stages). Weak: bulk (editor at GB scale — **the opposite extreme of an SDK form**); the license is a lease, not ownership (Epic may change the rules at any time — the 2024 runtime-fee storm -> 2025 rollback, listed beside the HashiCorp incident in the Open Core trust textbook); embedding into third-party applications = mission impossible.

## For VisiaEngine
- **Adopt**: ① **Nanite is the endgame answer for massive BIM/oblique-photography geometry** — no need to copy the implementation (person-year scale), but visia-render's render-command IR must **reserve the extension slot for cluster-based geometry streaming** (meshlet/indirect-draw semantics are available on wgpu, see `_memo-wgpu-direct`); don't let the MVP architecture seal off this road; ② the UE6 "scene-model generational-change cost" warning: **scene-graph semantics are the engine's foundation, get it right once** (the Actor->SceneGraph migration dragged across three generations); ③ World Partition's open-world streaming partitioning design = the reference for twin-city scale.
- **Avoid**: ① depend on nothing: the source-available "readable but not usable" boundary swings entirely with Epic's mood; ② the reflection/GC universe (UObject) tied to the engine's lifecycle — hitting squarely Visia's two creeds of "no GC / embeddable".
- **Current-state ruling**: the commercial control group (the last puzzle piece of the royalty vs subscription vs Open Core three-form comparison); on the technical side read only design docs and GDC talks, no decision dependencies.

## Sources
en.wikipedia.org/wiki/Unreal_Engine (2026-09-03); UE6 detail citations include The Verge/Lex Fridman/State of Unreal 2026 retellings.
