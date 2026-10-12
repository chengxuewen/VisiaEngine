# deck.gl / kepler.gl — the "data-visualization layer" shape baseline

**Snapshot 2026-09-03, measured**: visgl/deck.gl 15k★ (active the same day); npm 9.3.11 (2026-08-28); keplergl/kepler.gl 12k★ (3.3.0-alpha.10, 2026-09-02). MIT. OpenJS Foundation governance.

## Profile
A WebGL/WebGPU data-visualization layer library (Uber origins): ~20 GPU layer types (Scatterplot3D/Arc/HexBin/S3/TextLayer/GeoJson…), a data->GPU-attribute pipeline + lightweight shader injection + JSON-declared props. **It makes no map itself**: it rides on maplibre/google-maps, filling the "data-driven 2.5D layers" half of the space. kepler.gl = the analytical app on top (a React+redux UI dictionary).

## For VisiaEngine
- **Adopt (heavy)**: ① the abstraction of layers-as-GPU-pipeline modules (uniform injection + extension-layer registration); Visia's "spatial entity / annotation / IoT effect" layer family can use its inventory as a requirements document; ② the **aggregation/density layer family** (hexbin/grid/clustering) = the standard shape for digital twins and AGV heatmaps; ③ the picking interaction implementation (color-encoded ID picking + collision); ④ luma.gl's WebGPU migration path (GL/GPU dual-backend lifecycle management) = a multi-backend transition engineering sample; ⑤ the WebGPU backend's post-2024 landing experience = a free wgpu-compatibility reference.
- **Avoid**: ① the JS glue: data must be fully uploaded to the GPU, no persistent spatial index -> frame hitches with million-scale live data, Visia's native kernel hits exactly this pain point (a differentiation argument); ② 20 layers each with a combinatorial explosion of parameters — do not copy its breadth into the abstraction layer, copy the semantic classification; ③ kepler's alpha has dragged for years (3.3 still -alpha): an analytical-app productization cost signal.
- **Current-state ruling**: the functional spec for Visia §5.2/§5.4 (twin/AGV); the MVP's 2.5D minimal layer set = GeoJson + Scatterplot + Text + Path/Arc + Heatmap.

## Sources
shields/npm (2026-09-03); public documentation.
