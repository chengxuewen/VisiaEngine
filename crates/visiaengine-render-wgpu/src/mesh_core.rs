//! 网格管线核心：设备持有 + mesh/material 资源表 + `frame→任意 view` 绘制。
//! 双消费者（DRY）：`HeadlessBackend`（L1 离屏）与 `examples/`（L2 窗口）。

use std::collections::HashMap;

use visiaengine_core::srgb_to_linear;
use wgpu::util::DeviceExt as _;

use visiaengine_render::{
    BackendError, Capability, DrawCommand, Frame, InstanceDesc, MaterialDesc, MaterialId, MeshDesc,
    MeshId, PointTableDesc, StrokeTableDesc, TableId, TextureDesc, TextureId, Viewport,
};

struct GpuMesh {
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
    index_count: u32,
}

/// 无表面依赖的渲染核心；target view 由调用方每帧提供。
pub struct MeshCore {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    /// attach/surface 复用同一 instance+adapter（设备配对约束，I3）
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    meshes: HashMap<MeshId, GpuMesh>,
    materials: HashMap<MaterialId, MatGpu>,
    next_id: u64,
    /// 纹理槽位（WGPU-14）：tex id → (texture, view)；sampler 全局单例（REPEAT/linear）
    textures: HashMap<u64, (wgpu::Texture, wgpu::TextureView)>,
    next_tex: u64,
    sampler: Option<wgpu::Sampler>,
    /// 实例表（WGPU-16）：id → (storage buffer, 实例数)
    instances: HashMap<u64, GpuInstances>,
    /// 线段表（WGPU-17）：id → (storage buffer, 段数)
    strokes_t: HashMap<TableId, GpuInstances>,
    /// 点表（WGPU-18）：id → (storage buffer, 点数)
    points_t: HashMap<TableId, GpuInstances>,
    /// 共享扩片四边形（side∈±1，2 三角）——strokes/points 常设顶点源
    quad_vb: wgpu::Buffer,
    quad_ib: wgpu::Buffer,
    /// (色彩格式, 变体) → 管线（三 layout 三管线族：mega-bool 否决 [E3D:B2]；
    /// 格式维=I3 surface 遗产 [Momus-R3]；Instanced=4c/WGPU-16）
    pipelines: HashMap<(wgpu::TextureFormat, Variant), wgpu::RenderPipeline>,
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    layout_tex: wgpu::PipelineLayout,
    layout_inst: wgpu::PipelineLayout,
    layout_stroke: wgpu::PipelineLayout,
    layout_point: wgpu::PipelineLayout,
    bgl: wgpu::BindGroupLayout,
    bgl_tex: wgpu::BindGroupLayout,
    bgl_inst: wgpu::BindGroupLayout,
    bgl_stroke: wgpu::BindGroupLayout,
    bgl_point: wgpu::BindGroupLayout,
    /// 深度面缓存（WGPU-13：尺寸变更重建；pipeline 与 attachment 格式同源）
    depth_cache: Option<(u32, u32, wgpu::Texture, wgpu::TextureView)>,
    /// WGPU-19：shadow map（1024²，懒建常驻）+ dummy 1×1（None 位恒绑）+ 比较采样器
    /// + params-off UBO（enabled=0，light_dir 载默认位型=旧 LIGHT 常数逐位）
    shadow_map: Option<(wgpu::Texture, wgpu::TextureView)>,
    sh_dummy: wgpu::TextureView,
    sh_sampler: wgpu::Sampler,
    sh_off: wgpu::Buffer,
    layout_shadow: wgpu::PipelineLayout,
    layout_shadow_inst: wgpu::PipelineLayout,
    bgl_shadow: wgpu::BindGroupLayout,
    bgl_shadow_inst: wgpu::BindGroupLayout,
}

/// 材质 GPU 态：32B uniform 块 + 可选纹理 view（textured 变体键源）。
struct MatGpu {
    buf: wgpu::Buffer,
    textured: bool,
    view: Option<wgpu::TextureView>,
}

/// 管线变体（REND-27 键源；Textured×Instanced 组合不装 [四不装②]）。
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Variant {
    Flat,
    Textured,
    Instanced,
    Strokes,
    Points,
    /// WGPU-19 caster pass（无色彩目标；Mesh=DrawMesh 源，Inst=DrawInstances 源）
    ShadowMesh,
    ShadowInst,
}

/// GPU 实例表（WGPU-16）：32B/条 storage 缓冲 + 条数。
struct GpuInstances {
    buf: wgpu::Buffer,
    count: u32,
}

/// 深度管线格式（wgpu 原生 [0,1]，PIT-5 纪律的正方）
pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// 无 surface 请求 PRIMARY 适配器并建设备；失败=None（可报告语义）。
#[must_use]
pub struct Shared {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
}

/// 无 surface 请求 PRIMARY 适配器并建设备（instance/adapter 随还——surface 配对约束）；
/// 失败=None（可报告语义）。
#[must_use]
pub fn create_shared_device() -> Option<Shared> {
    pollster::block_on(create_shared_device_async())
}

/// async 主体（web 主线程禁阻塞，批 7 J1；native 走 pollster 等价壳）。
pub async fn create_shared_device_async() -> Option<Shared> {
    let instance = crate::create_instance();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            ..Default::default()
        })
        .await
        .ok()?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("visiaengine-core-dev"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        })
        .await
        .ok()?;
    Some(Shared {
        device,
        queue,
        instance,
        adapter,
    })
}

impl MeshCore {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mesh-shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../shaders/mesh.wgsl"
            ))),
        });
        // material block 32B（base_color+repeat+specular+pad，binding2 两 layout 同尺寸
        // ——flat shader 忽略后部字段=逐像素零回归的构造保证 [Momus-B1]）
        let mk_bgl = |label: &'static str, variant: Variant| {
            let mut entries = vec![
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        // View 块 256B（WGPU-21 含裁切面段；前 176B 位序不变）
                        min_binding_size: wgpu::BufferSize::new(256),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                },
            ];
            match variant {
                Variant::Flat => {}
                Variant::Textured => {
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    });
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    });
                }
                Variant::Instanced => entries.push(wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                }),
                // WGPU-18 点表 storage（32B/条下限，binding6；材质不参与同裁决点 a）
                Variant::Points => {
                    entries.remove(1);
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(32),
                        },
                        count: None,
                    });
                }
                // WGPU-19 接收端三元在 match 后统一追加（dummy 常驻=动态分支非布局分支）
                Variant::ShadowMesh | Variant::ShadowInst => unreachable!("caster bgl 独立造"),
                // WGPU-17 线段表 storage（48B/条下限）；材质 binding2 不参与 [裁决点 a]
                Variant::Strokes => {
                    entries.remove(1);
                    entries.push(wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(48),
                        },
                        count: None,
                    });
                }
            }
            if matches!(
                variant,
                Variant::Flat | Variant::Textured | Variant::Instanced
            ) {
                entries.extend_from_slice(&[
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(32),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ]);
            }
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &entries,
            })
        };
        // WGPU-19 caster bgl 对（binding0 [+5]；无色彩目标管线用）
        let mk_shadow_entries = |inst: bool| {
            let mut e = vec![wgpu::BindGroupLayoutEntry {
                binding: 0,
                // WGPU-22：fs_shadow 消费 view.planes（caster 同裁）→ +FRAGMENT 可见
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(256),
                },
                count: None,
            }];
            if inst {
                e.push(wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(32),
                    },
                    count: None,
                });
            }
            e
        };
        let bgl_shadow = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesh-bgl-shadow"),
            entries: &mk_shadow_entries(false),
        });
        let bgl_shadow_inst = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesh-bgl-shadow-inst"),
            entries: &mk_shadow_entries(true),
        });
        let bgl_flat = mk_bgl("mesh-bgl", Variant::Flat);
        let bgl_tex = mk_bgl("mesh-bgl-tex", Variant::Textured);
        let bgl_inst = mk_bgl("mesh-bgl-inst", Variant::Instanced);
        let bgl_stroke = mk_bgl("mesh-bgl-stroke", Variant::Strokes);
        let bgl_point = mk_bgl("mesh-bgl-point", Variant::Points);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl"),
            bind_group_layouts: &[Some(&bgl_flat)],
            immediate_size: 0,
        });
        let layout_tex = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-tex"),
            bind_group_layouts: &[Some(&bgl_tex)],
            immediate_size: 0,
        });
        let layout_inst = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-inst"),
            bind_group_layouts: &[Some(&bgl_inst)],
            immediate_size: 0,
        });
        let layout_stroke = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-stroke"),
            bind_group_layouts: &[Some(&bgl_stroke)],
            immediate_size: 0,
        });
        // 共享扩片四边形（[E3D:B4] GS→VS 形态的常设顶点源）
        let quad_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad-vb"),
            contents: bytemuck::cast_slice(&[-1.0f32, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0]),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let quad_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad-ib"),
            contents: bytemuck::cast_slice(&[0u32, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX,
        });
        let layout_point = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-point"),
            bind_group_layouts: &[Some(&bgl_point)],
            immediate_size: 0,
        });
        let layout_shadow = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-shadow"),
            bind_group_layouts: &[Some(&bgl_shadow)],
            immediate_size: 0,
        });
        let layout_shadow_inst = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl-shadow-inst"),
            bind_group_layouts: &[Some(&bgl_shadow_inst)],
            immediate_size: 0,
        });
        let sh_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow-cmp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            compare: Some(wgpu::CompareFunction::Less),
            ..Default::default()
        });
        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow-dummy"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let sh_dummy = dummy.create_view(&wgpu::TextureViewDescriptor::default());
        let sh_off = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shadow-off"),
            contents: bytemuck::cast_slice(&[0.5f32, 0.7, 0.4, 0.0, 0.0, 1.0, 0.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let mut pipelines = HashMap::new();
        for (variant, pl) in [
            (Variant::Flat, &layout),
            (Variant::Textured, &layout_tex),
            (Variant::Instanced, &layout_inst),
            (Variant::Strokes, &layout_stroke),
            (Variant::Points, &layout_point),
        ] {
            pipelines.insert(
                (wgpu::TextureFormat::Rgba8Unorm, variant),
                build_pipeline(
                    &device,
                    pl,
                    &shader,
                    wgpu::TextureFormat::Rgba8Unorm,
                    variant,
                ),
            );
        }
        for (variant, pl) in [
            (Variant::ShadowMesh, &layout_shadow),
            (Variant::ShadowInst, &layout_shadow_inst),
        ] {
            pipelines.insert(
                (wgpu::TextureFormat::Rgba8Unorm, variant),
                build_pipeline(
                    &device,
                    pl,
                    &shader,
                    wgpu::TextureFormat::Rgba8Unorm,
                    variant,
                ),
            );
        }
        let _ = dummy;
        Self {
            device,
            queue,
            instance,
            adapter,
            meshes: HashMap::new(),
            materials: HashMap::new(),
            instances: HashMap::new(),
            strokes_t: HashMap::new(),
            points_t: HashMap::new(),
            quad_vb,
            quad_ib,
            next_id: 0,
            textures: HashMap::new(),
            next_tex: 0,
            sampler: None,
            pipelines,
            shader,
            layout,
            layout_tex,
            layout_inst,
            layout_stroke,
            layout_point,
            bgl: bgl_flat,
            bgl_tex,
            bgl_inst,
            bgl_stroke,
            bgl_point,
            depth_cache: None,
            shadow_map: None,
            sh_dummy,
            sh_sampler,
            sh_off,
            layout_shadow,
            layout_shadow_inst,
            bgl_shadow,
            bgl_shadow_inst,
        }
    }

    pub fn uniform(&self, bytes: &[u8], label: &'static str) -> wgpu::Buffer {
        let mut padded = bytes.to_vec();
        padded.resize(padded.len().div_ceil(256) * 256, 0);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: padded.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, &padded);
        buf
    }

    pub fn upload_mesh(&mut self, desc: &MeshDesc<'_>) -> Result<MeshId, BackendError> {
        if desc.positions.len() != desc.normals.len()
            || desc.indices.is_empty()
            || (!desc.uv.is_empty() && desc.uv.len() != desc.positions.len())
        {
            return Err(BackendError {
                reason: "MeshDesc length mismatch or empty indices".into(),
            });
        }
        let mut interleaved: Vec<f32> = Vec::with_capacity(desc.positions.len() * 8);
        for (idx, (p, n)) in desc.positions.iter().zip(desc.normals.iter()).enumerate() {
            interleaved.extend_from_slice(p);
            interleaved.extend_from_slice(n);
            let uv = desc.uv.get(idx).copied().unwrap_or([0.0, 0.0]);
            interleaved.extend_from_slice(&uv);
        }
        let vbuf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mesh-vb"),
            size: (interleaved.len() * 4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue
            .write_buffer(&vbuf, 0, bytemuck::cast_slice(&interleaved));
        let ibuf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mesh-ib"),
            size: (desc.indices.len() * 4) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue
            .write_buffer(&ibuf, 0, bytemuck::cast_slice(desc.indices));
        self.next_id += 1;
        let id = self.next_id;
        self.meshes.insert(
            id,
            GpuMesh {
                vbuf,
                ibuf,
                index_count: desc.indices.len() as u32,
            },
        );
        Ok(id)
    }

    pub fn upload_material(&mut self, base_color: [f32; 4]) -> Result<MaterialId, BackendError> {
        self.upload_material_desc(&MaterialDesc {
            base_color,
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
        })
    }

    /// 材质描述版（WGPU-14）：32B uniform 块 [base_color, repeat, specular, pad]；
    /// texture=Some → textured 变体（引用未知纹理=Err，不静默降级）。
    pub fn upload_material_desc(
        &mut self,
        desc: &MaterialDesc,
    ) -> Result<MaterialId, BackendError> {
        let (textured, view) = match desc.texture {
            Some(tid) => match self.textures.get(&tid) {
                Some((_, view)) => (true, Some(view.clone())),
                None => {
                    return Err(BackendError {
                        reason: "material references unknown texture".into(),
                    });
                }
            },
            None => (false, None),
        };
        // CORE-16 咽喉一：base_color 宿主面=sRGB（CSS 惯例），写出=线性（文档串注同批）
        let lin = srgb_to_linear(desc.base_color);
        let block = [
            lin[0],
            lin[1],
            lin[2],
            lin[3],
            desc.repeat[0],
            desc.repeat[1],
            desc.specular,
            0.0,
        ];
        let buf = self.uniform(bytemuck::cast_slice(&block), "material");
        self.next_id += 1;
        let id = self.next_id;
        self.materials.insert(
            id,
            MatGpu {
                buf,
                textured,
                view,
            },
        );
        Ok(id)
    }

    /// RGBA8 → GPU 纹理（WGPU-14）：行距补零至 256 对齐（write_texture 约束）。
    pub fn upload_texture(&mut self, desc: &TextureDesc<'_>) -> Result<TextureId, BackendError> {
        if desc.width == 0 || desc.height == 0 {
            return Err(BackendError {
                reason: "zero-size texture".into(),
            });
        }
        let need = desc.width as usize * desc.height as usize * 4;
        if desc.rgba.len() < need {
            return Err(BackendError {
                reason: "texture data shorter than width*height*4".into(),
            });
        }
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("diffuse"),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // CORE-16：宿主纹理字节=sRGB 编码（PNG/JPEG/位图惯例），Srgb 格式
            // =采样时硬件解码进线性光照域（与材质块/clear 同域，K3 片）。
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let row = desc.width as usize * 4;
        let stride = row.next_multiple_of(256);
        let mut data = Vec::with_capacity(stride * desc.height as usize);
        if stride == row {
            data.extend_from_slice(&desc.rgba[..need]);
        } else {
            for y in 0..desc.height as usize {
                data.extend_from_slice(&desc.rgba[y * row..(y + 1) * row]);
                data.resize(data.len() + (stride - row), 0);
            }
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride as u32),
                rows_per_image: Some(desc.height),
            },
            wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth_or_array_layers: 1,
            },
        );
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.next_tex += 1;
        let id = self.next_tex;
        self.textures.insert(id, (tex, view));
        Ok(id)
    }

    /// 实例表上传（WGPU-16）：32B/条 storage 表；空表建期即拒（REND-27 闭合裁决）。
    pub fn create_instances(&mut self, desc: &InstanceDesc<'_>) -> Result<u64, BackendError> {
        if desc.data.is_empty() {
            return Err(BackendError {
                reason: "empty instance table".into(),
            });
        }
        let mut data = desc.data.to_vec(); // CORE-16 咽喉五：实例色线性化副本（链乘同域）
        for inst in &mut data {
            inst.color = srgb_to_linear([inst.color[0], inst.color[1], inst.color[2], 1.0])[..3]
                .try_into()
                .expect("3ch");
        }
        let bytes = bytemuck::cast_slice(&data);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instances"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, bytes);
        self.next_id += 1;
        let id = self.next_id;
        self.instances.insert(
            id,
            GpuInstances {
                buf,
                count: desc.data.len() as u32,
            },
        );
        Ok(id)
    }

    /// View uniform 块 128B（REND-29 兑现；[f32;32] 与 WGSL struct 同布局）：
    /// mat@0..16 / right@16 / up@20 / px_scale@23 / eye_local@24..27（世界 eye−origin f32）。
    /// right/up=view_rot 行抽取——与 screen_to_ray_* 同一 view_rot 单一事实源。
    fn view_block(frame: &Frame, origin: &[f64; 3], transform: &[[f64; 4]; 4]) -> [f32; 64] {
        let mvp = visiaengine_render::rebase::compose_mvp(
            &frame.proj,
            &frame.view_rot,
            &frame.eye,
            origin,
            transform,
        );
        let mut block = [0.0f32; 64];
        block[..16].copy_from_slice(bytemuck::cast_slice(&mvp));
        let r = &frame.view_rot;
        // 列主序 m[col][row]：世界 right=行0=(m00,m10,m20)；up=行1
        block[16] = r[0][0];
        block[17] = r[1][0];
        block[18] = r[2][0];
        block[20] = r[0][1];
        block[21] = r[1][1];
        block[22] = r[2][1];
        block[23] = frame.px_world_scale;
        block[24] = (frame.eye[0] - origin[0]) as f32;
        block[25] = (frame.eye[1] - origin[1]) as f32;
        block[26] = (frame.eye[2] - origin[2]) as f32;
        if let Some(s) = &frame.shadow {
            let lvp = visiaengine_render::rebase::compose_mvp(
                &s.proj,
                &s.view_rot,
                &s.eye,
                origin,
                transform,
            );
            block[28..44].copy_from_slice(bytemuck::cast_slice(&lvp));
        }
        // WGPU-21：面系数 per-draw 换算（origin+transform 就在本函数签名里——
        // 无新 binding/绑组构造的设计红利）；None/EMPTY=全零=shader 早退恒通。
        if let Some(cs) = &frame.clip {
            for (i, pl) in cs.planes[..cs.count].iter().enumerate() {
                let (nm, dl) = visiaengine_render::clip_to_local(*pl, *origin, transform);
                let o = 44 + i * 4;
                block[o] = nm[0];
                block[o + 1] = nm[1];
                block[o + 2] = nm[2];
                block[o + 3] = dl;
            }
            block[60] = cs.count as f32;
        }
        block
    }

    /// [4f §2.4] GL 后端降级门（wgpu-30 中 GLES=Gl 子版本，单变体覆盖）：
    /// 到达即 warn-once + 全链路按 None 处理（dummy/off 常驻位，不 panic 不黑屏）。
    fn shadow_active(&self, frame: &Frame) -> bool {
        if frame.shadow.is_none() {
            return false;
        }
        if matches!(self.adapter.get_info().backend, wgpu::Backend::Gl) {
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| {
                eprintln!("warn: shadow unsupported on GL backend, disabled [4f 降级]");
            });
            return false;
        }
        true
    }

    /// 帧共享 shadow 资源（非激活=dummy/off 常驻位）：返回 (params buf, map view)。
    fn shadow_frame_res(&mut self, frame: &Frame) -> (wgpu::Buffer, wgpu::TextureView) {
        let Some(s) = frame.shadow.filter(|_| self.shadow_active(frame)) else {
            return (self.sh_off.clone(), self.sh_dummy.clone());
        };
        const MAP: u32 = 1024;
        if self.shadow_map.is_none() {
            let tex = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("shadow-map"),
                size: wgpu::Extent3d {
                    width: MAP,
                    height: MAP,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            self.shadow_map = Some((tex, view));
        }
        let mview = self.shadow_map.as_ref().expect("just built").1.clone();
        let params = [
            s.light_dir[0],
            s.light_dir[1],
            s.light_dir[2],
            1.0, // enabled
            s.size,
            1.0 / MAP as f32,
            s.bias.constant,
            s.bias.slope,
        ];
        let buf = self.uniform(bytemuck::cast_slice(&params), "shadow-params");
        (buf, mview)
    }

    /// 线段表上传（WGPU-17）：48B/条；空表建期拒（4c 同纪律）。
    pub fn create_strokes(&mut self, desc: &StrokeTableDesc<'_>) -> Result<TableId, BackendError> {
        if desc.data.is_empty() {
            return Err(BackendError {
                reason: "empty stroke table".into(),
            });
        }
        // CORE-16 咽喉三：扩片族色=线性化副本入表（Pod 契约不变，消费面=色域注记）
        let mut data = desc.data.to_vec();
        for seg in &mut data {
            seg.color = srgb_to_linear([seg.color[0], seg.color[1], seg.color[2], 1.0])[..3]
                .try_into()
                .expect("3ch");
        }
        let bytes = bytemuck::cast_slice(&data);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("strokes"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, bytes);
        self.next_id += 1;
        let id = self.next_id;
        self.strokes_t.insert(
            id,
            GpuInstances {
                buf,
                count: desc.data.len() as u32,
            },
        );
        Ok(id)
    }

    /// 点表上传（WGPU-18）：32B/条；空表建期拒（族纪律）。
    pub fn create_points(&mut self, desc: &PointTableDesc<'_>) -> Result<TableId, BackendError> {
        if desc.data.is_empty() {
            return Err(BackendError {
                reason: "empty point table".into(),
            });
        }
        let mut data = desc.data.to_vec(); // CORE-16 咽喉三同型（点色线性化副本）
        for m in &mut data {
            m.color = srgb_to_linear([m.color[0], m.color[1], m.color[2], 1.0])[..3]
                .try_into()
                .expect("3ch");
        }
        let bytes = bytemuck::cast_slice(&data);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("points"),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, bytes);
        self.next_id += 1;
        let id = self.next_id;
        self.points_t.insert(
            id,
            GpuInstances {
                buf,
                count: desc.data.len() as u32,
            },
        );
        Ok(id)
    }

    fn ensure_sampler(&mut self) {
        if self.sampler.is_none() {
            self.sampler = Some(self.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("diffuse-samp"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }));
        }
    }

    #[must_use]
    pub fn supports(capability: Capability) -> bool {
        matches!(capability, Capability::Mesh3D | Capability::OrthoCamera)
    }

    fn ensure_depth(&mut self, width: u32, height: u32) {
        if matches!(&self.depth_cache, Some((w, h, _, _)) if *w == width && *h == height) {
            return;
        }
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.depth_cache = Some((width, height, tex, view));
    }

    /// 一帧渲染到任意 view（清屏色取 commands 首个 ClearColor；DrawMesh 全量重绘，v0 无剔除）。
    /// 尺寸驱动内部深度面（WGPU-13：遮挡正确性——近物遮远物不依赖 draw 顺序）。
    pub fn render_view(
        &mut self,
        frame: &Frame,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) {
        self.render_view_format(frame, view, width, height, wgpu::TextureFormat::Rgba8Unorm);
    }

    /// 色彩格式版（surface 路径与 headless 固定格式分流，I3 格式对齐）。
    pub fn render_view_format(
        &mut self,
        frame: &Frame,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        color_format: wgpu::TextureFormat,
    ) {
        let _unused: Option<Viewport> = None;
        let clear = frame
            .commands
            .iter()
            .find_map(|c| match c {
                DrawCommand::ClearColor { rgba } => Some(*rgba),
                DrawCommand::DrawMesh { .. }
                | DrawCommand::DrawInstances { .. }
                | DrawCommand::DrawStrokes { .. }
                | DrawCommand::DrawPoints { .. }
                | DrawCommand::DrawLabels { .. } => None,
            })
            .unwrap_or([0.0; 4]);
        // CORE-16 咽喉二：ClearColor 宿主面=sRGB；srgb 目标 load 按线性解释
        // （探针实锤 0.05 直载→63 亮），先转线性保 CSS 原感字节形
        let cl = srgb_to_linear(clear);
        self.ensure_depth(width.max(1), height.max(1));
        let depth_view = self
            .depth_cache
            .as_ref()
            .map(|(_, _, _, v)| v.clone())
            .expect("depth just ensured");
        let (sh_params, sh_view) = self.shadow_frame_res(frame);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        // ===== WGPU-19 caster pre-pass（shadow=Some 才跑；DrawMesh/DrawInstances 投影，
        // 扩片族不投 [不装②]；无色彩目标 pass，bias 住 caster 管线 depth_stencil）=====
        if frame.shadow.is_some() && self.shadow_active(frame) {
            let mut sp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow-pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &sh_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            for cmd in &frame.commands {
                let (mesh, origin, transform, inst_id) = match *cmd {
                    DrawCommand::DrawMesh {
                        mesh,
                        origin,
                        transform,
                        ..
                    } => (mesh, origin, transform, None),
                    DrawCommand::DrawInstances {
                        mesh,
                        origin,
                        transform,
                        instances,
                        ..
                    } => (mesh, origin, transform, Some(instances)),
                    _ => continue,
                };
                let vblock = self.uniform(
                    bytemuck::cast_slice(&Self::view_block(frame, &origin, &transform)),
                    "view-shadow",
                );
                let Some(gm) = self.meshes.get(&mesh) else {
                    continue;
                };
                let (vbuf, ibuf, index_count) = (gm.vbuf.clone(), gm.ibuf.clone(), gm.index_count);
                let variant = if inst_id.is_some() {
                    Variant::ShadowInst
                } else {
                    Variant::ShadowMesh
                };
                let pipeline = match self.pipelines.get(&(color_format, variant)) {
                    Some(p) => p.clone(),
                    None => {
                        let layout = if inst_id.is_some() {
                            &self.layout_shadow_inst
                        } else {
                            &self.layout_shadow
                        };
                        let p = build_pipeline(
                            &self.device,
                            layout,
                            &self.shader,
                            color_format,
                            variant,
                        );
                        self.pipelines.insert((color_format, variant), p.clone());
                        p
                    }
                };
                sp.set_pipeline(&pipeline);
                let mut inst_buf: Option<wgpu::Buffer> = None;
                let mut inst_count = 1u32;
                if let Some(iid) = inst_id {
                    let Some(gi) = self.instances.get(&iid) else {
                        continue;
                    };
                    inst_buf = Some(gi.buf.clone());
                    inst_count = gi.count;
                }
                let mut entries = vec![wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vblock.as_entire_binding(),
                }];
                if let Some(ib) = &inst_buf {
                    entries.push(wgpu::BindGroupEntry {
                        binding: 5,
                        resource: ib.as_entire_binding(),
                    });
                }
                let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("shadow-caster-bg"),
                    layout: if inst_id.is_some() {
                        &self.bgl_shadow_inst
                    } else {
                        &self.bgl_shadow
                    },
                    entries: &entries,
                });
                sp.set_bind_group(0, &bg, &[]);
                sp.set_vertex_buffer(0, vbuf.slice(..));
                sp.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                sp.draw_indexed(0..index_count, 0, 0..inst_count);
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mesh-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        // CORE-16 咽喉二：ClearColor 经上方 cl（srgb 目标 load=线性域，探针实锤）
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(cl[0]),
                            g: f64::from(cl[1]),
                            b: f64::from(cl[2]),
                            a: f64::from(cl[3]),
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            for cmd in &frame.commands {
                match cmd {
                    // N1 半程：扩片族臂先占位（N2/N3 实装出图）——穷举同步器锁在此，
                    // 漏臂=编译失败（契约面兑现，非静默跳过）
                    DrawCommand::ClearColor { .. } => {}
                    // REND-33 Labels：管线出图在 N3（atlas 通道+vs/fs_label）落地，
                    // 本带 N2 仅立 IR——穷举臂占位（缺 create_labels 后端=表不建，此臂空转安全）
                    DrawCommand::DrawLabels { .. } => {}
                    DrawCommand::DrawPoints {
                        table,
                        origin,
                        transform,
                    } => {
                        let mvp_buf = self.uniform(
                            bytemuck::cast_slice(&Self::view_block(frame, origin, transform)),
                            "view",
                        );
                        let Some(gi) = self.points_t.get(table) else {
                            continue; // 缺表=skip（族纪律，WGPU-18）
                        };
                        let tbuf = gi.buf.clone();
                        let count = gi.count;
                        let pipeline = match self.pipelines.get(&(color_format, Variant::Points)) {
                            Some(p) => p.clone(),
                            None => {
                                let p = build_pipeline(
                                    &self.device,
                                    &self.layout_point,
                                    &self.shader,
                                    color_format,
                                    Variant::Points,
                                );
                                self.pipelines
                                    .insert((color_format, Variant::Points), p.clone());
                                p
                            }
                        };
                        pass.set_pipeline(&pipeline);
                        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("point-bg"),
                            layout: &self.bgl_point,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: mvp_buf.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 6,
                                    resource: tbuf.as_entire_binding(),
                                },
                            ],
                        });
                        pass.set_bind_group(0, &bg, &[]);
                        pass.set_vertex_buffer(0, self.quad_vb.slice(..));
                        pass.set_index_buffer(self.quad_ib.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..6, 0, 0..count);
                    }
                    DrawCommand::DrawMesh {
                        mesh,
                        material,
                        origin,
                        transform,
                    } => {
                        let mvp_buf = self.uniform(
                            bytemuck::cast_slice(&Self::view_block(frame, origin, transform)),
                            "view",
                        );
                        let (vbuf, ibuf, index_count) = match self.meshes.get(mesh) {
                            Some(gm) => (gm.vbuf.clone(), gm.ibuf.clone(), gm.index_count),
                            None => continue,
                        };
                        let Some(mat) = self.materials.get(material) else {
                            continue;
                        };
                        let buf = mat.buf.clone();
                        let (textured, view) = (mat.textured, mat.view.clone());
                        let variant = if textured {
                            Variant::Textured
                        } else {
                            Variant::Flat
                        };
                        let pipeline = match self.pipelines.get(&(color_format, variant)) {
                            Some(p) => p.clone(),
                            None => {
                                let layout = if textured {
                                    &self.layout_tex
                                } else {
                                    &self.layout
                                };
                                let p = build_pipeline(
                                    &self.device,
                                    layout,
                                    &self.shader,
                                    color_format,
                                    variant,
                                );
                                self.pipelines.insert((color_format, variant), p.clone());
                                p
                            }
                        };
                        pass.set_pipeline(&pipeline);
                        let bg = if textured {
                            let Some(view) = view else { continue };
                            self.ensure_sampler();
                            let sampler = self.sampler.clone().expect("sampler just ensured");
                            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("mesh-bg-tex"),
                                layout: &self.bgl_tex,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: mvp_buf.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 2,
                                        resource: buf.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 3,
                                        resource: wgpu::BindingResource::TextureView(&view),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 4,
                                        resource: wgpu::BindingResource::Sampler(&sampler),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: sh_params.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 7,
                                        resource: wgpu::BindingResource::TextureView(&sh_view),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 8,
                                        resource: wgpu::BindingResource::Sampler(&self.sh_sampler),
                                    },
                                ],
                            })
                        } else {
                            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("mesh-bg"),
                                layout: &self.bgl,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: mvp_buf.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 2,
                                        resource: buf.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: sh_params.as_entire_binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 7,
                                        resource: wgpu::BindingResource::TextureView(&sh_view),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 8,
                                        resource: wgpu::BindingResource::Sampler(&self.sh_sampler),
                                    },
                                ],
                            })
                        };
                        pass.set_bind_group(0, &bg, &[]);
                        pass.set_vertex_buffer(0, vbuf.slice(..));
                        pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..index_count, 0, 0..1);
                    }
                    // ponytail: 两臂 ~25 行 mvp/查表/管线缓存 同形——提公共 helper 的触发
                    // =4de 第三消费者（线/点扩片）；现保持 Flat/Textured 路径逐字不动=零回归构造保证
                    DrawCommand::DrawInstances {
                        mesh,
                        material,
                        instances,
                        origin,
                        transform,
                    } => {
                        let mvp_buf = self.uniform(
                            bytemuck::cast_slice(&Self::view_block(frame, origin, transform)),
                            "view",
                        );
                        let (vbuf, ibuf, index_count) = match self.meshes.get(mesh) {
                            Some(gm) => (gm.vbuf.clone(), gm.ibuf.clone(), gm.index_count),
                            None => continue,
                        };
                        let Some(gi) = self.instances.get(instances) else {
                            continue; // 缺表=skip（mesh 缺失同纪律，WGPU-16）
                        };
                        let inst_buf = gi.buf.clone();
                        let count = gi.count;
                        let Some(mat) = self.materials.get(material) else {
                            continue;
                        };
                        let buf = mat.buf.clone();
                        let pipeline = match self.pipelines.get(&(color_format, Variant::Instanced))
                        {
                            Some(p) => p.clone(),
                            None => {
                                let p = build_pipeline(
                                    &self.device,
                                    &self.layout_inst,
                                    &self.shader,
                                    color_format,
                                    Variant::Instanced,
                                );
                                self.pipelines
                                    .insert((color_format, Variant::Instanced), p.clone());
                                p
                            }
                        };
                        pass.set_pipeline(&pipeline);
                        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("mesh-bg-inst"),
                            layout: &self.bgl_inst,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: mvp_buf.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: buf.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 5,
                                    resource: inst_buf.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: sh_params.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 7,
                                    resource: wgpu::BindingResource::TextureView(&sh_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 8,
                                    resource: wgpu::BindingResource::Sampler(&self.sh_sampler),
                                },
                            ],
                        });
                        pass.set_bind_group(0, &bg, &[]);
                        pass.set_vertex_buffer(0, vbuf.slice(..));
                        pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..index_count, 0, 0..count);
                    }
                    DrawCommand::DrawStrokes {
                        table,
                        origin,
                        transform,
                    } => {
                        let mvp_buf = self.uniform(
                            bytemuck::cast_slice(&Self::view_block(frame, origin, transform)),
                            "view",
                        );
                        let Some(gi) = self.strokes_t.get(table) else {
                            continue; // 缺表=skip（族纪律，WGPU-17）
                        };
                        let tbuf = gi.buf.clone();
                        let count = gi.count;
                        let pipeline = match self.pipelines.get(&(color_format, Variant::Strokes)) {
                            Some(p) => p.clone(),
                            None => {
                                let p = build_pipeline(
                                    &self.device,
                                    &self.layout_stroke,
                                    &self.shader,
                                    color_format,
                                    Variant::Strokes,
                                );
                                self.pipelines
                                    .insert((color_format, Variant::Strokes), p.clone());
                                p
                            }
                        };
                        pass.set_pipeline(&pipeline);
                        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("stroke-bg"),
                            layout: &self.bgl_stroke,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: mvp_buf.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 5,
                                    resource: tbuf.as_entire_binding(),
                                },
                            ],
                        });
                        pass.set_bind_group(0, &bg, &[]);
                        pass.set_vertex_buffer(0, self.quad_vb.slice(..));
                        pass.set_index_buffer(self.quad_ib.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..6, 0, 0..count);
                    }
                }
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

/// 按目标色彩格式建管线（`Rgba8Unorm` 与 `Bgra8Unorm*` 系由 surface caps 决定）。
#[must_use]
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn build_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    color_format: wgpu::TextureFormat,
    variant: Variant,
) -> wgpu::RenderPipeline {
    let stroke_attrs = wgpu::vertex_attr_array![0 => Float32x2];
    let mesh_attrs = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];
    let buffers: [Option<wgpu::VertexBufferLayout>; 1] =
        if matches!(variant, Variant::Strokes | Variant::Points) {
            [Some(wgpu::VertexBufferLayout {
                array_stride: (2 * std::mem::size_of::<f32>()) as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &stroke_attrs,
            })]
        } else {
            [Some(wgpu::VertexBufferLayout {
                array_stride: (8 * std::mem::size_of::<f32>()) as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &mesh_attrs,
            })]
        };
    let cts = [Some(wgpu::ColorTargetState {
        format: color_format,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    })];
    let color_targets: &[Option<wgpu::ColorTargetState>] =
        if matches!(variant, Variant::ShadowMesh | Variant::ShadowInst) {
            &[]
        } else {
            &cts
        };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mesh-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some(match variant {
                Variant::Instanced => "vs_inst",
                Variant::Strokes => "vs_stroke",
                Variant::Points => "vs_point",
                Variant::ShadowMesh => "vs_shadow",
                Variant::ShadowInst => "vs_shadow_inst",
                _ => "vs",
            }),
            compilation_options: Default::default(),
            buffers: &buffers,
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            // polygon-offset 伴生件 [E3D:B7③]：扩片族盖面不斗同深度 z-fight
            bias: if matches!(variant, Variant::Strokes | Variant::Points) {
                wgpu::DepthBiasState {
                    constant: -1,
                    slope_scale: -1.0,
                    ..Default::default()
                }
            } else if matches!(variant, Variant::ShadowMesh | Variant::ShadowInst) {
                // caster 侧 acne 双保险（接收端另有常数偏置 [R2]；与 DEFAULT_BIAS 同步）
                wgpu::DepthBiasState {
                    constant: -1,
                    slope_scale: -0.5,
                    ..Default::default()
                }
            } else {
                wgpu::DepthBiasState::default()
            },
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(match variant {
                Variant::Textured => "fs_textured",
                Variant::Strokes => "fs_stroke",
                Variant::Points => "fs_point",
                Variant::ShadowMesh | Variant::ShadowInst => "fs_shadow",
                _ => "fs",
            }),
            compilation_options: Default::default(),
            targets: color_targets,
        }),
        multiview_mask: None,
        cache: None,
    })
}
