use openmatrix9_rust::viewport_title::{MODES, native_mode, next_single};
#[test]
fn viewport_label_toggles_only_the_selected_slot() {
    assert_eq!(next_single(None, 2), Some(Some(2)));
    assert_eq!(next_single(Some(2), 2), Some(None));
    assert_eq!(next_single(Some(2), 1), Some(Some(1)));
    assert_eq!(next_single(None, 4), None);
}
#[test]
fn original_display_names_require_a_real_adapter() {
    assert_eq!(
        &MODES[..6],
        &[
            "Wireframe",
            "Shaded",
            "Rendered",
            "Ghosted",
            "X-Ray",
            "Technical"
        ]
    );
    assert_eq!(native_mode("Wireframe"), Some(1));
    assert_eq!(native_mode("Shaded"), Some(0));
    for name in [
        "Rendered",
        "Ghosted",
        "X-Ray",
        "ClayooSculpt",
        "Plastic",
        "",
    ] {
        assert_eq!(native_mode(name), None);
    }
}
