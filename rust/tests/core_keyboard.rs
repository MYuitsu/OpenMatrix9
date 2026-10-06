use openmatrix9_rust::{core_keyboard, curve::CurveSession};

#[test]
fn original_shortcuts_are_unique_and_keep_matrix_targets() {
    let map = core_keyboard::SHORTCUTS;
    for (key, target) in [
        ("F2", "CommandHistory"),
        ("F4", "@origin"),
        ("F5", "CenterViewport"),
        ("F6", "F6"),
        ("F7", "@grid"),
        ("F8", "Ortho"),
        ("F10", "PointsOn"),
        ("Ctrl+T", "Properties"),
        ("Ctrl+Q", "Group"),
        ("Ctrl+W", "UnGroup"),
        ("Ctrl+Alt+C", "gvCenterObjects"),
    ] {
        assert!(map.contains(&(key, target)), "{key}");
    }
    let keys: std::collections::HashSet<_> = map.iter().map(|entry| entry.0).collect();
    assert_eq!(keys.len(), map.len());
    for (i, (key, target)) in map.iter().enumerate() {
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(core_keyboard::om9_keyboard_key(i)) }
                .to_str()
                .unwrap(),
            *key
        );
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(core_keyboard::om9_keyboard_target(i)) }
                .to_str()
                .unwrap(),
            *target
        );
    }
    assert!(core_keyboard::om9_keyboard_key(map.len()).is_null());
    assert!(core_keyboard::om9_keyboard_target(map.len()).is_null());
}

#[test]
fn shift_temporarily_inverts_ortho_without_changing_saved_mode() {
    assert!(core_keyboard::ortho_active(true, false, true));
    assert!(!core_keyboard::ortho_active(true, true, true));
    assert!(core_keyboard::ortho_active(false, true, true));
    assert!(!core_keyboard::ortho_active(false, true, false));
}

#[test]
fn mouse_ortho_uses_cplane_axes_and_leaves_typed_points_exact() {
    let mut session = CurveSession::default();
    session.start("Line").unwrap();
    session.point([0., 0., 0.]).unwrap();
    session.mouse_point([8., 3., 0.], true).unwrap();
    assert_eq!(session.output(), &[[0., 0., 0.], [8., 0., 0.]]);
    session.start("Line").unwrap();
    session
        .set_frame([0., 0., 0.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]])
        .unwrap();
    session.point([0., 0., 0.]).unwrap();
    session.mouse_point([0., 3., 8.], true).unwrap();
    assert_eq!(session.output(), &[[0., 0., 0.], [0., 0., 8.]]);
    session.start("Line").unwrap();
    session.point([0., 0., 0.]).unwrap();
    session.point([8., 3., 0.]).unwrap();
    assert_eq!(session.output(), &[[0., 0., 0.], [8., 3., 0.]]);
}
