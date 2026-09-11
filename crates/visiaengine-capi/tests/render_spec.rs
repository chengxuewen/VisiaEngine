//! CAPI-04：headless 出图字节（I2）——装载→渲染→回读→像素断言全链过 C ABI。

use std::ffi::CString;
use visiaengine::{
    MISS, VE_ERR_SIZE, visiaengine_create_headless, visiaengine_destroy, visiaengine_entity_at,
    visiaengine_entity_count, visiaengine_load_geojson, visiaengine_load_gltf, visiaengine_pick,
    visiaengine_readback, visiaengine_render, visiaengine_viewport,
};

fn fixture(name: &str) -> CString {
    CString::new(format!(
        "{}/../../resources/data/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

// spec: CAPI-04
#[test]
fn headless_load_render_readback_pipeline() {
    let ve = visiaengine_create_headless(160, 120);
    assert_ne!(ve, 0);
    // I1 占位语义已退场：装载实装 → 0
    assert_eq!(
        visiaengine_load_gltf(ve, fixture("twoprim.glb").as_ptr()),
        0
    );
    assert_eq!(
        visiaengine_entity_count(ve),
        2,
        "twoprim=2 件（GLTF-01 家族事实）"
    );
    assert_eq!(visiaengine_render(ve), 0);
    // 缓冲量值归因（CAPI-05 -5 专属运行时尺寸）
    assert_eq!(
        visiaengine_readback(ve, std::ptr::null_mut(), 0),
        VE_ERR_SIZE
    );
    let mut small = vec![0u8; 100];
    assert_eq!(
        visiaengine_readback(ve, small.as_mut_ptr(), small.len() as u64),
        VE_ERR_SIZE
    );
    let mut buf = vec![0u8; 160 * 120 * 4];
    assert_eq!(
        visiaengine_readback(ve, buf.as_mut_ptr(), buf.len() as u64),
        0
    );
    let mid = ((60 * 160 + 80) * 4) as usize;
    let lum = buf[mid] as u32 + buf[mid + 1] as u32 + buf[mid + 2] as u32;
    assert!(
        lum > 40,
        "中心像素非背景，亮度={lum} rgba={:?}",
        &buf[mid..mid + 4]
    );
    // pick 经同一帧几何：中心命中实体句柄 ∈ entity_at 值域
    let hit = visiaengine_pick(ve, 80.0, 60.0);
    assert_ne!(hit, MISS, "中心应命中");
    assert!(
        hit == visiaengine_entity_at(ve, 0) || hit == visiaengine_entity_at(ve, 1),
        "命中句柄必等于载入序某件（代际稳定）"
    );
    // viewport 变更→重渲染出图尺寸合法
    assert_eq!(visiaengine_viewport(ve, 200, 150), 0);
    assert_eq!(visiaengine_render(ve), 0);
    let mut buf2 = vec![0u8; 200 * 150 * 4];
    assert_eq!(
        visiaengine_readback(ve, buf2.as_mut_ptr(), buf2.len() as u64),
        0
    );
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-04
#[test]
fn geo_ramp_scene_renders_colored_pixels() {
    // heights.geojson（GEO-20 ramp 三件套）过 capi：三峰色族在像素面复现
    let ve = visiaengine_create_headless(256, 256);
    let geo = fixture("heights.geojson");
    assert_eq!(visiaengine_load_geojson(ve, geo.as_ptr()), 0);
    assert_eq!(visiaengine_entity_count(ve), 4, "4 件（含无列回退件）");
    assert_eq!(visiaengine_render(ve), 0);
    let mut buf = vec![0u8; 256 * 256 * 4];
    assert_eq!(
        visiaengine_readback(ve, buf.as_mut_ptr(), buf.len() as u64),
        0
    );
    // 非背景像素计数（ramp 着色的直接可见证据；逐像素色族断言归 WGPU 系 golden 域）
    let fg = buf
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| (p[0] as u32 + p[1] as u32 + p[2] as u32) > 60)
        .count();
    assert!(fg > 2000, "geo 面片像素不足: {fg}");
    visiaengine_destroy(ve);
}

// spec: CAPI-05
#[test]
fn wheel_zoom_changes_ortho_frame() {
    // 行为路径选 geo ortho 场景：twoprim persp 相机位于 orbit 极点（pitch=90°
    // →cp=0，yaw 退化）且 persp 投影不吃 zoom——换镜头几何会测到假阴性，
    // 故用 zoom 真实驱动像素的 ortho 面（WHEEL=共享 zoom 乘性 [E3D:B6]）。
    use visiaengine::{
        KIND_WHEEL, VE_ERR_ARG, VeInput, visiaengine_load_geojson, visiaengine_on_input,
    };
    let ve = visiaengine_create_headless(160, 120);
    assert_eq!(
        visiaengine_load_geojson(ve, fixture("heights.geojson").as_ptr()),
        0
    );
    let fg_of = |buf: &[u8]| -> usize {
        buf.as_chunks::<4>()
            .0
            .iter()
            .filter(|p| (p[0] as u32 + p[1] as u32 + p[2] as u32) > 60)
            .count()
    };
    let frame = |buf: &mut Vec<u8>| {
        assert_eq!(visiaengine_render(ve), 0);
        assert_eq!(
            visiaengine_readback(ve, buf.as_mut_ptr(), buf.len() as u64),
            0
        );
    };
    let mut a = vec![0u8; 160 * 120 * 4];
    frame(&mut a);
    let before = fg_of(&a);
    assert!(before > 1000, "geo 面应有前景，得 {before}");
    // WHEEL 正=zoom×exp(+) 放大 → 前景增长
    let mk_wheel = |w: f32| VeInput {
        struct_size: std::mem::size_of::<VeInput>(),
        kind: KIND_WHEEL,
        px: 80.0,
        py: 60.0,
        wheel: w,
        button: 0,
        mods: 0,
    };
    assert_eq!(visiaengine_on_input(ve, &mk_wheel(3.0)), 1);
    let mut b = vec![0u8; 160 * 120 * 4];
    frame(&mut b);
    let after = fg_of(&b);
    let diff = a
        .iter()
        .zip(b.iter())
        .filter(|pair| {
            let (x, y) = *pair;
            (i16::from(*x) - i16::from(*y)).abs() > 8
        })
        .count();
    assert!(diff > 3000, "zoom 后画面应显著变化，diff={diff}");
    assert!(after != before, "前景像素应随 zoom 变（{before}→{after}）");
    // 演进锚行为面：struct_size 过小的 WHEEL 被拒且不改状态
    let mut tiny = mk_wheel(3.0);
    tiny.struct_size = 4;
    assert_eq!(visiaengine_on_input(ve, &tiny), VE_ERR_ARG);
    assert_eq!(visiaengine_destroy(ve), 0);
}
