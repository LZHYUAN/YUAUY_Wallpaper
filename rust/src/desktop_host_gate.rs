//! Evidence-based Windows desktop host acceptance, independent from rendering math.
//!
//! This is a strict product gate, not a probe that declares success from HRESULT.
//! Real WindowsEnv observations must be supplied by synchronized pixel capture,
//! icon-interaction verification and child-window cleanup readback. Synthetic
//! unit tests only prove the policy logic; they do not approve a desktop host.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    BelowIcons,
    AboveIcons,
}

/// A shell-parent relationship independently validated on WindowsEnv.
/// WorkerW must be a real, visible, correctly ordered host, not merely an HWND
/// whose class name matches WorkerW.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostParent {
    Progman,
    ValidatedWorkerW,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompositorEvidence {
    pub layer: Layer,
    pub parent: HostParent,
    pub below_defview: bool,
    pub present_ok: u32,
    /// Number of verified marker pixels in the test ROI *while the renderer runs*.
    /// A controller screenshot after the child has exited is not evidence.
    pub marker_pixels_during: Option<u32>,
    /// Independent frame captures taken while the test window was still alive.
    pub marker_samples: u8,
    /// Size of the measured region expected to show the diagnostic marker.
    pub marker_region_pixels: u32,
    /// Residual marker-color excess vs. a clean desktop reference after
    /// destroying the probe-owned HWND. Do not count legitimate wallpaper
    /// colors as a failure; screenshot timestamps must be verified.
    pub marker_pixels_after_cleanup: Option<u32>,
    /// Some(true) only after comparing visible icon pixels and click-through
    /// behavior; never infer this from WS_EX_TRANSPARENT alone.
    pub icons_preserved: Option<bool>,
    pub own_window_destroyed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateBlocker {
    WrongLayer,
    NoSuccessfulPresent,
    VisualEvidenceMissing,
    InvisibleUnderlay,
    InsufficientMarkerCoverage,
    InsufficientTemporalEvidence,
    IconsNotPreserved,
    CleanupUnverified,
    CleanupVisualMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateDecision {
    Blocked(GateBlocker),
    HostAccepted,
}

/// A host is accepted *only* when all independent observations succeed.
/// The Foundation Gate has additional requirements (T006/T010, etc.).
pub fn decide(evidence: CompositorEvidence) -> GateDecision {
    use GateBlocker as Block;
    if evidence.layer != Layer::BelowIcons
        || !matches!(
            evidence.parent,
            HostParent::Progman | HostParent::ValidatedWorkerW
        )
        || !evidence.below_defview
    {
        return GateDecision::Blocked(Block::WrongLayer);
    }
    if evidence.present_ok == 0 {
        return GateDecision::Blocked(Block::NoSuccessfulPresent);
    }
    let Some(visible_pixels) = evidence.marker_pixels_during else {
        return GateDecision::Blocked(Block::VisualEvidenceMissing);
    };
    if visible_pixels == 0 {
        return GateDecision::Blocked(Block::InvisibleUnderlay);
    }
    // A stray one-pixel color match is not visual proof. Demand visible marker
    // coverage across at least one tenth of the nominated sample region.
    if evidence.marker_region_pixels == 0
        || u64::from(visible_pixels) * 10 < u64::from(evidence.marker_region_pixels)
    {
        return GateDecision::Blocked(Block::InsufficientMarkerCoverage);
    }
    if evidence.marker_samples < 2 {
        return GateDecision::Blocked(Block::InsufficientTemporalEvidence);
    }
    if evidence.icons_preserved != Some(true) {
        return GateDecision::Blocked(Block::IconsNotPreserved);
    }
    if !evidence.own_window_destroyed {
        return GateDecision::Blocked(Block::CleanupUnverified);
    }
    if evidence.marker_pixels_after_cleanup != Some(0) {
        return GateDecision::Blocked(Block::CleanupVisualMismatch);
    }
    GateDecision::HostAccepted
}
