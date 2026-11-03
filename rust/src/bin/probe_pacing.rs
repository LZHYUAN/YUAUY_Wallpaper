//! T010: bounded, *diagnostic* DXGI Present(1) timing probe.
//! A hidden window cannot prove actual VSync-to-display synchronization.
//! It intentionally does not alter a monitor mode, Explorer, the registry,
//! user desktop icons, or the preserved C# transition equations.

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("SKIPPED: probe_pacing only runs on Windows.");
    std::process::ExitCode::from(2)
}

#[cfg(windows)]
fn main() -> std::process::ExitCode {
    let mut frames = 12_u32;
    let mut cap = 120_u32;
    let mut driver = "warp";
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--frames" => {
                frames = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            }
            "--max-fps" => {
                cap = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            }
            "--driver" => {
                let d = args.next().unwrap_or_default();
                if d != "warp" && d != "hardware" {
                    eprintln!("FAIL: --driver expects warp|hardware");
                    return std::process::ExitCode::FAILURE;
                }
                driver = if d == "warp" { "warp" } else { "hardware" };
            }
            _ => {
                eprintln!("FAIL: unsupported argument");
                return std::process::ExitCode::FAILURE;
            }
        }
    }
    if !(2..=120).contains(&frames) || !(1..=1000).contains(&cap) {
        eprintln!("FAIL: --frames 2..120 and --max-fps 1..1000 required");
        return std::process::ExitCode::FAILURE;
    }
    match probe::run(frames, cap, driver) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("SKIPPED/FAIL: pacing probe {error}");
            std::process::ExitCode::from(2)
        }
    }
}

#[cfg(windows)]
mod probe {
    use std::time::{Duration, Instant};
    use windows::Win32::Foundation::{HMODULE, HWND};
    use windows::Win32::Graphics::Direct3D::{
        D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL,
    };
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDeviceAndSwapChain,
        ID3D11Device, ID3D11DeviceContext,
    };
    use windows::Win32::Graphics::Dxgi::Common::{
        DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
    };
    use windows::Win32::Graphics::Dxgi::{
        DXGI_PRESENT, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD,
        DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIAdapter, IDXGISwapChain,
    };
    use windows::Win32::Graphics::Gdi::{
        DEVMODEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICEW, ENUM_CURRENT_SETTINGS,
        EnumDisplayDevicesW, EnumDisplaySettingsW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, WINDOW_EX_STYLE, WS_POPUP,
    };
    use windows::core::{BOOL, PCWSTR, Result, w};
    use yuauy_wallpaper::pacing::FramePacer;

    struct HiddenWindow(HWND);
    impl Drop for HiddenWindow {
        fn drop(&mut self) {
            // SAFETY: owned HWND created by this same thread.
            unsafe {
                let _ = DestroyWindow(self.0);
            }
        }
    }

    fn output_modes() -> Vec<(String, u32)> {
        let mut result = Vec::new();
        // Query attached Win32 display devices only; no display mode changes.
        for n in 0..16 {
            let mut dev = DISPLAY_DEVICEW {
                cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                ..Default::default()
            };
            // SAFETY: properly sized output buffer, null means enumerate adapters.
            let ok =
                unsafe { EnumDisplayDevicesW(PCWSTR(std::ptr::null()), n, &mut dev, 0) }.as_bool();
            if !ok {
                break;
            }
            if !dev.StateFlags.contains(DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) {
                continue;
            }
            let mut mode = DEVMODEW {
                dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                ..Default::default()
            };
            // SAFETY: buffer remains valid, device name is a fixed NUL-terminated
            // WCHAR array returned by Win32; do not record private device strings.
            let valid = unsafe {
                EnumDisplaySettingsW(
                    PCWSTR(dev.DeviceName.as_ptr()),
                    ENUM_CURRENT_SETTINGS,
                    &mut mode,
                )
            }
            .as_bool();
            if valid && (1..=2000).contains(&mode.dmDisplayFrequency) {
                result.push((format!("display{n}"), mode.dmDisplayFrequency));
            }
        }
        result
    }

    pub(super) fn run(frames: u32, cap: u32, driver_name: &str) -> Result<()> {
        let modes = output_modes();
        if modes.is_empty() {
            println!("SKIPPED display_modes=none; no refresh-frequency assumption made");
            return Err(windows::core::Error::from_thread());
        }
        let refresh = f64::from(modes[0].1);
        let mut pacer =
            FramePacer::new(refresh, cap).map_err(|_| windows::core::Error::from_thread())?;
        for (device, hz) in &modes {
            println!("AVAILABLE output={device} refresh_hz={hz} max_fps={cap}");
        }
        // These are an acceptance test matrix, *not* fabricated measurements.
        for hz in [60, 120, 144, 240] {
            for ceiling in [60, 120, 240] {
                if modes.iter().any(|(_, found)| *found == hz) {
                    println!(
                        "AVAILABLE target_hz={hz} cap={ceiling} scheduled_target_fps={}",
                        hz.min(ceiling)
                    );
                } else {
                    println!(
                        "UNSUPPORTED target_hz={hz} cap={ceiling} reason=display_mode_unavailable"
                    );
                }
            }
        }
        let window = HiddenWindow(unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("YUAUY VSync diagnostic"),
                WS_POPUP,
                0,
                0,
                64,
                64,
                None,
                None,
                None,
                None,
            )?
        });
        let desc = DXGI_SWAP_CHAIN_DESC {
            BufferDesc: DXGI_MODE_DESC {
                Width: 64,
                Height: 64,
                RefreshRate: DXGI_RATIONAL {
                    Numerator: modes[0].1,
                    Denominator: 1,
                },
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                ..Default::default()
            },
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 1,
            OutputWindow: window.0,
            Windowed: BOOL(1),
            SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
            Flags: 0,
        };
        let driver = if driver_name == "warp" {
            D3D_DRIVER_TYPE_WARP
        } else {
            D3D_DRIVER_TYPE_HARDWARE
        };
        let mut chain: Option<IDXGISwapChain> = None;
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        let mut level = D3D_FEATURE_LEVEL::default();
        // SAFETY: all pointers are valid output parameters, owned COM pointers
        // remain scoped to this diagnostic. No desktop-host attach.
        unsafe {
            D3D11CreateDeviceAndSwapChain(
                None::<&IDXGIAdapter>,
                driver,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&desc),
                Some(&mut chain),
                Some(&mut device),
                Some(&mut level),
                Some(&mut context),
            )?
        }
        let chain = chain.ok_or_else(windows::core::Error::from_thread)?;
        let _device = device.ok_or_else(windows::core::Error::from_thread)?;
        let _context = context.ok_or_else(windows::core::Error::from_thread)?;
        let started = Instant::now();
        println!(
            "BEGIN requested_driver={driver_name} feature_level=0x{:x} requested_vsync_interval=1 fps_target={} window_visible=false",
            level.0,
            pacer.target_fps()
        );
        for frame in 0..frames {
            if started.elapsed() > Duration::from_secs(8) {
                println!("SKIPPED reason=runtime_budget_exceeded samples={frame}");
                break;
            }
            let now = started.elapsed();
            if let Some(deadline) = pacer.next_deadline()
                && now < deadline
            {
                std::thread::sleep(deadline - now);
            }
            let before = started.elapsed();
            // SyncInterval=1 requests vblank synchronization. Hidden windows
            // or WARP may be *unthrottled*, so this is not proof of VSync.
            let hr = unsafe { chain.Present(1, DXGI_PRESENT(0)) };
            hr.ok()?;
            let after = started.elapsed();
            pacer.record_submit(after);
            println!(
                "FRAME index={frame} submit_ms={:.3} present_return_ms={:.3} hresult=0x{:08x} request_syncinterval=1",
                before.as_secs_f64() * 1e3,
                after.as_secs_f64() * 1e3,
                hr.0 as u32
            );
        }
        println!(
            "VERDICT diagnostic_only=true true_vsync=UNVERIFIED desktop_surface=UNVERIFIED elapsed_ms={:.3}",
            started.elapsed().as_secs_f64() * 1e3
        );
        Ok(())
    }
}
