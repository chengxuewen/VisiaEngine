//! L1 无头后端：MeshCore + 离屏 target + 回读（golden 测试消费面）。

use visiaengine_render::{
    BackendError, Capability, Frame, InstanceDesc, InstanceId, MaterialDesc, MaterialId, MeshDesc,
    MeshId, PointTableDesc, RenderBackend, StrokeTableDesc, TableId, TextureDesc, TextureId,
    Viewport,
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

    /// async 构造（web/批 7 J1；native 等价 `new`）。
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

    /// Canvas attach（wasm32+web feature；swapchain 与设备同 instance）。
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
        // 行对齐（COPY_BYTES_PER_ROW_ALIGNMENT）：任意 w 合法（capi 宿主尺寸不可控——
        // 原实现仅 64px 倍数宽度可用，golden 域恰好躲过，I2 出图链实锤修复）
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
        // 去 padding：按行裁回 packed（对外 OffscreenFrame 契约不变，消费方零感知）
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

    fn create_points(&mut self, desc: &PointTableDesc<'_>) -> Result<TableId, BackendError> {
        self.core.create_points(desc)
    }
}

impl HeadlessBackend {
    /// attach 面（CAPI-06）：裸句柄建 swapchain（配置成功才返回；失败不动现目标）。
    ///
    /// # Safety
    /// 句柄存活义务见 [`Swapchain::from_raw_handles`]（capi attach 合同承接）。
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

    /// 窗口帧渲染（swapchain 路径；headless target 不参与）。
    pub fn render_swapchain(
        &mut self,
        frame: &visiaengine_render::Frame,
        sw: &mut crate::surface::Swapchain,
    ) -> Result<crate::surface::SwapOutcome, BackendError> {
        sw.render(&mut self.core, frame)
    }
}
