# Implementation Plan: YUAUY Wallpaper — Rust + Direct3D 11

**Branch**: `rust_build` | **Date**: 2026-11-01 | **Spec**: [spec.md](spec.md)

**Input**: `spec.md` (FR-001～FR-026), [Constitution v2.2.1](../../.specify/memory/constitution.md), [舊版行為稽核](reference-audit.md).

**Phase**: Spec Kit `$speckit-plan` / implementation design; **no Rust implementation or Windows integration tests have been run yet**. This document is an implementation proposal, not a release certificate.

## Summary

Create an independent Windows 11 desktop wallpaper executable in Rust using Direct3D 11. Preserve the latest common C# animation math, monitor-area-weighted slideshow selection, and selection/placement rules. Replace the legacy accelerated startup with the owner-approved per-monitor composition: one center image followed by ten staggered overlapping slides behind it. A shared shuffle-without-replacement image queue handles every monitor and the startup; it rescans only on cycle rollover, explicit YAML reload, or a scheduled empty-state retry. The tray controls settings and status. GPU load is measured only when a regular transition is due; first startup bypasses the load guard. On the GPU-less WindowsEnv, use a software rendering path and test the resource gate with CPU input.

## Technical Context

**Language/Version**: Rust stable, pinned via `rust-toolchain.toml` during implementation; build target `x86_64-pc-windows-msvc`.

**Primary Dependencies**: `windows` crate for Win32/COM/D3D11/DXGI/PDH integration; `serde` and maintained `yaml_serde` for YAML; `rand` for shuffled image order; Win32/WIC image decoding (confirm format and alpha behavior in Windows PoC). Exact versions and features are to be pinned and audited at implementation time. The historical C# and unfinished C++ branches are not Rust dependencies.

**Storage**: User-editable YAML at a per-user Windows configuration location; user-owned image folders, never uploaded or committed; transient image metadata and textures only. Application-local diagnostics with no image bytes or private folder paths in routine logs.

**Testing**: Rust unit/property/integration tests with deterministic RNG and injected clock, filesystem, image loader, monitor topology, and utilization sampler; WindowsEnv 25H2 integration tests for tray, WorkerW, WARP and CPU mode; later physical GPU/multi-monitor acceptance.

**Target Platform**: Windows 11 24H2 and newer (WindowsEnv 25H2 for development). x64 first; other architectures not promised.

**Project Type**: Local Windows desktop / system-tray application with wallpaper rendering, no external service.

**Performance Goals**: Normal idle: zero new-frame render calls, zero utilization samples and zero directory scans except the configured due-time empty-state check. During animation: reproduce C# time samples and 40 ms frame cadence as an equivalence baseline, not a claim of a fixed FPS. Use GPU without keeping a continuous render loop alive.

**Constraints**: Single global shuffled image queue; shared monitors use it sequentially; hard separation between startup choreography and ordinary animation math; do not change `master` or `directx_build`; do not write a polling process or monitoring thread. Only call Windows APIs on intended Windows threads, serialize render/context ownership, handle loss of desktop shell and display reconfiguration. No images means no image rendering, but tray and event loop remain active.

**Scale/Scope**: One desktop session, multiple monitors with negative virtual-desktop coordinates, zero to many local photos, one main + ten background pictures per monitor at startup; queue accesses serialized; the image cache must avoid retaining unbounded decoded textures.

## Constitution Check

*GATE: review before Phase 0 research and again after Phase 1 design.*

| Principle | Design response / gate |
| --- | --- |
| I. Preserve C# behavior | `animation/legacy_math.rs` uses exact source equations; table-driven golden tests for float curves, phase boundaries, image placement, screen-area weights. Startup is a documented owner-approved exception, not an excuse to modify normal transitions. **Pass at design level.** |
| II. Rust + D3D11 | Win32 shell and Direct3D11 behind explicit adapters; hardware device with supported WARP path for WindowsEnv; device-loss and Explorer-respawn tests. **Pass at design level; test hardware later.** |
| III. Tray + YAML | Win32 message loop and tray commands; typed YAML schema with validation, atomic application of a new valid config, error indication. **Pass at design level.** |
| IV. Load guard on due time | Scheduler first waits, then performs a bounded one-shot utilization request before choosing an image; no sampling in idle or startup. When no images, due tick rescans once, without querying GPU. **Pass at design level; actual GPU accounting needs PoC.** |
| V. Testable equivalence | Deterministic animation, monitor geometry, image sequence, timer and empty-state tests, plus recorded WindowsEnv and physical GPU acceptance. **Pass at design level; execution pending.** |
| VI. Shared image provider | A single injected ImageQueue owns sorted cycle snapshots and reshuffle; empty result blocks rendering and sets tray status; automatic rescan only on due tick. **Pass at design level.** |

**Outstanding validation, not waived**: D3D11 visual equivalence on actual screens; Windows 11 24H2 WorkerW semantics; GPU utilization metric attribution, latency and sampling accuracy; actual WARP behavior on WindowsEnv; package signing/distribution. Any PoC failure reopens the design, not the frozen C# math.

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
    └── tray-behavior.md
```

### Source Code (future work, not yet created)

```text
rust/
├── Cargo.toml
├── rust-toolchain.toml
├── src/
│   ├── main.rs                # Win32 message loop, startup/exit
│   ├── settings.rs            # YAML parser/validator, atomic reload
│   ├── tray.rs                # Shell_NotifyIcon state and commands
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
│       └── swapchain.rs
└── tests/
    ├── animation_golden.rs
    ├── image_queue_cycle.rs
    ├── scheduler_load.rs
    ├── config_contract.rs
    └── windows_integration.rs
```

**Structure Decision**: A single Rust crate under `rust/` keeps the original C# root untouched while retaining testable modules; promote to a Cargo workspace only when compile-time integration warrants it. No continuous renderer, duplicate per-monitor picker, or installer service is needed.

## Workstreams and Ordering

1. **Ground-truth and Windows API spikes**: extract immutable `master` animation samples and edge cases; confirm WindowsEnv build/runtime, D3D11 WARP path, WorkerW integration, tray notifications, and whether a one-shot bounded GPU/CPU sample is accurate enough. If an API is unsuitable, document alternatives before writing the production adapter.
2. **Platform-free core**: typed config, image queue with cycle-end rescan, deterministic monitor selection, scheduler with empty-source deadlines and GPU gate, legacy animation math; lock in golden tests first.
3. **Startup composition**: per-monitor center image then ten simultaneous-but-staggered slide-ins; reuse the shared queue, ignore load sensing during initial composition; test layering and completion-to-idle.
4. **Windows adapters**: system tray, WorkerW, monitor change events, image decode/texture upload and D3D11 renderer with hardware/WARP selection. Keep window-handle lifetime and drawing thread explicit.
5. **Integration & acceptance**: WindowsEnv 25H2 smoke tests, abnormal YAML/source tests, Explorer restart, monitor hot-plug, CPU-sampler fallback; physical GPU utilization plus multi-monitor visual acceptance, logs and release readiness.
6. **Packaging**: deliver Windows executable and documented install/start/quit behavior after validation. Installer format and signing are not silently assumed.

## State and Deadline Contract

- `NoImages`: keep tray and slideshow timer active. On each due tick, perform **one** directory scan, then if still empty schedule `now + slideshow_interval`; no GPU read and no drawing. Explicit `ReloadConfig` may trigger one immediate scan.
- `Startup`: as soon as valid images are available for a fresh launch, initialize the per-monitor main + ten background slide-ins; do not read GPU/CPU. After completion schedule the first normal interval. The case of a late recovery from no-images while a stale composed wallpaper exists is handled by the `Startup`/`Idle` state distinction and must not overwrite an already valid static image unnecessarily.
- `Idle`: wait without drawing or monitoring. On due tick, request one bounded utilization measurement; if high, keep current image and reschedule `now + interval`; otherwise claim the next image and run the unchanged normal transition.
- `PausedByUser`: retain current static wallpaper; only explicit unpause/reload/quit events trigger work. The details of pause/resume timer semantics are a proposed UI contract in [tray-behavior.md](contracts/tray-behavior.md).
- `Rendering`: continue to completion even if utilization later rises, and preserve exact animation math. After completion return to `Idle`; no catch-up burst.
- `ExplorerRecreated`/`DisplaysChanged`: restore the desktop surface and screen records without mutating the random slideshow algorithm. Re-rendering an existing static frame to restore lost resources is a recovery action and is not an unauthorized slideshow transition.

## Design Risks and Provisional Parameters

- **Settings defaults are proposals**, not user-approved thresholds: release-mode C# used 30 seconds between normal transitions; proposal is `slideshow_interval_seconds: 30`; `gpu_pause_at_percent: 80`, `gpu_resume_at_percent: 60`, `load_guard_enabled: true`. Validate threshold ordering and require explicit approval before labeling these values final.
- **Sampling semantics**: Many performance-counter metrics need two points to calculate a rate. Perform both samples *after* the scheduled deadline, within a bounded one-shot operation; never run a background timer to precompute the value. Define unknown/failed reading distinctly from 0%; if no usable sample is produced, prefer a conservative skip for that due transition, log once and retry at the next full interval. This is a plan-level safety proposal, subject to an actual Windows GPU test.
- **GPU aggregation**: Do not naively sum unrelated GPU Engine usage percentages and claim it is a true device-wide 0–100% number. Verify per-adapter/engine aggregation against Windows tooling on physical GPUs; for multiple adapters document chosen aggregation in logs and tests.
- **No-image recovery**: User selected auto-recovery on the normal timer. A first-time successful recovery into `Startup` is a design proposal; verify the user experience before finalizing implementation. No separate image polling process is permitted.
- **Visual equivalence**: D3D11 sampling, antialiasing, alpha blending and pixel placement may differ from GDI+ even with identical math. Use pixel/geometry-based tests to decide what needs matching; never change source equations to mask blending differences.

## Design Review After Phase 1

- All dependencies have a plan artifact in [research.md](research.md); hardware-specific behavior is explicitly flagged for proof-of-concept.
- Entities and transitions are documented in [data-model.md](data-model.md).
- User-facing YAML and tray controls are defined as proposed contracts in `contracts/`, not secretly hardcoded.
- [quickstart.md](quickstart.md) describes repeatable tests; real results will be recorded when the Rust application exists.
- **Constitution gates remain passed at design level.** Release acceptance is explicitly pending actual tests. No justified exceptions beyond the user-approved opening scene.

## Complexity Tracking

No constitution violations are proposed. The Win32 adapter layer exists to keep unsafe platform APIs out of pure deterministic math/queue/scheduler code and is not an alternative animation engine.
