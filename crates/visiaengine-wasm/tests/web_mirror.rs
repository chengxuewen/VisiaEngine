//! CAPI-09 web/C 双面镜像（#[ignore]=web-check 显式 T2 通道，依赖构建产物）。

// spec: CAPI-09
#[test]
#[ignore = "web-check 链：需 scripts/web-build.sh 产物先行 [FFI-R:v13-FEAS-7]"]
fn web_capi_pair_mirror() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/web-mirror.mjs");
    let out = std::process::Command::new("node")
        .arg(script)
        .output()
        .expect("node 不可用：pixi 环境内跑（default 含 nodejs）");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "mirror 失败：stdout={stdout}\nstderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("MIRROR 3/3"), "缺 MIRROR 完成行: {stdout}");
}
