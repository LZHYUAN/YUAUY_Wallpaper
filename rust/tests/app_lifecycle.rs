//! T016: pure serialized command handling on a single owner thread.
//! No OS registry, file IO, timer, animation or GPU side effects.
use yuauy_wallpaper::app_state::{AppAction, AppCommand, AppPhase, AppState};

#[test]
fn all_eight_commands_round_trip_in_yaml() {
    let all = [
        AppCommand::Start,
        AppCommand::Tick,
        AppCommand::Reload,
        AppCommand::Pause,
        AppCommand::Resume,
        AppCommand::Quit,
        AppCommand::ShellRestart,
        AppCommand::DisplayChanged,
    ];
    for command in all {
        let yaml = yaml_serde::to_string(&command).unwrap();
        let read: AppCommand = yaml_serde::from_str(&yaml).unwrap();
        assert_eq!(read, command);
    }
}

#[test]
fn startup_is_single_shot_and_leaves_story_details_to_later_phases() {
    let mut app = AppState::default();
    assert_eq!(app.phase(), AppPhase::Uninitialized);
    assert_eq!(
        app.handle(AppCommand::Start),
        vec![AppAction::InitializeNativeShell]
    );
    assert_eq!(app.phase(), AppPhase::Running);
    assert_eq!(app.handle(AppCommand::Start), vec![]);
}

#[test]
fn pausing_skips_ticks_but_allows_settings_reload() {
    let mut app = AppState::default();
    app.handle(AppCommand::Start);
    assert_eq!(app.handle(AppCommand::Tick), vec![AppAction::HandleDueTick]);
    assert_eq!(
        app.handle(AppCommand::Pause),
        vec![AppAction::StopSlideshowDeadline]
    );
    assert_eq!(app.phase(), AppPhase::Paused);
    assert!(app.handle(AppCommand::Tick).is_empty());
    assert_eq!(
        app.handle(AppCommand::Reload),
        vec![AppAction::RequestValidatedReload]
    );
    assert_eq!(
        app.handle(AppCommand::Resume),
        vec![AppAction::ArmFullSlideshowInterval]
    );
    assert_eq!(app.phase(), AppPhase::Running);
}

#[test]
fn explorer_and_display_recovery_preserve_pause_and_do_not_pick_an_image() {
    let mut app = AppState::default();
    app.handle(AppCommand::Start);
    app.handle(AppCommand::Pause);
    assert_eq!(
        app.handle(AppCommand::ShellRestart),
        vec![AppAction::ReattachDesktop, AppAction::ReregisterTray]
    );
    assert_eq!(
        app.handle(AppCommand::DisplayChanged),
        vec![AppAction::ReconcileDisplaySurfaces]
    );
    assert_eq!(app.phase(), AppPhase::Paused);
}

#[test]
fn quit_is_terminal_idempotent_and_does_not_change_autostart_setting() {
    let mut app = AppState::default();
    app.handle(AppCommand::Start);
    assert_eq!(
        app.handle(AppCommand::Quit),
        vec![AppAction::ReleaseOwnedResources]
    );
    assert_eq!(app.phase(), AppPhase::Stopped);
    for cmd in [
        AppCommand::Quit,
        AppCommand::Tick,
        AppCommand::Start,
        AppCommand::Reload,
        AppCommand::Resume,
        AppCommand::ShellRestart,
    ] {
        assert!(app.handle(cmd).is_empty());
    }
}
