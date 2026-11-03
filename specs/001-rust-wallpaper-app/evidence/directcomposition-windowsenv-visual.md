# WindowsEnv 25H2 DirectComposition desktop visibility — 2026-11-04

**Verdict: API PASS / behind-icon visual acceptance FAIL / probe cleanup PASS.** This is a diagnostic investigation only, **not** a production wallpaper acceptance or a general claim that DirectComposition cannot work on Windows 11. Keep T009 and T019 **unchecked** and the Phase 2 foundation gate **BLOCKED** until a truly visible, behind-icon D3D11 output is demonstrated.

## Build identity and environment

- Git branch and revision: `rust_build` at `0fb88d6d75f99df5f604829431055493f5f83654`.
- Windows GitHub-hosted CI: [run 41690158919](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41690158919), **success** for Windows x64 release build, Clippy, tests and **read-only** DComp inspection. The CI runner is **not** WindowsEnv.
- Downloaded release diagnostic from GitHub Actions artifact `yuauy-foundation-probes-windows-x64` (artifact ID `13255598066`) to the authorized WindowsEnv, without installing a toolchain.
- Exact `probe_desktop_dcomp.exe` SHA-256, independently matched on Linux and WindowsEnv: `8e6a5273ce582a8a6e9d90f4fc4152dfe943f3287394c5995b83ebe9090b9438`.
- WindowsEnv CUA health reported Windows x64 **10.0.26200.9711** (Windows 11 25H2), UI Automation and D3D11 screen-capture capability PASS. This is distinct from the simplified `Environment.OSVersion` value `10.0.26200.0`.
- 1920 × 1080 primary desktop, with the original `Program Manager` HWND **65788** / Explorer PID **6864**. Task Manager PID **4568** remained open before/after.
- `--inspect` before and after the controlled run returned `raised=true same_icon_owner=true matches_Explorer=true attach_eligible=true composition=NOT_RUN`. This inspection does not alter the shell.

## Native WARP / DirectComposition run

The diagnostic is opt-in and bounded: `probe_desktop_dcomp.exe --attach-modern --duration-ms 10000`. It creates only its own temporary 220 × 190 Progman child positioned underneath `SHELLDLL_DefView`, uses a WARP D3D11 device, `CreateSwapChainForComposition` with flip-sequential buffers, a DComposition target/visual, and a conspicuous opaque magenta clear. It does not change Explorer-owned styles, parents, relative Explorer window order, registry or the user's real wallpaper settings.

During the final unobstructed run, the test controller was minimized **before** starting the separate child process, excluding its own console from the screen region. WindowsEnv recorded:

```text
PID=12820
START=2026-11-04T02:15:24.4529863+08:00
EXIT=0
END=2026-11-04T02:15:34.5226928+08:00
API_SUBMISSIONS_ONLY path=dcomp_composition_flip_sequential driver=warp feature_level=0xb000 present_s_ok=606 busy=0 last_present_hresult=0x00000000 duration_ms=10000 child_destroyed=true visual_wallpaper=UNVERIFIED requires_live_screenshot=true
TEST_PROCESS_RUNNING=False
```

The program reports successful native submissions and destruction of its owned HWND, but intentionally **does not** claim visibility from `Present` alone.

## Independent CUA screen comparison

The CUA driver captured native-resolution 1920 × 1080 screenshots, with its own cursor overlay excluded. The expected test rectangle is **x=12..231, y=12..201** (220 × 190 pixels). We compared each RGB pixel with the clean pre-run baseline, counting changes where any channel differs by more than 30, and counted pixels matching the expected saturated magenta marker (`R>170, G<70, B>100`).

| Image and local capture time | Within active 02:15:24.452–02:15:34.522 run? | Changed pixels in test rectangle | Magenta pixels |
| --- | --- | ---: | ---: |
| Clean baseline 02:15:00.740 | Before | — | 0 |
| Frame 01 02:15:29.095 | **Yes** | **0** | **0** |
| Frame 02 02:15:32.151 | **Yes** | **0** | **0** |
| Frame 03 02:15:34.251 | **Yes** | **0** | **0** |
| Frame 04 02:15:37.799 | After | **0** | **0** |

The unoccluded region was **bit-for-bit identical** to the baseline in all three during-run frames and the first post-run frame. Thus the intended DirectComposition color never appeared **visibly** behind the icons in this WindowsEnv test despite 606 successful API submissions.

Selected private screenshot SHA-256 metadata, not screenshots committed to Git:

| Capture | SHA-256 |
| --- | --- |
| Clean baseline | `2c8463679687fe7ca6871c50cacd1757bcd5757c3b35e668ef44e15ea6857161` |
| Active frame 01 | `29f50ca6f4b9e44e765af5e5d7d95328799c12ef4b53f763c1fb43a9a71574d8` |
| Active frame 02 | `7cffb2b8beb5c70e088ebb994860e689282c0baa2fd1c862bfafcf702aad8a11` |
| Active frame 03 | `82426ebd7ec058cff402e3f509ffe82a7c134ac61520f847f0e435125a9b00f1` |
| After window expiry | `2dc2b7797804f9f3d7cf95f9daae724eaa88bda323178e77836c0828f4dcd9e3` |
| After controller cleanup | `25dc13e6f1ac31680dbcf9662044c8eed4bd6f34c8dc1ac53cdc3e75ffa0cd87` |

Images remain **private and Git-ignored** under `rust/target/private-evidence/windowsenv-25h2/verification-0fb88d6/`. Earlier screenshots with a PowerShell console overlapping the test rectangle were discarded **as visibility evidence**; only the final unobstructed timed run supports the verdict.

## Desktop restoration / no permanent changes observed

- `EXIT=0`, `child_destroyed=true`; a separate Windows process read-back reported `TEST_PROCESS_RUNNING=False`.
- A new `--inspect` after child exit had the same raised Progman / Explorer icon-view eligibility flags as before.
- All temporary PowerShell and CMD **controller** windows created for this check were subsequently closed. Fresh CUA `list_windows` reported **no remaining PowerShell/CMD windows**, with the original Explorer `Program Manager` HWND 65788, Explorer PID 6864 and Task Manager PID 4568 still present.
- A fresh screenshot after all controller cleanup matched the original screenshot **exactly** in the 220 × 190 test rectangle and in the full **x=0..466, y=0..1039** left desktop/icon strip. The unobstructed upper-right region `x=467..1919, y=0..354` also matched exactly. Dynamic application regions elsewhere were not assumed pixel-identical.
- No application-owned test window/process remained, and no Explorer restart, reparenting, window restyling, autostart registry write, display-mode change or persistent background modification was requested.

**Scoped cleanup result: PASS.** This supports no observed persistent desktop damage in the checked regions and live window topology. It does not establish every possible Explorer property in the system is globally unchanged.

## Gate consequence and next investigation

- **PASS**: artifact identity; real WindowsEnv WARP DirectComposition API submission; controlled duration; native HWND destruction and command/process/controller cleanup.
- **FAIL**: actual visual desktop layer **behind icons**, currently zero changed pixels in the expected region.
- **NOT TESTED / NOT CLAIMED**: real GPU performance, physical multi-monitor visibility, high refresh modes or the final application rendering pipeline.
- **T009 and T019 remain open**; do not bypass the WindowsEnv visual foundation gate or modify preserved C# math.

The next diagnostic should isolate why a DComposition visual on this raised `Progman` does not appear: compare owned test surfaces using explicit above/below icon-view visibility controls and an appropriate DWM/DirectComposition host strategy, while keeping all Explorer-owned HWNDs and user settings untouched. Record live screenshots **during** each bounded run before considering any product implementation.
