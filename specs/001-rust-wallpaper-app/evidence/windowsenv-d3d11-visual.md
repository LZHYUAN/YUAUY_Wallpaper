# WindowsEnv 25H2 — Direct3D11 behind-icon acceptance (T008/T009/T019)

**Status: IN PROGRESS; no behind-icon visual acceptance result yet.**

## Actual WindowsEnv execution (2026-11-04)

Hermes Windows CUA driver health: Windows 11 x64 OS `10.0.26200.9711`, UI Automation and Graphics Capture PASS. Successfully launched a background-visible PowerShell (without installing Rust or overriding Windows execution policy). Downloaded the exact tested GitHub Actions artifact `yuauy-foundation-probes-windows-x64` from a short-lived GitHub artifact storage redirect. Its D3D11 EXE SHA-256 matched the reviewed CI artifact:

```
D3D11_HASH_OK=True
PASS driver_requested=warp feature_level=0xb000 present_hresult=0x00000000 occluded=false elapsed_ms=23 window_visible=false
D3D11_EXIT=0
SKIPPED: WorkerW child of Progman absent; no shell message sent
WORKERW_EXIT=2
```

WARP device, swapchain and one Present succeeded on WindowsEnv, **but that first probe used a hidden window**. Therefore this is **not** evidence of rendering behind icons.

## Actual 25H2 Explorer topology read-back

A second explicitly read-only PowerShell Win32 P/Invoke enumeration (not a guess from CI) reported:

```
TOPLEVEL: 10 WorkerW windows (parent=<desktop>)
Progman: HWND=65788 EXSTYLE=0x00200080
Progman child SHELLDLL_DefView: HWND=65794 EXSTYLE=0x00080000
SHELLDLL_DefView child SysListView32: HWND=65796
Progman has NO direct WorkerW child.
```

`WS_EX_NOREDIRECTIONBITMAP (0x00200000)` is set on Progman. Original WorkerW probe incorrectly assumed every 25H2 desktop has a direct WorkerW child. Added platform-independent failing-then-passing topology tests, corrected `probe_workerw --inspect` to use Progman/DefView without requiring WorkerW in raised mode, and introduced a dedicated opt-in `probe_desktop_d3d11.exe --attach-modern --duration-ms 9000`:

- Requires raised Progman, direct DefView child, same Explorer owner PID, and explicit modern flag. Never sends undocumented Progman messages, restyles/reparents Explorer, writes HKCU Run, or modifies permanent wallpaper settings.
- Creates an **owned** 220×190 layered Progman child, places only this child immediately below DefView, creates a real Direct3D11 WARP swap chain on that child, repeatedly clears the back buffer in bright green and calls Present for at most 10 seconds.
- Writes redacted results to `%TEMP%\YUAUY-Win25-Probes\desktop_probe_result.txt`, destroys all COM resources and its own HWND; the exit record checks cleanup. Visual proof needs a CUA screenshot **while the child exists**, plus a screenshot after cleanup. A successful API HRESULT or Z-order relationship **alone** is insufficient.
- CI runs `--inspect` only (no visible desktop mutation). Native WindowsEnv execution and icon-occlusion screenshot remain PENDING until the compiled binary is available from CI.

Task gates remain separate. T008's WARP hidden-swapchain feasibility is now verified specifically on WindowsEnv, but actual icon-behind D3D11 acceptance is still to be determined.

## Failed layered child creation and manifest root-cause investigation (2026-11-04)

A visual run of `probe_desktop_d3d11.exe --attach-modern --duration-ms 10000` on **WindowsEnv 25H2** returned `FAIL CreateWindowExW HRESULT=0x00000000`. The 220×190 test region in its CUA desktop screenshot was pixel-identical to its baseline (0 changed pixels), and no Direct3D11 Present call was reached. The failure is **not** evidence that Direct3D11 desktop rendering is inherently impossible.

A follow-up independent Rust Win32 test binary `probe_win32_window.exe`, source commit `21538b1`, reported on the SAME WindowsEnv:

```text
PROGMAN_EXISTS=true
CASE=own_top_level PASS valid=true last_error=0
CASE=simple_child PASS valid=true last_error=0
CASE=layered_child FAIL hresult=0x80070057 last_error=87
```

A separate read-only/reversible PowerShell C# P/Invoke diagnostic created/destroyed its own 220×190 `WS_EX_LAYERED` child under Progman with identical `WS_CHILD|WS_CLIPSIBLINGS|WS_DISABLED` and `WS_EX_LAYERED|WS_EX_TRANSPARENT|WS_EX_NOACTIVATE` flags, returning successful HWND and Win32 error 0. This **isolates a Rust EXE compatibility-context difference** rather than a blanket Win32 prohibition. An additional C# control test returned `alpha=True pos=True below=True FILL=1`, then `CLEANUP requested` for its temporary child; the captured screen did not conclusively show the marker, so icon-behind visibility remains unverified.

**Root-cause hypothesis grounded in Microsoft docs:** [Microsoft Using Windows](https://learn.microsoft.com/en-us/windows/win32/winmsg/using-windows) states that `WS_EX_LAYERED` child windows require a Windows 8-aware **application manifest**. [Microsoft Application (executable) manifest](https://learn.microsoft.com/en-us/windows/compatibility/application-executable-manifest) documents that an unmanifested EXE receives legacy Windows Vista compatibility behavior even on newer OS versions. The Rust crate did not embed a supportedOS manifest, unlike PowerShell/.NET.

Added `rust/assets/windows-app.manifest` with official Windows 8, 8.1 and Windows 10/11 supportedOS GUIDs and `asInvoker/uiAccess=false`; `rust/build.rs` passes `/MANIFEST:EMBED` and `/MANIFESTINPUT` to MSVC LINK for every Windows binary. This requires a **new Windows MSVC build and actual WindowsEnv retest**, including visual evidence; do not mark T009/T019 complete based on compilation alone.
