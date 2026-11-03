//! T015: object-safe renderer seam allows per-monitor surfaces to share one
//! composition pipeline without owning or changing the C# animation math.
use std::time::Duration;
use yuauy_wallpaper::{
    error::AppError,
    monitor::{MonitorBounds, MonitorId, MonitorRecord},
    render::{FramePresentation, PresentOutcome, RenderBackend, SurfaceKey, WallpaperRenderer},
};

#[derive(Default)]
struct Recorder {
    attached: Vec<SurfaceKey>,
    submitted: Vec<(SurfaceKey, u64)>,
    released: Vec<SurfaceKey>,
}

impl WallpaperRenderer for Recorder {
    fn backend(&self) -> RenderBackend {
        RenderBackend::Warp
    }
    fn attach(&mut self, record: &MonitorRecord) -> Result<SurfaceKey, AppError> {
        let key = SurfaceKey {
            monitor_id: record.id.clone(),
            generation: self.attached.len() as u64 + 1,
        };
        self.attached.push(key.clone());
        Ok(key)
    }
    fn present(
        &mut self,
        surface: &SurfaceKey,
        frame: &FramePresentation,
    ) -> Result<PresentOutcome, AppError> {
        self.submitted.push((surface.clone(), frame.scene_version));
        Ok(PresentOutcome::Submitted)
    }
    fn detach(&mut self, surface: &SurfaceKey) -> Result<(), AppError> {
        self.released.push(surface.clone());
        Ok(())
    }
}

fn monitor(id: &str) -> MonitorRecord {
    MonitorRecord::new(
        MonitorId::new(id).unwrap(),
        MonitorBounds {
            left: 0,
            top: 0,
            width: 1920,
            height: 1080,
        },
    )
}

#[test]
fn trait_is_object_safe_and_backend_is_explicit() {
    let mut recorder = Recorder::default();
    let renderer: &mut dyn WallpaperRenderer = &mut recorder;
    assert_eq!(renderer.backend(), RenderBackend::Warp);
    assert_ne!(renderer.backend(), RenderBackend::Hardware);
    let key = renderer.attach(&monitor("MON-1")).unwrap();
    assert_eq!(key.monitor_id.as_str(), "MON-1");
    let request = FramePresentation {
        scene_version: 1,
        elapsed_since_transition_start: Duration::from_millis(500),
    };
    assert_eq!(
        renderer.present(&key, &request).unwrap(),
        PresentOutcome::Submitted
    );
    renderer.detach(&key).unwrap();
    assert_eq!(recorder.submitted.len(), 1);
    assert_eq!(recorder.released.len(), 1);
}

#[test]
fn surface_keys_are_monitor_scoped_and_do_not_share_lifetimes() {
    let mut rec = Recorder::default();
    let (a, b);
    {
        let render: &mut dyn WallpaperRenderer = &mut rec;
        a = render.attach(&monitor("A")).unwrap();
        b = render.attach(&monitor("B")).unwrap();
        assert_ne!(a, b);
        render.detach(&a).unwrap();
        render
            .present(
                &b,
                &FramePresentation {
                    scene_version: 5,
                    elapsed_since_transition_start: Duration::ZERO,
                },
            )
            .unwrap();
    }
    assert_eq!(rec.released, vec![a]);
    assert_eq!(rec.submitted[0].0, b);
}

#[test]
fn scene_request_contains_elapsed_time_not_frame_count() {
    let p = FramePresentation {
        scene_version: 7,
        elapsed_since_transition_start: Duration::from_millis(750),
    };
    assert_eq!(p.elapsed_since_transition_start, Duration::from_millis(750));
    assert_eq!(p.scene_version, 7);
}
