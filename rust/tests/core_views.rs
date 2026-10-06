use openmatrix9_rust::core_views::{VIEWS, command};

#[test]
fn matrix_grid_subdivides_major_cells_and_keeps_axes_separate() {
    use openmatrix9_rust::core_views::grid_line_kind;
    assert_eq!(grid_line_kind(0), 2);
    for i in -100..=100 {
        if i != 0 {
            assert_eq!(grid_line_kind(i), if i % 5 == 0 { 1 } else { 0 });
        }
    }
}

#[test]
fn om9_view_006_007_preserve_original_zoom_commands() {
    assert_eq!(command("ViewZoomZoomExtents"), Some("Zoom_Extents"));
    assert_eq!(command("ViewZoomZoomSelected"), Some("Zoom_Selected"));
}

#[test]
fn om9_view_005_preserves_original_capture_command() {
    assert_eq!(command("ViewCaptureToFile"), Some("ViewCaptureToFile"));
}

#[test]
fn om9_viewport_001_four_named_views_have_normalized_frames() {
    assert_eq!(
        VIEWS.map(|v| v.title),
        ["Looking Down", "Perspective", "Side View", "Through Finger"]
    );
    for view in VIEWS {
        let norm: f64 = view.rotation.iter().map(|v| v * v).sum();
        assert!((norm - 1.0).abs() < 1e-5);
    }
    assert!(VIEWS[1].perspective);
    assert!(!VIEWS[0].perspective && !VIEWS[2].perspective && !VIEWS[3].perspective);
    assert_eq!(VIEWS[1].plane_rotation, VIEWS[0].plane_rotation);
    assert_ne!(VIEWS[2].plane_rotation, VIEWS[3].plane_rotation);
}

#[test]
fn om9_view_003_preserves_original_command_name() {
    assert_eq!(
        command("OthersViewSynchronizeViews"),
        Some("SynchronizeViews")
    );
    assert_eq!(command("ViewRestoreViewports"), Some("RestoreViewports"));
    assert_eq!(
        command("ViewSetCameraCenterViewport"),
        Some("CenterViewport")
    );
    assert_eq!(command("ShowGrid"), Some("ShowGrid"));
    assert_eq!(command("CurveLineSingleLine"), None);
}
