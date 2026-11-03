//! Explicit 10-second maximum Direct3D11 WARP *desktop* composition acceptance
//! probe for Windows 11 24H2+ raised desktops. NOT a wallpaper product.
//!
//! This program creates only its own small, layered Progman child. It leaves
//! Explorer's window styles, parent pointers, and icon-view z-order untouched.
//! Default: read-only. Opt-in --attach-modern --duration-ms 9000 is required.
//! Writes a redacted result log to %TEMP%/YUAUY-Win25-Probes/desktop_probe_result.txt.

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("SKIPPED: desktop Direct3D11 visual probe is Windows-only");
    std::process::ExitCode::from(2)
}

#[cfg(windows)]
#[allow(clippy::large_stack_arrays)]
fn main() -> std::process::ExitCode {
    use std::fs;
    use yuauy_wallpaper::workerw_probe_options::{Mode, parse_args};
    let mode = match parse_args(std::env::args().skip(1)) {
        Ok(mode @ (Mode::Inspect | Mode::AttachModern { .. })) => mode,
        _ => {
            eprintln!("FAIL: use --inspect or --attach-modern [--duration-ms 1..10000]");
            return std::process::ExitCode::FAILURE;
        }
    };

    let folder = std::env::temp_dir().join("YUAUY-Win25-Probes");
    let _ = fs::create_dir_all(&folder);
    let out = folder.join("desktop_probe_result.txt");
    let _ = fs::write(&out, "RUNNING desktop_d3d11_probe\n");
    let (code, message) = match probe::run(mode) {
        Ok(msg) => (0u8, format!("PASS {msg}\n")),
        Err(probe::Outcome::Skip(msg)) => (2u8, format!("SKIPPED {msg}\n")),
        Err(probe::Outcome::Fail(msg)) => (1u8, format!("FAIL {msg}\n")),
    };
    let _ = fs::write(&out, &message);
    println!("{message}");
    std::process::ExitCode::from(code)
}

#[cfg(windows)]
mod probe {
    use std::{
        thread,
        time::{Duration, Instant},
    };
    use windows::{
        Win32::{
            Foundation::{COLORREF, HMODULE, HWND},
            Graphics::{
                Direct3D::{D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL},
                Direct3D11::{
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
                    D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext,
                    ID3D11RenderTargetView, ID3D11Resource, ID3D11Texture2D,
                },
                Dxgi::Common::{
                    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
                },
                Dxgi::{
                    DXGI_PRESENT, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD,
                    DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIAdapter, IDXGISwapChain,
                },
            },
            UI::WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, DispatchMessageW, FindWindowExW, FindWindowW,
                GW_HWNDNEXT, GWL_EXSTYLE, GetParent, GetWindow, GetWindowLongW, GetWindowRect,
                GetWindowThreadProcessId, IsWindow, IsWindowVisible, LWA_ALPHA, MSG, PM_REMOVE,
                PeekMessageW, SWP_NOACTIVATE, SWP_SHOWWINDOW, SetLayeredWindowAttributes,
                SetWindowPos, TranslateMessage, WINDOW_EX_STYLE, WS_CHILD, WS_CLIPSIBLINGS,
                WS_DISABLED, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
                WS_EX_TRANSPARENT,
            },
        },
        core::{BOOL, Interface, w},
    };
    use yuauy_wallpaper::workerw_probe_options::{
        AttachTarget, DesktopTopology, Mode, select_target_for_topology,
    };

    pub(super) enum Outcome {
        Skip(String),
        Fail(String),
    }
    fn failure(op: &str, error: windows::core::Error) -> Outcome {
        Outcome::Fail(format!("{op} HRESULT=0x{:08x}", error.code().0 as u32))
    }

    struct OwnedChild(HWND);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            // SAFETY: only our own HWND is destroyed on its creating thread.
            unsafe {
                let _ = DestroyWindow(self.0);
            }
        }
    }

    fn pid(hwnd: HWND) -> u32 {
        let mut out = 0;
        // SAFETY: read-only owner query on a valid HWND.
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut out)) };
        out
    }

    pub(super) fn run(mode: Mode) -> Result<String, Outcome> {
        let progman = unsafe { FindWindowW(w!("Progman"), None) }
            .map_err(|_| Outcome::Skip("Progman_absent".to_owned()))?;
        let defview = unsafe { FindWindowExW(Some(progman), None, w!("SHELLDLL_DefView"), None) }
            .map_err(|_| Outcome::Skip("DefView_not_under_Progman".to_owned()))?;
        let worker_child = unsafe { FindWindowExW(Some(progman), None, w!("WorkerW"), None) }.ok();
        let parent = unsafe { GetParent(defview) }.map_err(|e| failure("GetParent", e))?;
        let same_icon_owner =
            pid(progman) != 0 && pid(progman) == pid(defview) && parent == progman;
        let raised = (unsafe { GetWindowLongW(progman, GWL_EXSTYLE) } as u32
            & WS_EX_NOREDIRECTIONBITMAP.0)
            != 0;
        let topology = DesktopTopology {
            has_progman: true,
            has_defview_child: true,
            has_workerw_child: worker_child.is_some(),
            raised_desktop: raised,
            same_icon_owner,
        };
        let selected =
            select_target_for_topology(mode, topology).map_err(|e| Outcome::Skip(e.to_owned()))?;
        let mut rect = Default::default();
        unsafe { GetWindowRect(progman, &mut rect) }.map_err(|e| failure("GetWindowRect", e))?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if mode == Mode::Inspect {
            return Ok(format!(
                "inspect=true raised={raised} defview_child=true worker_child={}                 same_owner={same_icon_owner} progman_size={width}x{height}                 d3d11_present=NOT_RUN behind_icons=UNVERIFIED",
                worker_child.is_some(),
            ));
        }
        if selected != Some(AttachTarget::ProgmanLayered) {
            return Err(Outcome::Skip("modern_raised_desktop_required".to_owned()));
        }
        let duration_ms = match mode {
            Mode::AttachModern { duration_ms } => duration_ms,
            _ => return Err(Outcome::Skip("explicit_modern_opt_in_required".to_owned())),
        };
        if width < 240 || height < 220 {
            return Err(Outcome::Skip("desktop_too_small_for_probe".to_owned()));
        }

        let size_x = 220;
        let size_y = 190;
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(WS_EX_NOACTIVATE.0 | WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0),
                w!("STATIC"),
                w!("YUAUY D3D11 Desktop WARP Test"),
                WS_CHILD | WS_CLIPSIBLINGS | WS_DISABLED,
                12,
                12,
                size_x,
                size_y,
                Some(progman),
                None,
                None,
                None,
            )
        }
        .map_err(|e| failure("CreateWindowExW", e))?;
        let owned = OwnedChild(hwnd);
        if unsafe { GetParent(hwnd) }.map_err(|e| failure("GetParent_child", e))? != progman {
            return Err(Outcome::Fail("probe_child_parent_mismatch".to_owned()));
        }
        // SAFETY: only our own child changes alpha/position. Explorer-owned
        // DefView and Progman are neither reparented nor restyled.
        unsafe {
            SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)
                .map_err(|e| failure("LayeredAttributes", e))?;
            SetWindowPos(
                hwnd,
                Some(defview),
                12,
                12,
                size_x,
                size_y,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )
            .map_err(|e| failure("SetWindowPos", e))?;
        }
        let visible = unsafe { IsWindowVisible(hwnd).as_bool() };
        let below_icon_view = unsafe { GetWindow(defview, GW_HWNDNEXT).ok() == Some(hwnd) };

        let swap_desc = DXGI_SWAP_CHAIN_DESC {
            BufferDesc: DXGI_MODE_DESC {
                Width: size_x as u32,
                Height: size_y as u32,
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
            OutputWindow: hwnd,
            Windowed: BOOL(1),
            SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
            Flags: 0,
        };
        let mut chain: Option<IDXGISwapChain> = None;
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        let mut level = D3D_FEATURE_LEVEL::default();
        unsafe {
            D3D11CreateDeviceAndSwapChain(
                None::<&IDXGIAdapter>,
                D3D_DRIVER_TYPE_WARP,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&swap_desc),
                Some(&mut chain),
                Some(&mut device),
                Some(&mut level),
                Some(&mut context),
            )
        }
        .map_err(|e| failure("D3D11CreateDeviceAndSwapChain", e))?;
        let chain = chain.ok_or_else(|| Outcome::Fail("no_swap_chain".to_owned()))?;
        let device = device.ok_or_else(|| Outcome::Fail("no_device".to_owned()))?;
        let context = context.ok_or_else(|| Outcome::Fail("no_context".to_owned()))?;
        let texture: ID3D11Texture2D =
            unsafe { chain.GetBuffer(0) }.map_err(|e| failure("GetBuffer", e))?;
        let resource: ID3D11Resource = texture.cast().map_err(|e| failure("TextureCast", e))?;
        let mut view: Option<ID3D11RenderTargetView> = None;
        unsafe { device.CreateRenderTargetView(&resource, None, Some(&mut view)) }
            .map_err(|e| failure("CreateRenderTargetView", e))?;
        let view = view.ok_or_else(|| Outcome::Fail("no_rtv".to_owned()))?;
        let started = Instant::now();
        let mut presents = 0u32;
        let mut last_status = 0u32;
        while started.elapsed() < Duration::from_millis(duration_ms) {
            // Bright lime D3D11 clear; 220x190 upper-left overlaps desktop
            // icon positions to assess icon occlusion visually via CUA.
            unsafe { context.ClearRenderTargetView(&view, &[0.05, 0.96, 0.13, 1.0]) };
            let hr = unsafe { chain.Present(0, DXGI_PRESENT(0)) };
            hr.ok().map_err(|e| failure("Present", e))?;
            last_status = hr.0 as u32;
            presents += 1;
            let mut msg = MSG::default();
            while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() } {
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            thread::sleep(Duration::from_millis(80));
        }
        drop(view);
        drop(resource);
        drop(texture);
        drop(chain);
        drop(context);
        drop(device);
        let handle = owned.0;
        drop(owned);
        let cleaned = !unsafe { IsWindow(Some(handle)).as_bool() };
        if !cleaned {
            return Err(Outcome::Fail("child_cleanup_not_verified".to_owned()));
        }
        Ok(format!(
            "driver=warp feature_level=0x{:x} direct3d_present=true             last_present_hresult=0x{last_status:08x} frames={presents}             progman_layered_child=true behind_defview_zorder={below_icon_view}             child_was_visible={visible} destroyed=true duration_ms={duration_ms}             visual_icon_occlusion=REQUIRES_SCREENSHOT",
            level.0,
        ))
    }
}
