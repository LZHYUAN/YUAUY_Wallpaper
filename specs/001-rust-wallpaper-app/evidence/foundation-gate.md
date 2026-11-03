# Phase 2 Foundation Gate — Rust + Windows 11 D3D11/WorkerW

**Status: BLOCKED for WindowsEnv desktop integration. T019 remains unchecked.**

The following measurements distinguish *hosted Windows CI*, *Hermes Linux cross-checks*, and *target WindowsEnv 25H2*. A successful hosted CI WARP smoke is **not** evidence of WorkerW being behind Explorer desktop icons.

| Gate | Source and recorded evidence | Status |
| --- | --- | --- |
| Git `rust_build`, independent C# reference, project toolchain | `rust/Cargo.lock`, `legacy_reference.json`, source SHA audit, GitHub actions | **PASS** |
| Windows x64 MSVC Release compilation | [GitHub Actions 41679718337](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41679718337) and earlier completed runs | **PASS** for tested commits |
| WARP D3D11 device, clear and Present(0) | `evidence/d3d11-warp.md`; hosted Windows runner real HRESULT 0x0 | **PASS on hosted CI only** |
| Win32 WorkerW topology and reversible visual behind-icon composition | `evidence/workerw.md`; hosted CI returned SKIPPED because WorkerW absent; explicit legacy/raised-desktop reversible probes and 5 contract tests now available | **NOT PROVEN**; WindowsEnv visual acceptance outstanding |
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

**2026-11-04:** T009 probing strategy now distinguishes `WS_EX_NOREDIRECTIONBITMAP` raised desktops from legacy WorkerW hosting. The modern test is explicitly opt-in, bounded and restores its own HWND; the actual 25H2 desktop validation gate remains BLOCKED.

## Portable, read-only WindowsEnv acceptance bundle (2026-11-04)

The Windows CI workflow now uploads a short-lived (3-day retention) **non-production** artifact named `yuauy-foundation-probes-windows-x64`, containing only the four compiled diagnostic EXEs and `rust/scripts/foundation-inventory.ps1` — no user YAML, private pictures, logs or production wallpaper executable. The artifact is not a substitute for actual WindowsEnv execution.

`foundation-inventory.ps1` defaults to read-only OS/architecture and Rust/MSVC/SDK **presence checks**. Only `-RunReadOnlyProbes` additionally runs `probe_d3d11.exe --warp`, `probe_workerw.exe --inspect` and bounded `probe_usage.exe --metric cpu_total`; it never requests Explorer attachment, GPU calibration or a display mode change. **No `--attach` or `--attach-modern` is executed by CI or this script.**

When a trusted non-focus-stealing WindowsEnv PowerShell route exists, extract the artifact to a dedicated test directory and run:

```powershell
.\scripts\foundation-inventory.ps1 -ProbeDirectory .\target\x86_64-pc-windows-msvc\release -RunReadOnlyProbes
```

The exact path depends on artifact extraction layout. Save the redacted console output, identify machine class **WindowsEnv** versus **GitHub hosted runner**, then run WorkerW visual `--attach-modern` separately only in a supervised test session. A probe that says SKIPPED does not count as PASS.

## Hosted CI diagnostic bundle verification (2026-11-04)

[GitHub Actions workflow 41681902301](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41681902301), job `126077454969`, source commit `1325ac5e2d0813ddfc6c35ef6dc289cdd6a3870d`, completed **SUCCESS** (format, Clippy, MSVC release build, WARP, VSync diagnostic, CPU/GPU diagnostic, WorkerW read-only inspection, bounded hidden Win32 message loop, inventory, artifact upload and all 50 Windows-target deterministic tests). Actual inventory script output was:

```text
BASELINE os_windows=True version=10.0.26100.0 architecture=X64 source=local_process
TOOL rustc=True cargo=True cl_in_path=False link_in_path=True
TOOL visual_studio_cpp_detected=True sdk_registry_key_present=True
PROBE probe=probe_d3d11.exe status=PASS api_only=true
PROBE probe=probe_workerw.exe status=SKIPPED api_only=true
PROBE probe=probe_usage.exe status=PASS api_only=true
VERDICT diagnostics_only=true desktop_layering=UNVERIFIED production_wallpaper=UNVERIFIED
```

**Environment distinction:** `10.0.26100.0` above is the **GitHub hosted Windows runner**, not the user's WindowsEnv CUA OS build `10.0.26200.9711`. The runner's `cl_in_path=False` does not mean the compiler is absent, since the separately detected Visual Studio C++ workload and Windows CI build succeeded. None of this establishes which tools are installed inside WindowsEnv.

The Actions run uploaded one checked `yuauy-foundation-probes-windows-x64` artifact (artifact ID `13252888438`, size `348854` bytes, expiry `2026-11-06T17:13:50Z`; user download via the workflow run page). It contains the four probe executables and the read-only inventory script, **not** the product executable, personal images or configuration. The verified bundle can be transferred into WindowsEnv when a safe command/file route exists; it **has not** been executed on that target. T006, T008, T009, T010 and T019 therefore remain unchecked.
