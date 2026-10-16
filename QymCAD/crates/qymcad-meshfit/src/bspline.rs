//! A FREE FORM FITTED TO THE POINTS OF A REGION: one bicubic B-spline surface over the whole region, as reverse
//! engineering programs lay a smooth wall - not a mosaic of the primitives that lie on it strip by strip. The region is
//! laid flat by projecting it on the plane square to its mean normal, the surface is fitted to its corners by least
//! squares, and its spans are doubled until every corner lies within the tolerance.

use nalgebra::{DMatrix, DVector, Vector3};

/// A bicubic B-spline surface: clamped knots in each direction, the poles row by row (`nu` of them along u in a row,
/// `nv` rows), as the kernel's `Geom_BSplineSurface` takes them.
#[derive(Clone, Debug, PartialEq)]
pub struct BSpline {
    pub knots_u: Vec<f64>,
    pub knots_v: Vec<f64>,
    pub nu: usize,
    pub nv: usize,
    pub poles: Vec<[f64; 3]>,
    /// HOW THE REGION WAS LAID FLAT: the two axes of the plane (`frame[0..3]`, `frame[3..6]`) and the start and length
    /// of the region along each (`frame[6..10]`). A point q of the region sits at u = (q.e1 - u0) / du, v = (q.e2 - v0)
    /// / dv, where the surface passes within the tolerance of it - so an edge of the region is laid on the surface by the
    /// same rule, with no search.
    pub frame: [f64; 10],
}

impl BSpline {
    /// Where point `q` of the region lies in the parameters of the surface.
    pub fn uv_of(&self, q: [f64; 3]) -> (f64, f64) {
        let f = &self.frame;
        ((q[0] * f[0] + q[1] * f[1] + q[2] * f[2] - f[6]) / f[7], (q[0] * f[3] + q[1] * f[4] + q[2] * f[5] - f[8]) / f[9])
    }
}

const DEGREE: usize = 3;

/// The most spans a direction is given: 21 poles across, 441 in all - past that a region is not one smooth wall, and
/// the solve grows as the cube of the poles.
const MOST_SPANS: usize = 18;

/// How far a normal of the region may turn from the mean before the region stops lying flat on the mean's plane: past a
/// right angle the projection folds the region over itself.
const MOST_TURN_DEG: f64 = 75.0;

/// Clamped uniform knots for `spans` spans: DEGREE+1 zeros, the inner knots, DEGREE+1 ones.
fn knots(spans: usize) -> Vec<f64> {
    let mut k = vec![0.0; DEGREE + 1];
    for i in 1..spans {
        k.push(i as f64 / spans as f64);
    }
    k.extend(std::iter::repeat_n(1.0, DEGREE + 1));
    k
}

/// The DEGREE+1 basis functions not zero at `t`, and the index of the first of them (Cox - de Boor).
fn basis(knots: &[f64], n: usize, t: f64) -> (usize, [f64; DEGREE + 1]) {
    let t = t.clamp(0.0, 1.0);
    // the span: the last knot interval starting at or before t, never past the last real one
    let mut span = DEGREE;
    while span + 1 < n && knots[span + 1] <= t {
        span += 1;
    }
    let mut out = [0.0; DEGREE + 1];
    out[0] = 1.0;
    let (mut left, mut right) = ([0.0; DEGREE + 1], [0.0; DEGREE + 1]);
    for j in 1..=DEGREE {
        left[j] = t - knots[span + 1 - j];
        right[j] = knots[span + j] - t;
        let mut saved = 0.0;
        for r in 0..j {
            let denom = right[r + 1] + left[j - r];
            let temp = if denom.abs() > 1e-300 { out[r] / denom } else { 0.0 };
            out[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        out[j] = saved;
    }
    (span - DEGREE, out)
}

impl BSpline {
    /// The point of the surface at (u, v), both in [0, 1].
    pub fn at(&self, u: f64, v: f64) -> [f64; 3] {
        let (iu, bu) = basis(&self.knots_u, self.nu, u);
        let (iv, bv) = basis(&self.knots_v, self.nv, v);
        let mut p = [0.0; 3];
        for (a, wu) in bu.iter().enumerate() {
            for (b, wv) in bv.iter().enumerate() {
                let q = self.poles[(iv + b) * self.nu + iu + a];
                for k in 0..3 {
                    p[k] += wu * wv * q[k];
                }
            }
        }
        p
    }
}

/// What a fit came to: the surface, and the largest distance of a point from it.
pub struct Fitted {
    pub surface: BSpline,
    pub worst: f64,
}

/// THE FREE FORM THROUGH `points`, facing `normals` (one per point, unit, pointing out), every point within `tol` of it;
/// `None` where the points do not lie flat enough to be laid on one plane, or no surface of up to `MOST_SPANS` spans
/// holds them within `tol`.
pub fn fit(points: &[[f64; 3]], normals: &[[f64; 3]], tol: f64) -> Option<Fitted> {
    fit_held(points, normals, points.len(), tol, tol)
}

/// The same, the first `firm` points held within `tol` and the rest within `loose`: the corners of a mesh lie on the
/// wall it was made from, while a point amid a flat triangle stands off it by the triangle's own sag.
pub fn fit_held(points: &[[f64; 3]], normals: &[[f64; 3]], firm: usize, tol: f64, loose: f64) -> Option<Fitted> {
    if points.len() < 16 {
        return None;
    }
    let mean = normals.iter().fold(Vector3::zeros(), |m, n| m + Vector3::from(*n)).try_normalize(1e-12)?;
    let cos_most = MOST_TURN_DEG.to_radians().cos();
    if normals.iter().any(|n| Vector3::from(*n).dot(&mean) < cos_most) {
        return None;
    }
    // THE REGION LAID FLAT: two axes square to the mean normal, the points' coordinates along them scaled into [0, 1]
    let helper = if mean.x.abs() < 0.9 { Vector3::x() } else { Vector3::y() };
    let e1 = mean.cross(&helper).normalize();
    let e2 = mean.cross(&e1);
    let flat: Vec<(f64, f64)> = points.iter().map(|p| (Vector3::from(*p).dot(&e1), Vector3::from(*p).dot(&e2))).collect();
    let (u0, u1) = flat.iter().fold((f64::MAX, f64::MIN), |(lo, hi), q| (lo.min(q.0), hi.max(q.0)));
    let (v0, v1) = flat.iter().fold((f64::MAX, f64::MIN), |(lo, hi), q| (lo.min(q.1), hi.max(q.1)));
    let (du, dv) = ((u1 - u0).max(1e-12), (v1 - v0).max(1e-12));
    let uv: Vec<(f64, f64)> = flat.iter().map(|q| ((q.0 - u0) / du, (q.1 - v0) / dv)).collect();
    // spans in proportion to the region's extent, doubled until it holds
    let aspect = (du / dv).clamp(0.25, 4.0);
    let mut spans = 1usize;
    loop {
        let su = ((spans as f64 * aspect.sqrt()).round() as usize).clamp(1, MOST_SPANS);
        let sv = ((spans as f64 / aspect.sqrt()).round() as usize).clamp(1, MOST_SPANS);
        let mut surface = solve(points, &uv, firm, su, sv)?;
        surface.frame = [e1.x, e1.y, e1.z, e2.x, e2.y, e2.z, u0, du, v0, dv];
        let off = |k: usize| dist(surface.at(uv[k].0, uv[k].1), points[k]);
        let worst = (0..firm.min(points.len())).map(off).fold(0.0, f64::max);
        let worst_amid = (firm.min(points.len())..points.len()).map(off).fold(0.0, f64::max);
        if worst <= tol && worst_amid <= loose {
            return Some(Fitted { surface, worst });
        }
        if su >= MOST_SPANS && sv >= MOST_SPANS {
            return None;
        }
        spans *= 2;
    }
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// How much a point amid a triangle weighs against a corner. It stands off the wall by the triangle's sag, always to the
/// same side, and at full weight it pulled the surface off the corners by half of that: measured on a vault of 60 x 60
/// cells, the corners stayed 0.0062 off every surface up to 18 x 18 spans, against a sag of 0.0055. Its weight is to
/// keep the surface from bowing across a big triangle, not to hold it.
const AMID_WEIGHT: f64 = 0.05;

/// The least-squares surface of `su` x `sv` spans through the points at `uv` - the first `firm` of them at full weight,
/// the rest at `AMID_WEIGHT` - kept smooth where the region leaves the square of its parameters empty: the second
/// differences of the poles along each direction weigh in a little, so a pole no point reaches follows its neighbours
/// rather than being left undetermined.
fn solve(points: &[[f64; 3]], uv: &[(f64, f64)], firm: usize, su: usize, sv: usize) -> Option<BSpline> {
    let (ku, kv) = (knots(su), knots(sv));
    let (nu, nv) = (su + DEGREE, sv + DEGREE);
    let n = nu * nv;
    let mut ata = DMatrix::<f64>::zeros(n, n);
    let mut atb = [DVector::<f64>::zeros(n), DVector::<f64>::zeros(n), DVector::<f64>::zeros(n)];
    for (k, (p, &(u, v))) in points.iter().zip(uv).enumerate() {
        let weight = if k < firm { 1.0 } else { AMID_WEIGHT };
        let (iu, bu) = basis(&ku, nu, u);
        let (iv, bv) = basis(&kv, nv, v);
        let mut idx = [(0usize, 0.0f64); (DEGREE + 1) * (DEGREE + 1)];
        let mut m = 0;
        for (b, wv) in bv.iter().enumerate() {
            for (a, wu) in bu.iter().enumerate() {
                idx[m] = ((iv + b) * nu + iu + a, wu * wv);
                m += 1;
            }
        }
        for &(i, wi) in &idx {
            for &(j, wj) in &idx {
                ata[(i, j)] += weight * wi * wj;
            }
            for (c, b) in atb.iter_mut().enumerate() {
                b[i] += weight * wi * p[c];
            }
        }
    }
    // the smoothing: a millionth of the fit's weight per point, on the second differences of the poles
    let lambda = 1e-6 * points.len() as f64 / n as f64;
    let mut bend = |a: usize, b: usize, c: usize| {
        for (i, wi) in [(a, 1.0), (b, -2.0), (c, 1.0)] {
            for (j, wj) in [(a, 1.0), (b, -2.0), (c, 1.0)] {
                ata[(i, j)] += lambda * wi * wj;
            }
        }
    };
    for r in 0..nv {
        for c in 1..nu - 1 {
            bend(r * nu + c - 1, r * nu + c, r * nu + c + 1);
        }
    }
    for c in 0..nu {
        for r in 1..nv - 1 {
            bend((r - 1) * nu + c, r * nu + c, (r + 1) * nu + c);
        }
    }
    let chol = ata.cholesky()?;
    let xs: Vec<DVector<f64>> = atb.iter().map(|b| chol.solve(b)).collect();
    let poles = (0..n).map(|i| [xs[0][i], xs[1][i], xs[2][i]]).collect();
    Some(BSpline { knots_u: ku, knots_v: kv, nu, nv, poles, frame: [0.0; 10] })
}
