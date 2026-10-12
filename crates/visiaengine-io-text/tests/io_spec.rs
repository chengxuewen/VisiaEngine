//! IO-07/08/09 contract unit tests (pure CPU T1, no GPU). Assertion counts = the DejaVu probe's measurements (PIT-8 ahead).

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
    // probe-measured shape: A@24 = w17 h18 adv~16.418 (change the font and this ledger must sync)
    let (m, bmp) = f.rasterize('A', 24.0);
    assert_eq!((m.width, m.height, bmp.len()), (17, 18, 17 * 18));
    assert!((m.advance_width - 16.417_969).abs() < 1e-3);
}

// spec: IO-08
#[test]
fn cache命中_零覆盖路_dirty语义() {
    let f = FontFace::from_bytes(FIXTURE).expect("font");
    let mut c = GlyphCache::new();
    // domain gate: size <= 0 / non-finite = None
    assert!(c.get(&f, 'A', 0.0).is_none());
    assert!(c.get(&f, 'A', f32::NAN).is_none());
    // first rasterization = dirty; a hit on the same slot does not dirty again
    let s1 = c.get(&f, 'A', 24.0).expect("A");
    assert!(c.take_dirty(), "a new glyph must dirty");
    let s2 = c.get(&f, 'A', 24.0).expect("A");
    assert_eq!(s1, s2, "hit = same slot (determinism)");
    assert!(!c.take_dirty(), "the hit road does not dirty");
    // zero coverage (space measured w0 h0 adv7.63): a slot without pixels does not dirty the texture
    let sp = c.get(&f, ' ', 24.0).expect("space slot");
    assert_eq!(sp.rect, [0, 0, 0, 0]);
    assert!((sp.advance - 7.628_906_3).abs() < 1e-3);
    assert!(!c.take_dirty(), "zero coverage does not dirty");
    // blit lands pixels: non-zero coverage inside A's slot rect (first-row center probe)
    let stride = GLYPH_ATLAS_PX as usize;
    let [x, y, w, h] = s1.rect;
    assert!(w > 0 && h > 0);
    let any = (0..h)
        .any(|r| (0..w).any(|k| c.pixels()[(r + y) as usize * stride + (k + x) as usize] > 0));
    assert!(any, "the atlas rectangle must carry coverage pixels");
    // size-keyed split: same char different size = different slot (key = (char, size x 10))
    let s3 = c.get(&f, 'A', 48.0).expect("A48");
    assert_ne!(s1.rect, s3.rect);
    assert_ne!(
        (s1.top, s3.top),
        (s1.top, s1.top),
        "top height follows font size"
    );
}

// spec: IO-08
#[test]
fn atlas满_全清重烘透明降级() {
    let f = FontFace::from_bytes(FIXTURE).expect("font");
    let mut c = GlyphCache::new();
    // two 500px giant glyphs must exceed one 512^2 -> the second triggers a reset re-bake and still gets a slot
    let a = c.get(&f, 'A', 500.0).expect("huge A");
    let b = c.get(&f, 'B', 500.0).expect("huge B after reclaim");
    assert!(a.rect[2] > 0 && b.rect[2] > 0);
    assert_eq!(
        b.rect[0], 0,
        "after a re-bake, packing restarts at the shelf origin"
    );
    // A was reclaimed: taking it again = rebuild into a new slot (jitter without crash, the result stays correct = determinism)
    let a2 = c.get(&f, 'A', 500.0).expect("A rebuilt");
    assert_eq!(
        a2.advance, a.advance,
        "re-baked metrics must equal (fontdue determinism)"
    );
}

// spec: IO-09
#[test]
fn layout_排布advance序_uv界_空形零quad() {
    let f = FontFace::from_bytes(FIXTURE).expect("font");
    let mut c = GlyphCache::new();
    // domain gate
    assert_eq!(layout("", &f, &mut c, 24.0), (Vec::new(), 0.0));
    let (q0, w0) = layout("AB", &f, &mut c, 0.0);
    assert!(q0.is_empty() && w0 == 0.0, "size <= 0 = empty");
    // "AB" two quads, dx strictly increasing; total width = Sigma-adv measured (A 16.418 + B probe sum: lock relatively)
    let (qs, pen) = layout("AB", &f, &mut c, 24.0);
    assert_eq!(qs.len(), 2);
    assert!(
        qs[1].top_left_px[0] > qs[0].top_left_px[0],
        "LTR dx increases"
    );
    // pen = A's prefix start + advA + advB structural lock: the last quad's start < pen, residual width in (0, advA] (B's trailing advance)
    assert!(
        pen > qs[1].top_left_px[0] && pen - qs[1].top_left_px[0] < 30.0,
        "trailing width residual {pen}"
    );
    for q in &qs {
        assert!(q.size_px[0] > 0.0 && q.size_px[1] > 0.0);
        assert!(
            q.uv0[0] < q.uv1[0] && q.uv0[1] < q.uv1[1],
            "uv monotonic within bounds"
        );
        assert!(q.uv1.iter().all(|&v| (0.0f32..=1.0f32).contains(&v)));
    }
    // space advances but emits no quad; "A B" = 2 quads and the final pen includes the space width
    let (qs2, pen2) = layout("A B", &f, &mut c, 24.0);
    assert_eq!(qs2.len(), 2);
    assert!(pen2 > pen, "the total width with a space must be larger");
    assert_eq!(
        qs2[0], qs[0],
        "prefix determinism (same cache, same result)"
    );
}
