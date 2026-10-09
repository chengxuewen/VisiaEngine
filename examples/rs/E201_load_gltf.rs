//! E201 · 数据装载·glTF —— io-gltf × MeshCore 合流 + 键盘轨道（4 连招：装载→属性(E203)→样式→IO）。
//! 用法: cargo run --example E201_load_gltf [path.glb] [--frames N]（smoke: N=3 自动退出）

use examples::viewer::{Ctx, FormatPolicy, orbit_drag, zoom_dist};

use visiaengine_io_gltf::load_gltf;
use visiaengine_render::{Camera, CameraRig, DrawCommand, Frame, MeshDesc, MeshId, Viewport};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

/// 上传期分解记录：mesh、material、世界 origin、origin-local 位姿（D7）。
type DrawRecord = (MeshId, MeshId, [f64; 3], [[f64; 4]; 4]);

const CLEAR: [f32; 4] = [0.05, 0.07, 0.10, 1.0];

struct App {
    ctx: Option<Ctx>,
    core: Option<MeshCore>,
    dragging: Option<(f64, f64)>,
    rig: CameraRig,
    statics: Vec<DrawRecord>, // mesh, material, origin, local 位姿（D7 分解）
    frames_left: Option<u32>,
    glb_path: String,
}

impl App {
    fn glb_path(&self) -> &str {
        &self.glb_path
    }
}

impl App {
    fn handle_key(&mut self, key: &KeyEvent) {
        if key.state != ElementState::Pressed {
            return;
        }
        let (mut yaw, mut pitch, mut dist) = (self.rig.yaw, self.rig.pitch, self.rig.dist);
        match &key.logical_key {
            Key::Character(s) => match s.as_str() {
                "a" => yaw += 0.12,
                "d" => yaw -= 0.12,
                "w" => pitch = (pitch + 0.08).min(1.5),
                "s" => pitch = (pitch - 0.08).max(-1.5),
                "e" => dist *= 1.12,
                "q" => dist /= 1.12,
                _ => return,
            },
            Key::Named(NamedKey::Escape) => dist *= 0.9,
            _ => return,
        }
        self.rig.yaw = yaw;
        self.rig.pitch = pitch;
        self.rig.dist = dist;
    }
}

impl App {
    fn request_redraw(&self) {
        if let Some(ctx) = &self.ctx {
            ctx.request_redraw();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.core.is_some() {
            return;
        }
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "VisiaEngine E201 · glTF（左键拖=轨道 滚轮=缩放 WASD/QE 关窗退出）",
            960,
            600,
            FormatPolicy::SurfaceDefault,
        ) else {
            event_loop.exit();
            return;
        };
        // 场景上传：每个 glTF 实体 → mesh + material
        let doc = match load_gltf(self.glb_path()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("load failed: {e}");
                event_loop.exit();
                return;
            }
        };
        // GLTF-11 纹理槽位 → GPU 纹理（image 槽位序，与 capi mount 同构）
        let mut tex_ids = Vec::with_capacity(doc.textures().len());
        for t in doc.textures() {
            match core.upload_texture(&visiaengine_render::TextureDesc {
                rgba: &t.rgba,
                width: t.width,
                height: t.height,
            }) {
                Ok(id) => tex_ids.push(id),
                Err(e) => eprintln!("skip texture: {e:?}"),
            }
        }
        let mut bbox = examples::BBox::new();
        for e in doc.entities() {
            for pt in &e.mesh.positions {
                bbox.push(*pt, &e.world);
            }
            let Ok(mesh) = core.upload_mesh(&MeshDesc {
                positions: &e.mesh.positions,
                normals: &e.mesh.normals,
                indices: &e.mesh.indices,
                uv: &e.mesh.uv,
            }) else {
                eprintln!("skip entity: mesh upload failed");
                continue;
            };
            let mat = visiaengine_render::MaterialDesc {
                base_color: e.mesh.base_color,
                texture: e.mesh.texture.and_then(|s| tex_ids.get(s).copied()),
                repeat: [1.0, 1.0],
                // mock-up [4ab①]：(1-metallic)*roughness（WGPU-14 Lambert 系数）
                specular: (1.0 - e.mesh.metallic_factor) * e.mesh.roughness_factor,
                roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
                metallic: 0.0,
            };
            let Ok(mat) = core.upload_material_desc(&mat) else {
                continue;
            };
            // D7：world 4x4 分解 = 平移列 origin + 纯位姿 local
            let mut origin = [e.world[3][0], e.world[3][1], e.world[3][2]];
            let _ = &mut origin;
            let local = {
                let mut m = e.world;
                m[3] = [0.0, 0.0, 0.0, 1.0];
                m
            };
            self.statics.push((mesh, mat, origin, local));
        }
        println!("loaded {} entities", self.statics.len());
        // 装载期取景拟合（BBox 门）：target=场景中心、dist=半径×2.8——任意 glb 均成景
        let r = bbox.radius();
        self.rig.target = bbox.center();
        self.rig.dist = r * 2.2;
        self.rig.zoom = (r * 1.2).max(0.2);
        self.rig.pitch = self.rig.pitch.max(0.95); // 薄平面场景：低俯角=贴地细条（扫描实锤 .35→红仅 205px）
        self.core = Some(core);
        self.ctx = Some(ctx);
        if let Some(ctx) = &self.ctx {
            ctx.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(core), Some(ctx)) = (self.core.as_ref(), self.ctx.as_mut()) {
                    ctx.on_resize(core, size);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => self.handle_key(&event),
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.dragging = matches!(state, ElementState::Pressed).then_some((-1.0, -1.0));
            }
            WindowEvent::CursorMoved { position, .. } => {
                if orbit_drag(&mut self.rig, &mut self.dragging, position.x, position.y) {
                    self.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                zoom_dist(&mut self.rig, &delta, 0.3, 100.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(ctx)) = (&mut self.core, &mut self.ctx) else {
                    event_loop.exit();
                    return;
                };
                let st = std::mem::take(&mut self.statics);
                let mut commands = vec![DrawCommand::ClearColor { rgba: CLEAR }];
                commands.extend(
                    st.iter()
                        .map(|(m, mat, origin, local)| DrawCommand::DrawMesh {
                            mesh: *m,
                            material: *mat,
                            origin: *origin,
                            transform: *local,
                        }),
                );
                self.statics = st;
                let aspect = ctx.config.width as f32 / ctx.config.height.max(1) as f32;
                let (near, far) = ((self.rig.dist * 0.01) as f32, (self.rig.dist * 30.0) as f32);
                let Some(proj) = self
                    .rig
                    .perspective(self.rig.fov_y as f32, aspect, near, far)
                else {
                    event_loop.exit();
                    return;
                };
                let frame = Frame {
                    viewport: Viewport::new(ctx.config.width, ctx.config.height, 1.0),
                    camera: Camera::perspective(self.rig.fov_y as f32, aspect, near, far),
                    view_rot: self.rig.view_rotation(),
                    eye: self.rig.eye(),
                    proj,
                    px_world_scale: 1.0,
                    shadow: None,
                    clip: None,
                    edl: None,
                    post: Vec::new(),
                    commands,
                };
                ctx.present(core, &frame);
                let again = match self.frames_left {
                    None => true,
                    Some(1) => false,
                    Some(n) => {
                        self.frames_left = Some(n - 1);
                        true
                    }
                };
                if again {
                    ctx.request_redraw();
                } else {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = String::from("resources/data/hierarchy.glb");
    let mut frames: Option<u32> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--frames" => {
                frames = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n: &u32| *n > 0);
            }
            p => path = p.to_string(),
        }
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        ctx: None,
        core: None,
        dragging: None,
        rig: CameraRig::orbit([0.0, 0.0, 0.0], 0.7, 0.35, 8.0, 1.0, 1.1, 0.01, 500.0),
        statics: Vec::new(),
        frames_left: frames,
        glb_path: path,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
