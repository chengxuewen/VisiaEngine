//! Cross-example window bootstrap (band V, plan `viewer-shared-band-2026-10-08`).
//!
//! Adjudicated scope (C2=A): the parts that are *literally identical* across the
//! E-series — the `--frames` argv loop and the winit/wgpu open-a-window sequence
//! (instance → adapter → device → surface → config → `MeshCore`), the resize
//! reconfigure, and the present match including the `Outdated | Lost` retry arm.
//!
//! Interaction semantics: shared ONLY where the measurement said the code is the
//! same claim (`orbit_drag`, `zoom_dist` -- band V4, V4 trigger). Measured, not
//! assumed: E201/E501/E504/E901 have byte-identical drag bodies (E201 differed only
//! by `.take()` on a `Copy` Option, which is behaviourally the same thing) and one
//! wheel formula differing only in its per-scene clamp bounds. Still deliberately
//! local, each for a reason the shared piece would erase: E402/E403 (the drag arm is
//! a 3-tuple and its else-branch does hover-pick / box-select -- that IS the lesson),
//! E306 (pan + right-button tilt), E508/E509/E510/E511/E206 (they inline their own
//! spherical maths or give the wheel a non-camera meaning: E510 wheel = flow phase),
//! E303 (per-viewport follow).
//!
//! What stays local, and must: the scene, the `Frame`, and every pixel assertion —
//! those ARE the lesson of each example.
//!
//! Format policy is a parameter, not a constant: 22 examples take the surface
//! default, 4 (E204/E206/E502/E503) explicitly prefer an Srgb format. Collapsing
//! that difference is exactly how the historical Bgra/Rgba mismatch (PIT-9) was
//! born, so `Ctx::new` requires the caller to say which one it wants.

use std::sync::Arc;

use visiaengine_render::CameraRig;
use visiaengine_render_wgpu::mesh_core::MeshCore;
use winit::dpi::PhysicalSize;
use winit::event::MouseScrollDelta;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

/// Which surface texture format to pick from the capabilities list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatPolicy {
    /// The surface default (`get_default_config`), i.e. what 22 examples do today.
    SurfaceDefault,
    /// Prefer `Rgba8UnormSrgb`/`Bgra8UnormSrgb`, fall back to `formats[0]` —
    /// what E204/E206/E502/E503 do today. Color-calibration examples need this:
    /// it is the difference between the byte value on screen matching the CSS
    /// input or not.
    PreferSrgb,
}

/// Window + surface + configuration. The `MeshCore` comes back separately from
/// [`Ctx::new`] because examples hold it in their own app state.
pub struct Ctx {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl Ctx {
    /// Open the window and build the device/surface/`MeshCore` bundle every example
    /// otherwise re-types. `None` means no adapter (headless CI without lavapipe or
    /// a real GPU); the caller should exit its event loop, exactly as the per-example
    /// code did before this was shared.
    pub fn new(
        event_loop: &ActiveEventLoop,
        title: &str,
        width: u32,
        height: u32,
        policy: FormatPolicy,
    ) -> Option<(Self, MeshCore)> {
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(PhysicalSize::new(width, height))
                        .with_title(title),
                )
                .expect("create_window"),
        );
        let size = window.inner_size();
        let instance = visiaengine_render_wgpu::create_instance();
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("create_surface");
        let Some(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .ok()
        else {
            eprintln!("no adapter — lavapipe/real GPU required");
            return None;
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("visiaengine-window"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("request_device");
        let core = MeshCore::new(device, queue, instance, adapter.clone());

        let caps = surface.get_capabilities(&adapter);
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("surface config");
        if policy == FormatPolicy::PreferSrgb {
            config.format = caps
                .formats
                .iter()
                .copied()
                .find(|f| {
                    matches!(
                        f,
                        wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb
                    )
                })
                .unwrap_or(caps.formats[0]);
        }
        surface.configure(&core.device, &config);

        Some((
            Self {
                window,
                surface,
                config,
            },
            core,
        ))
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.config.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.config.height.max(1)
    }

    #[must_use]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// `WindowEvent::Resized` handler. Zero-sized sizes are ignored, matching the
    /// per-example code this replaced (minimized windows must not configure a 0×0
    /// surface).
    pub fn on_resize(&mut self, core: &MeshCore, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&core.device, &self.config);
    }

    /// One `RedrawRequested` present: acquire, render `frame` to the view, present.
    /// `Outdated | Lost` reconfigures and returns without drawing — the next
    /// `RedrawRequested` (which the caller should always re-request) retries.
    pub fn present(&mut self, core: &mut MeshCore, frame: &visiaengine_render::Frame) {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                let view = tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                core.render_view_format(
                    frame,
                    &view,
                    self.config.width,
                    self.config.height.max(1),
                    self.config.format,
                );
                core.queue.present(tex);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&core.device, &self.config);
            }
            other => eprintln!("skipped: {other:?}"),
        }
    }
}

/// Shared `--frames N` parse: `Some(n)` = headless self-asserting lane (CI/ctest),
/// `None` = resident human window (C15 dual mode). `0` is treated as absent, and a
/// non-numeric value likewise — the filter every example with this flag has today.
#[must_use]
pub fn argv_frames() -> Option<u32> {
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--frames" {
            return args
                .next()
                .and_then(|v| v.parse().ok())
                .filter(|n: &u32| *n > 0);
        }
    }
    None
}

/// For the 5 examples that carry extra flags: the shared `--frames` result plus the
/// untouched token list, so each example still decides what its own switches mean.
#[must_use]
pub fn argv_frames_and_rest() -> (Option<u32>, Vec<String>) {
    let all: Vec<String> = std::env::args().skip(1).collect();
    let frames = argv_frames();
    let mut rest = Vec::new();
    let mut skip_next = false;
    for a in &all {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a == "--frames" {
            skip_next = true;
            continue;
        }
        rest.push(a.clone());
    }
    (frames, rest)
}

/// Orbit-by-drag, the shape E201/E501/E504/E901 wrote identically.
///
/// The press arm stores the sentinel `(-1.0, -1.0)`: the *first* motion after a
/// press only seeds the anchor (no rotation), every later motion rotates by the
/// delta. Returns whether it rotated, so the caller keeps deciding whether to redraw.
/// Sensitivity is a constant, not a parameter -- all four callers agreed on 0.006,
/// and a knob nobody varies is decoration.
pub fn orbit_drag(rig: &mut CameraRig, dragging: &mut Option<(f64, f64)>, x: f64, y: f64) -> bool {
    let Some((lx, ly)) = dragging.take() else {
        return false;
    };
    if lx >= 0.0 {
        rig.orbit_delta((x - lx) * 0.006, (y - ly) * 0.006);
    }
    *dragging = Some((x, y));
    lx >= 0.0
}

/// Wheel -> dolly, clamped. The formula is shared; the bounds are per scene because
/// they are scene scale, not input feel (E201 0.3..100, E501 6..200, E504 10..160,
/// E901 20..180 -- measured from the four call sites).
pub fn zoom_dist(rig: &mut CameraRig, delta: &MouseScrollDelta, lo: f64, hi: f64) {
    let d = match delta {
        MouseScrollDelta::LineDelta(_, y) => -f64::from(*y) * 40.0,
        MouseScrollDelta::PixelDelta(p) => -p.y,
    };
    rig.dist = (rig.dist * (1.0 - d * 0.0012)).clamp(lo, hi);
}

// A compile-time reminder that the helper itself is only reachable from the
// examples package, never from the shipped SDK surface.
const _: fn() -> Option<u32> = argv_frames;
