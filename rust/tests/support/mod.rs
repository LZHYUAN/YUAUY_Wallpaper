//! Reusable, no-Win32 and no-real-user-files T017 fixtures for the five stories.
//! All interactions are explicit; constructing fixtures does not render frames,
//! query GPU/CPU, scan image folders, or start a background timer.
#![allow(dead_code)] // Intentionally exported for future integration test modules.

use std::{
    cell::Cell,
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use tempfile::TempDir;
use yuauy_wallpaper::{
    error::AppError,
    monitor::{MonitorBounds, MonitorId, MonitorRecord, MonitorTopology},
    platform::{FileCatalog, MonotonicClock, TotalUsageSampler, UsageMetric, UsageReading},
    render::{FramePresentation, PresentOutcome, RenderBackend, SurfaceKey, WallpaperRenderer},
};

#[derive(Default)]
pub struct VirtualClock {
    now: Cell<Duration>,
}
impl VirtualClock {
    pub fn advance(&self, by: Duration) {
        self.now.set(self.now.get().saturating_add(by));
    }
}
impl MonotonicClock for VirtualClock {
    fn elapsed(&self) -> Duration {
        self.now.get()
    }
}

pub struct SequenceUsageSampler {
    remaining: VecDeque<UsageReading>,
    calls: Vec<(UsageMetric, Duration)>,
}
impl SequenceUsageSampler {
    pub fn new(values: impl IntoIterator<Item = UsageReading>) -> Self {
        Self {
            remaining: values.into_iter().collect(),
            calls: Vec::new(),
        }
    }
    pub fn call_count(&self) -> usize {
        self.calls.len()
    }
    pub fn calls(&self) -> &[(UsageMetric, Duration)] {
        &self.calls
    }
}
impl TotalUsageSampler for SequenceUsageSampler {
    fn sample(&mut self, metric: UsageMetric, budget: Duration) -> UsageReading {
        self.calls.push((metric, budget));
        self.remaining.pop_front().unwrap_or(UsageReading::Unknown)
    }
}

pub struct TempFolderCatalog {
    directory: TempDir,
    scans: usize,
}
impl Default for TempFolderCatalog {
    fn default() -> Self {
        Self {
            directory: tempfile::tempdir().expect("temporary synthetic folder"),
            scans: 0,
        }
    }
}
impl TempFolderCatalog {
    pub fn root(&self) -> &Path {
        self.directory.path()
    }
    pub fn scan_count(&self) -> usize {
        self.scans
    }
    pub fn write_synthetic(&self, name: &str, data: &[u8]) {
        // Fixture names must be files, never caller-controlled folder traversal.
        assert_eq!(
            Path::new(name).file_name().and_then(|s| s.to_str()),
            Some(name)
        );
        fs::write(self.root().join(name), data).expect("write synthetic fixture");
    }
}
impl FileCatalog for TempFolderCatalog {
    fn list_paths(&mut self, folders: &[PathBuf]) -> Result<Vec<PathBuf>, &'static str> {
        self.scans += 1;
        let mut found = Vec::new();
        for folder in folders {
            let Ok(entries) = fs::read_dir(folder) else {
                continue;
            };
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|ft| ft.is_file()) {
                    found.push(entry.path());
                }
            }
        }
        found.sort();
        Ok(found)
    }
}

pub fn two_monitor_layout() -> MonitorTopology {
    MonitorTopology::new(vec![
        MonitorRecord::new(
            MonitorId::new("LEFT").expect("stable fixture ID"),
            MonitorBounds {
                left: -1920,
                top: 0,
                width: 1920,
                height: 1080,
            },
        ),
        MonitorRecord::new(
            MonitorId::new("PRIMARY").expect("stable fixture ID"),
            MonitorBounds {
                left: 0,
                top: 0,
                width: 2560,
                height: 1440,
            },
        ),
    ])
    .expect("fixture geometry")
}

#[derive(Default)]
pub struct RecordingRenderer {
    next_generation: u64,
    attached: Vec<SurfaceKey>,
    presented: Vec<(SurfaceKey, FramePresentation)>,
    detached: Vec<SurfaceKey>,
}
impl RecordingRenderer {
    pub fn present_count(&self) -> usize {
        self.presented.len()
    }
    pub fn detach_count(&self) -> usize {
        self.detached.len()
    }
    pub fn frames(&self) -> &[(SurfaceKey, FramePresentation)] {
        &self.presented
    }
}
impl WallpaperRenderer for RecordingRenderer {
    fn backend(&self) -> RenderBackend {
        RenderBackend::Warp
    }
    fn attach(&mut self, monitor: &MonitorRecord) -> Result<SurfaceKey, AppError> {
        self.next_generation += 1;
        let key = SurfaceKey {
            monitor_id: monitor.id.clone(),
            generation: self.next_generation,
        };
        self.attached.push(key.clone());
        Ok(key)
    }
    fn present(
        &mut self,
        surface: &SurfaceKey,
        frame: &FramePresentation,
    ) -> Result<PresentOutcome, AppError> {
        self.presented.push((surface.clone(), *frame));
        Ok(PresentOutcome::Submitted)
    }
    fn detach(&mut self, surface: &SurfaceKey) -> Result<(), AppError> {
        self.detached.push(surface.clone());
        Ok(())
    }
}
