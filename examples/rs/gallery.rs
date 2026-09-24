//! Gallery thumbnail helper — included by E-series examples via `#[path]`.
//!
//! One call after a `--frames` headless render:
//! `gallery::save_frame(&img, "E101_clear");`
//! writes `build/gallery/assets/<name>.png`. Best-effort by contract: failures
//! warn, never assert — gallery generation must not break example CI semantics.
//!
//! B1 super-band: thumbnails stay in lockstep with example behavior because
//! the EXAMPLE renders them (zero drift, zero re-implementation).

/// Best-effort PNG dump of a rendered frame into the gallery asset tree.
pub fn save_frame(img: &visiaengine_render_wgpu::OffscreenFrame, name: &str) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/gallery/assets");
    if std::fs::create_dir_all(dir).is_err() {
        eprintln!("gallery: mkdir failed, thumbnail skipped ({name})");
        return;
    }
    let path = format!("{dir}/{name}.png");
    match image::RgbaImage::from_raw(img.width, img.height, img.rgba.clone()) {
        Some(rgba_img) => {
            if let Err(e) = rgba_img.save(&path) {
                eprintln!("gallery: save {path} failed: {e}");
            }
        }
        None => eprintln!("gallery: rgba size mismatch for {name}"),
    }
}
