//! T009 red tests: inspecting the desktop must be the default and mutating
//! a probe window must require an explicit, bounded, one-shot opt-in.
use yuauy_wallpaper::workerw_probe_options::{AttachTarget, Mode, parse_args, select_target};

#[test]
fn inspect_by_default_no_desktop_mutations() {
    assert_eq!(parse_args([] as [&str; 0]).unwrap(), Mode::Inspect);
    assert_eq!(parse_args(["--inspect"]).unwrap(), Mode::Inspect);
}

#[test]
fn explicit_attachment_has_short_finite_duration() {
    assert_eq!(
        parse_args(["--attach", "--duration-ms", "1500"]).unwrap(),
        Mode::Attach { duration_ms: 1500 }
    );
    assert_eq!(
        parse_args(["--attach"]).unwrap(),
        Mode::Attach { duration_ms: 3000 }
    );
}

#[test]
fn rejects_unbounded_and_unsafe_options() {
    for args in [
        vec!["--duration-ms", "2000"],
        vec!["--attach", "--duration-ms", "0"],
        vec!["--attach", "--duration-ms", "10001"],
        vec!["--attach", "--duration-ms", "nan"],
        vec!["--attach", "--request-workerw"],
        vec!["--attach", "--attach"],
        vec!["--inspect", "--attach"],
        vec!["--anything"],
    ] {
        assert!(parse_args(args.clone()).is_err(), "accepted {args:?}");
    }
}

#[test]
fn new_raised_desktop_requires_explicit_layered_progman_test() {
    let modern = Mode::AttachModern { duration_ms: 3000 };
    assert_eq!(parse_args(["--attach-modern"]).unwrap(), modern);
    assert_eq!(
        select_target(modern, true).unwrap(),
        Some(AttachTarget::ProgmanLayered)
    );
    assert!(select_target(modern, false).is_err());
    assert!(select_target(Mode::Attach { duration_ms: 3000 }, true).is_err());
}

#[test]
fn legacy_workerw_child_is_not_silently_used_on_raised_shell() {
    let legacy = Mode::Attach { duration_ms: 250 };
    assert_eq!(
        select_target(legacy, false).unwrap(),
        Some(AttachTarget::WorkerW)
    );
    assert_eq!(select_target(Mode::Inspect, true).unwrap(), None);
    assert_eq!(select_target(Mode::Inspect, false).unwrap(), None);
    assert_eq!(
        parse_args(["--attach-modern", "--duration-ms", "5000"]).unwrap(),
        Mode::AttachModern { duration_ms: 5000 }
    );
    assert!(parse_args(["--attach-modern", "--duration-ms", "10001"]).is_err());
    assert!(parse_args(["--attach-modern", "--request-workerw"]).is_err());
}
