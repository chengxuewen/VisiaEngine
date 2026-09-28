use visiaengine_io_hdr::decode_hdr;
#[test]
fn decode_fixture() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/demo_sky.hdr"
    ))
    .expect("fixture");
    let img = decode_hdr(&bytes).expect("decode");
    assert_eq!((img.width, img.height), (64, 32));
    let over1 = img
        .pixels
        .iter()
        .filter(|p| p[0] > 1.0 || p[1] > 1.0 || p[2] > 1.0)
        .count();
    eprintln!("fixture: {} px, {} px over 1.0", img.pixels.len(), over1);
    assert!(over1 > 400);
}
