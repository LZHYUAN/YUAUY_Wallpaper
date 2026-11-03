//! T018 read-only shell signaling contract, independent of Win32 dispatch.
use yuauy_wallpaper::{
    app_state::{AppAction, AppCommand, AppPhase, AppState},
    shell::{ShellSignal, command_for_signal},
};

#[test]
fn shell_notifications_map_to_single_thread_commands() {
    assert_eq!(
        command_for_signal(ShellSignal::SlideshowDeadline),
        AppCommand::Tick
    );
    assert_eq!(
        command_for_signal(ShellSignal::ExplorerRestart),
        AppCommand::ShellRestart
    );
    assert_eq!(
        command_for_signal(ShellSignal::DisplayChanged),
        AppCommand::DisplayChanged
    );
    assert_eq!(
        command_for_signal(ShellSignal::ExitRequested),
        AppCommand::Quit
    );
}

#[test]
fn no_native_side_effect_occurs_when_shell_merely_constructs_commands() {
    let state = AppState::default();
    assert_eq!(state.phase(), AppPhase::Uninitialized);
    let _ = command_for_signal(ShellSignal::DisplayChanged);
    assert_eq!(state.phase(), AppPhase::Uninitialized);
}

#[test]
fn shell_restart_does_not_start_new_slideshow_while_paused() {
    let mut app = AppState::default();
    app.handle(AppCommand::Start);
    app.handle(AppCommand::Pause);
    assert_eq!(
        app.handle(command_for_signal(ShellSignal::ExplorerRestart)),
        vec![AppAction::ReattachDesktop, AppAction::ReregisterTray]
    );
    assert_eq!(app.phase(), AppPhase::Paused);
}
