//! IO-07/08/09 契约单测（纯 CPU T1，无 GPU）。断言数=DejaVu 探针实测（PIT-8 先行）。

use visiaengine_io_text::{FaceError, FontFace, GLYPH_ATLAS_PX, GlyphCache, layout};

const FIXTURE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../resources/data/DejaVuSans.ttf"
));

// spec: IO-07
#[test]
fn face装载契约_空与垃圾拒_合法成() {
    assert_eq!(FontFace::from_bytes(&[]).unwrap_err(), FaceError::Empty);
    assert_eq!(
        FontFace::from_bytes(b"not a font").unwrap_err(),
        FaceError::Parse
    );
    let f = FontFace::from_bytes(FIXTURE).expect("DejaVu");
    // 探针实测形：A@24 = w17 h18 adv≈16.418（改字体必同步此账）
    let (m, bmp) = f.rasterize('A', 24.0);
    assert_eq!((m.width, m.height, bmp.len()), (17, 18, 17 * 18));
    assert!((m.advance_width - 16.417_969).abs() < 1e-3);
}

// spec: IO-08
#[test]
fn cache命中_零覆盖路_dirty语义() {
    let f = FontFace::from_bytes(FIXTURE).expect("font");
    let mut c = GlyphCache::new();
    // 域闸：size≤0/非有限=None
    assert!(c.get(&f, 'A', 0.0).is_none());
    assert!(c.get(&f, 'A', f32::NAN).is_none());
    // 首次栅格=dirty；命中同槽不再脏
    let s1 = c.get(&f, 'A', 24.0).expect("A");
    assert!(c.take_dirty(), "新字形必脏");
    let s2 = c.get(&f, 'A', 24.0).expect("A");
    assert_eq!(s1, s2, "命中=同槽（确定性）");
    assert!(!c.take_dirty(), "命中路不脏");
    // 零覆盖（space 实测 w0 h0 adv7.63）：有槽无像素不脏图
    let sp = c.get(&f, ' ', 24.0).expect("space slot");
    assert_eq!(sp.rect, [0, 0, 0, 0]);
    assert!((sp.advance - 7.628_906_3).abs() < 1e-3);
    assert!(!c.take_dirty(), "零覆盖不脏");
    // blit 落像素：A 槽 rect 内有非零覆盖（首行中心探针）
    let stride = GLYPH_ATLAS_PX as usize;
    let [x, y, w, h] = s1.rect;
    assert!(w > 0 && h > 0);
    let any = (0..h)
        .any(|r| (0..w).any(|k| c.pixels()[(r + y) as usize * stride + (k + x) as usize] > 0));
    assert!(any, "atlas 对应矩形必有覆盖像素");
    // 尺寸分键：同字不同 size=不同槽（key=(char,size×10)）
    let s3 = c.get(&f, 'A', 48.0).expect("A48");
    assert_ne!(s1.rect, s3.rect);
    assert_ne!((s1.top, s3.top), (s1.top, s1.top), "顶高随字号");
}

// spec: IO-08
#[test]
fn atlas满_全清重烘透明降级() {
    let f = FontFace::from_bytes(FIXTURE).expect("font");
    let mut c = GlyphCache::new();
    // 两张 500px 巨字必超一张 512² → 第二张触发 reset 重烘后仍成槽
    let a = c.get(&f, 'A', 500.0).expect("huge A");
    let b = c.get(&f, 'B', 500.0).expect("huge B after reclaim");
    assert!(a.rect[2] > 0 && b.rect[2] > 0);
    assert_eq!(b.rect[0], 0, "重烘后从 shelf 起点重新装箱");
    // A 被 reclaim：再取=重建新槽（抖而不崩，结果仍正确=确定性）
    let a2 = c.get(&f, 'A', 500.0).expect("A rebuilt");
    assert_eq!(a2.advance, a.advance, "重烘度量必等（fontdue 确定性）");
}

// spec: IO-09
#[test]
fn layout_排布advance序_uv界_空形零quad() {
    let f = FontFace::from_bytes(FIXTURE).expect("font");
    let mut c = GlyphCache::new();
    // 域闸
    assert_eq!(layout("", &f, &mut c, 24.0), (Vec::new(), 0.0));
    let (q0, w0) = layout("AB", &f, &mut c, 0.0);
    assert!(q0.is_empty() && w0 == 0.0, "size≤0=空");
    // "AB" 两 quad，dx 严格递增；总宽=Σadv 实测（A 16.418+B 探针取和：用相对锁）
    let (qs, pen) = layout("AB", &f, &mut c, 24.0);
    assert_eq!(qs.len(), 2);
    assert!(qs[1].top_left_px[0] > qs[0].top_left_px[0], "LTR dx 递增");
    // pen=A 前缀起点+advA+advB 结构锁：末 quad 起点 < pen，残宽∈(0, advA]（B 尾 advance）
    assert!(
        pen > qs[1].top_left_px[0] && pen - qs[1].top_left_px[0] < 30.0,
        "尾宽残量 {pen}"
    );
    for q in &qs {
        assert!(q.size_px[0] > 0.0 && q.size_px[1] > 0.0);
        assert!(q.uv0[0] < q.uv1[0] && q.uv0[1] < q.uv1[1], "uv 单调界内");
        assert!(q.uv1.iter().all(|&v| (0.0f32..=1.0f32).contains(&v)));
    }
    // space 只进 advance 不出 quad；"A B"=2 quad 且末 pen 含空格宽
    let (qs2, pen2) = layout("A B", &f, &mut c, 24.0);
    assert_eq!(qs2.len(), 2);
    assert!(pen2 > pen, "含空串总宽必大");
    assert_eq!(qs2[0], qs[0], "前缀确定性（同 cache 同结果）");
}
