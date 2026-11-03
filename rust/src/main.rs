//! Native entry point. Production wallpaper startup remains gated by
//! WorkerW/WindowsEnv acceptance; only a bounded hidden-window smoke is enabled.
use std::process::ExitCode;

#[cfg(windows)]
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--shell-smoke") || args.next().is_some() {
        eprintln!(
            "YUAUY Wallpaper: production desktop app is not implemented; use --shell-smoke for bounded Win32 event loop acceptance."
        );
        return ExitCode::FAILURE;
    }
    match yuauy_wallpaper::shell::native::run_smoke(400) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("FAIL shell_smoke: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    eprintln!("SKIPPED: Win32 shell smoke requires Windows.");
    ExitCode::FAILURE
}
