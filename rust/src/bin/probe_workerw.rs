//! T009: non-invasive Win11 24H2+ desktop-host topology and child-window probe.
//!
//! Default is READ-ONLY. Explicit --attach creates a tiny legacy WorkerW
//! child. Explicit --attach-modern creates a tiny *layered* Progman child
//! behind SHELLDLL_DefView on raised desktops. Only this process-owned HWND
//! is repositioned/destroyed. Never sends 0x052C, changes Explorer styles,
//! or reorders Explorer-owned HWNDs. Icon layering needs WindowsEnv capture.

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("SKIPPED: WorkerW probe is Windows-only.");
    std::process::ExitCode::from(2)
}

#[cfg(windows)]
fn main() -> std::process::ExitCode {
    use yuauy_wallpaper::workerw_probe_options::parse_args;
    let mode = match parse_args(std::env::args().skip(1)) {
        Ok(mode) => mode,
        Err(e) => {
            eprintln!("FAIL bad args: {e}");
            eprintln!(
                "Usage: probe_workerw [--inspect | --attach | --attach-modern] [--duration-ms 1..10000]"
            );
            return std::process::ExitCode::FAILURE;
        }
    };
    match win::run(mode) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(win::ProbeError::Skip(reason)) => {
            eprintln!("SKIPPED: {reason}");
            std::process::ExitCode::from(2)
        }
        Err(win::ProbeError::Win(error)) => {
            eprintln!("FAIL: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(windows)]
mod win {
    use std::{
        thread,
        time::{Duration, Instant},
    };
    use windows::{
        Win32::{
            Foundation::{COLORREF, HWND},
            UI::WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, DispatchMessageW, FindWindowExW, FindWindowW,
                GW_HWNDNEXT, GWL_EXSTYLE, GetParent, GetWindow, GetWindowLongW, GetWindowRect,
                GetWindowThreadProcessId, IsWindow, IsWindowVisible, LWA_ALPHA, MSG, PM_REMOVE,
                PeekMessageW, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
                SetLayeredWindowAttributes, SetWindowPos, ShowWindow, TranslateMessage,
                WINDOW_EX_STYLE, WS_CHILD, WS_CLIPSIBLINGS, WS_DISABLED, WS_EX_LAYERED,
                WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TRANSPARENT,
            },
        },
        core::w,
    };
    use yuauy_wallpaper::workerw_probe_options::{AttachTarget, Mode, select_target};

    pub(super) enum ProbeError {
        Skip(&'static str),
        Win(windows::core::Error),
    }
    impl From<windows::core::Error> for ProbeError {
        fn from(value: windows::core::Error) -> Self {
            Self::Win(value)
        }
    }

    /// Guard only the probe-owned HWND: Explorer's window hierarchy is untouched.
    struct OwnedChild(HWND);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            // SAFETY: created on this thread; DestroyWindow only acts on our child.
            unsafe {
                let _ = DestroyWindow(self.0);
            }
        }
    }

    fn process_id(hwnd: HWND) -> u32 {
        let mut pid = 0;
        // SAFETY: read-only query of a valid window returned by FindWindowExW.
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        pid
    }

    pub(super) fn run(mode: Mode) -> std::result::Result<(), ProbeError> {
        // SAFETY: Win32 read-only lookup; None means any window title.
        let progman = unsafe { FindWindowW(w!("Progman"), None) }
            .map_err(|_| ProbeError::Skip("Progman absent; no shell desktop to inspect"))?;
        // On >=24H2 both are expected direct children; never guess a parent.
        let worker =
            unsafe { FindWindowExW(Some(progman), None, w!("WorkerW"), None) }.map_err(|_| {
                ProbeError::Skip("WorkerW child of Progman absent; no shell message sent")
            })?;
        let defview = unsafe { FindWindowExW(Some(progman), None, w!("SHELLDLL_DefView"), None) }
            .map_err(|_| {
            ProbeError::Skip("SHELLDLL_DefView not a Progman child (topology unsupported)")
        })?;

        // SAFETY: these read-only queries do not mutate the desktop or z-order.
        if unsafe { GetParent(worker)? } != progman || unsafe { GetParent(defview)? } != progman {
            return Err(ProbeError::Skip(
                "WorkerW/DefView parent changed while inspecting",
            ));
        }
        let owner_pid = process_id(progman);
        if owner_pid == 0 || process_id(worker) != owner_pid || process_id(defview) != owner_pid {
            return Err(ProbeError::Skip(
                "desktop window owners differ; refusing attach",
            ));
        }
        let mut rect = Default::default();
        unsafe { GetWindowRect(worker, &mut rect)? };
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        let worker_visible = unsafe { IsWindowVisible(worker).as_bool() };
        let raised_desktop_hint = (unsafe { GetWindowLongW(progman, GWL_EXSTYLE) } as u32
            & WS_EX_NOREDIRECTIONBITMAP.0)
            != 0;
        println!(
            "INSPECT progman_child_workerw=true progman_child_defview=true same_owner=true worker_rect={width}x{height} worker_visible={worker_visible} progman_no_redirection={raised_desktop_hint} attach_requested={} behind_icons=UNVERIFIED",
            !matches!(mode, Mode::Inspect)
        );

        let target = select_target(mode, raised_desktop_hint).map_err(ProbeError::Skip)?;
        if let Some(target) = target {
            let duration_ms = match mode {
                Mode::Attach { duration_ms } | Mode::AttachModern { duration_ms } => duration_ms,
                Mode::Inspect => unreachable!("select_target(Inspect) returns None"),
            };
            if width < 128 || height < 72 {
                return Err(ProbeError::Skip(
                    "desktop host dimensions unsuitable for bounded child probe",
                ));
            }
            // Only the probe-owned HWND is created and moved. No shell window
            // receives SetParent, SetWindowPos, SetWindowLong, or 0x052C.
            let (host, class, label, ex_style, strategy) = match target {
                AttachTarget::WorkerW => (
                    worker,
                    w!("BUTTON"),
                    w!("YUAUY legacy WorkerW probe"),
                    WINDOW_EX_STYLE(WS_EX_NOACTIVATE.0 | WS_EX_TRANSPARENT.0),
                    "workerw_legacy",
                ),
                AttachTarget::ProgmanLayered => (
                    progman,
                    w!("STATIC"),
                    w!("YUAUY raised-desktop probe"),
                    WINDOW_EX_STYLE(WS_EX_NOACTIVATE.0 | WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0),
                    "progman_layered",
                ),
            };
            let child = OwnedChild(unsafe {
                CreateWindowExW(
                    ex_style,
                    class,
                    label,
                    WS_CHILD | WS_CLIPSIBLINGS | WS_DISABLED,
                    12,
                    12,
                    110,
                    40,
                    Some(host),
                    None,
                    None,
                    None,
                )?
            });
            if unsafe { GetParent(child.0)? } != host {
                return Err(ProbeError::Skip("temporary child parent validation failed"));
            }
            let mut zorder_below_icons = false;
            match target {
                AttachTarget::ProgmanLayered => {
                    // On >=24H2 a layered child of Progman immediately below
                    // SHELLDLL_DefView can be composited behind desktop icons.
                    // This does not change the z-order of any Explorer window.
                    unsafe {
                        SetLayeredWindowAttributes(child.0, COLORREF(0), 255, LWA_ALPHA)?;
                        SetWindowPos(
                            child.0,
                            Some(defview),
                            12,
                            12,
                            110,
                            40,
                            SWP_NOACTIVATE | SWP_SHOWWINDOW,
                        )?;
                    }
                    zorder_below_icons =
                        unsafe { GetWindow(defview, GW_HWNDNEXT).ok() == Some(child.0) };
                }
                AttachTarget::WorkerW => {
                    // SAFETY: shows only our 110x40 disabled non-activating test button.
                    let _ = unsafe { ShowWindow(child.0, SW_SHOWNOACTIVATE) };
                }
            }
            let was_visible = unsafe { IsWindowVisible(child.0).as_bool() };
            let started = Instant::now();
            while started.elapsed() < Duration::from_millis(duration_ms) {
                let mut msg = MSG::default();
                // SAFETY: read/pump only this test process's window messages.
                while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() } {
                    unsafe {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
                thread::sleep(Duration::from_millis(16));
            }
            let hwnd = child.0;
            drop(child);
            let cleaned = !unsafe { IsWindow(Some(hwnd)).as_bool() };
            if !cleaned {
                return Err(ProbeError::Skip(
                    "temporary child cleanup could not be verified",
                ));
            }
            println!(
                "ATTACH strategy={strategy} result=PASS owned_child_parent_valid=true child_iswindowvisible={was_visible} zorder_below_defview={zorder_below_icons} destroyed=true duration_ms={duration_ms} icon_occlusion=UNVERIFIED"
            );
        }
        Ok(())
    }
}
