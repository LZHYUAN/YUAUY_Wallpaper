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
7. **Windows D3D path** — Create a hardware device if available; otherwise verify an explicit WARP device (log driver type). Verify a wallpaper window behind desktop icons, not an overlay; minimize/restart Explorer, plug/unplug monitors and quit cleanly.
8. **YAML and tray** — Valid YAML loads; invalid/partial YAML keeps prior valid settings and scene; tray menu opens config and reloads without restarting; no repeated warning balloons.
9. **Physical-GPU follow-up** — Separate machine with a real GPU and preferably two unequal monitors: verify GPU total utilization reading and driver path, latency, multiple-adapter behavior if present, animation fidelity against C# reference and absence of interference with foreground apps. Mark untested fields explicitly.

## Evidence format

For every run record: Windows build; CPU/GPU name (omit serial IDs); D3D11 driver type (Hardware/WARP); monitor topology; test dataset counts; Rust toolchain commit; `cargo test` summary; date; PASS/FAIL/SKIPPED; known issues and a redacted screenshot/log if relevant. Never conflate WARP CPU fallback and real GPU utilization validation.

## Exit criteria

- Automated deterministic logic and error-handling tests pass.
- WindowsEnv D3D11/WARP, tray and empty-state behavior visibly verified.
- WorkerW handling and Explorer recreation proven on target Windows builds.
- Physical GPU and multi-monitor parity independently verified, or release explicitly blocked/pending these gates.
- Owner signs off provisional YAML thresholds, initial recovery UX and packaging approach before final release.
