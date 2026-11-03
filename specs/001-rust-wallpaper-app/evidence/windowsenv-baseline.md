# WindowsEnv baseline — T006 (partial, not acceptance)

**Observation date**: 2026-11-03. **Target**: the explicitly authorized WindowsEnv session reached through Hermes Link CUA. Status is **PARTIAL**; T006 is left unchecked until installed toolchains and device capabilities are independently established.

| Check | Status | Evidence and limitation |
| --- | --- | --- |
| OS version and architecture | **PASS** | `mcp__win_cua__health_report` returned `win32`, x86_64, Windows `10.0.26200.9711` (Windows 11 25H2 build family). Driver version `0.37.0`; interactive session active. |
| Desktop UIA accessible | **PASS** | CUA driver `ax_capability: pass`, UIAutomation reachable. This is driver health, not app integration. |
| D3D11 available for desktop capture | **PASS** | CUA driver `screen_capture_capability: pass`: its D3D11 device supports Windows Graphics Capture. Does **not** prove that the app can create its own WARP device/swapchain. |
| Win32 D3D11 WARP clear/present | **SKIPPED** | No wallpaper-specific executable has yet run on WindowsEnv. T008 must test software device creation and an actual Present operation. |
| WorkerW behind icons | **SKIPPED** | No reversible WorkerW probe has run. T009 is required. |
| Rust/cargo installed on WindowsEnv | **SKIPPED** | Could not run read-only Windows CLI discovery; no version recorded. Do not confuse the pinned Linux host toolchain or GitHub Actions runner with WindowsEnv. |
| MSVC compiler/linker on WindowsEnv | **SKIPPED** | CLI not accessible for `cl.exe`/`link.exe` detection. |
| Windows SDK on WindowsEnv | **PARTIAL** | CUA installed application inventory contains Windows Kits debugger and performance-tool launchers; this does **not** establish headers, libraries, full SDK installation or compiler usability. |
| Physical GPU presence/absence | **SKIPPED** | User documents a GPU-less WindowsEnv, but no live `Win32_VideoController` or DirectX adapter enumeration has confirmed it here. |
| Screen configuration and refresh rate | **SKIPPED** | No graphics probe yet. |
| Windows 11 MSVC build on external CI | **PASS (separate environment)** | [GitHub Actions run 41618507709](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41618507709) on `67b744d`: format, Clippy, x64 MSVC release build and deterministic tests all succeeded. This is not WindowsEnv runtime evidence. |

## Access attempt and fallback decision

- Hermes Linux host `ssh WindowsEnv` failed with `Could not resolve hostname windowsenv: Name or service not known`; this is a hostname-resolution issue, not proof the target is powered off.
- `mcp__win_cua__health_report` successfully reached the Windows interactive session and its driver.
- A background-minimized PowerShell **read-only** toolchain discovery launch was refused by CUA: `Windows did not grant the foreground lock required to prevent the new process from activating. No process was started`. Because user focus/work must be preserved, no foreground-changing retry was attempted.
- Do not create a shortcut, change OS security policy, reboot or infer installed tools from filenames. Once a verified non-disruptive terminal/SSH path exists, run `Get-ComputerInfo`, `rustc --version`, `cargo --version`, `where.exe cl`, `where.exe link`, SDK discovery and video-adapter enumeration, then execute the D3D11/WARP probe.

**Verdict:** Win32 remote desktop connectivity and D3D11 capture-driver health are proven; WindowsEnv toolchain availability, app WARP Present and WorkerW layering remain **unverified**, not **PASS**. The user-approved original C# formula and old branches have not been modified.

## Live recheck — 2026-11-04 (Hermes CUA, read-only)

The `mcp__win_cua__health_report` again returned an active Windows x86_64 session on OS build **10.0.26200.9711** with `ax_capability: pass` and `screen_capture_capability: pass`. A read-only `mcp__win_cua__list_windows` call listed **5 top-level windows**, including **Explorer's Program Manager**; this proves a shell UI exists but does **not** establish a usable nested WorkerW, correct Z-order or desktop-icon layering. No GUI input was issued.

The Hermes Linux SSH configuration contains no WindowsEnv alias and `getent hosts windowsenv` returned no address; an approved, non-focus-stealing Windows command path is still **not available**. An absent SSH alias is not evidence that the Windows machine is powered down. No PowerShell launch or focus-stealing retry was attempted in this recheck.

For future acceptance without requiring Rust/MSVC on WindowsEnv, the dedicated GitHub Windows CI workflow can upload a 3-day **diagnostics-only** probe artifact and the read-only `rust/scripts/foundation-inventory.ps1`. That artifact is evidence of **hosted CI build**, not WindowsEnv installation or execution. The actual WindowsEnv Rust/MSVC/SDK inventory, app WARP Present, WorkerW visual proof and physical GPU presence all remain explicitly **SKIPPED/UNVERIFIED**.
