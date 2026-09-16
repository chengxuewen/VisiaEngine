//! E201 · 数据装载·glTF —— io-gltf × MeshCore 合流 + 键盘轨道（4 连招：装载→属性(E203)→样式→IO）。
//! 用法: cargo run --example E201_load_gltf [path.glb] [--frames N]（smoke: N=3 自动退出）

use std::sync::Arc;

use visiaengine_io_gltf::load_gltf;
use visiaengine_render::{Camera, CameraRig, DrawCommand, Frame, MeshDesc, MeshId, Viewport};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

/// 上传期分解记录：mesh、material、世界 origin、origin-local 位姿（D7）。
type DrawRecord = (MeshId, MeshId, [f64; 3], [[f64; 4]; 4]);

const CLEAR: [f32; 4] = [0.05, 0.07, 0.10, 1.0];

struct App {
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    window: Option<Arc<Window>>,
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
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
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
                            "VisiaEngine E201 · glTF（左键拖=轨道 滚轮=缩放 WASD/QE 关窗退出）",
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
        config.format = caps.formats[0];
        surface.configure(&core.device, &config);

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
        for e in doc.entities() {
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
            WindowEvent::KeyboardInput { event, .. } => self.handle_key(&event),
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
                        self.request_redraw();
                    }
                    self.dragging = Some((x, y));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    MouseScrollDelta::PixelDelta(p) => -p.y,
                };
                self.rig.dist = (self.rig.dist * (1.0 - d * 0.0012)).clamp(0.3, 100.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &self.config)
                else {
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
                let aspect = config.width as f32 / config.height.max(1) as f32;
                let (near, far) = ((self.rig.dist * 0.01) as f32, (self.rig.dist * 30.0) as f32);
                let Some(proj) = self
                    .rig
                    .perspective(self.rig.fov_y as f32, aspect, near, far)
                else {
                    event_loop.exit();
                    return;
                };
                let frame = Frame {
                    viewport: Viewport::new(config.width, config.height, 1.0),
                    camera: Camera::perspective(self.rig.fov_y as f32, aspect, near, far),
                    view_rot: self.rig.view_rotation(),
                    eye: self.rig.eye(),
                    proj,
                    px_world_scale: 1.0,
                    shadow: None,
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
                let again = match self.frames_left {
                    None => true,
                    Some(1) => false,
                    Some(n) => {
                        self.frames_left = Some(n - 1);
                        true
                    }
                };
                if again {
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
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
        core: None,
        surface: None,
        config: None,
        window: None,
        dragging: None,
        rig: CameraRig::orbit([0.0, 0.0, 0.0], 0.7, 0.35, 8.0, 1.0, 1.1, 0.01, 500.0),
        statics: Vec::new(),
        frames_left: frames,
        glb_path: path,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
