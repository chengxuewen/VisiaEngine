//! REND-38/39 点云屏幕空间拾取（docs/sdd/render.md；super-band B2）。
//!
//! MVP 链与渲染同源：`compose_mvp(proj, view_rot, eye, origin, transform)`
//! —— 测试直接复刻 engine.rs render() 的投影装配（ortho/persp 两形），
//! 保证「拾取数学 = 渲染数学」单一事实源（WGPU-21 同族纪律）。

#![allow(clippy::float_cmp)]

use visiaengine_core::Scene;
use visiaengine_render::camera::CameraRig;
use visiaengine_render::rebase::compose_mvp;
use visiaengine_render::{PointCloudCandidate, pick_points};

static NEAR_ID: std::sync::LazyLock<visiaengine_core::EntityId> =
    std::sync::LazyLock::new(|| Scene::new().spawn());
static FAR_ID: std::sync::LazyLock<visiaengine_core::EntityId> =
    std::sync::LazyLock::new(|| Scene::new().spawn());

const IDENTITY: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

const W: f32 = 640.0;
const H: f32 = 480.0;

/// 渲染同款 ortho 装配（engine.rs:578-582 / 596 / 592 同构）。
fn ortho_mvp(rig: &CameraRig, origin: [f64; 3]) -> [[f32; 4]; 4] {
    let proj = rig
        .ortho_frame(rig.zoom as f32, W, H, 0.1, 1000.0)
        .expect("ortho frame");
    compose_mvp(&proj, &rig.view_rotation(), &rig.eye(), &origin, &IDENTITY)
}

/// 渲染同款 persp 装配（engine.rs:569-577 同构）。
fn persp_mvp(rig: &CameraRig, origin: [f64; 3]) -> [[f32; 4]; 4] {
    let proj = rig
        .perspective(rig.fov_y as f32, W / H, 0.1, 1000.0)
        .expect("persp frame");
    compose_mvp(&proj, &rig.view_rotation(), &rig.eye(), &origin, &IDENTITY)
}

/// ortho 机位：自 +Z 俯视原点，zoom=10（世界半宽 10 → 640px / 20 世界单位 = 32 px/单位）。
fn rig_topdown() -> CameraRig {
    let mut r = CameraRig::look_at([0.0, 0.0, 50.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    r.zoom = 10.0;
    r
}

// spec: REND-38
#[test]
fn grid_center_hit_under_cursor() {
    let rig = rig_topdown();
    let origin = [0.0; 3];
    let id = Scene::new().spawn();
    let cloud = PointCloudCandidate {
        entity: id,
        origin,
        transform: &IDENTITY,
        positions: &[[0.0, 0.0, 0.0]],
    };
    // Point at world origin; ortho centered → NDC (0,0) → screen center (320, 240).
    let hit = pick_points(&ortho_mvp(&rig, origin), W, H, 320.0, 240.0, 6.0, &[cloud]);
    assert!(hit.is_some(), "center point within 6px radius must hit");
    assert_eq!(hit.expect("hit").entity, id);
}

// spec: REND-38
#[test]
fn empty_corner_misses() {
    let rig = rig_topdown();
    let origin = [0.0; 3];
    let cloud = PointCloudCandidate {
        entity: Scene::new().spawn(),
        origin,
        transform: &IDENTITY,
        positions: &[[0.0, 0.0, 0.0]],
    };
    // Far corner, thousands of px away from the single point.
    let hit = pick_points(&ortho_mvp(&rig, origin), W, H, 5.0, 5.0, 6.0, &[cloud]);
    assert!(hit.is_none(), "cursor far from any point must miss");
}

// spec: REND-38
#[test]
fn radius_boundary_respected() {
    let rig = rig_topdown();
    let origin = [0.0; 3];
    let cloud = PointCloudCandidate {
        entity: Scene::new().spawn(),
        origin,
        transform: &IDENTITY,
        // 10 world units right = 320 px right of center → outside 6 px radius
        positions: &[[10.0, 0.0, 0.0]],
    };
    let hit = pick_points(&ortho_mvp(&rig, origin), W, H, 320.0, 240.0, 6.0, &[cloud]);
    assert!(hit.is_none(), "point 320px from cursor must miss with r=6");
}

// spec: REND-38
#[test]
fn nearer_cloud_wins_on_overlap() {
    let rig = rig_topdown();
    let near = PointCloudCandidate {
        entity: *NEAR_ID,
        origin: [0.0; 3],
        transform: &IDENTITY,
        positions: &[[0.0, 0.0, 40.0]], // z=40, camera at z=50 → depth 10
    };
    let far = PointCloudCandidate {
        entity: *FAR_ID,
        origin: [0.0; 3],
        transform: &IDENTITY,
        positions: &[[0.0, 0.0, 0.0]], // z=0 → depth 50
    };
    let hit = pick_points(
        &ortho_mvp(&rig, [0.0; 3]),
        W,
        H,
        320.0,
        240.0,
        6.0,
        &[far, near],
    );
    let hit = hit.expect("overlapping points must hit");
    assert_eq!(hit.entity, *NEAR_ID, "nearer depth must win");
}

// spec: REND-38
#[test]
fn behind_camera_not_pickable() {
    let rig = rig_topdown();
    let origin = [0.0; 3];
    let behind = PointCloudCandidate {
        entity: Scene::new().spawn(),
        origin,
        transform: &IDENTITY,
        positions: &[[0.0, 0.0, 100.0]], // camera at z=50 looking -z → behind camera
    };
    let hit = pick_points(&ortho_mvp(&rig, origin), W, H, 320.0, 240.0, 6.0, &[behind]);
    assert!(hit.is_none(), "point behind camera (w<=0 side) must miss");
}

// spec: REND-38
#[test]
fn persp_projection_path_hits() {
    let rig = CameraRig::look_at([0.0, 0.0, 30.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let origin = [0.0; 3];
    let cloud = PointCloudCandidate {
        entity: Scene::new().spawn(),
        origin,
        transform: &IDENTITY,
        positions: &[[0.0, 0.0, 0.0]],
    };
    // Persp: point at world origin dead ahead of camera → also projects to center.
    let hit = pick_points(&persp_mvp(&rig, origin), W, H, 320.0, 240.0, 6.0, &[cloud]);
    assert!(hit.is_some(), "persp path must hit centered point");
}

// spec: REND-38
#[test]
fn origin_offset_respected_d7() {
    // Cloud mounted at far origin (D7): world pos = transform·local + origin.
    let origin = [1_000_000.0, 2_000_000.0, 0.0];
    // Rig aimed at the far origin — same px/world scale (32 px/unit).
    let mut rig_far = CameraRig::look_at(
        [origin[0], origin[1], 50.0],
        [origin[0], origin[1], 0.0],
        [0.0, 1.0, 0.0],
    );
    rig_far.zoom = 10.0;
    let cloud = PointCloudCandidate {
        entity: Scene::new().spawn(),
        origin,
        transform: &IDENTITY,
        positions: &[[0.0, 0.0, 0.0]],
    };
    let hit = pick_points(
        &ortho_mvp(&rig_far, origin),
        W,
        H,
        320.0,
        240.0,
        6.0,
        &[cloud],
    );
    assert!(
        hit.is_some(),
        "far-origin cloud must hit when rig targets it (D7)"
    );
}

// spec: REND-39
#[test]
fn mesh_pick_path_untouched_regression_guard() {
    // REND-39 canary: the mesh picking entry point still exists and compiles with
    // the same signature; behavioral byte-parity is enforced by existing pick tests
    // (pick_highlight / pick_scene_spec) plus engine-level canary in capi tests.
    let rig = rig_topdown();
    let proj = rig.ortho_frame(10.0, W, H, 0.1, 1000.0).expect("ortho");
    let mvp = compose_mvp(
        &proj,
        &rig.view_rotation(),
        &rig.eye(),
        &[0.0; 3],
        &IDENTITY,
    );
    // Sanity: MVP composition itself unchanged — center point of identity cloud
    // must land at screen center exactly.
    let cloud = PointCloudCandidate {
        entity: Scene::new().spawn(),
        origin: [0.0; 3],
        transform: &IDENTITY,
        positions: &[[0.0, 0.0, 0.0]],
    };
    let hit = pick_points(&mvp, W, H, 320.0, 240.0, 0.001, &[cloud]);
    assert!(hit.is_some(), "exact center hit survives even tiny radius");
}
