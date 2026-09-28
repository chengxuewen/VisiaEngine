//! Per-property golden pilot (N1.3, band gap-closure-2026-09-28).
//!
//! One style property per fixture (MapLibre pattern, C6 pilot sizing): a
//! minimal scene renders with ONE property varied; the predicate asserts the
//! property's observable pixel effect. Self-test discipline (testing.md
//! dual-insurance rule): every predicate must DISTINGUISH good from a sabotaged variant — the
//! test renders both and asserts accept/reject, making each fixture a real
//! regression gate rather than a tautology.
//!
//! Construction shapes are copied verbatim from the existing verified tests
//! (strokes.rs frame_with / shadows.rs scene / clip.rs coefficient form /
//! multiview.rs pixel ViewportRect) — no new IR, no generator module (YAGNI).
//! New properties land as +1 fixture entry.
//!
//! SPEC-ANCHOR NOTE (review #5): this pilot is deliberately UNANCHORED —
//! the 10 fixtures lock probe-verified rendering behavior of EXISTING
//! clauses (WGPU-17/19/26/27/30 families) rather than introducing new
//! contract surface. If promoted to a named gate, add a clause + anchors.

#![allow(clippy::float_cmp)]

use visiaengine_render::{
    Camera, CameraRig, ClipSetup, DrawCommand, Frame, MeshDesc, PointMark, PointTableDesc,
    RenderBackend, ShadowBias, ShadowSetup, StrokeSeg, StrokeTableDesc, TableId, Viewport,
    ViewportRect,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 320;
const H: u32 = 240;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

fn px(img: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [
        img.rgba[i],
        img.rgba[i + 1],
        img.rgba[i + 2],
        img.rgba[i + 3],
    ]
}

fn count(img: &visiaengine_render_wgpu::OffscreenFrame, fam: impl Fn([u8; 4]) -> bool) -> u32 {
    let mut n = 0;
    for y in 0..H {
        for x in 0..W {
            if fam(px(img, x, y)) {
                n += 1;
            }
        }
    }
    n
}

/// Frame with the strokes.rs-verified camera shape (eye [0,-8,4], fov 60°).
fn frame_with(
    commands: Vec<DrawCommand>,
    shadow: Option<ShadowSetup>,
    clip: Option<ClipSetup>,
) -> Frame {
    let rig = CameraRig::look_at([0.0, -8.0, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap(),
        px_world_scale: 0.16,
        shadow,
        clip,
        edl: None,
        post: Vec::new(),
        commands,
    }
}

// ── shared scenes ──

/// Unit quad mesh at origin (facing +z), given material color.
/// stroke/point tables carry linear RGB (no alpha) — drop the 4th channel.
fn color3(c: [f32; 4]) -> [f32; 3] {
    [c[0], c[1], c[2]]
}

fn upload_quad(b: &mut HeadlessBackend, color: [f32; 4]) -> (TableId, TableId) {
    let verts = [
        [-1.0f32, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
    ];
    let normals = [[0.0f32, 0.0, 1.0]; 4];
    let idx = [0u32, 1, 2, 0, 2, 3];
    let mesh = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &verts,
            normals: &normals,
            indices: &idx,
        })
        .expect("mesh");
    let mat = b.create_material(color).expect("material");
    (mesh, mat)
}

fn upload_line(b: &mut HeadlessBackend, color: [f32; 4], width: f32) -> TableId {
    b.create_strokes(&StrokeTableDesc {
        data: &[StrokeSeg::new(
            [-1.2, 0.0, 0.0],
            [1.2, 0.0, 0.0],
            color3(color),
            width,
        )],
    })
    .expect("strokes")
}

fn upload_point(b: &mut HeadlessBackend, color: [f32; 4], radius: f32) -> TableId {
    b.create_points(&PointTableDesc {
        data: &[PointMark::new([0.0, 0.0, 0.0], color3(color), radius)],
    })
    .expect("points")
}

// ── fixture type ──

struct Fixture {
    name: &'static str,
    good: Box<dyn Fn(&mut HeadlessBackend) -> visiaengine_render_wgpu::OffscreenFrame>,
    bad: Box<dyn Fn(&mut HeadlessBackend) -> visiaengine_render_wgpu::OffscreenFrame>,
    check: fn(&visiaengine_render_wgpu::OffscreenFrame) -> bool,
}

fn run(f: &Fixture) {
    let mut b = HeadlessBackend::new(W, H).expect("adapter (lavapipe)");
    let img_good = (f.good)(&mut b);
    let img_bad = (f.bad)(&mut b);
    assert!(
        (f.check)(&img_good),
        "[{}] predicate must accept the good variant",
        f.name
    );
    assert!(
        !(f.check)(&img_bad),
        "[{}] predicate must reject the sabotaged variant (self-test)",
        f.name
    );
}

/// shadows.rs-verified ShadowSetup shape (light rig triplet contract REND-31).
const LIGHT_DIR: [f32; 3] = [0.35, 0.4, -0.85];

fn shadow_on() -> ShadowSetup {
    let n =
        (LIGHT_DIR[0] * LIGHT_DIR[0] + LIGHT_DIR[1] * LIGHT_DIR[1] + LIGHT_DIR[2] * LIGHT_DIR[2])
            .sqrt();
    let eye = [
        f64::from(LIGHT_DIR[0] / n) * 30.0,
        f64::from(LIGHT_DIR[1] / n) * 30.0,
        f64::from(LIGHT_DIR[2] / n) * 30.0 + 0.6,
    ];
    let rig = CameraRig::look_at(eye, [0.0, 0.0, 0.6], [0.0, 1.0, 0.0]);
    ShadowSetup {
        proj: rig
            .ortho_frame(10.0, W as f32, H as f32, 1.0, 80.0)
            .unwrap(),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        light_dir: LIGHT_DIR,
        size: 0.0,
        bias: ShadowBias {
            constant: -1.2,
            slope: -1.5,
        },
    }
}

fn draw_mesh_cmd(b: &mut HeadlessBackend, color: [f32; 4], commands: &mut Vec<DrawCommand>) {
    let (mesh, mat) = upload_quad(b, color);
    commands.push(DrawCommand::DrawMesh {
        mesh,
        material: mat,
        origin: [0.0; 3],
        transform: T4,
    });
}

fn fixtures() -> Vec<Fixture> {
    vec![
        // 1. fill-color: red quad vs blue quad
        Fixture {
            name: "fill-color",
            good: Box::new(|b| {
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                draw_mesh_cmd(b, [1.0, 0.0, 0.0, 1.0], &mut c);
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            bad: Box::new(|b| {
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                draw_mesh_cmd(b, [0.0, 0.0, 1.0, 1.0], &mut c);
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            check: |img| {
                let p = px(img, W / 2, H / 2);
                p[0] > 180 && p[1] < 80 && p[2] < 80
            },
        },
        // 2. fill-opacity: translucent vs opaque (blend attenuation present)
        Fixture {
            name: "fill-opacity",
            good: Box::new(|b| {
                let mut c = vec![DrawCommand::ClearColor {
                    rgba: [0.05, 0.05, 0.05, 1.0],
                }];
                draw_mesh_cmd(b, [1.0, 0.0, 0.0, 0.45], &mut c);
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            bad: Box::new(|b| {
                let mut c = vec![DrawCommand::ClearColor {
                    rgba: [0.05, 0.05, 0.05, 1.0],
                }];
                draw_mesh_cmd(b, [1.0, 0.0, 0.0, 1.0], &mut c);
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            check: |img| {
                // blended red over dark (probe-verified 2026-09-28): a=0.45
                // lands R≈145 (linear-domain blend vs material 207); band keeps
                // a conservative margin on the probe value.
                let p = px(img, W / 2, H / 2);
                (100..=180).contains(&p[0]) && p[1] < 60
            },
        },
        // 3. stroke-width: 9px vs 1px line
        Fixture {
            name: "stroke-width",
            good: Box::new(|b| {
                let t = upload_line(b, [1.0, 0.0, 0.0, 1.0], 9.0);
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                c.push(DrawCommand::DrawStrokes {
                    table: t,
                    origin: [0.0; 3],
                    transform: T4,
                });
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            bad: Box::new(|b| {
                let t = upload_line(b, [1.0, 0.0, 0.0, 1.0], 1.0);
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                c.push(DrawCommand::DrawStrokes {
                    table: t,
                    origin: [0.0; 3],
                    transform: T4,
                });
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            check: |img| {
                let reds = count(img, |p| p[0] > 180 && p[1] < 80 && p[2] < 80);
                reds >= 400
            },
        },
        // 4. stroke-color: green vs red line
        Fixture {
            name: "stroke-color",
            good: Box::new(|b| {
                let t = upload_line(b, [0.0, 1.0, 0.0, 1.0], 4.0);
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                c.push(DrawCommand::DrawStrokes {
                    table: t,
                    origin: [0.0; 3],
                    transform: T4,
                });
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            bad: Box::new(|b| {
                let t = upload_line(b, [1.0, 0.0, 0.0, 1.0], 4.0);
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                c.push(DrawCommand::DrawStrokes {
                    table: t,
                    origin: [0.0; 3],
                    transform: T4,
                });
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            check: |img| {
                let greens = count(img, |p| p[1] > 180 && p[0] < 80 && p[2] < 80);
                greens >= 100
            },
        },
        // 5. point-radius: r9 disc vs r2 dot
        Fixture {
            name: "point-radius",
            good: Box::new(|b| {
                let t = upload_point(b, [1.0, 0.0, 0.0, 1.0], 9.0);
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                c.push(DrawCommand::DrawPoints {
                    table: t,
                    origin: [0.0; 3],
                    transform: T4,
                });
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            bad: Box::new(|b| {
                let t = upload_point(b, [1.0, 0.0, 0.0, 1.0], 2.0);
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                c.push(DrawCommand::DrawPoints {
                    table: t,
                    origin: [0.0; 3],
                    transform: T4,
                });
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            check: |img| {
                // probe-verified 2026-09-28: r9 → 4672 red px, r2 → 232.
                let reds = count(img, |p| p[0] > 180 && p[1] < 80 && p[2] < 80);
                reds >= 2000
            },
        },
        // 6. point-color: blue vs red point
        Fixture {
            name: "point-color",
            good: Box::new(|b| {
                let t = upload_point(b, [0.2, 0.5, 1.0, 1.0], 9.0);
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                c.push(DrawCommand::DrawPoints {
                    table: t,
                    origin: [0.0; 3],
                    transform: T4,
                });
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            bad: Box::new(|b| {
                let t = upload_point(b, [1.0, 0.0, 0.0, 1.0], 9.0);
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                c.push(DrawCommand::DrawPoints {
                    table: t,
                    origin: [0.0; 3],
                    transform: T4,
                });
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            check: |img| {
                let blues = count(img, |p| p[2] > 180 && p[0] < 120);
                blues >= 100
            },
        },
        // 7. shadow-on: shadowed lit quad vs unshadowed
        Fixture {
            name: "shadow-on",
            good: Box::new(|b| {
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                draw_mesh_cmd(b, [0.8, 0.8, 0.85, 1.0], &mut c);
                b.render_to_pixels(&frame_with(c, Some(shadow_on()), None))
                    .unwrap()
            }),
            bad: Box::new(|b| {
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                draw_mesh_cmd(b, [0.8, 0.8, 0.85, 1.0], &mut c);
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            check: |img| {
                // probe-verified 2026-09-28: shadowed quad face dims into the
                // mid band (1320 px) vs flat-lit bright (1320 px) — count
                // split is the discriminator, not absolute darkness.
                let luma = |p: [u8; 4]| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]);
                let mid = count(img, |p| luma(p) >= 220 && luma(p) < 500);
                let bright = count(img, |p| luma(p) >= 500);
                mid > 1000 && bright < 500
            },
        },
        // 8. clear-color: teal clear vs red clear
        Fixture {
            name: "clear-color",
            good: Box::new(|b| {
                let c = vec![DrawCommand::ClearColor {
                    rgba: [0.1, 0.3, 0.5, 1.0],
                }];
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            bad: Box::new(|b| {
                let c = vec![DrawCommand::ClearColor {
                    rgba: [0.5, 0.1, 0.1, 1.0],
                }];
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            check: |img| {
                let [r, g, b, _] = px(img, 2, 2);
                g > r && b > r
            },
        },
        // 9. viewport-rect: second sub-viewport renders its own frame vs single
        // (multiview.rs verified shape: render_to_pixels_rects + per-rect frames)
        Fixture {
            name: "viewport-rect",
            good: Box::new(|b| {
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                draw_mesh_cmd(b, [1.0, 0.0, 0.0, 1.0], &mut c);
                let base = frame_with(c, None, None);
                let mut c2 = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                draw_mesh_cmd(b, [1.0, 0.0, 0.0, 1.0], &mut c2);
                let sub = frame_with(c2, None, None);
                let ra = ViewportRect::new(0, 0, W / 2, H / 2);
                b.render_to_pixels_rects(
                    &[(base, ViewportRect::new(0, 0, W, H)), (sub, ra)],
                    visiaengine_render_wgpu::MultiClearPolicy::FirstClearRestLoad,
                )
                .unwrap()
            }),
            bad: Box::new(|b| {
                // sabotage: sub-viewport renders a BLUE quad (position same,
                // color wrong) — the second-viewport surface exists but does
                // not carry the red family.
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                draw_mesh_cmd(b, [1.0, 0.0, 0.0, 1.0], &mut c);
                let base = frame_with(c, None, None);
                let mut c2 = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                draw_mesh_cmd(b, [0.0, 0.0, 1.0, 1.0], &mut c2);
                let sub = frame_with(c2, None, None);
                let ra = ViewportRect::new(0, 0, W / 2, H / 2);
                b.render_to_pixels_rects(
                    &[(base, ViewportRect::new(0, 0, W, H)), (sub, ra)],
                    visiaengine_render_wgpu::MultiClearPolicy::FirstClearRestLoad,
                )
                .unwrap()
            }),
            check: |img| {
                // probe-verified: full-frame quad leaves the (40..70)² corner
                // black; the sub-viewport's own centered quad covers it.
                let mut reds = 0u32;
                for y in 40..70u32 {
                    for x in 40..70u32 {
                        let p = px(img, x, y);
                        if p[0] > 180 && p[1] < 80 {
                            reds += 1;
                        }
                    }
                }
                reds > 50
            },
        },
        // 10. clip-plane: half-space clipped quad vs unclipped
        Fixture {
            name: "clip-plane",
            good: Box::new(|b| {
                let (mesh, mat) = upload_quad(b, [1.0, 0.0, 0.0, 1.0]);
                let clip = ClipSetup::new(&[[0.0, 1.0, 0.0, 0.0]]).expect("half clip");
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                c.push(DrawCommand::DrawMesh {
                    mesh,
                    material: mat,
                    origin: [0.0; 3],
                    transform: T4,
                });
                b.render_to_pixels(&frame_with(c, None, Some(clip)))
                    .unwrap()
            }),
            bad: Box::new(|b| {
                let mut c = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
                draw_mesh_cmd(b, [1.0, 0.0, 0.0, 1.0], &mut c);
                b.render_to_pixels(&frame_with(c, None, None)).unwrap()
            }),
            check: |img| {
                // probe-verified 2026-09-28: keep y>=0 (n=[0,1,0]) → reds ALL
                // in the screen-upper half (534) vs unclipped 534+786 split.
                // (Camera looks down; world +y projects to screen-up.)
                let mut lower = 0u32;
                let mut upper = 0u32;
                for y in 0..H {
                    for x in 0..W {
                        let p = px(img, x, y);
                        if p[0] > 180 && p[1] < 80 && p[2] < 80 {
                            if y < H / 2 {
                                upper += 1;
                            } else {
                                lower += 1;
                            }
                        }
                    }
                }
                upper > 300 && lower < 20
            },
        },
    ]
}

#[test]
fn property_golden_pilot_10_fixtures() {
    let all = fixtures();
    assert_eq!(all.len(), 10, "pilot = 10 properties");
    for f in &all {
        run(f);
    }
}

#[test]
fn property_golden_fixture_names_unique() {
    let names: std::collections::HashSet<_> = fixtures().iter().map(|f| f.name).collect();
    assert_eq!(names.len(), 10, "one property per fixture (no dupes)");
}
