# Implementation Plan: YUAUY Wallpaper — Rust + Direct3D 11

**Branch**: `rust_build` | **Date**: 2026-11-03 | **Spec**: [spec.md](spec.md)

**Input**: `spec.md` (FR-001～FR-030), [Constitution v2.4.0](../../.specify/memory/constitution.md), [舊版行為稽核](reference-audit.md).

**Phase**: Spec Kit `$speckit-plan` / implementation design; **no Rust implementation or Windows integration tests have been run yet**. This document is an implementation proposal, not a release certificate.

## Summary

Create an independent Windows 11 desktop wallpaper executable in Rust using Direct3D 11. Preserve the latest common C# animation math, monitor-area-weighted slideshow selection, and selection/placement rules. Replace the legacy accelerated startup with the owner-approved per-monitor composition: one center image followed by ten staggered overlapping slides behind it. A shared shuffle-without-replacement image queue handles every monitor and the startup; it rescans only on cycle rollover, explicit YAML reload, or a scheduled empty-state retry. The tray controls settings and status plus an immediate current-user Windows sign-in autostart toggle. The normal slideshow defaults to 30 seconds, adjustable in YAML; a single configurable 60% total GPU load threshold is checked only with a quick bounded due-time measurement (CPU test equivalent in WindowsEnv), with no hysteresis. Active animations use VSync plus a YAML-adjustable `max_fps` cap (default 120) so their presentation rate targets `min(display_refresh_rate, max_fps)` while preserving original elapsed-time animation formulas and duration. First startup bypasses the load guard. On the GPU-less WindowsEnv, use a software rendering path and test the resource gate with CPU input.

## Technical Context

**Language/Version**: Rust stable, pinned via `rust-toolchain.toml` during implementation; build target `x86_64-pc-windows-msvc`.

**Primary Dependencies**: `windows` crate for Win32/COM/D3D11/DXGI/PDH integration; `serde` and maintained `yaml_serde` for YAML; `rand` for shuffled image order; Win32/WIC image decoding (confirm format and alpha behavior in Windows PoC). Exact versions and features are to be pinned and audited at implementation time. The historical C# and unfinished C++ branches are not Rust dependencies.

**Storage**: User-editable YAML at a per-user Windows configuration location; user-owned image folders, never uploaded or committed; transient image metadata and textures only. Autostart is stored in the OS current-user Run registration, **not YAML**, and its checked state is verified against Windows on each toggle. Application-local diagnostics with no image bytes or private folder paths in routine logs.

**Testing**: Rust unit/property/integration tests with deterministic RNG and injected clock, filesystem, image loader, monitor topology, and utilization sampler; WindowsEnv 25H2 integration tests for tray, WorkerW, WARP and CPU mode; later physical GPU/multi-monitor acceptance.

**Target Platform**: Windows 11 24H2 and newer (WindowsEnv 25H2 for development). x64 first; other architectures not promised.

**Project Type**: Local Windows desktop / system-tray application with wallpaper rendering, no external service.

**Performance Goals**: Normal idle: zero new-frame render calls, zero utilization samples and zero directory scans except the configured due-time empty-state check. **During animation, use vertical synchronization plus the user-configurable `max_fps` cap (default 120)**, targeting `min(monitor_refresh_hz, max_fps)` and generally using DXGI `Present(1,0)` on eligible frames. When `max_fps` is below refresh, schedule/skip unneeded frame production before rendering; VSync alone is insufficient to enforce a lower FPS cap. No fixed 40ms frame interval or busy-wait loop. Compute progress from monotonic elapsed time against the historical C# transition time scale and **preserve all original phase/math functions**. At 60/120/144Hz do not shorten the animation due to higher frame count. When idle, do not keep the render loop alive. Hardware limitations (WARP, multiple refresh rates, DWM) must be measured and recorded, not claimed as guaranteed FPS.

**Constraints**: Single global shuffled image queue; shared monitors use it sequentially; hard separation between startup choreography and ordinary animation math; do not change `master` or `directx_build`; do not write a polling process or monitoring thread. Single adjustable load threshold (default 60%, high is >=, normal is <) and positive YAML max_fps (default 120); no independent recovery value, no hysteresis or lengthy calibration. Windows sign-in autostart registration is user-scoped, updated immediately on tray interaction and never silently written when the feature is merely documented. Only call Windows APIs on intended Windows threads, serialize render/context ownership, handle loss of desktop shell and display reconfiguration. No images means no image rendering, but tray and event loop remain active.

**Scale/Scope**: One desktop session, multiple monitors with negative virtual-desktop coordinates, zero to many local photos, one main + ten background pictures per monitor at startup; queue accesses serialized; the image cache must avoid retaining unbounded decoded textures.

## Constitution Check

*GATE: review before Phase 0 research and again after Phase 1 design.*

| Principle | Design response / gate |
| --- | --- |
| I. Preserve C# behavior | `animation/legacy_math.rs` uses exact source equations; table-driven golden tests for float curves, phase boundaries, image placement, screen-area weights. Startup is a documented owner-approved exception, not an excuse to modify normal transitions. **Pass at design level.** |
| II. Rust + D3D11/VSync | Win32 shell and Direct3D11 behind explicit adapters; hardware device with WARP for WindowsEnv; submit frames with VSync capped at `min(refresh_hz, max_fps)` during animations, with default cap 120; elapsed-time mapping retains C# math/duration; zero idle frames; test display refresh differences and device loss. **Pass at design level; test hardware later.** |
| III. Tray + YAML + autostart | Typed YAML with 30s default and 60% single threshold plus validation; tray commands and an immediate per-user HKCU Run checkbox that writes, reads back and reports failure without admin rights. OS autostart state is not mirrored in YAML. **Pass at design level.** |
| IV. Simple due-time guard | Scheduler waits, then performs one short bounded utilization request at each normal due time; >= YAML threshold skips, < allows; no hysteresis, no long-term calibration or idle sampling. Startup ignores the guard; empty state rescans only once per due tick. **Pass at design level; actual GPU metric needs PoC.** |
| V. Testable equivalence | Deterministic animation, monitor geometry, image sequence, timer and empty-state tests, plus recorded WindowsEnv and physical GPU acceptance. **Pass at design level; execution pending.** |
| VI. Shared image provider | A single injected ImageQueue owns sorted cycle snapshots and reshuffle; empty result blocks rendering and sets tray status; automatic rescan only on due tick. **Pass at design level.** |

**Outstanding validation, not waived**: D3D11 visual equivalence on actual screens; Windows 11 24H2 WorkerW semantics; GPU utilization metric availability and bounded sampling latency (precision is deliberately best-effort rather than a lengthy calibration); per-monitor VSync under WorkerW; actual WARP behavior on WindowsEnv; package signing/distribution. Any PoC failure reopens the design, not the frozen C# math.

## Project Structure

### Documentation (this feature)

```text
specs/001-rust-wallpaper-app/
├── spec.md
├── reference-audit.md
├── checklists/requirements.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    ├── config.example.yaml
    ├── config-schema.md
    ├── tray-behavior.md
    └── autostart.md
```

### Source Code (future work, not yet created)

```text
rust/
├── Cargo.toml
├── rust-toolchain.toml
├── src/
│   ├── main.rs                # Win32 message loop, startup/exit
│   ├── settings.rs            # YAML parser/validator, atomic reload incl. max_fps
│   ├── tray.rs                # Shell_NotifyIcon state and commands
│   ├── autostart.rs           # Per-user HKCU Run toggle/read-back
│   ├── app_state.rs           # Lifecycle and orchestration
│   ├── image_queue.rs         # One shuffled cycle across all monitors
│   ├── scheduler.rs           # Deadline-only tick, pause/resume/empty retry
│   ├── usage.rs               # One-shot GPU/CPU utilization interface
│   ├── monitor.rs             # Enumeration, DPI, virtual geometry
│   ├── shell.rs               # WorkerW and Explorer lifecycle adapter
│   ├── animation/
│   │   ├── mod.rs
│   │   ├── legacy_math.rs      # Frozen normal transition math
│   │   └── startup.rs          # Owner-approved intro choreography
│   └── render/
│       ├── mod.rs
│       ├── d3d11.rs
│       ├── image_decode.rs
│       └── swapchain.rs       # VSync presentation under per-monitor FPS cap
└── tests/
    ├── animation_golden.rs
    ├── image_queue_cycle.rs
    ├── scheduler_load.rs
    ├── config_contract.rs
    ├── windows_integration.rs
    ├── vsync_pacing.rs        # 60/120/144/240Hz and editable max_fps
    └── autostart_registry.rs
```

**Structure Decision**: A single Rust crate under `rust/` keeps the original C# root untouched while retaining testable modules; promote to a Cargo workspace only when compile-time integration warrants it. No continuous renderer, duplicate per-monitor picker, or installer service is needed.

## Workstreams and Ordering

1. **Ground-truth and Windows API spikes**: extract immutable `master` animation samples and edge cases; confirm WindowsEnv build/runtime, D3D11 WARP path, WorkerW integration, tray notifications, and whether a simple one-shot bounded GPU/CPU load sample is available. If an API is unsuitable, document alternatives before writing the production adapter.
2. **Platform-free core**: typed config (30 seconds / one 60% threshold / max_fps default 120), image queue with cycle-end rescan, deterministic monitor selection, scheduler with empty-source deadlines and GPU gate, legacy animation math; lock in golden tests first.
3. **Startup composition**: per-monitor center image then ten simultaneous-but-staggered slide-ins; reuse the shared queue, ignore load sensing during initial composition; test layering and completion-to-idle.
4. **Windows adapters**: system tray with immediate HKCU Run autostart registration, WorkerW, monitor change events, image decode/texture upload and D3D11 renderer with VSync (sync interval 1), max_fps-aware scheduling and hardware/WARP selection. Keep window-handle lifetime and drawing thread explicit.
5. **Integration & acceptance**: WindowsEnv 25H2 smoke tests, YAML defaults/overrides, 59.9/60/60.1% guard boundaries, tray autostart registry write/delete/read-back, Explorer restart, monitor hot-plug, CPU-sampler fallback, 60/120/144/240Hz VSync + max_fps=60/120/240 pacing when supported; physical GPU utilization plus multi-monitor visual acceptance, logs and release readiness.
6. **Packaging**: deliver Windows executable and documented install/start/quit behavior after validation. Installer format and signing are not silently assumed.

## State and Deadline Contract

- `NoImages`: keep tray and slideshow timer active. On each due tick, perform **one** directory scan, then if still empty schedule `now + slideshow_interval`; no GPU read and no drawing. Explicit `ReloadConfig` may trigger one immediate scan.
- `Startup`: as soon as valid images are available for a fresh launch, initialize the per-monitor main + ten background slide-ins; do not read GPU/CPU. After completion schedule the first normal interval. The case of a late recovery from no-images while a stale composed wallpaper exists is handled by the `Startup`/`Idle` state distinction and must not overwrite an already valid static image unnecessarily.
- `Idle`: wait without drawing or monitoring. On due tick, request a short bounded total-GPU (test CPU) measurement; if usage >= configured one threshold (default 60%), keep current image and reschedule `now + interval`; otherwise (< threshold) claim the next image and run the unchanged normal transition. No recovery latch or background averaging.
- `PausedByUser`: retain current static wallpaper; only explicit unpause/reload/quit events trigger work. The details of pause/resume timer semantics are a proposed UI contract in [tray-behavior.md](contracts/tray-behavior.md).
- `Rendering`: present only while animating, pace eligible presentation with display VSync (`Present(1,0)` where compatible), throttle render/frame submission to no more than `min(refresh_hz, max_fps)` where default max_fps=120, and derive original animation parameter from monotonic elapsed time so neither refresh nor cap alters animation duration. Keep source cosh/phase/transform math unchanged; no load checks mid-animation. On completion return to `Idle` with no render loop and no catch-up burst.
- `ExplorerRecreated`/`DisplaysChanged`: restore the desktop surface and screen records without mutating the random slideshow algorithm. Re-rendering an existing static frame to restore lost resources is a recovery action and is not an unauthorized slideshow transition.

## Design Risks and Approved Parameters

- **Owner-approved defaults**: `slideshow_interval_seconds: 30`, `load_guard.threshold_percent: 60`, `load_guard.enabled: true`; both numbers are user-changeable in YAML. There is NO recovery threshold, hysteresis, or background polling period. Exact behavior is >=60% skip / <60% allow at each scheduled due time; on WindowsEnv CPU mode the same rule applies.
- **Sampling semantics**: Favor a cheap, quick best-effort system-wide utilization reading at the transition deadline; do NOT run long calibration, a periodic sampler or an accuracy-maximizing history collector. If the chosen Windows API needs multiple snapshots, they MUST occur only as part of one short bounded due-time operation. Represent unavailable readings as `Unknown` rather than zero; proposed failure handling is skip once and retry next full interval, pending Windows PoC.
- **GPU aggregation**: Select a lightweight reasonably representative system-wide GPU usage metric; do not naively sum engine percentages or claim precision beyond the chosen API. Record adapter/engine policy and limitations in diagnostics; do not add costly long-run calibration merely to refine the 60% gate.
- **No-image recovery**: User selected auto-recovery on the normal timer. A first-time successful recovery into `Startup` is a design proposal; verify the user experience before finalizing implementation. No separate image polling process is permitted.
- **Presentation pacing and cap**: DXGI SyncInterval=1 requests vertical sync, but does NOT itself enforce `max_fps < refresh_hz`. Throttle scheduled frames before rendering/submission using monotonic time, avoid sleep-free busy spins, and present at eligible vblanks; target `min(refresh_hz, max_fps)` with actual rates allowed to be lower due to display cadence, WARP/DWM, and scheduling jitter. At 144Hz with `max_fps=120`, skipped/irregular vblank pacing may yield less than perfectly even 120FPS; do not claim exact 120. A spanning window may sync with one output, so use per-monitor presentation surfaces where feasible. Keep C# curve/time scale independent of number of frames. On YAML reload the cap becomes effective on next scheduled animation frame.
- **Autostart state**: Implement per-user Windows sign-in autostart using `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` with a single app-owned value. Quote the installed executable path, verify the path is usable and registry command-line length; clicking the tray item writes/deletes immediately and reads back before updating checkmark. On startup and opening the tray, read OS state rather than YAML. OS may defer launch or user may disable the startup item externally; do not promise exact boot-time execution.
- **Visual equivalence**: D3D11 sampling, antialiasing, alpha blending and pixel placement may differ from GDI+ even with identical math. Use pixel/geometry-based tests to decide what needs matching; never change source equations to mask blending differences.

## Design Review After Phase 1

- All dependencies have a plan artifact in [research.md](research.md); hardware-specific behavior is explicitly flagged for proof-of-concept.
- Entities and transitions are documented in [data-model.md](data-model.md).
- User-facing YAML, tray controls and the immediate OS autostart transaction are defined as contracts in `contracts/`; approved values are not only hardcoded in source, and registry autostart is not duplicated in YAML.
- [quickstart.md](quickstart.md) describes repeatable tests; real results will be recorded when the Rust application exists.
- **Constitution gates remain passed at design level.** Release acceptance is explicitly pending actual tests. No justified exceptions beyond the user-approved opening scene. Newly added VSync pacing and system-tray autostart are approved requirements, not changes to the C# transition math.

## Complexity Tracking

No constitution violations are proposed. The Win32 adapter layer exists to keep unsafe platform APIs out of pure deterministic math/queue/scheduler code and is not an alternative animation engine.

## Approved Product Parameters (2026-11-03)

| Setting / behavior | Approved contract |
| --- | --- |
| Slideshow interval | `30` seconds by default, editable through YAML |
| System GPU utilization gate | One `60%` threshold by default, editable through YAML; at due time >= threshold skip, otherwise play |
| WindowsEnv fallback | The same one-threshold rule on system CPU usage where GPU is unavailable |
| Utilization sampling | Quick bounded measurement only after a normal slideshow deadline; no long calibration |
| Animation frame pacing | DXGI VSync plus adjustable `max_fps` (default 120), target `min(refresh_rate, max_fps)` only during active animation; no 40ms frame cap, no idle rendering |
| Original normal animation | Preserve historic math and time scale using monotonic elapsed-time progression independent of display refresh |
| Tray sign-in startup | User checkbox writes or removes current-user Windows autostart registry entry immediately and verifies via read-back; no admin or YAML toggle |
