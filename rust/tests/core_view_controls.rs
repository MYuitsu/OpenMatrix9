use openmatrix9_rust::core_view_controls::{Drag, Outcome, Tool, apparent_height};

#[test]
fn dynamic_zoom_preserves_projection_scale_and_bounds() {
    use openmatrix9_rust::core_view_controls::scaled_value;
    let angle = std::f64::consts::FRAC_PI_4;
    let zoomed = scaled_value(true, angle, 0.5).unwrap();
    let before = apparent_height(100.0, angle).unwrap();
    let after = apparent_height(100.0, zoomed).unwrap();
    assert!((after / before - 0.5).abs() < 1e-12);
    assert_eq!(scaled_value(false, 50.0, 0.5), Some(25.0));
    assert_eq!(scaled_value(false, 1e9, 100.0), Some(1e9));
    assert_eq!(scaled_value(false, 1e-7, 0.001), Some(1e-7));
    for factor in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(scaled_value(false, 50.0, factor).is_none());
        assert!(scaled_value(true, angle, factor).is_none());
    }
    assert!(scaled_value(true, std::f64::consts::PI, 1.0).is_none());
}

#[test]
fn om9_view_003_perspective_focal_plane_scale_is_finite() {
    assert!(
        (apparent_height(100.0, std::f64::consts::FRAC_PI_4).unwrap() - 82.842712474619).abs()
            < 1e-9
    );
    for (distance, angle) in [
        (0.0, 1.0),
        (-1.0, 1.0),
        (1.0, 0.0),
        (1.0, std::f64::consts::PI),
        (f64::NAN, 1.0),
    ] {
        assert!(apparent_height(distance, angle).is_none());
    }
}
#[test]
fn om9_view_008_window_accepts_reverse_drag_and_rejects_tiny_pick() {
    let mut drag = Drag::new(Tool::Window);
    assert_eq!(drag.release([1.0, 1.0]), Outcome::Waiting);
    drag.press([100.0, 80.0]).unwrap();
    assert_eq!(
        drag.release([20.0, 30.0]),
        Outcome::Window([20.0, 30.0, 100.0, 80.0])
    );
    assert!(!drag.active());
    let mut drag = Drag::new(Tool::Window);
    drag.press([10.0, 10.0]).unwrap();
    assert_eq!(drag.release([11.0, 11.0]), Outcome::Waiting);
    assert!(drag.active());
}
#[test]
fn om9_view_004_incremental_dynamic_drag_and_cancel() {
    let mut drag = Drag::new(Tool::Dynamic);
    drag.press([10.0, 10.0]).unwrap();
    assert_eq!(
        drag.motion([10.0, 110.0], 100.0).unwrap(),
        Outcome::Scale(2.0)
    );
    assert_eq!(
        drag.motion([10.0, 10.0], 100.0).unwrap(),
        Outcome::Scale(0.5)
    );
    drag.cancel();
    assert_eq!(drag.release([10.0, 20.0]), Outcome::Waiting);
    assert!(!drag.active());
}
#[test]
fn invalid_drag_does_not_corrupt_session() {
    let mut drag = Drag::new(Tool::Dynamic);
    assert!(drag.press([f64::NAN, 0.0]).is_err());
    drag.press([0.0, 0.0]).unwrap();
    assert!(drag.motion([0.0, 10.0], 0.0).is_err());
    assert_eq!(
        drag.motion([0.0, 100.0], 100.0).unwrap(),
        Outcome::Scale(2.0)
    );
}
