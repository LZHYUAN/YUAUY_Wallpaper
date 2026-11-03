# WindowsEnv 25H2 Validation Quickstart (planned)

This guide is an acceptance runbook, **not a record of executed tests**. No Rust executable has been built yet. See [plan.md](plan.md), [spec.md](spec.md), [YAML example](contracts/config.example.yaml).

## Prerequisites

- WindowsEnv 25H2 available via the connected remote Windows tools; a Windows 11 test desktop with Explorer and Win32 UI.
- Rust stable MSVC target, MSVC build tools/Windows SDK, a configured WARP-compatible graphics stack, PowerShell and a path for test photos. Check toolchain availability before issuing build commands.
- The new `rust/` project must be created during implementation, following [plan.md](plan.md). Do not try to execute `cargo` against nonexistent source files yet.
- Prepare synthetic public-domain or owned sample images for all tests. Do not commit personal photos, private folders or runtime YAML containing personal paths.

## Expected build commands (once source exists)

```powershell
cd <repo>\rust
rustup target add x86_64-pc-windows-msvc
cargo test --target x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

If WindowsEnv lacks the Rust/MSVC toolchain or its machine is not suitable for full builds, compile on a separate authorized Windows builder and deploy the resulting exe to WindowsEnv, then record where it was built. Never claim a build passed without a real command log.

## Repeatable acceptance scenarios

1. **Pure Rust parity** — Use golden animation vectors obtained from the latest common C# algorithm. Test original `cosh` easing, phase boundaries (0.25, 0.50, 0.75), translation, angle, scalar placement, coverage and monitor-area-weighted selection with deterministic input. No software renderer needed.
2. **Global image queue** — 3 images -> no repeated path before 3 claims; add a fourth mid-cycle -> do not pick it until next cycle; cycle rollover -> rescan and reshuffle. A single-image folder should supply 11 startup claims across cycles without a busy loop.
3. **Empty startup** — Empty directory -> tray says `沒有圖片`, no photo decode, no opening animation, no GPU/CPU reading. Add an image before normal timer deadline -> no scan until deadline; on deadline -> one scan -> status clears and imagery resumes. Three empty deadlines -> exactly three scans; tray manual reload -> one immediate scan.
4. **Single monitor startup** — One main + ten background images with varying start delays that overlap, all backgrounds behind the main; finish as a static screen with zero frame render calls between transitions.
5. **Two monitor startup (simulated then physical)** — Each monitor independently receives one main + ten background images; whole system shares a single randomized non-repeating queue. Check negative coordinates and DPI scaling. When only one photo exists, repeated appearances must occur across cycles, not before exhaustion.
6. **Idle load sampling** — Run a virtual time source; no due event -> zero load reads. Due tick on high system load -> exactly one bounded measuring operation, zero image claims and zero new frame render; next attempt at full YAML interval. Opening scene ignores even a mock 100% reading.
7. **Windows D3D path and VSync** — Create a hardware device if available; otherwise verify explicit WARP (log driver type). During an animation require DXGI VSync (`Present(1,0)` when compatible). On 60/120/144Hz monitors record actual present intervals and animation duration; a faster monitor must not shorten the original time-mapped animation. Test mixed-rate monitors if available. After completion Present count must stop increasing in idle. Verify WorkerW behind desktop icons, restart Explorer, hot-plug monitors and quit cleanly; mark refresh-rate results not available on WARP as SKIPPED, never PASS.
8. **YAML and tray** — Confirm default `slideshow_interval_seconds: 30` and single `load_guard.threshold_percent: 60`; edit both in YAML and reload, confirming effective scheduling changes. Valid YAML loads; invalid/partial YAML keeps prior valid settings and scene; no repeated warning balloons. The old `pause_at_percent` / `resume_at_percent` keys must be rejected rather than ignored.
9. **GPU/CPU threshold boundaries** — Inject usage 59.9%, 60%, 60.1% on scheduled due events: allow, skip, skip; next due with 59.9% resumes immediately. Zero utilization reads between deadlines and in opening composition. No lengthy calibration. WindowsEnv uses `cpu_total` test mode and the same 60% single threshold.
10. **Immediate Windows autostart** — In an isolated WindowsEnv user profile, click the tray checkbox ON and verify the application-owned current-user `Run` value is written and readable immediately; click OFF and verify it disappears immediately. Simulate registry access failure or mismatched read-back and confirm error plus correct checkmark. Reopen menu to reload actual OS status. Exercise quoted exe paths with spaces, test login behavior, and clean up only the test entry afterward. Do not alter another user's or real machine's startup registry.
11. **Physical-GPU follow-up** — Separate machine with a real GPU and preferably two unequal monitors: verify GPU total utilization reading and driver path, latency, multiple-adapter behavior if present, animation fidelity against C# reference and absence of interference with foreground apps. Mark untested fields explicitly.

## Evidence format

For every run record: Windows build; CPU/GPU name (omit serial IDs); D3D11 driver type (Hardware/WARP); monitor topology; test dataset counts; Rust toolchain commit; `cargo test` summary; date; PASS/FAIL/SKIPPED; known issues and a redacted screenshot/log if relevant. Never conflate WARP CPU fallback and real GPU utilization validation.

## Exit criteria

- Automated deterministic logic and error-handling tests pass.
- WindowsEnv D3D11/WARP, tray and empty-state behavior visibly verified.
- WorkerW handling and Explorer recreation proven on target Windows builds.
- Physical GPU and multi-monitor parity independently verified, or release explicitly blocked/pending these gates.
- Owner-approved 30s/60% single-threshold defaults and VSync target are verified; hardware-specific VSync, start-on-sign-in implementation, initial recovery UX and packaging approach remain subject to actual acceptance before final release.
