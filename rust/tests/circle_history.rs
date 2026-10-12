//! OM9-CURVE-005 persistent recipes replay without touching the live command.
use openmatrix9_rust::{
    circle::{Circle, Mode, Size},
    circle_history::{Recipe, replay},
};
fn recipe(mode: Mode) -> Recipe {
    Recipe::new(
        &Circle {
            mode,
            ..Circle::default()
        },
        None,
    )
}
#[test]
fn center_numeric_replays_changed_source_center() {
    let r = Recipe::new(&Circle::default(), Some(4.));
    let result = replay(&r, &[[20., 30., 40.]], None).unwrap();
    assert_eq!(result.plan.center, [20., 30., 40.]);
    assert_eq!(result.plan.radius, 4.);
}
#[test]
fn diameter_pick_replays_live_vertices() {
    let r = recipe(Mode::TwoPoint);
    let result = replay(&r, &[[0.; 3], [0., 0., 10.]], None).unwrap();
    assert_eq!(result.plan.center, [0., 0., 5.]);
    assert_eq!(result.plan.radius, 5.);
    assert!(result.plan.normal[2].abs() < 1e-12);
}
#[test]
fn three_point_radius_rejects_grown_chord() {
    let c = Circle {
        mode: Mode::ThreePoint,
        radius_constraint: Some(5.),
        ..Circle::default()
    };
    let r = Recipe::new(&c, None);
    assert!(replay(&r, &[[-6., 0., 0.], [6., 0., 0.], [0., 10., 0.]], None).is_err());
    let result = replay(&r, &[[-3., 0., 0.], [3., 0., 0.], [0., 10., 0.]], None).unwrap();
    assert!((result.plan.center[1] - 4.).abs() < 1e-12);
}
#[test]
fn fit_replays_spatial_samples_and_deviation() {
    let r = recipe(Mode::FitPoints);
    let result = replay(
        &r,
        &[[1., 2., 6.], [5., 2., 2.], [1., 2., -2.], [-3., 2., 2.]],
        None,
    )
    .unwrap();
    assert!((result.plan.radius - 4.).abs() < 1e-10);
    assert!(result.fit_deviation < 1e-10);
    assert!(replay(&r, &[[0.; 3], [1., 0., 0.], [2., 0., 0.]], None).is_err());
}
#[test]
fn area_picked_radius_and_numeric_radius_keep_distinct_semantics() {
    let c = Circle {
        size: Size::Area,
        ..Circle::default()
    };
    assert_eq!(
        replay(&Recipe::new(&c, None), &[[0.; 3], [4., 0., 0.]], None)
            .unwrap()
            .plan
            .radius,
        4.
    );
    assert_eq!(
        replay(&Recipe::new(&c, Some(2.)), &[[0.; 3]], None)
            .unwrap()
            .plan
            .radius,
        2.
    );
}
#[test]
fn deformable_replay_is_owned_periodic_geometry() {
    let c = Circle {
        deformable: true,
        point_count: 12,
        ..Circle::default()
    };
    let result = replay(&Recipe::new(&c, Some(5.)), &[[1., 2., 3.]], None).unwrap();
    let spline = result.spline.unwrap();
    assert!(spline.periodic);
    assert_eq!(spline.poles.len(), 12);
    assert_eq!(spline.degree, 3);
    assert!(result.approx_deviation < 0.01);
}
#[test]
fn tangent_requires_native_recomputed_plan_and_checks_radius() {
    let c = Circle {
        mode: Mode::Tangent,
        radius_constraint: Some(4.),
        ..Circle::default()
    };
    let r = Recipe::new(&c, None);
    assert!(replay(&r, &[[0.; 3], [1., 0., 0.]], None).is_err());
    assert!(
        replay(
            &r,
            &[[0.; 3], [1., 0., 0.]],
            Some([0., 0., 0., 0., 0., 1., 3.])
        )
        .is_err()
    );
    assert_eq!(
        replay(
            &r,
            &[[0.; 3], [1., 0., 0.]],
            Some([0., 0., 0., 0., 0., 1., 4.])
        )
        .unwrap()
        .plan
        .radius,
        4.
    );
}
#[test]
fn encoded_recipe_roundtrip_and_corrupt_fields_reject() {
    let r = Recipe::new(&Circle::default(), Some(4.));
    let encoded = r.encode();
    assert_eq!(Recipe::decode(&encoded).unwrap().encode(), encoded);
    for (index, value) in [
        (0, 1.),
        (1, 99.),
        (5, 2.),
        (9, f64::NAN),
        (19, 3.5),
        (22, -2.),
    ] {
        let mut corrupt = encoded;
        corrupt[index] = value;
        assert!(Recipe::decode(&corrupt).is_err(), "field {index}");
    }
}
#[test]
fn replay_rejects_extra_picks_and_invalid_frames() {
    assert!(
        replay(
            &recipe(Mode::TwoPoint),
            &[[0.; 3], [1., 0., 0.], [0., 1., 0.]],
            None
        )
        .is_err()
    );
    let mut r = recipe(Mode::Center);
    r.options.axes = Some([[1., 0., 0.], [0., 1., 0.], [0., 0., 2.]]);
    assert!(replay(&r, &[[0.; 3], [1., 0., 0.]], None).is_err());
}
#[test]
fn vertical_pick_and_three_point_replay_keep_spatial_planes() {
    let r = recipe(Mode::Vertical);
    let result = replay(&r, &[[1., 2., 3.], [1., 7., 3.]], None).unwrap();
    assert_eq!(result.plan.radius, 5.);
    assert_eq!(result.plan.normal, [1., 0., 0.]);
    let result = replay(
        &recipe(Mode::ThreePoint),
        &[[10., 5., 0.], [10., 0., 5.], [10., -5., 0.]],
        None,
    )
    .unwrap();
    assert_eq!(result.plan.center, [10., 0., 0.]);
    assert_eq!(result.plan.radius, 5.);
    assert_eq!(result.plan.normal, [1., 0., 0.]);
}
#[test]
fn around_curve_live_center_and_normal_update_picked_radius() {
    let r = recipe(Mode::AroundCurve);
    let result = replay(
        &r,
        &[[0.; 3], [0., 10., 0.]],
        Some([0., 2., 0., 1., 0., 0., 3.]),
    )
    .unwrap();
    assert_eq!(result.plan.center, [0., 2., 0.]);
    assert_eq!(result.plan.radius, 8.);
    assert_eq!(result.plan.normal, [1., 0., 0.]);
}
#[test]
fn ffi_failure_preserves_output_and_success_uses_owned_buffer() {
    use openmatrix9_rust::circle_history::om9_circle_history_replay;
    let config = Recipe::new(&Circle::default(), Some(4.)).encode();
    let p = [1., 2., 3.];
    let mut output = [-9.; 4096];
    assert!(!unsafe {
        om9_circle_history_replay(
            config.as_ptr(),
            p.as_ptr(),
            1025,
            std::ptr::null(),
            output.as_mut_ptr(),
        )
    });
    assert!(output.iter().all(|v| *v == -9.));
    assert!(unsafe {
        om9_circle_history_replay(
            config.as_ptr(),
            p.as_ptr(),
            1,
            std::ptr::null(),
            output.as_mut_ptr(),
        )
    });
    assert_eq!(&output[..7], &[1., 2., 3., 0., 0., 1., 4.]);
    assert_eq!(&output[9..13], &[0.; 4]);
}
#[test]
fn native_direction_vectors_normalize_before_projecting_radius_points() {
    let c = Circle {
        normal: Some([0., 0., 10.]),
        ..Circle::default()
    };
    let result = replay(&Recipe::new(&c, None), &[[0.; 3], [4., 0., 5.]], None).unwrap();
    assert_eq!(result.plan.radius, 4.);
    assert_eq!(result.plan.normal, [0., 0., 1.]);
}
#[test]
fn tangent_vertical_and_first_contact_validate_native_plan() {
    let c = Circle {
        mode: Mode::Tangent,
        tangent_vertical: true,
        radius_constraint: Some(4.),
        ..Circle::default()
    };
    let r = Recipe::new(&c, None);
    let points = [[0., 4., 0.], [4., 0., 0.]];
    assert!(replay(&r, &points, Some([4., 4., 0., 0., 0., 1., 4.])).is_err());
    let c = Circle {
        from_first: true,
        tangent_vertical: false,
        ..c
    };
    let r = Recipe::new(&c, None);
    assert!(replay(&r, &points, Some([4., 4., 0., 0., 0., 1., 4.])).is_ok());
    assert!(replay(&r, &points, Some([4., 5., 0., 0., 0., 1., 4.])).is_err());
    assert!(replay(&r, &points, Some([4., 4., 1., 0., 0., 1., 4.])).is_err());
}
#[test]
fn native_radius_measurement_is_bounded_and_finite() {
    use openmatrix9_rust::circle_history::om9_circle_history_radius;
    let anchor = [3., 0., 0.];
    let endpoint = [3., 5., 0.];
    assert_eq!(
        unsafe { om9_circle_history_radius(anchor.as_ptr(), endpoint.as_ptr()) },
        5.
    );
    assert!(unsafe { om9_circle_history_radius(anchor.as_ptr(), anchor.as_ptr()) }.is_nan());
    assert!(unsafe { om9_circle_history_radius(std::ptr::null(), endpoint.as_ptr()) }.is_nan());
}
#[test]
fn asymmetric_fit_deformable_replay_checks_relative_approximation() {
    let c = Circle {
        mode: Mode::FitPoints,
        deformable: true,
        point_count: 12,
        ..Circle::default()
    };
    let diagonal = 7. / 2f64.sqrt();
    let points = [
        [7., 0., 10.],
        [0., 7., 10.],
        [-7., 0., 10.],
        [0., -7., 10.],
        [diagonal, diagonal, 10.],
    ];
    let result = replay(&Recipe::new(&c, None), &points, None).unwrap();
    println!(
        "Fit R7 center={:?} normal={:?} radius={} approx={} ratio={}",
        result.plan.center,
        result.plan.normal,
        result.plan.radius,
        result.approx_deviation,
        result.approx_deviation / result.plan.radius
    );
    assert!((result.plan.radius - 7.).abs() < 1e-10);
    assert!(result.approx_deviation / result.plan.radius < 0.002);
    let mut session = openmatrix9_rust::curve::CurveSession::default();
    session.start("Circle").unwrap();
    for input in ["FitPoints", "Deformable=Yes", "PointCount=12"] {
        session.input(input).unwrap();
    }
    session.circle_fit_batch(&points).unwrap();
    session.input("").unwrap();
    assert!((session.circle_deviations().1 - result.approx_deviation).abs() < 1e-12);
    let live = session.spline().unwrap();
    let restored = result.spline.as_ref().unwrap();
    for i in 0..1024 {
        assert!(
            openmatrix9_rust::spline::distance(
                live.value(i as f64 / 1024.),
                restored.value(i as f64 / 1024.)
            ) < 1e-12
        );
    }
}
