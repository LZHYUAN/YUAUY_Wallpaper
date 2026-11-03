//! T008: bounded D3D11 device + swap-chain clear/Present feasibility probe.
//!
//! Creates one OWNED hidden window. It never touches WorkerW, wallpaper
//! registration, system tray, HKCU Run, or persistent desktop state.
//! A successful WARP run proves *this* device/Present path on *this* machine,
//! not hardware GPU, VSync, refresh rate, or desktop-icon layering.

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("SKIPPED: probe_d3d11 must be built and executed on Windows.");
    std::process::ExitCode::FAILURE
}

#[cfg(windows)]
fn main() -> std::process::ExitCode {
    let mut args = std::env::args().skip(1);
    let requested = args.next().unwrap_or_else(|| "--warp".to_owned());
    if args.next().is_some() || (requested != "--warp" && requested != "--hardware") {
        eprintln!("Usage: probe_d3d11 [--warp|--hardware]");
        return std::process::ExitCode::FAILURE;
    }

    match probe::run(&requested) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("FAIL: requested_driver={} error={e}", &requested[2..]);
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(windows)]
mod probe {
    use std::time::Instant;
    use windows::Win32::Foundation::{HMODULE, HWND};
    use windows::Win32::Graphics::Direct3D::{
        D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL,
    };
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDeviceAndSwapChain,
        ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Resource, ID3D11Texture2D,
    };
    use windows::Win32::Graphics::Dxgi::Common::{
        DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
    };
    use windows::Win32::Graphics::Dxgi::{
        DXGI_PRESENT, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD,
        DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIAdapter, IDXGISwapChain,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, WINDOW_EX_STYLE, WS_POPUP,
    };
    use windows::core::{BOOL, Interface, Result, w};

    /// This window is deliberately never shown. Drop is on its creating thread.
    struct HiddenWindow(HWND);
    impl Drop for HiddenWindow {
        fn drop(&mut self) {
            // SAFETY: HWND was created by this thread and is owned by this RAII guard.
            unsafe {
                let _ = DestroyWindow(self.0);
            }
        }
    }

    pub(super) fn run(requested: &str) -> Result<()> {
        let started = Instant::now();

        // SAFETY: STATIC is a predefined Win32 window class. HWND is not
        // attached to Explorer and has no WS_VISIBLE style or global lifetime.
        let window = HiddenWindow(unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("YUAUY WARP feasibility probe"),
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
        let swap_desc = DXGI_SWAP_CHAIN_DESC {
            BufferDesc: DXGI_MODE_DESC {
                Width: 64,
                Height: 64,
                RefreshRate: DXGI_RATIONAL {
                    Numerator: 60,
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
        let driver = if requested == "--warp" {
            D3D_DRIVER_TYPE_WARP
        } else {
            D3D_DRIVER_TYPE_HARDWARE
        };
        let mut chain: Option<IDXGISwapChain> = None;
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        let mut level = D3D_FEATURE_LEVEL::default();

        // SAFETY: pointers are valid for the duration of this call; all returned
        // COM interfaces are managed by windows-rs RAII. We supply no DXGI adapter
        // because D3D_DRIVER_TYPE_WARP/HARDWARE selects the driver internally.
        unsafe {
            D3D11CreateDeviceAndSwapChain(
                None::<&IDXGIAdapter>,
                driver,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&swap_desc),
                Some(&mut chain),
                Some(&mut device),
                Some(&mut level),
                Some(&mut context),
            )?;
        }

        let chain = chain.expect("D3D11 succeeded but returned no swap chain");
        let device = device.expect("D3D11 succeeded but returned no device");
        let context = context.expect("D3D11 succeeded but returned no context");

        // SAFETY: texture is the valid back buffer of the returned swap chain.
        // ClearRenderTargetView touches only this process-owned 64x64 texture.
        let texture: ID3D11Texture2D = unsafe { chain.GetBuffer(0)? };
        let resource: ID3D11Resource = texture.cast()?;
        let mut view: Option<ID3D11RenderTargetView> = None;
        unsafe { device.CreateRenderTargetView(&resource, None, Some(&mut view))? };
        let view = view.expect("CreateRenderTargetView succeeded without a view");
        unsafe { context.ClearRenderTargetView(&view, &[0.12, 0.25, 0.4, 1.0]) };

        // Present(0,0) isolates swap-chain availability. It is intentionally NOT
        // the VSync or FPS scheduling acceptance test (that is T010).
        let result = unsafe { chain.Present(0, DXGI_PRESENT(0)) };
        result.ok()?;
        let code = result.0 as u32;
        println!(
            "PASS driver_requested={} feature_level=0x{:x} present_hresult=0x{code:08x} occluded={} elapsed_ms={} window_visible=false",
            &requested[2..],
            level.0,
            code == 0x087a0001, // DXGI_STATUS_OCCLUDED means no visible presentation
            started.elapsed().as_millis(),
        );
        Ok(())
    }
}
