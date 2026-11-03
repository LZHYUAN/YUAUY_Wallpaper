//! T012 contract: the core works with injected monotonic time, RNG, catalog,
//! and one-shot load probes instead of polling production services.
use std::{cell::Cell, num::NonZeroUsize, path::PathBuf, time::Duration};
use yuauy_wallpaper::platform::{
    FileCatalog, MonotonicClock, RandomIndex, SeededRandom, SystemMonotonicClock,
    TotalUsageSampler, UsageMetric, UsageReading,
};

struct VirtualClock(Cell<Duration>);
impl MonotonicClock for VirtualClock {
    fn elapsed(&self) -> Duration {
        self.0.get()
    }
}

#[test]
fn monotonic_clock_is_injectable_without_wall_time() {
    let clock = VirtualClock(Cell::new(Duration::ZERO));
    assert_eq!(clock.elapsed(), Duration::ZERO);
    clock.0.set(Duration::from_secs(31));
    assert_eq!(clock.elapsed(), Duration::from_secs(31));
    let physical = SystemMonotonicClock::default();
    assert!(physical.elapsed() <= Duration::from_secs(1));
}

#[test]
fn seeded_rng_has_reproducible_bounded_draws() {
    let mut a = SeededRandom::from_seed(0x1209);
    let mut b = SeededRandom::from_seed(0x1209);
    let size = NonZeroUsize::new(11).unwrap();
    let left: Vec<_> = (0..30).map(|_| a.index(size)).collect();
    let right: Vec<_> = (0..30).map(|_| b.index(size)).collect();
    assert_eq!(left, right);
    assert!(left.iter().all(|&i| i < size.get()));
}

struct RecordingCatalog {
    scans: Cell<u32>,
}
impl FileCatalog for RecordingCatalog {
    fn list_paths(&mut self, folders: &[PathBuf]) -> Result<Vec<PathBuf>, &'static str> {
        self.scans.set(self.scans.get() + 1);
        Ok(folders.to_vec())
    }
}

#[test]
fn injected_file_catalog_does_not_scan_implicitly() {
    let mut catalog = RecordingCatalog {
        scans: Cell::new(0),
    };
    assert_eq!(catalog.scans.get(), 0);
    let folders = vec![PathBuf::from("fixtures")];
    assert_eq!(catalog.list_paths(&folders).unwrap(), folders);
    assert_eq!(catalog.scans.get(), 1);
}

struct OneShotSampler {
    calls: Vec<(UsageMetric, Duration)>,
}
impl TotalUsageSampler for OneShotSampler {
    fn sample(&mut self, metric: UsageMetric, budget: Duration) -> UsageReading {
        self.calls.push((metric, budget));
        UsageReading::from_percent(59.9)
    }
}

#[test]
fn sampler_is_called_only_when_requested_with_bounded_budget() {
    let mut sampler = OneShotSampler { calls: Vec::new() };
    assert!(sampler.calls.is_empty());
    assert_eq!(
        sampler.sample(UsageMetric::CpuTotal, Duration::from_millis(250)),
        UsageReading::Percent(59.9)
    );
    assert_eq!(
        sampler.calls,
        [(UsageMetric::CpuTotal, Duration::from_millis(250))]
    );
}

#[test]
fn invalid_utilization_reading_must_never_mean_zero() {
    for value in [-1.0, 100.01, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(UsageReading::from_percent(value), UsageReading::Unknown);
    }
    assert_eq!(UsageReading::from_percent(0.0), UsageReading::Percent(0.0));
    assert_eq!(
        UsageReading::from_percent(100.0),
        UsageReading::Percent(100.0)
    );
}
