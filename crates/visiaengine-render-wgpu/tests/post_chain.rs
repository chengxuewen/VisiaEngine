//! WGPU-34: post chain — OFF = byte-identical legacy canary; bloom/outline
//! each visibly change the frame; chain order composes deterministically.
//! Predicate discipline [PIT-8]: thresholds recorded from probes (see
//! comments at assertion sites).

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MeshDesc, PostEffect, RenderBackend,
    Viewport,
};
use visiaengine_render_wgpu::{HeadlessBackend, OffscreenFrame};

const W: u32 = 128;
const H: u32 = 128;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Ground + one bright quad (bloom fodder), oblique camera (E304 city-scene
/// family). Resources upload on the SAME backend that renders (single-owner —
/// table ids are backend-local).
fn scene(b: &mut HeadlessBackend) -> Vec<DrawCommand> {
    let ground = [
        [-6.0f32, -6.0, 0.0],
        [6.0, -6.0, 0.0],
        [6.0, 6.0, 0.0],
        [-6.0, 6.0, 0.0],
    ];
    let gm = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &ground,
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
        })
        .expect("ground");
    let gt = b
        .create_material_desc(&MaterialDesc {
            base_color: [0.22, 0.30, 0.24, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
        })
        .expect("gt");
    // Bright standing quad (luma above bloom threshold after sRGB encode).
    let bright = [
        [-1.0f32, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 3.0],
        [-1.0, 1.0, 3.0],
    ];
    let bm = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &bright,
            normals: &[[0.0, -1.0, 0.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
        })
        .expect("bright");
    let bt = b
        .create_material_desc(&MaterialDesc {
            base_color: [0.95, 0.90, 0.75, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
        })
        .expect("bt");
    vec![
        DrawCommand::ClearColor {
            rgba: [0.05, 0.07, 0.10, 1.0],
        },
        DrawCommand::DrawMesh {
            mesh: gm,
            material: gt,
            origin: [0.0; 3],
            transform: T4,
        },
        DrawCommand::DrawMesh {
            mesh: bm,
            material: bt,
            origin: [0.0; 3],
            transform: T4,
        },
    ]
}

fn frame(cmds: &[DrawCommand], post: Vec<PostEffect>) -> Frame {
    let rig = CameraRig::look_at([0.0, -14.0, 9.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let proj = rig
        .perspective(rig.fov_y as f32, W as f32 / H as f32, 0.1, 1000.0)
        .expect("proj");
    Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, W as f32 / H as f32, 0.1, 1000.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post,
        commands: cmds.to_vec(),
    }
}

fn diff_px(a: &OffscreenFrame, b: &OffscreenFrame) -> u32 {
    a.rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.rgba.as_chunks::<4>().0)
        .filter(|(x, y)| x[..3] != y[..3])
        .count() as u32
}

// spec: WGPU-34
// spec: REND-43
#[test]
fn post_off_bitwise() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    // Zero-effect law: empty post renders byte-identical across repeats.
    let off_a = b
        .render_to_pixels(&frame(&cmds, Vec::new()))
        .expect("off-a");
    let off_b = b
        .render_to_pixels(&frame(&cmds, Vec::new()))
        .expect("off-b");
    assert_eq!(off_a.rgba, off_b.rgba, "OFF canary drifted");
    // OFF after ON: per-frame switch — legacy path must stay exact.
    let _ = b
        .render_to_pixels(&frame(&cmds, vec![PostEffect::bloom(0.8).expect("bloom")]))
        .expect("bloom frame");
    let off_c = b
        .render_to_pixels(&frame(&cmds, Vec::new()))
        .expect("off-c");
    assert_eq!(off_c.rgba, off_a.rgba, "OFF-after-ON drifted");
    // Scene sanity: content actually rendered (not a clear-only frame).
    let lit: u32 = off_a
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| u32::from(px[0]) + u32::from(px[1]) + u32::from(px[2]) > 100)
        .count() as u32;
    assert!(lit > 1000, "scene must render content: lit={lit}");
}

// spec: WGPU-34
// spec: REND-43
#[test]
fn post_bloom_differs() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    let off = b.render_to_pixels(&frame(&cmds, Vec::new())).expect("off");
    let on = b
        .render_to_pixels(&frame(&cmds, vec![PostEffect::bloom(0.8).expect("bloom")]))
        .expect("on");
    let diff = diff_px(&off, &on);
    // Probe (lavapipe, 128² city scene, strength 0.8, THRESHOLD 0.25):
    // diff=432 px (bright quad face + halo ring). Floor 100 (−77%, PIT-8
    // conservative) — catches regression without pinning aesthetics.
    eprintln!("WGPU-34 bloom probe: diff={diff}");
    assert!(diff > 100, "bloom produced too few pixel deltas: {diff}");
}

// spec: WGPU-34
// spec: REND-43
#[test]
fn post_outline_differs() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    let off = b.render_to_pixels(&frame(&cmds, Vec::new())).expect("off");
    let on = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::outline(2.0).expect("outline")],
        ))
        .expect("on");
    let diff = diff_px(&off, &on);
    // Probe (lavapipe, 128² city scene, width 2.0): diff=678 px (silhouette
    // bands at ground-quad boundary + tower top edge). Floor 150 (−78%).
    eprintln!("WGPU-34 outline probe: diff={diff}");
    assert!(diff > 150, "outline produced too few pixel deltas: {diff}");
}

// spec: WGPU-34
// spec: REND-43
#[test]
fn post_chain_order() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    let off = b.render_to_pixels(&frame(&cmds, Vec::new())).expect("off");
    let bloom_first = b
        .render_to_pixels(&frame(
            &cmds,
            vec![
                PostEffect::bloom(0.8).expect("bloom"),
                PostEffect::outline(2.0).expect("outline"),
            ],
        ))
        .expect("bloom-then-outline");
    let outline_first = b
        .render_to_pixels(&frame(
            &cmds,
            vec![
                PostEffect::outline(2.0).expect("outline"),
                PostEffect::bloom(0.8).expect("bloom"),
            ],
        ))
        .expect("outline-then-bloom");
    // Both orders differ from off (non-degenerate); mutual order difference
    // is NOT asserted (may legitimately be subtle at v1 single-pass effects).
    let d1 = diff_px(&off, &bloom_first);
    let d2 = diff_px(&off, &outline_first);
    // Probe (lavapipe): [bloom,outline] diff=1038 px; [outline,bloom] diff=1640 px.
    eprintln!("WGPU-34 chain probe: bloom-first={d1} outline-first={d2}");
    assert!(d1 > 100, "chain[bloom,outline] too few deltas: {d1}");
    assert!(d2 > 100, "chain[outline,bloom] too few deltas: {d2}");
    // Determinism: same order twice = byte-identical.
    let again = b
        .render_to_pixels(&frame(
            &cmds,
            vec![
                PostEffect::outline(2.0).expect("outline"),
                PostEffect::bloom(0.8).expect("bloom"),
            ],
        ))
        .expect("again");
    assert_eq!(again.rgba, outline_first.rgba, "chain not deterministic");
}
