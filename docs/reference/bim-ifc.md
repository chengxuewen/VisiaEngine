# BIM/IFC — display-layer boundaries and the license minefield

**Snapshot 2026-09-03, measured**: IfcOpenShell 2.7k★ (active yesterday, **LGPL**); ThatOpen/web-ifc 1k★ (last Friday, MPL *recheck*); FreeCAD 33k★; OCCT 2.8k★ (**LGPL-2.1+exception**). Bonsai (formerly BlenderBIM) stars unmeasured (repo mid-migration) *UNCERTAIN*.

## Profile
IFC = the BIM data-exchange standard (buildingSMART, STEP physical format). IfcOpenShell = the de-facto standard parsing-geometry kernel (C++ core + Python API, BlenderBIM/Bonsai built on it); web-ifc = a WASM parser (under Web-side BIM viewers); OCCT = the B-rep CAD geometry kernel (FreeCAD) — **IFC geometry (CSG/B-rep mix) -> tessellation is the correct landing point for visualization**.

## For VisiaEngine (the boundary file for the "high-fidelity lightweight BIM display" promise)
- **Adopt**: ① **the display-layer stance**: the IFC semantic tree (IfcProject->Site->Building->Storey->Element) + pre-tessellated geometry = the minimal data surface needed for "sectioning/measuring/annotating" (measurement = mesh ray picking; semantic navigation = tree filtering) — **do not enter parametric-geometry replay (that is OCCT's 20 years)**; ② web-ifc's wasm streaming strategy (on-demand element parsing) = the web-side precedent for lightweighting large models; ③ the interface shape of IfcOpenShell's geom processing layer (element -> triangulated item).
- **Avoid (license)**: IfcOpenShell is **LGPL** -> statically packaging it into the Visia SDK carries obligation boundaries; **the right posture = IFC parsing in an optional plugin crate (dlopen/dynamic-link isolation), not core** — the same principle as the gdal-proj file: all copyleft pieces feature-gated into the enterprise/extension layer (hands the Open Core paid-plugin candidate boundary over for free).
- **Current-state ruling**: MVP promises only glTF (the BIM-tools-export-glTF path exists and is license-clean — guide users that way); native IFC support is slated Beta+ and designed per the boundary above. Point clouds: see `evidence/2026-09-03-wgpu-direct` R1 (las maintained, pcd dead, draco-core new and thin).

## Sources
shields (2026-09-03); docs.buildingsmart.org / ifcopenshell / readthedocs public.
