# YUAUY Wallpaper — Rust workspace

This crate is the **incomplete Rust/Direct3D 11 scaffold**, not yet a working wallpaper. It targets Windows 11 24H2+ on `x86_64-pc-windows-msvc`. No changes to the preserved C# implementation are required.

## Build and test on Windows

Install Visual Studio 2022 or newer with Desktop development with C++ (MSVC x64 build tools and a Windows SDK), then install [rustup](https://rustup.rs/). The pinned `rust-toolchain.toml` selects Rust, rustfmt, clippy, and the Windows MSVC target. From `rust/` in a **Developer PowerShell**:

```powershell
rustup show active-toolchain
cargo fmt --all -- --check
cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings
cargo build --locked --release --target x86_64-pc-windows-msvc
cargo test --locked --target x86_64-pc-windows-msvc
```

The `--locked` commands require a reviewed, committed `Cargo.lock` (T002). Do not interpret a GitHub workflow definition as evidence that the build has passed. This scaffold intentionally exits unsuccessfully until the Win32 event loop and renderer are implemented.

## WindowsEnv and acceptance

Use the procedure in [WindowsEnv quickstart](../specs/001-rust-wallpaper-app/quickstart.md). WindowsEnv's GPU-less WARP path, desktop WorkerW layering, display mode, system tray and CPU load-gate tests are separate from format/lint/build/test CI and must be recorded as **PASS**, **FAIL**, or **SKIPPED** with real evidence. Hardware GPU, physical multi-monitor and high-refresh measurements require separate real-machine validation; do not promote a WARP test to proof of hardware performance. Avoid changing registry autostart or the desktop outside the explicitly scoped interactive acceptance tests.

## Privacy and test fixtures

Use only synthetic generated photographs or approved public/sample inputs. Do **not** commit personal photos, arbitrary image collections, user-specific configuration, absolute private paths, machine logs, auth tokens, screenshots containing private data, or real registry exports. Use `rust/photos/`, `rust/images/`, `rust/screenshots/`, `rust/private-logs/` or `specs/001-rust-wallpaper-app/evidence/private/` for local private fixtures; these are Git-ignored. Keep `specs/001-rust-wallpaper-app/contracts/config.example.yaml` tracked as the public specification example. Preserve the original normal-animation formula and monitor-selection behavior for parity fixtures; only the approved intro choreography differs.
