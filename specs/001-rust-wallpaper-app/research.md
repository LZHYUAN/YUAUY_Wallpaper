# Phase 0 Research: Windows/Rust Wallpaper Implementation

**Scope**: Decision log for [plan.md](plan.md). This is a technical proposal with cited primary references; devices, tools and runtime integration are **not yet verified**.

## 1. Direct3D backend: Direct3D 11 with WARP option

- **Decision**: target Direct3D11. Prefer the available hardware driver on real machines; support the D3D11 WARP software rasterizer in GPU-less WindowsEnv for graphics integration and CPU-based tests. Never start with GDI+ as a second production renderer.
- **Rationale**: D3D11 APIs support `D3D_DRIVER_TYPE_WARP` for software rasterization, as documented by Microsoft. It avoids changing the API or math between WindowsEnv and hardware tests, but not all performance attributes are equivalent.
- **Alternatives considered**: D3D12 (greater complexity, not required by the C++ exploratory branch), C# GDI+ (violates Rust/DX goal), Vulkan (not requested).
- **Risk to prove**: WARP device and swap-chain creation in actual WindowsEnv 25H2; WorkerW/backbuffer presentation; driver loss and Explorer restart; performance with ten concurrent startup transitions.
- **References**: https://learn.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-devices-create-warp ; https://learn.microsoft.com/en-us/windows/win32/api/d3dcommon/ne-d3dcommon-d3d_driver_type

## 2. Windows API integration

- **Decision**: use Rust `windows` crate for Win32, COM, D3D11, DXGI and native tray integration. Render to a desktop-compatible window under an isolated shell adapter and make image geometry/DPI observable in tests.
- **Rationale**: Native Windows API usage permits precise management of window lifetime and shell notifications without an additional GUI application runtime.
- **Alternatives considered**: `winit`/GUI frameworks for event loop; tray wrapper crates. They may be useful after a PoC but cannot substitute for verified WorkerW behavior.
- **Risk to prove**: `WorkerW`/`Progman` integration in Windows 11 24H2/25H2 is not treated as a documented stability guarantee; show failure behavior and recovery. First test a small hidden message window and notification icon, then a D3D11 test window, then desktop layering.
- **References**: https://learn.microsoft.com/en-us/windows/win32/shell/notification-area ; https://learn.microsoft.com/zh-tw/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw

## 3. Utilization reading without idle polling

- **Decision**: a `UtilizationSampler` is called only by the scheduler once a normal transition becomes due, never during first startup, idle or empty-state directory scans. It may internally collect the bounded samples required to compute one rate **at due time**; it does not maintain its own timer thread. Use total GPU utilization for the production backend; CPU total usage is a WindowsEnv test fallback. If the metric is unavailable/invalid, return `Unknown` instead of 0%.
- **Rationale**: Microsoft explains that some performance counter values need two time-separated samples. A single instantaneous query may not represent meaningful utilization.
- **Alternatives considered**: long-lived GPU monitor (rejected by owner), permanently sampled PDH query (rejected), measuring only process GPU workload (does not meet total-system GPU criterion), read Process CPU usage (wrong system-wide scope).
- **Risk to prove**: identify correct GPU counterset per adapter, define adapter/engine aggregation without incorrectly summing percentages, determine maximum allowed sampling latency, test Counter ACL/availability and multi-GPU behavior. The `Unknown` fallback policy in plan is provisional pending user acceptance.
- **References**: https://learn.microsoft.com/en-us/windows/win32/perfctrs/collecting-performance-data ; https://learn.microsoft.com/en-us/windows/win32/direct3dtools/pix/articles/timing-captures/pix-timing-captures

## 4. YAML library and contract

- **Decision**: type-safe settings via `serde` and maintained `yaml_serde` (YAML organization's fork) rather than unmaintained `serde_yaml` or `serde_yml`; validate all values before applying to runtime, and swap the active settings only if the entire reload succeeds.
- **Rationale**: the original `serde_yaml` warns it is no longer maintained, and the official YAML organization's maintained fork advertises Serde-compatible parsing. The `serde_yml` project is also deprecated.
- **Alternatives considered**: `serde_yaml` (unmaintained), `serde_yml` (deprecated), generic regex-based YAML (unsafe/incomplete).
- **Risk to prove**: version compatibility with Rust toolchain, key validation/unknown-key policy, Unicode and Windows path handling; package security audits before release.
- **References**: https://github.com/yaml/yaml-serde ; https://docs.rs/serde_yaml/latest/serde_yaml/ ; https://docs.rs/serde_yml/latest/serde_yml/

## 5. Image decoding and shared picker semantics

- **Decision**: separate a pure `ImageQueue` with injected filesystem and RNG from the Windows image decoder. Scan valid candidate files on initialization, reshuffle and rescan only after current cycle exhaustion, an explicit YAML reload, or a due-time empty-state rescan. Use random permutation (shuffle), not random *selection with replacement*. All startup and slideshow requests go through this single queue.
- **Rationale**: the C# `Random_NotRepeating` implementation uses ordered, deduplicated path visits per cycle but refreshes on changed file count mid-cycle, and `Reset()` just resets the index. The owner requires more predictable cycle-boundary refresh. A one-image folder must still fill each monitor's startup composition by using consecutive cycles.
- **Alternatives considered**: one picker per monitor (could repeat within a global cycle), reshuffle on each call (repeats before exhaustion), directory watcher or timer (not requested).
- **Risk to prove**: corrupted files, removal mid-cycle, Unicode paths, symlinks/aliases, and concurrency must never produce duplicate path claims within one cycle; no folder scans in idle except due-time empty-state checks.
- **Reference**: `master:YUAUY_Wallpaper/Image/ImagePicker.cs` lines 44–112; [reference-audit.md](reference-audit.md).

## 6. Timing, animation parity and startup choreography

- **Decision**: normal slideshow uses independent pure Rust functions matching the C# `cosh`-based curve, original phase thresholds, image placement, rotation, alpha and monitor-weighted selection. The new owner-approved startup is a separate compositor with 1 main image + 10 stochastically staggered slide-ins per monitor. Each monitor has static composition at end of startup. The normal timer begins after completion and does not run per-frame outside animations.
- **Rationale**: retains the validated effect while allowing precisely the authorized startup redesign. Fixing a bug in shell startup does not authorize changing the original math.
- **Alternatives considered**: ease-in/ease-out approximations (rejected), rebuilding frames in a perpetual 60 FPS loop (rejected by idle requirement), replacing monitor weighting with round robin (rejected).
- **Risk to prove**: GDI+ versus D3D11 pixels, floating point precision and truncation, virtual screen negative coordinates, per-monitor DPI and headless WARP output. Capture deterministic C# formula vectors (by reading the equations, not by claiming the old app runs today) and compare Rust outputs.
- **Reference**: [reference-audit.md](reference-audit.md); `master:YUAUY_Wallpaper/Form1.cs` lines 109–225.

## 7. Resource budgets and provisional defaults

- **Decision proposal** (needs owner review before final release): use legacy release-mode normal interval 30 seconds; pause at `80%`, resume at `60%` (hysteresis); `load_guard_enabled: true`. Do not add a configuration key for a periodic sampler. When GPU/CPU measurement fails, propose `Unknown -> skip this due transition`, retry on next full interval rather than triggering unbounded immediate attempts.
- **Rationale**: 30 seconds comes from C# release mode; 80/60 was a previously offered proposal, **not owner-confirmed**. Favor predictable resource usage over exact resume latency.
- **Alternatives considered**: instant resume based on background monitoring (rejected), auto-detect fastest available interval (unpredictable), fail-open on unknown load (may interrupt games).
- **Risk to prove**: on-demand PDH latency, diagnostics, visual smoothness, and safe upper bounds for decoded GPU resources. Budget values should be validated in a physical GPU test before approving defaults.

## 8. Recovery from no images

- **Decision**: user selected FR-026 B. When the image queue is empty, keep tray and normal timer alive. When a normal slideshow deadline arrives, rescan once. If still empty, schedule another complete normal interval; if images appear, clear tray status and resume display. Explicit tray reload may scan immediately. No auto-polling thread or catch-up burst.
- **Rationale**: timely recovery without the previously rejected independent monitors.
- **Alternatives considered**: only manual reload (user rejected); `ReadDirectoryChangesW` background notification (unnecessary for this explicit contract).
- **Design proposal to validate**: first successful recovery with no existing composed wallpaper starts the normal startup composition; recovery with a retained composed wallpaper should continue normal scheduling without destroying the retained image.

## Phase 0 Conclusions

The requested product is implementable with testable seams. The technical choices above provide a design baseline, **not verified Windows API implementations**. The first implementation tasks must prove WARP, WorkerW and the GPU/CPU one-shot sampling path, with evidence, before the final rendering adapter is locked. No requirement has been downgraded to resolve an uncertain API behavior.
