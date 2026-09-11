//! L1 无头后端：MeshCore + 离屏 target + 回读（golden 测试消费面）。

use visiaengine_render::{
    BackendError, Capability, Frame, MaterialId, MeshDesc, MeshId, RenderBackend, Viewport,
};

use crate::mesh_core::MeshCore;
use crate::offscreen::OffscreenFrame;

/// 无窗口后端：公开面与 REND 契约一致（G3 窗口路径共享 MeshCore）。
pub struct HeadlessBackend {
    core: MeshCore,
    viewport: Viewport,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
}

impl HeadlessBackend {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let (device, queue) = crate::mesh_core::create_shared_device()?;
        let core = MeshCore::new(device, queue);
        let viewport = Viewport::new(width, height, 1.0);
        let (target, target_view) = Self::make_target(&core.device, width, height);
        Some(Self {
            core,
            viewport,
            target,
            target_view,
        })
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
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        (target, view)
    }

    /// 渲染并回读（同步）。
    #[must_use]
    pub fn render_to_pixels(&mut self, frame: &Frame) -> Option<OffscreenFrame> {
        self.render(frame);
        let (w, h) = (self.viewport.width(), self.viewport.height());
        let bpr = (w * 4) as u64;
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
        let rgba = readback
            .get_mapped_range(0..size)
            .ok()
            .map(|v| v.to_vec())?;
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
        self.core.render_view(
            frame,
            &self.target_view,
            self.viewport.width(),
            self.viewport.height(),
        );
    }

    fn create_mesh(&mut self, desc: &MeshDesc<'_>) -> Result<MeshId, BackendError> {
        self.core.upload_mesh(desc)
    }

    fn create_material(&mut self, base_color: [f32; 4]) -> Result<MaterialId, BackendError> {
        self.core.upload_material(base_color)
    }
}
