// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-03 wave W3: surface kernel ported from the legacy bridge.
//!
//! The oracles are analytic: a plane is exactly a plane, a cylinder's points stay at the
//! radius and its normals stay radial, an iso-curve is exact, and a refit may only
//! approximate its source.
use om9_core::{
    Error,
    budget::Limits,
    curve_build::{KnotMode, interpolate},
    evaluation::{Parameter, evaluate_curve},
    geometry::{Curve, NurbsSurface, Point3, Transform},
    surface_kernel::{
        SurfaceIso, Sweep2Options, arc_length_stations, cylinder, derivatives, evaluate, isocurve,
        loft, normal, path_frames, plane, refit, sweep1, sweep2,
    },
};

fn limits() -> Limits {
    Limits::default()
}

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() <= tolerance)
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn plane_fixture() -> NurbsSurface {
    plane(
        [10.0, 20.0, 30.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        4.0,
        2.0,
        &limits(),
    )
    .unwrap()
}

fn cylinder_fixture() -> NurbsSurface {
    cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
        8.0,
        &limits(),
    )
    .unwrap()
}

/* ------------------------- evaluation ------------------------- */

#[test]
fn a_plane_evaluates_to_its_own_frame() {
    let surface = plane_fixture();
    // Corner and centre points, exactly on the frame.
    assert!(close(
        evaluate(&surface, 0.0, 0.0, &limits()).unwrap(),
        [10.0, 20.0, 30.0],
        1e-12
    ));
    assert!(close(
        evaluate(&surface, 1.0, 1.0, &limits()).unwrap(),
        [10.0, 24.0, 32.0],
        1e-12
    ));
    assert!(close(
        evaluate(&surface, 0.5, 0.5, &limits()).unwrap(),
        [10.0, 22.0, 31.0],
        1e-12
    ));
    // The normal is the frame normal everywhere, and the tangents are the frame axes.
    for (u, v) in [(0.0, 0.0), (0.3, 0.7), (1.0, 1.0)] {
        let d = derivatives(&surface, u, v, &limits()).unwrap();
        assert!(close(d.du, [0.0, 4.0, 0.0], 1e-12), "{u},{v}: {:?}", d.du);
        assert!(close(d.dv, [0.0, 0.0, 2.0], 1e-12), "{u},{v}: {:?}", d.dv);
        assert!(close(
            normal(&surface, u, v, &limits()).unwrap(),
            [1.0, 0.0, 0.0],
            1e-12
        ));
    }
}

#[test]
fn a_cylinder_stays_at_its_radius_and_normal() {
    let surface = cylinder_fixture();
    for step in 0..=12 {
        let u = step as f64 / 12.0;
        for vstep in 0..=4 {
            let v = vstep as f64 / 4.0;
            let point = evaluate(&surface, u, v, &limits()).unwrap();
            let radial = (point[0] * point[0] + point[1] * point[1]).sqrt();
            assert!((radial - 5.0).abs() < 1e-9, "u={u} v={v}: radius {radial}");
            assert!(
                (point[2] - 8.0 * v).abs() < 1e-9,
                "u={u} v={v}: height {:?}",
                point[2]
            );
            // The normal is radial and unit length: no component along the axis.
            let n = normal(&surface, u, v, &limits()).unwrap();
            assert!((dot(n, n) - 1.0).abs() < 1e-9);
            assert!(n[2].abs() < 1e-9, "u={u} v={v}: normal {n:?}");
            assert!(close(n, [point[0] / 5.0, point[1] / 5.0, 0.0], 1e-9));
        }
    }
}

#[test]
fn a_surface_parameter_outside_the_domain_is_refused() {
    let surface = plane_fixture();
    assert_eq!(
        evaluate(&surface, -0.1, 0.5, &limits()),
        Err(Error::Invalid("surface parameter"))
    );
    assert_eq!(
        evaluate(&surface, 0.5, 1.1, &limits()),
        Err(Error::Invalid("surface parameter"))
    );
    assert_eq!(
        evaluate(&surface, f64::NAN, 0.5, &limits()),
        Err(Error::Invalid("surface parameter"))
    );
}

#[test]
fn a_degenerate_surface_has_no_normal() {
    // A surface whose v direction collapses to a point: S_u and S_v are parallel at v=0.
    let mut surface = plane_fixture();
    // Fold both v rows onto the same line so S_v becomes parallel to S_u.
    surface.poles[0] = [10.0, 20.0, 30.0];
    surface.poles[1] = [10.0, 20.0, 30.0];
    surface.poles[2] = [10.0, 24.0, 30.0];
    surface.poles[3] = [10.0, 24.0, 30.0];
    let d = derivatives(&surface, 0.5, 0.5, &limits()).unwrap();
    assert!(dot(d.dv, d.dv) < 1e-24, "S_v must vanish here: {:?}", d.dv);
    assert_eq!(
        normal(&surface, 0.5, 0.5, &limits()),
        Err(Error::Invalid("degenerate surface normal"))
    );
}

/* ------------------------- iso-curves ------------------------- */

#[test]
fn an_iso_curve_of_a_plane_is_an_exact_line() {
    let surface = plane_fixture();
    // Vary u, hold v: a horizontal line of length 4.
    let horizontal = isocurve(&surface, SurfaceIso::U, 0.5, &limits()).unwrap();
    assert_eq!(horizontal.degree, 1);
    assert_eq!(horizontal.poles.len(), 2);
    let shape = Curve::Nurbs(horizontal);
    assert!(close(
        evaluate_curve(
            &shape,
            Parameter::Fraction(0.0),
            0,
            Transform::identity(),
            &limits()
        )
        .unwrap()[0],
        [10.0, 20.0, 31.0],
        1e-12
    ));
    assert!(close(
        evaluate_curve(
            &shape,
            Parameter::Fraction(1.0),
            0,
            Transform::identity(),
            &limits()
        )
        .unwrap()[0],
        [10.0, 24.0, 31.0],
        1e-12
    ));
    // Vary v, hold u: a vertical line of length 2.
    let vertical = isocurve(&surface, SurfaceIso::V, 0.25, &limits()).unwrap();
    let shape = Curve::Nurbs(vertical);
    assert!(close(
        evaluate_curve(
            &shape,
            Parameter::Fraction(1.0),
            0,
            Transform::identity(),
            &limits()
        )
        .unwrap()[0],
        [10.0, 21.0, 32.0],
        1e-12
    ));
}

#[test]
fn an_iso_curve_of_a_cylinder_is_the_exact_rational_circle() {
    let surface = cylinder_fixture();
    // Hold v: the result is the circle, still exactly at the radius.
    let circle = isocurve(&surface, SurfaceIso::U, 0.0, &limits()).unwrap();
    assert_eq!(circle.degree, 2);
    assert_eq!(circle.poles.len(), 9);
    assert!((circle.weights[0] - 1.0).abs() < 1e-12);
    assert!((circle.weights[1] - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12);
    let shape = Curve::Nurbs(circle);
    for step in 0..=16 {
        let point = evaluate_curve(
            &shape,
            Parameter::Fraction(step as f64 / 16.0),
            0,
            Transform::identity(),
            &limits(),
        )
        .unwrap()[0];
        let radial = (point[0] * point[0] + point[1] * point[1]).sqrt();
        assert!((radial - 5.0).abs() < 1e-9, "radius {radial}");
        assert!(point[2].abs() < 1e-9);
    }
    // Hold u: the ruling is a vertical line of length 8.
    let ruling = isocurve(&surface, SurfaceIso::V, 0.0, &limits()).unwrap();
    let shape = Curve::Nurbs(ruling);
    assert!(close(
        evaluate_curve(
            &shape,
            Parameter::Fraction(0.0),
            0,
            Transform::identity(),
            &limits()
        )
        .unwrap()[0],
        [5.0, 0.0, 0.0],
        1e-9
    ));
    assert!(close(
        evaluate_curve(
            &shape,
            Parameter::Fraction(1.0),
            0,
            Transform::identity(),
            &limits()
        )
        .unwrap()[0],
        [5.0, 0.0, 8.0],
        1e-9
    ));
}

/* ------------------------- refit ------------------------- */

#[test]
fn a_refit_of_a_plane_reproduces_it() {
    let surface = plane_fixture();
    let refit = refit(&surface, [1, 1], [2, 2], &limits()).unwrap();
    assert_eq!(refit.degree, [1, 1]);
    for step in 0..=4 {
        let u = step as f64 / 4.0;
        let point = evaluate(&refit, u, u, &limits()).unwrap();
        assert!(close(
            point,
            evaluate(&surface, u, u, &limits()).unwrap(),
            1e-9
        ));
    }
}

#[test]
fn a_refit_of_a_cylinder_stays_close_to_the_source() {
    let surface = cylinder_fixture();
    let refit = refit(&surface, [2, 2], [16, 3], &limits()).unwrap();
    assert_eq!(refit.degree, [2, 2]);
    assert_eq!(refit.size, [16, 3]);
    for step in 0..=8 {
        let u = step as f64 / 8.0;
        let source = evaluate(&surface, u, 0.5, &limits()).unwrap();
        let fitted = evaluate(&refit, u, 0.5, &limits()).unwrap();
        let deviation = (0..3)
            .map(|axis| (source[axis] - fitted[axis]).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(deviation < 0.05, "u={u}: deviation {deviation}");
    }
}

#[test]
fn a_refit_refuses_a_grid_that_cannot_hold_the_degree() {
    let surface = plane_fixture();
    assert_eq!(
        refit(&surface, [2, 1], [2, 3], &limits()),
        Err(Error::Invalid("surface refit grid"))
    );
    assert_eq!(
        refit(&surface, [1, 3], [3, 3], &limits()),
        Err(Error::Invalid("surface refit grid"))
    );
}

/* ------------------------- builders ------------------------- */

#[test]
fn the_analytic_builders_refuse_a_bad_frame_or_size() {
    assert_eq!(
        plane(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            2.0,
            2.0,
            &limits()
        ),
        Err(Error::Invalid("plane frame"))
    );
    assert_eq!(
        plane(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            0.0,
            2.0,
            &limits()
        ),
        Err(Error::Invalid("plane size"))
    );
    assert_eq!(
        cylinder(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            5.0,
            8.0,
            &limits()
        ),
        Err(Error::Invalid("cylinder frame"))
    );
    assert_eq!(
        cylinder(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            -1.0,
            8.0,
            &limits()
        ),
        Err(Error::Invalid("cylinder size"))
    );
}

#[test]
fn a_cylinder_built_rational_curve_matches_the_curve_kernel_circle() {
    // Cross-check the two kernels through an independent path: the u-iso-curve of the
    // cylinder must agree with the curve kernel's circle through three points.
    let surface = cylinder_fixture();
    let iso = isocurve(&surface, SurfaceIso::U, 0.0, &limits()).unwrap();
    let reference =
        om9_core::curve_build::circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0])
            .unwrap();
    for step in 0..=8 {
        let fraction = step as f64 / 8.0;
        let a = evaluate_curve(
            &Curve::Nurbs(iso.clone()),
            Parameter::Fraction(fraction),
            0,
            Transform::identity(),
            &limits(),
        )
        .unwrap()[0];
        let b = evaluate_curve(
            &reference,
            Parameter::Fraction(fraction),
            0,
            Transform::identity(),
            &limits(),
        )
        .unwrap()[0];
        assert!(close(a, b, 1e-9), "fraction {fraction}: {a:?} != {b:?}");
    }
    // The curve kernel still refuses an even interpolation degree for the command form.
    assert_eq!(
        interpolate(
            &[[0.0; 3], [1.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            2,
            KnotMode::Uniform,
            &limits()
        ),
        Err(Error::Invalid("interpolation degree"))
    );
}

/* ------------------------- loft ------------------------- */

#[test]
fn a_loft_between_identical_circles_is_a_cylinder() {
    // Three circles of radius 5 at z = 0, 5, 10: the loft must reproduce the radius at
    // every (u, v) and place each section at its own height.
    let sections: Vec<Curve> = [0.0, 5.0, 10.0]
        .iter()
        .map(|z| {
            om9_core::curve_build::circle_through([5.0, 0.0, *z], [0.0, 5.0, *z], [-5.0, 0.0, *z])
                .unwrap()
        })
        .collect();
    let surface = loft(&sections, [3, 2], 17, &limits()).unwrap();
    assert_eq!(surface.size, [17, 3]);
    assert_eq!(surface.degree, [3, 2]);
    for ustep in 0..=8 {
        let u = ustep as f64 / 8.0;
        for (index, z) in [0.0, 5.0, 10.0].iter().enumerate() {
            let v = index as f64 / 2.0;
            let point = evaluate(&surface, u, v, &limits()).unwrap();
            let radial = (point[0] * point[0] + point[1] * point[1]).sqrt();
            assert!((radial - 5.0).abs() < 0.05, "u={u} v={v}: radius {radial}");
            assert!(
                (point[2] - z).abs() < 1e-9,
                "u={u} v={v}: height {:?} != {z}",
                point[2]
            );
        }
    }
    // The surface is a cylinder, so the normal has no vertical component.
    let n = normal(&surface, 0.3, 0.5, &limits()).unwrap();
    assert!(n[2].abs() < 1e-6, "{n:?}");
}

#[test]
fn a_loft_refuses_impossible_sections() {
    let circle =
        om9_core::curve_build::circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0])
            .unwrap();
    assert_eq!(
        loft(std::slice::from_ref(&circle), [3, 1], 17, &limits()),
        Err(Error::Invalid("loft section count"))
    );
    // Two sections cannot carry a quadratic v degree.
    assert_eq!(
        loft(&[circle.clone(), circle.clone()], [3, 2], 17, &limits()),
        Err(Error::Invalid("loft section count"))
    );
    // Too few u samples for the u degree.
    assert_eq!(
        loft(&[circle.clone(), circle], [3, 1], 3, &limits()),
        Err(Error::Invalid("loft section samples"))
    );
}

/* ------------------------- sweep1 ------------------------- */

fn straight_rail() -> Curve {
    Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [0.0, 0.0, 10.0],
        domain: [0.0, 1.0],
    }
}

#[test]
fn path_frames_stay_orthonormal_and_perpendicular_to_the_tangent() {
    // A quarter circle in the XY plane: the tangent turns, so the transport matters.
    let rail = om9_core::curve_build::arc_through(
        [5.0, 0.0, 0.0],
        [
            std::f64::consts::FRAC_1_SQRT_2 * 5.0,
            std::f64::consts::FRAC_1_SQRT_2 * 5.0,
            0.0,
        ],
        [0.0, 5.0, 0.0],
    )
    .unwrap();
    let frames = path_frames(&rail, 9, &limits()).unwrap();
    assert_eq!(frames.len(), 9);
    for frame in &frames {
        assert!((dot(frame.x, frame.x) - 1.0).abs() < 1e-9);
        assert!((dot(frame.y, frame.y) - 1.0).abs() < 1e-9);
        assert!((dot(frame.tangent, frame.tangent) - 1.0).abs() < 1e-9);
        assert!(dot(frame.x, frame.y).abs() < 1e-9);
        assert!(dot(frame.x, frame.tangent).abs() < 1e-9);
        assert!(dot(frame.y, frame.tangent).abs() < 1e-9);
        // Every station sits on the rail circle.
        let radial = (frame.origin[0].powi(2) + frame.origin[1].powi(2)).sqrt();
        assert!((radial - 5.0).abs() < 1e-9);
    }
}

#[test]
fn a_sweep1_along_a_straight_rail_reproduces_the_profile_at_every_station() {
    let profile =
        om9_core::curve_build::circle_through([2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [-2.0, 0.0, 0.0])
            .unwrap();
    let surface = sweep1(&profile, &straight_rail(), [17, 5], [3, 2], &limits()).unwrap();
    assert_eq!(surface.size, [17, 5]);
    for ustep in 0..=8 {
        let u = ustep as f64 / 8.0;
        for vstep in 0..=4 {
            let v = vstep as f64 / 4.0;
            let point = evaluate(&surface, u, v, &limits()).unwrap();
            let radial = (point[0] * point[0] + point[1] * point[1]).sqrt();
            assert!((radial - 2.0).abs() < 1e-6, "u={u} v={v}: radius {radial}");
            // The sweep is a cylinder: the height only depends on v.
            assert!(
                (point[2] - 10.0 * v).abs() < 1e-9,
                "u={u} v={v}: z {:?}",
                point[2]
            );
        }
    }
    let n = normal(&surface, 0.2, 0.5, &limits()).unwrap();
    assert!(
        n[2].abs() < 1e-6,
        "a straight sweep has no vertical normal: {n:?}"
    );
}

#[test]
fn a_sweep1_carries_a_square_prism_rigidly() {
    // A square profile in the XY plane, swept along Z: corner positions are predictable.
    let profile = om9_core::curve_build::rectangle(
        [-1.0, -1.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        2.0,
        2.0,
    )
    .unwrap();
    // A polyline profile must use a linear u degree: a cubic would bulge outside the
    // corners. With degree 1 the surface follows the edges exactly.
    let surface = sweep1(&profile, &straight_rail(), [9, 3], [1, 2], &limits()).unwrap();
    let mut stations = Vec::new();
    for ustep in 0..=16 {
        let u = ustep as f64 / 16.0;
        let point = evaluate(&surface, u, 0.5, &limits()).unwrap();
        assert!((point[2] - 5.0).abs() < 1e-9, "u={u}: z {:?}", point[2]);
        let on_x_edge = (point[0].abs() - 1.0).abs() < 1e-9;
        let on_y_edge = (point[1].abs() - 1.0).abs() < 1e-9;
        assert!(
            on_x_edge || on_y_edge,
            "u={u}: {point:?} left the square boundary"
        );
        stations.push(point);
    }
    // Every corner is reached exactly (rigid carrying, no shear).
    for corner in [
        [-1.0, -1.0, 5.0],
        [1.0, -1.0, 5.0],
        [1.0, 1.0, 5.0],
        [-1.0, 1.0, 5.0],
    ] {
        assert!(
            stations.iter().any(|point| close(*point, corner, 1e-9)),
            "corner {corner:?} missing from the swept cross-section: {stations:?}"
        );
    }
}

#[test]
fn a_sweep1_refuses_impossible_input() {
    let profile =
        om9_core::curve_build::circle_through([2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [-2.0, 0.0, 0.0])
            .unwrap();
    assert_eq!(
        sweep1(&profile, &straight_rail(), [3, 5], [3, 2], &limits()),
        Err(Error::Invalid("sweep profile samples"))
    );
    assert_eq!(
        sweep1(&profile, &straight_rail(), [17, 2], [3, 2], &limits()),
        Err(Error::Invalid("sweep rail samples"))
    );
    // A rail with no extent cannot carry frames.
    let degenerate = Curve::Line {
        start: [1.0, 1.0, 1.0],
        end: [1.0, 1.0, 1.0],
        domain: [0.0, 1.0],
    };
    assert!(path_frames(&degenerate, 5, &limits()).is_err());
}

/* ------------------------- sweep2 (documented oracle) ------------------------- */

/// The rails of the Matrix 8 record: width grows from 5 to 9 mm over 10 mm of length.
fn sweep2_rails() -> (Curve, Curve) {
    let rail_a = Curve::Line {
        start: [0.0, 0.0, 0.0],
        end: [0.0, 0.0, 10.0],
        domain: [0.0, 1.0],
    };
    let rail_b = Curve::Line {
        start: [5.0, 0.0, 0.0],
        end: [9.0, 0.0, 10.0],
        domain: [0.0, 1.0],
    };
    (rail_a, rail_b)
}

/// An arch from rail A start to rail B start with a 2 mm crown, lying in the section plane
/// (the frame spans separation x and the normal): the section of the record. A profile that
/// bulged along the rails would be a section parallel to the sweep direction.
fn sweep2_profile() -> Curve {
    om9_core::curve_build::arc_through([0.0, 0.0, 0.0], [2.5, -2.0, 0.0], [5.0, 0.0, 0.0]).unwrap()
}

/// The crown height: the perpendicular distance of the section midpoint from the chord
/// between the two rails. The frame's normal comes from `x cross primary_tangent`, so the
/// height is measured in the section plane rather than along a world axis.
fn crown_height(surface: &NurbsSurface, v: f64) -> f64 {
    let (rail_a, rail_b) = sweep2_rails();
    let a = evaluate_curve(
        &rail_a,
        Parameter::Fraction(v),
        0,
        Transform::identity(),
        &limits(),
    )
    .unwrap()[0];
    let b = evaluate_curve(
        &rail_b,
        Parameter::Fraction(v),
        0,
        Transform::identity(),
        &limits(),
    )
    .unwrap()[0];
    let chord = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let chord_length = dot(chord, chord).sqrt();
    let x = [
        chord[0] / chord_length,
        chord[1] / chord_length,
        chord[2] / chord_length,
    ];
    let midpoint = [
        (a[0] + b[0]) / 2.0,
        (a[1] + b[1]) / 2.0,
        (a[2] + b[2]) / 2.0,
    ];
    let crown = evaluate(surface, 0.5, v, &limits()).unwrap();
    let offset = [
        crown[0] - midpoint[0],
        crown[1] - midpoint[1],
        crown[2] - midpoint[2],
    ];
    let along = dot(offset, x);
    let perpendicular = [
        offset[0] - x[0] * along,
        offset[1] - x[1] * along,
        offset[2] - x[2] * along,
    ];
    dot(perpendicular, perpendicular).sqrt()
}

#[test]
fn sweep2_matches_the_recorded_maintain_height_oracle() {
    let (rail_a, rail_b) = sweep2_rails();
    let profile = sweep2_profile();
    let options = Sweep2Options {
        samples: [17, 65],
        degree: [3, 3],
        maintain_height: true,
    };
    let height_on = sweep2(&profile, &rail_a, &rail_b, options, &limits()).unwrap();
    assert_eq!(height_on.size, [17, 65]);
    // Maintain Height ON: the 2 mm crown is preserved while the width grows 5 -> 9.
    assert!((crown_height(&height_on, 0.0) - 2.0).abs() < 1e-6);
    assert!((crown_height(&height_on, 1.0) - 2.0).abs() < 1e-6);

    let options = Sweep2Options {
        maintain_height: false,
        ..options
    };
    let height_off = sweep2(&profile, &rail_a, &rail_b, options, &limits()).unwrap();
    // OFF scales the normal coordinate with the width: 2 * 9 / 5 = 3.6 mm, exactly the
    // number in docs/features/OM9-SURFACE-003.md.
    assert!(
        (crown_height(&height_off, 1.0) - 3.6).abs() < 1e-6,
        "OFF crown: v0={} v05={} v1={} | ON v1={}",
        crown_height(&height_off, 0.0),
        crown_height(&height_off, 0.5),
        crown_height(&height_off, 1.0),
        crown_height(&height_on, 1.0)
    );
    assert!((crown_height(&height_off, 0.0) - 2.0).abs() < 1e-6);
}

#[test]
fn sweep2_keeps_both_rails_in_contact() {
    let (rail_a, rail_b) = sweep2_rails();
    let profile = sweep2_profile();
    let surface = sweep2(
        &profile,
        &rail_a,
        &rail_b,
        Sweep2Options::default(),
        &limits(),
    )
    .unwrap();
    // Width grows with the separation, so the profile first and last sample land exactly on
    // the two rails at every station.
    for step in 0..=4 {
        let v = step as f64 / 4.0;
        let start = evaluate(&surface, 0.0, v, &limits()).unwrap();
        let end = evaluate(&surface, 1.0, v, &limits()).unwrap();
        let z = 10.0 * v;
        assert!(
            close(start, [0.0, 0.0, z], 1e-6),
            "rail A at v={v}: {start:?}"
        );
        let width = 5.0 + 4.0 * v;
        assert!(
            close(end, [width, 0.0, z], 1e-6),
            "rail B at v={v}: {end:?}"
        );
    }
}

#[test]
fn arc_length_stations_are_equally_spaced_along_an_arc() {
    // A half-circle arc of radius 5: equal arc length means equal turn.
    let arc =
        om9_core::curve_build::arc_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0])
            .unwrap();
    let stations = arc_length_stations(&arc, 9, &limits()).unwrap();
    assert_eq!(stations.len(), 9);
    let mut lengths = Vec::new();
    for pair in stations.windows(2) {
        let d = [
            pair[1].0[0] - pair[0].0[0],
            pair[1].0[1] - pair[0].0[1],
            pair[1].0[2] - pair[0].0[2],
        ];
        lengths.push(dot(d, d).sqrt());
    }
    // Chord lengths are equal to within the table resolution (half turn, 8 steps).
    let first = lengths[0];
    for length in &lengths {
        assert!(
            (length - first).abs() < 1e-3 * first,
            "uneven station spacing: {lengths:?}"
        );
    }
}

#[test]
fn sweep2_refuses_what_the_contract_forbids() {
    let (rail_a, rail_b) = sweep2_rails();
    let profile = sweep2_profile();
    // Degree above three is outside the recorded bound.
    assert_eq!(
        sweep2(
            &profile,
            &rail_a,
            &rail_b,
            Sweep2Options {
                samples: [17, 65],
                degree: [4, 3],
                maintain_height: false
            },
            &limits()
        ),
        Err(Error::Invalid("sweep degree"))
    );
    // Closed rails need a closed loft, which is not implemented: refuse by name.
    let closed =
        om9_core::curve_build::circle_through([5.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-5.0, 0.0, 0.0])
            .unwrap();
    assert_eq!(
        sweep2(
            &profile,
            &closed,
            &rail_b,
            Sweep2Options::default(),
            &limits()
        ),
        Err(Error::Unsupported("sweep2 closed rails"))
    );
    // A profile that does not span the rails is refused (Matrix: the profile must intersect
    // both rail starts).
    let short =
        om9_core::curve_build::arc_through([0.0, 0.0, 0.0], [0.5, -1.0, 0.0], [1.0, 0.0, 0.0])
            .unwrap();
    assert_eq!(
        sweep2(
            &short,
            &rail_a,
            &rail_b,
            Sweep2Options::default(),
            &limits()
        ),
        Err(Error::Invalid("sweep2 profile span"))
    );
}
