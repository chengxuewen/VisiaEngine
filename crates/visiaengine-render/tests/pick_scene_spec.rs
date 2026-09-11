//! REND-23/24：场景级拾取编排（bbox 即算剪除 + 最近命中 + transform 合成）。

use visiaengine_core::{Ray, Scene, Vec3};
use visiaengine_render::{MeshCandidate, PickHit, pick_meshes};

/// 单位立方体（±1），12 三角 CCW 外向。
#[must_use]
fn cube() -> (Vec<[f32; 3]>, Vec<u32>) {
    let p = |x: f32, y: f32, z: f32| [x, y, z];
    let positions = vec![
        p(-1., -1., -1.), p(1., -1., -1.), p(1., 1., -1.), p(-1., 1., -1.),
        p(-1., -1., 1.), p(1., -1., 1.), p(1., 1., 1.), p(-1., 1., 1.),
    ];
    // 6 面 ×2 三角（外向 CCW 约定与 MT 正面口一致）
    let idx: &[&[u32]] = &[
        &[0, 2, 1], &[0, 3, 2],         // -z 面（朝 -z 视点正对）
        &[4, 5, 6], &[4, 6, 7],         // +z
        &[0, 1, 5], &[0, 5, 4],         // -y
        &[2, 3, 7], &[2, 7, 6],         // +y
        &[1, 2, 6], &[1, 6, 5],         // +x
        &[3, 0, 4], &[3, 4, 7],         // -x
    ];
    (positions, idx.concat())
}

#[must_use]
fn translated(dx: f64, dy: f64, dz: f64) -> [[f64; 4]; 4] {
    let mut m = IDENTITY;
    m[3] = [dx, dy, dz, 1.0];
    m
}

const IDENTITY: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

// spec: REND-23
#[test]
fn nearest_wins_pruning_and_miss() {
    let (pos, idx) = cube();
    let mut scene = Scene::new();
    let e7 = scene.spawn();
    let e9 = scene.spawn();
    let near = translated(0.0, 0.0, 1.0); // 顶 z=2 面 @z∈[0,2]
    let far = translated(0.0, 0.0, 5.0); // z∈[4,6]
    let cands = [
        MeshCandidate { entity: e7, positions: &pos, indices: &idx, world: &near },
        MeshCandidate { entity: e9, positions: &pos, indices: &idx, world: &far },
    ];
    let ray = Ray { origin: Vec3::new(0.0, 0.0, 10.0), dir: Vec3::new(0.0, 0.0, -1.0) };
    let hit: PickHit = pick_meshes(ray, &cands).expect("中心必命中");
    assert_eq!(hit.entity, e7, "近件胜");
    assert!((hit.t - 8.0).abs() < 1e-9, "顶面 z=2 → t=8");
    assert!((hit.point.z - 2.0).abs() < 1e-9);
    // 移除近件 → 远件浮出
    let hit2 = pick_meshes(ray, &cands[1..]).expect("远件");
    assert_eq!(hit2.entity, e9);
    assert!((hit2.t - 4.0).abs() < 1e-9, "远件顶 z=6 → t=4");
    // 全旁路 → None
    let side = Ray { origin: Vec3::new(99.0, 99.0, 10.0), dir: Vec3::new(0.0, 0.0, -1.0) };
    assert!(pick_meshes(side, &cands).is_none());
}

// spec: REND-24
#[test]
fn transform_scale_and_empty_inputs() {
    let (pos, idx) = cube();
    let mut scene = Scene::new();
    let e1 = scene.spawn();
    let e2 = scene.spawn();
    // 2× 缩放（对角）：局部 ±1 → 世界 ±2，-z 面 z=-2
    let scaled = [
        [2.0, 0.0, 0.0, 0.0],
        [0.0, 2.0, 0.0, 0.0],
        [0.0, 0.0, 2.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let cands = [MeshCandidate { entity: e1, positions: &pos, indices: &idx, world: &scaled }];
    let ray = Ray { origin: Vec3::new(0.5, 0.5, 10.0), dir: Vec3::new(0.0, 0.0, -1.0) };
    let hit = pick_meshes(ray, &cands).expect("缩放件命中");
    assert!((hit.t - 12.0).abs() < 1e-9, "-z 面在 z=-2 → t=12");
    // 空几何候选不 panic（positions/indices 空）
    let empty = [MeshCandidate { entity: e2, positions: &[], indices: &[], world: &scaled }];
    assert!(pick_meshes(ray, &empty).is_none());
    assert!(pick_meshes(ray, &[]).is_none());
}
