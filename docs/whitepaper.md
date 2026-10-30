# VisiaEngine Whitepaper

> Translated from the Chinese original (v0.1.0, 2026-09). The Chinese original is preserved in git history (tag: whitepaper-zh-v0.1.0).

## The multi-dimensional spatial visualization engine

**Version 0.1.0 | September 2026**

---

## Abstract

**VisiaEngine** is an open-source, lightweight multi-dimensional spatial visualization engine. It provides a unified **2D, 2.5D and 3D rendering pipeline**, purpose-built for **maps, geographic information (GIS), 3D models, simulation scenes and digital-twin data**.

VisiaEngine does not chase game-grade lighting effects; it focuses on **presenting data clearly, accurately and efficiently**. Its core is implemented in high-performance **Rust**, delivered in a flexible **SDK shape**, embeddable in **Qt and Web** applications (Flutter/C# are roadmap items), with a planned official editor, **Visia Studio**. The project follows the **Open Core** model: the core runtime is permanently open source, while advanced enterprise features and the editor are commercial products.

---

## §1. Problem and Opportunity

### 1.1 The split in today's market
The current visualization tooling market has a pronounced fault line:
- **Game engines (Unreal, Unity)**: huge footprint, weak GIS support, hard to embed into ordinary desktop/Web apps, and their "game-style" rendering habitually distracts from data visualization.
- **GIS and Web 3D libraries (Cesium, MapLibre, deck.gl)**: good at geographic data display, but most are confined to the Web stack, and deep embedding into native desktop frameworks (Qt, WPF) is costly; even mature products that offer 2D/2.5D/3D mode switching usually run a different scene semantic per dimension, making continuously interpolated transitions over one scene tree with one layer/entity semantics hard to achieve.
- **General graphics frameworks (Three.js, Qt 3D)**: lack the layer system, coordinate transforms and interaction purpose-built for spatial data; the development bar is high.

### 1.2 The unmet need
Developers urgently need a **solution between the lightweight library and the heavy engine**:
- Native support for **2D maps, the 2.5D oblique view and 3D scenes** with free switching and camera flight transitions **within the same viewport**.
- Native understanding and optimization of spatial data such as **GIS coordinate systems, vector tiles, and ODR high-definition roads**.
- Packageable as a lightweight SDK embedded into existing applications, yet able to drive the future editor as a standalone engine.
- Outstanding performance, cross-platform consistency, and a dependable visualization base for digital twins and industrial simulation.

---

## §2. What VisiaEngine Is

**VisiaEngine** is a multi-dimensional visualization engine whose first goal is **the clear presentation of data**.
- **Visia** is drawn from the vision word family (Latin visio, "sight"), signifying **"the domain of vision"**.
- It is not a game engine and does not compete with Unreal or Unity; it focuses on **turning complex spatial data into intuitive, interactive views**.

### Core creed
> **Visia — see the essence of your data.**
> *Visia — See the essence of your data.*

---

## §3. Core Features

### 3.1 A unified 2D / 2.5D / 3D core
- On a flat entity table (scene-tree grouping / parent-child offsets already delivered: the create_group/set_parent family), switch freely between the **bird's-eye map (2D)**, the **oblique view (2.5D)** and the **fully immersive 3D viewport**.
- All layers, entities and annotations keep consistent interaction logic across the dimensions.

### 3.2 First-class spatial data capability
- **Maps and GIS**: built-in GeoJSON and mainstream coordinate-projection support; vector tiles (MVT) support is delivered (Phase 0 = Rust-native decode/scheduling; Phase 1 = the C API entries load_mvt_dir/set_tile_view + the HTTP source [ureq], delivered in this band; WMS/WMTS raster imagery is in the roadmap).
- **Autonomous driving and simulation**: **OpenDRIVE (ODR)** and **OpenSCENARIO (OSC)** through official simulation plugins, auto-generating road and traffic scenes (delivered in Beta, see the roadmap).
- **Models and BIM**: efficient loading of glTF, point clouds and other standard formats, supporting high-fidelity lightweight display of building information models.

### 3.3 Lightweight, embeddable, high-performance
- **Rust core**: memory safe, no GC, performance comparable to C++.
- Modern rendering on **wgpu**: first-class Vulkan / Metal / DX12 / WebGPU; also provides the OpenGL 3.3+ / OpenGL ES 3.0+ / WebGL2 degraded compatibility tiers — what stays consistent across platforms is the API surface and the quality-tier semantics; the degraded tiers trade a feature subset for coverage.
- **Delivered as an SDK**: shipped as runtime libraries (`visiaengine-core`, `visiaengine-render`) over a C API for hosts such as Qt, Flutter and C# (WPF/Unity), without hijacking the application main loop.

### 3.4 Built for developers
- A compact, prunable modular design; the startup size can be kept **within 10 MB**.
- Clear, stable APIs backed by thorough documentation and examples.
- Future second-development through official scripting or an extension system (WebAssembly considered).

---

## §4. Technical Architecture

### 4.1 The core idea
An architecture of **"engine kernel + pluggable rendering backends + application-layer bindings"**.

```mermaid
flowchart TB
    host["Application layer (host)<br/>Qt / Flutter / C# / Web"]
    core["visiaengine-core<br/>data model · spatial index · coordinates · scene graph · resource management"]
    render["visiaengine-render<br/>unified rendering abstraction<br/>(Trait definitions, render-command generation)"]
    wgpu["visiaengine-render-wgpu (default backend)<br/>in-house rendering pipeline on wgpu"]
    future["future optional backends"]
    host -->|FFI (C API)| core --> render --> wgpu
    render -.-> future
```

### 4.2 Key design decisions
- **Why wgpu direct, pipeline in-house?** (the 2026-09 final backend ruling) wgpu provides a Rust-native modern GPU abstraction spanning Vulkan/Metal/DX12/WebGPU and ships its own GL/GLES degradation backends (an optional ANGLE build on Windows) — main-loop ownership, the size budget and compatibility tiering can all be held by the engine.
- Reusing an off-the-shelf engine's rendering stack (Bevy) was evaluated and not adopted: its rendering layer is inseparable from its own ECS/App framework, and the upstream API-stability cadence does not match an SDK's long-term compatibility promise.
- The `visiaengine-render` abstraction layer stays: future optional backends (a mobile-exclusive or an enterprise customization, say) still join through the same C API compatibility discipline.
- **How are 2D/2.5D/3D unified?** All visual primitives (points, lines, faces, volumes) exist as spatial entities; perspective/orthographic projections switch freely and bird's-eye/oblique camera flights need no renderer swap. (The continuously interpolated projection morph between viewpoints is delivered: the `set_morph_time` continuous slider, E304.)

---

## §5. Application Scenarios

1. **Maps and geographic information systems (GIS)**
   A next-generation high-performance desktop/mobile map component replacing Mapbox/Leaflet, with zoomless continuous scaling and massive vector overlays.

2. **Digital twins and smart cities**
   Binding live IoT data streams into 2.5D/3D scenes, driving the dynamic presentation of devices, buildings and traffic conditions.

3. **Autonomous-driving simulation (AV / ADAS)**
   Consuming OpenDRIVE/OpenSCENARIO data through official plugins to generate lifelike 3D road environments, for sensor simulation and planning-and-control algorithm validation.

4. **AGV/AMR dispatch and monitoring**
   Embedding full-scene 2D/3D monitoring viewports into industrial WPF/Qt applications, drawing vehicles, paths, tasks and heatmaps live.

5. **Industrial design and model presentation**
   A lightweight, embeddable 3D viewer for CAD/BIM data, with sectioning, measurement and annotation.

---

## §6. Commercial Strategy and Roadmap

### 6.1 The Open Core model
- **Open-source core**: the `visiaengine-core` and `visiaengine-render` runtime libraries are fully open source under the MIT OR Apache-2.0 dual license, **licenses of released versions are irrevocable and the core is never retroactively closed**, integrable by any commercial or non-commercial project.
- **Commercial products**:
    - **Visia Studio**: a visual editor for designers and developers — scene building, style configuration, data-source management and one-click deployment. Subscription pricing.
    - **Enterprise plugins and support**: out-of-the-box enterprise-grade GIS, digital-twin and simulation plugins, plus SLA-backed technical support.

### 6.2 Roadmap

- **Phase 1: MVP (current)**
    - Core Rust libraries (`visiaengine-core`, `visiaengine-render`, `visiaengine-render-wgpu`)
    - Basic glTF and GeoJSON loading
    - Free switching between the 2D map and the 3D viewport, demonstrated
    - Qt/Web embedding examples (Flutter/C# are Alpha-tier candidates)

- **Phase 2: Alpha**
    - Mature spatial indexing and coordinate systems
    - 2.5D white-model buildings and the oblique view
    - C API stabilization and multi-language binding generation

- **Phase 3: Beta**
    - Visia Studio preview released
    - ODR/OSC parsing and visualization plugins
    - Digital-twin data binding and live-update mechanisms

- **Phase 4: 1.0 stable**
    - The complete editor + runtime product released
    - A plugin marketplace and developer community established
    - Formal commercial licensing and support services launched

---

## §7. Team and Community

*(This section fills in after the project launches: core contributors, organization background, contribution guide, code of conduct, etc.)*

VisiaEngine is committed to building a friendly, inclusive global developer community. We welcome contributions of any kind, from code and documentation to sharing application cases.

---

## §8. In Closing

VisiaEngine is a bold attempt — it sets out to fill a long-neglected gap: **to give a "data-driven" world a pure, powerful, ubiquitous visualization engine**.

We chose Rust for uncompromising performance and reliability; we insist on the SDK shape and the editor in parallel to grant developers the greatest freedom and efficiency. Visia is not merely an engine; it is a promise: **let your data be truly seen.**

---

**Contact**
- Official website: *(visiaengine.dev pending registration)*
- Repository: gitee.com/chengxuewen/VisiaEngine (GitHub organization pending creation)
- Email: *(TBD)*

*© 2026 The VisiaEngine project team. Some rights reserved.*
