#[test]
fn fontdue_compiles_and_parses_fixture() {
    let f = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/data/DejaVuSans.ttf"));
    assert!(visiaengine_io_text::fontdue_smoke(f), "DejaVu 应可 parse");
    assert!(!visiaengine_io_text::fontdue_smoke(b"not a font"), "垃圾字节应拒");
}
