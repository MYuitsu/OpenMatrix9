//! Bounded spatial contact solving; exact curve evaluation is supplied by the host.
use crate::circle::{Plan, Point, check_point, fit_circle};
#[derive(Clone, Copy)]
pub enum Constraint {
    Curve { index: usize, seed: f64 },
    Point(Point),
}
pub struct Options {
    pub radius: Option<f64>,
    pub from_first: bool,
    pub vertical: bool,
    pub preferred: Point,
    pub solution: Option<usize>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            radius: None,
            from_first: false,
            vertical: false,
            preferred: [0., 0., 1.],
            solution: None,
        }
    }
}
pub struct Solution {
    pub plan: Plan,
    pub fractions: Vec<f64>,
}
fn add(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] - b[i])
}
fn mul(a: Point, s: f64) -> Point {
    a.map(|v| v * s)
}
fn dot(a: Point, b: Point) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn norm(a: Point) -> f64 {
    dot(a, a).sqrt()
}
fn cross(a: Point, b: Point) -> Point {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn unit(a: Point) -> Result<Point, String> {
    let n = norm(a);
    if !n.is_finite() || n < 1e-14 {
        Err("Degenerate tangent direction".into())
    } else {
        Ok(mul(a, 1. / n))
    }
}
struct Problem<'a, F> {
    cs: &'a [Constraint],
    options: &'a Options,
    eval: F,
    origin: Point,
    scale: f64,
    slots: Vec<Option<usize>>,
    calls: usize,
}
impl<F: FnMut(usize, f64) -> Result<(Point, Point), String>> Problem<'_, F> {
    fn contacts(&mut self, x: &[f64]) -> Result<Vec<(Point, Option<Point>)>, String> {
        self.cs
            .iter()
            .enumerate()
            .map(|(i, c)| match c {
                Constraint::Point(p) => Ok((*p, None)),
                Constraint::Curve { index, seed } => {
                    self.calls += 1;
                    if self.calls > 24000 {
                        return Err("Spatial Tangent evaluation budget exceeded".into());
                    }
                    let u = self.slots[i].map_or(*seed, |j| x[j]);
                    let (p, t) = (self.eval)(*index, u)?;
                    check_point(p)?;
                    Ok((p, Some(unit(t)?)))
                }
            })
            .collect()
    }
    fn residual(&mut self, x: &[f64]) -> Result<Vec<f64>, String> {
        let c = [x[0], x[1], x[2]];
        let n = [x[3], x[4], x[5]];
        let r = x[6];
        let contacts = self.contacts(x)?;
        let mut out = vec![dot(n, n) - 1.];
        for (p, t) in contacts {
            let d = sub(mul(sub(p, self.origin), 1. / self.scale), c);
            out.extend([dot(d, n), norm(d) - r]);
            if let Some(t) = t {
                out.extend([dot(t, n), dot(d, t)]);
            }
        }
        if let Some(radius) = self.options.radius {
            out.push(r - radius / self.scale);
        }
        if self.options.vertical {
            out.push(dot(n, self.options.preferred));
        }
        if out.iter().any(|v| !v.is_finite()) {
            return Err("Nonfinite Tangent residual".into());
        }
        Ok(out)
    }
    fn candidate(
        &mut self,
        x: &[f64],
        original: &[Point],
    ) -> Result<Option<(Solution, f64)>, String> {
        let mut n = unit([x[3], x[4], x[5]])?;
        let orientation = if dot(n, self.options.preferred).abs() > 1e-10 {
            dot(n, self.options.preferred)
        } else {
            n.into_iter()
                .max_by(|a, b| a.abs().total_cmp(&b.abs()))
                .unwrap()
        };
        if orientation < 0. {
            n = mul(n, -1.);
        }
        let Ok(plan) = Plan::new(
            add(self.origin, mul([x[0], x[1], x[2]], self.scale)),
            n,
            x[6] * self.scale,
        ) else {
            return Ok(None);
        };
        if self
            .options
            .radius
            .is_some_and(|r| (r - plan.radius).abs() > 1e-6 * r.max(1.))
            || self.options.vertical && dot(n, self.options.preferred).abs() > 1e-8
        {
            return Ok(None);
        }
        let mut score = 0.;
        for ((p, t), original) in self.contacts(x)?.into_iter().zip(original) {
            let d = sub(p, plan.center);
            if dot(d, n).abs() > 1e-6 || (norm(d) - plan.radius).abs() > 1e-6 * plan.radius.max(1.)
            {
                return Ok(None);
            }
            if let Some(t) = t {
                if dot(t, n).abs() > 1e-8 || dot(mul(d, 1. / plan.radius), t).abs() > 1e-8 {
                    return Ok(None);
                }
            }
            score += dot(sub(p, *original), sub(p, *original));
        }
        let fractions = self
            .cs
            .iter()
            .enumerate()
            .map(|(i, c)| match c {
                Constraint::Point(_) => 0.,
                Constraint::Curve { seed, .. } => self.slots[i].map_or(*seed, |j| x[j]),
            })
            .collect();
        Ok(Some((Solution { plan, fractions }, score)))
    }
}
fn linear(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for i in 0..n {
        let pivot = (i..n).max_by(|j, k| a[*j][i].abs().total_cmp(&a[*k][i].abs()))?;
        if a[pivot][i].abs() < 1e-18 {
            return None;
        }
        a.swap(i, pivot);
        b.swap(i, pivot);
        let d = a[i][i];
        for k in i..n {
            a[i][k] /= d;
        }
        b[i] /= d;
        for j in 0..n {
            if j != i {
                let d = a[j][i];
                for k in i..n {
                    a[j][k] -= d * a[i][k];
                }
                b[j] -= d * b[i];
            }
        }
    }
    Some(b)
}
/// Search a bounded set of local contact solutions. No claim of exhaustive roots.
/// Accepted contacts are evaluated again against the exact supplied curves.
pub fn solve<F: FnMut(usize, f64) -> Result<(Point, Point), String>>(
    cs: &[Constraint],
    options: &Options,
    eval: F,
) -> Result<Solution, String> {
    if cs.len() != if options.radius.is_some() { 2 } else { 3 }
        || cs.iter().all(|c| matches!(c, Constraint::Point(_)))
    {
        return Err(
            "Spatial Tangent needs two radius-constrained or three constraints including a curve"
                .into(),
        );
    }
    if options
        .radius
        .is_some_and(|r| !r.is_finite() || !(1e-7..=1e9).contains(&r))
        || !options.preferred.iter().all(|n| n.is_finite())
        || (norm(options.preferred) - 1.).abs() > 1e-8
        || options.solution.is_some_and(|n| n > 100000)
    {
        return Err("Invalid spatial Tangent options".into());
    }
    if options.from_first && !matches!(cs[0], Constraint::Curve { .. }) {
        return Err("FromFirstPoint needs a curve first".into());
    }
    let mut slots = Vec::new();
    let mut initial = vec![0.; 7];
    for (i, c) in cs.iter().enumerate() {
        match c {
            Constraint::Point(p) => {
                check_point(*p)?;
                slots.push(None);
            }
            Constraint::Curve { seed, .. } => {
                if !seed.is_finite() || !(0.0..=1.).contains(seed) {
                    return Err("Invalid bounded Tangent parameter".into());
                }
                if i == 0 && options.from_first {
                    slots.push(None);
                } else {
                    slots.push(Some(initial.len()));
                    initial.push(*seed);
                }
            }
        }
    }
    let mut problem = Problem {
        cs,
        options,
        eval,
        origin: [0.; 3],
        scale: 1.,
        slots,
        calls: 0,
    };
    let contacts = problem.contacts(&initial)?;
    let original: Vec<_> = contacts.iter().map(|c| c.0).collect();
    problem.origin = mul(
        original.iter().copied().fold([0.; 3], add),
        1. / cs.len() as f64,
    );
    problem.scale = original
        .iter()
        .map(|p| norm(sub(*p, problem.origin)))
        .fold(options.radius.unwrap_or(0.), f64::max)
        .max(1e-7);
    let mut normals = vec![options.preferred];
    for i in 0..cs.len() {
        for j in i + 1..cs.len() {
            if let (Some(a), Some(b)) = (contacts[i].1, contacts[j].1) {
                if let Ok(n) = unit(cross(a, b)) {
                    normals.push(n);
                }
            }
            if let Some(t) = contacts[i].1 {
                if let Ok(n) = unit(cross(sub(original[j], original[i]), t)) {
                    normals.push(n);
                }
            }
        }
    }
    let fitted = fit_circle(&original).ok().map(|p| p.0);
    if let Some(p) = &fitted {
        normals.insert(0, p.normal);
    }
    let mut starts = Vec::new();
    for n in normals.iter().take(5) {
        let radius = options
            .radius
            .unwrap_or_else(|| fitted.as_ref().map_or(problem.scale, |p| p.radius));
        let mut centers = vec![fitted.as_ref().map_or(problem.origin, |p| p.center)];
        if cs.len() == 2 {
            let delta = sub(original[1], original[0]);
            let length = norm(delta);
            if let Ok(perp) = unit(cross(*n, delta)) {
                let h = (radius * radius - length * length * 0.25).max(0.).sqrt();
                centers = vec![
                    add(problem.origin, mul(perp, h)),
                    sub(problem.origin, mul(perp, h)),
                ];
            }
        }
        for center in centers {
            let mut x = initial.clone();
            x[..3].copy_from_slice(&mul(sub(center, problem.origin), 1. / problem.scale));
            x[3..6].copy_from_slice(n);
            x[6] = radius / problem.scale;
            starts.push(x);
        }
    }
    if let Some(base) = starts.first().cloned() {
        for offset in [-0.15, -0.05, 0.05, 0.15] {
            let mut x = base.clone();
            for j in problem.slots.iter().flatten() {
                x[*j] = (x[*j] + offset).clamp(0., 1.);
            }
            starts.push(x);
        }
    }
    let mut candidates: Vec<(Solution, f64)> = Vec::new();
    for mut x in starts.into_iter().take(14) {
        let mut lambda = 1e-3;
        for _ in 0..64 {
            let residual = match problem.residual(&x) {
                Ok(r) => r,
                Err(e) => {
                    if candidates.is_empty() {
                        return Err(e);
                    }
                    break;
                }
            };
            let cost = residual.iter().map(|v| v * v).sum::<f64>();
            if cost < 1e-22 {
                break;
            }
            let n = x.len();
            let mut jac = vec![vec![0.; n]; residual.len()];
            for j in 0..n {
                let h = 1e-6 * x[j].abs().max(1.);
                let mut y = x.clone();
                y[j] += h;
                if j >= 7 && y[j] > 1. {
                    y[j] = x[j] - h;
                }
                let actual = y[j] - x[j];
                let r = problem.residual(&y)?;
                for i in 0..r.len() {
                    jac[i][j] = (r[i] - residual[i]) / actual;
                }
            }
            let mut a = vec![vec![0.; n]; n];
            let mut b = vec![0.; n];
            for row in 0..residual.len() {
                for i in 0..n {
                    b[i] -= jac[row][i] * residual[row];
                    for j in 0..n {
                        a[i][j] += jac[row][i] * jac[row][j];
                    }
                }
            }
            for (i, row) in a.iter_mut().enumerate() {
                row[i] += lambda;
            }
            let Some(step) = linear(a, b) else {
                break;
            };
            let mut y: Vec<_> = x.iter().zip(step).map(|(x, s)| x + s).collect();
            y[6] = y[6].max(1e-7 / problem.scale);
            for v in &mut y[7..] {
                *v = v.clamp(0., 1.);
            }
            let next = problem.residual(&y)?.iter().map(|v| v * v).sum::<f64>();
            if next < cost {
                x = y;
                lambda = (lambda * 0.25).max(1e-12);
            } else {
                lambda *= 10.;
                if lambda > 1e12 {
                    break;
                }
            }
        }
        if let Some(candidate) = problem.candidate(&x, &original)? {
            if !candidates.iter().any(|(s, _)| {
                norm(sub(s.plan.center, candidate.0.plan.center)) <= 1e-6
                    && (s.plan.radius - candidate.0.plan.radius).abs() <= 1e-6
                    && dot(s.plan.normal, candidate.0.plan.normal).abs() > 1. - 1e-8
            }) {
                candidates.push(candidate);
            }
            if options.solution.is_none() && candidates.iter().any(|(_, score)| *score < 1e-20) {
                break;
            }
        }
        if problem.calls > 20000 {
            break;
        }
    }
    candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
    let index = options.solution.unwrap_or(0);
    if candidates.is_empty() {
        return Err(
            "No verified spatial Tangent contact found within bounded search; change picks/options"
                .into(),
        );
    }
    if options.solution.is_none()
        && candidates.len() > 1
        && (candidates[0].1 - candidates[1].1).abs() < 1e-8 * problem.scale.powi(2)
    {
        return Err("Ambiguous spatial Tangent contacts; choose Solution=n".into());
    }
    if index >= candidates.len() {
        return Err(format!(
            "Spatial Tangent has {} verified solutions; choose Solution=1..{}",
            candidates.len(),
            candidates.len()
        ));
    }
    Ok(candidates.remove(index).0)
}
pub type NativeEval = unsafe extern "C" fn(*mut std::ffi::c_void, usize, f64, *mut f64) -> bool;
/// # Safety
/// constraints has count*4 doubles (count 2..3); seeds has count doubles; frame
/// has 13 doubles; output has ten writable doubles; error has capacity writable
/// bytes (1..4096). Buffers do not alias. Callback/context remain valid throughout
/// this synchronous call, do not unwind, and write six doubles on success.
/// No pointer, context, or callback is retained. Output is unchanged on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_spatial_solve(
    constraints: *const f64,
    seeds: *const f64,
    count: usize,
    frame: *const f64,
    from_first: bool,
    vertical: bool,
    solution: i32,
    eval: Option<NativeEval>,
    context: *mut std::ffi::c_void,
    output: *mut f64,
    error: *mut std::ffi::c_char,
    capacity: usize,
) -> bool {
    if constraints.is_null()
        || seeds.is_null()
        || frame.is_null()
        || output.is_null()
        || error.is_null()
        || !(1..=4096).contains(&capacity)
        || !(2..=3).contains(&count)
    {
        return false;
    }
    let result = std::panic::catch_unwind(|| -> Result<Solution, String> {
        let b = unsafe { std::slice::from_raw_parts(constraints, count * 4) };
        let seeds = unsafe { std::slice::from_raw_parts(seeds, count) };
        let f = unsafe { std::slice::from_raw_parts(frame, 13) };
        if b.iter().any(|n| !n.is_finite()) || f.iter().any(|n| !n.is_finite()) || solution < -1 {
            return Err("Invalid spatial Tangent native input".into());
        }
        let mut cs = Vec::new();
        for i in 0..count {
            cs.push(match b[i * 4 + 3] {
                0. => Constraint::Curve {
                    index: i,
                    seed: seeds[i],
                },
                1. => Constraint::Point([b[i * 4], b[i * 4 + 1], b[i * 4 + 2]]),
                _ => return Err("Invalid spatial Tangent constraint kind".into()),
            });
        }
        let eval = eval.ok_or("Missing exact native Tangent evaluator")?;
        solve(
            &cs,
            &Options {
                radius: if f[12] == 0. { None } else { Some(f[12]) },
                from_first,
                vertical,
                preferred: [f[9], f[10], f[11]],
                solution: if solution < 0 {
                    None
                } else {
                    Some(solution as usize)
                },
            },
            |i, u| {
                let mut data = [0.; 6];
                if !unsafe { eval(context, i, u, data.as_mut_ptr()) } {
                    return Err("Native Tangent source evaluation failed".into());
                }
                Ok(([data[0], data[1], data[2]], [data[3], data[4], data[5]]))
            },
        )
    });
    match result {
        Ok(Ok(s)) => {
            let mut data = [0.; 10];
            data[..3].copy_from_slice(&s.plan.center);
            data[3..6].copy_from_slice(&s.plan.normal);
            data[6] = s.plan.radius;
            data[7..7 + count].copy_from_slice(&s.fractions);
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), output, 10);
                *error = 0;
            }
            true
        }
        failure => {
            let message = match failure {
                Ok(Err(e)) => e,
                _ => "Spatial Tangent solver failed".into(),
            };
            let size = message.len().min(capacity - 1);
            unsafe {
                std::ptr::copy_nonoverlapping(message.as_ptr(), error.cast(), size);
                *error.add(size) = 0;
            }
            false
        }
    }
}
/// # Safety
/// points has count*3 readable doubles, frame/output have 12 doubles; buffers
/// do not alias, count 1..30003. Stateless inference for live or History replay.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_circle_plane_frame(
    points: *const f64,
    count: usize,
    frame: *const f64,
    output: *mut f64,
) -> bool {
    if points.is_null() || frame.is_null() || output.is_null() || !(1..=30003).contains(&count) {
        return false;
    }
    let p = unsafe { std::slice::from_raw_parts(points, count * 3) }
        .as_chunks::<3>()
        .0;
    let f = unsafe { std::slice::from_raw_parts(frame, 12) }
        .as_chunks::<3>()
        .0;
    let Ok(axes) = crate::circle::tangent_frame(p, f[0], [f[1], f[2], f[3]]) else {
        return false;
    };
    unsafe {
        std::ptr::copy_nonoverlapping(frame, output, 3);
        for (i, a) in axes.iter().enumerate() {
            std::ptr::copy_nonoverlapping(a.as_ptr(), output.add(3 + i * 3), 3);
        }
    }
    true
}
