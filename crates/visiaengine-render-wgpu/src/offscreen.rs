//! L1 离屏渲染：无表面三角形 + 回读（golden 测试消费；CI lavapipe 必绿面）。

use wgpu::util::DeviceExt as _;

/// 一帧回读结果（RGBA8，行距由 width*4 天然 256 对齐于 640）。
pub struct OffscreenFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

const CLEAR: wgpu::Color = wgpu::Color {
    r: 0.05,
    g: 0.07,
    b: 0.10,
    a: 1.0,
};

// 覆盖中心像素的红三角（NDC）
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}

const VERTICES: [Vertex; 3] = [
    Vertex {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.0, 0.6],
        color: [1.0, 0.0, 0.0],
    },
];

/// 离屏渲染 640×480 红三角；无适配器返回 None（可报告状态）。
#[must_use]
pub fn render_offscreen_triangle() -> Option<OffscreenFrame> {
    let width: u32 = 640;
    let height: u32 = 480;
    let instance = crate::create_instance();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        ..Default::default()
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("visiaengine-offscreen"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .ok()?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("triangle"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
            "../shaders/triangle.wgsl"
        ))),
    });
    let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("verts"),
        contents: bytemuck::cast_slice(&VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("triangle"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3],
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

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("golden"),
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
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("golden"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        // v30 规范对齐：draw/set_* 状态属于 render pass 作用域
        pass.set_pipeline(&pipeline);
        pass.set_vertex_buffer(0, vbuf.slice(..));
        pass.draw(0..3, 0..1);
    }

    let bytes_per_row = (width * 4) as u64;
    let readback_size = bytes_per_row * height as u64;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: readback_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row as u32),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));

    let (sender, receiver) = std::sync::mpsc::channel();
    readback.map_async(wgpu::MapMode::Read, 0..readback_size, move |res| {
        let _ = sender.send(res);
    });
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    // v30：map_async 为回调式，poll 完成后 channel 即有 Result<Result<..>>
    receiver
        .recv()
        .ok()?
        .map_err(|e| eprintln!("map failed: {e}"))
        .ok()?;
    let data = readback
        .get_mapped_range(0..readback_size)
        .ok()
        .map(|v| v.to_vec())?;
    readback.unmap();
    Some(OffscreenFrame {
        width,
        height,
        rgba: data,
    })
}

/// 24 顶点/36 索引标准立方（每面独立法线；边长 1.0 居中原点）。
#[must_use]
pub fn cube_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        (
            [0.0, 0.0, 1.0],
            [
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
        ),
        (
            [0.0, 0.0, -1.0],
            [
                [0.5, -0.5, -0.5],
                [-0.5, -0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [0.5, 0.5, -0.5],
            ],
        ),
        (
            [1.0, 0.0, 0.0],
            [
                [0.5, -0.5, 0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [0.5, 0.5, 0.5],
            ],
        ),
        (
            [-1.0, 0.0, 0.0],
            [
                [-0.5, -0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [-0.5, 0.5, 0.5],
                [-0.5, 0.5, -0.5],
            ],
        ),
        (
            [0.0, 1.0, 0.0],
            [
                [-0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
            ],
        ),
        (
            [0.0, -1.0, 0.0],
            [
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, -0.5, 0.5],
                [-0.5, -0.5, 0.5],
            ],
        ),
    ];
    let mut pos = Vec::with_capacity(24);
    let mut nrm = Vec::with_capacity(24);
    let mut idx = Vec::with_capacity(36);
    for (n, quad) in faces {
        let base = pos.len() as u32;
        for v in quad {
            pos.push(v);
            nrm.push(n);
        }
        idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    (pos, nrm, idx)
}

/// 单位盒：xy∈[-0.5,0.5]、**z∈[0,1] 底对齐**（WGPU-16 挤出语义的几何前提），
/// 5 面 24 顶点/30 索引（底面省略——顶视/斜视永远不可见，白模压力例体积减半）。
/// tests/instances.rs 与 examples/bench_twin.rs 双消费（DRY，[E3D:A5] 教学件同区）。
#[must_use]
pub fn unit_box_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let faces: [([f32; 3], [[f32; 3]; 4]); 4] = [
        (
            [0.0, -1.0, 0.0],
            [
                [-0.5, -0.5, 1.0],
                [0.5, -0.5, 1.0],
                [0.5, -0.5, 0.0],
                [-0.5, -0.5, 0.0],
            ],
        ),
        (
            [1.0, 0.0, 0.0],
            [
                [0.5, -0.5, 1.0],
                [0.5, 0.5, 1.0],
                [0.5, 0.5, 0.0],
                [0.5, -0.5, 0.0],
            ],
        ),
        (
            [0.0, 1.0, 0.0],
            [
                [0.5, 0.5, 1.0],
                [-0.5, 0.5, 1.0],
                [-0.5, 0.5, 0.0],
                [0.5, 0.5, 0.0],
            ],
        ),
        (
            [0.0, 0.0, 1.0],
            [
                [-0.5, 0.5, 1.0],
                [0.5, 0.5, 1.0],
                [0.5, -0.5, 1.0],
                [-0.5, -0.5, 1.0],
            ],
        ),
    ];
    let mut pos = Vec::with_capacity(16);
    let mut nrm = Vec::with_capacity(16);
    let mut idx = Vec::with_capacity(24);
    for (n, quad) in faces {
        let b = idx.len() as u32 / 4 * 4;
        for v in quad {
            pos.push(v);
            nrm.push(n);
        }
        idx.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
    (pos, nrm, idx)
}

/// 离屏渲染 640×480 单立方（正交正面，L1 golden 入口；无适配器=None）。
#[must_use]
pub fn render_offscreen_cube(base_color: [f32; 4]) -> Option<OffscreenFrame> {
    render_offscreen_cube_at_with(base_color, [0.0; 3])
}

/// 立方在 offset 远点仍与原点渲染像素一致（WGPU-10：相机随至 10m 前，f64 相消）。
#[must_use]
pub fn render_offscreen_cube_at(offset: [f64; 3]) -> Option<OffscreenFrame> {
    render_offscreen_cube_at_with([1.0, 0.0, 0.0, 1.0], offset)
}

fn render_offscreen_cube_at_with(base_color: [f32; 4], offset: [f64; 3]) -> Option<OffscreenFrame> {
    use visiaengine_render::{
        Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport,
    };
    let mut backend = crate::headless::HeadlessBackend::new(640, 480)?;
    let (pos, nrm, idx) = cube_mesh();
    let mesh = backend
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .ok()?;
    let material = backend.create_material(base_color).ok()?;
    let eye = [offset[0], offset[1], offset[2] + 3.0];
    let rig = CameraRig::look_at(eye, offset, [0.0, 1.0, 0.0]);
    let view = rig.view_rotation();
    let proj = rig.ortho_frame(1.0, 640.0, 480.0, 0.1, 100.0)?;
    let frame = Frame {
        viewport: Viewport::new(640, 480, 1.0),
        camera: Camera::ortho(1.0, 1.0 / (640.0 / 480.0), 0.1, 100.0),
        view_rot: view,
        eye,
        proj,
        px_world_scale: 1.0,
        shadow: None,
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0.05, 0.07, 0.10, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh,
                material,
                origin: offset,
                transform: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
        ],
    };
    backend.render_to_pixels(&frame)
}
