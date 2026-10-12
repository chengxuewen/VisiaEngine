//! The attach-side swapchain wrapper (I3/CAPI-06; web Canvas reuses the same form, batch 7 J1).
//! Contract: the surface and MeshCore's device/adapter pair on the same backend; Outdated/Lost
//! reconfiguration = digested within the frame (the wgpu counterpart of Filament's frame-skip semantics, D8 trigger (1) delivered).

use visiaengine_render::{BackendError, Frame};

use crate::mesh_core::MeshCore;

/// Window exchange-chain state (the headless backend does not hold this; after attach it becomes the engine target state).
pub struct Swapchain {
    surface: wgpu::Surface<'static>,
    pub width: u32,
    pub height: u32,
    configured: bool,
    chosen: Option<wgpu::TextureFormat>,
}

/// One frame's exchange result.
#[derive(PartialEq, Eq, Debug)]
pub enum SwapOutcome {
    /// already presented
    Presented,
    /// vsync backpressure / occlusion: this frame is dropped without fault (the host may keep firing)
    Skipped,
    /// size/capability outdated; recovers on the next frame after configure
    Reconfigured,
}

impl Swapchain {
    /// Build the surface from a generic target (Canvas/OffscreenCanvas/native wrapper).
    ///
    /// # Errors
    /// Returns an error when the backend does not support the target.
    pub fn from_target(
        core: &MeshCore,
        target: wgpu::SurfaceTarget<'static>,
        width: u32,
        height: u32,
    ) -> Result<Self, BackendError> {
        let surface = core
            .instance
            .create_surface(target)
            .map_err(|e| BackendError {
                reason: format!("{e:?}"),
            })?;
        Ok(Self {
            surface,
            width,
            height,
            configured: false,
            chosen: None,
        })
    }

    /// Build the surface from raw window handles (CAPI-06 attach; x11 must carry display, win32 hinstance may be absent).
    ///
    /// # Safety
    /// The raw `display`/`window` handles must outlive this `Swapchain` (a host duty,
    /// a CAPI-06 contract clause), and must be legal platform handles of the matching kind.
    pub unsafe fn from_raw_handles(
        core: &MeshCore,
        display: Option<raw_window_handle::RawDisplayHandle>,
        window: raw_window_handle::RawWindowHandle,
        width: u32,
        height: u32,
    ) -> Result<Self, BackendError> {
        // SAFETY: the above obligations are borne by the caller (capi ffi attach) via host-contract validation
        let surface = unsafe {
            core.instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: display,
                    raw_window_handle: window,
                })
                .map_err(|e| BackendError {
                    reason: format!("{e:?}"),
                })?
        };
        Ok(Self {
            surface,
            width,
            height,
            configured: false,
            chosen: None,
        })
    }

    /// Configure/reconfigure (shared by size changes and Outdated recovery).
    ///
    /// # Errors
    /// Returns an error when no supported format exists or the size is illegal.
    pub fn configure(&mut self, core: &MeshCore) -> Result<(), BackendError> {
        if self.width == 0 || self.height == 0 {
            return Err(BackendError {
                reason: "swapchain zero size".into(),
            });
        }
        let mut config = self
            .surface
            .get_default_config(&core.adapter, self.width, self.height)
            .ok_or_else(|| BackendError {
                reason: "no compatible surface config".into(),
            })?;
        // align the format with the pipeline (the mesh pipeline is fixed Rgba8Unorm -- the default config may pick
        // a Bgra* variant and blow up render-pass validation; proven by I3 smoke-x11). When capabilities
        // lack Rgba, fall back to default and let the pre-configure compatibility check report the error.
        // the pipeline is built lazily by format (Bgra* is legal too): Rgba8Unorm first to keep the golden form identical
        let caps = self.surface.get_capabilities(&core.adapter);
        let fmt = if caps.formats.contains(&wgpu::TextureFormat::Rgba8Unorm) {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            config.format
        };
        config.format = fmt;
        self.chosen = Some(fmt);
        config.width = self.width;
        config.height = self.height;
        config.present_mode = wgpu::PresentMode::AutoVsync;
        self.surface.configure(&core.device, &config);
        self.configured = true;
        Ok(())
    }

    /// Set the new size (the viewport notification path; configure on the next frame).
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.configured = false; // force configure
        self.chosen = None;
    }

    /// One frame: acquire -> render -> present. Timeout/Occluded = Skipped (not an error).
    ///
    /// # Errors
    /// Unconfigured/device errors propagate.
    /// 5b second wave: multi-viewport frame sequence (when the corner window is on, main + top view both project).
    pub fn render_multi(
        &mut self,
        core: &mut MeshCore,
        passes: &[(Frame, visiaengine_render::ViewportRect)],
    ) -> Result<SwapOutcome, BackendError> {
        if !self.configured {
            self.configure(core)?;
        }
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                let view = tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let fmt = self.chosen.expect("configured");
                core.render_view_rects(
                    passes,
                    &view,
                    self.width,
                    self.height,
                    fmt,
                    crate::MultiClearPolicy::FirstClearRestLoad,
                );
                core.queue.present(tex);
                Ok(SwapOutcome::Presented)
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                Ok(SwapOutcome::Skipped)
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.configured = false;
                self.configure(core)?;
                Ok(SwapOutcome::Reconfigured)
            }
            _ => Err(BackendError {
                reason: "surface acquire failed".into(),
            }),
        }
    }

    pub fn render(
        &mut self,
        core: &mut MeshCore,
        frame: &Frame,
    ) -> Result<SwapOutcome, BackendError> {
        if !self.configured {
            self.configure(core)?;
        }
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                let view = tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let fmt = self.chosen.expect("configured");
                core.render_view_format(frame, &view, self.width, self.height, fmt);
                core.queue.present(tex);
                Ok(SwapOutcome::Presented)
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                Ok(SwapOutcome::Skipped)
            }
            // Outdated/Lost: reconfigure and recover (digested in-frame; same dispatch in the examples)
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.configured = false;
                self.configure(core)?;
                Ok(SwapOutcome::Reconfigured)
            }
            _ => Err(BackendError {
                reason: "surface acquire failed".into(),
            }),
        }
    }
}
