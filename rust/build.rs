//! Embed Windows 8+/Windows 11 compatibility, required for WS_EX_LAYERED on
//! a *child* HWND. Without a manifest, Windows supplies legacy Vista behavior
//! to Rust console EXEs and CreateWindowExW fails with ERROR_INVALID_PARAMETER.
//! This build-script changes only this crate's Windows MSVC binary images.
use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/windows-app.manifest");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
        || env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc")
    {
        return;
    }
    let dir = env::var_os("CARGO_MANIFEST_DIR").expect("Cargo must set CARGO_MANIFEST_DIR");
    let manifest = PathBuf::from(dir).join("assets/windows-app.manifest");
    println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
        manifest.display()
    );
}
