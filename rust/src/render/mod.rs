//! Object-safe, platform-independent D3D11 renderer contract.
//!
//! The original C# easing, placement and slideshow selection formulas are
//! deliberately NOT implemented or modified here. Per-monitor surfaces keep
//! separate ownership; all story workflows may share an image queue.

use crate::{
    error::AppError,
    monitor::{MonitorId, MonitorRecord},
};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    Hardware,
    Warp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceKey {
    pub monitor_id: MonitorId,
    pub generation: u64,
}

/// Carries monotonic transition time, not a frame index. The production
/// animation math will evaluate the unchanged C# reference separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramePresentation {
    pub scene_version: u64,
    pub elapsed_since_transition_start: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentOutcome {
    Submitted,
    Skipped,
}

pub trait WallpaperRenderer {
    fn backend(&self) -> RenderBackend;
    fn attach(&mut self, monitor: &MonitorRecord) -> Result<SurfaceKey, AppError>;
    fn present(
        &mut self,
        surface: &SurfaceKey,
        frame: &FramePresentation,
    ) -> Result<PresentOutcome, AppError>;
    fn detach(&mut self, surface: &SurfaceKey) -> Result<(), AppError>;
}

/// Windows-only handles. These are *owned* Win32/D3D11/DXGI resources, not
/// adopted WorkerW/Progman handles belonging to Explorer.
#[cfg(windows)]
pub mod native {
    use crate::error::AppError;
    use std::{marker::PhantomData, rc::Rc};
    use windows::Win32::{
        Foundation::HWND,
        Graphics::{
            Direct3D11::{ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView},
            Dxgi::IDXGISwapChain,
        },
        UI::WindowsAndMessaging::{DestroyWindow, IsWindow},
    };

    /// Owns an HWND created by this application's thread. Not Send/Sync.
    pub struct OwnedWin32Window {
        hwnd: HWND,
        _thread_affinity: PhantomData<Rc<()>>,
    }
    impl OwnedWin32Window {
        /// # Safety
        /// Caller created this exact HWND in the current thread, owns it
        /// exclusively, and must not pass Explorer-owned WorkerW/Progman HWNDs.
        pub unsafe fn from_new_owned_hwnd(hwnd: HWND) -> Result<Self, AppError> {
            // SAFETY: read-only validation on the caller-provided HWND.
            if !unsafe { IsWindow(Some(hwnd)).as_bool() } {
                return Err(AppError::DesktopUnavailable);
            }
            Ok(Self {
                hwnd,
                _thread_affinity: PhantomData,
            })
        }
        pub fn hwnd(&self) -> HWND {
            self.hwnd
        }
    }
    impl Drop for OwnedWin32Window {
        fn drop(&mut self) {
            // SAFETY: the !Send marker prevents moving this guard to another
            // thread. Destroy ONLY this application's window, never its parent.
            unsafe {
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }

    /// Field declaration order drops GPU COM handles before the owned HWND.
    /// Every COM interface uses the windows crate's AddRef/Release RAII.
    pub struct OwnedD3d11Surface {
        pub target: ID3D11RenderTargetView,
        pub swap_chain: IDXGISwapChain,
        pub context: ID3D11DeviceContext,
        pub device: ID3D11Device,
        pub window: OwnedWin32Window,
    }
}
