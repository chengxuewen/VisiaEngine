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
    meshes: HashMap<MeshId, GpuMesh>,
    materials: HashMap<MaterialId, wgpu::Buffer>,
    next_id: u64,
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
}

/// 无 surface 请求 PRIMARY 适配器并建设备；失败=None（可报告语义）。
#[must_use]
pub fn create_shared_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = crate::create_instance();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        ..Default::default()
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("visiaengine-core-dev"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .ok()?;
    Some((device, queue))
}

impl MeshCore {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
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
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesh-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
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
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            device,
            queue,
            meshes: HashMap::new(),
            materials: HashMap::new(),
            next_id: 0,
            pipeline,
            bgl,
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

    /// 一帧渲染到任意 view（清屏色取 commands 首个 ClearColor；DrawMesh 全量重绘，v0 无剔除）。
    pub fn render_view(&mut self, frame: &Frame, view: &wgpu::TextureView) {
        let _unused: Option<Viewport> = None;
        let clear = frame
            .commands
            .iter()
            .find_map(|c| match c {
                DrawCommand::ClearColor { rgba } => Some(*rgba),
                DrawCommand::DrawMesh { .. } => None,
            })
            .unwrap_or([0.0; 4]);
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
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
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
