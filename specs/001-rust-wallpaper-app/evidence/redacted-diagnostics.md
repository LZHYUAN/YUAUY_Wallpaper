# T013 — Typed failures and redacted diagnostics

**Status: PASS (core-level; native Win32 integration still pending)**

- `rust/src/error.rs`: `AppError` distinguishes invalid YAML, unknown utilization, no images, unavailable desktop, filesystem error **kind**, and typed native operation/HRESULT. No variants accept arbitrary error strings, private photo paths or image bytes.
- `rust/src/diagnostics.rs`: an allowlisted `DiagnosticCode` and size-bounded `DiagnosticLog`. Old events are evicted at capacity, with static code-based summary only; cannot record filenames/YAML payloads.
- `rust/tests/error_diagnostics.rs`: 5 deterministic tests cover error distinctions, native HRESULT rendering, redaction, FIFO buffer bound and empty-state representation.

**TDD RED:** `cargo test --locked --test error_diagnostics --target x86_64-unknown-linux-gnu` initially failed with `E0432` (missing `yuauy_wallpaper::diagnostics` and `::error`). **GREEN:** 5/5 tests passed after implementing the minimal modules. Windows MSVC `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings` passed.

This is a diagnostic *contract*, not a running tray or logging service. T016/T018 will integrate it with the event loop, and private folder paths must never enter routine logs.
