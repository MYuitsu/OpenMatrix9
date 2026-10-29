// SPDX-License-Identifier: LGPL-2.1-or-later
//! Surface kernel (OM9-L2-03 wave W3): evaluation, exact iso-curves, refit and the
//! analytic builders that snapping needs.
//!
//! Ported from the legacy bridge (`rust/src/{support_snapshot,analytic_support,surface_refit,
//! derived_support}.rs`) into safe Rust. The legacy C ABI and the command sessions stay
//! behind; the portable arithmetic is re-implemented against `NurbsSurface` and every
//! constructor returns a validated surface.
use crate::{
    Error, Result,
    budget::Limits,
    curve_build::{KnotMode, interpolate_any},
    geometry::{Curve, NurbsCurve, NurbsSurface, Point3, finite},
};

/// Which direction an iso-curve runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceIso {
    /// Vary `u`, holding `v` fixed.
    U,
    /// Vary `v`, holding `u` fixed.
    V,
}

/// A surface point with its two first partial derivatives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceDerivatives {
    pub point: Point3,
    pub du: Point3,
    pub dv: Point3,
}

fn check_parameter(surface: &NurbsSurface, axis: usize, value: f64) -> Result<()> {
    let [low, high] = surface.domain[axis];
    if !value.is_finite() || value < low || value > high {
        return Err(Error::Invalid("surface parameter"));
    }
    Ok(())
}

/// Basis values `N_i(t)` for every pole in `axis`.
fn basis_row(surface: &NurbsSurface, axis: usize, t: f64) -> Result<Vec<f64>> {
    let degree = surface.degree[axis];
    let mut values = Vec::new();
    values
        .try_reserve_exact(surface.size[axis])
        .map_err(|_| Error::Budget("surface basis row"))?;
    for index in 0..surface.size[axis] {
        values.push(crate::curve_build::basis(
            degree,
            &surface.knots[axis],
            index,
            t,
        )?);
    }
    Ok(values)
}

/// First derivative `N'_i(t)` of every basis function in `axis`.
fn basis_row_derivative(surface: &NurbsSurface, axis: usize, t: f64) -> Result<Vec<f64>> {
    let degree = surface.degree[axis];
    let knots = &surface.knots[axis];
    let mut values = Vec::new();
    values
        .try_reserve_exact(surface.size[axis])
        .map_err(|_| Error::Budget("surface basis row"))?;
    for index in 0..surface.size[axis] {
        let mut value = 0.0;
        if degree > 0 {
            let left_span = knots[index + degree] - knots[index];
            if left_span > 0.0 {
                value += degree as f64 / left_span
                    * crate::curve_build::basis(degree - 1, knots, index, t)?;
            }
            let right_span = knots[index + degree + 1] - knots[index + 1];
            if right_span > 0.0 && index + 1 < surface.size[axis] {
                value -= degree as f64 / right_span
                    * crate::curve_build::basis(degree - 1, knots, index + 1, t)?;
            }
        }
        values.push(value);
    }
    Ok(values)
}

fn pole(surface: &NurbsSurface, u: usize, v: usize) -> Point3 {
    surface.poles[u * surface.size[1] + v]
}

fn weight(surface: &NurbsSurface, u: usize, v: usize) -> f64 {
    surface.weights[u * surface.size[1] + v]
}

/// The Greville parameter of pole `index` on one axis.
pub fn greville_parameter(surface: &NurbsSurface, axis: usize, index: usize) -> f64 {
    let degree = surface.degree[axis];
    (1..=degree)
        .map(|offset| surface.knots[axis][index + offset])
        .sum::<f64>()
        / degree as f64
}

/// All surface edit points in one tensor contraction: O(n_u n_v (n_u + n_v)) instead of
/// evaluating each grid point against every pole. Row-major, the same order as `poles`.
pub fn edit_points(surface: &NurbsSurface, limits: &Limits) -> Result<Vec<Point3>> {
    surface.validate(limits)?;
    let [nu_count, nv_count] = surface.size;
    let mut nu_rows = Vec::with_capacity(nu_count);
    for i in 0..nu_count {
        nu_rows.push(basis_row(surface, 0, greville_parameter(surface, 0, i))?);
    }
    let mut nv_rows = Vec::with_capacity(nv_count);
    for j in 0..nv_count {
        nv_rows.push(basis_row(surface, 1, greville_parameter(surface, 1, j))?);
    }
    let mut weighted = vec![[0.0; 3]; nu_count * nv_count];
    let mut denominators = vec![0.0; nu_count * nv_count];
    for k in 0..nu_count {
        for j in 0..nv_count {
            let mut numerator = [0.0; 3];
            let mut denominator = 0.0;
            for (l, nv) in nv_rows[j].iter().enumerate() {
                let coefficient = nv * weight(surface, k, l);
                denominator += coefficient;
                let point = pole(surface, k, l);
                for axis in 0..3 {
                    numerator[axis] += coefficient * point[axis];
                }
            }
            weighted[k * nv_count + j] = numerator;
            denominators[k * nv_count + j] = denominator;
        }
    }
    let mut points = Vec::with_capacity(nu_count * nv_count);
    for nu in &nu_rows {
        for j in 0..nv_count {
            let mut numerator = [0.0; 3];
            let mut denominator = 0.0;
            for (k, coefficient) in nu.iter().enumerate() {
                denominator += coefficient * denominators[k * nv_count + j];
                let value = weighted[k * nv_count + j];
                for axis in 0..3 {
                    numerator[axis] += coefficient * value[axis];
                }
            }
            if !denominator.is_finite() || denominator <= 0.0 {
                return Err(Error::Invalid("edit-point surface denominator"));
            }
            let point = [
                numerator[0] / denominator,
                numerator[1] / denominator,
                numerator[2] / denominator,
            ];
            finite(point)?;
            points.push(point);
        }
    }
    Ok(points)
}

/// The rational basis influence of one pole at its own Greville point:
/// `N_i(u_i) N_j(v_j) w_ij / W(u_i, v_j)`.
pub fn edit_influence(surface: &NurbsSurface, i: usize, j: usize, limits: &Limits) -> Result<f64> {
    surface.validate(limits)?;
    if i >= surface.size[0] || j >= surface.size[1] {
        return Err(Error::Invalid("surface edit index"));
    }
    let nu = basis_row(surface, 0, greville_parameter(surface, 0, i))?;
    let nv = basis_row(surface, 1, greville_parameter(surface, 1, j))?;
    let mut denominator = 0.0;
    for (k, nu_k) in nu.iter().enumerate() {
        for (l, nv_l) in nv.iter().enumerate() {
            denominator += nu_k * nv_l * weight(surface, k, l);
        }
    }
    let own = nu[i] * nv[j] * weight(surface, i, j);
    if !denominator.is_finite() || denominator.abs() <= 1e-15 || !own.is_finite() {
        return Err(Error::Invalid("surface edit influence"));
    }
    let influence = own / denominator;
    if influence.abs() <= 1e-15 {
        return Err(Error::Invalid("surface edit influence"));
    }
    Ok(influence)
}

/// `S(u_i, v_j)` for one edit point, without materialising the whole grid.
pub fn edit_point(surface: &NurbsSurface, i: usize, j: usize, limits: &Limits) -> Result<Point3> {
    if i >= surface.size[0] || j >= surface.size[1] {
        return Err(Error::Invalid("surface edit index"));
    }
    evaluate(
        surface,
        greville_parameter(surface, 0, i),
        greville_parameter(surface, 1, j),
        limits,
    )
}

/// The surface point `S(u, v)` with rational weights applied.
pub fn evaluate(surface: &NurbsSurface, u: f64, v: f64, limits: &Limits) -> Result<Point3> {
    surface.validate(limits)?;
    check_parameter(surface, 0, u)?;
    check_parameter(surface, 1, v)?;
    let nu = basis_row(surface, 0, u)?;
    let nv = basis_row(surface, 1, v)?;
    let mut numerator = [0.0; 3];
    let mut denominator = 0.0;
    for (i, ni) in nu.iter().enumerate() {
        if *ni == 0.0 {
            continue;
        }
        for (j, nj) in nv.iter().enumerate() {
            let coefficient = ni * nj;
            if coefficient == 0.0 {
                continue;
            }
            let w = weight(surface, i, j);
            let p = pole(surface, i, j);
            denominator += coefficient * w;
            for axis in 0..3 {
                numerator[axis] += coefficient * w * p[axis];
            }
        }
    }
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(Error::Invalid("homogeneous surface denominator"));
    }
    let point = [
        numerator[0] / denominator,
        numerator[1] / denominator,
        numerator[2] / denominator,
    ];
    finite(point)?;
    Ok(point)
}

/// `S(u,v)`, `S_u` and `S_v`, with the quotient rule applied to the rational form.
pub fn derivatives(
    surface: &NurbsSurface,
    u: f64,
    v: f64,
    limits: &Limits,
) -> Result<SurfaceDerivatives> {
    surface.validate(limits)?;
    check_parameter(surface, 0, u)?;
    check_parameter(surface, 1, v)?;
    let nu = basis_row(surface, 0, u)?;
    let nv = basis_row(surface, 1, v)?;
    let dnu = basis_row_derivative(surface, 0, u)?;
    let dnv = basis_row_derivative(surface, 1, v)?;
    let mut a = [0.0; 3];
    let mut du_a = [0.0; 3];
    let mut dv_a = [0.0; 3];
    let mut w = 0.0;
    let mut wu = 0.0;
    let mut wv = 0.0;
    for i in 0..surface.size[0] {
        for j in 0..surface.size[1] {
            let p = pole(surface, i, j);
            let weight_ij = weight(surface, i, j);
            let n_uv = nu[i] * nv[j];
            w += n_uv * weight_ij;
            wu += dnu[i] * nv[j] * weight_ij;
            wv += nu[i] * dnv[j] * weight_ij;
            for axis in 0..3 {
                a[axis] += n_uv * weight_ij * p[axis];
                du_a[axis] += dnu[i] * nv[j] * weight_ij * p[axis];
                dv_a[axis] += nu[i] * dnv[j] * weight_ij * p[axis];
            }
        }
    }
    if !w.is_finite() || w <= 0.0 {
        return Err(Error::Invalid("homogeneous surface denominator"));
    }
    let point = [a[0] / w, a[1] / w, a[2] / w];
    let du = std::array::from_fn(|axis| (du_a[axis] - point[axis] * wu) / w);
    let dv = std::array::from_fn(|axis| (dv_a[axis] - point[axis] * wv) / w);
    finite(point)?;
    finite(du)?;
    finite(dv)?;
    Ok(SurfaceDerivatives { point, du, dv })
}

/// The unit normal `S_u x S_v / |S_u x S_v|`. A point where the two tangents are
/// parallel has no normal and is refused instead of returning a zero vector.
pub fn normal(surface: &NurbsSurface, u: f64, v: f64, limits: &Limits) -> Result<Point3> {
    let derivatives = derivatives(surface, u, v, limits)?;
    let cross = cross_product(derivatives.du, derivatives.dv);
    let length = dot(cross, cross).sqrt();
    let scale =
        dot(derivatives.du, derivatives.du).sqrt() * dot(derivatives.dv, derivatives.dv).sqrt();
    if !length.is_finite() || length <= 1e-12 || !scale.is_finite() || length <= 1e-9 * scale {
        return Err(Error::Invalid("degenerate surface normal"));
    }
    Ok([cross[0] / length, cross[1] / length, cross[2] / length])
}

/// The exact iso-curve of a rational surface. Holding one parameter fixed leaves a
/// rational curve whose poles are the weighted combination of the fixed-direction row and
/// whose weights are the row's combined weights, so no geometry is approximated.
pub fn isocurve(
    surface: &NurbsSurface,
    iso: SurfaceIso,
    value: f64,
    limits: &Limits,
) -> Result<NurbsCurve> {
    surface.validate(limits)?;
    // `fixed` is the axis held constant, `varying` the axis of the resulting curve.
    let (fixed, varying) = match iso {
        SurfaceIso::U => (1, 0),
        SurfaceIso::V => (0, 1),
    };
    check_parameter(surface, fixed, value)?;
    let basis_values = basis_row(surface, fixed, value)?;
    let mut poles = Vec::new();
    let mut weights = Vec::new();
    poles
        .try_reserve_exact(surface.size[varying])
        .map_err(|_| Error::Budget("iso-curve poles"))?;
    weights
        .try_reserve_exact(surface.size[varying])
        .map_err(|_| Error::Budget("iso-curve weights"))?;
    for index in 0..surface.size[varying] {
        let mut denominator = 0.0;
        let mut numerator = [0.0; 3];
        for (row, coefficient) in basis_values.iter().enumerate() {
            if *coefficient == 0.0 {
                continue;
            }
            let (u, v) = if varying == 0 {
                (index, row)
            } else {
                (row, index)
            };
            let w = weight(surface, u, v);
            let p = pole(surface, u, v);
            denominator += coefficient * w;
            for axis in 0..3 {
                numerator[axis] += coefficient * w * p[axis];
            }
        }
        if !denominator.is_finite() || denominator <= 0.0 {
            return Err(Error::Invalid("homogeneous iso-curve denominator"));
        }
        poles.push([
            numerator[0] / denominator,
            numerator[1] / denominator,
            numerator[2] / denominator,
        ]);
        weights.push(denominator);
    }
    let curve = NurbsCurve {
        degree: surface.degree[varying],
        poles,
        weights,
        knots: surface.knots[varying].clone(),
        domain: surface.domain[varying],
        periodic: false,
        reversed: false,
        seam: None,
    };
    curve.validate(limits)?;
    Ok(curve)
}

/// Refit a surface to a new degree and pole grid by sampling it and interpolating each
/// direction in turn (the legacy `surface_refit` idea: a rebuilt record, not an edit).
pub fn refit(
    surface: &NurbsSurface,
    degree: [usize; 2],
    size: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface.validate(limits)?;
    for axis in 0..2 {
        if size[axis] <= degree[axis] {
            return Err(Error::Invalid("surface refit grid"));
        }
    }
    // Sample the source on the target grid.
    let mut samples = vec![[0.0; 3]; size[0] * size[1]];
    for i in 0..size[0] {
        let u = parameter_at(surface, 0, i, size[0]);
        for j in 0..size[1] {
            let v = parameter_at(surface, 1, j, size[1]);
            samples[i * size[1] + j] = evaluate(surface, u, v, limits)?;
        }
    }
    // Interpolate each v-row along u, then each u-column along v.
    let mut rows: Vec<NurbsCurve> = Vec::new();
    rows.try_reserve_exact(size[1])
        .map_err(|_| Error::Budget("surface refit rows"))?;
    for j in 0..size[1] {
        let points: Vec<Point3> = (0..size[0]).map(|i| samples[i * size[1] + j]).collect();
        rows.push(interpolate_any(
            &points,
            degree[0],
            KnotMode::Uniform,
            limits,
        )?);
    }
    let knots_u = rows[0].knots.clone();
    let mut poles = vec![[0.0; 3]; size[0] * size[1]];
    let mut knots_v = None;
    for i in 0..size[0] {
        let points: Vec<Point3> = rows.iter().map(|row| row.poles[i]).collect();
        let column = interpolate_any(&points, degree[1], KnotMode::Uniform, limits)?;
        if knots_v.is_none() {
            knots_v = Some(column.knots.clone());
        }
        for (j, pole) in column.poles.iter().enumerate() {
            poles[i * size[1] + j] = *pole;
        }
    }
    let refit = NurbsSurface {
        degree,
        size,
        poles,
        weights: vec![1.0; size[0] * size[1]],
        knots: [
            knots_u,
            knots_v.ok_or(Error::Invalid("surface refit grid"))?,
        ],
        domain: surface.domain,
        periodic: [false, false],
        reversed: false,
    };
    refit.validate(limits)?;
    Ok(refit)
}

fn parameter_at(surface: &NurbsSurface, axis: usize, index: usize, count: usize) -> f64 {
    let [low, high] = surface.domain[axis];
    if count <= 1 {
        low
    } else {
        low + (high - low) * index as f64 / (count - 1) as f64
    }
}

/* ------------------------- analytic builders ------------------------- */

/// A planar surface on the frame `(origin, x, y)`, spanning `width` along `x` and `height`
/// along `y`. Degree 1 in both directions: an exact plane, and the reference the normal
/// test uses.
pub fn plane(
    origin: Point3,
    x: Point3,
    y: Point3,
    width: f64,
    height: f64,
    limits: &Limits,
) -> Result<NurbsSurface> {
    for point in [origin, x, y] {
        finite(point)?;
    }
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(Error::Invalid("plane size"));
    }
    if (dot(x, x) - 1.0).abs() > 1e-9 || (dot(y, y) - 1.0).abs() > 1e-9 || dot(x, y).abs() > 1e-9 {
        return Err(Error::Invalid("plane frame"));
    }
    let corner = |along_x: f64, along_y: f64| -> Point3 {
        [
            origin[0] + x[0] * along_x + y[0] * along_y,
            origin[1] + x[1] * along_x + y[1] * along_y,
            origin[2] + x[2] * along_x + y[2] * along_y,
        ]
    };
    let surface = NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles: vec![
            corner(0.0, 0.0),
            corner(0.0, height),
            corner(width, 0.0),
            corner(width, height),
        ],
        weights: vec![1.0; 4],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    surface.validate(limits)?;
    Ok(surface)
}

/// A right circular cylinder: exact rational circle in `u`, linear ruling in `v`. The axis
/// is `x cross y`, so a point at `(u, v)` has `|P x axis| = radius` and height `v * height`.
pub fn cylinder(
    origin: Point3,
    x: Point3,
    y: Point3,
    radius: f64,
    height: f64,
    limits: &Limits,
) -> Result<NurbsSurface> {
    for point in [origin, x, y] {
        finite(point)?;
    }
    if !radius.is_finite() || !height.is_finite() || radius <= 0.0 || height <= 0.0 {
        return Err(Error::Invalid("cylinder size"));
    }
    if (dot(x, x) - 1.0).abs() > 1e-9 || (dot(y, y) - 1.0).abs() > 1e-9 || dot(x, y).abs() > 1e-9 {
        return Err(Error::Invalid("cylinder frame"));
    }
    let half = std::f64::consts::FRAC_1_SQRT_2;
    let unit = [
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [-1.0, 1.0],
        [-1.0, 0.0],
        [-1.0, -1.0],
        [0.0, -1.0],
        [1.0, -1.0],
        [1.0, 0.0],
    ];
    let circle_weights = [1.0, half, 1.0, half, 1.0, half, 1.0, half, 1.0];
    let axis = cross_product(x, y);
    let mut poles = Vec::new();
    poles
        .try_reserve_exact(18)
        .map_err(|_| Error::Budget("cylinder poles"))?;
    let mut weights = Vec::new();
    weights
        .try_reserve_exact(18)
        .map_err(|_| Error::Budget("cylinder weights"))?;
    for (index, [cx, cy]) in unit.iter().enumerate() {
        for along in [0.0, height] {
            let along_x = cx * radius;
            let along_y = cy * radius;
            poles.push([
                origin[0] + x[0] * along_x + y[0] * along_y + axis[0] * along,
                origin[1] + x[1] * along_x + y[1] * along_y + axis[1] * along,
                origin[2] + x[2] * along_x + y[2] * along_y + axis[2] * along,
            ]);
            weights.push(circle_weights[index]);
        }
    }
    let surface = NurbsSurface {
        degree: [2, 1],
        size: [9, 2],
        poles,
        weights,
        knots: [
            vec![
                0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0,
            ],
            vec![0.0, 0.0, 1.0, 1.0],
        ],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    surface.validate(limits)?;
    Ok(surface)
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross_product(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Loft through cross-section curves (`L2-02-05 L3-002`): each section is resampled on a
/// common parameter grid and interpolated in `u`, then the resulting poles are
/// interpolated through the sections in `v`. The v poles interpolate the sections
/// exactly; a rational section becomes a non-rational approximation in `u`.
pub fn loft(
    sections: &[Curve],
    degree: [usize; 2],
    u_samples: usize,
    limits: &Limits,
) -> Result<NurbsSurface> {
    if sections.len() < 2 {
        return Err(Error::Invalid("loft section count"));
    }
    if sections.len() <= degree[1] {
        return Err(Error::Invalid("loft section count"));
    }
    if u_samples <= degree[0] {
        return Err(Error::Invalid("loft section samples"));
    }
    for section in sections {
        section.validate(limits)?;
    }
    // Stage A: resample every section on the same u grid and interpolate along u.
    let mut u_poles: Vec<Vec<Point3>> = Vec::new();
    u_poles
        .try_reserve_exact(sections.len())
        .map_err(|_| Error::Budget("loft sections"))?;
    let mut knots_u = None;
    for section in sections {
        let points = sample_curve(section, u_samples, limits)?;
        let row = interpolate_any(&points, degree[0], KnotMode::Uniform, limits)?;
        if knots_u.is_none() {
            knots_u = Some(row.knots.clone());
        }
        u_poles.push(row.poles);
    }
    // Stage B: interpolate the u poles through the sections in v.
    let mut poles = vec![[0.0; 3]; u_samples * sections.len()];
    let mut knots_v = None;
    for i in 0..u_samples {
        let column: Vec<Point3> = u_poles.iter().map(|row| row[i]).collect();
        let curve = interpolate_any(&column, degree[1], KnotMode::Uniform, limits)?;
        if knots_v.is_none() {
            knots_v = Some(curve.knots.clone());
        }
        for (j, pole) in curve.poles.iter().enumerate() {
            poles[i * sections.len() + j] = *pole;
        }
    }
    let surface = NurbsSurface {
        degree,
        size: [u_samples, sections.len()],
        poles,
        weights: vec![1.0; u_samples * sections.len()],
        knots: [
            knots_u.ok_or(Error::Invalid("loft section count"))?,
            knots_v.ok_or(Error::Invalid("loft section count"))?,
        ],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    surface.validate(limits)?;
    Ok(surface)
}

/// Sample any admitted curve at `count` stations.
///
/// A `Curve::Polycurve` cannot be evaluated with a single parameter (the evaluator
/// refuses it: "polycurve requires an explicit segment parameter"), so it is sampled
/// segment by segment on a uniform per-segment split; every other kind delegates to the
/// curve evaluator with a fraction parameter. When `count - 1` is a multiple of the
/// segment count, the seams are sampled exactly.
pub fn sample_curve(curve: &Curve, count: usize, limits: &Limits) -> Result<Vec<Point3>> {
    if count < 2 {
        return Err(Error::Invalid("sample count"));
    }
    curve.validate(limits)?;
    let mut points = Vec::new();
    points
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("curve samples"))?;
    match curve {
        Curve::Polycurve { segments, reversed } => {
            let total = segments.len();
            if total == 0 {
                return Err(Error::Invalid("polycurve segments"));
            }
            for index in 0..count {
                let fraction = index as f64 / (count - 1) as f64;
                let scaled = fraction * total as f64;
                let segment_index = (scaled as usize).min(total - 1);
                let local = scaled - segment_index as f64;
                let evaluated = crate::evaluation::evaluate_curve(
                    &segments[segment_index],
                    crate::evaluation::Parameter::Fraction(local),
                    0,
                    crate::geometry::Transform::identity(),
                    limits,
                )?;
                points.push(*evaluated.first().ok_or(Error::Invalid("curve sample"))?);
            }
            if *reversed {
                points.reverse();
            }
        }
        _ => {
            for index in 0..count {
                let fraction = index as f64 / (count - 1) as f64;
                let evaluated = crate::evaluation::evaluate_curve(
                    curve,
                    crate::evaluation::Parameter::Fraction(fraction),
                    0,
                    crate::geometry::Transform::identity(),
                    limits,
                )?;
                points.push(*evaluated.first().ok_or(Error::Invalid("curve sample"))?);
            }
        }
    }
    Ok(points)
}

/* ------------------------- sweep ------------------------- */

/// One station of a path frame: the path point, the two in-plane axes and the tangent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathFrame {
    pub origin: Point3,
    pub x: Point3,
    pub y: Point3,
    pub tangent: Point3,
}

fn unit_vector(value: Point3) -> Result<Point3> {
    let length = dot(value, value).sqrt();
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("degenerate path frame"));
    }
    Ok([value[0] / length, value[1] / length, value[2] / length])
}

/// Rotation-minimizing frames along a path (projection transport: each step removes the
/// tangential component of the previous `x` axis and renormalizes). Every frame is
/// orthonormal and perpendicular to the path tangent.
pub fn path_frames(path: &Curve, count: usize, limits: &Limits) -> Result<Vec<PathFrame>> {
    if count < 2 {
        return Err(Error::Invalid("path frame count"));
    }
    path.validate(limits)?;
    let mut points = Vec::new();
    let mut tangents = Vec::new();
    points
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("path frames"))?;
    tangents
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("path frames"))?;
    for index in 0..count {
        let fraction = index as f64 / (count - 1) as f64;
        let evaluated = match crate::evaluation::evaluate_curve(
            path,
            crate::evaluation::Parameter::Fraction(fraction),
            1,
            crate::geometry::Transform::identity(),
            limits,
        ) {
            Ok(evaluated) => evaluated,
            // A clamped NURBS seam can refuse D1 while the geometric tangent still exists.
            Err(Error::Unsupported(_)) => {
                let step = 1.0e-6;
                let low = (fraction - step).max(0.0);
                let high = (fraction + step).min(1.0);
                if high <= low {
                    return Err(Error::Invalid("path frame tangent"));
                }
                let start = crate::evaluation::evaluate_curve(
                    path,
                    crate::evaluation::Parameter::Fraction(low),
                    0,
                    crate::geometry::Transform::identity(),
                    limits,
                )?;
                let end = crate::evaluation::evaluate_curve(
                    path,
                    crate::evaluation::Parameter::Fraction(high),
                    0,
                    crate::geometry::Transform::identity(),
                    limits,
                )?;
                vec![
                    start[0],
                    [
                        end[0][0] - start[0][0],
                        end[0][1] - start[0][1],
                        end[0][2] - start[0][2],
                    ],
                ]
            }
            Err(error) => return Err(error),
        };
        if evaluated.len() < 2 {
            return Err(Error::Invalid("path frame tangent"));
        }
        points.push(evaluated[0]);
        tangents.push(unit_vector(evaluated[1])?);
    }
    // The initial x axis is any direction perpendicular to the first tangent; world axes
    // are tried in order so the choice is deterministic.
    let mut x = [0.0; 3];
    for candidate in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
        let projected = remove_along(candidate, tangents[0]);
        if dot(projected, projected) > 1e-12 {
            x = unit_vector(projected)?;
            break;
        }
    }
    let mut frames = Vec::new();
    frames
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("path frames"))?;
    for index in 0..count {
        if index > 0 {
            x = unit_vector(remove_along(x, tangents[index]))?;
        }
        let y = unit_vector(cross_product(tangents[index], x))?;
        frames.push(PathFrame {
            origin: points[index],
            x,
            y,
            tangent: tangents[index],
        });
    }
    Ok(frames)
}

fn remove_along(value: Point3, direction: Point3) -> Point3 {
    let projection = dot(value, direction);
    [
        value[0] - direction[0] * projection,
        value[1] - direction[1] * projection,
        value[2] - direction[2] * projection,
    ]
}

/// Sweep1: carry the profile rigidly along the rail using rotation-minimizing frames.
/// The profile is taken in the initial rail frame, so a straight rail reproduces the
/// profile at every station.
pub fn sweep1(
    profile: &Curve,
    rail: &Curve,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if samples[0] <= degree[0] {
        return Err(Error::Invalid("sweep profile samples"));
    }
    if samples[1] <= degree[1] {
        return Err(Error::Invalid("sweep rail samples"));
    }
    profile.validate(limits)?;
    let frames = path_frames(rail, samples[1], limits)?;
    // Sample the profile once (polycurve aware), then express it in the initial frame.
    let profile_points = sample_curve(profile, samples[0], limits)?;
    let initial = frames[0];
    let mut rows: Vec<Vec<Point3>> = Vec::new();
    rows.try_reserve_exact(frames.len())
        .map_err(|_| Error::Budget("sweep rows"))?;
    for frame in &frames {
        let mut row = Vec::new();
        row.try_reserve_exact(samples[0])
            .map_err(|_| Error::Budget("sweep row"))?;
        for point in &profile_points {
            // Local coordinates in the initial frame.
            let offset = [
                point[0] - initial.origin[0],
                point[1] - initial.origin[1],
                point[2] - initial.origin[2],
            ];
            let local = [
                dot(offset, initial.x),
                dot(offset, initial.y),
                dot(offset, initial.tangent),
            ];
            // Place that local offset in the current frame.
            row.push([
                frame.origin[0]
                    + frame.x[0] * local[0]
                    + frame.y[0] * local[1]
                    + frame.tangent[0] * local[2],
                frame.origin[1]
                    + frame.x[1] * local[0]
                    + frame.y[1] * local[1]
                    + frame.tangent[1] * local[2],
                frame.origin[2]
                    + frame.x[2] * local[0]
                    + frame.y[2] * local[1]
                    + frame.tangent[2] * local[2],
            ]);
        }
        rows.push(row);
    }
    grid_surface(&rows, degree, limits)
}

/// Build a tensor-product surface whose rows are already in the u direction.
fn grid_surface(rows: &[Vec<Point3>], degree: [usize; 2], limits: &Limits) -> Result<NurbsSurface> {
    let u_samples = rows.first().map(Vec::len).unwrap_or(0);
    if u_samples <= degree[0] || rows.len() <= degree[1] {
        return Err(Error::Invalid("surface grid"));
    }
    let mut u_poles: Vec<Vec<Point3>> = Vec::new();
    u_poles
        .try_reserve_exact(rows.len())
        .map_err(|_| Error::Budget("surface grid rows"))?;
    let mut knots_u = None;
    for row in rows {
        let curve = interpolate_any(row, degree[0], KnotMode::Uniform, limits)?;
        if knots_u.is_none() {
            knots_u = Some(curve.knots.clone());
        }
        u_poles.push(curve.poles);
    }
    let mut poles = vec![[0.0; 3]; u_samples * rows.len()];
    let mut knots_v = None;
    for i in 0..u_samples {
        let column: Vec<Point3> = u_poles.iter().map(|row| row[i]).collect();
        let curve = interpolate_any(&column, degree[1], KnotMode::Uniform, limits)?;
        if knots_v.is_none() {
            knots_v = Some(curve.knots.clone());
        }
        for (j, pole) in curve.poles.iter().enumerate() {
            poles[i * rows.len() + j] = *pole;
        }
    }
    let surface = NurbsSurface {
        degree,
        size: [u_samples, rows.len()],
        poles,
        weights: vec![1.0; u_samples * rows.len()],
        knots: [
            knots_u.ok_or(Error::Invalid("surface grid"))?,
            knots_v.ok_or(Error::Invalid("surface grid"))?,
        ],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    surface.validate(limits)?;
    Ok(surface)
}

/* ------------------------- sweep2 (two rails) ------------------------- */

/// Sweep 2 options. Defaults follow `docs/features/OM9-SURFACE-003.md`: 65 arc-length rail
/// stations, degree at most three, and Maintain Height **OFF** (the host's initial choice).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sweep2Options {
    /// `[profile samples, rail stations]`.
    pub samples: [usize; 2],
    pub degree: [usize; 2],
    pub maintain_height: bool,
}

impl Default for Sweep2Options {
    fn default() -> Self {
        Self {
            samples: [17, 65],
            degree: [3, 3],
            maintain_height: false,
        }
    }
}

/// Arc-length stations on a curve: `(point, unit tangent)` pairs spaced by equal distance.
/// The contract samples 65 equivalent arc-length stations per rail, so a fraction grid is
/// not enough; the table below inverts arc length by dense sampling and bisection.
pub fn arc_length_stations(
    curve: &Curve,
    count: usize,
    limits: &Limits,
) -> Result<Vec<(Point3, Point3)>> {
    if count < 2 {
        return Err(Error::Invalid("station count"));
    }
    let dense = (count * 8).max(64);
    let mut table: Vec<(f64, f64)> = Vec::new();
    table
        .try_reserve_exact(dense + 1)
        .map_err(|_| Error::Budget("arc length table"))?;
    let mut accumulated = 0.0;
    let mut previous: Option<Point3> = None;
    for index in 0..=dense {
        let fraction = index as f64 / dense as f64;
        let evaluated = crate::evaluation::evaluate_curve(
            curve,
            crate::evaluation::Parameter::Fraction(fraction),
            0,
            crate::geometry::Transform::identity(),
            limits,
        )?;
        let point = *evaluated
            .first()
            .ok_or(Error::Invalid("arc length sample"))?;
        if let Some(previous) = previous {
            accumulated += dot(
                [
                    point[0] - previous[0],
                    point[1] - previous[1],
                    point[2] - previous[2],
                ],
                [
                    point[0] - previous[0],
                    point[1] - previous[1],
                    point[2] - previous[2],
                ],
            )
            .sqrt();
        }
        previous = Some(point);
        table.push((fraction, accumulated));
    }
    let total = accumulated;
    if !total.is_finite() || total <= 0.0 {
        return Err(Error::Invalid("degenerate rail"));
    }
    let mut stations = Vec::new();
    stations
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("stations"))?;
    for index in 0..count {
        let target = total * index as f64 / (count - 1) as f64;
        // The table is monotonic, so a linear scan finds the bracketing pair; the fraction
        // is then interpolated linearly inside it (the table is dense).
        let mut low = 0.0;
        for pair in table.windows(2) {
            if target >= pair[0].1 && target <= pair[1].1 {
                let width = pair[1].1 - pair[0].1;
                let alpha = if width > 0.0 {
                    (target - pair[0].1) / width
                } else {
                    0.0
                };
                low = pair[0].0 + alpha * (pair[1].0 - pair[0].0);
                break;
            }
        }
        let evaluated = crate::evaluation::evaluate_curve(
            curve,
            crate::evaluation::Parameter::Fraction(low),
            1,
            crate::geometry::Transform::identity(),
            limits,
        )?;
        if evaluated.len() < 2 {
            return Err(Error::Invalid("station tangent"));
        }
        stations.push((evaluated[0], unit_vector(evaluated[1])?));
    }
    Ok(stations)
}

/// Sweep 2: transport the profile between two rails.
///
/// Per `docs/features/OM9-SURFACE-003.md` (Matrix 8 Book 1 pp.174–176): the profile must
/// span both rail starts, the frame uses the **rail separation** for `x` and the **primary
/// rail tangent** for the sweep direction, the width scales with the separation, the normal
/// (height) coordinate is preserved with Maintain Height ON and scaled with the width when
/// OFF, and the tangent coordinate never changes.
pub fn sweep2(
    profile: &Curve,
    rail_a: &Curve,
    rail_b: &Curve,
    options: Sweep2Options,
    limits: &Limits,
) -> Result<NurbsSurface> {
    if options.degree[0] == 0
        || options.degree[0] > 3
        || options.degree[1] == 0
        || options.degree[1] > 3
    {
        return Err(Error::Invalid("sweep degree"));
    }
    if options.samples[0] <= options.degree[0] {
        return Err(Error::Invalid("sweep profile samples"));
    }
    if options.samples[1] <= options.degree[1] {
        return Err(Error::Invalid("sweep rail samples"));
    }
    profile.validate(limits)?;
    let stations_a = arc_length_stations(rail_a, options.samples[1], limits)?;
    let stations_b = arc_length_stations(rail_b, options.samples[1], limits)?;
    // A closed rail would need a closed loft; that is not implemented, and a non-periodic
    // surface would silently break the seam, so it is refused by name.
    let first = stations_a[0].0;
    let last = stations_a[stations_a.len() - 1].0;
    if distance_between(first, last) <= 1e-9 {
        return Err(Error::Unsupported("sweep2 closed rails"));
    }
    // Initial frame: x = separation, normal = x cross primary tangent, tangent re-squared.
    let width0 = distance_between(stations_a[0].0, stations_b[0].0);
    if !width0.is_finite() || width0 <= 1e-9 {
        return Err(Error::Invalid("sweep2 rail separation"));
    }
    let x0 = unit_vector(sub_point(stations_b[0].0, stations_a[0].0))?;
    let normal0 = unit_vector(cross_product(x0, stations_a[0].1))?;
    let tangent0 = unit_vector(cross_product(normal0, x0))?;
    let profile_points = sample_curve(profile, options.samples[0], limits)?;
    let mut local = Vec::new();
    local
        .try_reserve_exact(profile_points.len())
        .map_err(|_| Error::Budget("sweep2 local profile"))?;
    let (mut min_x, mut max_x) = (f64::INFINITY, f64::NEG_INFINITY);
    for point in &profile_points {
        let offset = sub_point(*point, stations_a[0].0);
        let coordinates = [dot(offset, x0), dot(offset, normal0), dot(offset, tangent0)];
        min_x = min_x.min(coordinates[0]);
        max_x = max_x.max(coordinates[0]);
        local.push(coordinates);
    }
    // The profile must span both rail starts (Matrix rule); a profile that stops short
    // cannot be transported and is refused instead of extruded sideways.
    if min_x > 1e-6 || max_x < width0 - 1e-6 {
        return Err(Error::Invalid("sweep2 profile span"));
    }
    let mut rows: Vec<Vec<Point3>> = Vec::new();
    rows.try_reserve_exact(stations_a.len())
        .map_err(|_| Error::Budget("sweep2 rows"))?;
    for index in 0..stations_a.len() {
        let origin_a = stations_a[index].0;
        let origin_b = stations_b[index].0;
        let width = distance_between(origin_a, origin_b);
        if !width.is_finite() || width <= 1e-9 {
            return Err(Error::Invalid("sweep2 rail separation"));
        }
        let scale = width / width0;
        let x = unit_vector(sub_point(origin_b, origin_a))?;
        let normal = unit_vector(cross_product(x, stations_a[index].1))?;
        let tangent = unit_vector(cross_product(normal, x))?;
        let mut row = Vec::new();
        row.try_reserve_exact(local.len())
            .map_err(|_| Error::Budget("sweep2 row"))?;
        for coordinates in &local {
            let along_x = coordinates[0] * scale;
            let along_normal = if options.maintain_height {
                coordinates[1]
            } else {
                coordinates[1] * scale
            };
            let along_tangent = coordinates[2];
            row.push([
                origin_a[0]
                    + x[0] * along_x
                    + normal[0] * along_normal
                    + tangent[0] * along_tangent,
                origin_a[1]
                    + x[1] * along_x
                    + normal[1] * along_normal
                    + tangent[1] * along_tangent,
                origin_a[2]
                    + x[2] * along_x
                    + normal[2] * along_normal
                    + tangent[2] * along_tangent,
            ]);
        }
        rows.push(row);
    }
    grid_surface(&rows, options.degree, limits)
}

fn sub_point(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn distance_between(a: Point3, b: Point3) -> f64 {
    let d = sub_point(a, b);
    dot(d, d).sqrt()
}
