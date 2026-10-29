// SPDX-License-Identifier: LGPL-2.1-or-later
//! Ring rail, jewelry size, profile placement, custom-rail validation, profile merge and
//! cap primitives (`OM9-L3-04-02-001` … `-006`).
use om9_core::{
    Error,
    budget::Limits,
    builder::{BuilderKind, BuilderSession, register_reference_producers},
    curve_ops,
    geometry::{Curve, Geometry},
    model::{Document, Entity},
    producers::ProducerRegistry,
};

fn limits() -> Limits {
    Limits::default()
}

fn circle(radius: f64) -> Curve {
    Curve::Circle {
        center: [0.0, 0.0, 0.0],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    }
}

fn rectangle_rail(width: f64, height: f64) -> Curve {
    let points = [
        [-width / 2.0, -height / 2.0, 0.0],
        [width / 2.0, -height / 2.0, 0.0],
        [width / 2.0, height / 2.0, 0.0],
        [-width / 2.0, height / 2.0, 0.0],
        [-width / 2.0, -height / 2.0, 0.0],
    ];
    Curve::Polycurve {
        segments: points
            .windows(2)
            .map(|pair| Curve::Line {
                start: pair[0],
                end: pair[1],
                domain: [0.0, 1.0],
            })
            .collect(),
        reversed: false,
    }
}

/// A smooth, closed, non-circular rail: a rational quadratic ellipse. The same 9-pole
/// structure as a circle, with different x/y radii, so it exercises a real NURBS path rather
/// than a sampled polyline.
fn ellipse_rail(a: f64, b: f64) -> Curve {
    use om9_core::geometry::NurbsCurve;
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let poles = vec![
        [a, 0.0, 0.0],
        [a, b, 0.0],
        [0.0, b, 0.0],
        [-a, b, 0.0],
        [-a, 0.0, 0.0],
        [-a, -b, 0.0],
        [0.0, -b, 0.0],
        [a, -b, 0.0],
        [a, 0.0, 0.0],
    ];
    Curve::Nurbs(NurbsCurve {
        degree: 2,
        poles,
        weights: vec![1.0, s, 1.0, s, 1.0, s, 1.0, s, 1.0],
        knots: vec![0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0],
        domain: [0.0, 4.0],
        periodic: false,
        reversed: false,
        seam: None,
    })
}

fn bowtie() -> Curve {
    let points = [
        [0.0, 0.0, 0.0],
        [2.0, 2.0, 0.0],
        [2.0, 0.0, 0.0],
        [0.0, 2.0, 0.0],
        [0.0, 0.0, 0.0],
    ];
    Curve::Polycurve {
        segments: points
            .windows(2)
            .map(|pair| Curve::Line {
                start: pair[0],
                end: pair[1],
                domain: [0.0, 1.0],
            })
            .collect(),
        reversed: false,
    }
}

fn registry() -> ProducerRegistry {
    let mut registry = ProducerRegistry::new();
    register_reference_producers(&mut registry).unwrap();
    registry
}

#[test]
fn a_us_ring_size_resolves_through_the_frozen_table_and_an_unknown_locale_is_refused() {
    let document = Document::new("ring-size");
    let registry = registry();
    let mut session = BuilderSession::new(BuilderKind::RingRail, &document, &[]).unwrap();
    session
        .set_field(
            "standard",
            om9_core::request::Value::Choice("US".into()),
        )
        .unwrap();
    session
        .set_field("us_size", om9_core::request::Value::count(8))
        .unwrap();
    session.refresh(&document, &registry).unwrap();
    let summary = session.preview().unwrap().summary.clone();
    assert!(summary.contains("US size 8"), "{summary}");
    assert!(summary.contains("diameter 18.13 mm"), "{summary}");
    assert!(summary.contains("ISO 8653"), "{summary}");
    let Geometry::Curve(Curve::Circle { radius, .. }) = &session.preview().unwrap().geometry else {
        panic!("ring rail circle");
    };
    assert!((*radius - 9.065).abs() < 1.0e-9);

    let mut explicit = BuilderSession::new(BuilderKind::RingRail, &document, &[]).unwrap();
    explicit
        .set_field(
            "diameter_mm",
            om9_core::request::Value::length(om9_core::units::LengthMm::try_new(18.0).unwrap()),
        )
        .unwrap();
    explicit.refresh(&document, &registry).unwrap();
    let Geometry::Curve(Curve::Circle { radius, .. }) = &explicit.preview().unwrap().geometry
    else {
        panic!("explicit rail circle");
    };
    assert_eq!(*radius, 9.0);
    assert!(explicit.preview().unwrap().summary.contains("circumference 56.55"));

    // The Unicode-free command surface reaches the same standard: `UK` is not a verified
    // table and must not be silently approximated with the US one.
    let mut locale = session.clone();
    assert!(
        locale
            .set_field(
                "standard",
                om9_core::request::Value::Choice("UK".into())
            )
            .is_err(),
        "an unverified locale must not be admitted"
    );
}

#[test]
fn a_profile_placer_moves_the_section_along_the_rail_and_rolls_it_without_flipping() {
    let mut document = Document::new("profile-placer");
    let rail = Entity::new("Rail", Geometry::Curve(circle(9.0)), None);
    let rail_id = rail.id;
    document.entities.insert(rail_id, rail);
    let registry = registry();
    let mut session = BuilderSession::new(BuilderKind::RailProfile, &document, &[rail_id]).unwrap();
    session.refresh(&document, &registry).unwrap();
    let center = |session: &BuilderSession| {
        let Geometry::Curve(curve) = &session.preview().unwrap().geometry else {
            panic!("profile curve");
        };
        let points = curve_ops::sample_curve(curve, 9, &limits()).unwrap();
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for point in &points {
            for axis in 0..3 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }
        [
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ]
    };
    let start = center(&session);
    assert!(start[0].abs() > 8.9, "{start:?}");
    session
        .set_field(
            "position",
            om9_core::request::Value::ratio(0.25).unwrap(),
        )
        .unwrap();
    session.refresh(&document, &registry).unwrap();
    let quarter = center(&session);
    assert!(quarter[0].abs() < 1.0e-6 && (quarter[1] - 9.0).abs() < 1.0e-6, "{quarter:?}");
    let Geometry::Curve(straight_curve) = &session.preview().unwrap().geometry else {
        panic!("profile curve");
    };
    let straight_corner = curve_ops::point_at(straight_curve, 0.25, &limits()).unwrap();
    session
        .set_field(
            "roll_deg",
            om9_core::request::Value::angle(om9_core::units::AngleDeg::try_new(90.0).unwrap()),
        )
        .unwrap();
    session.refresh(&document, &registry).unwrap();
    // A roll rotates the section in its own plane: the profile centre stays on the rail but
    // the first sampled corner moves.
    let Geometry::Curve(rolled_curve) = &session.preview().unwrap().geometry else {
        panic!("profile curve");
    };
    let rolled_corner = curve_ops::point_at(rolled_curve, 0.25, &limits()).unwrap();
    let moved = ((rolled_corner[0] - straight_corner[0]).powi(2)
        + (rolled_corner[1] - straight_corner[1]).powi(2)
        + (rolled_corner[2] - straight_corner[2]).powi(2))
    .sqrt();
    assert!(moved > 1.0e-3, "roll must move the section corner: {moved}");
    assert!(session.preview().unwrap().summary.contains("roll 90 deg"));
}

#[test]
fn a_custom_rail_must_be_closed_planar_and_non_self_intersecting() {
    let limits = limits();
    curve_ops::validate_ring_rail(&circle(9.0), 1.0e-6, &limits).unwrap();
    curve_ops::validate_ring_rail(&ellipse_rail(9.0, 6.0), 1.0e-6, &limits).unwrap();
    curve_ops::validate_ring_rail(&rectangle_rail(10.0, 6.0), 1.0e-6, &limits).unwrap();

    let open = Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [5.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    };
    assert_eq!(
        curve_ops::validate_ring_rail(&open, 1.0e-6, &limits),
        Err(Error::Invalid("ring rail not closed"))
    );
    assert_eq!(
        curve_ops::validate_ring_rail(&bowtie(), 1.0e-6, &limits),
        Err(Error::Invalid("ring rail self-intersection"))
    );
}

#[test]
fn profile_sections_merge_when_they_are_connected_and_consistently_wound() {
    let limits = limits();
    let rail = circle(1.0);
    let (head, tail) = curve_ops::split_curve(&rail, 0.5, &limits).unwrap();
    let square = rectangle_rail(2.0, 2.0);
    let reversed = curve_ops::reverse_curve(&square, &limits).unwrap();
    assert_eq!(
        curve_ops::merge_profile_sections(&[square, reversed], 1.0e-6, &limits),
        Err(Error::Invalid("profile section winding"))
    );
    let merged = curve_ops::merge_profile_sections(&[head, tail], 1.0e-6, &limits).unwrap();
    assert!(curve_ops::is_closed(&merged, 1.0e-6, &limits).unwrap());
    assert_eq!(
        curve_ops::merge_profile_sections(std::slice::from_ref(&rail), 1.0e-6, &limits),
        Err(Error::Invalid("profile merge needs two sections"))
    );
}

#[test]
fn a_profile_placer_refuses_a_missing_rail_selection() {
    let document = Document::new("missing-profile-rail");
    assert!(
        BuilderSession::new(BuilderKind::RailProfile, &document, &[]).is_err(),
        "the profile placer needs one rail input"
    );
}

#[test]
fn a_builder_ring_sweep_refuses_an_open_rail_and_accepts_a_custom_closed_one() {
    let mut document = Document::new("custom-rail");
    let rail = Entity::new("Rail", Geometry::Curve(ellipse_rail(9.0, 6.0)), None);
    let rail_id = rail.id;
    document.entities.insert(rail_id, rail);
    let registry = registry();
    // The profile must be placed in the rail frame, exactly as the Profile Placer does.
    let mut placer =
        BuilderSession::new(BuilderKind::RailProfile, &document, &[rail_id]).unwrap();
    placer.refresh(&document, &registry).unwrap();
    let profile = Entity::new("Profile", placer.preview().unwrap().geometry.clone(), None);
    let profile_id = profile.id;
    document.entities.insert(profile_id, profile);
    let mut session =
        BuilderSession::new(BuilderKind::RailSweep, &document, &[rail_id, profile_id]).unwrap();
    session.refresh(&document, &registry).unwrap();
    assert!(matches!(
        session.preview().unwrap().geometry,
        Geometry::Surface(_)
    ));

    let mut open = document.clone();
    open.entities.get_mut(&rail_id).unwrap().geometry = Geometry::Curve(Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 1.0],
    });
    let mut open_session =
        BuilderSession::new(BuilderKind::RailSweep, &open, &[rail_id, profile_id]).unwrap();
    assert_eq!(
        open_session.refresh(&open, &registry),
        Err(Error::Invalid("ring rail not closed"))
    );
}
