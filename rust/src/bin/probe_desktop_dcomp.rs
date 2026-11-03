//! Opt-in DirectComposition counterpart to probe_desktop_d3d11. No product code.
//! Default inspection creates no HWND, COM device, log file or swapchain.
#[cfg(any(windows, test))]
use yuauy_wallpaper::workerw_probe_options::DesktopTopology;
use yuauy_wallpaper::workerw_probe_options::{Mode, parse_args};

/// The opt-in A/B test changes ONLY the owned child's sibling order.
/// The same DirectComposition device, swapchain, color and window styles are used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZLayer {
    BelowDefView,
    AboveDefView,
}

#[cfg(windows)]
impl ZLayer {
    const fn name(self) -> &'static str {
        match self {
            Self::BelowDefView => "below_defview",
            Self::AboveDefView => "above_defview",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProbeArgs {
    mode: Mode,
    layer: ZLayer,
    layered_alpha: Option<u8>,
}

fn options(args: impl IntoIterator<Item = String>) -> Result<ProbeArgs, &'static str> {
    let mut args: Vec<String> = args.into_iter().collect();
    // The above-icon-view control is deliberately accepted only as the final
    // token on an explicitly bounded modern attach command.
    let layer = if args.last().is_some_and(|arg| arg == "--above-icon-view") {
        args.pop();
        ZLayer::AboveDefView
    } else {
        ZLayer::BelowDefView
    };
    let layered_alpha = if args.len() >= 2 && args[args.len() - 2] == "--layered-alpha" {
        let value = args.pop().ok_or("missing layered alpha value")?;
        args.pop();
        match value.as_str() {
            "0" => Some(0),
            "128" => Some(128),
            "255" => Some(255),
            _ => return Err("--layered-alpha must be 0, 128, or 255"),
        }
    } else {
        None
    };
    match parse_args(args)? {
        Mode::Attach { .. } => Err("legacy --attach is forbidden; use --attach-modern"),
        Mode::Inspect if layer == ZLayer::AboveDefView || layered_alpha.is_some() => {
            Err("diagnostic flags require explicit --attach-modern")
        }
        mode => Ok(ProbeArgs {
            mode,
            layer,
            layered_alpha,
        }),
    }
}

#[cfg(any(windows, test))]
fn require_modern(
    mode: Mode,
    topology: DesktopTopology,
    matches_shell: bool,
) -> Result<u64, &'static str> {
    match mode {
        Mode::AttachModern { duration_ms }
            if (1..=10_000).contains(&duration_ms)
                && topology.has_progman
                && topology.has_defview_child
                && topology.raised_desktop
                && topology.same_icon_owner
                && matches_shell =>
        {
            Ok(duration_ms)
        }
        _ => Err(
            "explicit bounded modern opt-in and raised Progman/DefView matching Explorer required",
        ),
    }
}

fn main() -> std::process::ExitCode {
    let mode = match options(std::env::args().skip(1)) {
        Ok(mode) => mode,
        Err(message) => {
            eprintln!(
                "REFUSED {message}; use --inspect or --attach-modern [--duration-ms 1..10000] [--layered-alpha 0|128|255] [--above-icon-view]"
            );
            return std::process::ExitCode::FAILURE;
        }
    };
    #[cfg(windows)]
    let (code, message) = match probe::run(mode) {
        Ok(message) => (0, message),
        Err(probe::Outcome::Skip(message)) => (2, format!("SKIPPED {message}")),
        Err(probe::Outcome::Fail(message)) => (1, format!("API_FAILED {message}")),
    };
    #[cfg(not(windows))]
    let (code, message) = {
        let _ = mode;
        (
            2,
            "SKIPPED Windows-only; visual_wallpaper=UNVERIFIED".to_owned(),
        )
    };
    println!("{message}");
    std::process::ExitCode::from(code)
}

#[cfg(windows)]
mod probe {
    use super::{ProbeArgs, ZLayer, require_modern};
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
                    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice,
                    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D,
                },
                DirectComposition::{
                    DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget,
                    IDCompositionVisual,
                },
                Dxgi::{
                    Common::{
                        DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC,
                    },
                    DXGI_ERROR_WAS_STILL_DRAWING, DXGI_PRESENT_DO_NOT_WAIT, DXGI_SCALING_STRETCH,
                    DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                    DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIAdapter, IDXGIDevice, IDXGIFactory2,
                    IDXGIOutput,
                },
            },
            System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize},
            UI::WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, DispatchMessageW, FindWindowExW, FindWindowW,
                GW_HWNDNEXT, GWL_EXSTYLE, GetLayeredWindowAttributes, GetParent, GetShellWindow,
                GetWindow, GetWindowLongW, GetWindowRect, GetWindowThreadProcessId, HWND_TOP,
                IsWindow, IsWindowVisible, LAYERED_WINDOW_ATTRIBUTES_FLAGS, LWA_ALPHA, MSG,
                PM_REMOVE, PeekMessageW, SWP_NOACTIVATE, SWP_SHOWWINDOW,
                SetLayeredWindowAttributes, SetWindowPos, TranslateMessage, WS_CHILD,
                WS_CLIPSIBLINGS, WS_DISABLED, WS_EX_LAYERED, WS_EX_NOACTIVATE,
                WS_EX_NOREDIRECTIONBITMAP, WS_EX_TRANSPARENT,
            },
        },
        core::{Interface, w},
    };
    use yuauy_wallpaper::workerw_probe_options::{DesktopTopology, Mode};

    pub(super) enum Outcome {
        Skip(String),
        Fail(String),
    }
    fn failure(op: &str, error: windows::core::Error) -> Outcome {
        Outcome::Fail(format!(
            "stage={op} HRESULT=0x{:08x} visual_wallpaper=UNVERIFIED",
            error.code().0 as u32
        ))
    }
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            // SAFETY: balanced only after successful CoInitializeEx on this thread.
            unsafe { CoUninitialize() };
        }
    }
    struct OwnedChild(HWND);
    impl OwnedChild {
        fn destroy(&mut self) -> Result<(), Outcome> {
            if self.0.is_invalid() {
                return Ok(());
            }
            let hwnd = self.0;
            // SAFETY: this guard exclusively owns this thread's test HWND.
            unsafe { DestroyWindow(hwnd) }.map_err(|e| failure("DestroyWindow", e))?;
            if unsafe { IsWindow(Some(hwnd)).as_bool() } {
                return Err(Outcome::Fail("child_cleanup_not_verified".to_owned()));
            }
            self.0 = HWND::default();
            Ok(())
        }
    }
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            if let Err(Outcome::Fail(message)) = self.destroy() {
                eprintln!("CLEANUP_FAILED {message}");
            }
        }
    }
    // Interface wrappers Release on every return path. Drop disconnects the tree
    // before its interfaces and the child are released; no unbounded completion wait.
    struct Tree {
        visual: IDCompositionVisual,
        target: IDCompositionTarget,
        device: IDCompositionDevice,
        detached: bool,
    }
    impl Tree {
        fn detach(&mut self) -> windows::core::Result<()> {
            if self.detached {
                return Ok(());
            }
            unsafe {
                self.target.SetRoot(None::<&IDCompositionVisual>)?;
                self.visual.SetContent(None::<&windows::core::IUnknown>)?;
                self.device.Commit()?;
            }
            self.detached = true;
            Ok(())
        }
    }
    impl Drop for Tree {
        fn drop(&mut self) {
            if let Err(error) = self.detach() {
                eprintln!(
                    "CLEANUP_FAILED detach HRESULT=0x{:08x}",
                    error.code().0 as u32
                );
            }
        }
    }
    fn pid(hwnd: HWND) -> u32 {
        let mut result = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut result)) };
        result
    }
    struct Host {
        progman: HWND,
        defview: HWND,
        owner: u32,
        topology: DesktopTopology,
        matches_shell: bool,
    }
    fn inspect() -> Result<Host, Outcome> {
        // SAFETY: these queries only read the current desktop; no shell messages.
        let progman = unsafe { FindWindowW(w!("Progman"), None) }
            .map_err(|_| Outcome::Skip("Progman_absent".to_owned()))?;
        let defview = unsafe { FindWindowExW(Some(progman), None, w!("SHELLDLL_DefView"), None) }
            .map_err(|_| Outcome::Skip("direct_DefView_absent".to_owned()))?;
        let owner = pid(progman);
        let shell = unsafe { GetShellWindow() };
        let matches_shell = !shell.is_invalid() && owner != 0 && pid(shell) == owner;
        let topology = DesktopTopology {
            has_progman: true,
            has_defview_child: true,
            has_workerw_child: false, // Deliberately irrelevant for this strategy.
            raised_desktop: unsafe { GetWindowLongW(progman, GWL_EXSTYLE) } as u32
                & WS_EX_NOREDIRECTIONBITMAP.0
                != 0,
            same_icon_owner: owner != 0
                && pid(defview) == owner
                && unsafe { GetParent(defview) }.ok() == Some(progman),
        };
        Ok(Host {
            progman,
            defview,
            owner,
            topology,
            matches_shell,
        })
    }
    fn validate(host: &Host, mode: Mode) -> Result<(), Outcome> {
        let now = inspect()?;
        require_modern(mode, now.topology, now.matches_shell)
            .map_err(|e| Outcome::Skip(e.to_owned()))?;
        if now.progman != host.progman || now.defview != host.defview || now.owner != host.owner {
            return Err(Outcome::Skip("Explorer_topology_changed".to_owned()));
        }
        Ok(())
    }

    /// Read sibling order without changing any Explorer-owned HWND.
    fn expected_relative_order(hwnd: HWND, defview: HWND, layer: ZLayer) -> bool {
        let (start, target) = match layer {
            ZLayer::BelowDefView => (defview, hwnd),
            ZLayer::AboveDefView => (hwnd, defview),
        };
        let mut current = start;
        for _ in 0..128 {
            let Some(next) = (unsafe { GetWindow(current, GW_HWNDNEXT) }).ok() else {
                break;
            };
            if next == target {
                return true;
            }
            current = next;
        }
        false
    }

    pub(super) fn run(args: ProbeArgs) -> Result<String, Outcome> {
        let mode = args.mode;
        let layer = args.layer;
        let layered_alpha = args.layered_alpha;
        let host = inspect()?;
        if mode == Mode::Inspect {
            return Ok(format!(
                "INSPECT_ONLY raised={} same_icon_owner={} matches_Explorer={} attach_eligible={} composition=NOT_RUN visual_wallpaper=UNVERIFIED",
                host.topology.raised_desktop,
                host.topology.same_icon_owner,
                host.matches_shell,
                require_modern(
                    Mode::AttachModern { duration_ms: 1 },
                    host.topology,
                    host.matches_shell
                )
                .is_ok()
            ));
        }
        let duration_ms = require_modern(mode, host.topology, host.matches_shell)
            .map_err(|e| Outcome::Skip(e.to_owned()))?;
        let mut rect = Default::default();
        unsafe { GetWindowRect(host.progman, &mut rect) }
            .map_err(|e| failure("GetWindowRect", e))?;
        if rect.right - rect.left < 240 || rect.bottom - rect.top < 220 {
            return Err(Outcome::Skip("desktop_too_small".to_owned()));
        }
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
            .ok()
            .map_err(|e| failure("CoInitializeEx", e))?;
        let _apartment = Apartment;
        validate(&host, mode)?;
        // Hidden until the composition tree is ready. Layered mode intentionally
        // omits NOREDIRECTIONBITMAP and retains the same DComp WARP visual.
        let exstyle = WS_EX_NOACTIVATE
            | WS_EX_TRANSPARENT
            | if layered_alpha.is_some() {
                WS_EX_LAYERED
            } else {
                WS_EX_NOREDIRECTIONBITMAP
            };
        let hwnd = unsafe {
            CreateWindowExW(
                exstyle,
                w!("STATIC"),
                w!("YUAUY DComp WARP diagnostic"),
                WS_CHILD | WS_CLIPSIBLINGS | WS_DISABLED,
                12,
                12,
                220,
                190,
                Some(host.progman),
                None,
                None,
                None,
            )
        }
        .map_err(|e| failure("CreateWindowExW", e))?;
        let mut child = OwnedChild(hwnd);
        let mut layered_readback = "none".to_owned();
        if let Some(alpha) = layered_alpha {
            // Only configure OUR own HWND. Layered children require an explicit
            // opacity/update call before being made visible.
            unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA) }
                .map_err(|e| failure("SetLayeredWindowAttributes_owned_child", e))?;
            let mut read_alpha = 0u8;
            let mut read_flags = LAYERED_WINDOW_ATTRIBUTES_FLAGS::default();
            unsafe {
                GetLayeredWindowAttributes(hwnd, None, Some(&mut read_alpha), Some(&mut read_flags))
            }
            .map_err(|e| failure("GetLayeredWindowAttributes_owned_child", e))?;
            if read_alpha != alpha || read_flags != LWA_ALPHA {
                return Err(Outcome::Fail(format!(
                    "owned_layered_readback_mismatch requested={alpha} read={read_alpha} flags=0x{:x}",
                    read_flags.0
                )));
            }
            layered_readback = format!("alpha={alpha} flags=0x{:x}", read_flags.0);
        }
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        let mut level = D3D_FEATURE_LEVEL::default();
        unsafe {
            D3D11CreateDevice(
                None::<&IDXGIAdapter>,
                D3D_DRIVER_TYPE_WARP,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                Some(&mut level),
                Some(&mut context),
            )
        }
        .map_err(|e| failure("D3D11CreateDevice_WARP", e))?;
        let device = device.ok_or_else(|| Outcome::Fail("no_D3D_device".to_owned()))?;
        let context = context.ok_or_else(|| Outcome::Fail("no_D3D_context".to_owned()))?;
        let dxgi: IDXGIDevice = device.cast().map_err(|e| failure("IDXGIDevice", e))?;
        let adapter = unsafe { dxgi.GetAdapter() }.map_err(|e| failure("GetAdapter", e))?;
        let factory: IDXGIFactory2 =
            unsafe { adapter.GetParent() }.map_err(|e| failure("IDXGIFactory2", e))?;
        let desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: 220,
            Height: 190,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
            ..Default::default()
        };
        // No fallback to hardware or HWND blt: failure is evidence about this path.
        let chain =
            unsafe { factory.CreateSwapChainForComposition(&device, &desc, None::<&IDXGIOutput>) }
                .map_err(|e| failure("CreateSwapChainForComposition_WARP", e))?;
        let comp: IDCompositionDevice = unsafe { DCompositionCreateDevice(&dxgi) }
            .map_err(|e| failure("DCompositionCreateDevice", e))?;
        let target = unsafe { comp.CreateTargetForHwnd(hwnd, false) }
            .map_err(|e| failure("CreateTargetForHwnd", e))?;
        let visual = unsafe { comp.CreateVisual() }.map_err(|e| failure("CreateVisual", e))?;
        let mut tree = Tree {
            visual,
            target,
            device: comp,
            detached: false,
        };
        unsafe {
            tree.visual
                .SetContent(&chain)
                .map_err(|e| failure("SetContent", e))?;
            tree.target
                .SetRoot(&tree.visual)
                .map_err(|e| failure("SetRoot", e))?;
            tree.device.Commit().map_err(|e| failure("Commit", e))?;
        }
        let texture: ID3D11Texture2D =
            unsafe { chain.GetBuffer(0) }.map_err(|e| failure("GetBuffer", e))?;
        let mut view: Option<ID3D11RenderTargetView> = None;
        unsafe { device.CreateRenderTargetView(&texture, None, Some(&mut view)) }
            .map_err(|e| failure("CreateRenderTargetView", e))?;
        let view = view.ok_or_else(|| Outcome::Fail("no_render_target".to_owned()))?;
        validate(&host, mode)?;
        let started = Instant::now();
        unsafe {
            SetWindowPos(
                hwnd,
                Some(match layer {
                    ZLayer::BelowDefView => host.defview,
                    ZLayer::AboveDefView => HWND_TOP,
                }),
                12,
                12,
                220,
                190,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )
        }
        .map_err(|e| failure("SetWindowPos_owned_child", e))?;
        let mut frames = 0u32;
        let mut busy = 0u32;
        let mut last_hr = 0u32;
        while started.elapsed() < Duration::from_millis(duration_ms) {
            validate(&host, mode)?;
            if unsafe { GetParent(hwnd) }.ok() != Some(host.progman)
                || !expected_relative_order(hwnd, host.defview, layer)
                || !unsafe { IsWindowVisible(hwnd).as_bool() }
            {
                return Err(Outcome::Fail(format!(
                    "owned_child_parent_zorder_or_visibility_changed placement={}",
                    layer.name()
                )));
            }
            // Opaque saturated magenta, distinct from the legacy lime marker.
            unsafe { context.ClearRenderTargetView(&view, &[0.95, 0.03, 0.75, 1.0]) };
            // No blocking vblank wait in this bounded diagnostic; not an FPS test.
            let hr = unsafe { chain.Present(0, DXGI_PRESENT_DO_NOT_WAIT) };
            last_hr = hr.0 as u32;
            if hr == DXGI_ERROR_WAS_STILL_DRAWING {
                busy += 1;
            } else {
                hr.ok().map_err(|e| failure("Present", e))?;
                if hr.0 == 0 {
                    frames += 1;
                }
            }
            let mut msg = MSG::default();
            while unsafe { PeekMessageW(&mut msg, Some(hwnd), 0, 0, PM_REMOVE).as_bool() } {
                unsafe {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                if started.elapsed() >= Duration::from_millis(duration_ms) {
                    break;
                }
            }
            thread::sleep(
                Duration::from_millis(duration_ms)
                    .saturating_sub(started.elapsed())
                    .min(Duration::from_millis(16)),
            );
        }
        tree.detach().map_err(|e| failure("DetachCommit", e))?;
        drop(view);
        drop(texture);
        drop(tree);
        drop(chain);
        drop(factory);
        drop(adapter);
        drop(dxgi);
        drop(context);
        drop(device);
        child.destroy()?;
        if frames == 0 {
            return Err(Outcome::Fail(format!(
                "no_S_OK_present busy={busy} last_present_hresult=0x{last_hr:08x} child_destroyed=true visual_wallpaper=UNVERIFIED"
            )));
        }
        Ok(format!(
            "API_SUBMISSIONS_ONLY path=dcomp_composition_flip_sequential placement={} layered={} driver=warp feature_level=0x{:x} present_s_ok={frames} busy={busy} last_present_hresult=0x{last_hr:08x} duration_ms={duration_ms} sibling_order_verified=true child_destroyed=true visual_wallpaper=UNVERIFIED requires_live_screenshot=true",
            layer.name(),
            layered_readback,
            level.0
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<ProbeArgs, &'static str> {
        options(args.iter().map(|s| (*s).to_owned()))
    }
    fn eligible() -> DesktopTopology {
        DesktopTopology {
            has_progman: true,
            has_defview_child: true,
            has_workerw_child: false,
            raised_desktop: true,
            same_icon_owner: true,
        }
    }
    #[test]
    fn safe_default_and_duration_boundaries() {
        assert_eq!(
            parse(&[]),
            Ok(ProbeArgs {
                mode: Mode::Inspect,
                layer: ZLayer::BelowDefView,
                layered_alpha: None
            })
        );
        assert_eq!(
            parse(&["--inspect"]),
            Ok(ProbeArgs {
                mode: Mode::Inspect,
                layer: ZLayer::BelowDefView,
                layered_alpha: None
            })
        );
        assert_eq!(
            parse(&["--attach-modern"]),
            Ok(ProbeArgs {
                mode: Mode::AttachModern { duration_ms: 3000 },
                layer: ZLayer::BelowDefView,
                layered_alpha: None
            })
        );
        assert_eq!(
            parse(&[
                "--attach-modern",
                "--duration-ms",
                "5000",
                "--above-icon-view"
            ]),
            Ok(ProbeArgs {
                mode: Mode::AttachModern { duration_ms: 5000 },
                layer: ZLayer::AboveDefView,
                layered_alpha: None
            })
        );
        for alpha in ["0", "128", "255"] {
            assert_eq!(
                parse(&[
                    "--attach-modern",
                    "--duration-ms",
                    "100",
                    "--layered-alpha",
                    alpha
                ]),
                Ok(ProbeArgs {
                    mode: Mode::AttachModern { duration_ms: 100 },
                    layer: ZLayer::BelowDefView,
                    layered_alpha: Some(alpha.parse::<u8>().unwrap()),
                })
            );
        }
        assert_eq!(
            parse(&[
                "--attach-modern",
                "--layered-alpha",
                "128",
                "--above-icon-view"
            ]),
            Ok(ProbeArgs {
                mode: Mode::AttachModern { duration_ms: 3000 },
                layer: ZLayer::AboveDefView,
                layered_alpha: Some(128),
            })
        );
        for duration in ["1", "10000"] {
            assert!(parse(&["--attach-modern", "--duration-ms", duration]).is_ok());
        }
        for args in [
            vec!["--attach"],
            vec!["--inspect", "--attach-modern"],
            vec!["--duration-ms", "100"],
            vec!["--attach-modern", "--duration-ms"],
            vec!["--attach-modern", "--request-workerw"],
            vec!["--attach-modern", "--attach-modern"],
            vec!["--inspect", "--above-icon-view"],
            vec!["--inspect", "--layered-alpha", "255"],
            vec!["--layered-alpha", "255"],
            vec!["--attach-modern", "--layered-alpha", "1"],
            vec!["--attach-modern", "--layered-alpha", "127"],
            vec!["--attach-modern", "--layered-alpha", "256"],
            vec![
                "--attach-modern",
                "--layered-alpha",
                "255",
                "--layered-alpha",
                "255",
            ],
            vec!["--above-icon-view"],
            vec![
                "--attach-modern",
                "--above-icon-view",
                "--duration-ms",
                "100",
            ],
            vec!["--attach-modern", "--above-icon-view", "--above-icon-view"],
        ] {
            assert!(parse(&args).is_err(), "{args:?}");
        }
        for duration in ["0", "10001", "-1", "NaN", "1.5", "18446744073709551616"] {
            assert!(parse(&["--attach-modern", "--duration-ms", duration]).is_err());
        }
        assert!(parse(&["--attach-modern", "--duration-ms", "1", "extra"]).is_err());
    }
    #[test]
    fn refuses_unsafe_modes_and_topologies() {
        let mode = Mode::AttachModern { duration_ms: 10 };
        assert_eq!(require_modern(mode, eligible(), true), Ok(10));
        for invalid in [
            Mode::Inspect,
            Mode::Attach { duration_ms: 10 },
            Mode::AttachModern { duration_ms: 0 },
            Mode::AttachModern { duration_ms: 10001 },
        ] {
            assert!(require_modern(invalid, eligible(), true).is_err());
        }
        assert!(require_modern(mode, eligible(), false).is_err());
        for field in 0..4 {
            let mut topology = eligible();
            match field {
                0 => topology.has_progman = false,
                1 => topology.has_defview_child = false,
                2 => topology.raised_desktop = false,
                _ => topology.same_icon_owner = false,
            }
            assert!(require_modern(mode, topology, true).is_err());
        }
    }
}
