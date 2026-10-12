//! IO-01..06 contract tests (T1 pure CPU, zero fixture dependency -- PLY fully built inline).

use visiaengine_io_points::{MAX_POINTS_CAP, PclError, parse_pcl};

const ASCII: &str = "ply\nformat ascii 1.0\ncomment visia unit\nelement vertex 4\n\
     property float x\nproperty float y\nproperty float z\n\
     property uchar red\nproperty uchar green\nproperty uchar blue\n\
     end_header\n0 0 0 255 0 0\n1 0 0 0 255 0\n0 1 0 0 0 255\n1 1 0 255 255 255\n";

fn bin_le(n: u32, props: &str, body: &[u8]) -> Vec<u8> {
    let head =
        format!("ply\nformat binary_little_endian 1.0\nelement vertex {n}\n{props}end_header\n");
    let mut v = head.into_bytes();
    v.extend_from_slice(body);
    v
}
fn f32le(x: f32, y: f32, z: f32) -> Vec<u8> {
    [x, y, z].iter().flat_map(|c| c.to_le_bytes()).collect()
}

// spec: IO-01
#[test]
fn ascii_positions_colors_and_meta() {
    let cloud = parse_pcl(ASCII.as_bytes(), false).expect("the basic ascii form must pass");
    assert_eq!(cloud.positions.len(), 4);
    assert_eq!(cloud.colors[0], [1.0, 0.0, 0.0]);
    assert_eq!(cloud.colors[3], [1.0, 1.0, 1.0]);
    // D7: origin = bbox center in f64, positions = origin-local
    assert!((cloud.origin[0] - 0.5).abs() < 1e-9 && (cloud.origin[1] - 0.5).abs() < 1e-9);
    assert!(cloud.positions.iter().all(|p| p[0].abs() <= 1.0));
    // meta: cloud-level single row of 8 columns (IO-06)
    assert_eq!(cloud.meta.f64(0, "point_count"), Some(4.0));
    assert_eq!(cloud.meta.str_value(0, "format"), Some("ply/ascii"));
    assert_eq!(cloud.meta.f64(0, "bbox_max_x"), Some(1.0));
    assert_eq!(cloud.meta.f64(0, "bbox_min_z"), Some(0.0));
    assert_eq!(cloud.report.kept, 4);
}

// spec: IO-01
#[test]
fn missing_colors_default_gray_unknown_scalar_column_dropped() {
    let s = ASCII
        .replace(
            "property uchar red\nproperty uchar green\nproperty uchar blue\n",
            "property float heat\n",
        )
        .replace(" 255 0 0", " 0.5")
        .replace(" 0 255 0", " 0.6")
        .replace(" 0 0 255", " 0.7")
        .replace(" 255 255 255", " 0.8");
    let cloud = parse_pcl(s.as_bytes(), false)
        .expect("unknown scalar column = column-level loss, not point loss");
    assert_eq!(cloud.positions.len(), 4);
    assert_eq!(cloud.colors[1], [0.7, 0.75, 0.8], "missing color default");
    assert_eq!(
        cloud.report.dropped_unsupported, 1,
        "heat column counted (column level)"
    );
}

// spec: IO-01
#[test]
fn head_level_illegal_always_err_regardless_lenient() {
    for lenient in [false, true] {
        assert!(matches!(
            parse_pcl(b"not a ply file at all", lenient),
            Err(PclError::Malformed(_))
        ));
        let no_v = "ply\nformat ascii 1.0\nend_header\n";
        assert!(matches!(
            parse_pcl(no_v.as_bytes(), lenient),
            Err(PclError::Malformed(_))
        ));
    }
}

// spec: IO-02
#[test]
fn bin_le_float_xyz_uchar_rgb_matches_ascii() {
    let props = "property float x\nproperty float y\nproperty float z\n\
                 property uchar red\nproperty uchar green\nproperty uchar blue\n";
    let mut body = Vec::new();
    let rgb: [[u8; 3]; 4] = [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]];
    for (i, p) in [[0f32, 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 0.]]
        .iter()
        .enumerate()
    {
        body.extend_from_slice(&f32le(p[0], p[1], p[2]));
        body.extend_from_slice(&rgb[i]);
    }
    let bin = parse_pcl(&bin_le(4, props, &body), false).expect("bin_le must pass");
    let asc = parse_pcl(ASCII.as_bytes(), false).unwrap();
    assert_eq!(
        bin.positions, asc.positions,
        "same data, same result across encodings (15B stride order-driven)"
    );
    assert_eq!(bin.colors, asc.colors);
    assert_eq!(bin.meta.str_value(0, "format"), Some("ply/binary_le"));
}

// spec: IO-02
#[test]
fn double_positions_and_big_endian_reject() {
    let props = "property double x\nproperty double y\nproperty double z\n";
    let mut body = Vec::new();
    for p in [[0f64, 0., 0.], [2., 0., 0.]] {
        body.extend_from_slice(&p.iter().flat_map(|c| c.to_le_bytes()).collect::<Vec<_>>());
    }
    let cloud = parse_pcl(&bin_le(2, props, &body), false).expect("double carrying face");
    // D7: origin = bbox center (1,0,0) -> local +/-1.0 (not the raw value domain 0/2 -- the IO-06 duty)
    assert!((cloud.origin[0] - 1.0).abs() < 1e-12);
    assert!((cloud.positions[1][0] - 1.0).abs() < 1e-6);
    let be = b"ply\nformat binary_big_endian 1.0\nelement vertex 1\nproperty float x\nend_header\n\x00\x00\x00\x00";
    assert!(
        matches!(parse_pcl(be, true), Err(PclError::UnsupportedFormat(_))),
        "be = file-level rejection"
    );
}

// spec: IO-03
#[test]
fn four_domains_classify_lenient_and_fastfail() {
    // NaN/inf points + overflow points + face elements + normal points interleaved
    let s = "ply\nformat ascii 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\n\
     element face 1\nproperty list int32 int vertex_indices\nend_header\n0 0 0\n1 0 0\nnan 1 0\n1e39 0 0\n3 0 1 2\n";
    let fast = parse_pcl(s.as_bytes(), false);
    assert!(
        matches!(
            fast,
            Err(PclError::Dirty(_)) | Err(PclError::Malformed(_)) | Err(PclError::Truncated { .. })
        ),
        "FastFail must reject dirty wholesale: {fast:?}"
    );
    let cloud = parse_pcl(s.as_bytes(), true).expect("Lenient keeps clean points");
    assert_eq!(cloud.report.dropped_non_finite, 1, "nan row");
    assert_eq!(cloud.report.dropped_out_of_domain, 1, "1e39 overflow row");
    assert_eq!(cloud.report.kept, 2);
    assert!(
        cloud.report.dropped_unsupported >= 1,
        "face + list counted at element-column level"
    );
    // duplicate points form no class: writing the same point twice discards zero and counts zero (the living face of the IO-03 note)
    let dup = "ply\nformat ascii 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nend_header\n5 5 5\n5 5 5\n";
    let c2 = parse_pcl(dup.as_bytes(), false).unwrap();
    assert_eq!(
        (c2.report.kept, c2.positions.len()),
        (2, 2),
        "duplicate points = legal data"
    );
}

// spec: IO-04
#[test]
fn truncation_dual_semantics_pinned_both_ways() {
    // ascii half-line tail (byte-level truncation generated programmatically, not hand-copied)
    let cut = &ASCII[..ASCII.len() - "1 1 0 255 255 255\n".len() - 3];
    let fast = parse_pcl(cut.as_bytes(), false);
    assert!(
        matches!(
            fast,
            Err(PclError::Truncated {
                declared: 4,
                complete: 3
            })
        ),
        "FastFail = truncation rejects wholesale {fast:?}"
    );
    let lenient = parse_pcl(cut.as_bytes(), true).expect("Lenient keeps the prefix");
    assert_eq!(lenient.report.kept, 3);
    assert_eq!(
        lenient.report.truncated_points, 1,
        "residual point count reported truthfully"
    );
    // binary tail bytes below one stride
    let props = "property float x\nproperty float y\nproperty float z\n";
    let mut body = f32le(0., 0., 0.);
    body.extend_from_slice(&[1.0f32.to_le_bytes().as_slice(), &[9, 9]].concat()); // residual 5B < 12B
    let bc = parse_pcl(&bin_le(2, props, &body), true).expect("bin tail truncation under Lenient");
    assert_eq!((bc.report.kept, bc.report.truncated_points), (1, 1));
}

// spec: IO-05
#[test]
fn capacity_gate_on_declared_before_alloc() {
    let big = format!(
        "ply\nformat ascii 1.0\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\nend_header\n",
        MAX_POINTS_CAP + 1
    );
    for lenient in [false, true] {
        let e = parse_pcl(big.as_bytes(), lenient);
        assert!(
            matches!(e, Err(PclError::OverCapacity { declared, cap }) if declared == MAX_POINTS_CAP + 1 && cap == MAX_POINTS_CAP),
            "declared over cap rejects upfront {e:?}"
        );
    }
}

// spec: IO-06
#[test]
fn far_origin_shift_is_f64_first_then_f32() {
    // every point +1e6: the origin absorbs the large numbers, local stays a small domain, order kept (the D7 load-layer duty)
    let s = "ply\nformat ascii 1.0\nelement vertex 4\n\
property float x\nproperty float y\nproperty float z\n\
property uchar red\nproperty uchar green\nproperty uchar blue\n\
end_header\n1000000 1000000 0 255 0 0\n1000001 1000000 0 0 255 0\n\
1000000 1000001 0 0 0 255\n1000001 1000001 0 255 255 255\n";
    let c = parse_pcl(s.as_bytes(), false).unwrap();
    assert!((c.origin[0] - 1_000_000.5).abs() < 1e-6);
    assert!(
        c.positions.iter().all(|p| p[0].abs() < 1.0),
        "f64 subtract first, then convert to f32"
    );
    let ref_c = parse_pcl(ASCII.as_bytes(), false).unwrap();
    assert_eq!(
        c.positions, ref_c.positions,
        "translation invariance: reciprocal local arrays equal"
    );
}

/// The fixture's true source (same scheme as texquad.glb): consumed by the capi/E204 file roads.
#[test]
#[ignore = "fixture generation (not validation)"]
fn emit_pcl_fixtures() {
    let props = "property float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\n";
    let mut body = Vec::new();
    let pts = [
        [0f32, 0., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
        [1., 1., 0.],
        [0.5, 0.5, 0.8],
    ];
    let cols = [
        [255u8, 0, 0],
        [0, 255, 0],
        [0, 0, 255],
        [255, 255, 0],
        [255, 255, 255],
    ];
    for (p, c) in pts.iter().zip(cols) {
        body.extend_from_slice(&f32le(p[0], p[1], p[2]));
        body.extend_from_slice(&c);
    }
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/data");
    std::fs::write(format!("{dir}/pcl_tetra_ascii.ply"), ASCII).expect("ascii fixture");
    std::fs::write(format!("{dir}/pcl_tetra_bin.ply"), bin_le(5, props, &body))
        .expect("bin fixture");
    println!("wrote pcl_tetra_ascii.ply + pcl_tetra_bin.ply");
}
