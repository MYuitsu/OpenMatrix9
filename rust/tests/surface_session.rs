use openmatrix9_rust::surface::{Kind, Phase, Session};

#[test]
fn om9_surface_001_rails_precede_profiles_and_cancel_discards_state() {
    let mut s = Session::new(Kind::Sweep1);
    assert_eq!(s.phase(), Phase::Rail1);
    assert!(s.finish().is_err());
    s.add("rail", true).unwrap();
    assert_eq!(s.phase(), Phase::Sections);
    assert!(s.finish().is_err());
    s.add("profile", true).unwrap();
    assert!(s.add("rail", true).is_err());
    s.finish().unwrap();
    assert_eq!(s.phase(), Phase::Options);
    assert!(s.add("late", true).is_err());
    s.cancel();
    assert_eq!(s.phase(), Phase::Idle);
    assert_eq!(s.count(), 0);
}

#[test]
fn om9_surface_003_requires_two_distinct_rails_and_matching_profiles() {
    let mut s = Session::new(Kind::Sweep2);
    s.add("inner", true).unwrap();
    assert_eq!(s.phase(), Phase::Rail2);
    assert!(s.add("inner", true).is_err());
    s.add("outer", true).unwrap();
    s.add("a", false).unwrap();
    assert!(s.add("b", true).is_err());
    assert_eq!(s.count(), 3);
    s.finish().unwrap();
}

#[test]
fn om9_surface_009_needs_two_sections_and_preserves_selection_order() {
    let mut s = Session::new(Kind::Loft);
    s.add("second-in-space", false).unwrap();
    assert!(s.finish().is_err());
    s.add("first-in-space", false).unwrap();
    assert_eq!(s.keys(), vec!["second-in-space", "first-in-space"]);
    s.finish().unwrap();
    s.back().unwrap();
    assert_eq!(s.phase(), Phase::Sections);
    s.undo().unwrap();
    assert!(s.finish().is_err());
}

#[test]
fn surface_command_identity_and_options_are_explicit() {
    assert_eq!(Kind::from_name("Sweep1"), Some(Kind::Sweep1));
    assert_eq!(
        Kind::from_name("OM9_SurfaceSweepSweep2Rails"),
        Some(Kind::Sweep2)
    );
    assert_eq!(Kind::from_name("loft"), Some(Kind::Loft));
    assert_eq!(Kind::Sweep1.feature_id(), "OM9-SURFACE-001");
    assert_eq!(Kind::Sweep2.feature_id(), "OM9-SURFACE-003");
    assert_eq!(Kind::Loft.feature_id(), "OM9-SURFACE-009");
    assert!(Kind::Sweep1.options_valid(0, false));
    assert!(!Kind::Sweep1.options_valid(1, false));
    assert!(Kind::Loft.options_valid(1, true));
    assert!(!Kind::Loft.options_valid(2, false));
}

#[test]
fn om9_surface_003_avoids_unsupported_occt_border_contact_with_multiple_sections() {
    assert_eq!(openmatrix9_rust::surface::sweep2_contact(1), 0);
    assert_eq!(openmatrix9_rust::surface::sweep2_contact(2), 1);
    assert_eq!(openmatrix9_rust::surface::sweep2_contact(3), 1);
}

#[test]
fn om9_surface_003_transport_scales_a_section_to_the_two_rails() {
    use openmatrix9_rust::surface::transport;
    let m = transport(
        [0., 0., 0.],
        [5., 0., 0.],
        [0., 0., 1.],
        [0., 0., 10.],
        [9., 0., 10.],
        [0., 0., 1.],
    )
    .unwrap();
    // Original endpoint (5,0,0) maps to the second rail endpoint (9,0,10).
    let point = [m[0] * 5. + m[3], m[4] * 5. + m[7], m[8] * 5. + m[11]];
    assert_eq!(point, [9., 0., 10.]);
    assert!(
        transport(
            [0.; 3],
            [0.; 3],
            [0., 0., 1.],
            [0.; 3],
            [1., 0., 0.],
            [0., 0., 1.]
        )
        .is_none()
    );
}
