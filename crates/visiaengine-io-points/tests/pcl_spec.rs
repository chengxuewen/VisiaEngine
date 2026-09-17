//! IO-01..06 契约测试（T1 纯 CPU，零 fixture 依赖——PLY 全内构造）。

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
    let cloud = parse_pcl(ASCII.as_bytes(), false).expect("ascii 基本形必过");
    assert_eq!(cloud.positions.len(), 4);
    assert_eq!(cloud.colors[0], [1.0, 0.0, 0.0]);
    assert_eq!(cloud.colors[3], [1.0, 1.0, 1.0]);
    // D7：origin=bbox 中心 f64、positions=origin-local
    assert!((cloud.origin[0] - 0.5).abs() < 1e-9 && (cloud.origin[1] - 0.5).abs() < 1e-9);
    assert!(cloud.positions.iter().all(|p| p[0].abs() <= 1.0));
    // meta：云级单行 8 列（IO-06）
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
    let cloud = parse_pcl(s.as_bytes(), false).expect("未知标量列=列级丢不丢点");
    assert_eq!(cloud.positions.len(), 4);
    assert_eq!(cloud.colors[1], [0.7, 0.75, 0.8], "缺色默认");
    assert_eq!(cloud.report.dropped_unsupported, 1, "heat 列计数（列级）");
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
    let bin = parse_pcl(&bin_le(4, props, &body), false).expect("bin_le 必过");
    let asc = parse_pcl(ASCII.as_bytes(), false).unwrap();
    assert_eq!(
        bin.positions, asc.positions,
        "双编码同数据同果（15B stride 序驱动）"
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
    let cloud = parse_pcl(&bin_le(2, props, &body), false).expect("double 承载面");
    // D7：origin=bbox 中心 (1,0,0) → local ±1.0（非原值域 0/2——IO-06 义务）
    assert!((cloud.origin[0] - 1.0).abs() < 1e-12);
    assert!((cloud.positions[1][0] - 1.0).abs() < 1e-6);
    let be = b"ply\nformat binary_big_endian 1.0\nelement vertex 1\nproperty float x\nend_header\n\x00\x00\x00\x00";
    assert!(
        matches!(parse_pcl(be, true), Err(PclError::UnsupportedFormat(_))),
        "be=文件级拒"
    );
}

// spec: IO-03
#[test]
fn four_domains_classify_lenient_and_fastfail() {
    // NaN/inf 点 + 溢出点 + face 元素 + 正常点混排
    let s = "ply\nformat ascii 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\n\
     element face 1\nproperty list int32 int vertex_indices\nend_header\n0 0 0\n1 0 0\nnan 1 0\n1e39 0 0\n3 0 1 2\n";
    let fast = parse_pcl(s.as_bytes(), false);
    assert!(
        matches!(
            fast,
            Err(PclError::Dirty(_)) | Err(PclError::Malformed(_)) | Err(PclError::Truncated { .. })
        ),
        "FastFail 脏必整拒: {fast:?}"
    );
    let cloud = parse_pcl(s.as_bytes(), true).expect("Lenient 收净点");
    assert_eq!(cloud.report.dropped_non_finite, 1, "nan 行");
    assert_eq!(cloud.report.dropped_out_of_domain, 1, "1e39 溢出行");
    assert_eq!(cloud.report.kept, 2);
    assert!(
        cloud.report.dropped_unsupported >= 1,
        "face+list 元素列级计数"
    );
    // 重复点不成类：双写同点零丢弃零计数（IO-03 注记的活体面）
    let dup = "ply\nformat ascii 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nend_header\n5 5 5\n5 5 5\n";
    let c2 = parse_pcl(dup.as_bytes(), false).unwrap();
    assert_eq!(
        (c2.report.kept, c2.positions.len()),
        (2, 2),
        "重复点=合法数据"
    );
}

// spec: IO-04
#[test]
fn truncation_dual_semantics_pinned_both_ways() {
    // ascii 半行尾（字节级截尾程序化生成，非手抄）
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
        "FastFail=截断整拒 {fast:?}"
    );
    let lenient = parse_pcl(cut.as_bytes(), true).expect("Lenient 收前缀");
    assert_eq!(lenient.report.kept, 3);
    assert_eq!(lenient.report.truncated_points, 1, "残点数如实报");
    // binary 尾字节不足 stride
    let props = "property float x\nproperty float y\nproperty float z\n";
    let mut body = f32le(0., 0., 0.);
    body.extend_from_slice(&[1.0f32.to_le_bytes().as_slice(), &[9, 9]].concat()); // 残 5B < 12B
    let bc = parse_pcl(&bin_le(2, props, &body), true).expect("bin 截尾 Lenient");
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
            "声明超帽先拒 {e:?}"
        );
    }
}

// spec: IO-06
#[test]
fn far_origin_shift_is_f64_first_then_f32() {
    // 全点 +1e6：origin 吸收大数、local 域小值、序保持（D7 装载层义务）
    let s = "ply\nformat ascii 1.0\nelement vertex 4\n\
property float x\nproperty float y\nproperty float z\n\
property uchar red\nproperty uchar green\nproperty uchar blue\n\
end_header\n1000000 1000000 0 255 0 0\n1000001 1000000 0 0 255 0\n\
1000000 1000001 0 0 0 255\n1000001 1000001 0 255 255 255\n";
    let c = parse_pcl(s.as_bytes(), false).unwrap();
    assert!((c.origin[0] - 1_000_000.5).abs() < 1e-6);
    assert!(
        c.positions.iter().all(|p| p[0].abs() < 1.0),
        "f64 先减后转 f32"
    );
    let ref_c = parse_pcl(ASCII.as_bytes(), false).unwrap();
    assert_eq!(
        c.positions, ref_c.positions,
        "平移不变性：互逆 local 数组等"
    );
}

/// fixture 真源（texquad.glb 同谱）：capi/E204 文件路消费。
#[test]
#[ignore = "fixture 生成（非验证）"]
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
