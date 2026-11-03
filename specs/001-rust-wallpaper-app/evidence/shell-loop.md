# T018 — Win32 hidden message-window event loop and shutdown skeleton

**Status: PARTIAL — Windows-target compilation and deterministic shell signaling tests PASS; hosted Windows event-loop smoke and WindowsEnv acceptance pending. T018 remains unchecked until the native smoke result is available.**

- `rust/src/shell.rs` maps slideshow deadline, Explorer restart, display changes and shutdown to serializable `AppCommand` values for the T016 state reducer.
- Under `cfg(windows)`, `shell::native::run_smoke` owns its own **hidden STATIC HWND** on the current thread using T015 `OwnedWin32Window`, registers only a standard `TaskbarCreated` *message identifier*, and uses a blocking `GetMessageW`/TranslateMessage/DispatchMessageW event loop. It arms exactly one finite WM_TIMER, maps shell/display notifications, sends Quit to the state reducer, kills its own timer and destroys/checks its own HWND before returning.
- `rust/src/main.rs` only accepts `--shell-smoke` at this stage. With no arguments it exits and explicitly reports the production wallpaper app is still incomplete. Thus CI or accidental launches cannot create an invisible persistent background app. No autostart registry value, Explorer window parent/style, WorkerW, tray registration, GPU renderer or folder scan is modified.
- The single-thread-affine HWND ownership prevents cross-thread destruction in safe Rust. There is no idle animation draw/Present loop, background sampler, or polling timer outside the explicitly bounded smoke.
- **Important:** A hidden STATIC diagnostic is not the actual icon-behind desktop host or tray. T009 and later story tasks must prove shell changes/restart/recovery on WindowsEnv.

## TDD and validation — 2026-11-04

- RED: `cargo test --locked --test shell_contract --target x86_64-unknown-linux-gnu` initially failed with E0432 (missing `shell`).
- GREEN: 3/3 tests PASS for shell notification translation, no implicit action and paused Explorer restart behavior.
- Windows MSVC Clippy and cross-target build check: **PASS**.
- GitHub Actions hosted runner `--shell-smoke` actual HWND creation/timer/cleanup: **PENDING** until the new workflow executes.
- WindowsEnv 25H2 interactive shell behavior and WorkerW integration: **SKIPPED** until non-disruptive runtime command access is established.
