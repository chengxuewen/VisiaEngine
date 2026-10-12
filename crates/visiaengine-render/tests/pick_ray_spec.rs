//! REND-21/22: analytic construction of screen pixels -> world rays (the interaction slice, [E3D:4xx]).
//! Dual entry signatures (persp uses rig.fov_y / ortho uses rig.zoom) -- projection mode lives at the Frame assembly layer,
//! the ray entries split modes explicitly, no magic numbers.

use visiaengine_render::{CameraRig, screen_to_ray_ortho, screen_to_ray_persp};

const W: f32 = 800.0;
const H: f32 = 600.0;

// spec: REND-21
#[test]
fn persp_center_and_corner_rays() {
    let rig = CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    // center -> straight down -z hitting the origin (origin = eye, dir normalized)
    let r = screen_to_ray_persp(&rig, W / 2.0, H / 2.0, W, H).unwrap();
    assert!((r.origin.x).abs() < 1e-9 && (r.origin.y).abs() < 1e-9);
    assert!((r.dir.z + 1.0).abs() < 1e-9);
    let ty = (rig.fov_y / 2.0).tan();
    let aspect = f64::from(W) / f64::from(H);
    let tx = ty * aspect;
    // top-left corner (0,0): NDC (-1,+1) -> x negative, y positive
    let c = screen_to_ray_persp(&rig, 0.0, 0.0, W, H).unwrap();
    assert!((c.dir.x / -c.dir.z + tx).abs() < 1e-12 && (c.dir.y / -c.dir.z - ty).abs() < 1e-12);
    // bottom-right corner: mirrored
    let d = screen_to_ray_persp(&rig, W, H, W, H).unwrap();
    assert!((d.dir.x / -d.dir.z - tx).abs() < 1e-12 && (d.dir.y / -d.dir.z + ty).abs() < 1e-12);
    // normalization
    let n = (d.dir.x * d.dir.x + d.dir.y * d.dir.y + d.dir.z * d.dir.z).sqrt();
    assert!((n - 1.0).abs() < 1e-12);
}

// spec: REND-22
#[test]
fn ortho_and_d7_far_field() {
    let mut rig = CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    rig.zoom = 2.0; // look_at defaults to 1.0 -- set the ortho half-width semantics explicitly
    // center ray: dir = -z, origin = eye
    let r = screen_to_ray_ortho(&rig, W / 2.0, H / 2.0, W, H).unwrap();
    assert!(r.dir.x.abs() < 1e-9 && r.dir.y.abs() < 1e-9 && (r.dir.z + 1.0).abs() < 1e-9);
    // top-left corner: right/up basis offsets (-hw, +hh), hw = zoom = 2 -> hh = 2*h/w = 1.5
    let c = screen_to_ray_ortho(&rig, 0.0, 0.0, W, H).unwrap();
    assert!((c.origin.x + 2.0).abs() < 1e-9 && (c.origin.y - 1.5).abs() < 1e-9);
    // D7 far field: eye = (1e7,2,10) straight top view -> the start shifts along, the direction unchanged (no precision disaster with all f64)
    let mut far = CameraRig::look_at([1e7, 2.0, 10.0], [1e7, 2.0, 0.0], [0.0, 1.0, 0.0]);
    far.zoom = 2.0;
    let f = screen_to_ray_ortho(&far, W / 2.0, H / 2.0, W, H).unwrap();
    assert!((f.origin.x - 1e7).abs() < 1e-6 && (f.origin.y - 2.0).abs() < 1e-9);
    assert!((f.dir.z + 1.0).abs() < 1e-12);
}
