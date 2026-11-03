//! Isolated diagnostic for the Win11 25H2 CreateWindowExW discrepancy.
//! All windows are HIDDEN and destroyed immediately; no Explorer window
//! is restyled, reparented or presented, and no lasting OS state is changed.

#[cfg(not(windows))]
fn main() {
    eprintln!("SKIPPED: Windows only");
}

#[cfg(windows)]
fn main() {
    use windows::Win32::Foundation::GetLastError;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, FindWindowW, IsWindow, WINDOW_EX_STYLE, WS_CHILD,
        WS_CLIPSIBLINGS, WS_DISABLED, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT, WS_POPUP,
    };
    use windows::core::w;
    let Ok(progman) = (unsafe { FindWindowW(w!("Progman"), None) }) else {
        println!("SKIPPED: no Progman");
        return;
    };
    println!("PROGMAN_EXISTS=true");
    let xstyle = WINDOW_EX_STYLE(WS_EX_LAYERED.0 | WS_EX_NOACTIVATE.0 | WS_EX_TRANSPARENT.0);
    let variants = [
        ("own_top_level", WS_POPUP, WINDOW_EX_STYLE(0), None),
        ("simple_child", WS_CHILD, WINDOW_EX_STYLE(0), Some(progman)),
        (
            "layered_only",
            WS_CHILD,
            WINDOW_EX_STYLE(WS_EX_LAYERED.0),
            Some(progman),
        ),
        (
            "transparent_only",
            WS_CHILD,
            WINDOW_EX_STYLE(WS_EX_TRANSPARENT.0),
            Some(progman),
        ),
        (
            "noactivate_only",
            WS_CHILD,
            WINDOW_EX_STYLE(WS_EX_NOACTIVATE.0),
            Some(progman),
        ),
        (
            "layered_and_transparent",
            WS_CHILD,
            WINDOW_EX_STYLE(WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0),
            Some(progman),
        ),
        (
            "layered_and_noactivate",
            WS_CHILD,
            WINDOW_EX_STYLE(WS_EX_LAYERED.0 | WS_EX_NOACTIVATE.0),
            Some(progman),
        ),
        (
            "layered_child_no_disabled",
            WS_CHILD,
            WINDOW_EX_STYLE(WS_EX_LAYERED.0 | WS_EX_NOACTIVATE.0 | WS_EX_TRANSPARENT.0),
            Some(progman),
        ),
        (
            "layered_child",
            WS_CHILD | WS_CLIPSIBLINGS | WS_DISABLED,
            xstyle,
            Some(progman),
        ),
    ];
    for (name, style, ex, parent) in variants {
        let result = unsafe {
            CreateWindowExW(
                ex,
                w!("STATIC"),
                w!("YUAUY hidden Win32 diagnostic"),
                style,
                12,
                12,
                220,
                190,
                parent,
                None,
                None,
                None,
            )
        };
        let last_error = unsafe { GetLastError().0 };
        match result {
            Ok(hwnd) => {
                let valid = unsafe { IsWindow(Some(hwnd)).as_bool() };
                println!("CASE={name} PASS valid={valid} last_error={last_error}");
                let _ = unsafe { DestroyWindow(hwnd) };
            }
            Err(e) => println!(
                "CASE={name} FAIL hresult=0x{:08x} last_error={last_error}",
                e.code().0 as u32
            ),
        };
    }
    println!("DONE no visible window; all created windows cleaned");
}
