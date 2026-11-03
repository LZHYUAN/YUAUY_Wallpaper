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

- **Decision**: Owner confirmed a single YAML-adjustable threshold, default 60%. A `UtilizationSampler` is called only when a normal transition is due, never during first startup, idle or empty-state directory scans. Use one quick, bounded best-effort total GPU utilization measurement (CPU total in WindowsEnv). If an API requires two quick counter snapshots for this single reading, collect them within this due-time operation; do not keep a sampling thread, historical average or long calibration loop. `>= threshold` skips this due transition; `< threshold` allows it. An invalid reading returns `Unknown`, never 0%.
- **Rationale**: User explicitly prefers simple, responsive detection rather than maximum precision or time-consuming calibration. Some Windows rate counters require a short pair of observations, so implementation must choose a lightweight bounded API path and measure its overhead in the Windows PoC; do not add background sampling.
- **Alternatives considered**: long-lived GPU monitor (rejected), permanently sampled PDH query (rejected), two-threshold 80/60 hysteresis (rejected), long-running calibration (rejected), measuring only process GPU workload (does not meet total-system GPU criterion).
- **Risk to prove**: select a sufficiently representative but cheap system-wide GPU counterset, avoid summing unrelated engines, bound sample latency, handle missing counters and multi-GPU without claiming laboratory accuracy. The proposed `Unknown` fallback (skip this due tick) needs Windows PoC verification.
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

- **Decision**: normal slideshow uses independent pure Rust functions matching the C# `cosh`-based curve, original phase thresholds, image placement, rotation, alpha and monitor-weighted selection. Owner specifies animation presentation targeting each display refresh rate via VSync, not the original approximate 40ms frame step. Compute animation `time` from a monotonic elapsed-time mapping of the reference `0.03` per ~40ms progression so the transition time scale and curve are preserved at 60/120/144Hz; do not increment `time` by a constant per presented frame. Startup remains an approved separate compositor with 1 main + 10 staggered background images per monitor. Rendering stops when animations end.
- **Rationale**: retains the validated effect while allowing precisely the authorized startup redesign. Fixing a bug in shell startup does not authorize changing the original math.
- **Alternatives considered**: ease-in/ease-out approximations (rejected), 40ms fixed frame rate (rejected for Rust implementation), a fixed 60FPS cap or perpetual render loop (rejected), frame-count-based animation progress at high refresh rates (would alter transition duration), replacing monitor weighting with round robin (rejected).
- **Risk to prove**: per-monitor VSync under WorkerW/DWM, multi-rate monitors and a shared virtual desktop (one swapchain spanning outputs cannot necessarily synchronize to both), GPU hardware vs WARP pacing, GDI+ versus D3D11 pixels, float precision/truncation and negative coordinates. Capture deterministic C# formula vectors, compare Rust math, and measure elapsed animation duration rather than trusting raw frame count.
- **Reference**: [reference-audit.md](reference-audit.md); `master:YUAUY_Wallpaper/Form1.cs` lines 109–225.

## 7. Resource budgets and provisional defaults

- **Owner-approved defaults (2026-11-03)**: normal interval `slideshow_interval_seconds: 30` (editable), single `load_guard.threshold_percent: 60` (editable) and load guard enabled in the example. A single simple due-time sample compares directly to 60%; there is no second resume percentage, hysteresis, historical averaging or background sampling. On measurement failure, the proposed `Unknown -> skip this due transition` fallback remains a technical test decision, not a user-approved change of threshold.
- **Rationale**: User approved 30 seconds and 60% as adjustable defaults, explicitly rejected the more complex dual-threshold and long calibration design, and wants rendering to target the display refresh rate only while animations play.
- **Alternatives considered**: 80% pause/60% resume (rejected), instant resume based on background monitoring (rejected), long calibration (rejected), fail-open on unknown load (may interrupt games).
- **Risk to prove**: one-shot measurement latency and availability, physical-GPU interpretation and VSync performance; these affect whether implementation meets approved defaults, not whether the default values are approved.

## 8. Recovery from no images

- **Decision**: user selected FR-026 B. When the image queue is empty, keep tray and normal timer alive. When a normal slideshow deadline arrives, rescan once. If still empty, schedule another complete normal interval; if images appear, clear tray status and resume display. Explicit tray reload may scan immediately. No auto-polling thread or catch-up burst.
- **Rationale**: timely recovery without the previously rejected independent monitors.
- **Alternatives considered**: only manual reload (user rejected); `ReadDirectoryChangesW` background notification (unnecessary for this explicit contract).
- **Design proposal to validate**: first successful recovery with no existing composed wallpaper starts the normal startup composition; recovery with a retained composed wallpaper should continue normal scheduling without destroying the retained image.

## 9. DXGI vertical synchronization and per-monitor timing

- **Decision**: during active animations, use appropriate D3D11/DXGI swapchain presentation with `SyncInterval=1` (for example `Present(1,0)`), targeting the current monitor's refresh rate. Use monotonic elapsed time for original C# `time` parameter, rather than `time += 0.03` once per presented frame. Stop producing frames when animations finish.
- **Rationale**: Microsoft documents `IDXGISwapChain::Present` synchronization intervals 1–4 as display vertical-blank based. A 144Hz display must not make a historically ~1.3s animation last less than a second just because more frames fit in that time.
- **Alternatives**: fixed 40ms per frame, fixed 60 FPS, busy-spin presenting every frame, frame-count-driven original formula advancement (all rejected). Exact software/WARP frame rate is a measured outcome, not guaranteed by a sync interval.
- **Risks**: DWM compositing, WorkerW hosting, VRR, mixed refresh-rate monitors, per-monitor swapchains and output assignment; the Microsoft documentation notes that a swapchain surface spanning multiple displays syncs with the display containing the largest part of its client area. Prefer separate monitor presentation surfaces where feasible; document fallback behavior.
- **Primary reference**: https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgiswapchain-present

## 10. Immediate current-user Windows sign-in autostart

- **Decision**: tray checkbox directly creates/deletes a single application-owned entry in `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` (e.g. value name `YUAUY_Wallpaper`) for the installed executable. On each click perform the Windows API write/delete immediately and read back to verify; the tray checkmark reflects actual registration, not desired state or YAML. No elevated rights and no separate background service.
- **Rationale**: Microsoft documents current-user `Run` as launching registered applications on user sign-in; `RunOnce` would not satisfy the persistent setting. It is an OS setting, not a wallpaper playback setting.
- **Alternatives considered**: HKLM global auto-start (unnecessary admin rights), scheduled task/service (excess), storing `autostart: true` in YAML and applying at next restart (violates immediate update).
- **Risks**: executable path quoting and Windows Run command length, uninstall cleanup, user externally disabling the Startup Apps registration, portable exe moved to another path, redundant/double instances. Query actual OS registration rather than assuming command success. Windows may delay automatic launch after sign-in; do not promise execution precisely at machine boot.
- **Primary references**: https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys ; https://support.microsoft.com/en-gb/windows/experience/startup-boot/configure-startup-applications-in-windows

## Phase 0 Conclusions

The requested product is implementable with testable seams. The technical choices above provide a design baseline, **not verified Windows API implementations**. The first implementation tasks must prove WARP, WorkerW and the GPU/CPU one-shot sampling path, with evidence, before the final rendering adapter is locked. No requirement has been downgraded to resolve an uncertain API behavior.
