//! The serialized single-threaded application command reducer.
//!
//! It emits intents and performs NO Win32/IO/graphics action itself. The
//! corresponding shell/scheduler adapters will interpret intents in later
//! Spec Kit phases; normal C# animation equations remain outside this file.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppCommand {
    Start,
    Tick,
    Reload,
    Pause,
    Resume,
    Quit,
    ShellRestart,
    DisplayChanged,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppPhase {
    #[default]
    Uninitialized,
    Running,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    InitializeNativeShell,
    HandleDueTick,
    RequestValidatedReload,
    StopSlideshowDeadline,
    ArmFullSlideshowInterval,
    ReattachDesktop,
    ReregisterTray,
    ReconcileDisplaySurfaces,
    ReleaseOwnedResources,
}

#[derive(Debug, Default)]
pub struct AppState {
    phase: AppPhase,
}

impl AppState {
    pub fn phase(&self) -> AppPhase {
        self.phase
    }
    /// Only the owning Win32 message thread may call this reducer in the
    /// actual application. Does not spawn work or touch global OS state.
    pub fn handle(&mut self, command: AppCommand) -> Vec<AppAction> {
        use AppAction as A;
        use AppCommand as C;
        use AppPhase as P;
        match (self.phase(), command) {
            (P::Uninitialized, C::Start) => {
                self.phase = P::Running;
                vec![A::InitializeNativeShell]
            }
            (P::Running, C::Pause) => {
                self.phase = P::Paused;
                vec![A::StopSlideshowDeadline]
            }
            (P::Paused, C::Resume) => {
                self.phase = P::Running;
                vec![A::ArmFullSlideshowInterval]
            }
            (P::Running, C::Tick) => vec![A::HandleDueTick],
            (P::Running | P::Paused, C::Reload) => vec![A::RequestValidatedReload],
            (P::Running | P::Paused, C::ShellRestart) => {
                vec![A::ReattachDesktop, A::ReregisterTray]
            }
            (P::Running | P::Paused, C::DisplayChanged) => {
                vec![A::ReconcileDisplaySurfaces]
            }
            (P::Running | P::Paused | P::Uninitialized, C::Quit) => {
                self.phase = P::Stopped;
                vec![A::ReleaseOwnedResources]
            }
            _ => Vec::new(),
        }
    }
}
