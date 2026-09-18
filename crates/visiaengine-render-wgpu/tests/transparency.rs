//! REND-36/WGPU-27/28：半透明带五门（①=V0 混合域裁决门——预言翻=停带重呈）。
//! 顶视 ortho 精确路；期望值=线性域混合+硬件编码链（CORE-16 自洽预言）。

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

/// ① V0 裁决门：黑底不透明板 + 50% 红板 → 线性域=(188,0,0)；编码域=(128,0,0)。
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
    // 线性域预言=encode(0.5·lin(1)·shade0.627)=151；编码域=83（差 68 单值判案）
    println!("V0 VERDICT center={c:?} 线性预言=(151,0,0) 编码预言=(83,0,0)");
    assert!(
        near(c[0], 151, 8) && c[1] < 16 && c[2] < 16,
        "混合域预言双双落空：{c:?}（翻案=停带重呈）"
    );
}

/// ② 排序活性双向：深度互换→结果翻转；提交互换→结果不翻。
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
    // 构型 A：相机 z=50 俯视 → z7=近/红 z5=远 → 绿(近)后画 = (110,151,0)
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
    // 构型 B：深度互换（绿近红远）
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
    // 构型 C：A 提交互换（先红后绿，深度不变）
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
    println!("ORDER A(红近)={ca:?} B(绿近)={cb:?} C(提交换)={cc:?}");
    assert!(
        near(ca[0], 110, 10) && near(ca[1], 151, 10),
        "A 期望 (110,151,0) got {ca:?}"
    );
    assert!(
        near(cb[0], 151, 10) && near(cb[1], 110, 10),
        "B 深度互换必翻 (151,110,0) got {cb:?}"
    );
    assert_eq!(ca, cc, "提交互换必不翻（排序真锁深度序）");
}

/// ③ 不写深度锁：同深双灰板=互见（write ON 则后件被 Less 拒→单层）。
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
    // 含 shade 预言：双层=encode(0.5·L+0.5·(0.5L))=89；write-ON 单层=encode(0.5L)=73
    println!("DEPTHWRITE-OFF center={c:?}（双层=89 单层对照=73）");
    assert!(
        c[0] > 84 && c[0] < 95,
        "双层互见缺席=透明管线在写深度：{c:?}"
    );
}

/// ④ 材质 a==1 canary：走旧路纯覆盖（无混合痕）+分拣零触。
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
    // a==1 纯覆盖=不透明管线现值 encode(shade·1.0)=207（混合痕=对半 151 以下必辨）
    assert!(
        near(c[0], 207, 8) && c[1] < 10 && c[2] < 10,
        "a==1 必须纯覆盖无混合痕：{c:?}"
    );
}

/// ⑤ Labels 恒顶收尾：水板之中标签像素不被稀释。
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
    assert!(white > 30, "恒顶白标水墨中仍在场 got {white}");
    // 顶心像素=纯白（未与红水混合）
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
    assert!(hit > 5, "标签白芯必达 255（Always+末序=不被稀释）");
}
