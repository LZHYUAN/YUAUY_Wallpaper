//! Platform-free injection seams for later scheduler and image-queue stories.
//! No polling, background task, directory watcher, renderer, or Windows API.

use rand::{RngExt, SeedableRng, rngs::StdRng};
use std::{
    num::NonZeroUsize,
    path::PathBuf,
    time::{Duration, Instant},
};

/// Time since an arbitrary monotonic origin. No wall-clock timestamps.
pub trait MonotonicClock {
    fn elapsed(&self) -> Duration;
}

#[derive(Debug, Clone)]
pub struct SystemMonotonicClock {
    origin: Instant,
}
impl Default for SystemMonotonicClock {
    fn default() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}
impl MonotonicClock for SystemMonotonicClock {
    fn elapsed(&self) -> Duration {
        self.origin.elapsed()
    }
}

/// Uniform integer choice; callers cannot request an empty range.
pub trait RandomIndex {
    fn index(&mut self, exclusive_upper: NonZeroUsize) -> usize;
}

pub struct SeededRandom {
    rng: StdRng,
}
impl SeededRandom {
    /// Deterministic test/fixture seed, not the production entropy policy.
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}
impl RandomIndex for SeededRandom {
    fn index(&mut self, exclusive_upper: NonZeroUsize) -> usize {
        self.rng.random_range(0..exclusive_upper.get())
    }
}

/// A caller-owned, single-shot candidate catalog. T024 adds the real scanner.
/// No directories are read until the caller explicitly invokes this method.
pub trait FileCatalog {
    /// Error messages must be static/redacted: avoid exposing user photo paths.
    fn list_paths(&mut self, folders: &[PathBuf]) -> Result<Vec<PathBuf>, &'static str>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageMetric {
    GpuTotal,
    CpuTotal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsageReading {
    Percent(f32),
    Unknown,
}
impl UsageReading {
    pub fn from_percent(value: f32) -> Self {
        if value.is_finite() && (0.0..=100.0).contains(&value) {
            Self::Percent(value)
        } else {
            Self::Unknown
        }
    }
}

/// The scheduler invokes this only at the normal slideshow deadline, with
/// an explicit per-call time limit. There is no continuous sampler contract.
pub trait TotalUsageSampler {
    fn sample(&mut self, metric: UsageMetric, budget: Duration) -> UsageReading;
}
