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
- Windows GitHub Actions hosted-runner Present(1) diagnostic: **PENDING** after commit; will be recorded separately from WindowsEnv.
- WindowsEnv WARP/output refresh timing: **SKIPPED**, no trusted non-disruptive terminal path currently available.
- Hardware 60/120/144/240 Hz and cap 60/120/240 full matrix: **SKIPPED** where no actual display is accessible. Do not invent measurements.

## Required follow-up for T010 acceptance

Run the diagnostic on WindowsEnv, record stdout/exit code, D3D11 driver and measured frame submission intervals. Confirm effective VSync using an **actual visible test surface** and measured display mode rather than interpreting hidden swap-chain returns. Repeat the available refresh/cap combinations on physical 60, 120, 144, 240 Hz displays. Explicitly record unsupported modes. Compare normal-transition playback duration across refresh rates using the fixed C# formula reference.
