//! E510 Route flow — V2.1 parametric curves live demo (REND-47).
//!
//! - `--frames N` = headless assert path (this lane): CatmullRom route
//!   ribbon plus tube guard-rail uploaded as regular meshes (+0 ABI), N flow
//!   markers sampled along the curve and drawn as points; endpoint law
//!   asserted, marker spread pixel-verified.
//! - Zero-arg = prints the window-lane note (interactive orbit lives in
//!   E306/E901; this example is assert-first by design).
//!
//! Claim: consumer band — curve math + mesh generation only; no engine
//! changes. OpenDRIVE road-mesh prereq (V4.2 dependency anchor).

#![expect(clippy::cast_possible_truncation)]

#[path = "gallery.rs"]
mod gallery;
use examples::viewer::{Ctx, FormatPolicy};
use gallery::save_frame;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MeshDesc, PointMark, PointTableDesc,
    Viewport,
    curve::{ribbon_xz, sample_catmull_rom, tube},
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, MultiClearPolicy};

const W: u32 = 480;
const H: u32 = 320;

/// S-curved route knots (XZ plane, y=0 ground).
fn route_knots() -> Vec<[f32; 3]> {
    vec![
        [-18.0, 0.0, 0.0],
        [-8.0, 0.0, 6.0],
        [0.0, 0.0, -6.0],
        [8.0, 0.0, 6.0],
        [18.0, 0.0, 0.0],
    ]
}

fn scene(b: &mut MeshCore, flow_n: usize) -> (Vec<DrawCommand>, Vec<[f32; 3]>) {
    let knots = route_knots();
    let center = sample_catmull_rom(&knots, 24);
    assert_eq!(center.first(), knots.first(), "curve endpoint law (start)");
    let tail = *center.last().expect("non-empty");
    assert!(
        (tail[0] - knots[knots.len() - 1][0]).abs() < 1e-5,
        "curve endpoint law (end)"
    );

    // Road ribbon (dark asphalt) + tube guard rails (orange, offset +/-2.2 z
    // by a shifted knot copy — same curve family, cheaper than true offsets).
    let (rpos, ridx) = ribbon_xz(&center, 4.0);
    let road = b
        .upload_mesh(&MeshDesc {
            uv: &[],
            positions: &rpos,
            normals: &vec![[0.0, 1.0, 0.0]; rpos.len()],
            indices: &ridx,
        })
        .expect("road mesh");
    let road_mat = b
        .upload_material_desc(&MaterialDesc {
            base_color: [0.16, 0.17, 0.19, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 0.9,
            metallic: 0.0,
        })
        .expect("road mat");

    let mut shifted_up: Vec<[f32; 3]> = knots.iter().map(|p| [p[0], p[1], p[2] - 2.6]).collect();
    let c_up = sample_catmull_rom(shifted_up.as_slice(), 24);
    let (t1p, t1i) = tube(&c_up, 0.18, 6);
    shifted_up = knots.iter().map(|p| [p[0], p[1], p[2] + 2.6]).collect();
    let c_dn = sample_catmull_rom(shifted_up.as_slice(), 24);
    let (t2p, t2i) = tube(&c_dn, 0.18, 6);
    let rail1 = b
        .upload_mesh(&MeshDesc {
            uv: &[],
            positions: &t1p,
            normals: &t1p.iter().map(|p| norm_of(*p)).collect::<Vec<_>>(),
            indices: &t1i,
        })
        .expect("rail1");
    let rail2 = b
        .upload_mesh(&MeshDesc {
            uv: &[],
            positions: &t2p,
            normals: &t2p.iter().map(|p| norm_of(*p)).collect::<Vec<_>>(),
            indices: &t2i,
        })
        .expect("rail2");
    let rail_mat = b
        .upload_material_desc(&MaterialDesc {
            base_color: [0.95, 0.62, 0.18, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 0.6,
            metallic: 0.0,
        })
        .expect("rail mat");

    let mut cmds = vec![
        DrawCommand::ClearColor {
            rgba: [0.06, 0.08, 0.11, 1.0],
        },
        DrawCommand::DrawMesh {
            mesh: road,
            material: road_mat,
            origin: [0.0; 3],
            transform: identity(),
        },
        DrawCommand::DrawMesh {
            mesh: rail1,
            material: rail_mat,
            origin: [0.0; 3],
            transform: identity(),
        },
        DrawCommand::DrawMesh {
            mesh: rail2,
            material: rail_mat,
            origin: [0.0; 3],
            transform: identity(),
        },
    ];
    // flow_n = 0 = window lane (markers are the per-frame dynamic table,
    // not baked here); flow_n > 0 = assert lane (endpoint law applies).
    if flow_n > 0 {
        let marks: Vec<PointMark> = (0..flow_n)
            .map(|i| {
                let idx = i * (center.len() - 1) / flow_n;
                let mut p = center[idx];
                p[1] = 0.15; // float above road
                PointMark::new(p, [1.0, 0.9, 0.2], 6.0)
            })
            .collect();
        assert_eq!(
            marks.first().expect("first").pos[0],
            center[0][0] as f32,
            "flow endpoint law"
        );
        let pt_table = b
            .create_points(&PointTableDesc { data: &marks })
            .expect("points");
        cmds.push(DrawCommand::DrawPoints {
            table: pt_table,
            origin: [0.0; 3],
            transform: identity(),
        });
    }
    (cmds, center)
}

fn identity() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn norm_of(p: [f32; 3]) -> [f32; 3] {
    let l = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    if l > 1e-6 {
        [p[0] / l, p[1] / l, p[2] / l]
    } else {
        [0.0, 1.0, 0.0]
    }
}

/// World units per screen pixel at the orbit target (REND-29 exact path).
/// Point radii and stroke widths carry *pixel* units, so leaving
/// px_world_scale at 1.0 would turn a 6 px marker into a 6 *world unit* disc
/// (measured: the markers swallowed the road in both lanes).
fn px_scale(rig: &visiaengine_render::CameraRig, target: [f64; 3], height: u32) -> f32 {
    let e = rig.eye();
    let d = ((e[0] - target[0]).powi(2) + (e[1] - target[1]).powi(2) + (e[2] - target[2]).powi(2))
        .sqrt();
    (2.0 * d * (rig.fov_y / 2.0).tan() / f64::from(height.max(1))) as f32
}

fn frame(cmds: &[DrawCommand]) -> Frame {
    let rig = visiaengine_render::CameraRig::look_at(
        [0.0, -26.0, 18.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    );
    Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, W as f32 / H as f32, 0.1, 200.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(rig.fov_y as f32, W as f32 / H as f32, 0.1, 200.0)
            .expect("proj"),
        px_world_scale: px_scale(&rig, [0.0, 0.0, 0.0], H),
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: cmds.to_vec(),
    }
}

fn diff_px(
    a: &visiaengine_render_wgpu::OffscreenFrame,
    b: &visiaengine_render_wgpu::OffscreenFrame,
) -> u32 {
    a.rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.rgba.as_chunks::<4>().0)
        .filter(|(x, y)| x[..3] != y[..3])
        .count() as u32
}

fn prove(frames: u32) {
    let _ = frames;
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (cmds, _center) = scene(b.core_mut(), 12);

    let base = b.render_to_pixels(&frame(&cmds)).expect("render");
    // route spread: the S-curve must put geometry across the frame — sample
    // the road region lit vs clear pixels (probe floor, PIT-8 conservative).
    // Self-calibrated clear reference: the top-left corner of this scene is sky
    // by construction, so the render itself reports its clear colour. A hand
    // written triple here was silently wrong (the count matched *no* pixel,
    // making the old assertion constant-true) -- measured, not assumed.
    let px0 = &base.rgba.as_chunks::<4>().0[0];
    let clear = [px0[0], px0[1], px0[2]];
    let geo = base
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| [p[0], p[1], p[2]] != clear)
        .count();
    println!("E510 probe: clear={:?} geometry px={geo}", clear);
    assert!(geo > 4000, "route geometry must cover the frame: {geo}");

    // Flow density law: denser sampling = strictly more marker pixels than a
    // sparse render (same camera, same geometry, only points differ).
    let (cmds_sparse, _) = scene(b.core_mut(), 3);
    let sparse = b.render_to_pixels(&frame(&cmds_sparse)).expect("sparse");
    let d = diff_px(&sparse, &base);
    println!("E510 probe: flow-density diff={d}");
    assert!(d > 30, "flow marker density must be visible: {d}");

    save_frame(&base, "E510_route_flow");
    println!("E510 OK: curve endpoints exact; ribbon+tube uploaded; flow markers pixel-verified");
}

/// Window lane state (C15 dual-mode: zero-arg = resident human window).
struct App {
    ctx: Option<Ctx>,
    core: Option<MeshCore>,
    // Static GPU products (road + rails), built once in resumed().
    static_cmds: Vec<DrawCommand>,
    // Flow lane: sample positions along the route center; the point table is
    // rebuilt each frame from a advancing phase (wrap at the end).
    center: Vec<[f32; 3]>,
    table: Option<visiaengine_render::TableId>,
    phase: f64,
    last: Option<std::time::Instant>,
    // Orbit camera (E508 shape)
    rig: visiaengine_render::CameraRig,
    drag: bool,
    last_pos: (f64, f64),
    ready: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ready {
            return;
        }
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "E510 route flow · drag=orbit wheel=zoom · Esc exit",
            800,
            560,
            FormatPolicy::SurfaceDefault,
        ) else {
            event_loop.exit();
            return;
        };
        // Static scene (road + rails): reuse the prove-lane builder but with
        // flow_n = 0 (the point table is the per-frame dynamic part).
        let (cmds, center) = scene(&mut core, 0);
        self.static_cmds = cmds;
        self.center = center;
        self.core = Some(core);
        self.ctx = Some(ctx);
        self.ready = true;
        self.last = Some(std::time::Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == winit::event::ElementState::Pressed
                    && event.physical_key
                        == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =>
            {
                event_loop.exit();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    self.drag = state == winit::event::ElementState::Pressed;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.drag {
                    let dx = position.x - self.last_pos.0;
                    let dy = position.y - self.last_pos.1;
                    self.last_pos = (position.x, position.y);
                    let (eye, target) = (self.rig.eye(), [0.0, 0.0, 0.0]);
                    let off = [eye[0] - target[0], eye[1] - target[1], eye[2] - target[2]];
                    let r = (off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt();
                    let theta = off[1].atan2(off[0]) + dx * 0.005;
                    let phi = (off[2] / r).clamp(-1.4, 1.4) - dy * 0.005;
                    let (ex, ey, ez) = (
                        target[0] + r * phi.cos() * theta.cos(),
                        target[1] + r * phi.cos() * theta.sin(),
                        target[2] + r * phi.sin(),
                    );
                    self.rig = CameraRig::look_at([ex, ey, ez.max(0.5)], target, [0.0, 0.0, 1.0]);
                } else {
                    self.last_pos = (position.x, position.y);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => f64::from(y) * 1.0,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y * 0.05,
                };
                let (eye, target) = (self.rig.eye(), [0.0, 0.0, 0.0]);
                let off = [eye[0] - target[0], eye[1] - target[1], eye[2] - target[2]];
                let r = ((off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt()
                    * (1.0 - d * 0.1))
                    .clamp(6.0, 80.0);
                let len = (off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt();
                let k = r / len;
                self.rig = CameraRig::look_at(
                    [
                        target[0] + off[0] * k,
                        target[1] + off[1] * k,
                        target[2] + off[2] * k,
                    ],
                    target,
                    [0.0, 0.0, 1.0],
                );
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        // Flow clock: markers advance along the polyline, wrap at the end.
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last.unwrap_or(now)).as_secs_f64();
        self.last = Some(now);
        self.phase = (self.phase + dt * 6.0) % self.center.len() as f64;

        // The rects-based present is this example's subject (5b) -- it stays local.
        // The shared Ctx hands back the pieces the body needs, under the old names, so
        // everything past this point is untouched text.
        let (Some(core), Some(ctx)) = (self.core.as_mut(), self.ctx.as_mut()) else {
            return;
        };
        let surface = &ctx.surface;
        let config = &ctx.config;
        let window = &ctx.window;
        window.set_title(
            format!(
                "E510 route flow · phase={:.1} · drag=orbit wheel=zoom · Esc exit",
                self.phase
            )
            .as_str(),
        );
        // rebuild the flow point table at the new phase (12 markers, spaced
        // 1/12 of the route apart)
        let n = self.center.len();
        let marks: Vec<PointMark> = (0..12)
            .map(|i| {
                let idx = ((self.phase as usize) + i * n / 12) % n;
                let mut p = self.center[idx];
                p[1] = 0.15;
                PointMark::new(p, [1.0, 0.9, 0.2], 6.0)
            })
            .collect();
        if let Some(old) = self.table.take() {
            core.destroy_points(old);
        }
        let new_table = core.create_points(&PointTableDesc { data: &marks }).ok();
        let mut cmds = self.static_cmds.clone();
        if let Some(table) = new_table {
            cmds.push(DrawCommand::DrawPoints {
                table,
                origin: [0.0; 3],
                transform: identity(),
            });
        }
        self.table = new_table;
        let frame = {
            let rig = &self.rig;
            Frame {
                viewport: Viewport::new(config.width, config.height, 1.0),
                camera: Camera::perspective(
                    rig.fov_y as f32,
                    config.width as f32 / config.height.max(1) as f32,
                    0.1,
                    200.0,
                ),
                view_rot: rig.view_rotation(),
                eye: rig.eye(),
                proj: rig
                    .perspective(
                        rig.fov_y as f32,
                        config.width as f32 / config.height.max(1) as f32,
                        0.1,
                        200.0,
                    )
                    .expect("proj"),
                px_world_scale: px_scale(rig, [0.0, 0.0, 0.0], config.height),
                shadow: None,
                clip: None,
                edl: None,
                post: Vec::new(),
                commands: cmds,
            }
        };
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                let view = tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                core.render_view_rects(
                    &[(
                        frame,
                        visiaengine_render::ViewportRect::new(0, 0, config.width, config.height),
                    )],
                    &view,
                    config.width,
                    config.height,
                    config.format,
                    MultiClearPolicy::FirstClearRestLoad,
                );
                core.queue.present(tex);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                surface.configure(&core.device, config);
            }
            other => eprintln!("skip {other:?}"),
        }
    }
}

fn window_form() -> Result<(), Box<dyn std::error::Error>> {
    let el = EventLoop::new()?;
    el.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        ctx: None,
        core: None,
        static_cmds: Vec::new(),
        center: Vec::new(),
        table: None,
        phase: 0.0,
        last: None,
        rig: CameraRig::look_at([0.0, -26.0, 18.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        drag: false,
        last_pos: (0.0, 0.0),
        ready: false,
    };
    el.run_app(&mut app)?;
    Ok(())
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
    match frames {
        Some(n) => prove(n),
        None => window_form()?,
    }
    Ok(())
}
