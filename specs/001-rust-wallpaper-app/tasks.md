# Tasks: YUAUY Wallpaper — Rust + Direct3D 11

**Input**: [spec.md](spec.md) (FR-001～FR-030, SC-001～SC-017), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [reference-audit.md](reference-audit.md), [quickstart.md](quickstart.md), [contracts/](contracts/), [Constitution v2.4.0](../../.specify/memory/constitution.md).

**Branch**: `rust_build` — preserve historical `master` and `directx_build` unchanged.

**Tests**: Mandatory. The approved specification demands deterministic equation, geometry, image-queue, scheduling, YAML, WindowsEnv and physical-GPU acceptance tests. Write tests first for the relevant story, verify the meaningful failure, then implement the corresponding behavior; record actual PASS/FAIL/SKIPPED evidence instead of assuming WindowsEnv has a physical GPU.

**Organization**: Phase 1 Setup → Phase 2 Foundational → P1 story phases in dependency order (**US5 → US1 → US2**) → P2 (**US3 → US4**) → Final acceptance. Story IDs retain their original `spec.md` identifiers, even where execution order differs. Every task is intentionally unchecked; generating this list is not implementing it.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Parallelizable with other independent tasks within its phase after its prerequisites are complete; no simultaneous edits to the same file.
- **[US1]**: Existing C# animation parity, Windows desktop and monitor rendering.
- **[US2]**: Owner-approved per-monitor startup composition.
- **[US3]**: System tray, YAML, current-user Windows autostart.
- **[US4]**: Lightweight GPU/CPU load guard and deadline-only scheduling.
- **[US5]**: Shared no-repeat ImagePicker and no-image recovery.
- Every task includes the intended repository-relative file path; tests are written before behavior code within each story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create a separate Rust project, establish a reproducible Windows target and freeze legacy references without touching historical branches.

- [X] T001 Initialize the Rust Windows application/library skeleton at `rust/Cargo.toml`, `rust/src/main.rs`, `rust/src/lib.rs` and `rust/rust-toolchain.toml`, targeting `x86_64-pc-windows-msvc`; retain all C# files untouched.
- [X] T002 Declare and lock the audited `windows` crate Win32/D3D11/DXGI/WIC features and maintained Serde YAML, RNG and test dependencies in `rust/Cargo.toml` and `rust/Cargo.lock`, preserving the owner-approved Windows 11 baseline.
- [X] T003 [P] Add Windows Rust format, lint, build and test CI checks in `.github/workflows/rust-windows.yml`, working only under `rust/` and clearly separating hardware-only tests.
- [X] T004 [P] Update `.gitignore` for `rust/target/`, user YAML, image collections, screenshots and private Windows logs without ignoring tracked examples in `specs/001-rust-wallpaper-app/contracts/config.example.yaml`.
- [X] T005 [P] Extract immutable C# equation/geometry/phase constants and image-selection behavior from `master:YUAUY_Wallpaper/Form1.cs` and `master:YUAUY_Wallpaper/Image/ImagePicker.cs` into `rust/tests/fixtures/legacy_reference.json`, recording source revision and calculation assumptions in `specs/001-rust-wallpaper-app/reference-audit.md`; no easing substitutions.
- [ ] T006 Record the actual WindowsEnv 25H2 build number, installed Rust/MSVC/SDK availability, D3D11/WARP readiness and GPU absence/presence in `specs/001-rust-wallpaper-app/evidence/windowsenv-baseline.md`; mark unsupported checks SKIPPED, not PASS.
- [X] T007 [P] Document Rust build/test commands and test-data privacy requirements for WindowsEnv in `rust/README.md`, linked to `specs/001-rust-wallpaper-app/quickstart.md` and the pinned toolchain.

**Checkpoint**: A reproducible Rust project is present, source reference vectors are frozen and WindowsEnv prerequisites are recorded. A successful native build is required before starting feature code.

---

## Phase 2: Foundational (Blocking Technical Probes and Interfaces)

**Purpose**: Prove the Windows APIs and shared seams before committing production behavior. A failed probe is a design blocker, not permission to replace the C# formulas or silently abandon DirectX.

- [ ] T008 Build/run a D3D11 device-creation and clear/present probe in `rust/src/bin/probe_d3d11.rs`, explicitly testing WARP on WindowsEnv, distinguishing hardware/WARP driver type, and recording real output in `specs/001-rust-wallpaper-app/evidence/d3d11-warp.md`.
- [ ] T009 Implement a minimal reversible WorkerW/desktop-host probe in `rust/src/bin/probe_workerw.rs` and record whether the surface stays behind desktop icons on WindowsEnv in `specs/001-rust-wallpaper-app/evidence/workerw.md`; restore original desktop state after each probe.
- [ ] T010 [P] Build a dedicated timing/VSync + `max_fps` probe in `rust/src/bin/probe_pacing.rs` that logs actual frame submission times on available outputs, checks 60/120/144/240Hz with caps 60/120/240 when available, and writes measured/unsupported cases to `specs/001-rust-wallpaper-app/evidence/vsync-probe.md`.
- [ ] T011 [P] Probe quick bounded one-shot system-wide CPU and GPU utilization APIs (without idle monitoring or long calibration) in `rust/src/bin/probe_usage.rs`; document available counters, time cost, CPU fallback and missing GPU cases in `specs/001-rust-wallpaper-app/evidence/load-probe.md`.
- [X] T012 Implement a platform-independent monotonic clock, RNG, filesystem catalog and load-sampler abstraction in `rust/src/platform.rs`, injectable by deterministic tests; no new background sampling, filesystem watcher or drawing loop.
- [X] T013 [P] Define native error types and redacted diagnostics (no private image paths/bytes) in `rust/src/error.rs` and `rust/src/diagnostics.rs`, including `Unknown` load and invalid YAML distinctions.
- [X] T014 [P] Define monitor geometry, virtual desktop coordinates (including negative x/y), stable display IDs and per-monitor surface lifetime types in `rust/src/monitor.rs` without changing the C# area-weighted selection rule.
- [X] T015 [P] Define an object-safe rendering interface with hardware/WARP selection and owned Win32/DXGI handle lifetimes in `rust/src/render/mod.rs`; exclude any alternate GDI+ production renderer.
- [X] T016 Define the single-threaded application lifecycle and serializable commands (`Start`, `Tick`, `Reload`, `Pause`, `Resume`, `Quit`, `ShellRestart`, `DisplayChanged`) in `rust/src/app_state.rs`, leaving story-specific implementations for later phases.
- [ ] T017 [P] Define deterministic test doubles (virtual clock, sample sequence, temporary folder catalog, monitor layouts and recording renderer) in `rust/tests/support/mod.rs`, shareable by all five user stories.
- [ ] T018 Wire the Win32 hidden message-window event loop and safe shutdown skeleton in `rust/src/main.rs` and `rust/src/shell.rs`, with Windows API operations on their intended threads and no system-wide registration side effects.
- [ ] T019 Run foundational WindowsEnv WARP/WorkerW/UI probes and initial `cargo test`/`cargo clippy` verification, recording commands, outcomes and blockers in `specs/001-rust-wallpaper-app/evidence/foundation-gate.md`; stop for redesign if D3D11 desktop hosting cannot be proven.

**Checkpoint (blocking)**: WARP and desktop-host viability are evidenced, the Rust app builds, and deterministic interfaces exist. GPU-specific physical hardware checks remain explicitly pending rather than faked on WindowsEnv.

---

## Phase 3: User Story 5 — Shared Non-Repeating Image Source (Priority: P1)

**Goal**: A single global queue shared by all monitors and both animation modes, with one visit per file per cycle, folder rescan only at cycle boundaries, and a tray-visible no-images state.

**Independent Test**: For 0/1/3/11/22 valid images, deterministic claims never repeat before cycle exhaustion; additions are visible next cycle; missing files do not spin; a zero-image state draws nothing and schedules at most one rescan per normal interval. Covers FR-004, FR-005, FR-021～026 and SC-010～013.

### Tests — write before implementation

- [ ] T020 [P] [US5] Add red tests for 1/3/11/22-image global no-repeat shuffle cycles, single-image rollover, RNG determinism and shared two-monitor image claims in `rust/tests/image_queue_cycle.rs`.
- [ ] T021 [P] [US5] Add red tests for mid-cycle file additions/removals, scan-on-rollover only, unsupported JPG/JPEG/JFIF/PNG extensions, decode failures and bounded skip behavior in `rust/tests/image_queue_filesystem.rs`.
- [ ] T022 [P] [US5] Add red tests for zero images: zero decode/render/GPU reads, tray `沒有圖片` status model, one scan per due tick, user reload immediate scan and retained last scene in `rust/tests/no_images_recovery.rs`.

### Implementation

- [ ] T023 [P] [US5] Define image candidate/path identity, cycle snapshot, cursor, RNG state and a `NoImages` result contract in `rust/src/image_queue.rs`, guaranteeing one path claim per shuffled cycle across all monitors.
- [ ] T024 [US5] Implement one scan of configured nonrecursive image folders with case-insensitive JPG/JPEG/JFIF/PNG filtering and path deduplication in `rust/src/image_queue.rs`, using read-only file operations and tolerating invalid directories.
- [ ] T025 [US5] Implement deterministic seeded shuffle-without-replacement, serialize simultaneous image requests and roll over only after exhausting the current cycle in `rust/src/image_queue.rs`; never implement a per-monitor picker.
- [ ] T026 [US5] Implement new directory scan and fresh shuffled order only on cycle exhaustion or explicit settings reload in `rust/src/image_queue.rs`, safely handling removed/corrupted candidates without unbounded retries.
- [ ] T027 [P] [US5] Build a Windows WIC decoder facade that rejects unreadable/invalid image bytes, preserves original dimensions and isolates decode from queue sequencing in `rust/src/render/image_decode.rs`.
- [ ] T028 [US5] Implement the no-images status/event contract and one normal-deadline rescan with no load measurement in `rust/src/app_state.rs`, using the queue API; keep OS tray status plumbing for US3.
- [ ] T029 [US5] Integrate the image queue with a recording/mock compositor and prove one available image can supply eleven startup claims across completed cycles in `rust/tests/image_queue_cycle.rs`.
- [ ] T030 [US5] Run the US5 test files and save PASS/FAIL evidence, image counts, scan counts and outstanding decode issues to `specs/001-rust-wallpaper-app/evidence/us5-images.md`; do not mark the story complete if empty sources induce repeated work.

**Checkpoint**: The image provider and empty-recovery state are independently testable without a GPU or tray UI. Later phases may use them but must not duplicate their selection code.

---

## Phase 4: User Story 1 — Legacy Slideshow and Windows Desktop Rendering (Priority: P1) 🎯 MVP

**Goal**: A working desktop wallpaper renderer behind Windows icons, with the source C# `cosh` curves and four phases unchanged, monitor-area weighting retained, per-monitor static scene preserved, and animation frames paced by VSync + editable `max_fps`.

**Independent Test**: Fixed inputs reproduce source animation numbers and selected monitor/image placement; a WindowsEnv WARP renderer presents behind desktop icons (not as a floating overlay), handles display changes and stops rendering when idle. Covers FR-001～004, FR-012～014, FR-028, FR-030 and SC-001, SC-002, SC-005, SC-015, SC-017.

### Tests — write before implementation

- [ ] T031 [P] [US1] Add red golden-vector tests for all original `MathF.Cosh` curves, time thresholds 0.25/0.5/0.75, phase alpha/rotation/translation/scale, integer truncation and 40ms-era time-scale parity in `rust/tests/animation_golden.rs`, reading `rust/tests/fixtures/legacy_reference.json`.
- [ ] T032 [P] [US1] Add red deterministic monitor-area weighted selection, negative desktop coordinate, unequal DPI/resolution and retained per-monitor image-state tests in `rust/tests/monitor_selection.rs`.
- [ ] T033 [P] [US1] Add Windows-only red integration tests for desktop-window layering, WARP frame output, Explorer restart and display-change recreation in `rust/tests/windows_integration.rs`; tag external/hardware assumptions explicitly.
- [ ] T034 [P] [US1] Add red FPS-cap timing tests for `min(refresh_hz, max_fps)` with default 120, config updates (60,120,240), no idle frames, and unchanged animation duration at 60/120/144/240Hz in `rust/tests/vsync_pacing.rs`.

### Implementation

- [ ] T035 [US1] Port the *exact* current `master` equations, phase boundaries and time progression mapping into pure `rust/src/animation/legacy_math.rs`, using elapsed-time updates only to decouple animation length from number of VSynced frames, never changing the curve functions.
- [ ] T036 [US1] Implement original monitor-area weighted random target selection and per-monitor last-picture state in `rust/src/monitor.rs`, proving deterministic equality with C# reference cases.
- [ ] T037 [US1] Implement WorkerW attachment and desktop icon layering with Explorer restart and correct shutdown in `rust/src/shell.rs`, based on the verified T009 probe rather than hard-coded undocumented assumptions.
- [ ] T038 [US1] Implement D3D11 hardware/WARP device and swapchain management, resource release, device loss and color/alpha compatibility in `rust/src/render/d3d11.rs` and `rust/src/render/swapchain.rs`.
- [ ] T039 [US1] Implement image texture upload, white borders, placement, masked layers, rotation, scaling and alpha blending faithful to `master` in `rust/src/render/d3d11.rs` and `rust/src/render/image_decode.rs`.
- [ ] T040 [US1] Implement monotonic animation progress and an on-demand rendering loop that stops after transitions while preserving a static desktop image in `rust/src/animation/mod.rs` and `rust/src/app_state.rs`.
- [ ] T041 [US1] Implement active-frame VSync presentation with a monotonic `max_fps` limiter (default 120), sleeping/awaiting eligible frames rather than busy spinning or drawing skipped frames, in `rust/src/render/swapchain.rs`; never force 144/240Hz devices to render above configured max_fps.
- [ ] T042 [US1] Handle display add/remove, mixed-rate screens, negative origins, per-monitor surface regeneration and shell reattachment without reselection or unexpected animation in `rust/src/monitor.rs` and `rust/src/shell.rs`.
- [ ] T043 [US1] Connect the queue, unchanged ordinary transition math and Windows renderer into a standalone testable slideshow command path in `rust/src/app_state.rs` and `rust/src/main.rs`, without inventing a continuous frame loop.
- [ ] T044 [US1] Execute parity tests and WindowsEnv WARP desktop smoke checks, record actual 60/120/144/240Hz availability, achieved rates, known GDI+/D3D differences and limitations in `specs/001-rust-wallpaper-app/evidence/us1-render.md`.

**Checkpoint — first MVP**: A Rust desktop wallpaper can show and transition real local photos with preserved normal animation/multi-monitor behavior, with bounded VSynced FPS; physical GPU visual parity remains a release gate, not a claim of current completion.

---

## Phase 5: User Story 2 — Per-Monitor Main Image + Ten Staggered Photos Startup (Priority: P1)

**Goal**: Each monitor displays one central main photo first, then ten photos slide behind it with staggered overlapping delays; startup ignores GPU/CPU utilization and returns to static idle.

**Independent Test**: Single screen 11 photos, two screens 22 photos; layer order and random delays reproducible under fixed seed; one-folder-image inputs fill across queue cycles; no load reads during startup; static after completion. Covers FR-017～021 and SC-008, SC-009, SC-011.

### Tests — write before implementation

- [ ] T045 [P] [US2] Add red tests for each monitor's main-first ordering, exactly ten background images behind it, overlapping randomized start times and deterministic seeded positioning in `rust/tests/startup_composition.rs`.
- [ ] T046 [P] [US2] Add red tests for 1/2-screen counts (11/22), a single valid folder image across queue rollovers, 100%-usage startup still running with zero sampler invocations in `rust/tests/startup_load_bypass.rs`.

### Implementation

- [ ] T047 [P] [US2] Define per-monitor startup objects (main layer, background layers, randomized delay, motion, depth and completion state) in `rust/src/animation/startup.rs` independently from `legacy_math.rs`.
- [ ] T048 [US2] Claim one main plus exactly ten background entries per active monitor through the **single** shared `rust/src/image_queue.rs` API in `rust/src/animation/startup.rs`, allowing repeat only across exhausted cycles.
- [ ] T049 [US2] Implement main image display before background animations and ten overlapping randomized slide-in schedules, always beneath that monitor's main layer, in `rust/src/animation/startup.rs`.
- [ ] T050 [US2] Integrate startup sprites with the existing D3D11 renderer and `max_fps`-limited VSync presentation, without touching ordinary slideshow math, in `rust/src/render/d3d11.rs` and `rust/src/animation/startup.rs`.
- [ ] T051 [US2] Bypass any usage-sampler call through the complete startup state, then end at a static composed wallpaper and schedule the first normal slideshow deadline in `rust/src/app_state.rs`.
- [ ] T052 [US2] Handle image availability transitions before/after startup and shell reinitialization without rereading GPU or rerunning startup unnecessarily in `rust/src/app_state.rs`.
- [ ] T053 [US2] Run WindowsEnv startup with a deterministic sample set and capture single/multimonitor simulation screenshots and CPU/WARP limitations in `specs/001-rust-wallpaper-app/evidence/us2-startup.md`.
- [ ] T054 [US2] Mark the opening-scene implementation checkpoint against FR-017～FR-021 and SC-008/009/011 in `specs/001-rust-wallpaper-app/evidence/us2-startup.md`, confirming no change to `rust/src/animation/legacy_math.rs` golden outputs.

**Checkpoint**: The owner-approved introductory scene is a separate increment; the C# ordinary transition curves and area-weighted monitor choice remain unchanged.

---

## Phase 6: User Story 3 — Tray, YAML and Sign-In Autostart (Priority: P2)

**Goal**: Complete Traditional Chinese tray controls and typed YAML with approved defaults (30s interval, single 60% load threshold, max_fps 120), atomic reload and an immediate verified current-user Windows Run toggle.

**Independent Test**: Invalid YAML leaves the previous settings active; tray is available with no images; `max_fps` reload changes future presentation; autostart writes/deletes only its own HKCU Run value and reads back before checkmark changes. Covers FR-006～008, FR-013, FR-027, FR-029, FR-030 and SC-004, SC-014, SC-016, SC-017.

### Tests — write before implementation

- [ ] T055 [P] [US3] Add red YAML parser/default tests for 30s, one 60% threshold, `max_fps=120`, positive FPS/interval validation and rejection of obsolete `pause_at_percent`/`resume_at_percent` or unexpected keys in `rust/tests/config_contract.rs`.
- [ ] T056 [P] [US3] Add red transactional YAML reload tests for invalid files, Unicode Windows folder paths, no-image recovery and immediate `max_fps` effect on subsequent active frames in `rust/tests/config_reload.rs`.
- [ ] T057 [P] [US3] Add red mock-registry tests for checked/unchecked Run value state, immediate write/delete and read-back, access denied, moved path, idempotence and no unrelated value mutations in `rust/tests/autostart_registry.rs`.

### Implementation

- [ ] T058 [US3] Implement typed `WallpaperSettings` YAML parser and single-threshold/positive-FPS validation with explicit defaults in `rust/src/settings.rs`, following `specs/001-rust-wallpaper-app/contracts/config-schema.md`.
- [ ] T059 [US3] Implement validated atomic YAML reload (preserve last valid configuration, handle new folders and apply `max_fps` to future active frames) in `rust/src/settings.rs` and `rust/src/app_state.rs`.
- [ ] T060 [US3] Implement Traditional Chinese Win32 tray icon, status tooltip, hidden message-window menu dispatch and Explorer restart tray re-registration in `rust/src/tray.rs`.
- [ ] T061 [US3] Wire `沒有圖片`, paused, running, unknown load and configuration error states to the tray without repeated warning dialogs in `rust/src/tray.rs` and `rust/src/app_state.rs`.
- [ ] T062 [US3] Implement Open YAML, Reload, Pause, Resume and Quit menu actions with graceful render resources cleanup in `rust/src/tray.rs` and `rust/src/main.rs`.
- [ ] T063 [US3] Implement **current-user only** `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` registration (one app-owned value, quoted exe path), immediate menu-click write/delete, read-back and safe error reporting in `rust/src/autostart.rs` and `rust/src/tray.rs`; do not store this OS state in YAML.
- [ ] T064 [US3] Implement safe single-instance startup behavior and no duplicate wallpaper surfaces on Windows sign-in relaunch in `rust/src/main.rs` and `rust/src/app_state.rs`.
- [ ] T065 [US3] Verify tray with WindowsEnv, including status display, 30/60/120 defaults, YAML hot reload and no-picture error, recording screenshots/logs in `specs/001-rust-wallpaper-app/evidence/us3-tray.md`.
- [ ] T066 [US3] Verify immediate autostart registry toggle only in an authorized disposable WindowsEnv user/session, cleaning only the test-owned value and recording real success/failure in `specs/001-rust-wallpaper-app/evidence/us3-autostart.md`.

**Checkpoint**: Users can configure and control a complete desktop app from the tray without GUI settings window, and OS autostart is mutated immediately only upon the user's click.

---

## Phase 7: User Story 4 — Deadline-Only Simple GPU/CPU Guard (Priority: P2)

**Goal**: A lightweight single-threshold load decision only when a normal 30s (YAML-adjustable) slideshow deadline arrives; `>=60%` skip and `<60%` proceed, with exactly one short bounded measuring operation and full-interval retry; startup and empty state must not measure load.

**Independent Test**: Virtual clock and usage values 59.9/60/60.1%, unknown read, repeated high samples, idle with no reads, no new image claims before permission, CPU mode on GPU-less WindowsEnv. Covers FR-008～011, FR-014～016, FR-020, FR-026～027 and SC-003, SC-006, SC-007, SC-014.

### Tests — write before implementation

- [ ] T067 [P] [US4] Add red virtual-clock tests for the single YAML threshold (59.9 allow, 60 skip, 60.1 skip), three consecutive high skips, full 30s/default or overridden interval, no hysteresis and no catch-up in `rust/tests/scheduler_load.rs`.
- [ ] T068 [P] [US4] Add red tests for zero sampler calls in idle/first startup/no-images, load sample before image claim, single short bounded due-time measurement, and malformed/unknown reading in `rust/tests/scheduler_idle.rs`.
- [ ] T069 [P] [US4] Add red tests for no-image one-rescan-per-slideshow-tick, manual reload rescan and recovery without a directory watcher in `rust/tests/scheduler_empty.rs`.

### Implementation

- [ ] T070 [US4] Define `UsageSample::{Percent,Unknown}` and a one-shot `UtilizationSampler` contract in `rust/src/usage.rs`, requiring whole-system GPU on real hardware and whole-system CPU in WindowsEnv; never treat missing hardware as 0%.
- [ ] T071 [P] [US4] Implement short bounded whole-system CPU usage measurement for the test/fallback mode in `rust/src/usage.rs`, with no ongoing monitor, worker or calibration history.
- [ ] T072 [US4] Implement best-effort short bounded whole-system GPU usage detection, clearly documenting counter/adapter choice and `Unknown` failures in `rust/src/usage.rs`, using findings from `specs/001-rust-wallpaper-app/evidence/load-probe.md` rather than pretending WARP validates real GPU usage.
- [ ] T073 [US4] Implement normal-deadline transition ordering in `rust/src/scheduler.rs`: timer expiration → one load read → threshold comparison → *only if allowed* claim/decode image → existing normal animation; do not sample at startup or while idle.
- [ ] T074 [US4] Implement skip handling in `rust/src/scheduler.rs` with `next_due=decision_time+slideshow_interval`, retaining existing wallpaper, no shorter retry, no queue advance, no stored high/low hysteresis state.
- [ ] T075 [US4] Implement unknown-sample handling in `rust/src/scheduler.rs` and `rust/src/diagnostics.rs` as an explicit conservative single skip-and-retry policy (technical proposal subject to Windows integration review), not as zero or a rapid loop.
- [ ] T076 [US4] Integrate the scheduler with shared image queue, tray pause, empty-source recovery and active render lifecycle in `rust/src/app_state.rs`, ensuring no independent sampling timer is created.
- [ ] T077 [US4] Run deterministic and WindowsEnv CPU-mode scheduling acceptance tests, log actual call counts, sample latency, thresholds and limitations in `specs/001-rust-wallpaper-app/evidence/us4-load.md`.

**Checkpoint**: Normal slideshow starts only after an authorized deadline load check and preserves image selection, FPS capping and math; no background monitoring is active.

---

## Phase 8: Polish, Integration and Release Gates

**Purpose**: Validate the complete product, document real limitations and enforce hardware-specific release gates. Do not mark physical GPU or mixed-Hz acceptance PASS based on WARP alone.

- [ ] T078 [P] Re-run all Rust unit/property/integration tests (`cargo test`, `cargo fmt`, `cargo clippy`) and CI Windows build, capturing toolchain, warnings and actual PASS/FAIL status in `specs/001-rust-wallpaper-app/evidence/test-summary.md`.
- [ ] T079 Execute `specs/001-rust-wallpaper-app/quickstart.md` end-to-end on WindowsEnv 25H2 with WARP and CPU mode: opening scene, no-image tray, YAML `max_fps` reload, workerw/Explorer recovery, idle draw/sampler zero; record artifacts in `specs/001-rust-wallpaper-app/evidence/windowsenv-final.md`.
- [ ] T080 [P] Execute physical Windows 11 GPU and two real monitors acceptance (different areas, DPI and refresh where possible), verifying total-GPU load metric, source image placement and VSync plus 120 FPS cap in `specs/001-rust-wallpaper-app/evidence/physical-gpu-final.md`; if no access, mark BLOCKED/NOT TESTED and prevent release-ready claim.
- [ ] T081 [P] Measure CPU/GPU memory bounds, WARP vs hardware render overhead, startup ten-slide overlap and zero-work idle power/CPU use using redacted telemetry in `specs/001-rust-wallpaper-app/evidence/performance.md`; identify regressions and unbounded texture caching.
- [ ] T082 Re-run `rust/tests/animation_golden.rs` and `rust/tests/monitor_selection.rs` against source fixture hashes and freeze behavior evidence in `specs/001-rust-wallpaper-app/evidence/legacy-parity-final.md`; do not alter historical `master` or `directx_build`.
- [ ] T083 [P] Document Windows 11 installation/launch, YAML location/format (30s, 60%, 120FPS), tray controls, no-photo notification and GPU-less fallback in `rust/README.md` and `specs/001-rust-wallpaper-app/quickstart.md` without committing private photos.
- [ ] T084 Implement and test release packaging, uninstall/current-user Run entry cleanup and portable-exe path changes in `rust/package/README.md` and `rust/src/autostart.rs`, leaving user sign-in preferences alone on normal Quit.
- [ ] T085 [P] Audit Cargo dependencies, Windows API error handling, image decoder robustness, Unicode paths, dangling handles and registry entry scope in `rust/Cargo.lock` and `specs/001-rust-wallpaper-app/evidence/security-review.md`.
- [ ] T086 Review every requirement FR-001～FR-030 and acceptance criterion SC-001～SC-017 against evidence in `specs/001-rust-wallpaper-app/evidence/requirements-matrix.md`, explicitly distinguishing PASS, FAIL, SKIPPED and BLOCKED.
- [ ] T087 Produce a release-readiness decision (Go/No-Go) with all unresolved WindowsEnv, physical GPU, multi-monitor and timing limitations in `specs/001-rust-wallpaper-app/evidence/release-gate.md`; do not claim production completeness while a mandatory gate is missing.

**Final Checkpoint**: A complete Rust Windows wallpaper product with unmodified normal C# equations, owner-approved startup, one shared ImagePicker, YAML/tray/autostart, bounded on-demand load checks, VSync + adjustable default 120 FPS cap and traceable real test evidence.

---

## Requirement and Acceptance Traceability

Each source requirement and acceptance criterion is assigned to at least one implementing story phase; use the task ranges and final evidence tasks for concrete verification. Ranges below denote task IDs, **not** collapsed requirement IDs.

| Story / phase | Functional requirements explicitly covered | Acceptance criteria explicitly covered | Primary task IDs |
| --- | --- | --- | --- |
| US5 — image source | FR-004, FR-005, FR-021, FR-022, FR-023, FR-024, FR-025, FR-026 | SC-010, SC-011, SC-012, SC-013 | T020–T030 |
| US1 — original effects | FR-001, FR-002, FR-003, FR-012, FR-014, FR-015, FR-028, FR-030 | SC-001, SC-002, SC-005, SC-015, SC-017 | T031–T044 |
| US2 — opening scene | FR-017, FR-018, FR-019, FR-020 | SC-008, SC-009, SC-011 | T045–T054 |
| US3 — tray/config | FR-006, FR-007, FR-008, FR-013, FR-027, FR-029, FR-030 | SC-004, SC-014, SC-016, SC-017 | T055–T066 |
| US4 — load gate | FR-008, FR-009, FR-010, FR-011, FR-014, FR-015, FR-016, FR-020, FR-026, FR-027 | SC-003, SC-006, SC-007, SC-014 | T067–T077 |
| Cross-cutting release | FR-001 through FR-030 (final evidence matrix) | SC-001 through SC-017 (real test evidence) | T078–T087 |

**Gates**: T082 prevents a silent change to the original C# animation or weighted multi-monitor math; T080 and T087 prevent WindowsEnv-only WARP results from being misrepresented as a real-GPU/multi-monitor release validation.

---

## Dependencies & Execution Order

### Phase Dependencies

1. **Setup (T001–T007)** precedes all remaining tasks; within Setup, `[P]` tasks operate on different files after project paths exist.
2. **Foundational (T008–T019)** precedes every user story. Device/shell/window proofs must produce usable WindowsEnv evidence before production GPU/WorkerW adapters. `T010`/`T011` are independent probes and may run together.
3. **US5 (T020–T030, P1)** starts after foundation; global queue and empty-source API must exist before real slideshow and startup image claims.
4. **US1 (T031–T044, P1, MVP)** needs the completed US5 global image provider and verified foundational D3D11/WorkerW probes. Its pure math and geometry test/code can be developed in parallel with US5, but integrated slideshow acceptance waits for US5.
5. **US2 (T045–T054, P1)** requires the US5 queue and US1 rendering/monitor surfaces. It MUST use a separate startup animation implementation and cannot change US1's C# math.
6. **US3 (T055–T066, P2)** shares the `app_state` and Windows message window but its pure YAML/autostart tests and module implementation can proceed independently after foundation; final tray integration waits for US5/US1 UI state and US2 startup.
7. **US4 (T067–T077, P2)** requires the foundational sampler probes and image queue; deterministic scheduler development can overlap US3, but final wiring into tray and wallpaper depends on US1/US3.
8. **Polish (T078–T087)** follows the desired stories. Actual physical GPU/multi-monitor validation T080 may require a separate authorized machine; it is a release blocking gate, never silently substituted by WindowsEnv WARP.

### User Story Dependencies and Independent Acceptance

| Story | Priority | Needs | Independently testable success |
| --- | --- | --- | --- |
| US5 — image picker | P1 | Foundation | Every cycle covers the available paths once; empty remains quiescent and auto-rescans once on due time; 0/1/3/11/22 folders validated. |
| US1 — desktop & normal animation | P1 | Foundation + US5 | Legacy `cosh`/phase/geometry tests pass; WorkerW wallpaper renders in WindowsEnv; monitor area weighting and VSync+FPS cap proven. |
| US2 — new opening scene | P1 | Foundation + US5 + US1 | One main plus ten behind each monitor, staggered overlap, startup load-guard bypass and idle conclusion. |
| US3 — tray & YAML | P2 | Foundation; US1/US2 for final wiring | YAML rejects old dual thresholds, defaults 30/60/120, tray empty notification, autostart toggles HKCU immediately with read-back. |
| US4 — scheduled load guard | P2 | Foundation + US5; US1/US3 for final wiring | On deadline only, 59.9% plays, >=60% skips, next full interval retries, startup/idle/no-images samples remain zero. |

### Suggested Parallel Work

- **Foundation**: after T001–T002, T003 CI, T004 ignore rules, T005 golden source fixture, and T007 guide may proceed in parallel; T010 timing probe and T011 load probe are independent of each other after Windows WARP/toolchain availability.
- **US5**: T020/T021/T022 separate test suites can be prepared together, while T023 candidate model uses a different source file; queue logic is serialized inside T024–T026.
- **US1**: T031 math goldens, T032 monitor selection, T033 native render tests and T034 pacing tests are separate test files; T035 animation math and T036 monitor state may be developed independently after fixtures are ready.
- **US2**: T045 startup scene and T046 startup load-bypass tests are independent; startup math resides only in `rust/src/animation/startup.rs`.
- **US3**: T055 YAML tests, T056 reload tests and T057 registry mock tests are separate files and can run concurrently; T058 parser implementation and T063 registry adapter should avoid editing shared tray state simultaneously.
- **US4**: T067 load thresholds, T068 zero-sampling idle and T069 empty deadlines tests are independent suites; real GPU vs CPU adapter work must not assume WindowsEnv has physical GPU hardware.
- **Final**: T078 broad CI, T080 physical-GPU testing (on a separate machine), T081 perf analysis and T085 security review can be parallelized when the integrated build is ready. Never let two workers mutate the same file or user registry at once.

## Implementation Strategy

### MVP First (Foundation + US5 + US1)

1. Complete T001–T019; verify WARP/WorkerW platform viability and freeze C# golden vectors.
2. Complete T020–T030; certify one global non-repeating image queue and zero-image recovery.
3. Complete T031–T044; test Rust wallpaper render/animation behind icons with source-equivalent formulas and VSync+`max_fps=120`.
4. **STOP and validate** the independent US1 wallpaper MVP and record test evidence, not just compilation.

### Incremental Delivery

1. MVP → existing animated slideshow correctly reproduced.
2. US2 → new main-first, ten-background per-monitor introduction.
3. US3 → full tray controls, editable YAML and immediate Windows sign-in autostart toggle.
4. US4 → one quick 60%-default load decision only when the 30s-default slideshow timer is due.
5. Final phase → WindowsEnv integration, physical GPU/mixed-monitor acceptance, packaging and release decision.

### Release and Quality Rules

- A task is only checked `[x]` after its behavior/test evidence is produced; create small reviewable commits or PRs from `rust_build` without touching `master`/`directx_build`.
- Prefer TDD for logic and contracts: write a failing test, implement, then record the passing result. Hardware acceptance must record actual output and device identity (without private serial IDs).
- Rust `max_fps` is **a cap**, not promised achieved FPS. VSync remains enabled; a 144Hz/240Hz device with default cap 120 must not draw at native full rate; duration/math are elapsed-time based.
- New startup is explicitly exempt from the **GPU load gate**, not exempt from `max_fps` or picture queue rules.
- Do not install Windows Run startup entries until actual user interaction in the finished UI or an explicitly authorized disposable test.
- Any inability to verify real-GPU/multi-monitor fidelity blocks final release sign-off, while WindowsEnv-only results remain valuable and accurately reported.
