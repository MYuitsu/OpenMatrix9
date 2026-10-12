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
    assert!(!Kind::Loft.options_valid(6, false));
}

#[test]
fn om9_surface_additional_loft_styles_and_closed_sweeps_have_explicit_policies() {
    for style in 0..=5 {
        assert!(Kind::Loft.options_valid(style, false), "Loft style {style}");
    }
    assert!(Kind::Sweep1.options_valid(0, true));
    assert!(Kind::Sweep2.options_valid(0, true));
    assert!(
        !Kind::Loft.options_valid(5, true),
        "Developable builds separate pairs, not a closed loft"
    );
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

#[test]
fn om9_surface_003_height_and_width_are_independent_when_requested() {
    use openmatrix9_rust::surface::transport_with_height;
    for maintain in [false, true] {
        let m = transport_with_height(
            [0., 0., 0.],
            [5., 0., 0.],
            [0., 0., 1.],
            [0., 0., 10.],
            [9., 0., 10.],
            [0., 0., 1.],
            maintain,
        )
        .unwrap();
        assert!((m[0] * 2.5 + m[3] - 4.5).abs() < 1e-12);
        assert!((m[5] * 2. + m[7] - if maintain { 2. } else { 3.6 }).abs() < 1e-12);
        assert_eq!(m[11], 10.);
    }
}

#[test]
fn om9_surface_001_003_chain_is_one_rail_and_undo_does_not_remove_other_inputs() {
    let mut s = Session::new(Kind::Sweep2);
    s.begin_chain().unwrap();
    assert_eq!(s.phase(), Phase::Chain);
    assert!(s.finish_chain(false).is_err());
    s.chain_edge("A.Edge1").unwrap();
    assert!(s.chain_edge("A.Edge1").is_err());
    s.chain_edge("B.Edge2").unwrap();
    s.undo().unwrap();
    s.chain_edge("C.Edge3").unwrap();
    s.finish_chain(false).unwrap();
    assert_eq!(s.keys(), vec!["A.Edge1|C.Edge3"]);
    assert_eq!(s.phase(), Phase::Rail2);
    s.add("otherRail", false).unwrap();
    assert!(s.begin_chain().is_err());
    s.add("profile", false).unwrap();
    s.finish().unwrap();
    s.cancel();
    assert_eq!(s.count(), 0);
}
#[test]
fn uniform_loft_net_interpolates_constant_rows_and_uniform_knots() {
    use openmatrix9_rust::spline::uniform_net_interpolate;
    for n in [2, 3, 6, 9] {
        let points = (0..n)
            .map(|i| [4., (i * i) as f64, (i % 3) as f64])
            .collect::<Vec<_>>();
        let fit = uniform_net_interpolate(&points, false).unwrap();
        for (i, p) in points.iter().enumerate() {
            let actual = fit.value(i as f64 / (n - 1) as f64);
            assert!(actual.iter().zip(p).all(|(a, b)| (a - b).abs() < 1e-8));
        }
        let (knots, _) = fit.host_knots();
        let delta = knots[1] - knots[0];
        assert!(
            knots
                .windows(2)
                .all(|pair| (pair[1] - pair[0] - delta).abs() < 1e-10)
        );
    }
    assert!(uniform_net_interpolate(&[[2., 0., 0.]; 6], false).is_ok());
    let points = (0..7)
        .map(|i| [i as f64, (i % 3) as f64, 1.])
        .collect::<Vec<_>>();
    let fit = uniform_net_interpolate(&points, true).unwrap();
    for (i, p) in points.iter().enumerate() {
        assert!(
            fit.value(i as f64 / 7.)
                .iter()
                .zip(p)
                .all(|(a, b)| (a - b).abs() < 1e-8)
        );
    }
}
#[test]
fn slash_mapping_drives_correspondence_and_rejects_crossings() {
    use openmatrix9_rust::surface::slash_parameter;
    assert_eq!(slash_parameter(&[], 0.3), Some(0.3));
    let pairs = [(0.25, 0.65), (0.75, 0.8)];
    assert!((slash_parameter(&pairs, 0.25).unwrap() - 0.65).abs() < 1e-12);
    assert!((slash_parameter(&pairs, 0.5).unwrap() - 0.725).abs() < 1e-12);
    assert_eq!(slash_parameter(&pairs, 0.), Some(0.));
    assert_eq!(slash_parameter(&pairs, 1.), Some(1.));
    for bad in [
        vec![(0.5, 0.8), (0.6, 0.7)],
        vec![(0.5, 0.5), (0.5, 0.7)],
        vec![(0., 0.2)],
        vec![(0.3, 1.)],
        vec![(f64::NAN, 0.5)],
    ] {
        assert!(slash_parameter(&bad, 0.2).is_none());
    }
    assert!(slash_parameter(&pairs, f64::INFINITY).is_none());
}
