# WindowsEnv 25H2 — DirectComposition/DefView z-order A/B acceptance

**Date:** 2026-11-04, Asia/Taipei (UTC+08).
**Verdict:** The WARP DirectComposition image is **visible ABOVE** `SHELLDLL_DefView`, but **not visible BELOW** it. API submission succeeds in BOTH cases. The window, process, shell and screenshot restoration checks **PASS**. Behind-icon wallpaper acceptance **FAILS**; T009/T019 remain **unchecked**.

## Identity and safeguards

- Branch `rust_build`, source commit `1e982583c86ced63d1163ddac75ac9b3441681b5`. Only the opt-in `probe_desktop_dcomp.exe` diagnostic was changed; the C# original, `master`, `directx_build`, YAML, autostart, Explorer-owned HWNDs and real wallpaper settings were not modified.
- GitHub hosted Windows x64 [run 41699312039](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41699312039) completed **SUCCESS** for the committed source. The exact Windows x64 CI diagnostic came from nonproduction artifact `yuauy-foundation-probes-windows-x64`, artifact ID `13261413714`.
- SHA-256 of `probe_desktop_dcomp.exe`: `97a73b1e02b21c8a5cf143d23e6bfdbb6c76ae5d3a48c6907a586b97737eca46`; **verified identical on Hermes Linux and native WindowsEnv** before execution.
- Target: **WindowsEnv** Windows 11 25H2 build **26200.9711**, primary display **1920×1080**. Existing Explorer `Program Manager` HWND **65788**, Explorer PID **6864**, Task Manager PID **4568**.
- `probe_desktop_dcomp.exe --inspect` returned `raised=true same_icon_owner=true matches_Explorer=true attach_eligible=true` before and after both tests.
- A and B use the **same** owned 220×190 `Progman` child, Win32 styles, WARP D3D11 device, flip-sequential DXGI composition swapchain, DirectComposition visual tree, saturated magenta clear and bounded rendering loop. Only `SetWindowPos` insertion relative to `SHELLDLL_DefView` changes. Explicit sibling order, parent and window visibility are read back during each run.
- The above-icon-view mode is only accepted on explicit bounded modern attachment: `--attach-modern --duration-ms 10000 --above-icon-view`. No undocumented `0x052C` shell message, foreign-window reparenting/restyling, registry write, Explorer restart, display change or permanent replacement was requested.

## Native runtime evidence

| Mode | Command suffix | Window | Start (UTC+08) | End (UTC+08) | Exit | WARP Presents | HRESULT | Owned child |
| --- | --- | --- | --- | --- | ---: | ---: | --- | --- |
| **A BELOW** | `--attach-modern --duration-ms 10000` | PID 6712 | 03:05:05.737 | 03:05:15.832 | 0 | 606 | `0x00000000` | destroyed |
| **B ABOVE** | `--attach-modern --duration-ms 10000 --above-icon-view` | PID 7860 | 03:05:20.941 | 03:05:31.005 | 0 | 606 | `0x00000000` | destroyed |

Both console logs printed `API_SUBMISSIONS_ONLY path=dcomp_composition_flip_sequential`, the respective `placement=below_defview` / `placement=above_defview`, `sibling_order_verified=true`, `present_s_ok=606`, `busy=0`, `child_destroyed=true`, `visual_wallpaper=UNVERIFIED` and `requires_live_screenshot=true`. The diagnostic itself never treats successful Present as visual proof.

## Independent visual evidence

The controller was minimized before testing. It captured 220×190 on-screen ROI screenshots at x=12..231, y=12..201 at 2, 4 and 6 seconds after each native launch. A deterministic RGB sample compared every fourth pixel in both dimensions against a per-mode baseline (2640 samples), with a >30 channel delta as changed and `R>170,G<70,B>100` as magenta.

| Mode / sample | Sampled pixels | Changed vs mode baseline | Magenta |
| --- | ---: | ---: | ---: |
| A BELOW, t≈2s | 2640 | **0** | **0** |
| A BELOW, t≈4s | 2640 | **0** | **0** |
| A BELOW, t≈6s | 2640 | **0** | **0** |
| B ABOVE, t≈2s | 2640 | **2640** | **2640** |
| B ABOVE, t≈4s | 2640 | **2640** | **2640** |
| B ABOVE, t≈6s | 2640 | **2640** | **2640** |
| Each mode, after process exit | 2640 | **0** | **0** |

Separately, the Windows CUA driver independently captured full-resolution **1920×1080** desktop frames (not controller-generated screenshots). Four during-B frames at **03:05:22.841**, **03:05:24.893**, **03:05:27.301** and **03:05:30.525** showed **41,800 / 41,800** pixels changed AND matching saturated magenta inside the entire 220×190 rectangle. The magenta filled the rectangle and visually covered icons within it. Frames before the B surface appeared and after its expiry showed **0 / 41,800** magenta pixels. Thus the above-icon-view case is visibly confirmed independently.

Selected **private**, Git-ignored full-frame screenshot hashes (SHA-256):

| Capture | SHA-256 |
| --- | --- |
| Baseline before this investigation | `a3abdbe7a81a6346cf99d07122791b30fe96633fab3d1cd727548c5d77835284` |
| B during #1 (03:05:22.841) | `29ea9404c39d078f1f5231258c2cf495a0f36ae85ec02ea94142bd289e0447e5` |
| B during #2 (03:05:24.893) | `e81959cf75d8c7197fd9a8021fc566bac30f1d62ef40283e011f7a72cca02d7c` |
| B during #3 (03:05:27.301) | `abf94bd48f45d66d692f63036438c82345bf88f3770406461565fa6fe0343664` |
| B during #4 (03:05:30.525) | `b3bc5783f9c9d18243a32324aca0d4603d0428241b181510b6738e9ce16955dd` |
| Fully cleaned desktop | `d93a1c309caedfc62896783ba0af36afc87bfd06c24f9973865c1a165126dcfe` |

Private screenshots are retained under `rust/target/private-evidence/windowsenv-25h2/ab-1e98258/`, outside Git. The two controllers' temporary Windows screenshots/logs are stored locally under `%TEMP%\YUAUY-Win25-Probes\ab-1e98258`; personal desktop PNGs were **not** committed.

## Cleanup and original desktop preservation

- The two bounded diagnostic EXEs exited with code 0 and `child_destroyed=true`, and a separate process readback returned `PROBE_RUNNING=False`.
- Both test controllers' own PowerShell/CMD windows were closed; a fresh `list_windows` returned **no remaining PowerShell/CMD or diagnostic windows**.
- Explorer PID **6864** and `Program Manager` HWND **65788** persisted, along with the pre-existing Task Manager PID **4568**.
- A CUA screenshot after all test controllers closed matched the pre-run screenshot **exactly** in ROI (220×190), the desktop icon strip `x=0..466,y=0..1039` and unobstructed upper-right `x=467..1919,y=0..354`. Some dynamic taskbar/other application regions were excluded from pixel-equality claims.
- No Explorer-owned HWND was moved or modified, and no wallpaper/image settings or display modes were changed. This is **scoped** cleanup verification, not an assertion that every internal Explorer/DWM property was inspected.

## Interpretation / next engineering gate

**Established:** The same WARP+DXGI+DirectComposition rendering pipeline visibly works in the **ABOVE** sibling position; it is invisible **BELOW** DefView. Therefore the failure is localized to desktop DefView underlay composition / clipping / occlusion on this raised desktop, **not** an inability to issue or display WARP DirectComposition frames generally. The existing product requirement (wallpaper **behind clickable icons**) is **NOT** satisfied by the above-icon-view success; that mode covers desktop icons and is diagnostic-only.

**Not established:** Whether DefView has opaque alpha for this shell state or whether some DWM child-surface clipping rule prevents underlay composition. The A/B result distinguishes Z-order from a general rendering failure but does not measure actual DefView alpha. A separate read-only `GetLayeredWindowAttributes` attempt was inconclusive and is **not** used as evidence.

Reference context: [Microsoft CreateTargetForHwnd](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nf-dcomp-idcompositiondevice-createtargetforhwnd), [Microsoft SetWindowPos](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos), and [Lively's Win11 raised-desktop investigation](https://github.com/lively-community/lively/issues/2074) (community observations about DefView transparency vary by build/HDR/animation state; not an unconditional Microsoft guarantee).

**Gate:** Keep **T009 / T019 unchecked** and Phase 2 **BLOCKED**. Do not ship above-icon-view overlay as a wallpaper workaround or modify the original C# animation formulas. A further bounded diagnostic may compare an **opaque layered** child and explicit visual controls below DefView, or investigate supported shell composition states without altering existing Explorer-owned windows. Record screenshot evidence before any acceptance claim.
