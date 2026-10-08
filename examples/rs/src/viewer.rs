//! Cross-example window bootstrap (band V, plan `viewer-shared-band-2026-10-08`).
//!
//! Adjudicated scope (C2=A): the parts that are *literally identical* across the
//! E-series — the `--frames` argv loop and the winit/wgpu open-a-window sequence
//! (instance → adapter → device → surface → config → `MeshCore`), the resize
//! reconfigure, and the present match including the `Outdated | Lost` retry arm.
//!
//! Deliberately NOT here: interaction semantics. Drag/wheel/rig live in each
//! example, because the same event means different things across them (E402/E403
//! drag = pick/box-select, E510 wheel = phase, E303 = per-viewport follow). A
//! shared `Orbit` would homogenize distinct claims (C2 ruling: deferred to a
//! trigger, see V4 of the plan).
//!
//! What stays local, and must: the scene, the `Frame`, and every pixel assertion —
//! those ARE the lesson of each example.
//!
//! Format policy is a parameter, not a constant: 22 examples take the surface
//! default, 4 (E204/E206/E502/E503) explicitly prefer an Srgb format. Collapsing
//! that difference is exactly how the historical Bgra/Rgba mismatch (PIT-9) was
//! born, so `Ctx::new` requires the caller to say which one it wants.

use std::sync::Arc;

use visiaengine_render_wgpu::mesh_core::MeshCore;
use winit::dpi::PhysicalSize;
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

// A compile-time reminder that the helper itself is only reachable from the
// examples package, never from the shipped SDK surface.
const _: fn() -> Option<u32> = argv_frames;
