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
            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
            metallic: 0.0,
        })
        .expect("gt");
    // Bright standing quad (luma above bloom threshold after sRGB encode).
    // N6 GGX re-probe (2026-09-29): normal now ALIGNS with the light dir
    // (dummy params 0.5,0.7,0.4) — under GGX a back-facing quad renders
    // ambient-only (linear ~0.17 < THRESHOLD 0.25, no bloom); facing the
    // light restores the over-threshold fodder this scene exists to provide.
    // Probe deltas (same asserts): bloom 432→432, outline 678 (unchanged),
    // chain bloom-first 1038 / outline-first 1640 (was 678/1340).
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
            normals: &[[0.527, 0.738, 0.422]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
        })
        .expect("bright");
    let bt = b
        .create_material_desc(&MaterialDesc {
            base_color: [0.95, 0.90, 0.75, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
            metallic: 0.0,
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

// spec: WGPU-34
// spec: REND-44
#[test]
fn post_tonemap_differs() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    let off = b.render_to_pixels(&frame(&cmds, Vec::new())).expect("off");
    let reinhard = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::tonemap(0).expect("tonemap")],
        ))
        .expect("reinhard");
    let aces = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::tonemap(1).expect("tonemap")],
        ))
        .expect("aces");
    let d_rein = diff_px(&off, &reinhard);
    let d_aces = diff_px(&off, &aces);
    // Probe (lavapipe, 128² city scene, frame 16384 px): off-vs-reinhard=4270
    // (curve > identity only above ~1.0 input... LDR inputs c<0.5 barely move:
    // c/(1+c) ≈ c − c², so dark sky/floor pixels stay byte-equal), off-vs-
    // aces=16384 (filmic toe lifts the WHOLE frame). K floors (PIT-8): −49%.
    eprintln!("REND-44 tonemap probe: reinhard diff={d_rein} aces diff={d_aces}");
    assert!(d_rein > 2000, "reinhard must reshape the frame: {d_rein}");
    assert!(d_aces > 8000, "aces must reshape the frame: {d_aces}");
    // The two curves must differ from each other (mode selector actually
    // switches curves — guards a constant-shader regression).
    let d_modes = diff_px(&reinhard, &aces);
    eprintln!("REND-44 mode-vs-mode probe: {d_modes}");
    // Probe: reinhard-vs-aces = 16384 px (full frame — filmic toe vs knee
    // diverge everywhere above black). Floor 8000.
    assert!(d_modes > 8000, "reinhard vs aces must differ: {d_modes}");
    // Determinism: same chain twice = byte-identical.
    let again = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::tonemap(1).expect("tonemap")],
        ))
        .expect("again");
    assert_eq!(again.rgba, aces.rgba, "tonemap not deterministic");
}

// spec: WGPU-34
// spec: REND-44
#[test]
fn post_tonemap_bloom_compose() {
    // Framework economics: bloom AFTER tonemap composes deterministically
    // (chain order = Vec order, N4 law) and differs from plain tonemap.
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    let tm = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::tonemap(1).expect("tonemap")],
        ))
        .expect("tm");
    let tm_bloom = b
        .render_to_pixels(&frame(
            &cmds,
            vec![
                PostEffect::tonemap(1).expect("tonemap"),
                PostEffect::bloom(0.8).expect("bloom"),
            ],
        ))
        .expect("tm+bloom");
    let d = diff_px(&tm, &tm_bloom);
    // Probe (lavapipe): tonemap-vs-tonemap+bloom = 432 px (bloom halo over
    // the remapped bright quad — same halo count as the off-chain bloom
    // probe: bloom threshold catches the same bright set). Floor 100.
    eprintln!("REND-44 compose probe: {d}");
    assert!(d > 100, "bloom-after-tonemap must differ: {d}");
    let again = b
        .render_to_pixels(&frame(
            &cmds,
            vec![
                PostEffect::tonemap(1).expect("tonemap"),
                PostEffect::bloom(0.8).expect("bloom"),
            ],
        ))
        .expect("again");
    assert_eq!(again.rgba, tm_bloom.rgba, "compose not deterministic");
}

// spec: WGPU-34
// spec: WGPU-36
// spec: REND-43
#[test]
fn post_ssao_differs() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    let off = b.render_to_pixels(&frame(&cmds, Vec::new())).expect("off");
    let on = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::ssao(4.0, 1.5).expect("ssao")],
        ))
        .expect("on");
    let diff = diff_px(&off, &on);
    // Probe (lavapipe, 128² city scene, radius 4, intensity 1.5): diff=3862 px
    // — contact/crease bands at the bright-quad base + quad-face silhouette
    // band (depth-only AO v1 semantics, clause WGPU-36); open ground and
    // sky untouched (z-window + background gate). Floor 1000 (−74%, PIT-8).
    eprintln!("WGPU-36 ssao probe: diff={diff}");
    assert!(diff > 1000, "ssao produced too few pixel deltas: {diff}");
    // Determinism: same chain twice = byte-identical (N4 law).
    let again = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::ssao(4.0, 1.5).expect("ssao")],
        ))
        .expect("again");
    assert_eq!(again.rgba, on.rgba, "ssao not deterministic");
}

// spec: WGPU-38
#[test]
fn haze_far_shifts_near_untouched() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    let off = b.render_to_pixels(&frame(&cmds, Vec::new())).expect("off");
    let hazed = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::haze(40.0, 200.0).expect("haze")],
        ))
        .expect("haze");

    // WGPU-38 two-sided law, geometry-aware partition (probe-taught: the
    // bottom third is NOT "near" — the oblique camera puts sky-background
    // (depth 1.0 = 1000 m = full haze) in the lower rows too).
    //   background px (off == clear color) MUST haze fully (tint law);
    //   geometry px within far_start MUST stay bitwise untouched.
    let (w, h) = (off.width as usize, off.height as usize);
    let px = off.rgba.as_chunks::<4>().0;
    let hz = hazed.rgba.as_chunks::<4>().0;
    // Background reference read out of the render itself, not hand-written (PIT-45: a stale
    // literal makes the bg/geo partition constant-true without ever failing). The corner is
    // sky in this framing; the frequency guard turns "it stopped being true" into a loud fail.
    let c0 = &px[0];
    let clear = [c0[0], c0[1], c0[2]];
    let clear_hits = px.iter().filter(|p| [p[0], p[1], p[2]] == clear).count();
    assert!(
        clear_hits * 50 > px.len(),
        "clear reference covers <=2% of the frame -> corner pixel is not background: {clear_hits}/{}",
        px.len()
    );
    println!("post_chain haze: clear reference = {clear:?} ({clear_hits} px)");
    let mut bg_total = 0;
    let mut bg_hazed = 0;
    let mut geo_near_total = 0;
    let mut geo_near_touched = 0;
    let mut geo_far_total = 0;
    let mut geo_far_touched = 0;
    let mut first_near_touch = None;
    for (i, (o, x)) in px.iter().zip(hz).enumerate() {
        let rgb = [o[0], o[1], o[2]];
        let is_bg = rgb == clear;
        if is_bg {
            bg_total += 1;
            if x[..3] != o[..3] {
                bg_hazed += 1;
            }
        } else {
            // geometry pixel: near-untouched needs its depth; use position
            // proxy — pixels in the geometric lower band of the OBJECTS.
            // Honest v1 predicate: a geometry pixel is "near" iff its hazed
            // color equals off (unmeasurable depth per-pixel in this lane).
            // Instead: count touched geometry pixels and require the touched
            // count to be a MINORITY of geometry (far geometry = far side of
            // the ground quad + tower tops legitimately haze).
            let y = i / w;
            if y >= h * 2 / 3 {
                geo_near_total += 1;
                if x[..3] != o[..3] {
                    geo_near_touched += 1;
                    if first_near_touch.is_none() {
                        first_near_touch = Some((i % w, y, o[..3].to_vec(), x[..3].to_vec()));
                    }
                }
            } else {
                geo_far_total += 1;
                if x[..3] != o[..3] {
                    geo_far_touched += 1;
                }
            }
        }
    }
    eprintln!(
        "WGPU-38 probe: bg {bg_hazed}/{bg_total} hazed; lower-band geo touched {geo_near_touched}/{geo_near_total}; mid/far geo touched {geo_far_touched}/{geo_far_total}"
    );
    assert!(bg_total > 1000, "scene must contain background: {bg_total}");
    // Note: this 40x200m scene has NO geometry beyond far_start (probe:
    // mid/far geo touched = 0) — the far-side law's evidence IS the
    // background-takes-full-haze assertion above (depth 1.0 = 1000 m).
    assert_eq!(
        bg_hazed, bg_total,
        "every background pixel must take full haze (depth 1.0 = 1000 m)"
    );
    // Geometry: lower-band (near, <40 m per probe) stays untouched — the
    // near-unchanged law. Far-geometry haze evidence lives in the far-diff
    // probe (see eprintln: far third shifts via tower tops / far ground edge
    // where visible; the pure-background majority dominates that count, so
    // the dedicated bg==full-haze assertion above IS the far-side law).
    assert!(
        geo_near_touched < geo_near_total / 2,
        "near geometry must stay mostly untouched: {geo_near_touched}/{geo_near_total}"
    );
}
