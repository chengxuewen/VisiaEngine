//! The L1 headless backend: MeshCore + offscreen target + readback (the golden-test consumption face).

use visiaengine_render::{
    BackendError, Capability, Frame, InstanceDesc, InstanceId, LabelTableDesc, MaterialDesc,
    MaterialId, MeshDesc, MeshId, PointTableDesc, RenderBackend, StrokeTableDesc, TableId,
    TextureDesc, TextureId, Viewport,
};

use crate::mesh_core::MeshCore;
use crate::offscreen::OffscreenFrame;

/// The windowless backend: the public face matches the REND contract (the G3 window path shares MeshCore).
pub struct HeadlessBackend {
    core: MeshCore,
    viewport: Viewport,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
}

impl HeadlessBackend {
    /// I band (WGPU-39): forward env SH to the shared MeshCore.
    pub fn set_env_sh(&mut self, sh: [[f32; 3]; 9]) {
        self.core.set_env_sh(sh);
    }

    /// V2.1: direct MeshCore access (resource building on the headless
    /// backend's own device — E510 dual-lane scene builder shares one type).
    pub fn core_mut(&mut self) -> &mut MeshCore {
        &mut self.core
    }

    #[must_use]
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let sh = crate::mesh_core::create_shared_device()?;
        let core = MeshCore::new(sh.device, sh.queue, sh.instance, sh.adapter);
        let viewport = Viewport::new(width, height, 1.0);
        let (target, target_view) = Self::make_target(&core.device, width, height);
        Some(Self {
            core,
            viewport,
            target,
            target_view,
        })
    }

    /// Async construction (web / batch 7 J1; native equivalent of `new`).
    pub async fn new_async(width: u32, height: u32) -> Option<Self> {
        let sh = crate::mesh_core::create_shared_device_async().await?;
        let core = MeshCore::new(sh.device, sh.queue, sh.instance, sh.adapter);
        let viewport = Viewport::new(width, height, 1.0);
        let (target, target_view) = Self::make_target(&core.device, width, height);
        Some(Self {
            core,
            viewport,
            target,
            target_view,
        })
    }

    /// Canvas attach (wasm32 + web feature; the swapchain shares the device's instance).
    #[cfg(all(target_arch = "wasm32", feature = "web"))]
    pub fn attach_canvas(
        &mut self,
        canvas: &web_sys::HtmlCanvasElement,
    ) -> Result<crate::surface::Swapchain, visiaengine_render::BackendError> {
        let mut sw = crate::surface::Swapchain::from_target(
            &self.core,
            wgpu::SurfaceTarget::Canvas(canvas.clone()),
            self.viewport.width(),
            self.viewport.height(),
        )?;
        sw.configure(&self.core)?;
        Ok(sw)
    }

    fn make_target(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen-target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb, // the CORE-16 output end: write linear, the store hardware-encodes
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        (target, view)
    }

    /// 5b multi-viewport render + readback (rect domain; policy = the safe/ruling two forms of WGPU-26).
    #[must_use]
    pub fn render_to_pixels_rects(
        &mut self,
        passes: &[(Frame, visiaengine_render::ViewportRect)],
        policy: crate::MultiClearPolicy,
    ) -> Option<OffscreenFrame> {
        let (w, h) = (self.viewport.width(), self.viewport.height());
        self.core.render_view_rects(
            passes,
            &self.target_view,
            w,
            h,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            policy,
        );
        self.read_back()
    }

    /// Render and read back (synchronous).
    #[must_use]
    pub fn render_to_pixels(&mut self, frame: &Frame) -> Option<OffscreenFrame> {
        self.render(frame);
        self.read_back()
    }

    /// Read back the current target (row-alignment depadded; _pixels/_rects share the tail).
    #[must_use]
    fn read_back(&mut self) -> Option<OffscreenFrame> {
        let _unused: Option<Frame> = None;
        let (w, h) = (self.viewport.width(), self.viewport.height());
        // row alignment (COPY_BYTES_PER_ROW_ALIGNMENT): any w is legal (capi host sizes are uncontrollable --
        // the original implementation only worked at 64px multiple widths, the golden domain happened to dodge it, the I2 image chain nailed the fix)
        let bpr_raw = (w * 4) as u64;
        let bpr = bpr_raw.div_ceil(u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT))
            * u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let size = bpr * h as u64;
        let readback = self.core.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .core
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpr as u32),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        self.core.queue.submit(std::iter::once(encoder.finish()));
        let (sender, receiver) = std::sync::mpsc::channel();
        readback.map_async(wgpu::MapMode::Read, 0..size, move |res| {
            let _ = sender.send(res);
        });
        let _ = self.core.device.poll(wgpu::PollType::wait_indefinitely());
        receiver
            .recv()
            .ok()?
            .map_err(|e| eprintln!("map failed: {e}"))
            .ok()?;
        let raw = readback.get_mapped_range(0..size).ok()?;
        // depad: cut each row back from packed (the outward OffscreenFrame contract unchanged, consumers feel nothing)
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h as usize {
            let start = y * bpr as usize;
            rgba.extend_from_slice(&raw[start..start + bpr_raw as usize]);
        }
        drop(raw);
        readback.unmap();
        Some(OffscreenFrame {
            width: w,
            height: h,
            rgba,
        })
    }
}

impl RenderBackend for HeadlessBackend {
    fn name(&self) -> &'static str {
        "wgpu-headless"
    }

    fn supports(&self, capability: Capability) -> bool {
        MeshCore::supports(capability)
    }

    fn resize(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        let (t, v) = Self::make_target(&self.core.device, viewport.width(), viewport.height());
        self.target = t;
        self.target_view = v;
    }

    fn render(&mut self, frame: &Frame) {
        // the target format must match make_target's Srgb form (the pipeline/pass compatibility lock)
        self.core.render_view_format(
            frame,
            &self.target_view,
            self.viewport.width(),
            self.viewport.height(),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
    }

    fn create_mesh(&mut self, desc: &MeshDesc<'_>) -> Result<MeshId, BackendError> {
        self.core.upload_mesh(desc)
    }

    fn create_material(&mut self, base_color: [f32; 4]) -> Result<MaterialId, BackendError> {
        self.core.upload_material(base_color)
    }

    fn create_material_desc(&mut self, desc: &MaterialDesc) -> Result<MaterialId, BackendError> {
        self.core.upload_material_desc(desc)
    }

    fn upload_texture(&mut self, desc: &TextureDesc<'_>) -> Result<TextureId, BackendError> {
        self.core.upload_texture(desc)
    }

    fn create_instances(&mut self, desc: &InstanceDesc<'_>) -> Result<InstanceId, BackendError> {
        self.core.create_instances(desc)
    }

    fn create_strokes(&mut self, desc: &StrokeTableDesc<'_>) -> Result<TableId, BackendError> {
        self.core.create_strokes(desc)
    }

    fn destroy_points(&mut self, id: TableId) {
        self.core.destroy_points(id);
    }

    fn create_points(&mut self, desc: &PointTableDesc<'_>) -> Result<TableId, BackendError> {
        self.core.create_points(desc)
    }

    fn create_labels(&mut self, desc: &LabelTableDesc<'_>) -> Result<TableId, BackendError> {
        self.core.create_labels(desc)
    }
}

impl HeadlessBackend {
    /// Glyph atlas upload (WGPU-24): io-text `GlyphCache::pixels()` fed directly (R8 wholesale replacement).
    pub fn set_glyph_atlas(
        &mut self,
        r8: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), BackendError> {
        self.core.set_glyph_atlas(r8, width, height)
    }

    /// Total quads of the built label tables (host introspection / the GEO-25 semantics assertion entry).
    #[must_use]
    pub fn label_mark_total(&self) -> usize {
        self.core.label_mark_total()
    }

    /// attach face (CAPI-06): build the swapchain from raw handles (returns only on a successful configure; failure leaves the current target untouched).
    ///
    /// # Safety
    /// Handle-lifetime duty per [`Swapchain::from_raw_handles`] (carried in by the capi attach contract).
    pub unsafe fn attach_surface(
        &mut self,
        display: Option<raw_window_handle::RawDisplayHandle>,
        window: raw_window_handle::RawWindowHandle,
    ) -> Result<crate::surface::Swapchain, BackendError> {
        let mut sw = unsafe {
            crate::surface::Swapchain::from_raw_handles(
                &self.core,
                display,
                window,
                self.viewport.width(),
                self.viewport.height(),
            )
        }?;
        sw.configure(&self.core)?;
        Ok(sw)
    }

    /// Window frame render (the swapchain path; the headless target does not participate).
    pub fn render_swapchain(
        &mut self,
        frame: &visiaengine_render::Frame,
        sw: &mut crate::surface::Swapchain,
    ) -> Result<crate::surface::SwapOutcome, BackendError> {
        sw.render(&mut self.core, frame)
    }

    /// 5b second wave: the swapchain multi-frame path.
    pub fn render_swapchain_multi(
        &mut self,
        passes: &[(visiaengine_render::Frame, visiaengine_render::ViewportRect)],
        sw: &mut crate::surface::Swapchain,
    ) -> Result<crate::surface::SwapOutcome, BackendError> {
        sw.render_multi(&mut self.core, passes)
    }
}
