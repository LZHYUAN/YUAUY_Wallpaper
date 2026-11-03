//! Monotonic frame submission limiter, separate from animation equations.
//!
//! VSync remains the DXGI Present(sync_interval=1) mechanism on Windows. This
//! limiter supplies an additional software deadline for high-refresh monitors
//! (e.g. 144 Hz output with max_fps=120). It does NOT perform GPU Present.
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacingError {
    InvalidRefreshRate,
    InvalidFpsCap,
}

#[derive(Debug, Clone)]
pub struct FramePacer {
    target_fps: f64,
    minimum_interval: Duration,
    last_submit_at: Option<Duration>,
}

impl FramePacer {
    pub fn new(refresh_hz: f64, max_fps: u32) -> Result<Self, PacingError> {
        if !refresh_hz.is_finite() || refresh_hz <= 0.0 {
            return Err(PacingError::InvalidRefreshRate);
        }
        if max_fps == 0 {
            return Err(PacingError::InvalidFpsCap);
        }
        let target_fps = refresh_hz.min(f64::from(max_fps));
        let nanos = (1_000_000_000_f64 / target_fps).ceil();
        if !nanos.is_finite() || nanos > u64::MAX as f64 {
            return Err(PacingError::InvalidRefreshRate);
        }
        // ceil ensures the software limiter cannot exceed target FPS even with
        // fractional refresh rates; zero-nanosecond periods are forbidden.
        let minimum_interval = Duration::from_nanos((nanos as u64).max(1));
        Ok(Self {
            target_fps,
            minimum_interval,
            last_submit_at: None,
        })
    }
    pub fn target_fps(&self) -> f64 {
        self.target_fps
    }
    pub fn minimum_interval(&self) -> Duration {
        self.minimum_interval
    }
    pub fn next_deadline(&self) -> Option<Duration> {
        self.last_submit_at
            .map(|at| at.saturating_add(self.minimum_interval))
    }
    pub fn ready(&self, now: Duration) -> bool {
        self.next_deadline().is_none_or(|due| now >= due)
    }
    /// Call after submitting a frame. Missed slots are not compensated;
    /// no queued redraws or catch-up bursts are introduced.
    pub fn record_submit(&mut self, now: Duration) {
        self.last_submit_at = Some(now);
    }
}

/// Evaluate interpolation progress from a monotonic elapsed duration, never a
/// rendered-frame count. The legacy transition easing is NOT defined here.
pub fn normalized_progress(elapsed: Duration, duration: Duration) -> f64 {
    if duration.is_zero() {
        return 1.0;
    }
    (elapsed.as_secs_f64() / duration.as_secs_f64()).clamp(0.0, 1.0)
}
