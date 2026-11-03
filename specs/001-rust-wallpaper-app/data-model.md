# Phase 1 Data Model and State Machines

**Source**: [spec.md](spec.md), [plan.md](plan.md). All names are proposed Rust types; no source code exists yet.

## Core entities

| Entity | Fields (conceptual) | Invariants |
| --- | --- | --- |
| `WallpaperSettings` | `image_folders: Vec<PathBuf>`; `slideshow_interval: Duration`; `guard.enabled`; `guard.threshold_percent`; `max_fps: NonZeroU32` (default 120); optional startup shuffle seed for tests | Validated before atomic replacement; nonempty paths may be invalid on disk but report the absence safely; single threshold in [0,100], default 60; interval >0, default 30 seconds; max_fps positive integer, default 120; no resume threshold or background polling interval. |
| `MonitorGeometry` | Stable monitor ID, bounds `left/top/right/bottom`, DPI, area, center; render surface generation | Negative virtual coordinates allowed; weighted selection by area; monitor reconfiguration invalidates affected surfaces but not the original selection equation. |
| `ImageQueue` | Configured folder set, unique candidate-path snapshot, shuffled order, cursor, cycle generation, RNG state | A path is claimed at most once per completed cycle; all monitors share one queue; rescan and shuffle at rollover, explicit reload or due-time empty-source check only; mutex/owner serializes simultaneous claims. |
| `ImageCandidate` | Original file path, extension, decode status, dimensions after validation | Unsupported or corrupt image cannot be used as drawable; failed claim must not advance render state or spin. |
| `StartupComposition` | Per-monitor `main`, up to 10 background placement records, delay offsets, depth ordering, current animation progress | Main visible before any background transition; background layers strictly behind main; one + ten entries per monitor when >=1 usable image; no GPU/CPU samples during startup. |
| `ComposedWallpaper` | Current static scene per monitor (background stack, active main), asset IDs | Remains static in idle or high-load skip. Retained scene survives empty-source status where resources permit; restored on Explorer/display changes. |
| `UsageSample` | `gpu_total` or `cpu_total` source, sample timestamp, quick best-effort `percent` or `Unknown(reason)` | Only requested at normal-transition due time; one short bounded measuring operation per due tick; no long calibration/history; unavailable != 0%. |
| `Scheduler` | `state`, `next_deadline: Instant`, `interval`, `last_completed_transition`, `threshold_percent` | No work between deadlines except user/shell events; overdue work is not accumulated; on high-load skip, next due = decision time + full interval. |
| `TrayState` | Status (`Ready`, `NoImages`, `Paused`, `ConfigurationError`, `LoadUnknown`), checked auto-start state from Windows, menu enabled states and errors | Status is visible and actionable; auto-start checkmark reflects verified OS registration, not YAML; quit/menu remain operable. |

## Windows autostart and refresh-rate entities

| Entity | Fields (conceptual) | Invariants |
| --- | --- | --- |
| `WindowsAutostartRegistration` | Current-user Run key, one app-owned value name, quoted executable command, `enabled` derived from read-back, optional error | Toggled on tray click with immediate create/delete and verification; failure must not falsely set the checkmark; do not alter unrelated values or require admin; do not duplicate in YAML. |
| `PresentationTiming` | Monitor ID, effective refresh rate, YAML max_fps, `min(refresh_hz, max_fps)` target, next eligible frame instant, VSync/SyncInterval request, animation `started_at` monotonic timestamp, C# reference time scale | During active animation frame submissions target no more than min(refresh rate, max_fps) with VSync on eligible Presents; do not render/sleep-spin unneeded frames. Compute normal curve phase from elapsed time, independent of number of frames. After completion no continuous Present. |

## Relationships

- One `WallpaperSettings` configures one `ImageQueue`, one `Scheduler` and one load-sampler backend, but does not own Windows sign-in autostart state.
- One `ImageQueue` supplies all `StartupComposition` and all normal transitions; each monitor does **not** own a random picker.
- One `MonitorGeometry` record and `ComposedWallpaper` exist for each active monitor; the animation engine computes per-monitor changes but the scheduler selects the target monitor using the historical shared weighted rule.
- One `TrayState` reflects the combination of settings validity, queue availability and scheduler pause status; UI never mutates renderer state directly outside serialized app commands.

## State transitions

```text
                  Startup with valid images
Uninitialized --------------------------------> Startup
     |                                         |
     | no usable image                         | composition complete
     v                                         v
NoImages -- due time scan finds images --> Startup or Idle(*)
     |                                         |
     | due time scan still empty               | next normal deadline
     +------------------- NoImages             v
                                           LoadCheck
                                        /             \
                                overloaded        acceptable
                                    |                  |
                                    v                  v
                                   Idle <--- RenderTransition
                                    |
                            pause / resume / quit

(*) When no previous scene exists, Startup; when a valid static scene was retained,
    restore/resume without forcibly replaying the startup choreography.
```

- `NoImages` due: rescan once, no GPU read; none -> schedule next full interval; some -> remove warning, enter appropriate rendering lifecycle. User-triggered YAML reload can perform the same check instantly after validation.
- `Idle` due: one quick bounded utilization measuring operation. If measured usage >= the single YAML threshold (default 60%) -> keep existing composed image, next deadline after full YAML interval (default 30s). If usage < threshold -> claim next image, perform unchanged ordinary transition and schedule next deadline. No hysteresis or second threshold.
- `Startup`: claim main then ten backgrounds *per monitor* via shared queue; no utilization reads; staggered animation with overlapping time windows; present only active animation frames using target-monitor VSync and max_fps cap, and ensure final static scene exists before moving to `Idle`.
- `PauseByUser`: no new transitions; user can reload config, toggle Windows sign-in autostart immediately, or exit. Manual resume scheduling semantics remain a UI contract to verify before release.
- `ShellOrDisplayRecovery`: preserve logical scene, recreate native resources and per-monitor present timing as required; no arbitrary new image selection or animation formula changes.

## Consistency and validation

1. **Queue uniqueness**: shuffle a deduplicated path set (on current file identity/path policy); claim each once within a cycle. Across cycle boundaries the same filename may reappear, including a single-image folder.
2. **Files disappearing mid-cycle**: if an advertised path is missing/unreadable, do not render it; mark as failed for this cycle; limit error reporting and continue to the next candidate, never an unbounded loop. Re-evaluate zero-image status deterministically.
3. **Atomic YAML**: parse into candidate settings -> validate thresholds and paths -> construct a safe updated queue/schedule -> commit all valid fields at once, else preserve last valid runtime settings.
4. **Clock**: use monotonic timestamps for deadlines to avoid leap/clock changes; wall-clock only for diagnostics.
5. **Resource use**: image bytes and decoded GPU textures have bounded lifetimes; a queue stores paths, not all decoded images; startup uses only visible/fading assets and resources necessary for the frame.
6. **Sampling**: `UsageSample::Unknown` is distinct from numeric zero and handled with a bounded policy; no samples in `NoImages`, `Idle` between ticks, or `Startup`. Only one threshold is stored; no historical load state or resumption percentage.

## Remaining review (design-level, not hidden requirements)

- YAML defaults **are approved**: 30s interval, a single adjustable 60% threshold, and positive max_fps default 120. Startup autostart uses the Windows HKCU Run registration, not YAML.
- Adapter-specific GPU total and invalid-sample fallback must be verified by an instrumented Windows PoC.
- Actual native shell/window restoration path must be tested on Windows 11 24H2/25H2, not inferred from old C# window handles.
