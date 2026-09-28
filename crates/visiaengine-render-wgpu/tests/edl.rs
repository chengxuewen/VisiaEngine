//! WGPU-33: EDL post-pass — OFF = byte-identical legacy canary; ON = edge
//! darkening pass. Predicate discipline [PIT-8]: thresholds recorded from
//! probes (see comments at assertion sites).

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, EdlSetup, Frame, PointMark, PointTableDesc, RenderBackend,
    TableId, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 128;
const H: u32 = 128;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Grid of points on the z=0 plane, eye [0,-8,4] (strokes.rs camera family).
fn grid_marks() -> Vec<PointMark> {
    let mut marks = Vec::new();
    for i in 0..200u32 {
        let x = (i % 20) as f32 * 0.4 - 3.8;
        let y = (i / 20) as f32 * 0.4 - 3.8;
        marks.push(PointMark::new([x, y, 0.0], [0.9, 0.35, 0.2], 4.0));
    }
    marks
}

fn frame_with(commands: Vec<DrawCommand>, edl: Option<EdlSetup>) -> Frame {
    let rig = CameraRig::look_at([0.0, -8.0, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let proj = rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap();
    Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 0.16,
        shadow: None,
        clip: None,
        edl,
        post: Vec::new(),
        commands,
    }
}

fn cloud_frame(table: TableId, edl: Option<EdlSetup>) -> Frame {
    frame_with(
        vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            DrawCommand::DrawPoints {
                table,
                origin: [0.0; 3],
                transform: T4,
            },
        ],
        edl,
    )
}

fn upload(b: &mut HeadlessBackend, marks: &[PointMark]) -> TableId {
    b.create_points(&PointTableDesc { data: marks })
        .expect("points")
}

// spec: WGPU-33
// spec: REND-40
#[test]
fn edl_off_is_bitwise_stable_and_on_darkens_edges() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let tid = upload(&mut b, &grid_marks());

    // OFF canary: two identical renders must agree byte-for-byte (no hidden
    // state leak from the switch itself).
    let off_a = b.render_to_pixels(&cloud_frame(tid, None)).expect("off-a");
    let off_b = b.render_to_pixels(&cloud_frame(tid, None)).expect("off-b");
    assert_eq!(off_a.rgba, off_b.rgba, "OFF canary drifted");

    // ON: edge pixels darken. Differing pixels far above the trivial-count
    // floor; probe WGPU-33P1 = {diff>0} per-point-cloud band.
    let on = b
        .render_to_pixels(&cloud_frame(
            tid,
            Some(EdlSetup::new(0.35).expect("edl domain")),
        ))
        .expect("on");
    let diff: u32 = off_a
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(on.rgba.as_chunks::<4>().0)
        .filter(|(a, c)| a[..3] != c[..3])
        .count() as u32;

    // Silhouette probe: topmost point band (first ~12 rows containing cloud
    // pixels). EDL darkens boundary pixels against background — mean luma in
    // that band must drop, interior pixel deltas stay small.
    let lum = |p: &[u8]| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]);
    let rows_with_cloud = |img: &visiaengine_render_wgpu::OffscreenFrame| -> Vec<u32> {
        (0..H)
            .filter(|&y| {
                (0..W).any(|x| {
                    let o = ((y * W + x) * 4) as usize;
                    lum(&img.rgba[o..o + 3]) > 30
                })
            })
            .collect()
    };
    let rows = rows_with_cloud(&off_a);
    let top: &[u32] = &rows[..rows.len().min(12)];
    let band_mean = |img: &visiaengine_render_wgpu::OffscreenFrame| -> f64 {
        let mut sum = 0.0;
        let mut n = 0u32;
        for &y in top {
            for x in 0..W {
                let o = ((y * W + x) * 4) as usize;
                sum += f64::from(lum(&img.rgba[o..o + 3]));
                n += 1;
            }
        }
        sum / f64::from(n)
    };
    let (off_mean, on_mean) = (band_mean(&off_a), band_mean(&on));
    // Probe numbers (lavapipe, 128² 200-point grid, 128² target):
    // diff=2790 px; top-12-cloud-row band mean 331.0→322.9 (ratio 0.976).
    // Thresholds conservative (PIT-8): diff floor 400 (−86%), ratio cap
    // 0.985 (halfway to no-op 1.0 from measured 0.976).
    eprintln!(
        "WGPU-33 probe: diff={diff} off_mean={off_mean:.1} on_mean={on_mean:.1} rows={}",
        rows.len()
    );
    assert!(diff > 400, "EDL ON produced too few pixel deltas: {diff}");
    assert!(
        on_mean < off_mean * 0.985,
        "EDL must darken the silhouette band: off={off_mean:.1} on={on_mean:.1}"
    );

    // OFF after ON: switch is per-frame — legacy path must still be exact.
    let off_c = b.render_to_pixels(&cloud_frame(tid, None)).expect("off-c");
    assert_eq!(off_c.rgba, off_a.rgba, "OFF-after-ON drifted");
}
