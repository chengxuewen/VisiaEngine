//! CAPI-04：headless 出图字节（I2）——装载→渲染→回读→像素断言全链过 C ABI。

use std::ffi::CString;
use visiaengine::{
    MISS, VE_ERR_SIZE, visiaengine_create_group, visiaengine_create_headless, visiaengine_destroy,
    visiaengine_entity_at, visiaengine_entity_count, visiaengine_entity_set_visible,
    visiaengine_get_parent, visiaengine_load_geojson, visiaengine_load_gltf,
    visiaengine_load_mvt_dir, visiaengine_pick, visiaengine_readback, visiaengine_render,
    visiaengine_set_group_offset, visiaengine_set_parent, visiaengine_set_tile_view,
    visiaengine_viewport,
};
use visiaengine_io_tiles::TileSource;

fn repo_root() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !p.join("Cargo.lock").exists() {
        assert!(p.pop(), "Cargo.lock 上溯不中");
    }
    p
}

fn fixture(name: &str) -> CString {
    CString::new(format!("{}/resources/data/{name}", repo_root().display())).unwrap()
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

// spec: CAPI-04
#[test]
fn textured_glb_mount_renders_checker() {
    // M3 链路复核：texquad.glb（4×4 棋盘 PNG + TEXCOORD_0 + baseColor 0.5,0.25,0.5）
    // mount 后必须走 Textured 变体：棋盘红格（B≈0）与白格×base（紫：B≈R）双族并存。
    // uv 断链（=&[] 零填）→ 全采 (0,0) 红格 → 紫族=0 即红。
    let ve = visiaengine_create_headless(160, 120);
    assert_ne!(ve, 0);
    assert_eq!(
        visiaengine_load_gltf(ve, fixture("texquad.glb").as_ptr()),
        0
    );
    assert_eq!(visiaengine_entity_count(ve), 1);
    assert_eq!(visiaengine_render(ve), 0);
    let mut buf = vec![0u8; 160 * 120 * 4];
    assert_eq!(
        visiaengine_readback(ve, buf.as_mut_ptr(), buf.len() as u64),
        0
    );
    // CORE-16 域重钉（实测 56/23px）：sRGB 纹理+LINEAR 滤波下纯格心并入混色带，
    // 品红族按 **b 双峰**分治（红格带 b<135 / 白格带 b≥135，r=172 恒定域）。
    // uv 断链=全采 texel(0,0) 红 → 白族恒 0 的 canary 语义原样保持。
    let (mut reds, mut violets) = (0u32, 0u32);
    for y in 40..80 {
        for x in 50..110 {
            let p = &buf[((y * 160 + x) * 4) as usize..][..4];
            if p[0] > 150 && p[2] < 135 {
                reds += 1;
            } else if p[0] > 150 && p[2] >= 135 {
                violets += 1;
            }
        }
    }
    assert!(
        reds > 30 && violets > 15,
        "棋盘双族缺失 red={reds} violet={violets}"
    );
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-04
#[test]
fn viewport_resize_reconfigures_frame_dims() {
    // Qt 轮 Q1 行为锁：viewport(w,h) 即 resize 合同（headless 面=target 重建
    // 且 MeshCore 驻留；窗口面 sw.resize 路径由 smoke-x11/Qt demo 人验）。
    // 锁点：①换尺寸后新 dims readback 通过 ②旧尺寸缓冲=VE_ERR_SIZE（证 dims 真变了）
    // ③换回旧 dims 复现（可逆，非一次性重建事故）。
    let ve = visiaengine_create_headless(64, 48);
    assert_ne!(ve, 0);
    assert_eq!(
        visiaengine_load_gltf(ve, fixture("twoprim.glb").as_ptr()),
        0
    );
    assert_eq!(visiaengine_render(ve), 0);
    assert_eq!(visiaengine_viewport(ve, 80, 60), 0);
    assert_eq!(visiaengine_render(ve), 0);
    let mut big = vec![0u8; 80 * 60 * 4];
    assert_eq!(
        visiaengine_readback(ve, big.as_mut_ptr(), big.len() as u64),
        0
    );
    let mut old = vec![0u8; 64 * 48 * 4];
    assert_eq!(
        visiaengine_readback(ve, old.as_mut_ptr(), old.len() as u64),
        VE_ERR_SIZE,
        "换尺寸后旧 dims 必须被拒（合同真变）"
    );
    assert_eq!(visiaengine_viewport(ve, 64, 48), 0);
    assert_eq!(visiaengine_render(ve), 0);
    assert_eq!(
        visiaengine_readback(ve, old.as_mut_ptr(), old.len() as u64),
        0
    );
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-28
#[test]
fn tile_layer_renders_pixel_families_via_capi() {
    const W: usize = 160;
    const H: usize = 120;
    let tile_dir = CString::new(format!("{}/resources/data/tiles", repo_root().display())).unwrap();
    let ve = visiaengine_create_headless(W as u32, H as u32);
    assert_ne!(ve, 0);
    // mount: loads all 3x3 z10 tiles + auto-fits ortho camera to batch center
    let mounted = visiaengine_load_mvt_dir(ve, tile_dir.as_ptr(), 10);
    assert_eq!(mounted, 9, "3x3 fixture all mounted");
    // full-coverage bbox (same tiles, just triggers scheduler visible)
    let visible = visiaengine_set_tile_view(ve, -2.01e7, 1.995e7, -1.994e7, 2.005e7);
    assert_eq!(visible, 9, "all 9 tiles in view");
    assert_eq!(visiaengine_render(ve), 0, "render with tile layer");
    let mut buf = vec![0u8; W * H * 4];
    assert_eq!(
        visiaengine_readback(ve, buf.as_mut_ptr(), buf.len() as u64),
        0,
        "readback after tile render"
    );
    // pixel-family counts (same predicates as E206 headless gate, RGBA order)
    let cnt = |pred: &dyn Fn(u8, u8, u8) -> bool| -> usize {
        (0..W * H)
            .filter(|&i| {
                let (r, g, b) = (buf[i * 4], buf[i * 4 + 1], buf[i * 4 + 2]);
                pred(r, g, b)
            })
            .count()
    };
    let blue = cnt(&|r, _g, b| b > 90 && b > r + 20);
    let orange = cnt(&|r, g, b| r > 150 && g > 80 && b < 90);
    let teal = cnt(&|r, g, b| r < 110 && g > 60 && b > 60 && g >= r);
    let red = cnt(&|r, g, b| r > 150 && g < 90 && b < 90);
    println!("TILE RENDER blue={blue} orange={orange} teal={teal} red={red}");
    // conservative thresholds: measured −40% at 160×120 viewport
    // (E206 at 480×360: blue=16451 orange=11198 teal=16451; scaled ~1/9 area)
    // red=0 expected: POI [0.9,0.25,0.35]→sRGB G≈137, not <90; E206 480×360 catches edge AA only
    assert!(blue > 1500, "water fill coverage: got {blue}");
    assert!(orange > 2000, "road strokes visible: got {orange}");
    assert!(teal > 1000, "boundary seam visible: got {teal}");
    assert!(red > 800, "POI splats visible: got {red}");
    assert_eq!(visiaengine_destroy(ve), 0);
}
// spec: CAPI-28
#[test]
#[ignore = "debug: probe fixture class distribution"]
fn probe_tile_fixture_classes() {
    let root = {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        while !p.join("Cargo.lock").exists() {
            assert!(p.pop());
        }
        p.join("resources/data/tiles")
    };
    let src = visiaengine_io_tiles::FileSource::new(root);
    for x in 0..3u32 {
        for y in 0..3u32 {
            let bytes = src.load(10, x, y).expect("tile bytes");
            let tile = visiaengine_io_tiles::decode_tile(&bytes).expect("decode");
            for layer in &tile.layers {
                for f in &layer.features {
                    let tags = f.tags(layer);
                    let cls = tags.get("class");
                    println!(
                        "tile({},{}) layer={} gt={} class={:?}",
                        x,
                        y,
                        layer.name,
                        f.geom_type(),
                        cls
                    );
                }
            }
        }
    }
}

// spec: CAPI-31
#[test]
fn tree_group_offset_changes_rendered_pixels() {
    // Prove effective_offset is consumed by render(): move a group → its child entity
    // renders at a different screen position → pixel buffer changes.
    let ve = visiaengine_create_headless(160, 120);
    assert_ne!(ve, 0);
    let glb = fixture("twoprim.glb");
    assert_eq!(visiaengine_load_gltf(ve, glb.as_ptr()), 0);
    let h0 = visiaengine_entity_at(ve, 0);
    assert_ne!(h0, 0);

    // Baseline render
    assert_eq!(visiaengine_render(ve), 0);
    let mut buf_base = vec![0u8; 160 * 120 * 4];
    assert_eq!(
        visiaengine_readback(ve, buf_base.as_mut_ptr(), buf_base.len() as u64),
        0
    );

    // Parent h0 to a group
    let mut grp = 0u64;
    assert_eq!(visiaengine_create_group(ve, std::ptr::null(), &mut grp), 0);
    assert_eq!(visiaengine_set_parent(ve, h0, grp), 0);
    assert_eq!(visiaengine_get_parent(ve, h0), grp, "roundtrip");

    // Zero-offset render: must match baseline (effective_offset = [0,0,0])
    assert_eq!(visiaengine_render(ve), 0);
    let mut buf_zero = vec![0u8; 160 * 120 * 4];
    assert_eq!(
        visiaengine_readback(ve, buf_zero.as_mut_ptr(), buf_zero.len() as u64),
        0
    );
    assert_eq!(
        buf_base, buf_zero,
        "zero-offset must reproduce baseline pixel-for-pixel"
    );

    // Move group far outside frustum: effective_offset = [1e9, 1e9, 0]
    assert_eq!(visiaengine_set_group_offset(ve, grp, 1e9, 1e9, 0.0), 0);
    assert_eq!(visiaengine_render(ve), 0);
    let mut buf_moved = vec![0u8; 160 * 120 * 4];
    assert_eq!(
        visiaengine_readback(ve, buf_moved.as_mut_ptr(), buf_moved.len() as u64),
        0
    );

    // Check entity_visible: h0 should still be visible (group not hidden)
    let diff = buf_base
        .iter()
        .zip(buf_moved.iter())
        .filter(|(a, b)| a != b)
        .count();
    let moved_is_bg = buf_moved
        .chunks_exact(4)
        .filter(|p| p[0].abs_diff(63) <= 15 && p[1].abs_diff(75) <= 15 && p[2].abs_diff(89) <= 15)
        .count();
    let base_is_bg = buf_base
        .chunks_exact(4)
        .filter(|p| p[0].abs_diff(63) <= 15 && p[1].abs_diff(75) <= 15 && p[2].abs_diff(89) <= 15)
        .count();
    println!("tree offset diff={diff} bg_base={base_is_bg} bg_moved={moved_is_bg}");
    // More background pixels after moving h0 off-screen (h0 replaced by bg, h1 stays)
    assert!(
        moved_is_bg > base_is_bg || diff > 0,
        "moving group must reduce coverage: bg {base_is_bg}→{moved_is_bg} diff={diff}"
    );

    // Restore and verify round-trip back to baseline
    assert_eq!(visiaengine_set_group_offset(ve, grp, 0.0, 0.0, 0.0), 0);
    assert_eq!(visiaengine_render(ve), 0);
    let mut buf_restored = vec![0u8; 160 * 120 * 4];
    assert_eq!(
        visiaengine_readback(ve, buf_restored.as_mut_ptr(), buf_restored.len() as u64),
        0
    );
    assert_eq!(
        buf_base, buf_restored,
        "offset=0 restore must match baseline"
    );

    assert_eq!(visiaengine_destroy(ve), 0);
}
