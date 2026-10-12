//! WGPU-24/25: the label pipeline (atlas R8 + screen-constant size + always-top + clip anchor decision).
//! Predicate discipline [PIT-8]: ink-quantity interval + cross-zoom non-deformation; the font ink = pinned after the DejaVu probe measurement.

use visiaengine_io_text::{FontFace, GLYPH_ATLAS_PX, GlyphCache, layout};
use visiaengine_render::{
    Camera, CameraRig, ClipSetup, DrawCommand, Frame, LabelMark, LabelTableDesc, RenderBackend,
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
const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../resources/data/DejaVuSans.ttf"
));

/// DejaVu 'M' @ 32px white label (world anchor z = 0.1). Returns (backend with atlas + table injected).
fn scene() -> (HeadlessBackend, Vec<DrawCommand>) {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let face = FontFace::from_bytes(FIXTURE).expect("font");
    let mut cache = GlyphCache::new();
    let (quads, _pen) = layout("M", &face, &mut cache, 32.0);
    assert!(!quads.is_empty());
    b.set_glyph_atlas(cache.pixels(), GLYPH_ATLAS_PX, GLYPH_ATLAS_PX)
        .expect("atlas");
    let marks: Vec<LabelMark> = quads
        .iter()
        .map(|q| {
            LabelMark::new(
                [0.0, 0.0, 0.1],
                [1.0, 1.0, 1.0, 1.0],
                [q.uv0[0], q.uv0[1], q.uv1[0], q.uv1[1]],
                [
                    q.size_px[0],
                    q.size_px[1],
                    q.top_left_px[0],
                    q.top_left_px[1],
                ],
            )
        })
        .collect();
    let table = b
        .create_labels(&LabelTableDesc { data: &marks })
        .expect("labels table");
    let cmds = vec![
        DrawCommand::ClearColor {
            rgba: [0.0, 0.0, 0.0, 1.0],
        },
        DrawCommand::DrawLabels {
            table,
            origin: [0.0; 3],
            transform: T4,
        },
    ];
    (b, cmds)
}

/// The top-view ortho exact path [REND-29]: px_world_scale = 2*zoom/W stays always in sync with zoom --
/// the label's "screen-constant size" is **construction-exact** under this form (the perspective approximate road = the engine face, same division as E202).
fn frame(zoom: f32, commands: Vec<DrawCommand>, clip: Option<ClipSetup>) -> Frame {
    let rig = CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let px_scale = 2.0 * zoom / W as f32;
    Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(zoom, zoom * H as f32 / W as f32, 0.1, 1000.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .ortho_frame(zoom, W as f32, H as f32, 0.1, 1000.0)
            .expect("p"),
        px_world_scale: px_scale,
        shadow: None,
        clip,
        edl: None,
        post: Vec::new(),
        commands,
    }
}

fn ink(img: &visiaengine_render_wgpu::OffscreenFrame) -> u32 {
    img.rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > 100 && p[1] > 100 && p[2] > 100)
        .count() as u32
}

#[test]
// spec: WGPU-24
fn label_ink_present_and_cross_zoom_invariant() {
    let (mut b, cmds) = scene();
    let base = b
        .render_to_pixels(&frame(10.0, cmds.clone(), None))
        .expect("r0");
    let i0 = ink(&base);
    assert!(i0 > 30 && i0 < 2500, "the M ink-quantity interval got {i0}");
    // ortho exact path: px_scale two gears 0.15<->0.3 = the label's **pixel size unchanged** -> ink +/- 15%
    let f2 = frame(5.0, cmds.clone(), None); // 2x zoom: constant size = ink exactly equal
    let near = b.render_to_pixels(&f2).expect("r1");
    let i1 = ink(&near);
    assert_eq!(
        i0, i1,
        "the ortho exact path's cross-zoom ink must be pixel-for-pixel equal i0={i0} i1={i1}"
    );
}

#[test]
// spec: WGPU-25
fn label_clip_anchors_whole_or_none() {
    // anchor predicate (the whole label lives together and dies together, not a per-pixel half-cut)
    let (mut b, cmds) = scene();
    let kill = ClipSetup::new(&[[0.0, 0.0, -1.0, 0.05]])
        .expect("keeps z <= 0.05 (the anchor z = 0.1 is outside)");
    let img = b
        .render_to_pixels(&frame(10.0, cmds.clone(), Some(kill)))
        .expect("r1");
    assert_eq!(
        ink(&img),
        0,
        "the anchor clipped = whole label dead got {}",
        ink(&img)
    );
    let keep = ClipSetup::new(&[[0.0, 0.0, -1.0, 0.5]]).expect("keeps z <= 0.5");
    let img2 = b
        .render_to_pixels(&frame(10.0, cmds, Some(keep)))
        .expect("r2");
    assert!(ink(&img2) > 30, "the anchor kept = the whole label present");
}
