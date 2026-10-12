# Scene-graph lineage — OSG / OpenSG / Coin / VSG / osgEarth

**Snapshot 2026-09-03, measured**: OpenSceneGraph 3.6k★, **last-commit = 2022-12 (the maintenance-collapse specimen)**; coin3d/coin 376★ (active yesterday — the Inventor lineage keeps going via SoScenegraph/TEAM3D, *license once dual GPL/commercial, recheck*); vsg-dev/VulkanSceneGraph 1.8k★ (the official VSG org, author Robert Osfield the same person = OSG's spiritual successor); osgEarth (repo mid-migration, stars unmeasured — activity *UNCERTAIN*); OpenSG has moved to GitLab (the GitHub mirror is dead).

## Profile
- **OSG**: the de-facto dictionary of C++ scene graph + rendering abstraction (Visitor/LOD/PagedLOD/ osgDB plugins/ osgText). **Lesson**: a single core maintainer -> ten years active -> dead stop for three.
- **VSG**: the OSG author's Vulkan-native rebirth — thin abstraction (vk object wrappers) + parallel record + separation of scene graph and rendering.
- **osgEarth**: a geo engine on top of OSG (terrain tiles/imagery projection/elevation) — the C++ precedent closest to Visia's goal.
- **Coin**: the open-source clone of Open Inventor (the ancestor of node caching / lazy-evaluation state machines).

## For VisiaEngine
- **Adopt (high density)**: ① **the PagedLOD/ osgDB paging model = the design dictionary of streaming spatial-data loading** (LOD levels + external file references + frame-budgeted loading) — Rust has no equivalent mature piece, reading OSG source is the shortcut; ② the Visitor pattern's scene-graph evaluation of "traverse + collect render commands" (the visia-render command-generation layer); ③ VSG's thin-over-API validates "keep the rendering abstraction thin, don't over-trait-ify"; ④ osgEarth's terrain scheduling / projection integration = the second geo-engine requirements checklist (control against the cesium file).
- **Avoid**: ① the API debt of the C++ reflection/ intrusive_ptr era (ambiguous copy semantics, a bolted-on threading model) — a native Rust redesign pays the debt back; ② osgDB's format runaway (300+ drivers with no test matrix) -> Visia's IO layer builds the feature-gate and compatibility test matrix from day one; ③ **the maintenance model**: even domain-star projects die suddenly -> keep the critical parse/scheduling paths internalizable via vendoring (small-crate strategy, consistent with `evidence/2026-09-03-wgpu-direct` R4).
- **Current-state ruling**: design reference >> dependency candidate (triple mismatch: C++/LGPL/threading model).

## Sources
img.shields.io per repo (2026-09-03); openscenegraph.info/vsgsamples.org public.
