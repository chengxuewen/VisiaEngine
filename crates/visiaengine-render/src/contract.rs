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

/// 扩片表句柄（REND-30，4de；与 InstanceId 同计数域，命名表达新资源族 [裁决点 c]）。
pub type TableId = u64;

/// 线段（WGPU-17 扩片源）。布局=vec4 同宽字段消 CPU/GPU 对齐歧义 [4de R1]：
/// `a[0..16] b[16..32] color[32..44] width_px[44..48]`，WGSL struct 同形 48B。
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct StrokeSeg {
    pub a: [f32; 4],
    pub b: [f32; 4],
    pub color: [f32; 3],
    pub width_px: f32,
}

impl StrokeSeg {
    #[must_use]
    pub const fn new(a: [f32; 3], b: [f32; 3], color: [f32; 3], width_px: f32) -> Self {
        Self {
            a: [a[0], a[1], a[2], 0.0],
            b: [b[0], b[1], b[2], 0.0],
            color,
            width_px,
        }
    }
}

/// 标签字形 quad（REND-33，S2）：**64B=4×vec4 布局三重锁**（StrokeSeg 同制）。
/// pos/metrics 均 entity-local；uv 归一于 io-text atlas。色=**线性域 GPU 值**
/// （sRGB→线性转换住生产者面，CORE-16 咽喉扩展=label 表 [CAPI-22 面]）。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LabelMark {
    /// (xyz entity-local 锚点, w=pad)。
    pub pos: [f32; 4],
    /// 线性 RGBA。
    pub color: [f32; 4],
    /// atlas UV (u0, v0, u1, v1)。
    pub uv: [f32; 4],
    /// (w_px, h_px, dx 右正, dy 上正)：字形屏幕贴片相对锚点左上。
    pub metrics: [f32; 4],
}

impl LabelMark {
    /// 组装器（字段分组的显式形；布局锁测试钉偏移）。
    #[must_use]
    pub const fn new(pos: [f32; 3], color: [f32; 4], uv: [f32; 4], metrics: [f32; 4]) -> Self {
        Self {
            pos: [pos[0], pos[1], pos[2], 0.0],
            color,
            uv,
            metrics,
        }
    }
}

/// 标签表上传描述。
#[derive(Clone, Copy, Debug)]
pub struct LabelTableDesc<'a> {
    /// 字形 quad 序列（layout 产物拼接，多标签=同表续排）。
    pub data: &'a [LabelMark],
}

/// 线段表上传描述。
#[derive(Clone, Copy, Debug)]
pub struct StrokeTableDesc<'a> {
    pub data: &'a [StrokeSeg],
}

/// 点标记（WGPU-18 splat 源；32B 同 Instance 族形态）。
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct PointMark {
    pub pos: [f32; 3],
    pub radius_px: f32,
    pub color: [f32; 3],
    _pad: f32,
}

impl PointMark {
    #[must_use]
    pub const fn new(pos: [f32; 3], color: [f32; 3], radius_px: f32) -> Self {
        Self {
            pos,
            radius_px,
            color,
            _pad: 0.0,
        }
    }
}

/// 点表上传描述。
#[derive(Clone, Copy, Debug)]
pub struct PointTableDesc<'a> {
    pub data: &'a [PointMark],
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
    /// 线段批量扩片（REND-30，4de）：色/宽住表，材质不挂 [裁决点 a]。
    DrawStrokes {
        table: TableId,
        origin: [f64; 3],
        transform: [[f64; 4]; 4],
    },
    /// 点标记批量 splat（REND-30）：圆 mask 于 FS，色/半径住表。
    DrawPoints {
        table: TableId,
        origin: [f64; 3],
        transform: [[f64; 4]; 4],
    },
    /// 文字标签批量贴片（REND-33，S2）：世界锚定+屏幕恒大小；色/atlas 坐标住表，
    /// 材质不挂（扩片族裁决 a 同谱）；atlas 纹理由后端单例通道持有（WGPU-24）。
    DrawLabels {
        table: TableId,
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
            Self::DrawStrokes { .. } => "draw-strokes",
            Self::DrawPoints { .. } => "draw-points",
            Self::DrawLabels { .. } => "draw-labels",
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
    /// 1 屏幕像素 ≙ 世界单位数 @参考深度（REND-29，宿主给；扩片族宽度乘子。
    /// ortho 顶视=2·zoom/width 精确；透视=近似 [4de 不装①]。平铺三角系不读=零影响）。
    pub px_world_scale: f32,
    /// 方向光阴影（REND-31）：None=关闭且**逐位零回归**（后端 dummy 早退位）。
    pub shadow: Option<ShadowSetup>,
    /// 剖面裁切（REND-32）：None/`EMPTY`=全保留且**逐位零回归**（后端 count=0 恒绑早退）。
    pub clip: Option<ClipSetup>,
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

    /// 线段表上传（REND-30，4de）：默认=不支持（族协议）。
    fn create_strokes(&mut self, _desc: &StrokeTableDesc<'_>) -> Result<TableId, BackendError> {
        Err(BackendError {
            reason: "strokes unsupported by this backend".into(),
        })
    }

    /// 点表上传（REND-30）：默认=不支持（族协议）。
    fn create_points(&mut self, _desc: &PointTableDesc<'_>) -> Result<TableId, BackendError> {
        Err(BackendError {
            reason: "points unsupported by this backend".into(),
        })
    }

    /// 标签表上传（族协议第 6 员，REND-33）：默认体=显式 Err（静默跳过封堵）。
    fn create_labels(&mut self, _desc: &LabelTableDesc<'_>) -> Result<TableId, BackendError> {
        Err(BackendError {
            reason: "labels unsupported by this backend".into(),
        })
    }
}

/// 阴影深度偏置（[0,1] 约定；负值=向光拉离，防自影 acne）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowBias {
    pub constant: f32,
    pub slope: f32,
}

/// 方向光阴影配置（REND-31，4f）。**构造契约**：`proj/view_rot/eye` 只许经
/// `CameraRig::perspective/ortho_frame` 产出（PIT-5 教训升契约——GL [-1,1] 域手工矩阵
/// 进管线=静默整帧裁剪/全影，症状级死锁在 WGPU-19 像素面）；eye 走 f64 D7 同 Frame。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowSetup {
    pub proj: [[f32; 4]; 4],
    pub view_rot: [[f32; 4]; 4],
    pub eye: [f64; 3],
    /// 归一化光向（fs Lambert 与 shadow 数学同源；LIGHT shader 常数退役入 UBO）
    pub light_dir: [f32; 3],
    /// 光源角尺寸（世界单位；PCSS 半影源，0=硬 PCF）
    pub size: f32,
    pub bias: ShadowBias,
}

impl ShadowSetup {
    /// 调参基线（P2 实测定数后若变，此常数与条款体同步改）。
    pub const DEFAULT_BIAS: ShadowBias = ShadowBias {
        constant: -1.2,
        slope: -1.5,
    };
}

/// 剖面裁切配置（REND-32，B2 带）。**语义**：平面系数 `[nx,ny,nz,d]` 世界系 f64，
/// 法向指向**保留侧**，保留判据 `dot(n,P)+d ≥ 0`（面上=保留，闭区间）；多面 **AND**
/// （任一负侧即裁）。构造契约：`new` 归一化法向并拒零法向/非有限/>`MAX_PLANES`。
/// 精度纪律：世界系数持 f64（园区 3857 量级下 1mm 精度需 f64），shader 侧经
/// [`clip_to_local`] 降 f32（origin f64 相减后才降，同 D7/rebase 路径）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipSetup {
    /// 归一化平面系数 [nx,ny,nz,d]（前 `count` 个有效）。
    pub planes: [[f64; 4]; Self::MAX_PLANES],
    /// 生效面数 0..=4。
    pub count: usize,
}

impl ClipSetup {
    /// 面上限（§1b 裁决；六面盒=二带挂账，升 6 仅此一处）。
    pub const MAX_PLANES: usize = 4;
    /// 空裁切（全保留）。等价 `Frame.clip=None` 语义位。
    pub const EMPTY: Self = Self {
        planes: [[0.0; 4]; Self::MAX_PLANES],
        count: 0,
    };

    /// 从世界系数构造：归一化 + 退化拒收（零法向/非有限/超上限 → None）。
    #[must_use]
    pub fn new(planes: &[[f64; 4]]) -> Option<Self> {
        if planes.len() > Self::MAX_PLANES {
            return None;
        }
        let mut out = Self {
            planes: [[0.0; 4]; Self::MAX_PLANES],
            count: 0,
        };
        for (slot, src) in out.planes.iter_mut().zip(planes) {
            let [nx, ny, nz, d] = *src;
            if !(nx.is_finite() && ny.is_finite() && nz.is_finite() && d.is_finite()) {
                return None;
            }
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len == 0.0 {
                return None;
            }
            *slot = [nx / len, ny / len, nz / len, d / len];
        }
        out.count = planes.len();
        Some(out)
    }

    /// CPU 谓词（测试/拾取用）：点 P（世界系）是否保留。
    #[must_use]
    pub fn keeps(&self, p: [f64; 3]) -> bool {
        self.planes[..self.count]
            .iter()
            .all(|pl| pl[0] * p[0] + pl[1] * p[1] + pl[2] * p[2] + pl[3] >= 0.0)
    }
}

/// 世界平面 → **model-space** f32 系数（REND-32 精度纪律面）。shader 判定点=原始顶点 q
/// （compose_mvp 列主序列向量约定下 world = M·q + origin），保留条件恒等变形：
/// `dot(n, Mq+o)+d = dot(Mᵀn, q) + (dot(n, o+t_M)+d)`——法向经线性部 Mᵀ 推正、
/// d 并入 origin+M 平移列（t_world=origin+t_M）。
/// `d_local = dot(n,origin)+d` 与 `n_m` 全程 f64 相乘后才降 f32（D7/rebase 同款），
/// 面恰过 origin 时 local d **精确** =0（远坐标 f32 抖动条款级杜绝）。
/// 非均匀缩放下 n_m 未归一：不等式两侧同乘正标量语义不变，**无除法故无翻转风险**（det<0 翻转=拒，宿主勿负缩）。
#[must_use]
pub fn clip_to_local(plane: [f64; 4], origin: [f64; 3], model: &[[f64; 4]; 4]) -> ([f32; 3], f32) {
    let [nx, ny, nz, d] = plane;
    // n_m_i = Σ_j n_j · m[j][i]（列主序 [col][row] 下即 Mᵀn）
    let mut n_m = [0.0f64; 3];
    for (j, nj) in [nx, ny, nz].iter().enumerate() {
        for i in 0..3 {
            n_m[i] += nj * model[j][i];
        }
    }
    // 平移并入：t_world = origin + M 第四列（齐次平移分量）
    let dl = d
        + nx * (origin[0] + model[3][0])
        + ny * (origin[1] + model[3][1])
        + nz * (origin[2] + model[3][2]);
    ([n_m[0] as f32, n_m[1] as f32, n_m[2] as f32], dl as f32)
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
