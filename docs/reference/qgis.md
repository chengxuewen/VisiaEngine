# QGIS — the UX dictionary of layer semantics / plugin economy / dual-language bindings

**Snapshot 2026-09-03, measured**: qgis/qgis 14k★, active the same day. GPL-2.0+. The QGIS.org community + a large commercial user base (governments/utilities).

## Profile
A desktop GIS application (not an engine): C++ core + PyQGIS auto-bindings (sip); the QgsProject document model, **the layer tree + layer-type family** (vector/raster/mesh/point-cloud/temporal under one unified interface), renderers/symbol system (SVG markers, graduated renderers), the expression engine (filter/label syntax), the **plugin repository** (free Python plugins + some commercial support = a community-economy sample), QGIS Server (FWS/WMS/WFS), and its 3D view built on **Qt3D** (-> the qt-3d file's cold-field chain).

## For VisiaEngine
- **Adopt (heavy, but layer by layer)**: ① **the unified layer-type interface + the renderer/symbol system separation** (different renderers on one vector layer) = the semantic baseline of Visia's style system, more GIS-canonical than deck.gl; ② the expression language (field filtering / label computation) — the minimal viable form of twin data binding; ③ the temporal controller (timeline -> data refresh) = the product shape of digital-twin playback; ④ the dual-layer API discipline of **C++ core + thin Python bindings** = route validation for Visia's Rust core -> C API -> TS/Py/C# binding generator (a selection reference for binding-generation tooling); ⑤ the project file format (.qgz XML zip, carrying styles/layer tree) = the semantic checklist for Visia Studio's save format.
- **Avoid**: ① the gray boundary of a GPL core + closed-source plugins (Visia's MIT core + commercial plugins is actually cleaner — state the compatibility); ② the CPU QPainter rendering route's ceiling on massive data (weak 3D = the Qt3D drag — exactly Visia's opportunity, not its study target); ③ UI feature accretion (hundreds of entries per release notes — a feature-gravity-out-of-control sample; the Visia kernel stays restrained at the SDK boundary).
- **Current-state ruling**: do not touch its code base (GPL), but the MVP's "users recognize GIS semantics" acceptance checklist (layer toggles / attribute table / labels / symbol classes) comes from QGIS habits — the target users' muscle memory is this.

## Sources
shields (2026-09-03); docs.qgis.org public.
