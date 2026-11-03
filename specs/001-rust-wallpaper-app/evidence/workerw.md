# T009 — WorkerW desktop-host feasibility probe

**Status: PARTIAL — code and automated CLI safety tests pass; WindowsEnv behind-icon visual test is unverified. Do not check T009 yet.**

## Implemented behavior

- `rust/src/bin/probe_workerw.rs` defaults to `--inspect`: read-only enumeration of `Progman`, direct child `WorkerW`, direct child `SHELLDLL_DefView`, owner PID consistency, rectangle and visibility, and `Progman`'s `WS_EX_NOREDIRECTIONBITMAP` hint.
- If prerequisites are missing, exit code **2** with `SKIPPED` (not a false PASS); a Win32 API failure exits **1**.
- Only `--attach` creates an **owned** 110×40 disabled and non-activating `BUTTON` child inside an **already-existing** WorkerW. The child pumps its own messages for **3 seconds** by default (maximum 10 seconds via `--duration-ms`), and RAII destroys the HWND with a post-destruction `IsWindow` check.
- There is no call to undocumented `Progman` `0x052C`; the probe never changes shell window styles, display z-order, registry, icons, wallpapers, or the original C# code. No worker window is created when absent.
- The probe **does not claim** a valid parent relationship proves the window is actually composed behind icons; that requires an actual WindowsEnv screenshot and visual inspection while the child exists.

## Validation (2026-11-03)

| Test | Status | Evidence |
| --- | --- | --- |
| CLI input safe default / explicit bounded attach / reject unsafe options | **PASS** | TDD: `cargo test --locked --test workerw_probe_contract --target x86_64-unknown-linux-gnu`, 3/3 passed after initial expected missing-module failure |
| Windows x64 MSVC cross-build type check / Clippy | **PASS** | `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings` |
| Linux unit/integration suite | **PASS** | 7/7 passed |
| Windows hosted CI release build | **PASS** | [CI run 41624317570](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41624317570), commit `8a45e4a`: MSVC release build and deterministic tests passed |
| Windows hosted CI read-only WorkerW inspection | **SKIPPED (expected)** | Executed binary, but returned `SKIPPED: WorkerW child of Progman absent; no shell message sent` (worker not present on hosted runner) |
| WindowsEnv 25H2 desktop layering | **SKIPPED** | No approved, non-focus-stealing native Windows CLI/SSH channel available yet; CUA UIA/graphics capture health is not a WorkerW application test |
| WindowsEnv temporary child cleanup / Explorer restart recovery | **SKIPPED** | Requires WindowsEnv interactive acceptance; never assume success from CI |

## Manual WindowsEnv acceptance

When a verified **non-disruptive** WindowsEnv terminal is available, build and run:

```powershell
cd <project>\rust
cargo run --locked --release --bin probe_workerw -- --inspect
cargo run --locked --release --bin probe_workerw -- --attach --duration-ms 5000
```

While attaching, observe screenshot/window tree and prove this process-owned visible test square lies behind desktop icons. After the 5-second expiry verify it disappeared, no extra WorkerW is left, no Explorer HWND style/parent changed, and desktop icons still work. Record OS build, exact console output and screenshot metadata without personal desktop images. Repeat only in a test session; do **not** trigger a shell reset.

## References and design risk

- [Microsoft SetParent docs](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent): SetParent does not automatically reconcile `WS_CHILD`/`WS_POPUP`, and cross-process DPI compatibility is risky. This probe therefore **creates** the child with the correct parent rather than reparenting an existing user window.
- [Windows 11 24H2 community analysis](https://github.com/lively-community/lively/issues/2074): Progman/SHELLDLL_DefView/WorkerW composition changed and a seemingly correct HWND parent does not prove visual rendering. Treat these community notes as hypotheses until WindowsEnv verification.

**Next gate:** Acquire scoped WindowsEnv command execution or an explicitly authorized non-disruptive launch path, test the attached temporary control, then update this report and task checkbox only if all acceptance points pass.

## GitHub CI execution details (2026-11-03)

The [Windows workflow run 41624317570](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41624317570) completed successfully, including Windows release build, Windows Clippy and deterministic tests. Job `125877697232` emitted exactly:

```text
PASS driver_requested=warp feature_level=0xb000 present_hresult=0x00000000 occluded=false elapsed_ms=14 window_visible=false
SKIPPED: WorkerW child of Progman absent; no shell message sent
```

The second line is **NOT** a WorkerW acceptance PASS. It means the read-only probe correctly refused attachment on a hosted Windows runner without a usable WorkerW. WindowsEnv 25H2's actual desktop remains the outstanding execution/visualization gate.
