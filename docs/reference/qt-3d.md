# Qt3D / Qt Quick 3D / QRhi — the empirical proof that even the official vendor's domain 3D goes cold

**Snapshot 2026-09-03, measured**: Qt/qt3d 229★; Qt/qtquick3d **74★** (last-commit July). Commercial Qt + GPL/LGPL dual.

## Profile
Two generations of official Qt 3D: Qt3D (ECS-style, the Qt Widgets line, maintenance-ward) -> Quick3D (QML declarative, USD/glTF import, a Technology Preview for years). It rides on **QRhi** underneath: Qt's in-house GPU abstraction layer unifying Vulkan/Metal/D3D11/D3D12/OpenGL ES (note: **Qt6's D3D11 is a native backend, not ANGLE** — refuted by the ANGLE research; only the Qt5 `-opengl es2 -angle` era was the translation-layer usage). QGIS 3D is built on Qt3D (chained evidence: even the official component is merely at the "usable" level).

## For VisiaEngine
- **Adopt**: ① QRhi = the C++ engineering sample of multi-backend compatibility differences (backend capability probing, the glslang->per-target shader compilation pipeline, texture/resource abstraction), a mirror control for wgpu; ② Quick3D's QML property-binding <-> engine-data bridging shape = the problem checklist of "keeping host UI framework and render core in sync"; ③ the USD importer (Quick3D ships one) = a precedent for twin exchange-format intake.
- **Avoid (lessons first)**: ① **star counts = the truth of ecosystem heat**: Qt official + commercial promotion yields only 74 stars — in the 3D domain there is no developer gravity without a data story; Visia's moat must sit on geo/twin data semantics, not "yet another 3D rendering API"; ② the two-generation API break (Qt3D->Quick3D) = the cost of locking APIs early in an abstraction layer; the first visia-render trait version must reserve room for breaking evolution (crate version-number discipline); ③ a Technology Preview that takes years to graduate — beware "preview" semantics in outward commitments.
- **Current-state ruling**: host-side knowledge (pitfalls of the future Qt embedding example: QRhi coexisting with a wgpu surface = two GPU contexts in one window, the interaction mode must be measured; prefer the contention-free path of "render into a texture and feed QML/Widget").

## Sources
shields measured (2026-09-03); Qt documentation public.
