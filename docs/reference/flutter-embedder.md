# Flutter Embedder — the gold standard of C-API host embedding

**Snapshot 2026-09-03, measured**: flutter/flutter 179k★, active the same day. BSD-3 + patent grant.

## Profile
The **engine-host contract layer** of a mobile/desktop cross-platform UI framework: `flutter::FlutterEngineCreate/Awake/Shutdown` (versioned C API, embedder.h), a three-way decoupling of host window / platform thread / render thread (platform thread receives events, UI thread runs Dart, raster thread puts pixels on screen). The Windows desktop embedding history ran ANGLE(D3D11) (*pending cross-check against the ANGLE line citation*); the rendering core is migrating Skia->Impeller (direct Vulkan/Metal, dropping the GLES translation layer — **the reverse path: a new engine actively abandoning GLES**).

## For VisiaEngine
- **Adopt (the textbook of C-API discipline)**: ① **the versioned embedder ABI** (struct_size + API version field handshake) — visia-c-api should adopt the "first-field size + version" pattern outright, so future incompatibility is probeable; ② the three-state lifecycle (Create/Awake/Shutdown + the host decides when to pump) = the interface-language template for "does not hijack the main loop"; ③ the path of rendering into an external texture (the FlutterTexture callback) = the isomorphic solution for Qt/CEF/game-host embedding; ④ the decision chain of Impeller abandoning ANGLE is worth reading: the startup shader pre-compilation / feature-subset price paid for a GLES translation layer — **legacy compatibility is a tax, and how long you collect it must be actively decided**.
- **Avoid**: ① embedder.h's platform fragmentation (iOS/Android custom layers each growing their own bulk) — Visia's C API concentrates platform differences in one capability query; ② the tight coupling of engine version to host SDK (the Flutter SDK full bundle locks versions) = the counter-example of SDK distribution, the Visia core must be independently upgradeable.
- **Current-state ruling**: read the embedder.h comments before writing the visia-c-api design document; the "host priority" (desktop Qt/C# first, mobile later) together with Flutter's evolution direction (abandoning GLES) jointly hint: **the old-GLES device market may be evaporating** — the legacy research comes back as background weight.

## Sources
shields (2026-09-03); docs.flutter.dev/embedded/* public.
