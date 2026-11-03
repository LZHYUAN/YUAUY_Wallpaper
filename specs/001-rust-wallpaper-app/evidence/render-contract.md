# T015 — object-safe D3D11 rendering contract and native resource ownership

**Status: PASS (interface and Windows compile checks; not a visible wallpaper renderer).**

- `rust/src/render/mod.rs` defines `WallpaperRenderer` using object-safe, `&mut self` methods for backend selection, per-monitor `attach`, `present`, and `detach`. No generic methods or implicit global image picker: all scenes pass through a per-monitor `SurfaceKey`.
- `RenderBackend::{Hardware,Warp}` is explicit. `FramePresentation` carries monotonic elapsed transition time and scene version, *not* a frame counter, easing coefficient, image path or changed C# formula. No GDI+ alternate production renderer exists.
- On Windows, `OwnedWin32Window` is constructed **only from an exclusively owned HWND created on the same thread**, never an Explorer WorkerW/Progman HWND. It cannot be moved to other threads (`Rc` phantom marker), and destroys only its owned window on drop.
- `OwnedD3d11Surface` stores COM render-target view, swap chain, device context, device and the owned HWND. Rust field order releases GPU COM objects before destroying the owned window. Actual COM resource construction and WorkerW hosting belong to later rendering tasks.

## TDD and verification (2026-11-04)

RED: `cargo test --locked --test render_contract --target x86_64-unknown-linux-gnu` initially failed E0432 (module `render` missing).

GREEN: the 3 tests all passed:

1. Object-safe `dyn WallpaperRenderer` with explicit WARP backend and frame request.
2. Independent monitor-scoped surface keys and detach/present lifetimes.
3. Scene requests use real elapsed time instead of display-dependent frame counts.

`cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings` passes with Windows native HWND/D3D11/DXGI ownership types enabled.

**Not yet proven:** a real WorkerW attachment, multi-monitor swap chain, FPS and GPU performance. These remain separate tasks and system acceptance gates.
