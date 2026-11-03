//! The on-desktop D3D11 probe is a separate explicit binary; plain invocation
//! must not modify any desktop host on non-Windows test environments.
use std::process::Command;

#[test]
fn standalone_desktop_probe_can_be_launched_safely_in_readonly_mode() {
    let out = Command::new(env!("CARGO_BIN_EXE_probe_desktop_d3d11"))
        .arg("--inspect")
        .output()
        .expect("probe executable must exist");
    #[cfg(not(windows))]
    {
        assert_eq!(out.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&out.stderr).contains("SKIPPED"));
    }
    #[cfg(windows)]
    assert!(matches!(out.status.code(), Some(0) | Some(2)));
}
