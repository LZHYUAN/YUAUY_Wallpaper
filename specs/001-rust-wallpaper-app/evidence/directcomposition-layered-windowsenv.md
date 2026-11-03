# WindowsEnv 25H2 — layered child HWND / alpha / DirectComposition experiment

**Date:** 2026-11-04 (Asia/Taipei UTC+08).
**Verdict:** D3D11 WARP DirectComposition + `WS_EX_LAYERED` **works visibly above DefView** with verified global alpha 255/128, but **fails to appear below DefView** even at alpha 255. All six bounded probes submit successfully and destroy their owned HWNDs; the desktop and icon area return to the pre-run pixels. **Do not accept T009/T019 or the Phase 2 gate.**

## Identity and constraints

- Source `rust_build` commit `8bbe582189804c0545fb0ea9acde459f08f3f2dc`.
- GitHub Windows MSVC CI run: [41709833481](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41709833481) — **success**, includes release executable and deterministic tests. Native WindowsEnv is a **different** machine, not the GitHub runner.
- Downloaded CI artifact `yuauy-foundation-probes-windows-x64` ID `13266458826`. `probe_desktop_dcomp.exe` SHA-256 independently confirmed on Hermes Linux and WindowsEnv: `299b2c51aabad637c3e1e5cabde27a6f6ad531f145bb4155f1a0c8cdeb1b364c`.
- WindowsEnv 25H2 build `10.0.26200.9711`, primary 1920×1080. Before/after: Explorer PID `6864`, `Program Manager` HWND `65788`; original Task Manager PID `4568`.
- Existing embedded `rust/assets/windows-app.manifest` declares Windows 8, 8.1, 10/11 awareness. Microsoft documentation requires such awareness for layered child HWNDs.
- Test executable creates **only its own** 220×190 `Progman` child (`x=12..231, y=12..201`), keeps `SHELLDLL_DefView` and Explorer-owned HWNDs unmodified, retains `WS_EX_TRANSPARENT | WS_EX_NOACTIVATE`, and uses D3D11 **WARP** / BGRA / flip-sequential DXGI composition / DirectComposition visual.
- The unlayered control has `WS_EX_NOREDIRECTIONBITMAP`. The layered diagnostic replaces that style with `WS_EX_LAYERED`, calls `SetLayeredWindowAttributes` on the **owned** child, reads back via `GetLayeredWindowAttributes`, and keeps the same WARP visual. Thus layered-vs-unlayered differs in **host style and alpha initialization**, but the alpha 255/128/0 comparisons themselves preserve the layered host style. The above-vs-below comparisons preserve the alpha/style and modify only owned sibling placement.
- The opt-in CLI enforces `--attach-modern --duration-ms 1..10000 [--layered-alpha 0|128|255] [--above-icon-view]`; default remains no-side-effects `--inspect`. All six test cases used 7500 ms. No change to Explorer styles, parenting, registry, wallpaper configuration or original C# animation.

## Native logs from the WindowsEnv interactive desktop

Native launcher was minimized before sampling. Each case has baseline capture, images at approximately 2/4/6 seconds during the process, and an after image. The process reports `sibling_order_verified=true` and `child_destroyed=true`, with `busy=0`, `last_present_hresult=0x00000000`.

| Case | START (UTC+08) | END (UTC+08) | DComp S_OK Present | Verified layered attribute | Visual result |
| --- | --- | --- | ---: | --- | --- |
| `nonlayered_below` | 04:12:02.763 | 04:12:10.332 | 455 | none | no change |
| `layered255_below` | 04:12:12.514 | 04:12:20.080 | 454 | alpha=255, flags=0x2 | no change |
| `layered128_below` | 04:12:22.237 | 04:12:29.794 | 455 | alpha=128, flags=0x2 | no change |
| `layered0_below` | 04:12:31.916 | 04:12:39.470 | 455 | alpha=0, flags=0x2 | no change (expected fully transparent) |
| `layered255_above` | 04:12:41.610 | 04:12:49.169 | 455 | alpha=255, flags=0x2 | opaque magenta, covers icons |
| `layered128_above` | 04:12:51.296 | 04:12:58.852 | 455 | alpha=128, flags=0x2 | half-opacity magenta, icons show through tint |

**Important logging limitation:** The PowerShell `Start-Process` object's `ExitCode` printed as a blank string in this run despite `WaitForExit` finishing, so this document does **not** claim a verified numeric process exit code. It **does** have the native program's `API_SUBMISSIONS_ONLY` success output and `child_destroyed=true` for each of the six runs, and a later explicit `PROBE_RUNNING=False` process check.

All six read-only before/after topology inspections returned:
`INSPECT_ONLY raised=true same_icon_owner=true matches_Explorer=true attach_eligible=true composition=NOT_RUN visual_wallpaper=UNVERIFIED`.

## Deterministic interactive screenshot measurements

The controller uses `System.Drawing.Graphics.CopyFromScreen(12,12,220,190)` at 2, 4 and 6 seconds **while** the 7500-ms child is alive. It samples a regular 4-pixel grid (2640 RGB samples), counts a change where any channel delta from the per-case baseline exceeds 30, and counts magenta when `R>170,G<70,B>100`.

| Case | Changed at t2/t4/t6 (each / 2640) | Magenta at t2/t4/t6 (each / 2640) | Mean RGB of t2/t4/t6 |
| --- | --- | --- | --- |
| unlayered below | 0 / 0 / 0 | 0 / 0 / 0 | 27.19 / 99.17 / 109.48 |
| layered alpha 255 below | 0 / 0 / 0 | 0 / 0 / 0 | 27.19 / 99.17 / 109.48 |
| layered alpha 128 below | 0 / 0 / 0 | 0 / 0 / 0 | 27.19 / 99.17 / 109.48 |
| layered alpha 0 below | 0 / 0 / 0 | 0 / 0 / 0 | 27.19 / 99.17 / 109.48 |
| layered alpha 255 above | 2640 / 2640 / 2640 | 2640 / 2640 / 2640 | 242 / 8 / 191 |
| layered alpha 128 above | 2640 / 2640 / 2640 | 9 / 9 / 9 | 134.43 / 53.35 / 150.48 |

The 9/2640 magenta count in half-opacity mode is **not a failure**: blended magenta is no longer saturated enough to satisfy the *opaque-magenta detection threshold* at most pixels. All 2640 samples have changed, and average RGB moves approximately halfway from original desktop to opaque magenta.

Each case's **after** screenshot reverted to **0 changed samples and 0 magenta** in the test rectangle.

Separately, the native Windows CUA driver captured full-desktop **1920×1080** PNGs, independent of the controller. During alpha 255 **ABOVE** at 04:12:46.900 and 04:12:48.968, all **41,800/41,800 pixels** in the expected 220×190 region matched saturated magenta. During alpha 128 **ABOVE** at 04:12:53.620, 04:12:55.696 and 04:12:57.724, **41,800/41,800** pixels changed, with only **87** exceeding the strict saturated-magenta threshold due to blending. Earlier frames during below-icon cases had **0 changed pixels** in the target rectangle.

Independent CUA screenshots visually confirmed that alpha 255 **obscures** Explorer icons within the rectangle and alpha 128 **tints** those icons — both are overlays, **not** acceptable behind-icon wallpaper.

Selected SHA-256 (private, never committed as screenshots):

| File | SHA-256 |
| --- | --- |
| CUA full-screen baseline | `97f22575312671eaf140b7f55b69550e1b023649f011776a11b2a21bb48d46f8` |
| Opaque above frame 04:12:46 | `844a7784378ed500484eb5b1afd9d9bf958a885ff9e9ee22b1308a1bd940b76f` |
| Alpha-128 above frame 04:12:53 | `30a9dc15b77b8ee8914959a520e39acc6b62e114507009447a3a40215747507b` |
| After all cleanup | `56f8559aaeeef698790f14c7b6fdb1718841d954f673bab57a70a7be7d28aa14` |

Local private 1920×1080 screenshot copies are Git-ignored under `rust/target/private-evidence/windowsenv-25h2/layered-8bbe582/`. Windows controller ROI PNGs and full `layered-matrix.txt` are under the test user's `%TEMP%\YUAUY-Win25-Probes\layered-8bbe582`. Screenshots were **not** uploaded to GitHub.

## Cleanup and limitations

- All six owned child HWNDs reported destroyed. The final controller log recorded `PROBE_RUNNING=False`. Temporary PowerShell controller processes were closed.
- We opened the test-only `layered-matrix.txt` in Notepad to inspect the native log via UI Automation, then closed **only that newly opened tab**, keeping existing unrelated/unsaved Notepad tabs. The Notepad window was minimized after inspection.
- Explorer PID 6864 / `Program Manager` HWND 65788 and original Task Manager PID 4568 remained unchanged. A clean native CUA screenshot after controller and log inspection matched the original baseline **bit-for-bit** in (a) test rectangle 220×190, (b) left icon strip x=0..466 / y=0..1039 and (c) unobstructed top-right x=467..1919 / y=0..354.
- The result verifies **no observed persistent impact** on these regions and the tracked shell topology; this is not a complete proof that every uninspected DWM internal is unchanged.
- The comparison does **not** prove the precise DefView alpha channel value or distinguish a DefView opaque background from DWM's underlay clipping. A read-only DefView P/Invoke attempt from prior investigation was inconclusive and must not be reused as evidence.

## Engineering decision

**FAIL behind-icon requirement.** The native WARP/D3D11/DXGI/DComposition draw path and the layered opacity controls function on this WindowsEnv, but setting `WS_EX_LAYERED` and alpha to 255 or 128 on the **owned underlay child** did not expose any pixels behind the icons. Alpha=0 is deliberately transparent and offers no positive evidence. Moving the owned child **above DefView** makes both opaque and semi-transparent visuals visible but interferes with icons, so it must **never** be repurposed as a shipping wallpaper workaround.

Keep **T009, T019, and Phase 2 foundation gate blocked**. Next safe test should isolate the **DefView parent-region clipping versus desktop background/alpha** with more read-only geometry and compositor instrumentation or a separate alternate host that does not mutate foreign Explorer HWNDs. Do not modify the C# reference math, `master`, or `directx_build`.

Official references:
- [Microsoft: Layered Windows, child support since Windows 8](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features)
- [Microsoft: SetLayeredWindowAttributes](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setlayeredwindowattributes)
- [Microsoft: DirectComposition CreateTargetForHwnd and window clipping](https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nf-dcomp-idcompositiondevice-createtargetforhwnd)
- [Microsoft: Windows 8-aware manifest for layered children](https://learn.microsoft.com/en-us/windows/win32/winmsg/using-windows)
