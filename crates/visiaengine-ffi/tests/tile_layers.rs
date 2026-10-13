//! Band T (plan `ledger-tiles-keys-2026-10-08`): a raster basemap and a vector tile
//! layer coexist on one handle, and same-kind re-mount replaces rather than stacks
//! (per-kind addressing, Adjudication 2 = A). Clause anchors: CAPI-28/29/39/40.

use visiaengine::{
    visiaengine_create_headless, visiaengine_destroy, visiaengine_load_mvt_dir,
    visiaengine_load_raster_dir, visiaengine_readback, visiaengine_render,
    visiaengine_set_raster_view, visiaengine_set_tile_view,
};

const W: u32 = 320;
const H: u32 = 240;

fn fixture(rel: &str) -> std::ffi::CString {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !p.join("Cargo.lock").exists() {
        assert!(p.pop(), "no workspace root above");
    }
    p.push(rel);
    std::ffi::CString::new(p.to_str().expect("utf8 path")).expect("cstr")
}

/// Render and count pixels that differ from the frame's own corner colour.
/// The corner is sky/background in this scene family (same self-calibrating
/// reading the haze gates use — PIT-45: never compare against a typed literal).
fn coverage(ve: u64) -> usize {
    assert_eq!(visiaengine_render(ve), 0, "render must succeed");
    let mut buf = vec![0u8; (W * H * 4) as usize];
    assert_eq!(
        visiaengine_readback(ve, buf.as_mut_ptr(), buf.len() as u64),
        0
    );
    let c = [buf[0], buf[1], buf[2]];
    buf.as_chunks::<4>()
        .0
        .iter()
        .filter(|p| [p[0], p[1], p[2]] != c)
        .count()
}

fn mount_raster(ve: u64) {
    let ras = fixture("resources/data/raster");
    assert_eq!(
        visiaengine_load_raster_dir(ve, ras.as_ptr(), 10),
        4,
        "2x2 basemap"
    );
}

fn mount_vector(ve: u64) {
    let mvt = fixture("resources/data/tiles");
    assert_eq!(
        visiaengine_load_mvt_dir(ve, mvt.as_ptr(), 10),
        9,
        "3x3 vector"
    );
}

// spec: CAPI-28
// spec: CAPI-39
#[test]
fn raster_and_vector_layers_coexist_on_one_handle() {
    // single-kind references
    let only_r = visiaengine_create_headless(W, H);
    mount_raster(only_r);
    let cov_r = coverage(only_r);
    visiaengine_destroy(only_r);

    let only_v = visiaengine_create_headless(W, H);
    mount_vector(only_v);
    let cov_v = coverage(only_v);
    visiaengine_destroy(only_v);

    // both kinds on one handle, mounted in the order that used to self-erase.
    // Coverage is measured BEFORE any set_*_view call: those re-frame the camera
    // (CAPI-40's documented behaviour), and feeding a global bbox shrinks the
    // 2x2 basemap to a handful of pixels (measured: 8) — a test-design trap I
    // hit first, not an engine defect.
    let both = visiaengine_create_headless(W, H);
    mount_raster(both);
    mount_vector(both);
    let cov_b = coverage(both);
    let rv = visiaengine_set_raster_view(both, -2.0e7, -2.0e7, 2.0e7, 2.0e7);
    let vv = visiaengine_set_tile_view(both, -2.0e7, -2.0e7, 2.0e7, 2.0e7);
    visiaengine_destroy(both);

    println!(
        "COEXIST raster_only={cov_r} vector_only={cov_v} both={cov_b} visible(r,v)=({rv},{vv})"
    );
    assert!(
        rv > 0 && vv > 0,
        "both view entries must answer after both mounts: r={rv} v={vv}"
    );
    // Anti-erasure law. Before band T the second mount replaced the first, so the
    // vector mount wiped the basemap and cov_b collapsed to cov_v (measured:
    // 15771 vs 61440). The vector fixture lies outside the basemap-framed view,
    // so it may legitimately add no pixels here — the two-layer *visual* proof
    // lives in E207, and this gate deliberately claims only "nothing was erased".
    assert!(
        cov_b >= cov_r,
        "basemap must survive the vector mount: both={cov_b} raster_only={cov_r} (pre-band-T collapsed to {cov_v})"
    );
}

// spec: CAPI-40
#[test]
fn same_kind_remount_replaces_and_renders_identically() {
    let ve = visiaengine_create_headless(W, H);
    mount_raster(ve);
    mount_vector(ve);
    let first = coverage(ve);
    // same kind twice more: replace, never stack, never disturb the other kind
    mount_raster(ve);
    mount_vector(ve);
    let again = coverage(ve);
    visiaengine_destroy(ve);
    println!("REMOUNT cov_first={first} cov_again={again}");
    assert_eq!(first, again, "identical mounts must render identically");
}
