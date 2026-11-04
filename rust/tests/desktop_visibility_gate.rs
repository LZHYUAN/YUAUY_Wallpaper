//! A successful Present does not certify an Explorer-behind-icons wallpaper.
use yuauy_wallpaper::desktop_host_gate::{
    CompositorEvidence, GateBlocker, GateDecision, HostParent, Layer, decide,
};

fn evidence(
    layer: Layer,
    present_ok: u32,
    changed_pixels: Option<u32>,
    icons_preserved: Option<bool>,
) -> CompositorEvidence {
    CompositorEvidence {
        layer,
        parent: HostParent::Progman,
        below_defview: layer == Layer::BelowIcons,
        present_ok,
        marker_pixels_during: changed_pixels,
        marker_samples: 2,
        marker_region_pixels: 1_000,
        marker_pixels_after_cleanup: Some(0),
        icons_preserved,
        own_window_destroyed: true,
    }
}

#[test]
fn successful_present_without_visible_pixels_must_not_approve_host() {
    let diagnostic = evidence(Layer::BelowIcons, 303, Some(0), Some(true));
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::InvisibleUnderlay)
    );
}

#[test]
fn visible_overlay_that_covers_icons_is_not_wallpaper() {
    let diagnostic = evidence(Layer::AboveIcons, 456, Some(41_800), Some(false));
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::WrongLayer)
    );
}

#[test]
fn missing_pixel_proof_is_a_blocker_not_a_pass() {
    let diagnostic = evidence(Layer::BelowIcons, 303, None, None);
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::VisualEvidenceMissing)
    );
}

#[test]
fn unknown_or_changed_icon_interaction_is_never_approved() {
    let diagnostic = evidence(Layer::BelowIcons, 303, Some(100), Some(false));
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::IconsNotPreserved)
    );
    let diagnostic = evidence(Layer::BelowIcons, 303, Some(100), None);
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::IconsNotPreserved)
    );
}

#[test]
fn a_failed_cleanup_or_wrong_parent_blocks_even_if_pixels_show() {
    let mut diagnostic = evidence(Layer::BelowIcons, 303, Some(100), Some(true));
    diagnostic.own_window_destroyed = false;
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::CleanupUnverified)
    );
    diagnostic.own_window_destroyed = true;
    diagnostic.parent = HostParent::Other;
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::WrongLayer)
    );
}

#[test]
fn approve_only_actual_underlay_with_all_evidence_present() {
    let diagnostic = evidence(Layer::BelowIcons, 303, Some(300), Some(true));
    assert_eq!(decide(diagnostic), GateDecision::HostAccepted);
}

#[test]
fn a_verified_workerw_host_can_qualify_only_with_pixel_and_icon_evidence() {
    let mut diagnostic = evidence(Layer::BelowIcons, 303, Some(300), Some(true));
    diagnostic.parent = HostParent::ValidatedWorkerW;
    assert_eq!(decide(diagnostic), GateDecision::HostAccepted);
    diagnostic.marker_pixels_during = Some(0);
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::InvisibleUnderlay)
    );
}

#[test]
fn one_pixel_and_one_screenshot_cannot_certify_visible_wallpaper() {
    let mut diagnostic = evidence(Layer::BelowIcons, 303, Some(1), Some(true));
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::InsufficientMarkerCoverage)
    );
    diagnostic.marker_pixels_during = Some(300);
    diagnostic.marker_samples = 1;
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::InsufficientTemporalEvidence)
    );
}

#[test]
fn marker_must_disappear_after_own_window_is_destroyed() {
    let mut diagnostic = evidence(Layer::BelowIcons, 303, Some(300), Some(true));
    diagnostic.marker_pixels_after_cleanup = Some(300);
    assert_eq!(
        decide(diagnostic),
        GateDecision::Blocked(GateBlocker::CleanupVisualMismatch)
    );
}

#[test]
fn success_without_d3d11_present_does_not_qualify() {
    assert_eq!(
        decide(evidence(Layer::BelowIcons, 0, Some(300), Some(true))),
        GateDecision::Blocked(GateBlocker::NoSuccessfulPresent)
    );
}
