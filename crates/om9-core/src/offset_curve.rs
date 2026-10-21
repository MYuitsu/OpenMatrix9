//! Offset curves owned by OM9, not by the kernel.
//!
//! The offset of a curve is `P(u) = p(u) + d * Â(u)` with `Â = A / |A|` and `A = p′(u) × V`.
//! OCCT 8.0.1 computes `D2`/`D3` of that expression with a wrong expansion (its `D2` normal term has
//! the wrong sign and its `D3` expansion carries a spurious `6·Dr²/R⁴`, a `6` where the derivation
//! gives `9`, and a `D3r` missing the `1/R²` factor), which is why the local `TKG3d` patch exists.
//!
//! This module keeps the derivative contract in Rust: `OffsetCurve` evaluates `D0..D3` from the
//! basis curve's `D0..D4` with the closed form below, so the application never depends on the
//! kernel's offset derivative code, and the accuracy tests here use an oracle that does not come
//! from the same expression (analytic circles, and finite differences of the point).
//!
//! ```text
//! A = p′ × V,  R = |A|,  Dr = A·A′,  D2r = A·A″ + A′·A′,  D3r = A·A‴ + 3·A′·A″
//!
//! P   = p   + d·(A/R)
//! P′  = p′  + d·(A′/R − A·Dr/R³)
//! P″  = p″  + d·(A″/R − 2·A′·Dr/R³ − A·D2r/R³ + 3·A·Dr²/R⁵)
//! P‴  = p‴  + d·(A‴/R − 3·A″·Dr/R³ − 3·A′·D2r/R³ + 9·A′·Dr²/R⁵
//!                 − A·D3r/R³ + 9·A·Dr·D2r/R⁵ − 15·A·Dr³/R⁷)
//! ```
//!
//! `P′..P‴` need the basis `D1..D4`; `D4` enters only through `A‴ = p⁗ × V`.

use crate::error::{Error, Result};
use crate::geometry::{NurbsCurve, Point3};

/// Basis derivatives up to the fourth order, which is what the offset needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Derivatives {
    pub point: Point3,
    pub d1: Point3,
    pub d2: Point3,
    pub d3: Point3,
    pub d4: Point3,
}

/// A curve that can report its derivatives. Implementors own their own parameterisation; the offset
/// never changes it.
pub trait CurveBasis {
    fn domain(&self) -> [f64; 2];
    fn derivatives(&self, u: f64) -> Result<Derivatives>;
}

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: Point3, factor: f64) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: Point3) -> f64 {
    dot(a, a).sqrt()
}

fn unit(a: Point3) -> Result<Point3> {
    let length = norm(a);
    if !length.is_finite() || length <= f64::MIN_POSITIVE {
        return Err(Error::Invalid("offset basis derivative"));
    }
    Ok(scale(a, 1.0 / length))
}

/// Straight line `origin + u * direction`. Derivatives above the first are zero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineBasis {
    pub origin: Point3,
    pub direction: Point3,
    pub domain: [f64; 2],
}

impl LineBasis {
    pub fn new(origin: Point3, direction: Point3) -> Result<Self> {
        let direction = unit(direction)?;
        Ok(Self {
            origin,
            direction,
            domain: [f64::NEG_INFINITY, f64::INFINITY],
        })
    }
}

impl CurveBasis for LineBasis {
    fn domain(&self) -> [f64; 2] {
        self.domain
    }

    fn derivatives(&self, u: f64) -> Result<Derivatives> {
        if !u.is_finite() {
            return Err(Error::Invalid("offset basis parameter"));
        }
        Ok(Derivatives {
            point: add(self.origin, scale(self.direction, u)),
            d1: self.direction,
            d2: [0.0; 3],
            d3: [0.0; 3],
            d4: [0.0; 3],
        })
    }
}

/// Circle `centre + radius * (cos u * x + sin u * y)`, parameterised by the angle in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircleBasis {
    pub centre: Point3,
    /// Unit vector in the plane, at `u = 0`.
    pub x: Point3,
    /// Unit vector in the plane, at `u = π/2`.
    pub y: Point3,
    pub radius: f64,
}

impl CircleBasis {
    pub fn new(centre: Point3, normal: Point3, radius: f64) -> Result<Self> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(Error::Invalid("circle radius"));
        }
        let normal = unit(normal)?;
        let candidate = if normal[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let x = unit(cross(normal, candidate))?;
        let y = cross(normal, x);
        Ok(Self {
            centre,
            x,
            y,
            radius,
        })
    }
}

impl CurveBasis for CircleBasis {
    fn domain(&self) -> [f64; 2] {
        [0.0, std::f64::consts::TAU]
    }

    fn derivatives(&self, u: f64) -> Result<Derivatives> {
        if !u.is_finite() {
            return Err(Error::Invalid("offset basis parameter"));
        }
        let (sin, cos) = u.sin_cos();
        let radial = add(scale(self.x, cos), scale(self.y, sin));
        let tangent = add(scale(self.x, -sin), scale(self.y, cos));
        let r = self.radius;
        Ok(Derivatives {
            point: add(self.centre, scale(radial, r)),
            d1: scale(tangent, r),
            d2: scale(radial, -r),
            d3: scale(tangent, -r),
            d4: scale(radial, r),
        })
    }
}

/// Rational B-spline basis reading a validated [`NurbsCurve`] (poles, weights, expanded knots).
///
/// The homogeneous curve `(w·p, w)` is sampled with the basis derivatives, then the Euclidean
/// derivatives follow from the quotient rule, which is exact for any weight vector.
#[derive(Debug, Clone, Copy)]
pub struct NurbsBasis<'a> {
    pub curve: &'a NurbsCurve,
}

impl<'a> NurbsBasis<'a> {
    pub fn new(curve: &'a NurbsCurve) -> Self {
        Self { curve }
    }

    /// Knot value, clamped outside the vector so the recursions cannot index past the ends.
    fn knot(&self, index: isize) -> f64 {
        let knots = &self.curve.knots;
        if index < 0 {
            knots[0]
        } else if index as usize >= knots.len() {
            knots[knots.len() - 1]
        } else {
            knots[index as usize]
        }
    }

    /// Cox–de Boor value of basis function `index` at degree `degree`. The right endpoint of the
    /// domain belongs to the last non-empty span, so `u == last knot` returns 1 for the last index of
    /// that degree instead of dropping every basis function.
    fn basis_value(&self, index: usize, degree: usize, u: f64) -> f64 {
        if degree == 0 {
            let count = self.curve.poles.len();
            let last = count - 1;
            if index == last && u >= self.knot(index as isize) {
                return if u <= self.knot(index as isize + 1) {
                    1.0
                } else {
                    0.0
                };
            }
            let left = self.knot(index as isize);
            let right = self.knot(index as isize + 1);
            return if u >= left && u < right { 1.0 } else { 0.0 };
        }
        let mut value = 0.0;
        let denominator = self.knot(index as isize + degree as isize) - self.knot(index as isize);
        if denominator != 0.0 {
            value += (u - self.knot(index as isize)) / denominator
                * self.basis_value(index, degree - 1, u);
        }
        let denominator =
            self.knot(index as isize + degree as isize + 1) - self.knot(index as isize + 1);
        if denominator != 0.0 {
            value += (self.knot(index as isize + degree as isize + 1) - u) / denominator
                * self.basis_value(index + 1, degree - 1, u);
        }
        value
    }

    /// `N_{i,p}^{(order)}(u)` through the alpha-coefficient form of the derivative recursion:
    /// `N_{i,p}^{(k)} = p!/(p-k)! · Σ_j α_{k,j} · N_{i+j,p-k}`.
    fn basis_derivative(&self, index: usize, order: usize, u: f64) -> f64 {
        let degree = self.curve.degree;
        if order == 0 {
            return self.basis_value(index, degree, u);
        }
        if order > degree {
            return 0.0;
        }
        let mut coefficients = vec![1.0f64];
        for k in 1..=order {
            let mut next = vec![0.0f64; k + 1];
            next[0] = self.ratio(coefficients[0], index + degree - k + 1, index);
            for j in 1..k {
                next[j] = self.ratio(
                    coefficients[j] - coefficients[j - 1],
                    index + degree + j - k + 1,
                    index + j,
                );
            }
            next[k] = self.ratio(-coefficients[k - 1], index + degree + 1, index + k);
            coefficients = next;
        }
        let mut factor = 1.0;
        for step in 0..order {
            factor *= (degree - step) as f64;
        }
        let mut sum = 0.0;
        for (offset, coefficient) in coefficients.iter().enumerate() {
            sum += coefficient * self.basis_value(index + offset, degree - order, u);
        }
        factor * sum
    }

    /// `numerator / (knots[high] - knots[low])`, with a vanished denominator meaning the term is
    /// absent (the alpha recursion drops it rather than dividing by zero).
    fn ratio(&self, numerator: f64, high: usize, low: usize) -> f64 {
        let difference = self.knot(high as isize) - self.knot(low as isize);
        if difference == 0.0 {
            0.0
        } else {
            numerator / difference
        }
    }

    /// `N_{i,p}^{(order)}(u)` for every pole index.
    fn basis_derivatives(&self, u: f64, order: usize) -> Result<Vec<f64>> {
        let count = self.curve.poles.len();
        let mut values = vec![0.0; count];
        if order > self.curve.degree {
            return Ok(values);
        }
        for (index, value) in values.iter_mut().enumerate() {
            *value = self.basis_derivative(index, order, u);
        }
        Ok(values)
    }
}
impl CurveBasis for NurbsBasis<'_> {
    fn domain(&self) -> [f64; 2] {
        self.curve.domain
    }

    fn derivatives(&self, u: f64) -> Result<Derivatives> {
        if !u.is_finite() || u < self.curve.domain[0] || u > self.curve.domain[1] {
            return Err(Error::Invalid("offset basis parameter"));
        }
        // Homogeneous derivatives: A^(r) and w^(r).
        let mut homogeneous = [[0.0; 4]; 5];
        for (order, level) in homogeneous.iter_mut().enumerate() {
            let weights = self.basis_derivatives(u, order)?;
            for (index, weight) in weights.iter().enumerate() {
                let pole = self.curve.poles[index];
                let pole_weight = self.curve.weights[index];
                for (axis, component) in level.iter_mut().take(3).enumerate() {
                    *component += weight * pole_weight * pole[axis];
                }
                level[3] += weight * pole_weight;
            }
        }
        let w = homogeneous[0][3];
        if w == 0.0 {
            return Err(Error::Invalid("rational curve weight"));
        }
        let a = |order: usize| -> Point3 {
            [
                homogeneous[order][0],
                homogeneous[order][1],
                homogeneous[order][2],
            ]
        };
        let dw = |order: usize| homogeneous[order][3];
        // Quotient rule on C = A/w for orders 0..4.
        let p = scale(a(0), 1.0 / w);
        let d1 = scale(sub(a(1), scale(p, dw(1))), 1.0 / w);
        let d2 = scale(
            sub(sub(a(2), scale(d1, 2.0 * dw(1))), scale(p, dw(2))),
            1.0 / w,
        );
        let d3 = scale(
            sub(
                sub(sub(a(3), scale(d2, 3.0 * dw(1))), scale(d1, 3.0 * dw(2))),
                scale(p, dw(3)),
            ),
            1.0 / w,
        );
        let d4 = scale(
            sub(
                sub(
                    sub(sub(a(4), scale(d3, 4.0 * dw(1))), scale(d2, 6.0 * dw(2))),
                    scale(d1, 4.0 * dw(3)),
                ),
                scale(p, dw(4)),
            ),
            1.0 / w,
        );
        Ok(Derivatives {
            point: p,
            d1,
            d2,
            d3,
            d4,
        })
    }
}

/// `P(u) = p(u) + distance * (p′(u) × direction) / |p′(u) × direction|`.
#[derive(Debug, Clone, Copy)]
pub struct OffsetCurve<B> {
    pub basis: B,
    pub distance: f64,
    /// Unit offset direction (the plane normal for a planar basis curve).
    pub direction: Point3,
}

impl<B: CurveBasis> OffsetCurve<B> {
    pub fn new(basis: B, distance: f64, direction: Point3) -> Result<Self> {
        if !distance.is_finite() {
            return Err(Error::Invalid("offset distance"));
        }
        Ok(Self {
            basis,
            distance,
            direction: unit(direction)?,
        })
    }

    pub fn domain(&self) -> [f64; 2] {
        self.basis.domain()
    }

    /// Offset point and derivatives up to the third order.
    ///
    /// Fails with [`Error::Unsupported`] when the tangent is parallel to the offset direction: the
    /// offset direction is undefined there and the kernel refuses the same way.
    pub fn derivatives(&self, u: f64) -> Result<Derivatives> {
        let basis = self.basis.derivatives(u)?;
        let d = self.distance;
        let v = self.direction;
        let cross_all = |source: Point3| cross(source, v);
        let a0 = cross_all(basis.d1);
        let a1 = cross_all(basis.d2);
        let a2 = cross_all(basis.d3);
        let a3 = cross_all(basis.d4);
        let big_r = norm(a0);
        if !big_r.is_finite() || big_r <= 1e-12 {
            return Err(Error::Unsupported(
                "offset direction parallel to the tangent",
            ));
        }
        let r2 = big_r * big_r;
        let r3 = r2 * big_r;
        let r5 = r3 * r2;
        let r7 = r5 * r2;
        let dr = dot(a0, a1);
        let d2r = dot(a0, a2) + dot(a1, a1);
        let d3r = dot(a0, a3) + 3.0 * dot(a1, a2);

        let point = add(basis.point, scale(a0, d / big_r));
        let d1 = add(
            basis.d1,
            scale(sub(scale(a1, 1.0 / big_r), scale(a0, dr / r3)), d),
        );
        let d2 = add(
            basis.d2,
            scale(
                add(
                    add(
                        sub(scale(a2, 1.0 / big_r), scale(a1, 2.0 * dr / r3)),
                        scale(a0, 3.0 * dr * dr / r5),
                    ),
                    scale(a0, -d2r / r3),
                ),
                d,
            ),
        );
        let d3 = add(
            basis.d3,
            scale(
                add(
                    add(
                        add(
                            sub(scale(a3, 1.0 / big_r), scale(a2, 3.0 * dr / r3)),
                            scale(a1, 9.0 * dr * dr / r5 - 3.0 * d2r / r3),
                        ),
                        scale(a0, 9.0 * dr * d2r / r5 - 15.0 * dr * dr * dr / r7),
                    ),
                    scale(a0, -d3r / r3),
                ),
                d,
            ),
        );
        Ok(Derivatives {
            point,
            d1,
            d2,
            d3,
            d4: [0.0; 3],
        })
    }

    pub fn point(&self, u: f64) -> Result<Point3> {
        Ok(self.derivatives(u)?.point)
    }

    /// Curvature `|P′ × P″| / |P′|³` of the offset curve.
    pub fn curvature(&self, u: f64) -> Result<f64> {
        let derivatives = self.derivatives(u)?;
        let speed = norm(derivatives.d1);
        if speed <= f64::MIN_POSITIVE {
            return Err(Error::Unsupported("offset curve speed"));
        }
        Ok(norm(cross(derivatives.d1, derivatives.d2)) / (speed * speed * speed))
    }

    /// Largest distance between this offset and another curve, sampled at `samples` parameters.
    /// Used to accept a refit that has to become a B-Rep curve.
    pub fn deviation_against(&self, other: &impl CurveBasis, samples: usize) -> Result<f64> {
        if samples < 2 {
            return Err(Error::Invalid("deviation samples"));
        }
        let [start, end] = self.domain();
        let mut worst: f64 = 0.0;
        for index in 0..samples {
            let t = start + (end - start) * (index as f64 / (samples - 1) as f64);
            let mine = self.point(t)?;
            let theirs = other.derivatives(t)?.point;
            worst = worst.max(norm(sub(mine, theirs)));
        }
        Ok(worst)
    }
}

impl<B: CurveBasis> CurveBasis for OffsetCurve<B> {
    fn domain(&self) -> [f64; 2] {
        self.basis.domain()
    }

    fn derivatives(&self, u: f64) -> Result<Derivatives> {
        OffsetCurve::derivatives(self, u)
    }
}
