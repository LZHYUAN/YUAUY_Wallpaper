# T010 — VSync and max_fps probe

**Status: PARTIAL.** The software pacing core and Windows diagnostic compile. WindowsEnv and physical monitor verification are missing, so **T010 stays unchecked**.

## Implemented

- `rust/src/pacing.rs` isolates an opt-in frame submission limiter from the preserved C# normal-animation formulas. Target FPS is `min(display_refresh_hz, max_fps)`. Minimum period is rounded **up** to integer nanoseconds; it never schedules faster than the selected cap. No background drawing loop exists, and missed slots are not queued or rapidly replayed.
- Animation progress is based only on monotonic elapsed time and the chosen playback duration; it never depends on the number of frames, Hz, or display count. The original C# easing is intentionally not reimplemented in this scheduler module.
- `rust/src/bin/probe_pacing.rs` enumerates currently desktop-attached Windows display modes and reports which requested 60/120/144/240 Hz scenarios are **AVAILABLE** or **UNSUPPORTED**. It builds a separate, owned, hidden 64×64 Direct3D11/DXGI swap chain, selects WARP or hardware explicitly, waits on the additional `max_fps` deadline, and issues `Present(syncinterval=1)` at most 120 times, logging each real submission and return timestamp. It does **not** change the active display mode or alter Explorer.
- **Caveat:** A successful `Present(1)` from a **hidden** window does NOT demonstrate actual VSync pacing against a visible physical display. Report such results as diagnostics only. Even if the API returns success with a nonzero refresh rate, on-screen frame presentation remains unverified.

## Deterministic test evidence (2026-11-04)

- TDD RED: `cargo test --locked --test frame_pacing --target x86_64-unknown-linux-gnu` failed with E0432 (module `pacing` missing).
- GREEN: 5/5 tests PASS for cap matrix, 59.94 Hz, bad inputs, no catch-up bursts, elapsed-time independence from frames and zero playback duration.
- `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings`: PASS.
- Windows GitHub Actions hosted-runner Present(1) diagnostic: **PASS (diagnostic API execution only)** — [workflow 41679449309](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41679449309), job 126068877101, commit 47be141.
- WindowsEnv WARP/output refresh timing: **SKIPPED**, no trusted non-disruptive terminal path currently available.
- Hardware 60/120/144/240 Hz and cap 60/120/240 full matrix: **SKIPPED** where no actual display is accessible. Do not invent measurements.

## Required follow-up for T010 acceptance

Run the diagnostic on WindowsEnv, record stdout/exit code, D3D11 driver and measured frame submission intervals. Confirm effective VSync using an **actual visible test surface** and measured display mode rather than interpreting hidden swap-chain returns. Repeat the available refresh/cap combinations on physical 60, 120, 144, 240 Hz displays. Explicitly record unsupported modes. Compare normal-transition playback duration across refresh rates using the fixed C# formula reference.

## Hosted Windows runner measurements (2026-11-04)

GitHub workflow [41679449309](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41679449309) completed successfully. The owned hidden 64×64 window requested **WARP, syncinterval=1**, and a software ceiling of **120 FPS**. Host display enumeration returned **display0=64 Hz**, so `min(64,120)=64` was the diagnostic submission target. All 60/120/144/240 Hz × 60/120/240 FPS acceptance combinations were printed **UNSUPPORTED** because those refresh modes were not present.

Exact captured log excerpts:

```text
AVAILABLE output=display0 refresh_hz=64 max_fps=120
BEGIN requested_driver=warp feature_level=0xb000 requested_vsync_interval=1 fps_target=64 window_visible=false
FRAME index=0 submit_ms=0.034 present_return_ms=30.833 hresult=0x00000000 request_syncinterval=1
FRAME index=1 submit_ms=46.847 present_return_ms=78.620 hresult=0x00000000 request_syncinterval=1
FRAME index=11 submit_ms=505.186 present_return_ms=537.049 hresult=0x00000000 request_syncinterval=1
VERDICT diagnostic_only=true true_vsync=UNVERIFIED desktop_surface=UNVERIFIED elapsed_ms=537.080
```

There were **12 real successful Present(1) API submissions**. The return timestamps span ~506 ms across 11 gaps, noticeably slower than the nominal 64 FPS software target; **do not** report this as a 64 FPS achieved frame rate or proof of physical VSync. The hidden-WARP runner is diagnostic-only, and WindowsEnv/real monitor acceptance remains **SKIPPED**. T010 remains unchecked.
