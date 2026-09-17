//! E502 · 材质与光影·色彩标定 —— sRGB 全链往返「所见即所得」双模例（CORE-16 带 K2）。
//! 四色板法向∥光向 → shade=1.0 → 读回字节应≈CSS 原值（后端线性化输入 + Srgb 目标
//! 编码输出，8bit 量化内互逆）。色域错装=本例首现红。
//! 双模式（E501/E801 同制）：
//!   无参        = 常驻人验窗（IDE cargo-run_E502 路：四色板肉眼验收色彩链，关窗/Esc 退出）
//!   --frames N  = headless 自断言快退（ctest 路；逐板字节比对，FAIL 非零退）

use std::sync::Arc;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;
use visiaengine_render_wgpu::mesh_core::MeshCore;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

/// 四色板：CSS 原值（sRGB 域）。纯主色/中灰/白——中灰最能抓线性域错装。
const SWATCHES: [(u8, u8, u8); 4] = [
    (255, 0, 0),
    (128, 128, 128), // 错装=偏亮到 ~188，本案判据
    (0, 0, 255),
    (255, 255, 255),
];
const HEADLESS_W: u32 = 256; // 行 1024B=4×256 对齐（headless 读回路）
const HEADLESS_H: u32 = 256;

/// 光向（与后端 shadow-off dummy params 旧 LIGHT 同位型 normalize(0.5,0.7,0.4)）——
/// 面元法向取此向 → ndl=1 → shade=0.35+0.65=1.0（色板满亮，色域纯判）。
fn light_dir() -> [f32; 3] {
    let (x, y, z) = (0.5f32, 0.7, 0.4);
    let l = (x * x + y * y + z * z).sqrt();
    [x / l, y / l, z / l]
}

/// 单色板几何数据（局部系 z=0，法向=light_dir；四角两三角）。cx=板心 x。
fn swatch_quad(cx: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let n = light_dir();
    let (x0, x1, y0, y1) = (cx - 0.5, cx + 0.5, -1.0, 1.0);
    (
        vec![[x0, y0, 0.0], [x1, y0, 0.0], [x1, y1, 0.0], [x0, y1, 0.0]],
        vec![n; 4],
        vec![0, 1, 2, 0, 2, 3],
    )
}

const IDENT: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// headless 自断言路（--frames N）：逐板中心像素 vs CSS 原值，容差 ±8。
fn prove_headless(frames: u32) {
    let Some(mut b) = HeadlessBackend::new(HEADLESS_W, HEADLESS_H) else {
        eprintln!("ERROR: no adapter");
        std::process::exit(2);
    };
    let mut commands = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
    for (i, (r, g, bl)) in SWATCHES.into_iter().enumerate() {
        let css = [r as f32 / 255.0, g as f32 / 255.0, bl as f32 / 255.0, 1.0];
        let (pos, nrm, idx) = swatch_quad(-1.5 + i as f32);
        let mesh = b
            .create_mesh(&MeshDesc {
                positions: &pos,
                normals: &nrm,
                indices: &idx,
                uv: &[],
            })
            .expect("mesh");
        let mat = b.create_material(css).expect("mat");
        commands.push(DrawCommand::DrawMesh {
            mesh,
            material: mat,
            origin: [0.0; 3],
            transform: IDENT,
        });
    }
    let rig = CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let frame = Frame {
        viewport: Viewport::new(HEADLESS_W, HEADLESS_H, 1.0),
        camera: Camera::ortho(2.0, 2.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: [0.0, 0.0, 10.0],
        proj: rig
            .ortho_frame(2.0, HEADLESS_W as f32, HEADLESS_H as f32, 0.1, 100.0)
            .unwrap(),
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        commands,
    };
    for _ in 0..frames.max(1) {
        let img = b.render_to_pixels(&frame).expect("render");
        let mut bad = 0usize;
        for (i, (cr, cg, cb)) in SWATCHES.into_iter().enumerate() {
            // 板心像素：ortho 半宽 2 → px=(0.5 + x/4)·256，板心 -1.5..1.5 → 32/96/160/224
            let px = ((0.5 + (-1.5 + i as f32) / 4.0) * HEADLESS_W as f32) as u32;
            let o = (((HEADLESS_H / 2) * HEADLESS_W + px) * 4) as usize;
            let (r, g, bl) = (img.rgba[o], img.rgba[o + 1], img.rgba[o + 2]);
            let ok = r.abs_diff(cr) <= 8 && g.abs_diff(cg) <= 8 && bl.abs_diff(cb) <= 8;
            if !ok {
                bad += 1;
            }
            println!(
                "SWATCH {i} css=({cr},{cg},{cb}) read=({r},{g},{bl}) {}",
                if ok { "OK" } else { "✗ 色域漂移" }
            );
        }
        if bad > 0 {
            println!("FAIL color tuning bad={bad}");
            std::process::exit(1);
        }
    }
    println!("OK color tuning（sRGB 全链往返互逆·所见即所得）");
}

// ────────────────────────── 常驻人验窗路（无参） ──────────────────────────

struct App {
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    window: Option<Arc<Window>>,
    commands: Vec<DrawCommand>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.core.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(PhysicalSize::new(960u32, 600u32))
                        .with_title(
                            "VisiaEngine E502 · 色彩标定人验窗：四板=纯红 | 中灰#808080(不发亮) | 纯蓝 | 白(不偏粉)",
                        ),
                )
                .expect("create_window"),
        );
        let size = window.inner_size();
        let instance = visiaengine_render_wgpu::create_instance();
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("create_surface");
        let Some(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .ok()
        else {
            eprintln!("no adapter — lavapipe/real GPU required");
            event_loop.exit();
            return;
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("visiaengine-window"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("request_device");
        let mut core = MeshCore::new(device, queue, instance, adapter.clone());
        let caps = surface.get_capabilities(&adapter);
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("surface config");
        config.format = caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                matches!(
                    f,
                    wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb
                )
            })
            .unwrap_or(caps.formats[0]);
        surface.configure(&core.device, &config);

        let mut commands = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
        for (i, (r, g, bl)) in SWATCHES.into_iter().enumerate() {
            let css = [r as f32 / 255.0, g as f32 / 255.0, bl as f32 / 255.0, 1.0];
            let (pos, nrm, idx) = swatch_quad(-1.5 + i as f32);
            let mesh = core
                .upload_mesh(&MeshDesc {
                    positions: &pos,
                    normals: &nrm,
                    indices: &idx,
                    uv: &[],
                })
                .expect("mesh");
            let mat = core.upload_material(css).expect("mat");
            commands.push(DrawCommand::DrawMesh {
                mesh,
                material: mat,
                origin: [0.0; 3],
                transform: IDENT,
            });
        }
        println!("loaded {} swatches", SWATCHES.len());
        self.commands = commands;
        self.window = Some(window);
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }
                if let (Some(core), Some(surface), Some(config)) = (
                    self.core.as_mut(),
                    self.surface.as_ref(),
                    self.config.as_mut(),
                ) {
                    config.width = size.width.max(1);
                    config.height = size.height.max(1);
                    surface.configure(&core.device, config);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && matches!(&event.logical_key, Key::Named(NamedKey::Escape))
                {
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &self.config)
                else {
                    event_loop.exit();
                    return;
                };
                let rig = CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
                let Some(proj) = rig.ortho_frame(
                    2.0,
                    config.width as f32,
                    config.height.max(1) as f32,
                    0.1,
                    100.0,
                ) else {
                    event_loop.exit();
                    return;
                };
                let frame = Frame {
                    viewport: Viewport::new(config.width, config.height, 1.0),
                    camera: Camera::ortho(2.0, 2.0, 0.1, 100.0),
                    view_rot: rig.view_rotation(),
                    eye: [0.0, 0.0, 10.0],
                    proj,
                    px_world_scale: 1.0,
                    shadow: None,
                    clip: None,
                    commands: self.commands.clone(),
                };
                match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(tex)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                        let view = tex
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        core.render_view_format(
                            &frame,
                            &view,
                            config.width,
                            config.height.max(1),
                            config.format,
                        );
                        core.queue.present(tex);
                    }
                    wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                        surface.configure(&core.device, config);
                    }
                    other => eprintln!("skipped: {other:?}"),
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut frames: Option<u32> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--frames" {
            frames = args
                .next()
                .and_then(|v| v.parse().ok())
                .filter(|n: &u32| *n > 0);
        }
    }
    if let Some(n) = frames {
        prove_headless(n);
        return Ok(());
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        core: None,
        surface: None,
        config: None,
        window: None,
        commands: Vec::new(),
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
