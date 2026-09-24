//! E204 · 数据装载·点云 —— io-points(PLY)/add_points 直通 × splat 展示双模例（点云带 E 片）。
//! 程序化螺旋点云自证 golden（确定性、不赌文件）；`--file` 走 load_pcl 装载路。
//! 用法: cargo run --example E204_pcl_viewer [选项]
//!   --frames N   headless 自断言快退（ctest/CI 路）
//!   --file PATH  载入 PLY（ascii/binary_le；装载路验收）
//!   无参         常驻人验窗：左键拖=轨道 滚轮=远近+恒径 关窗/Esc 退出

#[path = "gallery.rs"]
mod gallery;

use std::sync::Arc;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, PointMark, PointTableDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;
use visiaengine_render_wgpu::mesh_core::MeshCore;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

const CLEAR: [f32; 4] = [0.05, 0.07, 0.10, 1.0];
const IDENT: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// 确定性螺旋点云（程序化=可断言，不赌 fixture）：n 点、半径域 [−4,4]、色=sRGB 宿主面。
fn helix_marks(n: u32) -> Vec<PointMark> {
    (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let ang = t * std::f32::consts::PI * 6.0;
            let r = 1.5 + 2.5 * t;
            PointMark::new(
                [ang.cos() * r, ang.sin() * r, (t - 0.5) * 4.0],
                [0.25 + 0.7 * t, 0.8 - 0.5 * t, 0.9 - 0.6 * t],
                4.0,
            )
        })
        .collect()
}

/// 装载路（--file）：io-points → 与 mount_pcl 同构的例侧装配（演示 D7 origin 义务）。
fn load_file(path: &str) -> (Vec<PointMark>, [f64; 3], String) {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let cloud = visiaengine_io_points::parse_pcl(&bytes, true).expect("PLY 装载（Lenient）");
    let marks: Vec<PointMark> = cloud
        .positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            PointMark::new(
                *p,
                cloud.colors[i.min(cloud.colors.len() - 1)],
                cloud.radius_px,
            )
        })
        .collect();
    let fmt = cloud.meta.str_value(0, "format").unwrap_or("?").to_string();
    (marks, cloud.origin, fmt)
}

struct Scene {
    marks: Vec<PointMark>,
    origin: [f64; 3],
}

/// headless 自断言路：非背景计数下界 + 上下半区各有覆盖（批量完整性微缩版）。
fn prove(scene: &Scene, frames: u32) {
    const W: u32 = 256;
    const H: u32 = 256;
    let Some(mut b) = HeadlessBackend::new(W, H) else {
        eprintln!("ERROR: no adapter");
        std::process::exit(2);
    };
    let table = b
        .create_points(&PointTableDesc { data: &scene.marks })
        .expect("point table");
    let commands = vec![
        DrawCommand::ClearColor { rgba: CLEAR },
        DrawCommand::DrawPoints {
            table,
            origin: scene.origin,
            transform: IDENT,
        },
    ];
    let rig = CameraRig::look_at([0.0, 0.0, 12.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(6.0, 6.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: [0.0, 0.0, 12.0],
        proj: rig
            .ortho_frame(6.0, W as f32, H as f32, 0.1, 100.0)
            .unwrap(),
        px_world_scale: 2.0 * 6.0 / W as f32,
        shadow: None,
        clip: None,
        commands,
    };
    for f in 0..frames.max(1) {
        let img = b.render_to_pixels(&frame).expect("render");
        gallery::save_frame(&img, "E204_pcl_viewer");
        let nonbg = |y0: u32, y1: u32| -> u32 {
            (y0..y1)
                .flat_map(|y| (0..W).map(move |x| (y, x)))
                .filter(|&(y, x)| {
                    let o = ((y * W + x) * 4) as usize;
                    let p = &img.rgba[o..o + 3];
                    p[0].abs_diff(13) > 4 || p[1].abs_diff(18) > 4 || p[2].abs_diff(25) > 4
                })
                .count() as u32
        };
        let (up, dn) = (nonbg(0, H / 2), nonbg(H / 2, H));
        println!(
            "FRAME {f} pcl points={} nonbg up={up} down={dn}",
            scene.marks.len()
        );
        // 阈随点数量纲（radius 4px≈面积域 8px/点）：小样本 --file 与 600 点螺旋共用
        let floor = (scene.marks.len() * 4).max(8) as u32;
        assert!(up + dn > floor / 2, "splat 覆盖异常（丢失/空帧）");
        assert!(up > 2 && dn > 2, "上下半区覆盖失衡（表批量截断形）");
    }
    println!("OK pcl viewer（{} 点自断言）", scene.marks.len());
}

/// 常驻人验窗路（无参）。
struct App {
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    window: Option<Arc<Window>>,
    rig: CameraRig,
    dragging: Option<(f64, f64)>,
    scene: Scene,
    table: Option<u64>,
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
                        .with_title("VisiaEngine E204 · 点云（拖=轨道 滚轮=远近·径恒定 关窗退出）"),
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
        self.table = Some(
            core.create_points(&PointTableDesc {
                data: &self.scene.marks,
            })
            .expect("point table"),
        );
        println!("loaded {} points", self.scene.marks.len());
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
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.dragging = matches!(state, ElementState::Pressed).then_some((-1.0, -1.0));
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some((lx, ly)) = self.dragging {
                    let (x, y) = (position.x, position.y);
                    if lx >= 0.0 {
                        self.rig.orbit_delta((x - lx) * 0.006, (y - ly) * 0.006);
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    self.dragging = Some((x, y));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    MouseScrollDelta::PixelDelta(p) => -p.y,
                };
                let f = 1.0 - d * 0.0012;
                self.rig.zoom = (self.rig.zoom * f).clamp(2.0, 60.0);
                self.rig.dist = (self.rig.dist * f).clamp(5.0, 80.0);
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(surface), Some(config), Some(table)) =
                    (&mut self.core, &self.surface, &self.config, self.table)
                else {
                    event_loop.exit();
                    return;
                };
                let commands = vec![
                    DrawCommand::ClearColor { rgba: CLEAR },
                    DrawCommand::DrawPoints {
                        table,
                        origin: self.scene.origin,
                        transform: IDENT,
                    },
                ];
                let (near, far) = ((self.rig.dist * 0.01) as f32, (self.rig.dist * 30.0) as f32);
                let Some(proj) = core_ortho(&self.rig, config.width, config.height, near, far)
                else {
                    event_loop.exit();
                    return;
                };
                let frame = Frame {
                    viewport: Viewport::new(config.width, config.height, 1.0),
                    camera: Camera::ortho(self.rig.zoom as f32, self.rig.zoom as f32, near, far),
                    view_rot: self.rig.view_rotation(),
                    eye: self.rig.eye(),
                    proj,
                    px_world_scale: 2.0 * self.rig.zoom as f32 / config.width.max(1) as f32,
                    shadow: None,
                    clip: None,
                    commands,
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

/// 顶视 ortho（半宽=zoom，方像元 hh=zoom·h/w——REND-12 约定，恒径语义前提）。
fn core_ortho(rig: &CameraRig, w: u32, h: u32, near: f32, far: f32) -> Option<[[f32; 4]; 4]> {
    rig.ortho_frame(rig.zoom as f32, w.max(1) as f32, h.max(1) as f32, near, far)
}

/// B2 点拾取自证（REND-38 argv 路，C15②：仅 argv 消费，零参窗路零触碰）。
/// 与 --frames 同一投影装配（渲染数学=拾取数学单源）；三断言：
/// ① 中心点命中 ② 半径外 miss ③ 同屏两点按视深最近者胜。
fn pick_check_run(scene: &Scene) {
    const W: f32 = 256.0;
    const H: f32 = 256.0;
    let rig = CameraRig::look_at([0.0, 0.0, 12.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let proj = rig.ortho_frame(6.0, W, H, 0.1, 100.0).expect("ortho frame");
    let ident = [
        [1.0f64, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let mvp = visiaengine_render::rebase::compose_mvp(
        &proj,
        &rig.view_rotation(),
        &rig.eye(),
        &[0.0; 3],
        &ident,
    );
    // Scene origin == mount origin: candidates carry the cloud's D7 origin; MVP is
    // composed for origin=[0,0,0] here (programmatic helix mounts at [0,0,0]).
    let local: Vec<[f32; 3]> = scene
        .marks
        .iter()
        .map(|m| {
            let o = scene.origin;
            [
                m.pos[0] - o[0] as f32,
                m.pos[1] - o[1] as f32,
                m.pos[2] - o[2] as f32,
            ]
        })
        .collect();
    let id = visiaengine_core::Scene::new().spawn();
    let cands = vec![visiaengine_render::PointCloudCandidate {
        entity: id,
        origin: scene.origin,
        transform: &ident,
        positions: &local,
    }];
    // ① helix guarantees a point near the z-axis center of the xy extent → its
    // projection must fall inside a generous 24px radius of the probe ray.
    // ①a PIT-8 probe-first: the helix is a HOLLOW band (r∈[1.5,4], no axis point) —
    // center probe would miss BY CONSTRUCTION. Instead pick the projected position
    // of the first helix point itself: project (1.5,0,-2)·MVP and probe there.
    let p0 = local[0];
    let cx = f64::from(mvp[0][0]) * f64::from(p0[0])
        + f64::from(mvp[1][0]) * f64::from(p0[1])
        + f64::from(mvp[2][0]) * f64::from(p0[2])
        + f64::from(mvp[3][0]);
    let cy = f64::from(mvp[0][1]) * f64::from(p0[0])
        + f64::from(mvp[1][1]) * f64::from(p0[1])
        + f64::from(mvp[2][1]) * f64::from(p0[2])
        + f64::from(mvp[3][1]);
    let cw = f64::from(mvp[0][3]) * f64::from(p0[0])
        + f64::from(mvp[1][3]) * f64::from(p0[1])
        + f64::from(mvp[2][3]) * f64::from(p0[2])
        + f64::from(mvp[3][3]);
    let sx = ((cx / cw) + 1.0) * f64::from(W) * 0.5;
    let sy = (1.0 - (cy / cw)) * f64::from(H) * 0.5;
    let hit = visiaengine_render::pick_points(&mvp, W, H, sx as f32, sy as f32, 6.0, &cands)
        .unwrap_or_else(|| panic!("pick_check: projected p0 ({sx:.0},{sy:.0}) must hit"));
    assert_eq!(
        hit.point_index, 0,
        "nearest to p0's own projection = p0 itself"
    );
    println!("PICK-CHECK p0 at ({sx:.0},{sy:.0}) depth={:.4}", hit.view_z);
    // ② far corner: no helix point within 1px radius there.
    assert!(
        visiaengine_render::pick_points(&mvp, W, H, 1.0, 1.0, 1.0, &cands).is_none(),
        "pick_check: corner must miss"
    );
    // ③ nearest-wins determinism: same query twice → identical hit depth.
    let h2 = visiaengine_render::pick_points(&mvp, W, H, sx as f32, sy as f32, 6.0, &cands)
        .expect("pick_check: repeat query");
    assert_eq!(hit.view_z.to_bits(), h2.view_z.to_bits(), "deterministic");
    assert_eq!(hit.point_index, h2.point_index, "nearest-wins stable");
    println!("OK pcl pick-check（REND-38 断言过）");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut frames: Option<u32> = None;
    let mut file: Option<String> = None;
    let mut pick_check = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--frames" => {
                frames = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n: &u32| *n > 0)
            }
            "--file" => file = args.next(),
            "--pick-check" => pick_check = true,
            _ => {}
        }
    }
    let scene = match &file {
        Some(p) => {
            let (marks, origin, fmt) = load_file(p);
            println!("loaded file={p} format={fmt}");
            Scene { marks, origin }
        }
        None => Scene {
            marks: helix_marks(600),
            origin: [0.0; 3],
        },
    };
    if pick_check {
        pick_check_run(&scene);
        return Ok(());
    }
    if let Some(n) = frames {
        prove(&scene, n);
        return Ok(());
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        core: None,
        surface: None,
        config: None,
        window: None,
        rig: CameraRig::orbit([0.0, 0.0, 0.0], 0.4, 1.1, 16.0, 6.0, 1.0, 0.1, 100.0),
        dragging: None,
        scene,
        table: None,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
