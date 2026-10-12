// SPDX-License-Identifier: LGPL-2.1-or-later
//! Portable, owned NURBS cage deformation; native kernels remain host adapters.
//! Full clamped knot vectors have count+degree+1 entries. Control points are
//! ordered with U fastest: index=((w*counts[1])+v)*counts[0]+u.
#[derive(Debug, Clone, PartialEq)]
pub struct Cage {
    pub counts: [usize; 3],
    pub degrees: [usize; 3],
    pub knots: [Vec<f64>; 3],
    pub points: Vec<[f64; 3]>,
    pub weights: Vec<f64>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Region {
    Global,
    Local {
        min: [f64; 3],
        max: [f64; 3],
        falloff: f64,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidCage,
    InvalidFrame,
    InvalidRegion,
    NonFinite,
    Denominator,
    InvalidBuffer,
    Capacity,
    Panic,
    InverseUnsupported,
    Singular,
    AmbiguousInverse,
    NoInverse,
}
pub const IDENTITY: [f64; 16] = [
    1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
];
pub struct Deformation {
    cage: Cage,
    cage_to_world: [f64; 16],
    affine_current: Option<[f64; 16]>,
    world_to_parameter: [f64; 16],
    region: Region,
    inverse: std::sync::OnceLock<Result<InverseState, Error>>,
}
enum InverseState {
    Affine([f64; 16]),
    Bounded { scale: f64 },
}
const MAX_POINTS: usize = 1_000_000;
impl Cage {
    /// Generate a uniform clamped box cage. Greville CV positions reproduce
    /// the affine box mapping; Rust spells the reserved name as `Cage::r#box`.
    pub fn r#box(
        counts: [usize; 3],
        degrees: [usize; 3],
        lo: [f64; 3],
        hi: [f64; 3],
    ) -> Result<(Self, [f64; 16]), Error> {
        let total = box_dimensions(counts, degrees)?;
        let mut reference = IDENTITY;
        for a in 0..3 {
            let span = hi[a] - lo[a];
            if !lo[a].is_finite() || !hi[a].is_finite() || !span.is_finite() || span <= 0. {
                return Err(Error::InvalidFrame);
            }
            reference[a * 4 + a] = 1. / span;
            reference[a * 4 + 3] = -lo[a] / span;
        }
        validate_frame(&reference)?;
        let knots: [Vec<f64>; 3] = std::array::from_fn(|a| {
            let n = counts[a];
            let p = degrees[a];
            (0..n + p + 1)
                .map(|i| {
                    if i <= p {
                        0.
                    } else if i >= n {
                        1.
                    } else {
                        (i - p) as f64 / (n - p) as f64
                    }
                })
                .collect()
        });
        let greville: [Vec<f64>; 3] = std::array::from_fn(|a| {
            (0..counts[a])
                .map(|i| {
                    knots[a][i + 1..i + degrees[a] + 1].iter().sum::<f64>() / degrees[a] as f64
                })
                .collect()
        });
        let mut points = Vec::with_capacity(total);
        for w in 0..counts[2] {
            for v in 0..counts[1] {
                for u in 0..counts[0] {
                    let index = [u, v, w];
                    points.push(std::array::from_fn(|a| {
                        lo[a] + (hi[a] - lo[a]) * greville[a][index[a]]
                    }));
                }
            }
        }
        let cage = Self {
            counts,
            degrees,
            knots,
            points,
            weights: vec![1.; total],
        };
        cage.validate()?;
        Ok((cage, reference))
    }
    pub fn validate(&self) -> Result<(), Error> {
        let mut total = 1usize;
        for axis in 0..3 {
            let n = self.counts[axis];
            let p = self.degrees[axis];
            let knots = &self.knots[axis];
            if !(1..=16).contains(&p) || n <= p || n > MAX_POINTS {
                return Err(Error::InvalidCage);
            }
            total = total
                .checked_mul(n)
                .filter(|n| *n <= MAX_POINTS)
                .ok_or(Error::InvalidCage)?;
            if knots.len() != n + p + 1
                || knots.iter().any(|x| !x.is_finite())
                || knots.windows(2).any(|w| w[0] > w[1])
            {
                return Err(Error::InvalidCage);
            }
            let lo = knots[p];
            let hi = knots[n];
            if lo >= hi
                || !(hi - lo).is_finite()
                || knots[..=p].iter().any(|x| *x != lo)
                || knots[n..].iter().any(|x| *x != hi)
                || knots[p + 1] == lo
                || knots[n - 1] == hi
            {
                return Err(Error::InvalidCage);
            }
            let mut run = 1;
            for w in knots.windows(2) {
                run = if w[0] == w[1] { run + 1 } else { 1 };
                if run > p + 1 {
                    return Err(Error::InvalidCage);
                }
            }
        }
        if self.points.len() != total
            || self.weights.len() != total
            || self.points.iter().flatten().any(|v| !v.is_finite())
            || self.weights.iter().any(|w| !w.is_finite() || *w <= 0.)
        {
            return Err(Error::InvalidCage);
        }
        Ok(())
    }
    /// Evaluate in the actual knot domains. Outside parameters continue the
    /// first/last polynomial span; invalid rational denominators are rejected.
    pub fn evaluate(&self, p: [f64; 3]) -> Result<[f64; 3], Error> {
        self.validate()?;
        if p.iter().any(|v| !v.is_finite()) {
            return Err(Error::NonFinite);
        }
        if let Some(affine) = self.exact_normalized_affine() {
            let normalized = std::array::from_fn(|a| {
                let lo = self.knots[a][self.degrees[a]];
                (p[a] - lo) / (self.knots[a][self.counts[a]] - lo)
            });
            return checked_transform(&affine, normalized);
        }
        self.evaluate_validated(p)
    }
    /// Call only after validation. Uniform weights and every Greville CV must
    /// agree exactly, so this proves the tensor map is affine independently of
    /// reference frames and influence regions. No fitting tolerance is used.
    fn exact_normalized_affine(&self) -> Option<[f64; 16]> {
        if self.weights.iter().any(|w| *w != self.weights[0]) {
            return None;
        }
        let c = self;
        let origin = c.points[0];
        let endpoints = [
            c.points[c.counts[0] - 1],
            c.points[(c.counts[1] - 1) * c.counts[0]],
            c.points[(c.counts[2] - 1) * c.counts[0] * c.counts[1]],
        ];
        let mut affine = IDENTITY;
        for a in 0..3 {
            affine[a * 4 + 3] = origin[a];
            for (b, end) in endpoints.iter().enumerate() {
                affine[a * 4 + b] = end[a] - origin[a];
            }
        }
        let greville: [Vec<f64>; 3] = std::array::from_fn(|a| {
            let p = c.degrees[a];
            let lo = c.knots[a][p];
            let size = c.knots[a][c.counts[a]] - lo;
            (0..c.counts[a])
                .map(|i| {
                    c.knots[a][i + 1..i + p + 1]
                        .iter()
                        .map(|k| (*k - lo) / size)
                        .sum::<f64>()
                        / p as f64
                })
                .collect()
        });
        for k in 0..c.counts[2] {
            for j in 0..c.counts[1] {
                for i in 0..c.counts[0] {
                    if transform(&affine, [greville[0][i], greville[1][j], greville[2][k]])
                        != c.points[(k * c.counts[1] + j) * c.counts[0] + i]
                    {
                        return None;
                    }
                }
            }
        }
        if affine.iter().any(|v| !v.is_finite()) {
            None
        } else {
            Some(affine)
        }
    }
    fn evaluate_validated(&self, p: [f64; 3]) -> Result<[f64; 3], Error> {
        if p.iter().any(|v| !v.is_finite()) {
            return Err(Error::NonFinite);
        }
        let mut starts = [0; 3];
        let mut basis = [[0.; 17]; 3];
        for a in 0..3 {
            let (start, b) = axis_basis(self.counts[a], self.degrees[a], &self.knots[a], p[a])?;
            starts[a] = start;
            basis[a] = b;
        }
        // Normalize weights to avoid gratuitous overflow for large uniform weights.
        let mut scale = 0f64;
        for k in 0..=self.degrees[2] {
            for j in 0..=self.degrees[1] {
                for i in 0..=self.degrees[0] {
                    let index = ((starts[2] + k) * self.counts[1] + starts[1] + j) * self.counts[0]
                        + starts[0]
                        + i;
                    let coefficient = basis[0][i] * basis[1][j] * basis[2][k];
                    if !coefficient.is_finite() {
                        return Err(Error::NonFinite);
                    }
                    if coefficient != 0. {
                        scale = scale.max(self.weights[index]);
                    }
                }
            }
        }
        let mut numerator = [0.; 3];
        let mut denominator = 0.;
        for k in 0..=self.degrees[2] {
            for j in 0..=self.degrees[1] {
                for i in 0..=self.degrees[0] {
                    let index = ((starts[2] + k) * self.counts[1] + starts[1] + j) * self.counts[0]
                        + starts[0]
                        + i;
                    let coefficient = basis[0][i] * basis[1][j] * basis[2][k];
                    if coefficient == 0. {
                        continue;
                    }
                    let factor = coefficient * (self.weights[index] / scale);
                    denominator += factor;
                    for (a, value) in numerator.iter_mut().enumerate() {
                        *value += factor * self.points[index][a];
                    }
                }
            }
        }
        if !denominator.is_finite() || denominator <= 0. {
            return Err(Error::Denominator);
        }
        let result = numerator.map(|v| v / denominator);
        if result.iter().any(|v| !v.is_finite()) {
            return Err(Error::NonFinite);
        }
        Ok(result)
    }
}
fn axis_basis(n: usize, p: usize, knots: &[f64], u: f64) -> Result<(usize, [f64; 17]), Error> {
    let span = if u <= knots[p] {
        p
    } else if u >= knots[n] {
        n - 1
    } else {
        knots.partition_point(|k| *k <= u) - 1
    };
    let mut b = [0.; 17];
    let mut left = [0.; 17];
    let mut right = [0.; 17];
    b[0] = 1.;
    for j in 1..=p {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.;
        for r in 0..j {
            // Computing the knot difference directly avoids cancellation in
            // right+left when extrapolating far beyond the reference box.
            let denominator = knots[span + r + 1] - knots[span + 1 - j + r];
            let term = if denominator == 0. {
                0.
            } else {
                b[r] / denominator
            };
            b[r] = saved + right[r + 1] * term;
            saved = left[j - r] * term;
        }
        b[j] = saved;
    }
    if b.iter().any(|v| !v.is_finite()) {
        return Err(Error::NonFinite);
    }
    Ok((span - p, b))
}
impl Deformation {
    /// Points stay in local cage coordinates. Validate/prove their map there,
    /// then compose an independent invertible affine pose to WORLD coordinates.
    /// The original world-to-parameter frame and Local world bounds are unchanged.
    pub fn with_pose(
        cage: Cage,
        world_to_parameter: [f64; 16],
        region: Region,
        cage_to_world: [f64; 16],
    ) -> Result<Self, Error> {
        validate_frame(&cage_to_world)?;
        let mut deformation = Self::new(cage, world_to_parameter, region)?;
        deformation.cage_to_world = cage_to_world;
        deformation.affine_current = match deformation.affine_current {
            Some(local) => Some(matrix_product(&cage_to_world, &local)?),
            None => None,
        };
        Ok(deformation)
    }
    /// Capture coordinates in the CURRENT cage, independent of the original
    /// reference frame. Affine cages support global inversion. Other cages use
    /// bounded multi-start Newton, with sampled Jacobian/fold checks; this is
    /// not an exhaustive proof of uniqueness and has no outside-domain solver.
    pub fn inverse_point(&self, p: [f64; 3]) -> Result<[f64; 3], Error> {
        if p.iter().any(|v| !v.is_finite()) {
            return Err(Error::NonFinite);
        }
        if self.region != Region::Global {
            return Err(Error::InverseUnsupported);
        }
        let state = self.inverse.get_or_init(|| self.prepare_inverse());
        let state = state.as_ref().map_err(Clone::clone)?;
        if let InverseState::Affine(m) = state {
            let uvw = transform(m, p);
            if uvw.iter().any(|v| !v.is_finite()) {
                return Err(Error::NonFinite);
            }
            return Ok(uvw);
        }
        let InverseState::Bounded { scale } = state else {
            return Err(Error::NoInverse);
        };
        let tolerance = scale * 1e-10;
        let mut solution: Option<[f64; 3]> = None;
        for k in 0..3 {
            for j in 0..3 {
                for i in 0..3 {
                    let mut uvw = [i as f64 * 0.5, j as f64 * 0.5, k as f64 * 0.5];
                    for _ in 0..40 {
                        let current = self.evaluate_normalized(uvw)?;
                        let residual = sub(current, p);
                        let distance = norm(residual);
                        if distance <= tolerance {
                            if let Some(previous) = solution {
                                if norm(sub(previous, uvw)) > 1e-6 {
                                    return Err(Error::AmbiguousInverse);
                                }
                            } else {
                                solution = Some(uvw);
                            }
                            break;
                        }
                        let jac = self.jacobian(uvw)?;
                        let Some(step) = solve(jac, residual) else {
                            break;
                        };
                        let mut advanced = false;
                        for power in 0..12 {
                            let factor = 0.5f64.powi(power);
                            let next = std::array::from_fn(|a| uvw[a] - step[a] * factor);
                            if next.iter().any(|v| *v < 0. || *v > 1. || !v.is_finite()) {
                                continue;
                            }
                            if norm(sub(self.evaluate_normalized(next)?, p)) < distance {
                                uvw = next;
                                advanced = true;
                                break;
                            }
                        }
                        if !advanced {
                            break;
                        }
                    }
                }
            }
        }
        solution.ok_or(Error::NoInverse)
    }
    pub fn evaluate_normalized(&self, p: [f64; 3]) -> Result<[f64; 3], Error> {
        if p.iter().any(|v| !v.is_finite()) {
            return Err(Error::NonFinite);
        }
        if let Some(affine) = self.affine_current {
            return checked_transform(&affine, p);
        }
        let parameters = std::array::from_fn(|a| {
            let lo = self.cage.knots[a][self.cage.degrees[a]];
            lo + p[a] * (self.cage.knots[a][self.cage.counts[a]] - lo)
        });
        let local = self.cage.evaluate_validated(parameters)?;
        checked_transform(&self.cage_to_world, local)
    }
    fn prepare_inverse(&self) -> Result<InverseState, Error> {
        if let Some(m) = self.affine_current {
            return invert_affine(m)
                .map(InverseState::Affine)
                .ok_or(Error::Singular);
        }
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for local in &self.cage.points {
            let p = checked_transform(&self.cage_to_world, *local)?;
            for a in 0..3 {
                min[a] = min[a].min(p[a]);
                max[a] = max[a].max(p[a]);
            }
        }
        let scale = norm(sub(max, min));
        if !scale.is_finite() || scale <= 0. {
            return Err(Error::Singular);
        }
        let mut sign = 0.;
        for k in 0..5 {
            for j in 0..5 {
                for i in 0..5 {
                    let jac = self.jacobian([i as f64 * 0.25, j as f64 * 0.25, k as f64 * 0.25])?;
                    let determinant = determinant(jac);
                    let jac_scale = jac
                        .iter()
                        .flatten()
                        .copied()
                        .map(f64::abs)
                        .fold(0., f64::max);
                    if !determinant.is_finite() || determinant.abs() <= 1e-10 * jac_scale.powi(3) {
                        return Err(Error::Singular);
                    }
                    if sign == 0. {
                        sign = determinant.signum();
                    } else if sign != determinant.signum() {
                        return Err(Error::AmbiguousInverse);
                    }
                }
            }
        }
        Ok(InverseState::Bounded { scale })
    }
    fn jacobian(&self, p: [f64; 3]) -> Result<[[f64; 3]; 3], Error> {
        let mut jac = [[0.; 3]; 3];
        for axis in 0..3 {
            let mut lo = p;
            let mut hi = p;
            lo[axis] = (p[axis] - 1e-5).max(0.);
            hi[axis] = (p[axis] + 1e-5).min(1.);
            let width = hi[axis] - lo[axis];
            if width <= 0. {
                return Err(Error::InverseUnsupported);
            }
            let a = self.evaluate_normalized(lo)?;
            let b = self.evaluate_normalized(hi)?;
            for row in 0..3 {
                jac[row][axis] = (b[row] - a[row]) / width;
            }
        }
        Ok(jac)
    }
    /// The reference is explicitly affine: row-major matrix maps original world
    /// coordinates to normalized cage parameters. Non-affine inverse cages are
    /// unsupported; callers must not substitute an approximate inverse.
    pub fn new(cage: Cage, world_to_parameter: [f64; 16], region: Region) -> Result<Self, Error> {
        cage.validate()?;
        validate_frame(&world_to_parameter)?;
        if let Region::Local { min, max, falloff } = &region {
            if !falloff.is_finite()
                || *falloff < 0.
                || (0..3).any(|i| !min[i].is_finite() || !max[i].is_finite() || min[i] > max[i])
            {
                return Err(Error::InvalidRegion);
            }
        }
        let affine_current = cage.exact_normalized_affine();
        Ok(Self {
            cage,
            cage_to_world: IDENTITY,
            affine_current,
            world_to_parameter,
            region,
            inverse: std::sync::OnceLock::new(),
        })
    }
    pub fn deform(&self, p: [f64; 3]) -> Result<[f64; 3], Error> {
        if p.iter().any(|v| !v.is_finite()) {
            return Err(Error::NonFinite);
        }
        let influence = region_influence(&self.region, p);
        if influence == 0. {
            return Ok(p);
        }
        let uvw = transform(&self.world_to_parameter, p);
        let target = self.evaluate_normalized(uvw)?;
        let result = if influence == 1. {
            target
        } else {
            std::array::from_fn(|a| (1. - influence) * p[a] + influence * target[a])
        };
        if result.iter().any(|v| !v.is_finite()) {
            return Err(Error::NonFinite);
        }
        Ok(result)
    }
    pub fn deform_points(&self, p: &[[f64; 3]]) -> Result<Vec<[f64; 3]>, Error> {
        if p.len() > MAX_POINTS {
            return Err(Error::Capacity);
        }
        p.iter().map(|p| self.deform(*p)).collect()
    }
    /// Conservative proof using uniform rational weights and all Greville CVs.
    /// Equality is exact (no fit tolerance); false negatives fall back to cage
    /// evaluation. Nonuniform weights and local regions deliberately return None.
    pub fn exact_affine_transform(&self) -> Result<Option<[f64; 16]>, Error> {
        if self.region != Region::Global {
            return Ok(None);
        }
        let Some(affine) = self.affine_current else {
            return Ok(None);
        };
        Ok(Some(matrix_product(&affine, &self.world_to_parameter)?))
    }
}
fn transform(m: &[f64; 16], p: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|a| {
        m[a * 4] * p[0] + m[a * 4 + 1] * p[1] + m[a * 4 + 2] * p[2] + m[a * 4 + 3]
    })
}
fn checked_transform(m: &[f64; 16], p: [f64; 3]) -> Result<[f64; 3], Error> {
    if p.iter().any(|v| !v.is_finite()) {
        return Err(Error::NonFinite);
    }
    let result = transform(m, p);
    if result.iter().any(|v| !v.is_finite()) {
        Err(Error::NonFinite)
    } else {
        Ok(result)
    }
}
fn matrix_product(left: &[f64; 16], right: &[f64; 16]) -> Result<[f64; 16], Error> {
    let mut result = [0.; 16];
    for r in 0..4 {
        for col in 0..4 {
            for k in 0..4 {
                result[r * 4 + col] += left[r * 4 + k] * right[k * 4 + col];
            }
        }
    }
    if result.iter().any(|v| !v.is_finite()) {
        Err(Error::NonFinite)
    } else {
        Ok(result)
    }
}
fn validate_frame(m: &[f64; 16]) -> Result<(), Error> {
    let det = m[0] * (m[5] * m[10] - m[6] * m[9]) - m[1] * (m[4] * m[10] - m[6] * m[8])
        + m[2] * (m[4] * m[9] - m[5] * m[8]);
    if m.iter().any(|v| !v.is_finite())
        || m[12..] != [0., 0., 0., 1.]
        || !det.is_finite()
        || det == 0.
    {
        Err(Error::InvalidFrame)
    } else {
        Ok(())
    }
}
fn box_dimensions(counts: [usize; 3], degrees: [usize; 3]) -> Result<usize, Error> {
    let mut total = 1usize;
    for a in 0..3 {
        if !(2..=128).contains(&counts[a])
            || !(1..=16).contains(&degrees[a])
            || degrees[a] >= counts[a]
        {
            return Err(Error::InvalidCage);
        }
        total = total
            .checked_mul(counts[a])
            .filter(|v| *v <= MAX_POINTS)
            .ok_or(Error::Capacity)?;
    }
    Ok(total)
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn norm(a: [f64; 3]) -> f64 {
    a.into_iter().fold(0., f64::hypot)
}
fn determinant(m: [[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}
fn solve(m: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let d = determinant(m);
    let scale = m.iter().flatten().copied().map(f64::abs).fold(0., f64::max);
    if !d.is_finite() || d.abs() <= 1e-12 * scale.powi(3) {
        return None;
    }
    let result = std::array::from_fn(|a| {
        let mut replaced = m;
        for row in 0..3 {
            replaced[row][a] = b[row];
        }
        determinant(replaced) / d
    });
    if result.iter().any(|v| !v.is_finite()) {
        None
    } else {
        Some(result)
    }
}
fn invert_affine(m: [f64; 16]) -> Option<[f64; 16]> {
    let linear = std::array::from_fn(|r| std::array::from_fn(|c| m[r * 4 + c]));
    let mut inverse = IDENTITY;
    for col in 0..3 {
        let mut unit = [0.; 3];
        unit[col] = 1.;
        let axis = solve(linear, unit)?;
        for row in 0..3 {
            inverse[row * 4 + col] = axis[row];
        }
    }
    let t = transform(&inverse, [-m[3], -m[7], -m[11]]);
    for row in 0..3 {
        inverse[row * 4 + 3] = t[row];
    }
    Some(inverse)
}
#[repr(C)]
pub struct Om9CageDescriptor {
    pub counts: [usize; 3],
    pub degrees: [usize; 3],
    pub knots: [*const f64; 3],
    pub knot_lengths: [usize; 3],
    pub points: *const f64,
    pub point_count: usize,
    pub weights: *const f64,
    pub weight_count: usize,
    pub world_to_parameter: [f64; 16],
    pub region: u32,
    pub local_min: [f64; 3],
    pub local_max: [f64; 3],
    pub falloff: f64,
}
pub struct CageHandle {
    deformation: Deformation,
}
/// Owned capture snapshot. Parameters are recovered from the CURRENT cage once;
/// every edit starts from captured world points, preventing accumulated drift.
pub struct CageBinding {
    parameters: Vec<[f64; 3]>,
    original_points: Vec<[f64; 3]>,
}
impl CageBinding {
    pub fn capture(current: &Deformation, points: &[[f64; 3]]) -> Result<Self, Error> {
        if points.len() > MAX_POINTS {
            return Err(Error::Capacity);
        }
        let parameters = points
            .iter()
            .map(|p| current.inverse_point(*p))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            parameters,
            original_points: points.to_vec(),
        })
    }
    pub fn parameters(&self) -> &[[f64; 3]] {
        &self.parameters
    }
    pub fn apply(&self, edited: &Deformation) -> Result<Vec<[f64; 3]>, Error> {
        self.parameters
            .iter()
            .zip(&self.original_points)
            .map(|(uvw, original)| {
                let influence = region_influence(&edited.region, *original);
                if influence == 0. {
                    return Ok(*original);
                }
                let target = edited.evaluate_normalized(*uvw)?;
                let result = if influence == 1. {
                    target
                } else {
                    std::array::from_fn(|a| (1. - influence) * original[a] + influence * target[a])
                };
                if result.iter().any(|v| !v.is_finite()) {
                    Err(Error::NonFinite)
                } else {
                    Ok(result)
                }
            })
            .collect()
    }
}
fn region_influence(region: &Region, p: [f64; 3]) -> f64 {
    match *region {
        Region::Global => 1.,
        Region::Local { min, max, falloff } => {
            let distance = (0..3)
                .map(|i| {
                    if p[i] < min[i] {
                        min[i] - p[i]
                    } else if p[i] > max[i] {
                        p[i] - max[i]
                    } else {
                        0.
                    }
                })
                .fold(0., f64::hypot);
            if distance == 0. {
                1.
            } else if falloff == 0. || distance >= falloff {
                0.
            } else {
                let t = distance / falloff;
                1. - t * t * (3. - 2. * t)
            }
        }
    }
}
fn status(error: Error) -> i32 {
    match error {
        Error::InvalidCage => -1,
        Error::InvalidFrame => -2,
        Error::InvalidRegion => -3,
        Error::NonFinite => -4,
        Error::Denominator => -5,
        Error::InvalidBuffer => -6,
        Error::Capacity => -7,
        Error::Panic => -8,
        Error::InverseUnsupported => -9,
        Error::Singular => -10,
        Error::AmbiguousInverse => -11,
        Error::NoInverse => -12,
    }
}
fn ffi_status(f: impl FnOnce() -> Result<i32, Error>) -> i32 {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(Ok(code)) => code,
        Ok(Err(e)) => status(e),
        Err(_) => status(Error::Panic),
    }
}
fn aligned<T>(pointer: *const T) -> bool {
    !pointer.is_null() && (pointer as usize).is_multiple_of(std::mem::align_of::<T>())
}
/// # Safety
/// Inputs are readable aligned arrays of three doubles; outputs are writable
/// aligned arrays of three usize values and may not overlap each other. Inputs
/// are copied before output writes. Invalid/nonintegral/out-of-range doubles
/// are rejected before integer conversion. All outputs remain unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_box_parameters(
    counts: *const f64,
    degrees: *const f64,
    out_counts: *mut usize,
    out_degrees: *mut usize,
) -> i32 {
    ffi_status(|| {
        if !aligned(counts) || !aligned(degrees) || !aligned(out_counts) || !aligned(out_degrees) {
            return Err(Error::InvalidBuffer);
        }
        let counts = unsafe { counts.cast::<[f64; 3]>().read() };
        let degrees = unsafe { degrees.cast::<[f64; 3]>().read() };
        if (0..3).any(|a| {
            !counts[a].is_finite()
                || !degrees[a].is_finite()
                || !(2. ..=128.).contains(&counts[a])
                || !(1. ..=16.).contains(&degrees[a])
                || counts[a].fract() != 0.
                || degrees[a].fract() != 0.
                || degrees[a] >= counts[a]
        }) {
            return Err(Error::InvalidCage);
        }
        let counts = counts.map(|v| v as usize);
        let degrees = degrees.map(|v| v as usize);
        box_dimensions(counts, degrees)?;
        unsafe {
            std::ptr::copy_nonoverlapping(counts.as_ptr(), out_counts, 3);
            std::ptr::copy_nonoverlapping(degrees.as_ptr(), out_degrees, 3);
        }
        Ok(0)
    })
}
/// # Safety
/// Inputs are readable aligned arrays of three counts/degrees/lo/hi values.
/// Points are writable XYZ triples (capacity in POINTS); each knot capacity is
/// in doubles, and reference has 16 writable doubles. Outputs must be mutually
/// non-overlapping. Inputs are copied first; every output is unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_box_fill(
    counts: *const usize,
    degrees: *const usize,
    lo: *const f64,
    hi: *const f64,
    points: *mut f64,
    point_capacity: usize,
    u: *mut f64,
    u_capacity: usize,
    v: *mut f64,
    v_capacity: usize,
    w: *mut f64,
    w_capacity: usize,
    reference: *mut f64,
) -> i32 {
    ffi_status(|| {
        if !aligned(counts)
            || !aligned(degrees)
            || !aligned(lo)
            || !aligned(hi)
            || !aligned(points)
            || !aligned(u)
            || !aligned(v)
            || !aligned(w)
            || !aligned(reference)
        {
            return Err(Error::InvalidBuffer);
        }
        let counts = unsafe { counts.cast::<[usize; 3]>().read() };
        let degrees = unsafe { degrees.cast::<[usize; 3]>().read() };
        let lo = unsafe { lo.cast::<[f64; 3]>().read() };
        let hi = unsafe { hi.cast::<[f64; 3]>().read() };
        let total = box_dimensions(counts, degrees)?;
        let capacity = [u_capacity, v_capacity, w_capacity];
        if point_capacity < total || (0..3).any(|a| capacity[a] < counts[a] + degrees[a] + 1) {
            return Err(Error::Capacity);
        }
        let (cage, frame) = Cage::r#box(counts, degrees, lo, hi)?;
        let outputs = [u, v, w];
        unsafe {
            std::ptr::copy_nonoverlapping(cage.points.as_ptr().cast::<f64>(), points, total * 3);
            for a in 0..3 {
                std::ptr::copy_nonoverlapping(
                    cage.knots[a].as_ptr(),
                    outputs[a],
                    cage.knots[a].len(),
                );
            }
            std::ptr::copy_nonoverlapping(frame.as_ptr(), reference, 16);
        }
        Ok(0)
    })
}
/// # Safety
/// Pointers must refer to live, aligned arrays of their declared sizes. Array
/// validity/provenance cannot be proved by Rust. Descriptor data is copied;
/// callers retain their arrays. `out` must be writable and may not overlap the
/// descriptor. Output is unchanged on error. The returned handle belongs to
/// Rust and must be destroyed exactly once after all concurrent calls finish.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_create(
    d: *const Om9CageDescriptor,
    out: *mut *mut CageHandle,
) -> i32 {
    unsafe { om9_cage_create_placed(d, IDENTITY.as_ptr(), out) }
}
/// # Safety
/// Same descriptor/output/ownership contract as `om9_cage_create`. Descriptor
/// points are LOCAL cage XYZ; pose points to 16 readable aligned doubles in
/// row-major order mapping cage-local XYZ to WORLD. The pose is copied and
/// validated as finite, affine and invertible. Descriptor layout is unchanged;
/// world_to_parameter remains the original WORLD reference-to-normalized frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_create_placed(
    d: *const Om9CageDescriptor,
    pose: *const f64,
    out: *mut *mut CageHandle,
) -> i32 {
    ffi_status(|| {
        if !aligned(d) || !aligned(out) || !aligned(pose) {
            return Err(Error::InvalidBuffer);
        }
        let pose = unsafe { pose.cast::<[f64; 16]>().read() };
        validate_frame(&pose)?;
        // SAFETY: caller supplies live aligned descriptor/output storage.
        let d = unsafe { &*d };
        let mut total = 1usize;
        for a in 0..3 {
            if !(1..=16).contains(&d.degrees[a])
                || d.counts[a] <= d.degrees[a]
                || d.counts[a] > MAX_POINTS
            {
                return Err(Error::InvalidCage);
            }
            total = total
                .checked_mul(d.counts[a])
                .filter(|v| *v <= MAX_POINTS)
                .ok_or(Error::Capacity)?;
            if d.knot_lengths[a] != d.counts[a] + d.degrees[a] + 1 || !aligned(d.knots[a]) {
                return Err(Error::InvalidBuffer);
            }
        }
        if d.point_count != total
            || d.weight_count != total
            || !aligned(d.points)
            || !aligned(d.weights)
        {
            return Err(Error::InvalidBuffer);
        }
        let knots = std::array::from_fn(|a| {
            unsafe { std::slice::from_raw_parts(d.knots[a], d.knot_lengths[a]) }.to_vec()
        });
        let points = unsafe { std::slice::from_raw_parts(d.points, total * 3) }
            .chunks_exact(3)
            .map(|p| [p[0], p[1], p[2]])
            .collect();
        let weights = unsafe { std::slice::from_raw_parts(d.weights, total) }.to_vec();
        let region = match d.region {
            0 => Region::Global,
            1 => Region::Local {
                min: d.local_min,
                max: d.local_max,
                falloff: d.falloff,
            },
            _ => return Err(Error::InvalidRegion),
        };
        let deformation = Deformation::with_pose(
            Cage {
                counts: d.counts,
                degrees: d.degrees,
                knots,
                points,
                weights,
            },
            d.world_to_parameter,
            region,
            pose,
        )?;
        let handle = Box::into_raw(Box::new(CageHandle { deformation }));
        unsafe { out.write(handle) };
        Ok(0)
    })
}
/// # Safety
/// A nonnull handle must come from either cage constructor, still be live, and have no
/// concurrent users. It must not have been destroyed before. Null is allowed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_destroy(h: *mut CageHandle) {
    if !h.is_null() {
        let _ = std::panic::catch_unwind(|| {
            drop(unsafe { Box::from_raw(h) });
        });
    }
}
/// # Safety
/// Handle must be live from either cage constructor. Nonempty input/output must be
/// aligned, readable/writable arrays of count/capacity XYZ triples. Inputs may
/// overlap outputs. Callers synchronize writes and handle destruction; shared
/// immutable handles are safe for concurrent independent output arrays.
/// No output is written unless all points succeed. At most 1,000,000 points.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_deform(
    h: *const CageHandle,
    input: *const f64,
    count: usize,
    out: *mut f64,
    capacity: usize,
) -> i32 {
    unsafe { batch(h, input, count, out, capacity, 0) }
}
/// # Safety
/// Same pointer/ownership contract as `om9_cage_deform`. Outputs normalized
/// CURRENT cage parameters. Non-affine inverse is bounded and conservatively
/// checked, with no exhaustive uniqueness guarantee or outside solver.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_inverse(
    h: *const CageHandle,
    input: *const f64,
    count: usize,
    out: *mut f64,
    capacity: usize,
) -> i32 {
    unsafe { batch(h, input, count, out, capacity, 1) }
}
/// # Safety
/// Same contract as `om9_cage_deform`; input is normalized UVW, output world XYZ.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_evaluate(
    h: *const CageHandle,
    input: *const f64,
    count: usize,
    out: *mut f64,
    capacity: usize,
) -> i32 {
    unsafe { batch(h, input, count, out, capacity, 2) }
}
/// # Safety
/// Handle must be live from either cage constructor. Source/UVW are readable aligned
/// arrays of count XYZ/UVW triples; out is a writable aligned array of capacity
/// XYZ triples. Inputs may overlap out. Both inputs must be finite, including
/// parameters whose Local influence is zero. Callers synchronize writes and
/// destruction. Results use captured world points for Local falloff and cached
/// normalized parameters for evaluation, independent of the reference frame.
/// No output is written unless the entire batch succeeds.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_apply(
    h: *const CageHandle,
    source: *const f64,
    uvw: *const f64,
    count: usize,
    out: *mut f64,
    capacity: usize,
) -> i32 {
    ffi_status(|| {
        if !aligned(h) {
            return Err(Error::InvalidBuffer);
        }
        if count > MAX_POINTS || capacity < count {
            return Err(Error::Capacity);
        }
        if count == 0 {
            return Ok(0);
        }
        if !aligned(source) || !aligned(uvw) || !aligned(out) {
            return Err(Error::InvalidBuffer);
        }
        let d = unsafe { &(*h).deformation };
        let source = unsafe { std::slice::from_raw_parts(source, count * 3) };
        let uvw = unsafe { std::slice::from_raw_parts(uvw, count * 3) };
        if source.iter().chain(uvw).any(|v| !v.is_finite()) {
            return Err(Error::NonFinite);
        }
        let binding = CageBinding {
            parameters: uvw.chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect(),
            original_points: source.chunks_exact(3).map(|p| [p[0], p[1], p[2]]).collect(),
        };
        let result = binding.apply(d)?;
        unsafe { std::ptr::copy_nonoverlapping(result.as_ptr().cast::<f64>(), out, count * 3) };
        Ok(0)
    })
}
unsafe fn batch(
    h: *const CageHandle,
    input: *const f64,
    count: usize,
    out: *mut f64,
    capacity: usize,
    operation: u32,
) -> i32 {
    ffi_status(|| {
        if !aligned(h) {
            return Err(Error::InvalidBuffer);
        }
        if count > MAX_POINTS || capacity < count {
            return Err(Error::Capacity);
        }
        if count == 0 {
            return Ok(0);
        }
        if !aligned(input) || !aligned(out) {
            return Err(Error::InvalidBuffer);
        }
        let d = unsafe { &(*h).deformation };
        let input = unsafe { std::slice::from_raw_parts(input, count * 3) };
        let mut result = Vec::with_capacity(count);
        for p in input.chunks_exact(3) {
            let p = [p[0], p[1], p[2]];
            result.push(match operation {
                0 => d.deform(p),
                1 => d.inverse_point(p),
                _ => d.evaluate_normalized(p),
            }?);
        }
        // Input borrow ends before mutation. Rust owns the independent output
        // staging buffer; copying supports caller-side in-place operation.
        unsafe { std::ptr::copy_nonoverlapping(result.as_ptr().cast::<f64>(), out, count * 3) };
        Ok(0)
    })
}
/// # Safety
/// Handle must be live; out must point to 16 writable aligned doubles. Returns
/// 1 for a non-affine cage, leaving out unchanged; 0 writes row-major transform.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cage_affine(h: *const CageHandle, out: *mut f64) -> i32 {
    ffi_status(|| {
        if !aligned(h) || !aligned(out) {
            return Err(Error::InvalidBuffer);
        }
        match unsafe { &(*h).deformation }.exact_affine_transform()? {
            Some(m) => {
                unsafe { std::ptr::copy_nonoverlapping(m.as_ptr(), out, 16) };
                Ok(0)
            }
            None => Ok(1),
        }
    })
}
