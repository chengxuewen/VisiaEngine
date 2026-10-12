//! REND-36/WGPU-27/28: the five gates of the transparency band ((1) = the V0 blending-domain ruling gate -- prediction flips = halt the band for re-presentation).
//! The top-view ortho exact road; expected values = linear-domain blending + hardware encoding chain (the CORE-16 self-consistent prediction).

use visiaengine_io_text::{FontFace, GLYPH_ATLAS_PX, GlyphCache, layout};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, LabelMark, LabelTableDesc, MeshDesc, RenderBackend,
    Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 64;
const H: u32 = 64;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
const FIX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../resources/data/DejaVuSans.ttf"
));

fn quad(b: &mut HeadlessBackend, s: f32, _z: f32) -> u64 {
    b.create_mesh(&MeshDesc {
        positions: &[[-s, -s, 0.0], [s, -s, 0.0], [s, s, 0.0], [-s, s, 0.0]],
        normals: &[[0.0, 0.0, 1.0]; 4],
        indices: &[0, 1, 2, 0, 2, 3],
        uv: &[],
    })
    .expect("quad")
}

fn mat(b: &mut HeadlessBackend, rgba: [f32; 4]) -> u64 {
    b.create_material(rgba).expect("mat")
}

fn top_frame(commands: Vec<DrawCommand>) -> Frame {
    let rig = CameraRig::look_at([0.0, 0.0, 50.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(20.0, 20.0, 0.5, 200.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .ortho_frame(20.0, W as f32, H as f32, 0.5, 200.0)
            .expect("p"),
        px_world_scale: 0.625,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands,
    }
}

fn px(img: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * W + x) * 4) as usize;
    [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]]
}
fn near(v: u8, t: u8, tol: u8) -> bool {
    v.abs_diff(t) <= tol
}
fn at(img: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> [u8; 3] {
    px(img, x, y)
}

/// (1) The V0 ruling gate: a black-background opaque plate + a 50% red plate -> linear domain = (188,0,0); encoding domain = (128,0,0).
#[test]
// spec: WGPU-27
fn blend_domain_verdict_and_linear_mix() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let base = quad(&mut b, 20.0, 0.0);
    let plate = quad(&mut b, 8.0, 5.0);
    let black = mat(&mut b, [0.0, 0.0, 0.0, 1.0]);
    let red50 = mat(&mut b, [1.0, 0.0, 0.0, 0.5]);
    let dz = |z: f64| [0.0, 0.0, z];
    let img = b
        .render_to_pixels(&top_frame(vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh: base,
                material: black,
                origin: dz(0.0),
                transform: T4,
            },
            DrawCommand::DrawMesh {
                mesh: plate,
                material: red50,
                origin: dz(5.0),
                transform: T4,
            },
        ]))
        .expect("render");
    let c = at(&img, 32, 32);
    // the linear-domain prediction = encode(0.5*lin(1)*shade0.627) = 151; the encoding domain = 83 (a single-value gap of 68 decides the case)
    println!("V0 VERDICT center={c:?} linear-prediction=(151,0,0) encoding-prediction=(83,0,0)");
    assert!(
        near(c[0], 151, 8) && c[1] < 16 && c[2] < 16,
        "the blending-domain predictions both missed: {c:?} (overturn = halt the band for re-presentation)"
    );
}

/// (2) Sort activity bidirectionally: depth swap -> result flips; submission swap -> result does not flip.
#[test]
// spec: WGPU-28
fn painter_order_alive_both_directions() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let base = quad(&mut b, 20.0, 0.0);
    let (g_n, r_f) = (quad(&mut b, 8.0, 7.0), quad(&mut b, 8.0, 5.0));
    let (r_n, g_f) = (quad(&mut b, 8.0, 7.0), quad(&mut b, 8.0, 5.0));
    let black = mat(&mut b, [0.0, 0.0, 0.0, 1.0]);
    let red = mat(&mut b, [1.0, 0.0, 0.0, 0.5]);
    let green = mat(&mut b, [0.0, 1.0, 0.0, 0.5]);
    let dm = |mesh: u64, material: u64, z: f64| DrawCommand::DrawMesh {
        mesh,
        material,
        origin: [0.0, 0.0, z],
        transform: T4,
    };
    // configuration A: camera z = 50 looking down -> z7 = near/red, z5 = far -> green (near) drawn later = (110,151,0)
    let a = b
        .render_to_pixels(&top_frame(vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            dm(base, black, 0.0),
            dm(g_n, green, 7.0),
            dm(r_f, red, 5.0),
        ]))
        .expect("A");
    // configuration B: depths swapped (green near, red far)
    let bb = b
        .render_to_pixels(&top_frame(vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            dm(base, black, 0.0),
            dm(r_n, red, 7.0),
            dm(g_f, green, 5.0),
        ]))
        .expect("B");
    // configuration C: A's submission swapped (red first then green, depths unchanged)
    let c = b
        .render_to_pixels(&top_frame(vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            dm(base, black, 0.0),
            dm(r_f, red, 5.0),
            dm(g_n, green, 7.0),
        ]))
        .expect("C");
    let (ca, cb, cc) = (at(&a, 32, 32), at(&bb, 32, 32), at(&c, 32, 32));
    println!("ORDER A(red-near)={ca:?} B(green-near)={cb:?} C(submit-swap)={cc:?}");
    assert!(
        near(ca[0], 110, 10) && near(ca[1], 151, 10),
        "A expected (110,151,0) got {ca:?}"
    );
    assert!(
        near(cb[0], 151, 10) && near(cb[1], 110, 10),
        "B must flip on depth swap to (151,110,0) got {cb:?}"
    );
    assert_eq!(
        ca, cc,
        "a submission swap must not flip (the sort truly locks the depth order)"
    );
}

/// (3) The no-depth-write lock: two gray plates at the same depth see each other (write ON would reject the later piece via Less -> single layer).
#[test]
// spec: WGPU-27
fn transparent_does_not_write_depth() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let base = quad(&mut b, 20.0, 0.0);
    let g1 = quad(&mut b, 8.0, 5.0);
    let g2 = quad(&mut b, 8.0, 5.0);
    let black = mat(&mut b, [0.0, 0.0, 0.0, 1.0]);
    let gray = mat(&mut b, [0.5, 0.5, 0.5, 0.5]);
    let dm = |mesh: u64, material: u64, z: f64| DrawCommand::DrawMesh {
        mesh,
        material,
        origin: [0.0, 0.0, z],
        transform: T4,
    };
    let img = b
        .render_to_pixels(&top_frame(vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            dm(base, black, 0.0),
            dm(g1, gray, 5.0),
            dm(g2, gray, 5.0),
        ]))
        .expect("render");
    let c = at(&img, 32, 32);
    // the prediction including shade: two layers = encode(0.5*L + 0.5*(0.5L)) = 89; write-ON single layer = encode(0.5L) = 73
    println!("DEPTHWRITE-OFF center={c:?} (two layers = 89, the single-layer control = 73)");
    assert!(
        c[0] > 84 && c[0] < 95,
        "the two-layer mutual visibility absent = the transparent pipeline is writing depth: {c:?}"
    );
}

/// (4) The material a == 1 canary: rides the old path pure overlay (no blending trace) + zero touched by the sort.
#[test]
// spec: REND-36
fn opaque_material_pure_overwrite() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let base = quad(&mut b, 20.0, 0.0);
    let plate = quad(&mut b, 8.0, 5.0);
    let black = mat(&mut b, [0.0, 0.0, 0.0, 1.0]);
    let red_op = mat(&mut b, [1.0, 0.0, 0.0, 1.0]);
    let img = b
        .render_to_pixels(&top_frame(vec![
            DrawCommand::ClearColor {
                rgba: [0.2, 0.2, 0.2, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh: base,
                material: black,
                origin: [0.0, 0.0, 0.0],
                transform: T4,
            },
            DrawCommand::DrawMesh {
                mesh: plate,
                material: red_op,
                origin: [0.0, 0.0, 5.0],
                transform: T4,
            },
        ]))
        .expect("render");
    let c = at(&img, 32, 32);
    // a == 1 pure overlay = the opaque pipeline's current values (blending traces must be distinguishable below half).
    // N6 GGX reprobe (2026-09-29): 207->198 (the GGX diffuse (1-F)*intensity chain + the IBL-lite
    // ambient re-set the brightness; the double-sided band +/- 8 kept). The g/b upper bound 10->20: dielectric specular F0 = 0.04 gray
    // highlight leaks physically white light in the g/b channel band ~ 14 (correct PBR behavior).
    // is still guarded independently by the r-band +/- 8).
    assert!(
        near(c[0], 198, 8) && c[1] <= 20 && c[2] <= 20,
        "a==1 must be a pure overlay with no blending trace: {c:?}"
    );
}

/// (5) Labels always end on top: the label pixels inside the water plate are not diluted.
#[test]
// spec: WGPU-28
fn labels_persist_on_top_of_water() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let base = quad(&mut b, 20.0, 0.0);
    let water = quad(&mut b, 8.0, 5.0);
    let black = mat(&mut b, [0.0, 0.0, 0.0, 1.0]);
    let w50 = mat(&mut b, [1.0, 0.0, 0.0, 0.5]);
    let face = FontFace::from_bytes(FIX).expect("face");
    let mut cache = GlyphCache::new();
    let (quads, pen) = layout("T", &face, &mut cache, 30.0);
    let marks: Vec<LabelMark> = quads
        .iter()
        .map(|q| {
            LabelMark::new(
                [0.0, 0.0, 0.2],
                [1.0, 1.0, 1.0, 1.0],
                [q.uv0[0], q.uv0[1], q.uv1[0], q.uv1[1]],
                [
                    q.size_px[0],
                    q.size_px[1],
                    q.top_left_px[0] - pen / 2.0,
                    q.top_left_px[1],
                ],
            )
        })
        .collect();
    let tbl = b
        .create_labels(&LabelTableDesc { data: &marks })
        .expect("labels");
    b.set_glyph_atlas(cache.pixels(), GLYPH_ATLAS_PX, GLYPH_ATLAS_PX)
        .expect("atlas");
    let dm = |mesh: u64, material: u64, z: f64| DrawCommand::DrawMesh {
        mesh,
        material,
        origin: [0.0, 0.0, z],
        transform: T4,
    };
    let img = b
        .render_to_pixels(&top_frame(vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            dm(base, black, 0.0),
            dm(water, w50, 5.0),
            DrawCommand::DrawLabels {
                table: tbl,
                origin: [0.0; 3],
                transform: T4,
            },
        ]))
        .expect("render");
    let white = (0..H)
        .flat_map(|y| (0..W).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let c = px(&img, *x, *y);
            c[0] > 245 && c[1] > 245 && c[2] > 245
        })
        .count();
    assert!(
        white > 30,
        "the always-top white label is still present amid the ink water, got {white}"
    );
    // the top-center pixel = pure white (not mixed with the red water)
    let mut hit = 0;
    'outer: for y in 0..H {
        for x in 0..W {
            if px(&img, x, y) == [255, 255, 255] {
                hit += 1;
                if hit > 5 {
                    break 'outer;
                }
            }
        }
    }
    assert!(
        hit > 5,
        "the label white core must reach 255 (Always + last order = never diluted)"
    );
}
