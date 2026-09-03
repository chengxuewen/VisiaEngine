//! 无头后端：离屏目标 + mesh/material 资源表（L1 golden 消费面，G3 窗口路径共享资源层）。

use std::collections::HashMap;

use visiaengine_render::{
    BackendError, Capability, DrawCommand, Frame, MaterialId, MeshDesc, MeshId, RenderBackend,
    Viewport,
};

use crate::offscreen::OffscreenFrame;

struct GpuMesh {
    vbuf: wgpu::Buffer,
    ibuf: wgpu::Buffer,
    index_count: u32,
}

pub struct HeadlessBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    viewport: Viewport,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    meshes: HashMap<MeshId, GpuMesh>,
    materials: HashMap<MaterialId, wgpu::Buffer>,
    next_id: u64,
}

fn pick_adapter() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = crate::create_instance();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        ..Default::default()
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("visiaengine-headless"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .ok()?;
    Some((device, queue))
}

impl HeadlessBackend {
    /// 以离屏尺寸建立无头设备；无可用适配器返回 None（WGPU-02 同款可报告语义）。
    #[must_use]
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let (device, queue) = pick_adapter()?;
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
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
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
                    binding: 1,
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
        Some(Self {
            device,
            queue,
            viewport: Viewport::new(width, height, 1.0),
            target,
            target_view,
            pipeline,
            bgl,
            meshes: HashMap::new(),
            materials: HashMap::new(),
            next_id: 0,
        })
    }

    fn uniform(&self, bytes: &[u8], label: &'static str) -> wgpu::Buffer {
        let mut padded = bytes.to_vec();
        padded.resize(padded.len().div_ceil(256) * 256, 0); // uniform 最小绑定对齐宽容
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: padded.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buf, 0, &padded);
        buf
    }

    /// 渲染并回读当前离屏目标（同步：render→copy→map→poll→取数）。
    #[must_use]
    pub fn render_to_pixels(&mut self, frame: &Frame) -> Option<OffscreenFrame> {
        self.render(frame);
        let (w, h) = (self.viewport.width(), self.viewport.height());
        let bpr = (w * 4) as u64;
        let size = bpr * h as u64;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
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
        self.queue.submit(std::iter::once(encoder.finish()));
        let (sender, receiver) = std::sync::mpsc::channel();
        readback.map_async(wgpu::MapMode::Read, 0..size, move |res| {
            let _ = sender.send(res);
        });
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
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
        matches!(capability, Capability::Mesh3D | Capability::OrthoCamera)
    }

    fn resize(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.target = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen-target"),
            size: wgpu::Extent3d {
                width: viewport.width(),
                height: viewport.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.target_view = self
            .target
            .create_view(&wgpu::TextureViewDescriptor::default());
    }

    fn create_mesh(&mut self, desc: &MeshDesc<'_>) -> Result<MeshId, BackendError> {
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
            size: (interleaved.len() * 4).next_multiple_of(4) as u64,
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

    fn create_material(&mut self, base_color: [f32; 4]) -> Result<MaterialId, BackendError> {
        let buf = self.uniform(bytemuck::cast_slice(&base_color), "material");
        self.next_id += 1;
        let id = self.next_id;
        self.materials.insert(id, buf);
        Ok(id)
    }

    fn render(&mut self, frame: &Frame) {
        let clear = frame
            .commands
            .iter()
            .find_map(|c| match c {
                DrawCommand::ClearColor { rgba } => Some(*rgba),
                DrawCommand::DrawMesh { .. } => None,
            })
            .unwrap_or([0.0; 4]);
        let view_proj: [[f32; 4]; 4] = {
            let p = glam::Mat4::from_cols_array_2d(&frame.proj);
            let v = glam::Mat4::from_cols_array_2d(&frame.view);
            (p * v).to_cols_array_2d()
        };
        let vp_buf = self.uniform(bytemuck::cast_slice(&view_proj), "view-proj");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("headless-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.target_view,
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
                    transform,
                } = cmd
                else {
                    continue;
                };
                let Some(gm) = self.meshes.get(mesh) else {
                    continue;
                };
                let Some(mat) = self.materials.get(material) else {
                    continue;
                };
                let model: [[f32; 4]; 4] = {
                    let m = glam::DMat4::from_cols_array_2d(transform);
                    m.as_mat4().to_cols_array_2d()
                };
                let model_buf = self.uniform(bytemuck::cast_slice(&model), "model");
                let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("mesh-bg"),
                    layout: &self.bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: vp_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: model_buf.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: mat.as_entire_binding(),
                        },
                    ],
                });
                pass.set_bind_group(0, &bg, &[]);
                pass.set_vertex_buffer(0, gm.vbuf.slice(..));
                pass.set_index_buffer(gm.ibuf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..gm.index_count, 0, 0..1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
