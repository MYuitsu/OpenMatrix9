// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    evaluation::{Parameter, evaluate_curve},
    geometry::{Curve, NurbsCurve, Transform},
};

#[test]
fn circle_native_parameter_mapping_preserves_phase_and_scales_every_derivative() {
    let tau = std::f64::consts::TAU;
    let c = Curve::Circle {
        center: [0.; 3],
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        radius: 10.,
        domain: [11., 17.],
        angle_domain: Some([1., 1. + tau]),
    };
    let rate = tau / 6.;
    for (parameter, angle, chain) in [
        (Parameter::Native(11.), 1., rate),
        (Parameter::Native(14.), 1. + std::f64::consts::PI, rate),
        (Parameter::Fraction(0.5), 1. + std::f64::consts::PI, tau),
    ] {
        let jet =
            evaluate_curve(&c, parameter, 3, Transform::identity(), &Limits::default()).unwrap();
        let (s, c) = angle.sin_cos();
        let directions = [[c, s, 0.], [-s, c, 0.], [-c, -s, 0.], [s, -c, 0.]];
        for (order, actual) in jet.iter().enumerate() {
            for axis in 0..3 {
                assert!(
                    (actual[axis] - 10. * directions[order][axis] * chain.powi(order as i32)).abs()
                        < 1e-10
                );
            }
        }
    }
}
fn close(a: [f64; 3], b: [f64; 3]) {
    for i in 0..3 {
        assert!((a[i] - b[i]).abs() <= 1e-10, "{a:?} != {b:?}");
    }
}
#[test]
fn mapped_circle_fraction_derivatives_do_not_overflow_canceling_native_scales() {
    let tau = std::f64::consts::TAU;
    let angle = 1. + std::f64::consts::PI;
    let (s, c) = angle.sin_cos();
    let directions = [[c, s, 0.], [-s, c, 0.], [-c, -s, 0.], [s, -c, 0.]];
    for domain in [[0., 1e120], [0., 1e-120], [1., 1. + f64::EPSILON]] {
        let span = domain[1] - domain[0];
        let circle = Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 10.,
            domain,
            angle_domain: Some([1., 1. + tau]),
        };
        let jet = evaluate_curve(
            &circle,
            Parameter::Fraction(0.5),
            3,
            Transform::identity(),
            &Limits::default(),
        )
        .unwrap();
        for (k, actual) in jet.iter().enumerate() {
            for axis in 0..3 {
                assert!(
                    (actual[axis] - 10. * directions[k][axis] * tau.powi(k as i32)).abs() < 1e-10
                );
            }
        }
        // D0 is still meaningful where native D3 itself is unrepresentable.
        evaluate_curve(
            &circle,
            Parameter::Native(domain[0] + span / 2.),
            0,
            Transform::identity(),
            &Limits::default(),
        )
        .unwrap();
        if span == 1e-120 {
            assert!(
                evaluate_curve(
                    &circle,
                    Parameter::Native(domain[0] + span / 2.),
                    3,
                    Transform::identity(),
                    &Limits::default()
                )
                .is_err()
            );
        }
    }
}
#[test]
fn analytic_circle_d0_d1_d2_d3_use_native_radians_and_world_frame() {
    let c = Curve::Circle {
        center: [1., 2., 3.],
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        radius: 10.,
        domain: [0., std::f64::consts::TAU],
        angle_domain: None,
    };
    let j = evaluate_curve(
        &c,
        Parameter::Native(std::f64::consts::FRAC_PI_2),
        3,
        Transform::translation([5., 0., 0.]),
        &Limits::default(),
    )
    .unwrap();
    close(j[0], [6., 12., 3.]);
    close(j[1], [-10., 0., 0.]);
    close(j[2], [0., -10., 0.]);
    close(j[3], [10., 0., 0.]);
}
#[test]
fn rational_quadratic_has_nonzero_third_derivative_from_quotient() {
    // Homogeneous polynomial: X=2t, W=1+t, so x=2t/(1+t).
    // At t=.5: D0=2/3, D1=8/9, D2=-32/27, D3=64/27.
    let c = Curve::Nurbs(NurbsCurve {
        degree: 2,
        poles: vec![[0., 0., 0.], [1., 0., 0.], [1., 0., 0.]],
        weights: vec![1., 1.5, 2.],
        knots: vec![0., 0., 0., 1., 1., 1.],
        domain: [0., 1.],
        periodic: false,
        reversed: false,
        seam: None,
    });
    // The middle homogeneous X is 1.5, making X(t)=3t-t² rather than 2t;
    // use pole 2/3 to encode X=2t exactly.
    let Curve::Nurbs(mut n) = c else {
        unreachable!()
    };
    n.poles[1][0] = 2. / 3.;
    let j = evaluate_curve(
        &Curve::Nurbs(n),
        Parameter::Native(0.5),
        3,
        Transform::identity(),
        &Limits::default(),
    )
    .unwrap();
    close(j[0], [2. / 3., 0., 0.]);
    close(j[1], [8. / 9., 0., 0.]);
    close(j[2], [-32. / 27., 0., 0.]);
    close(j[3], [64. / 27., 0., 0.]);
}
#[test]
fn reversed_fraction_derivatives_scale_each_order_without_translating_vectors() {
    let n = NurbsCurve {
        degree: 3,
        poles: vec![[0., 0., 0.], [0., 0., 0.], [0., 0., 0.], [8., 0., 0.]],
        weights: vec![1.; 4],
        knots: vec![2., 2., 2., 2., 4., 4., 4., 4.],
        domain: [2., 4.],
        periodic: false,
        reversed: true,
        seam: None,
    };
    let j = evaluate_curve(
        &Curve::Nurbs(n),
        Parameter::Fraction(0.5),
        3,
        Transform::translation([10., 20., 30.]),
        &Limits::default(),
    )
    .unwrap();
    close(j[0], [11., 20., 30.]);
    close(j[1], [-6., 0., 0.]);
    close(j[2], [24., 0., 0.]);
    close(j[3], [-48., 0., 0.]);
}
#[test]
fn endpoints_are_one_sided_and_parameters_outside_domain_reject() {
    let c = Curve::Line {
        start: [0., 0., 0.],
        end: [10., 0., 0.],
        domain: [2., 4.],
    };
    close(
        evaluate_curve(
            &c,
            Parameter::Native(4.),
            3,
            Transform::identity(),
            &Limits::default(),
        )
        .unwrap()[1],
        [5., 0., 0.],
    );
    for p in [
        Parameter::Native(1.),
        Parameter::Native(f64::NAN),
        Parameter::Fraction(1.1),
    ] {
        assert!(evaluate_curve(&c, p, 1, Transform::identity(), &Limits::default()).is_err());
    }
}
#[test]
fn interior_discontinuity_cannot_claim_unqualified_derivative() {
    let c = Curve::Nurbs(NurbsCurve {
        degree: 1,
        poles: vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.]],
        weights: vec![1.; 3],
        knots: vec![0., 0., 0.5, 1., 1.],
        domain: [0., 1.],
        periodic: false,
        reversed: false,
        seam: None,
    });
    assert!(
        evaluate_curve(
            &c,
            Parameter::Native(0.5),
            1,
            Transform::identity(),
            &Limits::default()
        )
        .is_err()
    );
    close(
        evaluate_curve(
            &c,
            Parameter::Native(0.5),
            0,
            Transform::identity(),
            &Limits::default(),
        )
        .unwrap()[0],
        [1., 0., 0.],
    );
}

#[test]
fn overflowing_native_parameter_span_never_produces_a_false_finite_line_value() {
    let curve = Curve::Line {
        start: [0., 0., 0.],
        end: [10., 0., 0.],
        domain: [-f64::MAX, f64::MAX],
    };
    assert!(
        evaluate_curve(
            &curve,
            Parameter::Native(0.),
            0,
            Transform::identity(),
            &Limits::default()
        )
        .is_err()
    );
}

#[test]
fn captured_curve_evaluator_retains_admission_limits_and_uses_polynomial_oracle() {
    use om9_core::evaluation::CurveEvaluator;
    let curve = Curve::Nurbs(NurbsCurve {
        degree: 3,
        poles: vec![[0., 0., 0.], [0., 0., 0.], [0., 0., 0.], [8., 0., 0.]],
        weights: vec![1.; 4],
        knots: vec![2., 2., 2., 2., 4., 4., 4., 4.],
        domain: [2., 4.],
        periodic: false,
        reversed: false,
        seam: None,
    });
    let denied = Limits {
        max_degree: 2,
        ..Limits::default()
    };
    assert!(CurveEvaluator::capture(&curve, &denied).is_err());
    let evaluator = CurveEvaluator::capture(&curve, &Limits::default()).unwrap();
    for u in [2., 2.25, 3., 3.5, 4.] {
        // x(u)=(u-2)^3; independent exact power polynomial in native units.
        let t: f64 = u - 2.;
        let values = evaluator
            .evaluate(
                Parameter::Native(u),
                3,
                Transform::translation([10., 0., 0.]),
            )
            .unwrap();
        close(values[0], [10. + t.powi(3), 0., 0.]);
        close(values[1], [3. * t * t, 0., 0.]);
        close(values[2], [6. * t, 0., 0.]);
        close(values[3], [6., 0., 0.]);
    }
    assert!(
        evaluator
            .evaluate(Parameter::Native(4.1), 0, Transform::identity())
            .is_err()
    );
}
