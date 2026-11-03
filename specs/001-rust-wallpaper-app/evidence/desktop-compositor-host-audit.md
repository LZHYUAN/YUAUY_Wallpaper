# WindowsEnv 25H2 — DefView/DWM topology and non-mutating alternate-host diagnostic

**Date:** 2026-11-04 (Asia/Taipei, UTC+08).
**Verdict:** No **validated** D3D11 WARP/DirectComposition host below desktop icons was found using only windows owned by this diagnostic. `SHELLDLL_DefView` and `SysListView32` cover the entire primary screen; the available `WorkerW` windows are hidden and unusably small. An owned, unparented top-level DirectComposition popup draws above `Progman` but obscures desktop icons. Requests to place it *below* `Progman`, first by using the `Progman` HWND as a Z-order anchor and then via documented `HWND_BOTTOM`, are **not honored on this WindowsEnv**: real Z-order readback places the popup **above** `Progman`. The probe refuses to render on this unexpected placement, exits with failure and destroys the owned window. This does NOT prove that every possible Windows shell integration route is impossible.

**Gate:** T009 and T019 remain unchecked; Phase 2 foundation gate remains BLOCKED. The original C# animation math and `master`/`directx_build` are unchanged.

## Host and live environment

- WindowsEnv is Windows 11 25H2, build `10.0.26200.9711`, native 1920×1080 interactive desktop with D3D11/WARP capture via CUA.
- Original Explorer PID `6864`, desktop `Program Manager` HWND `65788` (`0x100fc`); pre-existing Task Manager PID `4568`.
- A **read-only** Windows P/Invoke EnumWindows/EnumChildWindows/GetWindowRect/GetClientRect/GetWindowLongW/GetWindowRgn/DwmGetWindowAttribute audit was run in WindowsEnv without any `SetWindowLong`, `SetParent`, Explorer restart, registry edit, undocumented shell message or wallpaper change.

### Observed window topology and clipping signals

| Window | Parent | Visible | Rectangle | Extended/style observations |
| --- | --- | --- | --- | --- |
| `Progman`, HWND `65788` | top-level | yes | 0,0,1920×1080 | style `0x96000000`, exstyle `0x00200080` includes `WS_EX_NOREDIRECTIONBITMAP` |
| `SHELLDLL_DefView`, HWND `65794` | `Progman` | yes | 0,0,1920×1080 | style `0x56010000`, exstyle `0x00080000` includes **`WS_EX_LAYERED`** |
| `SysListView32`, HWND `65796` | `DefView` | yes | 0,0,1920×1080 | style `0x56000340`, exstyle `0x00000000` |
| `SysHeader32`, HWND `65800` | `SysListView32` | no | zero-size | unimportant to wallpaper surface |
| 10 top-level `WorkerW` windows | no parent | **no**, all 10 | mostly approximately 136×39 (one 0×0) | none is a visible full-size wallpaper host |

The inspected `Progman`, `DefView` and `SysListView32` returned `DWMWA_CLOAKED=0`. The `GetWindowRgn` copies did not return a defined custom region (`ERROR=0`); Microsoft documents that this return is ambiguous between *no region* and an API error and it must **not** be interpreted as per-pixel transparency or the effective occlusion region. `DWMWA_EXTENDED_FRAME_BOUNDS` on the child windows returned an error rather than useful geometry; `GetWindowRect` gave full-screen bounds. We did **not** read/modify the actual per-pixel alpha of the Explorer-owned DefView. The present observations indicate a full-rectangle DefView/icon surface and non-cloaked desktop, not a proven precise DWM clipping mechanism.

Existing prior experiments ([DefView A/B](directcomposition-defview-ab.md), [layered alpha](directcomposition-layered-windowsenv.md)) already showed the same WARP/DComp pixels are not visible when an **owned Progman child** is below DefView (including `WS_EX_LAYERED` alpha=255), and are visible **above** DefView, obscuring icons. The new experiment therefore changes the **parenting model**: an unowned `WS_POPUP` test window, not another child.

## Dedicated owned popup experiment

- Diagnostic source commits on `rust_build`:
  - `95b7f48bf7a4f4fc9969158ea3acbeade2be4fa3`: add explicit `--top-level-below-progman` / `--top-level-above-progman` modes; same BGRA WARP, flip-sequential DXGI and DirectComposition magenta clear, max 10 s, only own HWND is created/positioned/destroyed.
  - `0cb262be985c31cad23963a63334f2b5ded95180`: on Z-order refusal, report parent, visibility, adjacent HWNDs and requested/actual order.
  - `3a553d1dce26ed296e64ad15a80da26278306ff1`: attempt documented **`HWND_BOTTOM`** for the lower placement and verify relative top-level order, without modifying `Progman`.
- Windows MSVC GitHub Actions runs: [41724846195](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41724846195), [41725720241](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41725720241), [41726306055](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41726306055): all **SUCCESS** for their exact commits, including CI tests; GitHub runners are **not** WindowsEnv.
- Native `probe_desktop_dcomp.exe` downloaded from matching Windows CI artifact for **each commit**, and its SHA-256 matched independently on Linux and WindowsEnv before running:

| Commit | Native executable SHA-256 |
| --- | --- |
| `95b7f48` | `9fcd88a04668bbe7f46418a201b225aa044c20a51f6f1db54b07abf47124500e` |
| `0cb262b` | `b0c3a08fa352325bae8278d1dc5a9706240a5d6d8581fe7779491c3112ff9908` |
| `3a553d1` | `00b93e755a904df761e334a9690f26f84e3b9093262d94373bdc7c06916cd336` |

- Both modes require explicit `--attach-modern --duration-ms 1..10000`. Top-level tests refuse `--above-icon-view` / `--layered-alpha` mixing, invalid durations and unsafe defaults. All 4 deterministic Rust CLI tests passed and Windows-target `cargo clippy -D warnings` and `cargo check` passed. `--inspect` remained read-only and found `raised=true same_icon_owner=true matches_Explorer=true attach_eligible=true`.

### WindowsEnv results

| Test | Native time | API/result | Visual evidence | Interpretation |
| --- | --- | --- | --- | --- |
| `--top-level-above-progman` | 05:49:59.194–05:50:06.744 | 456 successful WARP `Present`; HRESULT `0x00000000`; `owned_window_destroyed=true` | three during-run 2640/2640 sampled pixels saturated magenta; independent CUA full-resolution frames at 05:50:00.400 and 05:50:02.960 both had **41800/41800** magenta pixels | visible **over desktop icons**, diagnostic positive control only |
| `--top-level-below-progman` anchored after `Progman` | 05:49:48.898–05:49:55.014 | `API_FAILED owned_window_parent_zorder_or_visibility_changed`; `frames=0`; `owned_window_destroyed=true` | 0/2640 changed in 2,4,6s samples | **NOT** a valid below-host rendering test: unexpected Z-order stopped before any Present |
| above-anchored popup instrumented `Progman`-after request | 05:55:28.269–05:55:28.302 | `EXE_EXIT=1`; `parent_ok=true order_ok=false visible=true actual_parent=None`; `own_next=HWND(0x100fc)` (Progman), `anchor_next=None`; `frames=0`, window destroyed | no new screenshot-based success claim | Windows placed owned popup **above** Progman despite the requested lower insertion |
| `HWND_BOTTOM` for same lower placement | native WindowsEnv test after `3a553d1` deployment | `BELOW_EXIT=1`; `parent_ok=true order_ok=false visible=true actual_parent=None`; again `own_next=HWND(0x100fc)`, `anchor_next=None`; `frames=0`, window destroyed | **no rendering attempted** because placement refused | even the documented global-bottom anchor did not achieve a lower-than-Progman position here |

**Important distinction:** The two below-host attempts failed the **placement precondition**, not a D3D11 `Present` call or a visual pixel acceptance. No claim is made that a true, successfully positioned top-level *below Progman* surface was rendered invisibly; the OS did not grant that placement in this test. The Explorer-owned `Progman` HWND itself was never positioned.

### Cleanup and private evidence

- All test probes returned either successful cleanup or explicit `owned_window_destroyed=true`, including both placement refusals. The positive popup was alive only during its bounded 7500 ms run; screenshot after expiry returned to the previous pixels.
- At completion `mcp__win_cua__list_windows` contained **no** diagnostic, PowerShell or CMD windows. Explorer `Program Manager` HWND `65788`, PID `6864`, and pre-existing Task Manager PID `4568` still existed. The unrelated Notepad file tabs were preserved and its window restored to the initial **minimized** state.
- A fresh 1920×1080 screenshot after closing all probe controllers was **bit-for-bit equal** to the preflight screenshot in: (1) x=12..231/y=12..201 test ROI, (2) x=0..466/y=0..1039 icon strip and (3) x=467..1919/y=0..354 unobstructed top-right screen. Dynamic regions elsewhere are not claimed equal.
- Private full-resolution PNGs, command-window evidence and screenshot hashes are stored on Hermes only in the Git-ignored paths:
  - `rust/target/private-evidence/windowsenv-25h2/host-clip-082521e/`
  - `rust/target/private-evidence/windowsenv-25h2/host-95b7f48/`
  - `rust/target/private-evidence/windowsenv-25h2/host-3a553d1/`

Selected SHA-256 of private evidence:

| Image | SHA-256 |
| --- | --- |
| Topology audit console | `75772ee2d43286354792e8c1a0af3a9a07a091f056472ee7bdd9a693dab03412` |
| Preflight full-screen | `d84db118b0e4d97743fdd4701ebf8592e6f9e0c20e7ef7198a70b8f9916358b6` |
| Popup positive-control CUA frame 1 | `4963bd4a0c2a8e93e2a477f357ec8f15a4f967d8798a5fd1ef86a4e3b004ba8e` |
| Popup positive-control CUA frame 2 | `06457ca3a3cd171d541eae69f38f00ea7fe2af70c0862862b91eae8a6cfa6bc0` |
| Anchored placement rejection console | `f10f757820fa2b93b3c979dd6a2795a64cbff3f4263459e1fc3f06be1d0e1dda` |
| `HWND_BOTTOM` rejection console | `4abf6f7605ccfde12924516c26b5d0bdf968018c3850a1674981a4d99162bda5` |
| Final cleaned full-screen | `fe53d416c9a38e9e37f8acbcf669b22c8924bfa3269cf2a570cf3c6cac3099c6` |

## Engineering verdict and limits

1. **Validated positive**: WARP/DXGI/DirectComposition works for an owned top-level popup **above Progman** on WindowsEnv, but this obscures icons and cannot serve as the user's wallpaper.
2. **Refuted in this environment**: merely changing owned-window parenting to an independent top-level popup and requesting a position below Progman via either tested official Z-order form does **not** yield a behind-icon host. Validating the actual HWND Z-order is mandatory because successful `SetWindowPos` alone is not evidence of that ordering.
3. **No viable pre-existing WorkerW** was found under current Explorer conditions: all enumerated WorkerW windows were hidden and small. We did not create/reveal WorkerW with undocumented `0x052C` or mutate foreign Explorer HWNDs.
4. **Unresolved**: exact per-pixel DefView alpha and whether occlusion occurs in the DefView layered surface, the SysListView content, or a DWM/desktop clipping rule. The read-only geometry / cloak / region results cannot distinguish these internals. Do **not** claim Windows 25H2 universally forbids dynamic wallpapers.
5. **Recommended safe next investigation**: read-only ETW/Windows Graphics compositor tracing focused on the participating shell HWNDs, or a distinct shell-supported integration interface if Microsoft documents one. Neither grants permission to change Explorer-owned attributes, create/reveal WorkerW, or override existing desktop wallpaper settings. Do not ship an above-icon-view overlay as a substitute.

Official API references:
- [DirectComposition target and inherited window clipping](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nf-dcomp-idcompositiondevice-createtargetforhwnd)
- [Window visible region and WS_CLIPCHILDREN/WS_CLIPSIBLINGS](https://learn.microsoft.com/en-us/windows/win32/gdi/window-regions)
- [GetWindowRgn result caveat](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowrgn)
- [DwmGetWindowAttribute / DWMWA_CLOAKED](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmgetwindowattribute)
- [SetWindowPos / HWND_BOTTOM](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos)
