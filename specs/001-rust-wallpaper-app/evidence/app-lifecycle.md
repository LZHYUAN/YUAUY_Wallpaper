# T016 — single-threaded serialized app command reducer

**Status: PASS (core lifecycle, no Win32 event loop yet).**

- `rust/src/app_state.rs`: `AppCommand` derives Serde `Serialize`/`Deserialize` for all eight approved commands: `Start`, `Tick`, `Reload`, `Pause`, `Resume`, `Quit`, `ShellRestart`, `DisplayChanged`.
- `AppState` has `Uninitialized`, `Running`, `Paused`, `Stopped` phases and an `&mut self`-only reducer emitting intent-only `AppAction` values.
- Pause suppresses ordinary Tick handling but allows validated config reload; Resume requests a **full** slideshow re-arm; shell/display recovery retains the phase and does not select new images. Quit releases owned resources once and makes all future commands no-ops.
- No Windows autostart registry changes, image selection, GPU sampling, filesystem watching, animation formula computation, or message processing occurs in this module; those are later tasks.
- The caller will confine reducer ownership to the T018 Win32 message thread, so no background worker or event-reordering facility is added here.

## TDD evidence (2026-11-04)

RED: `cargo test --locked --test app_lifecycle --target x86_64-unknown-linux-gnu` failed E0432 (module absent).

GREEN: 5/5 tests passed for Serde command round-trip, single-shot startup, pause/reload/resume semantics, Explorer/display recovery and idempotent shutdown. `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings` passed.

**Next:** T017 deterministic fixtures, T018 native message loop and actual integration of the emitted actions. Current state reducer does not execute shell, rendering or slideshow operations.
