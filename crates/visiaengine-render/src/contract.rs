//! 渲染契约：后端 trait + 帧 IR（纯数据，零后端类型——不变式②）。
//! 相机数学在兄弟模块 `camera`（REND-10..16）。
//! 条款：docs/sdd/render.md REND-01..05。

/// 网格资源标识（不透明；真实资源表属后续片）。
pub type MeshId = u64;

/// 材质资源标识。
pub type MaterialId = u64;

/// GPU 纹理资源标识（WGPU-14）。
pub type TextureId = u64;

/// GPU 实例表资源标识（REND-27，4c）。
pub type InstanceId = u64;
/// GPU 资源创建失败面（v0 仅承载后端拒绝；OOM 等设备丢失走 callback 侧）。
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("后端资源创建失败: {reason}")]
pub struct BackendError {
    pub reason: String,
}

/// CPU 侧网格上传描述（纯借用；positions/normals 等长由调用方担保，REND-06）。
#[derive(Clone, Copy, Debug)]
pub struct MeshDesc<'a> {
    /// UV 通道 0（WGPU-14；缺省=空，上传端零填充——Flat 管线不读）。
    pub uv: &'a [[f32; 2]],
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub indices: &'a [u32],
}

/// 楼块实例（REND-27，4c；范围裁决④：translate + z 挤出，无旋转仿射）。
/// 布局钉 = WGSL `Instance` 32B 逐字节同形（[f32;3] 对齐 4 vs vec3 对齐 16 的
/// 巧合互嵌：offset 0..12 / height 12..16 / color 16..28 / pad 28..32；
/// Pod 编译期担保，异同即红——test_pod_layout_32b 双锁）。
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct Instance {
    /// origin-local 偏移（D7：世界远坐标住在 DrawCommand::origin，REND-28）
    pub offset: [f32; 3],
    /// z 挤出高度（单位盒底 z=0 对齐，≥0）
    pub height: f32,
    /// 实例色（与 material.base_color 相乘链，REND-27 裁决点 c：alpha 不实例化）
    pub color: [f32; 3],
    _pad: f32,
}

impl Instance {
    #[must_use]
    pub const fn new(offset: [f32; 3], height: f32, color: [f32; 3]) -> Self {
        Self {
            offset,
            height,
            color,
            _pad: 0.0,
        }
    }
}

/// 实例表上传描述（借用式同 MeshDesc）。
#[derive(Clone, Copy, Debug)]
pub struct InstanceDesc<'a> {
    pub data: &'a [Instance],
}
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
    ClearColor {
        rgba: [f32; 4],
    },
    DrawMesh {
        mesh: MeshId,
        material: MaterialId,
        /// layer/模型级世界 origin（D7②）；local 网格相对它。
        origin: [f64; 3],
        /// origin-local 位姿，列主序 f64（REND-18）。
        transform: [[f64; 4]; 4],
    },
    /// 实例化批量绘制（REND-28，4c）：同一 mesh×material 的 N 次实例，
    /// origin D7 语义与 DrawMesh 同款（实例 offset 为 origin-local f32）。
    DrawInstances {
        mesh: MeshId,
        material: MaterialId,
        /// 实例表句柄（create_instances 产物，REND-27）。
        instances: InstanceId,
        origin: [f64; 3],
        transform: [[f64; 4]; 4],
    },
}

impl DrawCommand {
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::ClearColor { .. } => "clear-color",
            Self::DrawMesh { .. } => "draw-mesh",
            Self::DrawInstances { .. } => "draw-instances",
        }
    }
}

/// 一帧输入（架构③ BUILD 段产物）。
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub viewport: Viewport,
    /// 投影型判别位（后端数学不消费，host/演示消费，REND-17）。
    pub camera: Camera,
    /// 纯旋转视图基（平移经 eye 走 D7 rebase 路径，REND-17）。
    pub view_rot: [[f32; 4]; 4],
    /// 世界相机位，f64（与 DrawMesh.origin f64 相减后才降 f32）。
    pub eye: [f64; 3],
    pub proj: [[f32; 4]; 4],
    pub commands: Vec<DrawCommand>,
}

/// 后端契约（object-safe：宿主持有 `Box<dyn RenderBackend>` 多态分发）。
pub trait RenderBackend {
    fn name(&self) -> &'static str;
    fn supports(&self, capability: Capability) -> bool;
    fn resize(&mut self, viewport: Viewport);
    fn render(&mut self, frame: &Frame);
    /// CPU→GPU 网格上传，返回不透明资源 id（单调不复用，REND-07）。
    fn create_mesh(&mut self, desc: &MeshDesc<'_>) -> Result<MeshId, BackendError>;
    /// 材质注册（v0=base color 纯色，PBR 后续轮，REND-08）。
    fn create_material(&mut self, base_color: [f32; 4]) -> Result<MaterialId, BackendError>;

    /// 纹理材质面（WGPU-14）：默认体转发纯色（不支持纹理的后端零改动即合规）。
    fn create_material_desc(&mut self, desc: &MaterialDesc) -> Result<MaterialId, BackendError> {
        self.create_material(desc.base_color)
    }

    /// embedded 纹理上传（GLTF-11 下游）：默认=不支持（Err 语义与 supports 面正交）。
    fn upload_texture(&mut self, _desc: &TextureDesc<'_>) -> Result<TextureId, BackendError> {
        Err(BackendError {
            reason: "texture upload unsupported by this backend".into(),
        })
    }

    /// 实例表上传（REND-27，4c）：默认=不支持（与 upload_texture 同款显式拒绝协议）。
    fn create_instances(&mut self, _desc: &InstanceDesc<'_>) -> Result<InstanceId, BackendError> {
        Err(BackendError {
            reason: "instancing unsupported by this backend".into(),
        })
    }
}

/// 材质描述（WGPU-14 管线变体键源；`specular`=mock-up [4ab①]：参与既有 Lambert
/// 亮度系数（非 GGX 高光项——真 PBR=独立轮，诚实注记 WGPU-14 条款体）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialDesc {
    pub base_color: [f32; 4],
    pub texture: Option<TextureId>,
    pub repeat: [f32; 2],
    pub specular: f32,
}

/// CPU 侧 RGBA8 纹理借用（io-gltf/geo 产出 → 后端 upload 的中间 IR）。
#[derive(Clone, Copy, Debug)]
pub struct TextureDesc<'a> {
    pub rgba: &'a [u8],
    pub width: u32,
    pub height: u32,
}
