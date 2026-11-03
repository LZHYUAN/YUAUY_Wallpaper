//! T010 deterministic contract: refresh rate, max_fps, monotonic schedule and
//! elapsed-time-based animation progress are independent of C# easing formulas.
use std::time::Duration;
use yuauy_wallpaper::pacing::{FramePacer, normalized_progress};

#[test]
fn target_fps_never_exceeds_monitor_refresh_or_yaml_max() {
    for (hz, cap, expected) in [
        (60.0, 120, 60.0),
        (120.0, 120, 120.0),
        (144.0, 120, 120.0),
        (240.0, 120, 120.0),
        (144.0, 60, 60.0),
        (240.0, 240, 240.0),
        (59.94, 120, 59.94),
    ] {
        let pacer = FramePacer::new(hz, cap).unwrap();
        assert_eq!(pacer.target_fps(), expected, "hz={hz}, cap={cap}");
        assert!(pacer.minimum_interval().as_secs_f64() + 1e-15 >= 1.0 / expected);
    }
}

#[test]
fn invalid_configuration_is_rejected_not_silently_changed() {
    for hz in [0.0, -60.0, f64::NAN, f64::INFINITY] {
        assert!(FramePacer::new(hz, 120).is_err());
    }
    assert!(FramePacer::new(60.0, 0).is_err());
}

#[test]
fn pacer_does_not_run_in_background_or_burst_missed_frames() {
    let mut pacer = FramePacer::new(144.0, 120).unwrap();
    assert!(pacer.ready(Duration::ZERO)); // first frame now
    pacer.record_submit(Duration::ZERO);
    assert!(!pacer.ready(Duration::from_millis(5)));
    assert!(pacer.ready(Duration::from_millis(9)));
    // A stall does not cause catch-up bursts. It schedules from actual time.
    pacer.record_submit(Duration::from_secs(3));
    assert!(!pacer.ready(Duration::from_secs(3)));
    assert!(pacer.ready(Duration::from_secs(3) + Duration::from_millis(9)));
}

#[test]
fn animation_uses_real_elapsed_time_not_frame_counter() {
    let duration = Duration::from_millis(2000);
    for hz in [60.0, 120.0, 144.0, 240.0] {
        let pacer = FramePacer::new(hz, 120).unwrap();
        assert_eq!(
            normalized_progress(Duration::from_millis(500), duration),
            0.25
        );
        assert_eq!(
            normalized_progress(Duration::from_millis(1000), duration),
            0.5
        );
        assert_eq!(
            normalized_progress(Duration::from_millis(2000), duration),
            1.0
        );
        assert!(pacer.target_fps() <= hz);
    }
    assert_eq!(normalized_progress(Duration::ZERO, duration), 0.0);
    assert_eq!(
        normalized_progress(Duration::from_millis(9999), duration),
        1.0
    );
}

#[test]
fn zero_duration_finishes_instead_of_dividing_by_zero() {
    assert_eq!(normalized_progress(Duration::ZERO, Duration::ZERO), 1.0);
}
