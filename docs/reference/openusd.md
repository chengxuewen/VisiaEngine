# OpenUSD — the scene-exchange standard of digital twins (not an engine)

**Snapshot 2026-09-03, measured**: PixarAnimationStudios/OpenUSD 7.5k★, last-commit August. Apache-2.0. The AOUSD alliance (Pixar/ADSK/Bentley/NVIDIA).

## Profile
A scene-description and exchange framework: Prim tree + typed schemas + **composition (overlays/inheritance/variants/references — non-destructive assembly of large scenes)** + extension libraries (usdGeom/usdPhysics/usdRender...). Hgi = its internal graphics abstraction layer (yet another "thin backend abstraction" industrial sample, control against QRhi/wgpu-hal). Film/twin/CAD (the whole ADSK line, NVIDIA Omniverse) converging toward it.

## For VisiaEngine
- **Adopt**: ① **the composition-arcs idea** — the visualization layer model of digital twins as "base scene + per-tenant overlay layers" (if the Visia scene graph supports references/overrides it aligns with this semantics, even without reading USD); ② schema extensibility (a strong-typed prim-class registry) = a copyable design for the visia-core entity model; ③ usdRender's render settings/terminals = the precedent of separating the "scene <-> renderer" configuration surface.
- **Avoid**: ① **do not link USD C++** (heavy, slow to install, version hell) — build a **USD import/export adapter** (reading the usda text subset) and that suffices; twin customers want in/out compatibility, not runtime USD; ② the spec's breadth (USD is a production-pipeline standard; Visia takes only the geom+xform+material subset).
- **Current-state ruling**: not MVP-relevant; come back and read at the Visia Studio data-exchange + twin enterprise-plugin (Bentley/NVIDIA ecosystem hookup) stage. Pairs with the bim-ifc file (USD governs assembly/presentation, IFC governs BIM semantics).

## Sources
shields (2026-09-03); openusd.org/AOUSD public.
