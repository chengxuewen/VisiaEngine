//! 网格管线核心：设备持有 + mesh/material 资源表 + `frame→任意 view` 绘制。
//! 双消费者（DRY）：`HeadlessBackend`（L1 离屏）与 `examples/`（L2 窗口）。

use std::collections::HashMap;

use visiaengine_render::{
    BackendError, Capability, DrawCommand, Frame, MaterialDesc, MaterialId, MeshDesc, MeshId,
    TextureDesc, TextureId, Viewport,
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
    /// (色彩格式, textured) → 管线（双 layout 双管线：mega-bool 否决 [E3D:B2]；
    /// 格式维=I3 surface 遗产，≤4 管线现实 [Momus-R3]）
    pipelines: HashMap<(wgpu::TextureFormat, bool), wgpu::RenderPipeline>,
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    layout_tex: wgpu::PipelineLayout,
    bgl: wgpu::BindGroupLayout,
    bgl_tex: wgpu::BindGroupLayout,
    /// 深度面缓存（WGPU-13：尺寸变更重建；pipeline 与 attachment 格式同源）
    depth_cache: Option<(u32, u32, wgpu::Texture, wgpu::TextureView)>,
}

/// 材质 GPU 态：32B uniform 块 + 可选纹理 view（textured 变体键源）。
struct MatGpu {
    buf: wgpu::Buffer,
    textured: bool,
    view: Option<wgpu::TextureView>,
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
        let mk_bgl = |label: &'static str, textured: bool| {
            let mut entries = vec![
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
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
            if textured {
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
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &entries,
            })
        };
        let bgl_flat = mk_bgl("mesh-bgl", false);
        let bgl_tex = mk_bgl("mesh-bgl-tex", true);
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
        let mut pipelines = HashMap::new();
        pipelines.insert(
            (wgpu::TextureFormat::Rgba8Unorm, false),
            build_pipeline(
                &device,
                &layout,
                &shader,
                wgpu::TextureFormat::Rgba8Unorm,
                false,
            ),
        );
        pipelines.insert(
            (wgpu::TextureFormat::Rgba8Unorm, true),
            build_pipeline(
                &device,
                &layout_tex,
                &shader,
                wgpu::TextureFormat::Rgba8Unorm,
                true,
            ),
        );
        Self {
            device,
            queue,
            instance,
            adapter,
            meshes: HashMap::new(),
            materials: HashMap::new(),
            next_id: 0,
            textures: HashMap::new(),
            next_tex: 0,
            sampler: None,
            pipelines,
            shader,
            layout,
            layout_tex,
            bgl: bgl_flat,
            bgl_tex,
            depth_cache: None,
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
        let block = [
            desc.base_color[0],
            desc.base_color[1],
            desc.base_color[2],
            desc.base_color[3],
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
            format: wgpu::TextureFormat::Rgba8Unorm,
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
                DrawCommand::DrawMesh { .. } => None,
            })
            .unwrap_or([0.0; 4]);
        self.ensure_depth(width.max(1), height.max(1));
        let depth_view = self
            .depth_cache
            .as_ref()
            .map(|(_, _, _, v)| v.clone())
            .expect("depth just ensured");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mesh-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(clear[0]),
                            g: f64::from(clear[1]),
                            b: f64::from(clear[2]),
                            a: f64::from(clear[3]),
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
                let DrawCommand::DrawMesh {
                    mesh,
                    material,
                    origin,
                    transform,
                } = cmd
                else {
                    continue;
                };
                let mvp = visiaengine_render::rebase::compose_mvp(
                    &frame.proj,
                    &frame.view_rot,
                    &frame.eye,
                    origin,
                    transform,
                );
                let mvp_buf = self.uniform(bytemuck::cast_slice(&mvp), "mvp");
                let (vbuf, ibuf, index_count) = match self.meshes.get(mesh) {
                    Some(gm) => (gm.vbuf.clone(), gm.ibuf.clone(), gm.index_count),
                    None => continue,
                };
                let Some(mat) = self.materials.get(material) else {
                    continue;
                };
                let buf = mat.buf.clone();
                let (textured, view) = (mat.textured, mat.view.clone());
                let pipeline = match self.pipelines.get(&(color_format, textured)) {
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
                            textured,
                        );
                        self.pipelines.insert((color_format, textured), p.clone());
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
                        ],
                    })
                };
                pass.set_bind_group(0, &bg, &[]);
                pass.set_vertex_buffer(0, vbuf.slice(..));
                pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..index_count, 0, 0..1);
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
    textured: bool,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mesh-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: (8 * std::mem::size_of::<f32>()) as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2],
            })],
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
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(if textured { "fs_textured" } else { "fs" }),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
