//! T009 red tests: inspecting the desktop must be the default and mutating
//! a probe window must require an explicit, bounded, one-shot opt-in.
use yuauy_wallpaper::workerw_probe_options::{Mode, parse_args};

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
