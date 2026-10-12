//! OM9-CURVE-005: normalized best-plane algebraic least-squares circle fit.
use super::{Plan, Point, add, check_point, cross, dot, norm, scale, sub, unit};
pub fn fit_circle(points: &[Point]) -> Result<(Plan, f64), String> {
    if !(3..=1024).contains(&points.len()) {
        return Err("FitPoints needs 3 through 1024 points".into());
    }
    for &p in points {
        check_point(p)?;
    }
    let mean = points
        .iter()
        .fold([0.; 3], |a, &p| add(a, scale(p, 1. / points.len() as f64)));
    let span = points
        .iter()
        .map(|&p| norm(sub(p, mean)))
        .fold(0., f64::max);
    if span < 1e-7 {
        return Err("Fit points are coincident".into());
    }
    let q: Vec<_> = points
        .iter()
        .map(|&p| scale(sub(p, mean), 1. / span))
        .collect();
    let mut cov = [[0.; 3]; 3];
    for p in &q {
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += p[i] * p[j] / q.len() as f64;
            }
        }
    }
    let mut vectors = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
    for _ in 0..40 {
        let (i, j) = [(0, 1), (0, 2), (1, 2)]
            .into_iter()
            .max_by(|&(i, j), &(k, l)| cov[i][j].abs().total_cmp(&cov[k][l].abs()))
            .unwrap();
        if cov[i][j].abs() < 1e-15 {
            break;
        }
        let angle = 0.5 * (2. * cov[i][j]).atan2(cov[j][j] - cov[i][i]);
        let c = angle.cos();
        let s = angle.sin();
        let a = cov[i][i];
        let b = cov[j][j];
        let d = cov[i][j];
        cov[i][i] = c * c * a - 2. * s * c * d + s * s * b;
        cov[j][j] = s * s * a + 2. * s * c * d + c * c * b;
        cov[i][j] = 0.;
        cov[j][i] = 0.;
        for k in 0..3 {
            if k != i && k != j {
                let a = cov[k][i];
                let b = cov[k][j];
                cov[k][i] = c * a - s * b;
                cov[i][k] = cov[k][i];
                cov[k][j] = s * a + c * b;
                cov[j][k] = cov[k][j];
            }
            let a = vectors[k][i];
            let b = vectors[k][j];
            vectors[k][i] = c * a - s * b;
            vectors[k][j] = s * a + c * b;
        }
    }
    let mut order = [0, 1, 2];
    order.sort_by(|&i, &j| cov[i][i].total_cmp(&cov[j][j]));
    if cov[order[1]][order[1]] <= 1e-10 * cov[order[2]][order[2]] {
        return Err("Fit points are collinear or ill-conditioned".into());
    }
    let mut normal = unit(std::array::from_fn(|k| vectors[k][order[0]]))?;
    let sign = normal
        .iter()
        .max_by(|a, b| a.abs().total_cmp(&b.abs()))
        .unwrap();
    if *sign < 0. {
        normal = scale(normal, -1.);
    }
    let reference = if normal[0].abs() < 0.8 {
        [1., 0., 0.]
    } else {
        [0., 1., 0.]
    };
    let x = unit(sub(reference, scale(normal, dot(reference, normal))))?;
    let y = cross(normal, x);
    let mut matrix = [[0.; 4]; 3];
    for p in &q {
        let u = dot(*p, x);
        let v = dot(*p, y);
        let row = [u, v, 1.];
        let rhs = -(u * u + v * v);
        for i in 0..3 {
            for j in 0..3 {
                matrix[i][j] += row[i] * row[j];
            }
            matrix[i][3] += row[i] * rhs;
        }
    }
    for i in 0..3 {
        let pivot = (i..3)
            .max_by(|&a, &b| matrix[a][i].abs().total_cmp(&matrix[b][i].abs()))
            .unwrap();
        matrix.swap(i, pivot);
        if matrix[i][i].abs() < 1e-12 {
            return Err("Circle fit is ill-conditioned".into());
        }
        let d = matrix[i][i];
        for value in &mut matrix[i][i..] {
            *value /= d;
        }
        let pivot_row = matrix[i];
        for (k, row) in matrix.iter_mut().enumerate() {
            if k != i {
                let d = row[i];
                for (value, pivot) in row[i..].iter_mut().zip(&pivot_row[i..]) {
                    *value -= d * pivot;
                }
            }
        }
    }
    let u = -matrix[0][3] / 2.;
    let v = -matrix[1][3] / 2.;
    let r2 = u * u + v * v - matrix[2][3];
    if r2 <= 0. {
        return Err("Circle fit has no positive radius".into());
    }
    let center = add(mean, scale(add(scale(x, u), scale(y, v)), span));
    let plan = Plan::new(center, normal, span * r2.sqrt())?;
    let deviation = points
        .iter()
        .map(|&p| {
            let d = sub(p, center);
            let h = dot(d, normal);
            let r = (dot(d, d) - h * h).max(0.).sqrt();
            (r - plan.radius).hypot(h)
        })
        .fold(0., f64::max);
    Ok((plan, deviation))
}
