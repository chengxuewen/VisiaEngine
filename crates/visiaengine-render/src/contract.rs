//! 渲染契约：后端 trait + 帧 IR（纯数据，零后端类型——不变式②）。
//! 条款：docs/sdd/render.md REND-01..05。

use visiaengine_core::Transform;

/// 网格资源标识（不透明；真实资源表属后续片）。
pub type MeshId = u64;

/// 能力位（Tier 矩阵钩子，architecture.md ⑥；v0 枚举从简）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    Mesh3D,
    LineStrip,
    OrthoCamera,
}

/// 物理视口。缩放因子用于 HiDPI（逻辑/物理尺寸分离）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    width: u32,
    height: u32,
    scale_factor: f32,
}

impl Viewport {
    /// `scale_factor` 必须 >0（调用方边界校验；0 缩放是宿主 bug）。
    #[must_use]
    pub const fn new(width: u32, height: u32, scale_factor: f32) -> Self {
        debug_assert!(scale_factor > 0.0);
        Self {
            width,
            height,
            scale_factor,
        }
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// 逻辑尺寸 = 物理 / 缩放（winit 语义）。
    #[must_use]
    pub const fn logical_size(&self) -> (f32, f32) {
        (
            self.width as f32 / self.scale_factor,
            self.height as f32 / self.scale_factor,
        )
    }
}

/// 相机投影（2D/2.5D/3D 统一入口的投影侧；插值切换属后续片）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Camera {
    Ortho {
        half_width: f32,
        half_height: f32,
        near: f32,
        far: f32,
    },
    Perspective {
        fov_y_rad: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
}

impl Camera {
    #[must_use]
    pub const fn ortho(half_width: f32, half_height: f32, near: f32, far: f32) -> Self {
        Self::Ortho {
            half_width,
            half_height,
            near,
            far,
        }
    }

    #[must_use]
    pub const fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self::Perspective {
            fov_y_rad,
            aspect,
            near,
            far,
        }
    }

    #[must_use]
    pub const fn is_orthographic(&self) -> bool {
        matches!(self, Self::Ortho { .. })
    }
}

/// 渲染指令 IR v0（新图元=新变体；消费端穷举 match）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawCommand {
    ClearColor { rgba: [f32; 4] },
    DrawMesh { mesh: MeshId, transform: Transform },
}

impl DrawCommand {
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::ClearColor { .. } => "clear-color",
            Self::DrawMesh { .. } => "draw-mesh",
        }
    }
}

/// 一帧输入（架构③ BUILD 段产物）。
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub viewport: Viewport,
    pub camera: Camera,
    pub commands: Vec<DrawCommand>,
}

/// 后端契约（object-safe：宿主持有 `Box<dyn RenderBackend>` 多态分发）。
pub trait RenderBackend {
    fn name(&self) -> &'static str;
    fn supports(&self, capability: Capability) -> bool;
    fn resize(&mut self, viewport: Viewport);
    fn render(&mut self, frame: &Frame);
}
