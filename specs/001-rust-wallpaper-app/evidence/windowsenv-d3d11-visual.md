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

## Final WindowsEnv manifest-enabled D3D11 runtime findings (2026-11-04)

### Root cause fixed: Windows manifest for layered child HWND

The Windows CI release artifact for commit `163df7a86d3ce96df0ef6132ce14e127945aef5a` was downloaded to the same WindowsEnv, and the executable's SHA-256 was verified. `probe_win32_window.exe` SHA-256 was `696bba49f698ef6ef7045432f1c759ee9fc31034e2ea60036f0ca996e0a670b2`, and the manifest-enabled `probe_desktop_d3d11.exe` SHA-256 was `767f5603d6628f3fd7b1fc433be6775e89ad0932e98b572c9a2b8ef7e3034fdf`.

The native WindowsEnv diagnostic now reported **PASS for all** independent cases: `own_top_level`, `simple_child`, `layered_only`, `transparent_only`, `noactivate_only`, `layered_and_transparent`, `layered_and_noactivate`, `layered_child_no_disabled`, and `layered_child`, each with `valid=true last_error=0`. Before the Manifest change the same Rust `layered_child` test returned Win32 error 87. **This confirms the executable compatibility-manifest fix on WindowsEnv** rather than merely on hosted CI.

### Real WARP rendering to a child of Progman

Executed on WindowsEnv:

```powershell
& "$env:TEMP\YUAUY-Win25-Probes\target\x86_64-pc-windows-msvc\release\probe_desktop_d3d11.exe" --attach-modern --duration-ms 10000
```

The result file reported (exact fields):

```text
PASS driver=warp feature_level=0xb000 direct3d_present=true last_present_hresult=0x00000000 frames=125 progman_layered_child=true behind_defview_zorder=true child_was_visible=true destroyed=true duration_ms=10000 visual_icon_occlusion=REQUIRES_SCREENSHOT
```

The **125 successful Direct3D11 WARP Clear/Present submissions** and the `Progman` child HWND / `SHELLDLL_DefView` z-order read-back are native WindowsEnv measurements, not only CI. Cleanup checked `IsWindow` false after the 10-second window lifetime. The same program run from a hidden PowerShell host also returned PASS without a covering console.

### Independent CUA image comparison — visible desktop FAIL

The full-resolution WindowsEnv 1920×1080 desktop capture was taken **during** a hidden-console test run. Windows Task Manager independently showed the `probe_desktop_d3d11.exe` process running at the time of one capture. Image analysis of the intended **x=12..231, y=12..201** test region (220×190 pixels) compared the baseline image with two capture frames and a post-cleanup frame.

| Snapshot | SHA-256 of private PNG | Changed pixels with channel delta >30 | Lime-green pixels |
| --- | --- | --- | --- |
| Baseline | `523893c79e2fdbdcf2ff8a271fd83faf57d656afa2e79f6f9a92da4d50d4195b` | 0 | 95 |
| D3D11 running #1 | `6e2f13eeca0fd7a6798b3a02d1b9f5fd67fd851effbd19cfe0148700c62bc79a` | **0** | 95 |
| D3D11 running #2 | `76e030392bd0be0690c6b21fab9390fe0422765cb431519defd93d330b6e93a9` | **0** | 95 |
| After cleanup | `65469ec7ab5b377a80279ee0c12f29d8122d2fa90ef01a9a1f1aa26227ce8823` | **0** | 95 |

The expected **bright green D3D11 image was not visible** in this region. The region bytes matched exactly across all four screenshots despite successful Present calls and correct HWND z-order. The screenshots contain the user's desktop and are held **privately, Git-ignored** in `rust/target/private-evidence/windowsenv-25h2/`; **no PNG screenshot was committed**.

A supplemental short-lived test moved **only the probe-owned child HWND** above the icon view; native code reported `SetWindowPos=True` and `above_defview=True`, while its 125 Present calls still succeeded. The available screenshot did not conclusively capture the program while it was in that above-icon position; therefore **do not assert an above/below visual comparison PASS**. The separate GDI marker control also did not yield a conclusive *live* visual, so there is not enough evidence yet to distinguish an opaque DefView surface from unsupported DXGI-to-layered-child composition.

### Verdict and cleanup

**PASS:** Windows 11 25H2 WARP device/swapchain/render-target/Present; manifest-enabled layered child HWND; same Progman/DefView owner; correct sibling z-order; repeatable process-owned HWND cleanup.

**FAIL / NOT VERIFIED:** **the Direct3D11 image did not become visible behind Windows desktop icons** under the current legacy `D3D11CreateDeviceAndSwapChain` + `WS_EX_LAYERED` Progman child design. The app's visible-wallpaper hosting feasibility is **not proven**. Do not mark T009 or T019 complete.

**Next technical investigation:** test DXGI flip-model/`CreateSwapChainForHwnd` or DirectComposition's `CreateSwapChainForComposition` hosted in a suitably constructed child; capture *running, visible* A/B screenshots to distinguish icon view opacity from swap-chain composition. Keep existing Explorer HWNDs untouched and reset only owned diagnostic HWNDs. No `master`/C# formula modifications or permanent system changes occurred.
