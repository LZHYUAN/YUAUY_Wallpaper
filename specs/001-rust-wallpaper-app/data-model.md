# Phase 1 Data Model and State Machines

**Source**: [spec.md](spec.md), [plan.md](plan.md). All names are proposed Rust types; no source code exists yet.

## Core entities

| Entity | Fields (conceptual) | Invariants |
| --- | --- | --- |
| `WallpaperSettings` | `image_folders: Vec<PathBuf>`; `slideshow_interval: Duration`; `guard.enabled`; `guard.pause_at`; `guard.resume_at`; optional startup shuffle seed for tests | Validated before atomic replacement; nonempty paths may be invalid on disk but report the absence safely; percentages in [0,100], resume lower than pause; interval >0; no background polling interval. |
| `MonitorGeometry` | Stable monitor ID, bounds `left/top/right/bottom`, DPI, area, center; render surface generation | Negative virtual coordinates allowed; weighted selection by area; monitor reconfiguration invalidates affected surfaces but not the original selection equation. |
| `ImageQueue` | Configured folder set, unique candidate-path snapshot, shuffled order, cursor, cycle generation, RNG state | A path is claimed at most once per completed cycle; all monitors share one queue; rescan and shuffle at rollover, explicit reload or due-time empty-source check only; mutex/owner serializes simultaneous claims. |
| `ImageCandidate` | Original file path, extension, decode status, dimensions after validation | Unsupported or corrupt image cannot be used as drawable; failed claim must not advance render state or spin. |
| `StartupComposition` | Per-monitor `main`, up to 10 background placement records, delay offsets, depth ordering, current animation progress | Main visible before any background transition; background layers strictly behind main; one + ten entries per monitor when >=1 usable image; no GPU/CPU samples during startup. |
| `ComposedWallpaper` | Current static scene per monitor (background stack, active main), asset IDs | Remains static in idle or high-load skip. Retained scene survives empty-source status where resources permit; restored on Explorer/display changes. |
| `UsageSample` | `gpu_total` or `cpu_total` source, sample timestamp, `percent` or `Unknown(reason)` | Only requested at a normal-transition due time; one bounded measuring operation per due tick; unavailable != 0%. |
| `Scheduler` | `state`, `next_deadline: Instant`, `interval`, `last_completed_transition`, `resume_threshold_status` | No work between deadlines except user/shell events; overdue work is not accumulated; on high-load skip, next due = decision time + full interval. |
| `TrayState` | Status (`Ready`, `NoImages`, `Paused`, `ConfigurationError`, `LoadUnknown`), menu enabled states, last useful error | Status is visible and actionable; no repeated warning popups when empty; quit/menu remain operable. |

## Relationships

- One `WallpaperSettings` configures one `ImageQueue`, one `Scheduler` and one load-sampler backend.
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
- `Idle` due: one utilization measuring operation. Above pause threshold -> keep existing composed image, next deadline after full interval. At/below relevant recovery threshold -> claim next image, perform unchanged ordinary transition, then schedule next deadline.
- `Startup`: claim main then ten backgrounds *per monitor* via shared queue; no utilization reads; staggered animation with overlapping time windows; ensure final static scene exists before moving to `Idle`.
- `PauseByUser`: no new transitions; user can reload config or exit. Manual resume scheduling semantics are a plan proposal to verify with user before release.
- `ShellOrDisplayRecovery`: preserve logical scene, recreate native resources as required; no arbitrary new image selection or animation formula changes.

## Consistency and validation

1. **Queue uniqueness**: shuffle a deduplicated path set (on current file identity/path policy); claim each once within a cycle. Across cycle boundaries the same filename may reappear, including a single-image folder.
2. **Files disappearing mid-cycle**: if an advertised path is missing/unreadable, do not render it; mark as failed for this cycle; limit error reporting and continue to the next candidate, never an unbounded loop. Re-evaluate zero-image status deterministically.
3. **Atomic YAML**: parse into candidate settings -> validate thresholds and paths -> construct a safe updated queue/schedule -> commit all valid fields at once, else preserve last valid runtime settings.
4. **Clock**: use monotonic timestamps for deadlines to avoid leap/clock changes; wall-clock only for diagnostics.
5. **Resource use**: image bytes and decoded GPU textures have bounded lifetimes; a queue stores paths, not all decoded images; startup uses only visible/fading assets and resources necessary for the frame.
6. **Sampling**: `UsageSample::Unknown` is distinct from numeric zero and handled with a bounded policy; no samples in `NoImages`, `Idle` between ticks, or `Startup`.

## Remaining review (design-level, not hidden requirements)

- Proposed YAML threshold defaults are **not yet owner-approved** and will be reviewed before a release build.
- Adapter-specific GPU total and invalid-sample fallback must be verified by an instrumented Windows PoC.
- Actual native shell/window restoration path must be tested on Windows 11 24H2/25H2, not inferred from old C# window handles.
