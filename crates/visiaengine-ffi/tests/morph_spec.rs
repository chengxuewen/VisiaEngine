//! N2.1 (REND-41): projection morph — endpoint bitwise canaries + mid probes.
//!
//! Contract: morph t=0 produces byte-identical frames to the discrete ortho
//! path; t=1 to the discrete persp path; mid values are well-formed (near
//! mesh occludes far). Probes printed before pins (PIT-8).
//!
//! Drives Engine directly (rs-only band, decision card A — no C entry).

use visiaengine::{Engine, Proj};

const W: u32 = 160;
const H: u32 = 120;

/// Engine with a scene: unit quad near (red, z=-2) + far (blue, z=-4).
fn engine_scene() -> Engine {
    let mut e = Engine::new_headless(W, H).expect("headless engine");
    let quad = |e: &mut Engine, color: [f32; 4], z: f64| {
        e.add_mesh(
            &[
                [-1.0f32, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0],
            ],
            None,
            &[0u32, 1, 2, 0, 2, 3],
            color,
            [0.0, 0.0, z],
            1.0,
            0.0,
        )
        .expect("quad");
    };
    quad(&mut e, [1.0, 0.0, 0.0, 1.0], -2.0);
    quad(&mut e, [0.0, 0.0, 1.0, 1.0], -4.0);
    e
}

fn bytes(e: &mut Engine) -> Vec<u8> {
    e.render().expect("render");
    e.frame_rgba().expect("frame cache").to_vec()
}

// spec: REND-41
#[test]
fn morph_endpoints_bitwise_equal_discrete_paths() {
    // Discrete references (morph = None).
    let mut e = engine_scene();
    e.set_projection(Proj::Ortho);
    let ortho_bytes = bytes(&mut e);
    let mut e = engine_scene();
    e.set_projection(Proj::Persp);
    let persp_bytes = bytes(&mut e);
    assert_ne!(ortho_bytes, persp_bytes, "sanity: projections differ");

    // Morph endpoints must reproduce the discrete bytes exactly.
    let mut e = engine_scene();
    e.set_morph_time(Some(0.0));
    let t0 = bytes(&mut e);
    let mut e = engine_scene();
    e.set_morph_time(Some(1.0));
    let t1 = bytes(&mut e);

    assert_eq!(t0, ortho_bytes, "t=0 must be byte-identical to ortho path");
    assert_eq!(t1, persp_bytes, "t=1 must be byte-identical to persp path");
}

// spec: REND-41
#[test]
fn morph_mid_is_well_formed_and_occlusion_holds() {
    // Probe (2026-09-28): counts printed on first run, bands pinned after.
    let mut e = engine_scene();
    e.set_morph_time(Some(0.5));
    let mid = bytes(&mut e);
    let reds = mid
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > 150 && p[1] < 90 && p[2] < 90)
        .count();
    let blues = mid
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[2] > 150 && p[0] < 90 && p[1] < 90)
        .count();
    println!(
        "PROBE mid t=0.5: reds={reds} blues={blues} total={}",
        mid.len() / 4
    );
    // Occlusion law: near (red) occludes far (blue).
    assert!(reds > 100, "near mesh visible at mid-morph");
    // Background = clear family present (frame not degenerate).
    let bg = mid
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] <= 20 && p[1] <= 26 && p[2] <= 36)
        .count();
    assert!(bg > 0, "clear pixels present (frame not degenerate)");
}

// spec: REND-41
#[test]
fn morph_clamps_slider_domain() {
    // Slider semantics: t>1 clamps to persp bytes; NaN clamps to ortho bytes.
    let mut e = engine_scene();
    e.set_morph_time(Some(2.5));
    let t_clamped = bytes(&mut e);
    let mut e = engine_scene();
    e.set_morph_time(Some(1.0));
    let t1 = bytes(&mut e);
    assert_eq!(t_clamped, t1, "t>1 clamps to 1 (persp bytes)");

    let mut e = engine_scene();
    e.set_morph_time(Some(f64::NAN));
    let t_nan = bytes(&mut e);
    let mut e = engine_scene();
    e.set_morph_time(Some(0.0));
    let t0 = bytes(&mut e);
    assert_eq!(t_nan, t0, "NaN clamps to 0 (ortho bytes)");
}
