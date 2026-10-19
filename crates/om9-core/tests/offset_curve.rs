//! Offset curve derivative contract.
//!
//! The oracles here do not come from the expression under test: analytic circles give `D0..D3` in
//! closed form, and the finite-difference checks sample the *point* only. That is the same shape as
//! the OCCT conformance probe, so a wrong expansion fails here for the same reason it failed there.

use om9_core::error::Error;
use om9_core::geometry::{NurbsCurve, Point3};
use om9_core::offset_curve::{
    CircleBasis, CurveBasis, Derivatives, LineBasis, NurbsBasis, OffsetCurve,
};
use std::f64::consts::TAU;

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn norm(a: Point3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn relative(actual: Point3, expected: Point3) -> f64 {
    let scale = norm(expected).max(norm(actual)).max(1.0);
    norm(sub(actual, expected)) / scale
}

/// Five-point second derivative of the point, extrapolated (Richardson) to cancel `O(h²)`.
fn finite_d2(curve: &impl CurveBasis, u: f64, h: f64) -> Point3 {
    let fd = |step: f64| {
        let minus_two = curve
            .derivatives(u - 2.0 * step)
            .expect("parameter")
            .point;
        let minus_one = curve.derivatives(u - step).expect("parameter").point;
        let zero = curve.derivatives(u).expect("parameter").point;
        let plus_one = curve.derivatives(u + step).expect("parameter").point;
        let plus_two = curve.derivatives(u + 2.0 * step).expect("parameter").point;
        let factor = 1.0 / (12.0 * step * step);
        [
            (-minus_two[0] + 16.0 * minus_one[0] - 30.0 * zero[0] + 16.0 * plus_one[0]
                - plus_two[0])
                * factor,
            (-minus_two[1] + 16.0 * minus_one[1] - 30.0 * zero[1] + 16.0 * plus_one[1]
                - plus_two[1])
                * factor,
            (-minus_two[2] + 16.0 * minus_one[2] - 30.0 * zero[2] + 16.0 * plus_one[2]
                - plus_two[2])
                * factor,
        ]
    };
    let coarse = fd(h);
    let fine = fd(0.5 * h);
    [
        (4.0 * fine[0] - coarse[0]) / 3.0,
        (4.0 * fine[1] - coarse[1]) / 3.0,
        (4.0 * fine[2] - coarse[2]) / 3.0,
    ]
}

fn finite_d3(curve: &impl CurveBasis, u: f64, h: f64) -> Point3 {
    let fd = |step: f64| {
        let minus_two = curve
            .derivatives(u - 2.0 * step)
            .expect("parameter")
            .point;
        let minus_one = curve.derivatives(u - step).expect("parameter").point;
        let plus_one = curve.derivatives(u + step).expect("parameter").point;
        let plus_two = curve.derivatives(u + 2.0 * step).expect("parameter").point;
        let factor = 1.0 / (2.0 * step * step * step);
        [
            (plus_two[0] - 2.0 * plus_one[0] + 2.0 * minus_one[0] - minus_two[0]) * factor,
            (plus_two[1] - 2.0 * plus_one[1] + 2.0 * minus_one[1] - minus_two[1]) * factor,
            (plus_two[2] - 2.0 * plus_one[2] + 2.0 * minus_one[2] - minus_two[2]) * factor,
        ]
    };
    let coarse = fd(h);
    let fine = fd(0.5 * h);
    [
        (4.0 * fine[0] - coarse[0]) / 3.0,
        (4.0 * fine[1] - coarse[1]) / 3.0,
        (4.0 * fine[2] - coarse[2]) / 3.0,
    ]
}

/// A circle with a non-uniform parameterisation `θ(u) = u + warp·sin u`.
///
/// Its offset is still an exact circle, so the analytic oracle is known, while `R = |p′ × V| = r·θ′`
/// varies with `u` - the regime in which the OCCT 8.0.1 expansion is wrong and the plain-circle test
/// is blind.
#[derive(Clone, Copy)]
struct WarpedCircle {
    radius: f64,
    warp: f64,
}

impl WarpedCircle {
    fn angle(&self, u: f64) -> (f64, f64, f64, f64, f64) {
        let theta = u + self.warp * u.sin();
        let d1 = 1.0 + self.warp * u.cos();
        let d2 = -self.warp * u.sin();
        let d3 = -self.warp * u.cos();
        let d4 = self.warp * u.sin();
        (theta, d1, d2, d3, d4)
    }

    /// Exact derivatives of the concentric circle of `radius + distance` at parameter `u`.
    fn offset_oracle(&self, u: f64, distance: f64) -> Derivatives {
        let (theta, t1, t2, t3, _t4) = self.angle(u);
        let (sin, cos) = theta.sin_cos();
        let big_r = self.radius + distance;
        // radial = (cos, sin), tangent = (-sin, cos)
        let radial = [cos, sin, 0.0];
        let tangent = [-sin, cos, 0.0];
        let combination = |along_radial: f64, along_tangent: f64| {
            [
                big_r * (along_radial * radial[0] + along_tangent * tangent[0]),
                big_r * (along_radial * radial[1] + along_tangent * tangent[1]),
                0.0,
            ]
        };
        Derivatives {
            point: combination(1.0, 0.0),
            d1: combination(0.0, t1),
            d2: combination(-t1 * t1, t2),
            d3: combination(-3.0 * t1 * t2, t3 - t1 * t1 * t1),
            d4: [0.0; 3],
        }
    }
}

impl CurveBasis for WarpedCircle {
    fn domain(&self) -> [f64; 2] {
        [0.0, TAU]
    }

    fn derivatives(&self, u: f64) -> Result<Derivatives, om9_core::error::Error> {
        let (theta, t1, t2, t3, t4) = self.angle(u);
        let (sin, cos) = theta.sin_cos();
        let radial = [cos, sin, 0.0];
        let tangent = [-sin, cos, 0.0];
        let combine = |along_radial: f64, along_tangent: f64| {
            [
                self.radius * (along_radial * radial[0] + along_tangent * tangent[0]),
                self.radius * (along_radial * radial[1] + along_tangent * tangent[1]),
                0.0,
            ]
        };
        Ok(Derivatives {
            point: combine(1.0, 0.0),
            d1: combine(0.0, t1),
            d2: combine(-t1 * t1, t2),
            d3: combine(-3.0 * t1 * t2, t3 - t1 * t1 * t1),
            // d4 = r[(t1^4 - 4 t1 t3 - 3 t2^2) * radial + (t4 - 6 t1^2 t2) * tangent]
            d4: combine(
                t1 * t1 * t1 * t1 - 4.0 * t1 * t3 - 3.0 * t2 * t2,
                t4 - 6.0 * t1 * t1 * t2,
            ),
        })
    }
}

/// The exact expressions OCCT 8.0.1 uses, kept here as the negative control: the checks below must
/// reject them while accepting the module's own derivation.
fn occt_8_0_1_offset_derivatives(basis: &Derivatives, distance: f64, v: Point3) -> (Point3, Point3) {
    let sub3 = sub;
    let cross = |a: Point3, b: Point3| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let dot = |a: Point3, b: Point3| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let scale = |a: Point3, f: f64| [a[0] * f, a[1] * f, a[2] * f];
    let a0 = cross(basis.d1, v);
    let a1 = cross(basis.d2, v);
    let a2 = cross(basis.d3, v);
    let a3 = cross(basis.d4, v);
    let r2 = dot(a0, a0);
    let r = r2.sqrt();
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    let dr = dot(a0, a1);
    let d2r = dot(a0, a2) + dot(a1, a1);
    let d3r = dot(a0, a3) + 3.0 * dot(a1, a2);
    // D2: the normal term is subtracted although the derivation gives plus.
    let d2 = scale(
        sub3(
            sub3(a2, scale(a1, 2.0 * dr / r2)),
            scale(a0, 3.0 * dr * dr / r4 - d2r / r2),
        ),
        distance / r,
    );
    // D3: spurious 6·Dr²/R⁴, 6 instead of 9, and D3r without the 1/R² factor.
    let d3 = scale(
        [
            a3[0] - 3.0 * dr / r2 * a2[0] - 3.0 * (d2r / r2 + dr * dr / r4) * a1[0]
                + (6.0 * dr * dr / r4 + 6.0 * dr * d2r / r4 - 15.0 * dr * dr * dr / r6 - d3r)
                    * a0[0],
            a3[1] - 3.0 * dr / r2 * a2[1] - 3.0 * (d2r / r2 + dr * dr / r4) * a1[1]
                + (6.0 * dr * dr / r4 + 6.0 * dr * d2r / r4 - 15.0 * dr * dr * dr / r6 - d3r)
                    * a0[1],
            a3[2] - 3.0 * dr / r2 * a2[2] - 3.0 * (d2r / r2 + dr * dr / r4) * a1[2]
                + (6.0 * dr * dr / r4 + 6.0 * dr * d2r / r4 - 15.0 * dr * dr * dr / r6 - d3r)
                    * a0[2],
        ],
        distance / r,
    );
    (d2, d3)
}

#[test]
fn a_line_offset_is_a_parallel_line() {
    let basis = LineBasis::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]).expect("line");
    let offset = OffsetCurve::new(basis, 2.0, [0.0, 0.0, 1.0]).expect("offset");
    let derivatives = offset.derivatives(0.37).expect("derivatives");
    assert!(relative(derivatives.point, [0.37, -2.0, 0.0]) < 1e-12);
    assert!(norm(derivatives.d2) < 1e-12);
    assert!(norm(derivatives.d3) < 1e-12);
    assert!(offset.curvature(0.37).expect("curvature") < 1e-12);
}

#[test]
fn a_circle_offset_is_the_concentric_circle() {
    let radius = 4.0;
    let distance = 1.25;
    let basis = CircleBasis::new([1.0, -2.0, 3.0], [0.0, 0.0, 1.0], radius).expect("circle");
    let offset = OffsetCurve::new(basis, distance, [0.0, 0.0, 1.0]).expect("offset");
    let expected_radius = radius + distance;
    for u in [0.0, 0.4, 1.1, 2.7] {
        let derivatives = offset.derivatives(u).expect("derivatives");
        let local = sub(derivatives.point, [1.0, -2.0, 3.0]);
        assert!((norm(local) - expected_radius).abs() < 1e-12);
        let expected_d2 = [
            -expected_radius * local[0] / expected_radius,
            -expected_radius * local[1] / expected_radius,
            0.0,
        ];
        assert!(relative(derivatives.d2, expected_d2) < 1e-12);
        let expected_curvature = 1.0 / expected_radius;
        assert!((offset.curvature(u).expect("curvature") - expected_curvature).abs() < 1e-12);
    }
}

#[test]
fn a_negative_distance_moves_the_circle_inward() {
    let basis = CircleBasis::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 4.0).expect("circle");
    let offset = OffsetCurve::new(basis, -1.25, [0.0, 0.0, 1.0]).expect("offset");
    let point = offset.point(0.6).expect("point");
    assert!((norm(point) - 2.75).abs() < 1e-12);
}

#[test]
fn a_non_uniform_parameterisation_keeps_the_offset_exact() {
    let basis = WarpedCircle {
        radius: 5.0,
        warp: 0.6,
    };
    let distance = 1.5;
    let offset = OffsetCurve::new(basis, distance, [0.0, 0.0, 1.0]).expect("offset");
    for u in [0.2, 0.9, 1.7, 2.6] {
        let expected = basis.offset_oracle(u, distance);
        let actual = offset.derivatives(u).expect("derivatives");
        assert!(relative(actual.point, expected.point) < 1e-12);
        assert!(relative(actual.d1, expected.d1) < 1e-12);
        assert!(relative(actual.d2, expected.d2) < 1e-10);
        assert!(relative(actual.d3, expected.d3) < 1e-9);
    }
}

#[test]
fn the_finite_difference_oracle_agrees_with_the_closed_form() {
    let basis = WarpedCircle {
        radius: 5.0,
        warp: 0.6,
    };
    let offset = OffsetCurve::new(basis, 1.5, [0.0, 0.0, 1.0]).expect("offset");
    for u in [0.3, 1.4, 2.2] {
        let derivatives = offset.derivatives(u).expect("derivatives");
        assert!(relative(finite_d2(&offset, u, 1e-2), derivatives.d2) < 1e-7);
        assert!(relative(finite_d3(&offset, u, 2e-2), derivatives.d3) < 1e-5);
    }
}

#[test]
fn the_occt_8_0_1_expansion_fails_the_same_oracle() {
    let basis = WarpedCircle {
        radius: 5.0,
        warp: 0.6,
    };
    let distance = 1.5;
    let offset = OffsetCurve::new(basis, distance, [0.0, 0.0, 1.0]).expect("offset");
    let mut worst_d2: f64 = 0.0;
    let mut worst_d3: f64 = 0.0;
    for u in [0.3, 1.1, 2.0] {
        let correct = offset.derivatives(u).expect("derivatives");
        let (wrong_d2, wrong_d3) =
            occt_8_0_1_offset_derivatives(&basis.derivatives(u).expect("basis"), distance, [0.0, 0.0, 1.0]);
        worst_d2 = worst_d2.max(relative(wrong_d2, correct.d2));
        worst_d3 = worst_d3.max(relative(wrong_d3, correct.d3));
    }
    assert!(
        worst_d2 > 1e-2,
        "the wrong D2 expansion must differ from the correct curve, measured {worst_d2:e}"
    );
    assert!(
        worst_d3 > 1e-2,
        "the wrong D3 expansion must differ from the correct curve, measured {worst_d3:e}"
    );
}

fn quarter_circle(radius: f64) -> NurbsCurve {
    let h = std::f64::consts::FRAC_1_SQRT_2;
    NurbsCurve {
        degree: 2,
        poles: vec![[radius, 0.0, 0.0], [radius, radius, 0.0], [0.0, radius, 0.0]],
        weights: vec![1.0, h, 1.0],
        knots: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    }
}

#[test]
fn a_rational_quarter_circle_offsets_to_the_larger_circle() {
    let radius = 6.0;
    let curve = quarter_circle(radius);
    let basis = NurbsBasis::new(&curve);
    let distance = 0.75;
    let offset = OffsetCurve::new(basis, distance, [0.0, 0.0, 1.0]).expect("offset");
    for u in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let point = offset.point(u).expect("point");
        assert!(
            (norm(point) - (radius + distance)).abs() < 1e-9,
            "u={u}: |P|={} expected {}",
            norm(point),
            radius + distance
        );
    }
}

#[test]
fn rational_nurbs_derivatives_match_finite_differences() {
    let curve = quarter_circle(6.0);
    let basis = NurbsBasis::new(&curve);
    for u in [0.2, 0.5, 0.8] {
        let derivatives = basis.derivatives(u).expect("derivatives");
        assert!(relative(finite_d2(&basis, u, 1e-2), derivatives.d2) < 1e-6);
        assert!(relative(finite_d3(&basis, u, 2e-2), derivatives.d3) < 1e-5);
    }
}

#[test]
fn a_non_uniform_cubic_spline_matches_finite_differences() {
    let curve = NurbsCurve {
        degree: 3,
        poles: vec![
            [0.0, 0.0, 0.0],
            [2.0, 5.0, 0.0],
            [6.0, 7.5, 0.0],
            [9.0, 3.0, 0.0],
            [12.0, 6.0, 0.0],
            [15.0, 0.5, 0.0],
        ],
        weights: vec![1.0; 6],
        knots: vec![0.0, 0.0, 0.0, 0.0, 0.15, 0.55, 0.85, 1.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    };
    let basis = NurbsBasis::new(&curve);
    for u in [0.2, 0.45, 0.8] {
        let derivatives = basis.derivatives(u).expect("derivatives");
        assert!(relative(finite_d2(&basis, u, 1e-2), derivatives.d2) < 1e-5);
    }
    // The offset of that spline has the varying R regime as well.
    let offset = OffsetCurve::new(basis, 1.0, [0.0, 0.0, 1.0]).expect("offset");
    for u in [0.2, 0.5, 0.8] {
        let derivatives = offset.derivatives(u).expect("derivatives");
        assert!(relative(finite_d2(&offset, u, 1e-2), derivatives.d2) < 1e-5);
        assert!(relative(finite_d3(&offset, u, 1e-2), derivatives.d3) < 1e-3);
    }
}

#[test]
fn a_direction_parallel_to_the_tangent_is_unsupported() {
    let basis = LineBasis::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]).expect("line");
    let offset = OffsetCurve::new(basis, 1.0, [0.0, 0.0, 1.0]).expect("offset");
    assert!(matches!(
        offset.derivatives(0.5),
        Err(Error::Unsupported("offset direction parallel to the tangent"))
    ));
}

#[test]
fn invalid_inputs_are_rejected() {
    let basis = LineBasis::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]).expect("line");
    assert!(matches!(
        OffsetCurve::new(basis, f64::NAN, [0.0, 0.0, 1.0]),
        Err(Error::Invalid("offset distance"))
    ));
    assert!(matches!(
        OffsetCurve::new(basis, 1.0, [0.0, 0.0, 0.0]),
        Err(Error::Invalid("offset basis derivative"))
    ));
    assert!(matches!(
        CircleBasis::new([0.0; 3], [0.0, 0.0, 1.0], 0.0),
        Err(Error::Invalid("circle radius"))
    ));
}

#[test]
fn deviation_between_identical_offsets_is_zero() {
    let basis = WarpedCircle {
        radius: 3.0,
        warp: 0.4,
    };
    let first = OffsetCurve::new(basis, 1.0, [0.0, 0.0, 1.0]).expect("offset");
    let second = OffsetCurve::new(basis, 1.0, [0.0, 0.0, 1.0]).expect("offset");
    assert!(first.deviation_against(&second, 33).expect("deviation") < 1e-12);
    let shifted = OffsetCurve::new(basis, 1.5, [0.0, 0.0, 1.0]).expect("offset");
    assert!(first.deviation_against(&shifted, 33).expect("deviation") > 0.4);
}
