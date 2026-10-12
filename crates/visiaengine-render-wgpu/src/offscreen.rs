//! L1 offscreen rendering: surface-less triangles + readback (consumed by golden tests; the must-green CI lavapipe face).

use wgpu::util::DeviceExt as _;

/// One frame's readback result (RGBA8; the row pitch width*4 is naturally 256-aligned at 640).
pub struct OffscreenFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

const CLEAR: wgpu::Color = wgpu::Color {
    // CORE-16: the srgb target load = linear domain -- this is the srgb_to_linear of CSS (0.05,0.07,0.10)
    // literal pin (a const cannot call the conversion entry; changing the value must pass the CORE-16 dual entries in sync).
    r: 0.003872,
    g: 0.007894,
    b: 0.012201,
    a: 1.0,
};

// the red triangle covering the center pixel (NDC)
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

/// Offscreen-render a 640x480 red triangle; no adapter returns None (a reportable state).
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
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
        // v30 spec alignment: draw/set_* state belongs to the render-pass scope
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
    // v30: map_async is callback-style; once the poll completes the channel holds the Result<Result<..>>
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

/// The standard cube of 24 vertices / 36 indices (per-face normals; edge 1.0 centered at the origin).
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

/// The unit box: xy in [-0.5,0.5], **z in [0,1] bottom-aligned** (the geometric precondition of the WGPU-16 extrusion semantics),
/// 4 faces 16 vertices / 24 indices (bottom omitted -- never visible from top/oblique views, the white-model stress case halves the volume).
/// Double consumption by tests/instances.rs and examples/bench_twin.rs (DRY, [E3D:A5] teaching-piece same zone).
/// Winding = outward-normal CCW (the pre-existing mine unearthed in the 2026-09-17 E402 band: born reversed -- the GPU road's cull
/// defaults off so it stayed invisible, and once pick follows the front-face rule REND-23 with winding, every downward ray misses.
/// This line's semantics = the only correct form for picking / future cull installation, do not regress).
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
        idx.extend_from_slice(&[b, b + 2, b + 1, b, b + 3, b + 2]);
    }
    (pos, nrm, idx)
}

/// Offscreen-render a single 640x480 cube (orthographic front, the L1 golden entry; no adapter = None).
#[must_use]
pub fn render_offscreen_cube(base_color: [f32; 4]) -> Option<OffscreenFrame> {
    render_offscreen_cube_at_with(base_color, [0.0; 3])
}

/// The cube still renders pixel-identically at a far offset point (WGPU-10: the camera follows to 10m away, cancellation in f64).
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
        clip: None,
        edl: None,
        post: Vec::new(),
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
