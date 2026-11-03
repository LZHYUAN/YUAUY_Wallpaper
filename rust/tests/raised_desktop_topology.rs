//! A Windows 11 25H2 raised desktop can have top-level WorkerW windows while
//! Progman owns only the SHELLDLL_DefView icon subtree.
use yuauy_wallpaper::workerw_probe_options::{
    AttachTarget, DesktopTopology, Mode, select_target_for_topology,
};

#[test]
fn modern_progman_host_accepts_no_direct_workerw_child() {
    let t = DesktopTopology {
        has_progman: true,
        has_defview_child: true,
        has_workerw_child: false,
        raised_desktop: true,
        same_icon_owner: true,
    };
    assert_eq!(select_target_for_topology(Mode::Inspect, t).unwrap(), None);
    assert_eq!(
        select_target_for_topology(Mode::AttachModern { duration_ms: 7000 }, t).unwrap(),
        Some(AttachTarget::ProgmanLayered)
    );
}

#[test]
fn old_workerw_requires_actual_direct_child_and_unraised_desktop() {
    let valid = DesktopTopology {
        has_progman: true,
        has_defview_child: true,
        has_workerw_child: true,
        raised_desktop: false,
        same_icon_owner: true,
    };
    assert_eq!(
        select_target_for_topology(Mode::Attach { duration_ms: 1000 }, valid).unwrap(),
        Some(AttachTarget::WorkerW)
    );
    let invalid = DesktopTopology {
        has_workerw_child: false,
        ..valid
    };
    assert!(select_target_for_topology(Mode::Attach { duration_ms: 1000 }, invalid).is_err());
}

#[test]
fn modern_mode_rejects_missing_icon_host_or_unsupported_shell() {
    let base = DesktopTopology {
        has_progman: true,
        has_defview_child: true,
        has_workerw_child: false,
        raised_desktop: true,
        same_icon_owner: true,
    };
    for broken in [
        DesktopTopology {
            has_progman: false,
            ..base
        },
        DesktopTopology {
            has_defview_child: false,
            ..base
        },
        DesktopTopology {
            same_icon_owner: false,
            ..base
        },
        DesktopTopology {
            raised_desktop: false,
            ..base
        },
    ] {
        assert!(
            select_target_for_topology(Mode::AttachModern { duration_ms: 1000 }, broken).is_err()
        );
    }
}
