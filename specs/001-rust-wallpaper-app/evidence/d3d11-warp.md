# T008 — D3D11 device, WARP and clear/present probe

**Status: code implemented, WindowsEnv runtime probe NOT RUN. T008 remains unchecked.**

## Scope

`rust/src/bin/probe_d3d11.rs` builds a D3D11 device plus DXGI swap chain for its own **hidden 64×64 STATIC HWND**. It creates a render target for the back buffer, clears the resource and submits a single `Present(0,0)`; it does not attempt any WorkerW attachment. `--warp` explicitly requests `D3D_DRIVER_TYPE_WARP`; `--hardware` uses `D3D_DRIVER_TYPE_HARDWARE`. Each output includes the requested driver type, D3D feature level, Present HRESULT (and an occlusion distinction), and elapsed milliseconds. COM handles and the hidden HWND are owned by process-scoped RAII; registry, wallpapers, tray, and user settings remain untouched.

A hidden-window Present may return `DXGI_STATUS_OCCLUDED`. Such a return must **not** count as on-screen rendering or VSync success. This probe is strictly feasibility of device/swapchain/clear/present; T009 covers icon layering and T010 VSync timing.

## Verification matrix

| Check | Status | Evidence |
| --- | --- | --- |
| MSVC x64 Cargo compile check of `probe_d3d11.rs` | **PASS** | `cargo check --locked --all-targets --target x86_64-pc-windows-msvc` on Hermes Linux (compiles Windows APIs but cannot execute). |
| Linux/Cross-target format, Clippy, tests | **PASS** | `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings`, and Linux `cargo test --locked --target x86_64-unknown-linux-gnu` (4 passing dependency tests, probe binary compiles but is not run on Linux). |
| WARP runtime on GitHub Actions Windows runner | **PASS** | [Workflow run 41620251582](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41620251582), job `125863855827`, source commit `f984b2a`: full WARP clear/present stage passed. This is **not** WindowsEnv evidence. |
| WARP device and Present on WindowsEnv 25H2 | **SKIPPED** | WindowsEnv interactive CUA driver is healthy, but SSH host alias cannot resolve and minimized background PowerShell launch was refused; no foreground-stealing workaround used. |
| Hardware D3D11 device on real GPU | **SKIPPED** | No physical GPU integration run. |
| WorkerW desktop icon layering | **SKIPPED** | Separate T009 probe not implemented. |

Once a verified non-disruptive WindowsEnv terminal path is available, build/run `cargo run --locked --release --bin probe_d3d11 -- --warp` there, record full stdout/stderr/exit code, device details, cleanup and timing, then update this report before marking T008 completed. Always distinguish hardware GPU from WARP; WARP PASS alone never certifies hardware compatibility.

## GitHub Actions WARP output (2026-11-03)

Commit `f984b2a0101495c5007ce761dd35a59f650055e9` triggered [Rust Windows run 41620251582](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41620251582). Job `125863855827` reports **success** for pinned toolchain, lockfile, format, Clippy, MSVC release build, the optional WARP clear/present smoke, and deterministic tests. The downloaded job log contains the exact probe line:

```text
PASS driver_requested=warp feature_level=0xb000 present_hresult=0x00000000 occluded=false elapsed_ms=8 window_visible=false
```

Interpretation: Direct3D 11 WARP device selection, swap-chain creation, back-buffer clear and a non-failing `Present(0,0)` were demonstrated on the hosted Windows CI runner. `0xb000` is feature level 11.0. The 8 ms wall-time is **one CI-run observation**, not a performance target. `occluded=false` means the returned HRESULT was not `DXGI_STATUS_OCCLUDED`, **not** that the deliberately hidden window became visible or drew behind the desktop icons. T008's separate WindowsEnv 25H2 runtime condition and T009/T010 remain **SKIPPED**.
