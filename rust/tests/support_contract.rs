//! T017 red-green contract for reusable test doubles across all five stories.
mod support;
use std::{path::PathBuf, time::Duration};
use support::{
    RecordingRenderer, SequenceUsageSampler, TempFolderCatalog, VirtualClock, two_monitor_layout,
};
use yuauy_wallpaper::{
    monitor::SurfaceLifetime,
    platform::{FileCatalog, MonotonicClock, TotalUsageSampler, UsageMetric, UsageReading},
    render::{FramePresentation, WallpaperRenderer},
};

#[test]
fn virtual_clock_advances_without_sleeping() {
    let clock = VirtualClock::default();
    assert_eq!(clock.elapsed(), Duration::ZERO);
    clock.advance(Duration::from_secs(30));
    assert_eq!(clock.elapsed(), Duration::from_secs(30));
}

#[test]
fn total_usage_sampler_returns_one_result_per_due_call_only() {
    let mut sampler = SequenceUsageSampler::new([
        UsageReading::Percent(80.0),
        UsageReading::Percent(59.0),
        UsageReading::Unknown,
    ]);
    assert_eq!(sampler.call_count(), 0);
    for expected in [
        UsageReading::Percent(80.0),
        UsageReading::Percent(59.0),
        UsageReading::Unknown,
    ] {
        assert_eq!(
            sampler.sample(UsageMetric::GpuTotal, Duration::from_millis(250)),
            expected
        );
    }
    assert_eq!(sampler.call_count(), 3);
    assert_eq!(
        sampler.sample(UsageMetric::GpuTotal, Duration::from_millis(250)),
        UsageReading::Unknown
    );
}

#[test]
fn temp_folder_catalog_does_not_scan_until_requested() {
    let mut catalog = TempFolderCatalog::default();
    catalog.write_synthetic("sample.jpg", b"fake JPG bytes");
    catalog.write_synthetic("sample.png", b"fake PNG bytes");
    assert_eq!(catalog.scan_count(), 0);
    let paths = catalog.list_paths(&[catalog.root().to_path_buf()]).unwrap();
    assert_eq!(paths.len(), 2);
    assert_eq!(catalog.scan_count(), 1);
    let nothing = catalog
        .list_paths(&[PathBuf::from("non-existing-test-folder")])
        .unwrap();
    assert!(nothing.is_empty());
    assert_eq!(catalog.scan_count(), 2);
}

#[test]
fn mock_monitors_include_negative_origins_and_independent_surfaces() {
    let mut topology = two_monitor_layout();
    assert!(topology.records()[0].bounds.left < 0);
    assert_eq!(topology.records().len(), 2);
    topology.records_mut()[0].surface = SurfaceLifetime::Attached { generation: 1 };
    assert_eq!(topology.records()[1].surface, SurfaceLifetime::Unbound);
}

#[test]
fn recording_renderer_logs_only_explicit_present_calls() {
    let mut rec = RecordingRenderer::default();
    let monitors = two_monitor_layout();
    let key = rec.attach(&monitors.records()[0]).unwrap();
    assert_eq!(rec.present_count(), 0);
    rec.present(
        &key,
        &FramePresentation {
            scene_version: 3,
            elapsed_since_transition_start: Duration::ZERO,
        },
    )
    .unwrap();
    assert_eq!(rec.present_count(), 1);
    rec.detach(&key).unwrap();
    assert_eq!(rec.detach_count(), 1);
}
