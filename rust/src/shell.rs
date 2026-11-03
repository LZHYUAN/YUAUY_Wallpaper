//! Win32 hidden message-window shell skeleton.
//!
//! This is NOT a wallpaper or tray implementation. It never modifies Explorer,
//! creates WorkerW, registers global hotkeys, adds HKCU Run entries, reads user
//! images, or starts background monitoring. The one-shot smoke is explicitly
//! requested and runs on a single owning thread.

use crate::app_state::AppCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellSignal {
    SlideshowDeadline,
    ExplorerRestart,
    DisplayChanged,
    ExitRequested,
}

pub fn command_for_signal(signal: ShellSignal) -> AppCommand {
    match signal {
        ShellSignal::SlideshowDeadline => AppCommand::Tick,
        ShellSignal::ExplorerRestart => AppCommand::ShellRestart,
        ShellSignal::DisplayChanged => AppCommand::DisplayChanged,
        ShellSignal::ExitRequested => AppCommand::Quit,
    }
}

#[cfg(windows)]
pub mod native {
    use std::time::{Duration, Instant};
    use windows::{
        Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DispatchMessageW, GetMessageW, IsWindow, KillTimer, MSG,
            RegisterWindowMessageW, SetTimer, TranslateMessage, WINDOW_EX_STYLE, WM_DISPLAYCHANGE,
            WM_TIMER, WS_POPUP,
        },
        core::{Error, Result, w},
    };

    use crate::{
        app_state::{AppAction, AppCommand, AppPhase, AppState},
        render::native::OwnedWin32Window,
        shell::{ShellSignal, command_for_signal},
    };

    /// Starts an explicitly scoped, short-lived hidden test event loop on
    /// this thread. A WM_TIMER message wakes blocking GetMessageW; no polling.
    pub fn run_smoke(duration_ms: u32) -> Result<()> {
        if !(1..=2000).contains(&duration_ms) {
            return Err(Error::from_thread());
        }

        // SAFETY: STATIC is a predefined Win32 class and this window is hidden,
        // unparented and owned by this process. It cannot enter Explorer.
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("YUAUY hidden message loop diagnostic"),
                WS_POPUP,
                0,
                0,
                8,
                8,
                None,
                None,
                None,
                None,
            )?
        };
        // SAFETY: the window was created on the current thread and will only
        // be destroyed here by the thread-affine RAII guard.
        let owned = unsafe {
            OwnedWin32Window::from_new_owned_hwnd(hwnd).map_err(|_| Error::from_thread())?
        };
        let mut app = AppState::default();
        if app.handle(AppCommand::Start) != [AppAction::InitializeNativeShell] {
            return Err(Error::from_thread());
        }

        // TaskbarCreated is an ordinary well-known shell *message identifier*,
        // not a global window, hook, autostart or notification registration.
        let taskbar_message = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
        let timer_id = unsafe { SetTimer(Some(hwnd), 1, duration_ms, None) };
        if timer_id == 0 {
            return Err(Error::from_thread());
        }
        let started = Instant::now();
        let mut timer_fired = false;
        let mut msg = MSG::default();
        loop {
            // SAFETY: GetMessage blocks on this creating thread's queue and
            // returns -1 for error, 0 for WM_QUIT, >0 for a message.
            let result = unsafe { GetMessageW(&mut msg, None, 0, 0) }.0;
            if result == -1 {
                break;
            }
            if result == 0 {
                break;
            }
            if msg.hwnd == hwnd && msg.message == WM_TIMER && msg.wParam.0 == timer_id {
                // SAFETY: only this process-owned HWND's timer is removed.
                let _ = unsafe { KillTimer(Some(hwnd), timer_id) };
                let actions = app.handle(command_for_signal(ShellSignal::ExitRequested));
                if actions != [AppAction::ReleaseOwnedResources] {
                    return Err(Error::from_thread());
                }
                timer_fired = true;
                break;
            }
            if msg.hwnd == hwnd && msg.message == WM_DISPLAYCHANGE {
                let _ = app.handle(command_for_signal(ShellSignal::DisplayChanged));
            } else if taskbar_message != 0 && msg.message == taskbar_message {
                let _ = app.handle(command_for_signal(ShellSignal::ExplorerRestart));
            }
            // SAFETY: standard same-thread Win32 dispatch. No custom WndProc,
            // input focus, visible window, GPU draw or background thread.
            unsafe {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        let elapsed = started.elapsed();
        let stopped = app.phase() == AppPhase::Stopped;
        drop(owned);
        let released = !unsafe { IsWindow(Some(hwnd)).as_bool() };
        if !timer_fired || !stopped || !released || elapsed > Duration::from_secs(5) {
            return Err(Error::from_thread());
        }
        println!(
            "PASS shell_message_loop=true timer_fired=true state_stopped=true hwnd_destroyed=true window_visible=false duration_ms={}",
            elapsed.as_millis(),
        );
        Ok(())
    }
}
