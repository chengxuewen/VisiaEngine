//! R band: raster tile lane — FileSource::raster + decoded_raster round-trip.
//! Synthetic PNG fixture (image crate, zero external assets) proves the
//! scheduler machinery end-to-end on the raster payload enum.

use visiaengine_io_tiles::scheduler::{TilePayload, TileSet};
use visiaengine_io_tiles::source::FileSource;
use visiaengine_io_tiles::tiles::TileId;

// spec: IO-18
#[test]
fn raster_roundtrip_and_mvt_lane_coexist() {
    let dir = std::env::temp_dir().join(format!("ve-raster-{}-x", std::process::id()));
    std::fs::create_dir_all(dir.join("3/4/6")).expect("mkdir");
    // Synthetic 8x8 red tile at z=3 x=4 y=6 (slippy path layout)
    let img = image::RgbaImage::from_pixel(8, 8, image::Rgba([200u8, 30, 30, 255]));
    img.save(dir.join("3/4/6.png")).expect("png write");

    let mut set = TileSet::new(Box::new(FileSource::raster(&dir))).expect("set");
    let id = TileId::new(3, 4, 6).expect("id");
    let stats = set.ensure(&[id]).expect("ensure");
    assert_eq!(stats.loaded, 1);
    assert!(!set.decoded_cached(&id), "raw bytes only until decode");

    match set.decoded_raster(&id) {
        Some(TilePayload::Raster {
            rgba,
            width,
            height,
        }) => {
            assert_eq!((*width, *height), (8, 8));
            assert_eq!(rgba.len(), 8 * 8 * 4);
            assert_eq!(&rgba[..4], &[200, 30, 30, 255], "pixel intact");
        }
        other => panic!("expected raster payload, got {other:?}"),
    }
    // MVT lane on a raster tile = None (wrong lane), no panic, cache intact.
    assert!(
        set.decoded(&id).is_none(),
        "raster bytes must not decode as MVT"
    );

    // Cache hit on second raster access.
    assert!(set.decoded_cached(&id));
    let again = set.decoded_raster(&id).expect("cached");
    assert!(matches!(again, TilePayload::Raster { .. }));

    std::fs::remove_dir_all(&dir).ok();
}

// spec: IO-18
#[test]
fn raster_missing_tile_is_none_not_panic() {
    let dir = std::env::temp_dir().join(format!("ve-raster-missing-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let mut set = TileSet::new(Box::new(FileSource::raster(&dir))).expect("set");
    let id = TileId::new(1, 0, 0).expect("id");
    assert!(set.ensure(&[id]).is_err(), "missing file = source error");
    assert!(set.decoded_raster(&id).is_none());
    std::fs::remove_dir_all(&dir).ok();
}
