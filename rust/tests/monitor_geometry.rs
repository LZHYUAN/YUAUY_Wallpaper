//! T014: pure test vectors for monitor geometry, stable identity, original
//! C# area-weighted screen selection, and independent surface lifetimes.
use yuauy_wallpaper::monitor::{
    MonitorBounds, MonitorId, MonitorRecord, MonitorTopology, SurfaceLifetime,
};

fn monitor(id: &str, left: i32, top: i32, width: u32, height: u32) -> MonitorRecord {
    MonitorRecord::new(
        MonitorId::new(id).unwrap(),
        MonitorBounds {
            left,
            top,
            width,
            height,
        },
    )
}

#[test]
fn geometry_handles_negative_virtual_desktop_coordinates() {
    let screens = MonitorTopology::new(vec![
        monitor("MON-A", -1920, 0, 1920, 1080),
        monitor("MON-B", 0, -1440, 2560, 1440),
    ])
    .unwrap();
    assert_eq!(screens.records()[0].bounds.center(), (-960, 540));
    assert_eq!(screens.records()[1].bounds.center(), (1280, -720));
    assert_eq!(screens.virtual_bounds(), (-1920, -1440, 2560, 1080));
    assert_eq!(screens.records()[0].bounds.area(), 2_073_600);
}

#[test]
fn area_weighted_selector_matches_csharp_boundary_semantics() {
    let screens = MonitorTopology::new(vec![
        monitor("FIRST", -1920, 0, 1920, 1080),
        monitor("SECOND", 0, 0, 1280, 1024),
    ])
    .unwrap();
    let a = 1920.0_f64 * 1080.0_f64;
    let b = 1280.0_f64 * 1024.0_f64;
    let boundary = a / (a + b);
    assert_eq!(screens.choose_index(0.0).unwrap(), 0);
    assert_eq!(screens.choose_index(boundary).unwrap(), 0); // C# if (st <= 0) break
    assert_eq!(screens.choose_index(0.9999).unwrap(), 1);
}

#[test]
fn stable_ids_reject_duplicates_and_empty_dimensions() {
    assert!(MonitorId::new(" ").is_err());
    assert!(MonitorTopology::new(vec![monitor("A", 0, 0, 0, 1080)]).is_err());
    assert!(
        MonitorTopology::new(vec![
            monitor("A", 0, 0, 1920, 1080),
            monitor("A", 1920, 0, 1920, 1080)
        ])
        .is_err()
    );
    assert!(MonitorTopology::new(Vec::new()).is_err());
}

#[test]
fn invalid_random_draw_is_rejected_instead_of_overrunning_monitors() {
    let screens = MonitorTopology::new(vec![monitor("ONLY", 0, 0, 100, 100)]).unwrap();
    for d in [f64::NAN, f64::INFINITY, -0.1, 1.0] {
        assert!(screens.choose_index(d).is_err());
    }
    assert_eq!(screens.choose_index(0.999999).unwrap(), 0);
}

#[test]
fn monitor_surface_generations_are_independent() {
    let mut screens = MonitorTopology::new(vec![
        monitor("A", 0, 0, 1280, 720),
        monitor("B", 1280, 0, 1920, 1080),
    ])
    .unwrap();
    screens.records_mut()[0].surface = SurfaceLifetime::Attached { generation: 2 };
    assert_eq!(screens.records()[1].surface, SurfaceLifetime::Unbound);
    screens.records_mut()[1].surface = SurfaceLifetime::Attached { generation: 1 };
    screens.records_mut()[0].surface = SurfaceLifetime::Lost { generation: 2 };
    assert_eq!(
        screens.records()[1].surface,
        SurfaceLifetime::Attached { generation: 1 }
    );
}
