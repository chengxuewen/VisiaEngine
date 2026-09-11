//! 网格管线核心：设备持有 + mesh/material 资源表 + `frame→任意 view` 绘制。
//! 双消费者（DRY）：`HeadlessBackend`（L1 离屏）与 `examples/`（L2 窗口）。

use std::collections::HashMap;

use visiaengine_render::{
    BackendError, Capability, DrawCommand, Frame, MaterialId, MeshDesc, MeshId, Viewport,
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
    materials: HashMap<MaterialId, wgpu::Buffer>,
    next_id: u64,
    /// 色彩格式→管线（surface 格式多样性：Rgba/Bgra* 系，I3 格式对齐重构）
    pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
    shader: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    bgl: wgpu::BindGroupLayout,
    /// 深度面缓存（WGPU-13：尺寸变更重建；pipeline 与 attachment 格式同源）
    depth_cache: Option<(u32, u32, wgpu::Texture, wgpu::TextureView)>,
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
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesh-bgl"),
            entries: &[
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
                        min_binding_size: wgpu::BufferSize::new(16),
                    },
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pl"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let mut pipelines = HashMap::new();
        pipelines.insert(
            wgpu::TextureFormat::Rgba8Unorm,
            build_pipeline(&device, &layout, &shader, wgpu::TextureFormat::Rgba8Unorm),
        );
        Self {
            device,
            queue,
            instance,
            adapter,
            meshes: HashMap::new(),
            materials: HashMap::new(),
            next_id: 0,
            pipelines,
            shader,
            layout,
            bgl,
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
        if desc.positions.len() != desc.normals.len() || desc.indices.is_empty() {
            return Err(BackendError {
                reason: "MeshDesc length mismatch or empty indices".into(),
            });
        }
        let mut interleaved: Vec<f32> = Vec::with_capacity(desc.positions.len() * 6);
        for (p, n) in desc.positions.iter().zip(desc.normals.iter()) {
            interleaved.extend_from_slice(p);
            interleaved.extend_from_slice(n);
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
        let buf = self.uniform(bytemuck::cast_slice(&base_color), "material");
        self.next_id += 1;
        let id = self.next_id;
        self.materials.insert(id, buf);
        Ok(id)
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
            let pipeline = match self.pipelines.get(&color_format) {
                Some(p) => p.clone(),
                None => {
                    let p = build_pipeline(&self.device, &self.layout, &self.shader, color_format);
                    self.pipelines.insert(color_format, p.clone());
                    p
                }
            };
            pass.set_pipeline(&pipeline);
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
                    Some(gm) => (&gm.vbuf, &gm.ibuf, gm.index_count),
                    None => continue,
                };
                let Some(mat) = self.materials.get(material) else {
                    continue;
                };
                let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("mesh-bg"),
                    layout: &self.bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: mvp_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: mat.as_entire_binding(),
                        },
                    ],
                });
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
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mesh-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: (6 * std::mem::size_of::<f32>()) as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
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
            entry_point: Some("fs"),
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
