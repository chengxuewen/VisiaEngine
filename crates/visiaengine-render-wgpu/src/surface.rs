//! attach 侧 swapchain 封装（I3/CAPI-06；web Canvas 复用同型，批 7 J1）。
//! 契约：surface 与 MeshCore 的 device/adapter 同后端配对；Outdated/Lost
//! 重配=帧内消化（Filament 跳帧语义的 wgpu 对应物，D8 触发器①已兑现）。

use visiaengine_render::{BackendError, Frame};

use crate::mesh_core::MeshCore;

/// 窗口交换链状态（headless 后端不持有本件；attach 后置入引擎目标态）。
pub struct Swapchain {
    surface: wgpu::Surface<'static>,
    pub width: u32,
    pub height: u32,
    configured: bool,
    chosen: Option<wgpu::TextureFormat>,
}

/// 一帧交换结果。
#[derive(PartialEq, Eq, Debug)]
pub enum SwapOutcome {
    /// 已 present
    Presented,
    /// vsync 背压/遮挡：本帧丢弃不计错（宿主可连发）
    Skipped,
    /// 尺寸/能力过期，configure 后下一帧恢复
    Reconfigured,
}

impl Swapchain {
    /// 从通用 target（Canvas/OffscreenCanvas/native 包装）建面。
    ///
    /// # Errors
    /// 后端不支持该 target 时返回错误。
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

    /// 裸窗口句柄建面（CAPI-06 attach；x11 必带 display，win32 hinstance 可缺）。
    ///
    /// # Safety
    /// `display`/`window` 原始句柄必须存活至本 `Swapchain` 析构（宿主义务，
    /// CAPI-06 合同条款），且为对应 kind 的合法平台句柄。
    pub unsafe fn from_raw_handles(
        core: &MeshCore,
        display: Option<raw_window_handle::RawDisplayHandle>,
        window: raw_window_handle::RawWindowHandle,
        width: u32,
        height: u32,
    ) -> Result<Self, BackendError> {
        // SAFETY: 上界义务由调用方（capi ffi attach）以宿主合同校验承担
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

    /// 配置/重配（尺寸变更、Outdated 恢复共用）。
    ///
    /// # Errors
    /// 无可支持格式/尺寸非法时返回错误。
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
        // 格式对齐管线（mesh pipeline 固定 Rgba8Unorm——default config 可能选
        // Bgra* 变体致 render pass 验证炸；I3 smoke-x11 实锤）。capabilities
        // 不含 Rgba 时退回 default 并由 configure 前的兼容性检查报错。
        // 管线按格式惰性建（Bgra* 亦合法）：Rgba8Unorm 优先保 golden 同形
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

    /// 设新尺寸（viewport 通知路径；下帧 configure）。
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.configured = false; // 强制 configure
        self.chosen = None;
    }

    /// 一帧：取像→渲→present。Timeout/Occluded=Skipped（非错误）。
    ///
    /// # Errors
    /// 未配置/设备错误传播。
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
            // Outdated/Lost：重配即恢复（帧内消化；示例同款分派）
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
