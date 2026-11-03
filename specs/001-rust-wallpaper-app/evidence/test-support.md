# T017 — deterministic test support library for five user stories

**Status: PASS (test fixtures only).**

Added `rust/tests/support/mod.rs`, importable as `mod support;` from individual integration test files:

- `VirtualClock`: a monotonic `Duration` advanced only by test calls (never sleeps).
- `SequenceUsageSampler`: deterministic `GpuTotal`/`CpuTotal` readings consumed only on explicit calls. Logs metric and budget; exhausted values return `Unknown`, not idle/zero.
- `TempFolderCatalog`: owns a disposable temporary folder, writes synthetic test bytes with safe names, and increments scan count only for explicit `list_paths`; does not access real user photo folders. Invalid directories return an empty collection.
- `two_monitor_layout`: a deterministic negative-x left display and primary display with independent scene/surface state.
- `RecordingRenderer`: object-safe `WallpaperRenderer` implementation tracking `attach`, `present`, `detach` without any Win32/D3D11 calls, GPU allocation or drawing loop.

**TDD RED:** `cargo test --locked --test support_contract --target x86_64-unknown-linux-gnu` failed E0583 (missing `tests/support/mod.rs`). **GREEN:** all 5 tests passed, covering virtual time, on-demand loads, scan counts, multi-monitor isolation and explicit frame submissions.

**Build verification:** Windows MSVC cross-target Clippy `--all-targets -- -D warnings` passed. These helpers provide test seams only; they do not implement global image queue, startup choreography, YAML settings, native rendering or actual WindowsEnv acceptance.
