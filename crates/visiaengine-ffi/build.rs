//! Packaging-round P1: stamp an SONAME onto the cdylib.
//!
//! Why: without a SONAME the linker records whatever string appeared on its
//! command line as `DT_NEEDED`. Measured both ways on this machine:
//!   `cc app.c /abs/path/libvisiaengine.so` -> `DT_NEEDED=/abs/path/libvisiaengine.so`
//!   `cc app.c -Ldir -lvisiaengine`         -> `DT_NEEDED=libvisiaengine.so`
//! The first form bakes a build/install path into the consumer binary, which
//! makes an installed SDK tree non-relocatable. With a SONAME the library
//! declares its own name and the consumer always records the bare name.
//!
//! Gate: `scripts/gate-abi.sh` asserts the produced artifact's SONAME equals
//! `lib<[lib] name>.so` (derived live from Cargo.toml, so renaming the lib
//! without updating this file turns that gate red instead of silently passing).
//!
//! Platform coverage: Linux only, by design. macOS would use
//! `-install_name @rpath/libvisiaengine.dylib` and Windows has no SONAME
//! concept; both branches are declared gaps owned by the win/mac band
//! (`.omo/plans/packaging-round.md` §7.4 P1 / §7.5 open item 5). There is
//! deliberately no fallback `else` here: emitting a Linux flag on another
//! platform would fail the link, and emitting nothing silently would look
//! like support we have not verified.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // wasm32-unknown-unknown reports os="unknown"/arch="wasm32": emit nothing.
    // The linker there is wasm-ld and a POSIX soname would be rejected.
    if os == "linux" && arch != "wasm32" {
        // Name literal must match [lib] name = "visiaengine" in Cargo.toml;
        // gate-abi.sh is the machine check for that equality.
        println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,libvisiaengine.so");
    }
}
