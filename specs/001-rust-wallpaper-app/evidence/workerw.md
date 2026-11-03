# T009 — WorkerW desktop-host feasibility probe

**Status: PARTIAL — legacy and raised-desktop probe strategies compile and their CLI safety policies are tested. WindowsEnv behind-icon visual test remains UNVERIFIED. Do not check T009 yet.**

## Implemented behavior

- `rust/src/bin/probe_workerw.rs` defaults to `--inspect`: read-only enumeration of `Progman`, direct child `WorkerW`, direct child `SHELLDLL_DefView`, owner PID consistency, rectangle and visibility, and `Progman`'s `WS_EX_NOREDIRECTIONBITMAP` hint.
- If prerequisites are missing, exit code **2** with `SKIPPED` (not a false PASS); a Win32 API failure exits **1**.
- `--attach` requests a **legacy** 110×40 disabled, non-activating child under an **already-existing WorkerW** and refuses to run on a raised desktop. `--attach-modern` requests an explicit **raised-desktop** layered, non-activating child directly under Progman, positioned immediately below `SHELLDLL_DefView` by moving **only the new test HWND**. Both are bounded (default 3 seconds; maximum 10 seconds via `--duration-ms`) and RAII-destroyed with post-destruction `IsWindow` read-back. No automatic fallback can silently change attachment strategy.
- There is no undocumented `Progman` `0x052C` call; the probe never changes **Explorer-owned** window styles or their mutual z-order, registry, icons, wallpapers or C# code. Only an explicitly requested application-owned test HWND may be inserted behind the icon view; it is destroyed on exit. No WorkerW is spawned when absent.
- The probe **does not claim** a valid parent relationship proves the window is actually composed behind icons; that requires an actual WindowsEnv screenshot and visual inspection while the child exists.

## Validation (2026-11-03)

| Test | Status | Evidence |
| --- | --- | --- |
| CLI safe default / explicit bounded attach / reject unsafe options | **PASS** | Initial TDD: 3/3; additional raised-desktop strategy tests 2/2, Windows MSVC cross-clippy PASS; total 5 tests |
| Windows x64 MSVC cross-build type check / Clippy | **PASS** | `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings` |
| Linux unit/integration suite | **PASS (initial)** | Initial 7/7 passed; latest suite to be re-run for this update |
| Windows hosted CI release build | **PASS** | [CI run 41624317570](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41624317570), commit `8a45e4a`: MSVC release build and deterministic tests passed |
| Windows hosted CI read-only WorkerW inspection | **SKIPPED (expected)** | Executed binary, but returned `SKIPPED: WorkerW child of Progman absent; no shell message sent` (worker not present on hosted runner) |
| WindowsEnv 25H2 desktop layering | **SKIPPED** | No approved, non-focus-stealing native Windows CLI/SSH channel available yet; CUA UIA/graphics capture health is not a WorkerW application test |
| WindowsEnv temporary child cleanup / Explorer restart recovery | **SKIPPED** | Requires WindowsEnv interactive acceptance; never assume success from CI |

## Manual WindowsEnv acceptance

When a verified **non-disruptive** WindowsEnv terminal is available, build and run:

```powershell
cd <project>\rust
cargo run --locked --release --bin probe_workerw -- --inspect
cargo run --locked --release --bin probe_workerw -- --attach-modern --duration-ms 5000 # only when INSPECT progman_no_redirection=true
```

If `progman_no_redirection=false`, **do not use** `--attach-modern`; the explicit legacy `--attach` may be considered only after confirming its topology. While attaching, observe screenshot/window tree and prove this process-owned visible test square lies behind desktop icons. After the 5-second expiry verify it disappeared, no extra WorkerW is left, no Explorer HWND style/parent changed, and desktop icons still work. Record OS build, exact console output and screenshot metadata without personal desktop images. Repeat only in a test session; do **not** trigger a shell reset.

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

## Raised-desktop strategy refinement (2026-11-04)

The initial WorkerW-only probe was insufficient to test Windows 11 24H2+ layering: a direct child of WorkerW does not on its own prove visible composition beneath a potentially opaque desktop icon view. Public third-party reports describe a **raised desktop** using `WS_EX_NOREDIRECTIONBITMAP` on Progman, with `SHELLDLL_DefView` and `WorkerW` direct children, and place a fully-opaque *layered* wallpaper child immediately beneath the icon view. Sources include:

- [Lively community 24H2 compatibility investigation, issue #2074](https://github.com/lively-community/lively/issues/2074) — community reports and quoted Microsoft engineering guidance; not a public Microsoft Win32 API contract.
- [Lively v2.2 compatibility changelog](https://lively-community.github.io/lively/changelog/) — reports full 24H2 support after a shell integration redesign.
- [Ivy Windows Wallpaper host technique](https://github.com/Ivy-Interactive/Ivy-Windows-Wallpaper) — independently documents conditional Progman child layering for 24H2+.

**Implementation constraints:**

1. `--inspect` is still the default, performs **read-only** Win32 enumeration/parent and style checks, and never changes the desktop.
2. `select_target()` requires an explicit strategy: legacy `--attach` is refused when Progman has `WS_EX_NOREDIRECTIONBITMAP`, while `--attach-modern` is refused without it. No hidden downgrade or undocumented `0x052C` trigger.
3. Modern probe creates an owned layered, non-activating 110×40 disabled `STATIC` child under Progman. `SetLayeredWindowAttributes(alpha=255)` and `SetWindowPos(own_child, SHELLDLL_DefView, ...)` only manipulate the test HWND. It reads back whether that HWND lies immediately below the icon view; the original Explorer-owned windows are neither reparented nor restyled.
4. After at most 10 seconds it destroys the **probe-owned** child and verifies cleanup. GUI visual compatibility (including whether the icon view is transparent), WorkerW stability and *real DXGI swap-chain frame presentation* are still **NOT VERIFIED**. Even `zorder_below_defview=true` is insufficient for an overall PASS.
5. RED observed when adding TDD tests for missing `AttachTarget`, `select_target`, and `Mode::AttachModern`. GREEN: both new policy tests pass (total 5/5), Windows-target Clippy passed; native `--attach-modern` is **NOT RUN** on CI or WindowsEnv without explicit interactive approval.

T009 and Phase 2 T019 therefore remain unchecked. The new probe is an acceptance aid, not a new production desktop-host implementation or permission to change Explorer configuration.

## Windows CI verification of raised-desktop strategy (2026-11-04)

[GitHub Actions 41681681198](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41681681198), source commit `f3bf944b822292a948cd01610f5b1179c73e20f4`, completed **successfully**: Windows MSVC format, Clippy, release build and all **50 deterministic tests** passed, including the 5 WorkerW CLI/strategy safety tests. Windows-hosted `probe_workerw --inspect` returned:

```text
SKIPPED: WorkerW child of Progman absent; no shell message sent
```

The CI runner did not have an eligible desktop WorkerW. **Neither `--attach` nor `--attach-modern` was executed.** The modern path is confirmed to compile for Windows MSVC and its pure policy tests pass, but on-screen composition, icon occlusion and cleanup after *actual* attachment on WindowsEnv are still **UNVERIFIED**. T009 remains unchecked.
