use openmatrix9_rust::core_mouse::{Action, action, moved, selection_hit};

#[test]
fn rhino_navigation_and_matrix_override_keep_selection_clicks() {
    assert_eq!(action(2, 0, true), Action::Orbit);
    assert_eq!(action(2, 0, false), Action::Pan);
    assert_eq!(action(2, 1, true), Action::Pan);
    assert_eq!(action(2, 2, false), Action::Zoom);
    assert_eq!(action(2, 3, false), Action::Orbit);
    assert_eq!(action(2, 3, true), Action::Zoom);
    assert_eq!(action(1, 2, true), Action::Zoom);
    assert_eq!(action(1, 3, true), Action::Select);
    assert_eq!(action(2, 4, true), Action::Dolly);
    assert_eq!(action(2, 5, true), Action::Tilt);
    assert_eq!(action(2, 6, true), Action::Look);
    assert_eq!(action(8, 0, true), Action::None);
}

#[test]
fn a_click_never_becomes_navigation_and_window_requires_full_containment() {
    assert!(!moved([10., 20.], [12., 21.], 4.));
    assert!(moved([10., 20.], [15., 20.], 4.));
    assert!(!moved([f64::NAN, 20.], [15., 20.], 4.));
    assert!(selection_hit(
        [10., 10., 30., 30.],
        [12., 12., 28., 28.],
        false
    ));
    assert!(!selection_hit(
        [10., 10., 30., 30.],
        [12., 12., 35., 28.],
        false
    ));
    assert!(selection_hit(
        [10., 10., 30., 30.],
        [12., 12., 35., 28.],
        true
    ));
    assert!(!selection_hit(
        [10., 10., 30., 30.],
        [40., 40., 50., 50.],
        true
    ));
}
