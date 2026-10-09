//! Pure construction math; all coordinates are world-space mm.
use super::{MIN, Point, add, cross, dimension, dot, scale, sub};
pub(super) fn unit(v: Point) -> Result<Point, String> {
    let norm = dot(v, v).sqrt();
    if !norm.is_finite() || norm <= 1e-300 {
        return Err("Degenerate direction".into());
    }
    Ok(scale(v, 1. / norm))
}
pub(super) fn circle(a: Point, b: Point, c: Point) -> Result<(Point, f64, Point), String> {
    let u = sub(b, a);
    let v = sub(c, a);
    let n = cross(u, v);
    let uu = dot(u, u);
    let vv = dot(v, v);
    let nn = dot(n, n);
    if uu < MIN * MIN || vv < MIN * MIN || nn <= 1e-20 * uu * vv {
        return Err("Three non-collinear points are required".into());
    }
    let offset = scale(
        add(scale(cross(v, n), uu), scale(cross(n, u), vv)),
        0.5 / nn,
    );
    Ok((add(a, offset), dot(offset, offset).sqrt(), unit(n)?))
}
pub(super) fn radius_circle(
    a: Point,
    b: Point,
    r: f64,
    direction: Point,
) -> Result<(Point, f64, Point), String> {
    let edge = sub(b, a);
    let len = dimension(dot(edge, edge).sqrt())?;
    let x = scale(edge, 1. / len);
    if r <= 0. || r < len * 0.5 {
        return Err("Radius must be positive and at least half the chord".into());
    }
    let mid = scale(add(a, b), 0.5);
    let d = sub(direction, mid);
    let perpendicular = sub(d, scale(x, dot(d, x)));
    if dot(perpendicular, perpendicular) <= 1e-20 * dot(d, d).max(1.) {
        return Err("Pick a direction off the chord".into());
    }
    let y = unit(perpendicular)?;
    let h = ((r - len * 0.5) * (r + len * 0.5)).sqrt();
    Ok((add(mid, scale(y, h)), r, cross(x, y)))
}
pub(super) fn fourth(circle: (Point, f64, Point), p: Point) -> Result<(Point, f64), String> {
    let (c, r, n) = circle;
    let d = sub(p, c);
    let h = dot(d, n);
    if h.abs() <= 1e-10 * dot(d, d).sqrt().max(r) {
        return Err("Fourth point must be outside the circle plane".into());
    }
    let t = (dot(d, d) - r * r) / (2. * h);
    Ok((add(c, scale(n, t)), r.hypot(t)))
}
pub(super) fn cube_axes(axes: [Point; 3], delta: Point) -> Result<([Point; 3], f64), String> {
    let length = dimension(dot(delta, delta).sqrt())?;
    let target = scale(delta, 1. / length);
    let source = scale(add(add(axes[0], axes[1]), axes[2]), 1. / 3_f64.sqrt());
    let cosine = dot(source, target).clamp(-1., 1.);
    let v = cross(source, target);
    let sine = dot(v, v).sqrt();
    let axis = if sine > 1e-15 {
        scale(v, 1. / sine)
    } else {
        unit(cross(source, axes[0]))?
    };
    let rotate = |p: Point| {
        add(
            add(scale(p, cosine), scale(cross(axis, p), sine)),
            scale(axis, dot(axis, p) * (1. - cosine)),
        )
    };
    Ok((axes.map(rotate), length / 3_f64.sqrt()))
}
fn solve<const N: usize>(mut a: [[f64; N]; N], mut b: [f64; N]) -> Result<[f64; N], String> {
    let norm = a.iter().flatten().fold(0_f64, |m, &v| m.max(v.abs()));
    for i in 0..N {
        let row = (i..N)
            .max_by(|&x, &y| a[x][i].abs().total_cmp(&a[y][i].abs()))
            .unwrap();
        if a[row][i].abs() <= 1e-12 * norm {
            return Err("Point fit is degenerate or ill-conditioned".into());
        }
        a.swap(i, row);
        b.swap(i, row);
        let pivot = a[i][i];
        for value in &mut a[i][i..] {
            *value /= pivot;
        }
        b[i] /= pivot;
        let pivot_row = a[i];
        for row in 0..N {
            if row != i {
                let k = a[row][i];
                for (value, pivot_value) in a[row][i..].iter_mut().zip(&pivot_row[i..]) {
                    *value -= k * pivot_value;
                }
                b[row] -= k * b[i];
            }
        }
    }
    Ok(b)
}
fn fit<const N: usize>(points: &[[f64; N]], planar: bool) -> Result<Vec<f64>, String> {
    // Algebraic seed, then damped Gauss-Newton on radial (geometric) residuals.
    let mut a = [[0.; 4]; 4];
    let mut b = [0.; 4];
    let dim = if planar { 2 } else { 3 };
    for p in points {
        let mut row = [0.; 4];
        let mut rhs = 0.;
        for i in 0..dim {
            row[i] = 2. * p[i];
            rhs += p[i] * p[i];
        }
        row[dim] = 1.;
        for i in 0..=dim {
            b[i] += row[i] * rhs;
            for j in 0..=dim {
                a[i][j] += row[i] * row[j];
            }
        }
    }
    let seed = if planar {
        let v = solve(
            std::array::from_fn::<_, 3, _>(|i| std::array::from_fn(|j| a[i][j])),
            [b[0], b[1], b[2]],
        )?;
        [v[0], v[1], 0., v[2]]
    } else {
        solve(a, b)?
    };
    let mut center = seed[..dim].to_vec();
    let mut radius =
        (center.iter().map(|v| v * v).sum::<f64>() + seed[if planar { 3 } else { dim }]).sqrt();
    if !radius.is_finite() || radius <= 0. {
        return Err("Point fit has no positive radius".into());
    }
    let cost = |c: &[f64], r: f64| {
        points
            .iter()
            .map(|p| ((0..dim).map(|i| (c[i] - p[i]).powi(2)).sum::<f64>().sqrt() - r).powi(2))
            .sum::<f64>()
    };
    for _ in 0..40 {
        let mut a = [[0.; 4]; 4];
        let mut b = [0.; 4];
        for p in points {
            let dist = (0..dim)
                .map(|i| (center[i] - p[i]).powi(2))
                .sum::<f64>()
                .sqrt();
            if dist <= 1e-12 {
                return Err("Fit point coincides with fitted center".into());
            }
            let mut row = [0.; 4];
            for i in 0..dim {
                row[i] = (center[i] - p[i]) / dist;
            }
            row[dim] = -1.;
            let residual = dist - radius;
            for i in 0..=dim {
                b[i] -= row[i] * residual;
                for j in 0..=dim {
                    a[i][j] += row[i] * row[j];
                }
            }
        }
        let step = if planar {
            let v = solve(
                std::array::from_fn::<_, 3, _>(|i| std::array::from_fn(|j| a[i][j])),
                [b[0], b[1], b[2]],
            )?;
            [v[0], v[1], v[2], 0.]
        } else {
            solve(a, b)?
        };
        if step[..=dim].iter().map(|v| v.abs()).fold(0., f64::max) < 1e-12 {
            break;
        }
        let old = cost(&center, radius);
        let mut alpha = 1.;
        let mut accepted = false;
        for _ in 0..16 {
            let next: Vec<_> = (0..dim).map(|i| center[i] + alpha * step[i]).collect();
            let r = radius + alpha * step[dim];
            if r > 0. && cost(&next, r) <= old {
                center = next;
                radius = r;
                accepted = true;
                break;
            }
            alpha *= 0.5;
        }
        if !accepted {
            break;
        }
    }
    center.push(radius);
    Ok(center)
}
pub(super) fn fit_points(points: &[Point]) -> Result<(Point, f64), String> {
    if points.len() < 3 {
        return Err("FitPoints requires at least three points".into());
    }
    let mut mean = [0.; 3];
    for &p in points {
        mean = add(mean, scale(p, 1. / points.len() as f64));
    }
    let span = points
        .iter()
        .map(|&p| dot(sub(p, mean), sub(p, mean)).sqrt())
        .fold(0., f64::max);
    dimension(span)?;
    let normalized: Vec<_> = points
        .iter()
        .map(|&p| scale(sub(p, mean), 1. / span))
        .collect();
    let first = normalized[0];
    let edge = normalized
        .iter()
        .map(|&p| sub(p, first))
        .max_by(|a, b| dot(*a, *a).total_cmp(&dot(*b, *b)))
        .unwrap();
    let normal = normalized
        .iter()
        .map(|&p| cross(edge, sub(p, first)))
        .max_by(|a, b| dot(*a, *a).total_cmp(&dot(*b, *b)))
        .unwrap();
    if dot(normal, normal) <= 1e-20 {
        return Err("FitPoints requires non-collinear points".into());
    }
    let n = unit(normal)?;
    if normalized
        .iter()
        .all(|&p| dot(sub(p, first), n).abs() < 1e-9)
    {
        let x = unit(edge)?;
        let y = cross(n, x);
        let coords: Vec<_> = normalized.iter().map(|&p| [dot(p, x), dot(p, y)]).collect();
        let f = fit(&coords, true)?;
        // The fitted circle lies in the input plane, not necessarily through the mean's origin.
        let center = add(add(scale(x, f[0]), scale(y, f[1])), scale(n, dot(first, n)));
        Ok((add(mean, scale(center, span)), f[2] * span))
    } else {
        let f = fit(&normalized, false)?;
        Ok((add(mean, scale([f[0], f[1], f[2]], span)), f[3] * span))
    }
}
