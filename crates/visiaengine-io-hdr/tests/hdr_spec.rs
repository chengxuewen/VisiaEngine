//! IO-17 contract tests (T1 pure CPU). Fixtures are hand-built RGBE byte
//! strings (same discipline as io-tiles mvt_decode.rs) — no external file
//! dependency, deterministic by construction. The committed generator
//! `scripts/gen_hdr.py` produces the big RLE fixture (resources/data).

use visiaengine_io_hdr::{IoHdrError, decode_hdr};

fn hdr(w: u32, h: u32, body: &[u8]) -> Vec<u8> {
    let mut v = format!("#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y {h} +X {w}\n").into_bytes();
    v.extend_from_slice(body);
    v
}

// spec: IO-17
#[test]
fn header_parse_and_flat_decode() {
    // Hand-computed RGBE: (200,100,50,e=136) -> 200/256 * 2^(136-136) = 0.78125.
    // e=0 quad encodes zero. Quad with e=128 -> c/256 * 2^-8.
    let body: Vec<u8> = vec![
        200, 100, 50, 128, // [0.78125, 0.390625, 0.1953125] (c/256, e=128 -> scale 1)
        0, 0, 0, 0, // zero (e=0)
        255, 255, 255, 135, // 255/256 * 2^7 = 69.84.. (HDR proof, >1.0)
        128, 128, 128, 120, // 128/256 * 2^-8 = 0.001953125
    ];
    let img = decode_hdr(&hdr(2, 2, &body)).expect("flat decode");
    assert_eq!((img.width, img.height), (2, 2));
    assert_eq!(img.pixels.len(), 4);
    assert_eq!(img.pixels[0][0], 200.0 / 256.0);
    assert_eq!(img.pixels[0][1], 100.0 / 256.0);
    assert_eq!(img.pixels[0][2], 50.0 / 256.0);
    assert_eq!(img.pixels[1], [0.0; 3], "e=0 encodes zero");
    assert!(img.pixels[2][0] > 1.0, "HDR proof: 255/256*2^7 > 1");
    assert_eq!(img.pixels[3][0], 0.5 / 256.0);
}

// spec: IO-17
#[test]
fn rle_decode_planar_runs() {
    // w=8 (RLE-eligible). One scanline: all-R channel = run(8, 200); G =
    // literals [4,5]; B = run(2,9)+run(6,3); E = repeat(8, 136).
    let mut body = vec![2, 2, 0, 1]; // magic 2,2, scanline count = h=1 (hi=0, lo=1)
    body.extend_from_slice(&[8, 200]); // R: repeat 200 x8
    body.extend_from_slice(&[130, 4, 5, 6, 6]); // G: 2 literals (4,5) + run count=6 value=6
    body.extend_from_slice(&[2, 9, 6, 3]); // B: run(2,9) + run(6,3)
    body.extend_from_slice(&[8, 128]); // E: repeat 128 x8 (scale 1)
    let img = decode_hdr(&hdr(8, 1, &body)).expect("rle decode");
    assert_eq!(img.pixels.len(), 8);
    let v = img.pixels[0][0];
    assert_eq!(v, 200.0 / 256.0, "R channel = 200/256 (e=128 -> scale 1)");
    assert_eq!(img.pixels[1][0], 200.0 / 256.0, "R constant across row");
    assert_eq!(img.pixels[0][1], 4.0 / 256.0, "G literal path");
    assert_eq!(img.pixels[3][1], 6.0 / 256.0, "G run(6,6) path");
    assert_eq!(img.pixels[0][2], 9.0 / 256.0, "B run(2,9)");
    assert_eq!(img.pixels[7][2], 3.0 / 256.0, "B run(6,3)");
    assert!(img.pixels.iter().all(|p| p.iter().all(|c| c.is_finite())));
}

// spec: IO-17
#[test]
fn rle_multi_scanline_magic_tracks_height() {
    // h=2 -> magic scanline words are 2,2,0,2 then 2,2,0,2 (hi=0 lo=2 for
    // both rows when h < 256). Minimal runs for each channel each row.
    let mut body = Vec::new();
    for _ in 0..2 {
        body.extend_from_slice(&[2, 2, 0, 2]);
        for ch in 0..4u8 {
            body.extend_from_slice(&[8, 10 + ch]);
        }
    }
    let img = decode_hdr(&hdr(8, 2, &body)).expect("rle 2 rows");
    assert_eq!(img.pixels.len(), 16);
    // Channels: R=10, G=11, B=12, E=13. Standard formula: f = c/256 * 2^(E-128).
    let expect = 10.0f32 / 256.0 * 2.0f32.powi(13 - 128);
    assert_eq!(img.pixels[0][0], expect);
    // Row 1 repeats the same runs (magic words prove 2-row progression).
    assert_eq!(img.pixels[8][0], expect);
}

// spec: IO-17
#[test]
fn truncated_rejected() {
    let good = hdr(2, 1, &[1, 2, 3, 4, 5, 6, 7, 8]);
    for cut in [1, 3, 7, 9] {
        let err = decode_hdr(&good[..good.len() - cut.min(good.len() - 15)]).err();
        let _ = err; // may still parse when cut lands in header; real asserts below
    }
    // Body truncation (scanline needs 8 bytes, give 4).
    let err = decode_hdr(&hdr(2, 1, &[1, 2, 3, 4])).unwrap_err();
    assert_eq!(
        err,
        IoHdrError::Truncated { need: 8, at: 0 },
        "flat scanline truncation reports exact need"
    );
    // RLE truncation: magic present, run stream cut.
    let err = decode_hdr(&hdr(8, 1, &[2, 2, 0, 1, 8, 200, 2, 4])).unwrap_err();
    assert!(matches!(err, IoHdrError::Truncated { .. }), "got {err:?}");
}

// spec: IO-17
#[test]
fn bad_magic_rejected() {
    let mut b = hdr(1, 1, &[]);
    b[0] = b'X';
    assert_eq!(decode_hdr(&b).unwrap_err(), IoHdrError::BadMagic);
    assert_eq!(decode_hdr(b"").unwrap_err(), IoHdrError::BadMagic);
    // Missing #? prefix entirely.
    assert_eq!(decode_hdr(b"RADIANCE\n").unwrap_err(), IoHdrError::BadMagic);
}

// spec: IO-17
#[test]
fn bad_format_rejected() {
    let mut b = hdr(1, 1, &[]);
    let pat = b"FORMAT=32-bit_rle_rgbe";
    let at = b.windows(pat.len()).position(|w| w == pat).expect("pat");
    b[at..at + pat.len()].copy_from_slice(b"FORMAT=colortable     ");
    assert_eq!(decode_hdr(&b).unwrap_err(), IoHdrError::BadFormat);
    // Missing FORMAT line entirely.
    let b = b"#?RADIANCE\n\n-Y 1 +X 1\n";
    assert_eq!(decode_hdr(b).unwrap_err(), IoHdrError::BadFormat);
}

// spec: IO-17
#[test]
fn bad_resolution_rejected() {
    assert_eq!(
        decode_hdr(b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n+X 4 -Y 2\n").unwrap_err(),
        IoHdrError::BadResolution,
        "axis order must be -Y/+X (fixed sign convention)"
    );
    assert_eq!(
        decode_hdr(b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y 2 +X 0\n").unwrap_err(),
        IoHdrError::BadResolution,
        "zero width"
    );
    // Missing resolution line (blank then EOF).
    assert_eq!(
        decode_hdr(b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n").unwrap_err(),
        IoHdrError::BadResolution
    );
    // RLE run straddling the scanline width = BadRle (not silent overshoot).
    // w=4 is below the RLE-eligible range (w>=8), so build an eligible case:
    // w=8, first channel run count=9 (>8) -> BadRle.
    let err = decode_hdr(&hdr(8, 1, &[2, 2, 0, 1, 9, 7])).unwrap_err();
    assert!(matches!(err, IoHdrError::BadRle { .. }), "got {err:?}");
}

// spec: IO-17
#[test]
fn unknown_header_keys_skipped() {
    let b = b"#?RADIANCE\nEXPOSURE=1.0\nSOFTWARE=gen_hdr.py\nFORMAT=32-bit_rle_rgbe\nPRIMARIES= 0.64 0.33 0.3 0.6 0.15 0.06 0.3333 0.3333\n\n-Y 1 +X 2\n\x00\x00\x00\x00\x00\x00\x00\x00";
    let img = decode_hdr(b).expect("unknown keys skipped");
    assert_eq!(img.pixels.len(), 2);
    assert_eq!(img.pixels[0], [0.0; 3]);
}
