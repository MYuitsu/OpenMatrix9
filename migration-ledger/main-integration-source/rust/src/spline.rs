//! OM9-CURVE-003/009: non-rational B-splines, interpolation and uniform rebuild.
//! Millimetres; bounded dense solves, no FreeCAD/Rhino runtime dependency.
pub const MAX_POLES: usize = 256;
pub const MAX_SAMPLES: usize = 4096;
pub const TOLERANCE: f64 = 1e-7;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Knots {
    #[default]
    Uniform,
    Chord,
    SqrtChord,
}
#[derive(Clone, Debug)]
pub struct Spline {
    pub poles: Vec<[f64; 3]>,
    /// Expanded knot vector, including the extension required for periodic evaluation.
    pub knots: Vec<f64>,
    pub degree: usize,
    pub periodic: bool,
}

/// Owned geometry may expose this borrowed basis only during validation.
/// Knots are distinct and multiplicities follow the native FreeCAD/OCCT
/// convention: periodic pole count excludes the final seam multiplicity.
pub struct Basis<'a> {
    pub poles: &'a [[f64; 3]],
    pub weights: Option<&'a [f64]>,
    pub knots: &'a [f64],
    pub multiplicities: &'a [usize],
    pub degree: usize,
    pub periodic: bool,
    pub domain: [f64; 2],
}

impl Basis<'_> {
    pub fn validate(&self) -> Result<(), String> {
        rebuild_options(self.poles.len(), self.degree)?;
        if self.poles.iter().flatten().any(|v| !v.is_finite() || v.abs() > 1e9) {
            return Err("Spline poles must be finite within supported coordinates".into());
        }
        if let Some(weights) = self.weights {
            if weights.len() != self.poles.len()
                || weights.iter().zip(self.poles).any(|(&w, p)| {
                    !w.is_finite() || w <= 0. || p.iter().any(|v| !(v * w).is_finite())
                })
            {
                return Err("Spline weights must match poles, be positive, and not overflow".into());
            }
        }
        if self.knots.len() < 2 || self.knots.len() != self.multiplicities.len()
            || self.knots.iter().any(|u| !u.is_finite())
            || self.knots.windows(2).any(|u| u[0] >= u[1] || !(u[1] - u[0]).is_finite())
        {
            return Err("Spline knots must be finite, strictly increasing, and match multiplicities".into());
        }
        let last = self.knots.len() - 1;
        let total = self.multiplicities.iter().enumerate().try_fold(0usize, |sum, (i, &m)| {
            let limit = self.degree + usize::from(!self.periodic && (i == 0 || i == last));
            if m == 0 || m > limit { return None; }
            sum.checked_add(m)
        }).ok_or("Invalid spline knot multiplicity")?;
        let (minimum, maximum) = if self.periodic {
            if self.multiplicities[0] != self.multiplicities[last]
                || total.checked_sub(self.multiplicities[last]) != Some(self.poles.len())
            {
                return Err("Periodic spline seam multiplicities or pole count do not match".into());
            }
            (self.knots[0], self.knots[last])
        } else {
            if total != self.poles.len() + self.degree + 1 {
                return Err("Spline pole count does not match degree and knot multiplicities".into());
            }
            let at = |index: usize| {
                let mut end = 0;
                self.knots.iter().zip(self.multiplicities).find_map(|(&u, &m)| {
                    end += m;
                    (index < end).then_some(u)
                }).unwrap_or(f64::NAN)
            };
            (at(self.degree), at(self.poles.len()))
        };
        if self.domain.iter().any(|u| !u.is_finite())
            || self.domain[0] >= self.domain[1]
            || !(self.domain[1] - self.domain[0]).is_finite()
            || self.domain[0] < minimum || self.domain[1] > maximum
        {
            return Err("Spline domain must be finite, increasing, and inside the active knot interval".into());
        }
        Ok(())
    }
}
pub fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}
fn validate(points: &[[f64; 3]], limit: usize) -> Result<(), String> {
    if !(2..=limit).contains(&points.len()) {
        return Err(format!("Use 2 to {limit} distinct points"));
    }
    if points
        .iter()
        .flatten()
        .any(|x| !x.is_finite() || x.abs() > 1e9)
    {
        return Err("Coordinates must be finite and within 1e9 mm".into());
    }
    if points.windows(2).any(|p| distance(p[0], p[1]) < TOLERANCE) {
        return Err("Consecutive points must differ by at least 1e-7 mm".into());
    }
    Ok(())
}
pub fn rebuild_options(count: usize, degree: usize) -> Result<(), String> {
    if !(1..=11).contains(&degree) || !(degree + 1..=MAX_POLES).contains(&count) {
        return Err(
            "Degree must be 1 to 11; PointCount must exceed Degree and be at most 256".into(),
        );
    }
    Ok(())
}
pub fn parameters(points: &[[f64; 3]], mode: Knots, closed: bool) -> Vec<f64> {
    let mut params = vec![0.];
    let segments = points.len() - usize::from(!closed);
    for i in 0..segments {
        let d = distance(points[i], points[(i + 1) % points.len()]);
        let step = match mode {
            Knots::Uniform => 1.,
            Knots::Chord => d,
            Knots::SqrtChord => d.sqrt(),
        };
        params.push(params.last().unwrap() + step);
    }
    let total = *params.last().unwrap();
    params.iter_mut().for_each(|u| *u /= total);
    params
}
fn periodic_knots(params: &[f64], degree: usize) -> Vec<f64> {
    let n = params.len() - 1;
    (0..n + 2 * degree + 1)
        .map(|i| {
            let j = i as isize - degree as isize;
            params[j.rem_euclid(n as isize) as usize] + j.div_euclid(n as isize) as f64
        })
        .collect()
}
impl Spline {
    /// The generated evaluator uses an expanded, normalized [0,1] basis.
    /// Validate before slicing it into the host's distinct-knot convention.
    pub fn validate(&self) -> Result<(), String> {
        rebuild_options(self.poles.len(), self.degree)?;
        let extension = if self.periodic { 2 * self.degree } else { self.degree };
        let expected = self.poles.len().checked_add(extension).and_then(|n| n.checked_add(1));
        if Some(self.knots.len()) != expected
            || self.knots.iter().any(|u| !u.is_finite())
            || self.knots.windows(2).any(|u| u[0] > u[1])
        {
            return Err("Invalid expanded spline knot vector".into());
        }
        let end = self.poles.len() + if self.periodic { self.degree } else { 0 };
        if self.knots[self.degree] != 0. || self.knots[end] != 1. {
            return Err("Generated spline must use the normalized active domain [0,1]".into());
        }
        if self.periodic && (0..self.knots.len() - self.poles.len()).any(|i| {
            (self.knots[i + self.poles.len()] - self.knots[i] - 1.).abs() > 1e-12
        }) {
            return Err("Periodic spline extension does not repeat the knot intervals".into());
        }
        let (knots, multiplicities) = self.host_knots();
        Basis { poles: &self.poles, weights: None, knots: &knots, multiplicities: &multiplicities,
            degree: self.degree, periodic: self.periodic, domain: [0., 1.] }.validate()
    }
    pub fn basis(&self, u: f64) -> Vec<f64> {
        let n = self.poles.len();
        let extended = n + if self.periodic { self.degree } else { 0 };
        let u = if self.periodic {
            u.rem_euclid(1.)
        } else {
            u.clamp(0., 1.)
        };
        if !self.periodic && u == 1. {
            let mut out = vec![0.; n];
            out[n - 1] = 1.;
            return out;
        }
        let mut basis = vec![0.; self.knots.len() - 1];
        for (i, b) in basis.iter_mut().enumerate() {
            *b = f64::from(self.knots[i] <= u && u < self.knots[i + 1]);
        }
        for d in 1..=self.degree {
            for i in 0..basis.len() - d {
                let left = self.knots[i + d] - self.knots[i];
                let right = self.knots[i + d + 1] - self.knots[i + 1];
                basis[i] = if left > 0. {
                    (u - self.knots[i]) / left * basis[i]
                } else {
                    0.
                } + if right > 0. {
                    (self.knots[i + d + 1] - u) / right * basis[i + 1]
                } else {
                    0.
                };
            }
        }
        let mut out = vec![0.; n];
        for i in 0..extended {
            out[i % n] += basis[i];
        }
        out
    }
    pub fn value(&self, u: f64) -> [f64; 3] {
        let b = self.basis(u);
        std::array::from_fn(|axis| self.poles.iter().zip(&b).map(|(p, w)| p[axis] * w).sum())
    }
    /// OCCT consumes distinct knots/multiplicities on [0,1] for a periodic spline.
    pub fn host_knots(&self) -> (Vec<f64>, Vec<usize>) {
        let source = if self.periodic {
            &self.knots[self.degree..self.degree + self.poles.len() + 1]
        } else {
            &self.knots
        };
        let mut knots = Vec::new();
        let mut mults = Vec::new();
        for &u in source {
            if knots.last() == Some(&u) {
                *mults.last_mut().unwrap() += 1;
            } else {
                knots.push(u);
                mults.push(1);
            }
        }
        (knots, mults)
    }
}
/// Partial pivoting solves XYZ together; reject ill-conditioned systems instead of outputting NaNs.
fn solve(mut a: Vec<Vec<f64>>, mut rhs: Vec<[f64; 3]>) -> Result<Vec<[f64; 3]>, String> {
    let n = a.len();
    for k in 0..n {
        let pivot = (k..n)
            .max_by(|&i, &j| a[i][k].abs().total_cmp(&a[j][k].abs()))
            .unwrap();
        if a[pivot][k].abs() < 1e-12 {
            return Err(
                "Points produce an ill-conditioned spline; change spacing, degree or point count"
                    .into(),
            );
        }
        a.swap(k, pivot);
        rhs.swap(k, pivot);
        for i in k + 1..n {
            let f = a[i][k] / a[k][k];
            for j in k..n {
                a[i][j] -= f * a[k][j];
            }
            for axis in 0..3 {
                rhs[i][axis] -= f * rhs[k][axis];
            }
        }
    }
    let mut x = vec![[0.; 3]; n];
    for i in (0..n).rev() {
        for axis in 0..3 {
            x[i][axis] =
                (rhs[i][axis] - (i + 1..n).map(|j| a[i][j] * x[j][axis]).sum::<f64>()) / a[i][i];
        }
    }
    if x.iter().flatten().any(|v| !v.is_finite() || v.abs() > 1e9) {
        return Err("Spline poles exceed coordinate limits".into());
    }
    Ok(x)
}
pub fn interpolate(
    points: &[[f64; 3]],
    requested_degree: usize,
    mode: Knots,
    periodic: bool,
) -> Result<Spline, String> {
    validate(points, MAX_POLES)?;
    if !(1..=11).contains(&requested_degree) || requested_degree % 2 == 0 {
        return Err("InterpCrv supports odd degrees 1, 3, 5, 7, 9 and 11".into());
    }
    if periodic && (points.len() < 3 || distance(points[0], *points.last().unwrap()) < TOLERANCE) {
        return Err(
            "A smooth closed curve needs at least 3 distinct points without a repeated endpoint"
                .into(),
        );
    }
    let n = points.len();
    let mut degree = requested_degree.min(n - 1);
    if periodic && degree % 2 == 0 {
        degree -= 1;
    }
    let params = parameters(points, mode, periodic);
    let knots = if periodic {
        periodic_knots(&params, degree)
    } else {
        let mut knots = vec![0.; degree + 1];
        for j in 1..n - degree {
            knots.push(params[j..j + degree].iter().sum::<f64>() / degree as f64);
        }
        knots.extend(vec![1.; degree + 1]);
        knots
    };
    let mut spline = Spline {
        poles: vec![[0.; 3]; n],
        knots,
        degree,
        periodic,
    };
    let matrix = params[..n].iter().map(|&u| spline.basis(u)).collect();
    spline.poles = solve(matrix, points.to_vec())?;
    Ok(spline)
}
/// Interpolate control-net rows on genuinely uniform distinct knots. Constant
/// rows are allowed: this fits homogeneous surface coordinates, not a user curve.
pub fn uniform_net_interpolate(points: &[[f64; 3]], periodic: bool) -> Result<Spline, String> {
    let n = points.len();
    if !(2..=MAX_POLES).contains(&n)
        || (periodic && n < 3)
        || points
            .iter()
            .flatten()
            .any(|x| !x.is_finite() || x.abs() > 1e9)
    {
        return Err("Uniform Loft needs 2 to 256 finite control rows (3 for closed)".into());
    }
    let mut degree = 3.min(n - 1);
    if periodic && degree % 2 == 0 {
        degree -= 1;
    }
    let params = (0..if periodic { n + 1 } else { n })
        .map(|i| i as f64 / (n - usize::from(!periodic)) as f64)
        .collect::<Vec<_>>();
    let knots = if periodic {
        periodic_knots(&params, degree)
    } else {
        let mut knots = vec![0.; degree + 1];
        knots.extend((1..n - degree).map(|j| j as f64 / (n - degree) as f64));
        knots.extend(vec![1.; degree + 1]);
        knots
    };
    let mut spline = Spline {
        poles: vec![[0.; 3]; n],
        knots,
        degree,
        periodic,
    };
    let matrix = params[..n].iter().map(|&u| spline.basis(u)).collect();
    spline.poles = solve(matrix, points.to_vec())?;
    Ok(spline)
}
/// Fit equal arc-length samples to a uniformly knotted spline, preserving open endpoints.
/// Closed samples include the duplicated last endpoint, which is removed from the solve.
pub fn rebuild(
    samples: &[[f64; 3]],
    count: usize,
    degree: usize,
    periodic: bool,
) -> Result<Spline, String> {
    rebuild_options(count, degree)?;
    validate(samples, MAX_SAMPLES)?;
    if periodic && distance(samples[0], *samples.last().unwrap()) > TOLERANCE {
        return Err("Closed samples must end at their start point".into());
    }
    let samples = if periodic {
        &samples[..samples.len() - 1]
    } else {
        samples
    };
    if samples.len() < count * 2 {
        return Err("Rebuild needs at least twice PointCount samples".into());
    }
    let knots = if periodic {
        let params = (0..=count)
            .map(|i| i as f64 / count as f64)
            .collect::<Vec<_>>();
        periodic_knots(&params, degree)
    } else {
        let mut knots = vec![0.; degree + 1];
        knots.extend((1..count - degree).map(|i| i as f64 / (count - degree) as f64));
        knots.extend(vec![1.; degree + 1]);
        knots
    };
    let mut spline = Spline {
        poles: vec![[0.; 3]; count],
        knots,
        degree,
        periodic,
    };
    let first = if periodic { 0 } else { 1 };
    let end = if periodic { count } else { count - 1 };
    if !periodic {
        spline.poles[0] = samples[0];
        spline.poles[count - 1] = *samples.last().unwrap();
    }
    let unknowns = end - first;
    if unknowns == 0 {
        return Ok(spline);
    }
    let mut normal = vec![vec![0.; unknowns]; unknowns];
    let mut rhs = vec![[0.; 3]; unknowns];
    for (i, &point) in samples.iter().enumerate() {
        let u = i as f64 / (samples.len() - usize::from(!periodic)) as f64;
        let b = spline.basis(u);
        let target: [f64; 3] = std::array::from_fn(|axis| {
            point[axis]
                - if periodic {
                    0.
                } else {
                    b[0] * spline.poles[0][axis] + b[count - 1] * spline.poles[count - 1][axis]
                }
        });
        for j in first..end {
            for k in first..end {
                normal[j - first][k - first] += b[j] * b[k];
            }
            for axis in 0..3 {
                rhs[j - first][axis] += b[j] * target[axis];
            }
        }
    }
    let fit = solve(normal, rhs)?;
    spline.poles[first..end].copy_from_slice(&fit);
    Ok(spline)
}
