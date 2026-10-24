// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curve construction kernel (OM9-L2-03 wave W2).
//!
//! Ported from the legacy FreeCAD bridge (`rust/src/{nurbs_basis,spline,circle,rectangle}.rs`)
//! into safe Rust: the portable construction rules stay, the C ABI and the host command
//! shells do not. Every function returns a validated `Curve`, so a construction cannot
//! produce a record the rest of the document would refuse.
use crate::{
    Error, Result,
    budget::Limits,
    geometry::{Curve, NurbsCurve, Point3, finite},
};

/// How the interior parameters are distributed for interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnotMode {
    /// Chord length: spacing follows the distance between the input points.
    ChordLength,
    /// Uniform: spacing follows the index.
    Uniform,
}

/// Normalized parameters in `[0, 1]` for `points`. Two identical consecutive points
/// cannot define a chord, so `ChordLength` falls back to an even distribution for that
/// step instead of dividing by zero.
pub fn parameters(points: &[Point3], mode: KnotMode, closed: bool) -> Result<Vec<f64>> {
    let count = points.len();
    if count < 2 {
        return Err(Error::Invalid("interpolation point count"));
    }
    for point in points {
        finite(*point)?;
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("interpolation parameters"))?;
    match mode {
        KnotMode::Uniform => {
            values.extend((0..count).map(|index| index as f64 / (count - 1) as f64));
        }
        KnotMode::ChordLength => {
            let mut accumulated = vec![0.0; count];
            let mut steps = vec![0.0; count];
            for index in 1..count {
                let step = distance(points[index - 1], points[index]);
                steps[index] = step;
                accumulated[index] = accumulated[index - 1] + step;
            }
            let total = accumulated[count - 1];
            let degenerate = total <= 0.0 || steps[1..].iter().any(|step| *step <= 0.0);
            // A repeated point cannot define a chord: the whole curve falls back to
            // uniform parameters instead of dividing by zero or emitting a
            // non-increasing sequence.
            for (index, running) in accumulated.iter().enumerate() {
                values.push(if degenerate {
                    index as f64 / (count - 1) as f64
                } else {
                    running / total
                });
            }
        }
    }
    if closed {
        // A closed interpolation repeats the first point at the end.
        if values[count - 1] != 1.0 {
            return Err(Error::Invalid("closed interpolation"));
        }
    }
    let mut previous = -1.0;
    for value in &values {
        if !value.is_finite() || *value <= previous {
            return Err(Error::Invalid("interpolation parameters"));
        }
        previous = *value;
    }
    Ok(values)
}

fn distance(a: Point3, b: Point3) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// One B-spline basis function `N_{index, degree}(t)` (Cox–de Boor).
pub fn basis(degree: usize, knots: &[f64], index: usize, t: f64) -> Result<f64> {
    if knots.len() < degree + 2 || index + degree + 1 >= knots.len() {
        return Err(Error::Invalid("basis index"));
    }
    if degree == 0 {
        // Half-open span, except the closed domain end: the final knot value belongs
        // to the last non-empty span so an interpolating endpoint is exact.
        let low = knots[index];
        let high = knots[index + 1];
        if t >= low && t < high {
            return Ok(1.0);
        }
        let last = knots[knots.len() - 1];
        return Ok(if t == high && high == last && low < high {
            1.0
        } else {
            0.0
        });
    }
    let mut value = 0.0;
    let left_span = knots[index + degree] - knots[index];
    if left_span > 0.0 {
        value += (t - knots[index]) / left_span * basis(degree - 1, knots, index, t)?;
    }
    let right_span = knots[index + degree + 1] - knots[index + 1];
    if right_span > 0.0 {
        value +=
            (knots[index + degree + 1] - t) / right_span * basis(degree - 1, knots, index + 1, t)?;
    }
    Ok(value)
}

/// Clamped knot vector for `count` poles and `degree`, with the parameters averaged
/// inside (the standard interpolation knot vector). Ends repeat `degree + 1` times.
fn interpolation_knots(count: usize, degree: usize, parameters: &[f64]) -> Vec<f64> {
    let mut knots: Vec<f64> = std::iter::repeat_n(0.0, degree + 1).collect();
    // Interior knots: average of the parameters, one per span between the clamps.
    let interior = count.saturating_sub(degree).saturating_sub(1);
    for j in 1..=interior {
        let sum: f64 = parameters[j..j + degree].iter().sum();
        knots.push(sum / degree as f64);
    }
    knots.resize(count + degree + 1, 1.0);
    knots
}

/// Global interpolation: a clamped NURBS of `degree` passing through every point.
pub fn interpolate(
    points: &[Point3],
    degree: usize,
    mode: KnotMode,
    limits: &Limits,
) -> Result<NurbsCurve> {
    // The legacy rule (rust/src/spline.rs) admitted odd degrees 1..=11; keeping it
    // avoids the even-degree oscillation the legacy InterpCrv command refused.
    if degree == 0 || degree > 11 || degree.is_multiple_of(2) || degree > limits.max_degree {
        return Err(Error::Invalid("interpolation degree"));
    }
    interpolate_any(points, degree, mode, limits)
}

/// Interpolation without the command's odd-degree preference, for kernel callers such as
/// surface refit that must reproduce an even source degree.
pub fn interpolate_any(
    points: &[Point3],
    degree: usize,
    mode: KnotMode,
    limits: &Limits,
) -> Result<NurbsCurve> {
    if degree == 0 || degree > 11 || degree > limits.max_degree {
        return Err(Error::Invalid("interpolation degree"));
    }
    let count = points.len();
    if count <= degree {
        return Err(Error::Invalid("interpolation point count"));
    }
    let parameters = parameters(points, mode, false)?;
    let knots = interpolation_knots(count, degree, &parameters);
    // Solve N * P = Q for the poles.
    let mut matrix: Vec<Vec<f64>> = Vec::new();
    matrix
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("interpolation matrix"))?;
    for row in 0..count {
        let mut values = Vec::new();
        values
            .try_reserve_exact(count + 3)
            .map_err(|_| Error::Budget("interpolation matrix"))?;
        for column in 0..count {
            values.push(basis(degree, &knots, column, parameters[row])?);
        }
        values.extend_from_slice(&points[row]);
        matrix.push(values);
    }
    for column in 0..count {
        let mut best = column;
        for row in column + 1..count {
            if matrix[row][column].abs() > matrix[best][column].abs() {
                best = row;
            }
        }
        if matrix[best][column].abs() < 1e-12 {
            return Err(Error::Invalid("singular interpolation matrix"));
        }
        matrix.swap(column, best);
        let pivot = matrix[column][column];
        for value in matrix[column].iter_mut() {
            *value /= pivot;
        }
        for row in 0..count {
            if row == column {
                continue;
            }
            let factor = matrix[row][column];
            if factor == 0.0 {
                continue;
            }
            let row_slice: Vec<f64> = matrix[column].clone();
            for (index, value) in matrix[row].iter_mut().enumerate() {
                *value -= factor * row_slice[index];
            }
        }
    }
    let mut poles = Vec::new();
    poles
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("interpolation poles"))?;
    for row in &matrix {
        poles.push([row[count], row[count + 1], row[count + 2]]);
    }
    let curve = NurbsCurve {
        degree,
        poles,
        weights: vec![1.0; count],
        knots,
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    };
    curve.validate(limits)?;
    Ok(curve)
}

/// The full circle through three non-collinear points, as an analytic `Curve::Circle`
/// spanning one turn. Collinear or repeated points are refused.
pub fn circle_through(a: Point3, b: Point3, c: Point3) -> Result<Curve> {
    for point in [a, b, c] {
        finite(point)?;
    }
    let center = circumcenter(a, b, c)?;
    let radius = distance(center, a);
    if !radius.is_finite() || radius <= 0.0 {
        return Err(Error::Invalid("degenerate circle"));
    }
    // x points at the first input point; the plane normal comes from the three inputs
    // and y completes a right-handed orthonormal frame, so the parameter origin follows
    // the construction.
    let normal = unit(cross(sub(b, a), sub(c, a)))?;
    let mut x = unit(sub(a, center))?;
    if distance(center, b) > radius * (1.0 + 1e-9) || distance(center, c) > radius * (1.0 + 1e-9) {
        return Err(Error::Invalid("degenerate circle"));
    }
    // Orthonormalize x against the plane normal for a numerically clean frame.
    let dot = dot_product(x, normal);
    x = unit(sub(x, mul(normal, dot)))?;
    let y = unit(cross(normal, x))?;
    Ok(Curve::Circle {
        center,
        x,
        y,
        radius,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    })
}

fn circumcenter(a: Point3, b: Point3, c: Point3) -> Result<Point3> {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let normal = cross(ab, ac);
    let area2 = dot_product(normal, normal);
    if !area2.is_finite() || area2 <= 1e-24 {
        return Err(Error::Invalid("degenerate circle"));
    }
    // |ab|^2 * (ac x n) + |ac|^2 * (n x ab)  / (2 |ab x ac|^2)
    let ab2 = dot_product(ab, ab);
    let ac2 = dot_product(ac, ac);
    let term = add(mul(cross(ac, normal), ab2), mul(cross(normal, ab), ac2));
    Ok(add(a, mul(term, 1.0 / (2.0 * area2))))
}

/// An open polycurve of line segments through the points (at least two).
pub fn polyline(points: &[Point3]) -> Result<Curve> {
    if points.len() < 2 {
        return Err(Error::Invalid("interpolation point count"));
    }
    for point in points {
        finite(*point)?;
    }
    let mut segments = Vec::new();
    segments
        .try_reserve_exact(points.len() - 1)
        .map_err(|_| Error::Budget("polyline segments"))?;
    for pair in points.windows(2) {
        if pair[0] == pair[1] {
            return Err(Error::Invalid("degenerate line"));
        }
        segments.push(Curve::Line {
            start: pair[0],
            end: pair[1],
            domain: [0.0, 1.0],
        });
    }
    Ok(Curve::Polycurve {
        segments,
        reversed: false,
    })
}

/// A closed rectangle on the frame `(origin, x, y)`: four line segments, first to last
/// point closing the loop. `x` and `y` must be orthonormal.
pub fn rectangle(origin: Point3, x: Point3, y: Point3, width: f64, height: f64) -> Result<Curve> {
    for point in [origin, x, y] {
        finite(point)?;
    }
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(Error::Invalid("rectangle size"));
    }
    if (dot_product(x, x) - 1.0).abs() > 1e-9
        || (dot_product(y, y) - 1.0).abs() > 1e-9
        || dot_product(x, y).abs() > 1e-9
    {
        return Err(Error::Invalid("rectangle frame"));
    }
    let p0 = origin;
    let p1 = add(origin, mul(x, width));
    let p2 = add(p1, mul(y, height));
    let p3 = add(origin, mul(y, height));
    polyline(&[p0, p1, p2, p3, p0])
}

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn mul(a: Point3, scale: f64) -> Point3 {
    [a[0] * scale, a[1] * scale, a[2] * scale]
}
fn dot_product(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn unit(a: Point3) -> Result<Point3> {
    let length = dot_product(a, a).sqrt();
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("degenerate direction"));
    }
    Ok(mul(a, 1.0 / length))
}

/* ------------------------- arc ------------------------- */

fn wrap_turn(angle: f64) -> f64 {
    let turn = std::f64::consts::TAU;
    let mut value = angle % turn;
    if value < 0.0 {
        value += turn;
    }
    value
}

/// The angle of a point around the circle frame, measured from `x` towards `y`.
fn frame_angle(center: Point3, x: Point3, y: Point3, point: Point3) -> f64 {
    let offset = sub(point, center);
    dot_product(offset, y).atan2(dot_product(offset, x))
}

/// The arc from `a` through `b` to `c`, as an analytic circle whose parameter domain
/// is `[0,1]` and whose `angle_domain` maps that parameter onto the arc's absolute
/// angles (OM9-L3-01-08 semantics: the parameter map must be increasing, so the frame
/// is mirrored when the natural direction is clockwise).
pub fn arc_through(a: Point3, b: Point3, c: Point3) -> Result<Curve> {
    let Curve::Circle {
        center,
        mut x,
        mut y,
        radius,
        ..
    } = circle_through(a, b, c)?
    else {
        return Err(Error::Invalid("degenerate circle"));
    };
    let start = frame_angle(center, x, y, a);
    let through = frame_angle(center, x, y, b);
    let end = frame_angle(center, x, y, c);
    // Counter-clockwise sweep from `a`; if `b` is not on it, the arc runs clockwise and
    // the frame is mirrored so the mapped angle keeps increasing.
    let mut sweep = wrap_turn(end - start);
    if wrap_turn(through - start) > sweep {
        y = mul(y, -1.0);
        sweep = wrap_turn(-(end - start));
        if wrap_turn(-(through - start)) > sweep {
            return Err(Error::Invalid("degenerate arc"));
        }
        x = unit(x)?;
    }
    if !sweep.is_finite() || sweep <= 1e-12 {
        return Err(Error::Invalid("degenerate arc"));
    }
    Ok(Curve::Circle {
        center,
        x,
        y,
        radius,
        domain: [0.0, 1.0],
        angle_domain: Some([
            frame_angle(center, x, y, a),
            frame_angle(center, x, y, a) + sweep,
        ]),
    })
}

/* ------------------------- ellipse ------------------------- */

/// The exact rational quadratic ellipse on the frame `(origin, x, y)`. The two radii are
/// the semi-axes along `x` and `y`; the record is a NURBS because the document model has
/// no analytic ellipse kind.
pub fn ellipse(
    origin: Point3,
    x: Point3,
    y: Point3,
    radius_x: f64,
    radius_y: f64,
    limits: &Limits,
) -> Result<NurbsCurve> {
    for point in [origin, x, y] {
        finite(point)?;
    }
    if !radius_x.is_finite() || !radius_y.is_finite() || radius_x <= 0.0 || radius_y <= 0.0 {
        return Err(Error::Invalid("ellipse radius"));
    }
    if (dot_product(x, x) - 1.0).abs() > 1e-9
        || (dot_product(y, y) - 1.0).abs() > 1e-9
        || dot_product(x, y).abs() > 1e-9
    {
        return Err(Error::Invalid("ellipse frame"));
    }
    let half = std::f64::consts::FRAC_1_SQRT_2;
    // Unit-circle poles (x, y) scaled by the semi-axes, then placed in the frame.
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
    let mut poles = Vec::new();
    poles
        .try_reserve_exact(unit.len())
        .map_err(|_| Error::Budget("ellipse poles"))?;
    for [px, py] in unit {
        let local_x = px * radius_x;
        let local_y = py * radius_y;
        poles.push(add(origin, add(mul(x, local_x), mul(y, local_y))));
    }
    let weights = vec![1.0, half, 1.0, half, 1.0, half, 1.0, half, 1.0];
    let knots = vec![
        0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0,
    ];
    let curve = NurbsCurve {
        degree: 2,
        poles,
        weights,
        knots,
        domain: [0.0, 1.0],
        // A closed, clamped rational NURBS: the first and last pole coincide, but the
        // record is not marked `periodic` because the evaluator does not certify
        // periodic derivative continuity. Geometry is closed all the same.
        periodic: false,
        reversed: false,
        seam: None,
    };
    curve.validate(limits)?;
    Ok(curve)
}

/* ------------------------- rebuild ------------------------- */

/// Rebuild a curve by sampling it and interpolating through the samples (the legacy
/// `spline::rebuild` idea: a rebuilt curve is a new, denser record, not an edit).
pub fn rebuild(
    source: &Curve,
    degree: usize,
    points: usize,
    mode: KnotMode,
    limits: &Limits,
) -> Result<NurbsCurve> {
    if points < 4 || points <= degree {
        return Err(Error::Invalid("rebuild point count"));
    }
    let mut samples = Vec::new();
    samples
        .try_reserve_exact(points)
        .map_err(|_| Error::Budget("rebuild samples"))?;
    for index in 0..points {
        let fraction = index as f64 / (points - 1) as f64;
        let evaluated = crate::evaluation::evaluate_curve(
            source,
            crate::evaluation::Parameter::Fraction(fraction),
            0,
            crate::geometry::Transform::identity(),
            limits,
        )?;
        samples.push(*evaluated.first().ok_or(Error::Invalid("rebuild sample"))?);
    }
    interpolate(&samples, degree, mode, limits)
}

/* ------------------------- conversion (knot / degree / fit) ------------------------- */

/// Homogeneous pole `[x*w, y*w, z*w, w]`: every conversion below runs in this space, so a
/// rational curve keeps its weights through the operation.
fn to_homogeneous(curve: &NurbsCurve) -> Vec<[f64; 4]> {
    curve
        .poles
        .iter()
        .zip(&curve.weights)
        .map(|(pole, weight)| {
            [
                pole[0] * weight,
                pole[1] * weight,
                pole[2] * weight,
                *weight,
            ]
        })
        .collect()
}

fn from_homogeneous(
    poles: Vec<[f64; 4]>,
    degree: usize,
    knots: Vec<f64>,
    domain: [f64; 2],
    limits: &Limits,
) -> Result<NurbsCurve> {
    let mut cartesian = Vec::new();
    let mut weights = Vec::new();
    cartesian
        .try_reserve_exact(poles.len())
        .map_err(|_| Error::Budget("converted poles"))?;
    weights
        .try_reserve_exact(poles.len())
        .map_err(|_| Error::Budget("converted weights"))?;
    for pole in poles {
        let weight = pole[3];
        if !weight.is_finite() || weight <= 0.0 {
            return Err(Error::Invalid("converted homogeneous weight"));
        }
        cartesian.push([pole[0] / weight, pole[1] / weight, pole[2] / weight]);
        weights.push(weight);
    }
    let curve = NurbsCurve {
        degree,
        poles: cartesian,
        weights,
        knots,
        domain,
        periodic: false,
        reversed: false,
        seam: None,
    };
    curve.validate(limits)?;
    Ok(curve)
}

fn knot_span(knots: &[f64], degree: usize, t: f64) -> Result<usize> {
    let count = knots.len() - degree - 1;
    let mut span = None;
    for index in degree..count {
        if t >= knots[index] && t < knots[index + 1] {
            span = Some(index);
            break;
        }
    }
    span.ok_or(Error::Invalid("knot span"))
}

/// Insert the knot `t` `times` times (Boehm). The geometry is unchanged by construction;
/// the test compares evaluation before and after.
pub fn insert_knot(
    source: &NurbsCurve,
    t: f64,
    times: usize,
    limits: &Limits,
) -> Result<NurbsCurve> {
    if times == 0 {
        return Ok(source.clone());
    }
    if !t.is_finite() || t <= source.domain[0] || t >= source.domain[1] {
        return Err(Error::Invalid("knot insertion parameter"));
    }
    let degree = source.degree;
    let mut poles = to_homogeneous(source);
    let mut knots = source.knots.clone();
    let mut inserted = 0;
    while inserted < times {
        let span = knot_span(&knots, degree, t)?;
        // Boehm works on the *new* pole sequence: one extra pole, so index runs to
        // old_len(). Poles after the span take the previous old pole (`P[i-1]`); the
        // blended range is `(span-degree, span]`.
        let mut next = Vec::new();
        next.try_reserve_exact(poles.len() + 1)
            .map_err(|_| Error::Budget("inserted poles"))?;
        for index in 0..=poles.len() {
            if index <= span - degree {
                next.push(poles[index]);
            } else if index <= span {
                let left = knots[index];
                let right = knots[index + degree];
                let width = right - left;
                if !width.is_finite() || width <= 0.0 {
                    return Err(Error::Invalid("knot insertion interval"));
                }
                let alpha = (t - left) / width;
                let current = poles[index];
                let previous = poles[index - 1];
                next.push(std::array::from_fn(|axis| {
                    alpha * current[axis] + (1.0 - alpha) * previous[axis]
                }));
            } else {
                let last = *poles.last().ok_or(Error::Invalid("knot insertion poles"))?;
                next.push(if index < poles.len() {
                    poles[index - 1]
                } else {
                    last
                });
            }
        }
        // Rebuild knots with one more `t` after the span.
        let mut new_knots = Vec::new();
        new_knots
            .try_reserve_exact(knots.len() + 1)
            .map_err(|_| Error::Budget("inserted knots"))?;
        new_knots.extend_from_slice(&knots);
        new_knots.insert(span + 1, t);
        poles = next;
        knots = new_knots;
        inserted += 1;
    }
    from_homogeneous(poles, degree, knots, source.domain, limits)
}

/// Raise the degree by one without changing the geometry: split into Bézier segments,
/// elevate each segment, then re-join at C0 breakpoints.
pub fn elevate_degree(source: &NurbsCurve, limits: &Limits) -> Result<NurbsCurve> {
    let degree = source.degree;
    // 1. Insert interior knots until every interior value has multiplicity `degree`.
    let mut curve = source.clone();
    loop {
        let mut pending: Option<f64> = None;
        for index in degree + 1..curve.knots.len() - degree - 1 {
            let value = curve.knots[index];
            let multiplicity = curve.knots.iter().filter(|knot| **knot == value).count();
            if multiplicity < degree {
                pending = Some(value);
                break;
            }
        }
        let Some(value) = pending else {
            break;
        };
        curve = insert_knot(&curve, value, 1, limits)?;
    }
    // 2. One Bézier segment per span; elevate each in homogeneous space.
    let segments = curve.poles.len().saturating_sub(1) / degree;
    if segments == 0 {
        return Err(Error::Invalid("degree elevation segments"));
    }
    let homogeneous = to_homogeneous(&curve);
    let mut raised: Vec<[f64; 4]> = Vec::new();
    // Distinct interior knot values, in order: one breakpoint per segment join.
    let mut breakpoints: Vec<f64> = Vec::new();
    for value in curve.knots.iter().copied() {
        if value > curve.domain[0] && value < curve.domain[1] && breakpoints.last() != Some(&value)
        {
            breakpoints.push(value);
        }
    }
    for segment in 0..segments {
        let start = segment * degree;
        let control = &homogeneous[start..=start + degree];
        let mut elevated = Vec::with_capacity(degree + 2);
        elevated.push(control[0]);
        for index in 1..=degree {
            let weight = index as f64 / (degree + 1) as f64;
            elevated.push(std::array::from_fn(|axis| {
                weight * control[index - 1][axis] + (1.0 - weight) * control[index][axis]
            }));
        }
        elevated.push(control[degree]);
        if segment == 0 {
            raised.extend_from_slice(&elevated);
        } else {
            // The junction pole is shared: skip this segment's first pole.
            raised.extend_from_slice(&elevated[1..]);
        }
        let _ = segment;
    }
    // 3. Knots: multiplicity degree+2 at the ends, degree+1 at each interior breakpoint.
    // The distinct interior knot values are exactly the segment breakpoints, so read
    // them from the split knot vector instead of recomputing indices.
    let (start_value, end_value) = (curve.domain[0], curve.domain[1]);
    breakpoints.dedup();
    let mut knots: Vec<f64> = std::iter::repeat_n(start_value, degree + 2).collect();
    for value in breakpoints {
        knots.extend(std::iter::repeat_n(value, degree + 1));
    }
    knots.resize(raised.len() + (degree + 1) + 1, end_value);
    from_homogeneous(raised, degree + 1, knots, curve.domain, limits)
}

/// Least-squares fit through `points` with the first and last point interpolated
/// (Piegl–Tiller A9.4). With `control_points == points.len()` this is an interpolation.
pub fn fit_points(
    points: &[Point3],
    degree: usize,
    control_points: usize,
    mode: KnotMode,
    limits: &Limits,
) -> Result<NurbsCurve> {
    if degree == 0 || degree > limits.max_degree {
        return Err(Error::Invalid("fit degree"));
    }
    if control_points <= degree || control_points > points.len() {
        return Err(Error::Invalid("fit control point count"));
    }
    let parameters = parameters(points, mode, false)?;
    let count = control_points;
    // Evenly spaced interior knots. The A9.3 averaging formula is meant for
    // interpolation: with far fewer control points than samples it crowds every
    // interior knot near the start and leaves one huge end span (measured: 0.085 mm
    // error on a half circle), so the fit distributes its spans evenly instead.
    let spans = count - degree;
    let mut knots: Vec<f64> = std::iter::repeat_n(0.0, degree + 1).collect();
    for span in 1..spans {
        knots.push(span as f64 / spans as f64);
    }
    knots.resize(count + degree + 1, 1.0);
    let last = points.len() - 1;
    // Unknown poles are 1..=count-2 (the ends are fixed).
    let unknowns = count.saturating_sub(2);
    let mut matrix = vec![vec![0.0; unknowns + 3]; unknowns];
    // Build the normal equations of the reduced least-squares system.
    for k in 1..last {
        let u = parameters[k];
        let mut basis_values = Vec::with_capacity(count);
        for index in 0..count {
            basis_values.push(basis(degree, &knots, index, u)?);
        }
        let n0 = basis_values[0];
        let nn = basis_values[count - 1];
        let mut rhs = [0.0; 3];
        for axis in 0..3 {
            rhs[axis] = points[k][axis] - n0 * points[0][axis] - nn * points[last][axis];
        }
        for (row, line) in matrix.iter_mut().enumerate() {
            let a_row = basis_values[row + 1];
            for (column, value) in line[..unknowns].iter_mut().enumerate() {
                *value += a_row * basis_values[column + 1];
            }
            for (axis, value) in line[unknowns..].iter_mut().enumerate() {
                *value += a_row * rhs[axis];
            }
        }
    }
    // Solve with partial pivoting.
    for column in 0..unknowns {
        let mut best = column;
        for row in column + 1..unknowns {
            if matrix[row][column].abs() > matrix[best][column].abs() {
                best = row;
            }
        }
        if matrix[best][column].abs() < 1e-12 {
            return Err(Error::Invalid("singular fit matrix"));
        }
        matrix.swap(column, best);
        let pivot = matrix[column][column];
        for value in matrix[column].iter_mut() {
            *value /= pivot;
        }
        let pivot_row = matrix[column].clone();
        for (row, line) in matrix.iter_mut().enumerate() {
            if row == column {
                continue;
            }
            let factor = line[column];
            if factor == 0.0 {
                continue;
            }
            for (value, pivot_value) in line.iter_mut().zip(&pivot_row) {
                *value -= factor * pivot_value;
            }
        }
    }
    let mut poles = Vec::new();
    poles
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("fit poles"))?;
    poles.push(points[0]);
    for line in &matrix {
        poles.push([line[unknowns], line[unknowns + 1], line[unknowns + 2]]);
    }
    poles.push(points[last]);
    let curve = NurbsCurve {
        degree,
        poles,
        weights: vec![1.0; count],
        knots,
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    };
    curve.validate(limits)?;
    Ok(curve)
}

/* --------------------------- analytic creation --------------------------- */

fn check_frame(center: Point3, x: Point3, y: Point3) -> Result<()> {
    for point in [center, x, y] {
        finite(point)?;
    }
    if (dot_product(x, x) - 1.0).abs() > 1e-9
        || (dot_product(y, y) - 1.0).abs() > 1e-9
        || dot_product(x, y).abs() > 1e-9
    {
        return Err(Error::Invalid("circle frame"));
    }
    Ok(())
}

/// A full circle on an arbitrary orthonormal frame. The radius decides admission; the
/// native parameter is the angle in radians over `[0, tau]`.
pub fn circle_on_frame(center: Point3, x: Point3, y: Point3, radius: f64) -> Result<Curve> {
    check_frame(center, x, y)?;
    if !radius.is_finite() || radius <= 0.0 {
        return Err(Error::Invalid("radius"));
    }
    Ok(Curve::Circle {
        center,
        x,
        y,
        radius,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    })
}

/// An arc on an orthonormal frame from `start_angle` through a positive `sweep`
/// (`0 < sweep <= tau`). The angle domain is increasing, so the evaluator's positive
/// angular-rate rule holds for either turn direction (choose the frame accordingly).
pub fn arc_by_center(
    center: Point3,
    x: Point3,
    y: Point3,
    radius: f64,
    start_angle: f64,
    sweep: f64,
) -> Result<Curve> {
    check_frame(center, x, y)?;
    if !radius.is_finite() || radius <= 0.0 {
        return Err(Error::Invalid("radius"));
    }
    if !start_angle.is_finite()
        || !sweep.is_finite()
        || sweep <= 0.0
        || sweep > std::f64::consts::TAU + 1e-12
    {
        return Err(Error::Invalid("arc angle domain"));
    }
    Ok(Curve::Circle {
        center,
        x,
        y,
        radius,
        domain: [0.0, 1.0],
        angle_domain: Some([start_angle, start_angle + sweep]),
    })
}

/// A closed polygon (at least three distinct points). The first point is appended only
/// when the caller did not repeat it; consecutive duplicates are refused.
pub fn polygon(points: &[Point3]) -> Result<Curve> {
    if points.len() < 3 {
        return Err(Error::Invalid("polygon point count"));
    }
    for point in points {
        finite(*point)?;
    }
    let mut closed = Vec::new();
    closed
        .try_reserve_exact(points.len() + 1)
        .map_err(|_| Error::Budget("polygon points"))?;
    closed.extend_from_slice(points);
    let distinct = {
        let mut seen: Vec<Point3> = Vec::new();
        for point in points {
            if !seen.contains(point) {
                seen.push(*point);
            }
        }
        seen.len()
    };
    if distinct < 3 {
        return Err(Error::Invalid("polygon distinct points"));
    }
    if closed.first() != closed.last() {
        closed.push(closed[0]);
    }
    // A zero-area polygon is a degenerate point set, not a closed profile.
    let mut normal = [0.0; 3];
    let mut scale: f64 = 0.0;
    for pair in closed.windows(2) {
        let term = cross(pair[0], pair[1]);
        normal = add(normal, term);
        scale = scale.max(dot_product(pair[0], pair[0]).sqrt());
    }
    let area = dot_product(normal, normal).sqrt();
    if !area.is_finite() || area <= 1e-12 * scale * scale {
        return Err(Error::Invalid("degenerate polygon"));
    }
    polyline(&closed)
}
