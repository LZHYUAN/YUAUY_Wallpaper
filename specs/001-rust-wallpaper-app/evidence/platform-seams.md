# T012 — platform-free injectable seams

**Status: PASS (interface-level, not Windows adapter acceptance)**

Implemented in `rust/src/platform.rs`:

- `MonotonicClock` and `SystemMonotonicClock`: monotonic elapsed `Duration`, independent of wall-clock/DST adjustments.
- `RandomIndex` and `SeededRandom`: explicitly seeded, reproducible `StdRng` with `NonZeroUsize` upper bound (no empty range).
- `FileCatalog`: caller-initiated candidate listing, with static redacted error strings; no implicit scan, watcher or polling.
- `UsageMetric`, `UsageReading`, `TotalUsageSampler`: explicitly requested one-shot GPU/CPU utilization with a caller-supplied time budget. Invalid/non-finite percentages become `Unknown`, never zero.
- No system tray, network, Windows API, directory watcher, worker thread or continuous frame loop is introduced.

**TDD evidence (2026-11-03):** Before implementation,
`cargo test --locked --test platform_seams --target x86_64-unknown-linux-gnu`
failed with Rust E0432 (`yuauy_wallpaper::platform` missing), the expected RED result. After adding the minimal implementation and importing the `rand 0.10` `RngExt` trait, all five named interface tests passed:

1. `monotonic_clock_is_injectable_without_wall_time`
2. `seeded_rng_has_reproducible_bounded_draws`
3. `injected_file_catalog_does_not_scan_implicitly`
4. `sampler_is_called_only_when_requested_with_bounded_budget`
5. `invalid_utilization_reading_must_never_mean_zero`

**Scope:** This does **not** implement the actual GPU PDH adapter, directory rescan/queue logic, slideshow scheduler, or renderer. Those are later Spec Kit tasks; any claim of runtime GPU accuracy or WindowsEnv acceptance remains pending.
