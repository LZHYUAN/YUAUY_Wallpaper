//! Windows 11 uses compatibility aware behavior only with a declared supportedOS.
#[test]
fn manifest_declares_windows_8_and_10_11_and_non_elevated_execution() {
    let manifest = include_str!("../assets/windows-app.manifest");
    assert!(manifest.contains("{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"));
    assert!(manifest.contains("{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"));
    assert!(manifest.contains("level=\"asInvoker\""));
    assert!(manifest.contains("uiAccess=\"false\""));
}

#[test]
fn msvc_build_links_manifest_into_every_executable() {
    let build = include_str!("../build.rs");
    assert!(build.contains("CARGO_CFG_TARGET_OS"));
    assert!(build.contains("CARGO_CFG_TARGET_ENV"));
    assert!(build.contains("rustc-link-arg-bins=/MANIFEST:EMBED"));
    assert!(build.contains("/MANIFESTINPUT:"));
}
