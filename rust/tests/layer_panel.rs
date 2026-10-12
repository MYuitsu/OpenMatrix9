use openmatrix9_rust::layers::{self, ObjectAction};

#[test]
fn object_hiding_is_independent_of_layer_state() {
    assert!(layers::selectable_permission(true, false));
    assert!(!layers::selectable_permission(true, true));
    assert!(!layers::selectable_permission(false, false));
    assert!(layers::effective_visible(true, true, false));
    assert!(!layers::effective_visible(false, true, false));
    assert!(!layers::effective_visible(true, false, false));
    assert!(!layers::effective_visible(true, true, true));
    assert!(!layers::effective_selectable(true, true, false, true));
    assert!(layers::effective_selectable(true, true, false, false));
}

#[test]
fn mutation_policy_is_typed_and_locked_assignment_is_rejected() {
    assert!(layers::allows_action(
        ObjectAction::Assign,
        true,
        false,
        false
    ));
    assert!(!layers::allows_action(
        ObjectAction::Assign,
        true,
        true,
        false
    ));
    assert!(!layers::allows_action(
        ObjectAction::Assign,
        true,
        false,
        true
    ));
    assert!(!layers::allows_action(
        ObjectAction::Select,
        false,
        false,
        false
    ));
    assert!(layers::allows_action(ObjectAction::Hide, true, true, true));
}

#[test]
fn lights_are_separate_from_32_model_slots_and_art_restriction_is_explicit() {
    assert_eq!(layers::COUNT, 32);
    assert_eq!(layers::name(33), Some("Lights"));
    assert_eq!(
        layers::parse_selection("Layer=Lights"),
        layers::Selection::Index(33)
    );
    assert_eq!(layers::default_color(33), Some([1.0, 1.0, 1.0]));
    assert!(layers::allows_matrix_art(16));
    assert!(!layers::allows_matrix_art(17));
    assert!(!layers::allows_matrix_art(33));
}

#[test]
fn advanced_names_are_bounded_nonempty_and_do_not_accept_controls() {
    assert!(layers::valid_name(17, "Custom profiles"));
    assert!(layers::valid_name(32, "Đá phụ"));
    assert!(!layers::valid_name(1, "Custom metal"));
    assert!(!layers::valid_name(33, "New lights"));
    for name in ["", "  ", "a\nb", "a\0b"] {
        assert!(!layers::valid_name(17, name));
    }
    assert!(!layers::valid_name(17, &"a".repeat(129)));
}

#[test]
fn group_toggle_shows_mixed_and_hides_fully_visible_groups() {
    assert_eq!(layers::group_target_visible(0b1111, 0b1111), Some(false));
    assert_eq!(layers::group_target_visible(0b0101, 0b1111), Some(true));
    assert_eq!(layers::group_target_visible(0, 0), None);
}
