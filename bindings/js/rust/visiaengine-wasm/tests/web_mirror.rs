//! CAPI-09 web/C dual-face mirror (#[ignore] = the explicit T2 channel of web-check, depends on build artifacts).

// spec: CAPI-09
#[test]
#[ignore = "web-check chain: requires scripts/web-build.sh artifacts first [FFI-R:v13-FEAS-7]"]
fn web_capi_pair_mirror() {
    let script = {
        fn repo_root() -> std::path::PathBuf {
            let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            while !p.join("Cargo.lock").exists() {
                assert!(p.pop(), "walking up for Cargo.lock missed");
            }
            p
        }
        repo_root().join("scripts/web-mirror.mjs")
    };
    let out = std::process::Command::new("node")
        .arg(script)
        .output()
        .expect("node unavailable: run inside the pixi environment (default carries nodejs)");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "mirror failed: stdout={stdout}\\nstderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("MIRROR 3/3"),
        "MIRROR completion line missing: {stdout}"
    );
}
