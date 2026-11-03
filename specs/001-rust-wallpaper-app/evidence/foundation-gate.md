# Phase 2 Foundation Gate — Rust + Windows 11 D3D11/WorkerW

**Status: BLOCKED for WindowsEnv desktop integration. T019 remains unchecked.**

The following measurements distinguish *hosted Windows CI*, *Hermes Linux cross-checks*, and *target WindowsEnv 25H2*. A successful hosted CI WARP smoke is **not** evidence of WorkerW being behind Explorer desktop icons.

| Gate | Source and recorded evidence | Status |
| --- | --- | --- |
| Git `rust_build`, independent C# reference, project toolchain | `rust/Cargo.lock`, `legacy_reference.json`, source SHA audit, GitHub actions | **PASS** |
| Windows x64 MSVC Release compilation | [GitHub Actions 41679718337](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41679718337) and earlier completed runs | **PASS** for tested commits |
| WARP D3D11 device, clear and Present(0) | `evidence/d3d11-warp.md`; hosted Windows runner real HRESULT 0x0 | **PASS on hosted CI only** |
| Win32 WorkerW topology and reversible visual behind-icon composition | `evidence/workerw.md`; hosted CI returned SKIPPED because child WorkerW absent | **NOT PROVEN**; WindowsEnv visual acceptance outstanding |
| Present(1), `max_fps` software limiter, 60/120/144/240 Hz matrix | `evidence/vsync-probe.md`; CI 64Hz hidden-window 12 real Present calls; unsupported test modes listed | **DIAGNOSTIC only**, actual VSync and hardware modes SKIPPED |
| Whole-system GPU/CPU quick bounded API | `evidence/load-probe.md`; hosted CI CPU 4%/200 ms, GPU PDH skipped at 310 ms before collect | **CPU probe PASS; GPU Unknown**, production guard NOT approved |
| Monotonic clock, error handling, monitor geometry, renderer contract, app state, test doubles | T012–T017 tests and evidence | **PASS as testable interfaces** |
| Hidden Win32 native message loop, bounded owned HWND and shutdown | `evidence/shell-loop.md`; [hosted Windows CI 41680522990](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41680522990) emitted `PASS shell_message_loop=true timer_fired=true state_stopped=true hwnd_destroyed=true` | **PASS for hidden-window skeleton on hosted CI**; WindowsEnv actual Explorer integration pending |
| Actual WindowsEnv x64 Rust/MSVC/SDK inventory | `evidence/windowsenv-baseline.md`; CUA driver health and Windows build 26200.9711 verified | **PARTIAL**; no non-disruptive terminal channel to query toolchain |
| Physical GPU, multiple monitors, high refresh | User-approved separate real-PC acceptance | **SKIPPED** |

## Actionable blocker and decision

- The Hermes Windows CUA driver reports an active x64 Windows 11 25H2 session, UI Automation and D3D11 screenshot support. This proves transport health only.
- `ssh WindowsEnv` does not resolve as a configured hostname; this alone does not imply the Windows machine is offline. A CUA minimized-background PowerShell probe was refused by Windows foreground focus protection; the driver reported that **no process was started**. Do not retry with a potentially focus-stealing launch, change user workspace, or silently substitute another Windows computer.
- Because no authorized command/screenshot path has proven **an owned D3D11 WARP surface remains beneath desktop icons on WindowsEnv**, the Foundation Gate cannot be approved. Do not promote unsupported integration into a successful product release or substitute a GDI+ renderer. Continue only independent foundational work until the real integration path is verified.
- On an authorized non-disruptive WindowsEnv command channel: check OS/SDK/MSVC/Rust and active display modes; run `probe_d3d11 --warp` and `probe_workerw --inspect`; perform the **bounded** WorkerW attach visual inspection, prove cleanup, and record both console output and screenshot metadata. Exercise shell's bounded smoke, then assess T010, T011 with available display/GPU/CPU. Only then update T006/T008/T009/T010/T018/T019 checkboxes based on observed criteria.

No untouched C# branch was modified, no real user photo paths were used, and unrelated `.specify` or `.agents` local work has not been committed.
