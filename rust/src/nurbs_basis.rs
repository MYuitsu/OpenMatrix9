//! Shared, allocation-free validation of native distinct-knot NURBS bases.
//!
//! Limits are an explicit OM9 caller policy, not the generated spline limits or
//! a guarantee that a backend supports every structurally valid basis. Native
//! adapters must still enforce their integer ranges and kernel validity.

use std::fmt;

/// `max_poles` bounds the total surface grid; `max_knots` bounds each axis.
/// `max_bytes` bounds logical input payload, excluding containers, native copies,
/// allocator overhead and the aggregate worker budget.
#[derive(Clone, Copy, Debug)]
pub struct BasisPolicy {
    pub max_degree: usize,
    pub max_poles: usize,
    pub max_knots: usize,
    pub max_bytes: usize,
    pub max_coordinate: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct Axis<'a> {
    pub degree: usize,
    pub knots: &'a [f64],
    pub multiplicities: &'a [usize],
    pub periodic: bool,
    pub domain: [f64; 2],
}
#[derive(Clone, Copy, Debug)]
pub struct CurveBasis<'a> {
    pub poles: &'a [[f64; 3]],
    pub weights: Option<&'a [f64]>,
    pub axis: Axis<'a>,
}
#[derive(Clone, Copy, Debug)]
pub struct SurfaceBasis<'a> {
    /// U-major grid: pole and weight `(u, v)` have index `u * v_count + v`.
    pub poles: &'a [[f64; 3]],
    pub weights: Option<&'a [f64]>,
    pub u_count: usize,
    pub v_count: usize,
    pub axes: [Axis<'a>; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BasisError {
    InvalidPolicy,
    Degree,
    PoleCount,
    PoleLimit,
    Knots,
    KnotLimit,
    GridDimensions,
    GridCountMismatch,
    CountOverflow,
    ByteLimit,
    Poles,
    Weights,
    Multiplicity,
    PeriodicCardinality,
    Cardinality,
    Domain,
}
impl BasisPolicy {
    pub fn validate(self) -> Result<(), BasisError> {
        if self.max_degree == 0
            || self.max_degree == usize::MAX
            || self.max_poles < 2
            || self.max_knots < 2
            || self.max_bytes == 0
            || !self.max_coordinate.is_finite()
            || self.max_coordinate <= 0.
        {
            return Err(BasisError::InvalidPolicy);
        }
        Ok(())
    }

    /// May be called before native extraction or temporary buffer allocation.
    pub fn validate_degree(self, degree: usize) -> Result<(), BasisError> {
        self.validate()?;
        if degree == 0 || degree > self.max_degree {
            return Err(BasisError::Degree);
        }
        Ok(())
    }
}

impl fmt::Display for BasisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPolicy => "Invalid NURBS basis policy",
            Self::Degree => "Degree must be positive and within the basis policy",
            Self::PoleCount => "A basis axis requires at least two poles",
            Self::PoleLimit => "Pole count exceeds the basis policy",
            Self::Knots => "Knots must be finite, strictly increasing, and match multiplicities",
            Self::KnotLimit => "Knot count exceeds the basis policy",
            Self::GridDimensions => "A surface grid requires at least two poles in each direction",
            Self::GridCountMismatch => "Surface grid dimensions do not match its pole count",
            Self::CountOverflow => "NURBS count or byte arithmetic overflow",
            Self::ByteLimit => "Basis payload exceeds the byte policy or addressable slice limit",
            Self::Poles => "Poles must be finite and within the coordinate policy",
            Self::Weights => "Weights must match poles, be positive, finite, and not overflow homogeneous coordinates",
            Self::Multiplicity => "Invalid knot multiplicity",
            Self::PeriodicCardinality => "Periodic seam multiplicities or pole count do not match",
            Self::Cardinality => "Pole count does not match degree and knot multiplicities",
            Self::Domain => "Domain must be finite, increasing, and inside the active knot interval",
        })
    }
}

impl std::error::Error for BasisError {}

fn payload_bytes(
    pole_count: usize,
    weight_count: Option<usize>,
    axes: &[(usize, usize)],
    policy: BasisPolicy,
) -> Result<usize, BasisError> {
    if pole_count < 2 {
        return Err(BasisError::PoleCount);
    }
    if pole_count > policy.max_poles {
        return Err(BasisError::PoleLimit);
    }
    if weight_count.is_some_and(|count| count != pole_count) {
        return Err(BasisError::Weights);
    }
    let mut bytes = pole_count
        .checked_mul(std::mem::size_of::<[f64; 3]>())
        .ok_or(BasisError::CountOverflow)?;
    let mut add = |count: usize, size: usize| -> Result<(), BasisError> {
        bytes = count
            .checked_mul(size)
            .and_then(|n| bytes.checked_add(n))
            .ok_or(BasisError::CountOverflow)?;
        Ok(())
    };
    add(weight_count.unwrap_or(0), std::mem::size_of::<f64>())?;
    for &(knots, multiplicities) in axes {
        if knots < 2 || knots != multiplicities {
            return Err(BasisError::Knots);
        }
        if knots > policy.max_knots {
            return Err(BasisError::KnotLimit);
        }
        add(knots, std::mem::size_of::<f64>())?;
        add(multiplicities, std::mem::size_of::<usize>())?;
    }
    if bytes > policy.max_bytes || bytes > isize::MAX as usize {
        return Err(BasisError::ByteLimit);
    }
    Ok(bytes)
}

/// Count-only guard before allocation, slicing or value inspection.
/// Returns logical payload bytes; no input buffer is read or allocated.
pub fn preflight_curve_counts(
    pole_count: usize,
    weight_count: Option<usize>,
    knot_count: usize,
    multiplicity_count: usize,
    policy: BasisPolicy,
) -> Result<usize, BasisError> {
    policy.validate()?;
    payload_bytes(
        pole_count,
        weight_count,
        &[(knot_count, multiplicity_count)],
        policy,
    )
}

/// Surface product and count-only guard. Dimensions and flattened slices must
/// agree exactly; callers cannot silently truncate a native U-major grid.
pub fn preflight_surface_counts(
    u_count: usize,
    v_count: usize,
    pole_count: usize,
    weight_count: Option<usize>,
    knot_counts: [usize; 2],
    multiplicity_counts: [usize; 2],
    policy: BasisPolicy,
) -> Result<usize, BasisError> {
    policy.validate()?;
    let total = u_count
        .checked_mul(v_count)
        .ok_or(BasisError::CountOverflow)?;
    if u_count < 2 || v_count < 2 {
        return Err(BasisError::GridDimensions);
    }
    if total != pole_count {
        return Err(BasisError::GridCountMismatch);
    }
    payload_bytes(
        pole_count,
        weight_count,
        &[
            (knot_counts[0], multiplicity_counts[0]),
            (knot_counts[1], multiplicity_counts[1]),
        ],
        policy,
    )
}

fn validate_poles(
    poles: &[[f64; 3]],
    weights: Option<&[f64]>,
    policy: BasisPolicy,
) -> Result<(), BasisError> {
    if poles
        .iter()
        .flatten()
        .any(|v| !v.is_finite() || v.abs() > policy.max_coordinate)
    {
        return Err(BasisError::Poles);
    }
    if let Some(weights) = weights
        && weights
            .iter()
            .zip(poles)
            .any(|(&w, p)| !w.is_finite() || w <= 0. || p.iter().any(|v| !(v * w).is_finite()))
    {
        return Err(BasisError::Weights);
    }
    Ok(())
}

impl Axis<'_> {
    /// Structural checks extracted from the generated `spline::Basis` gate.
    /// Private entry: all degree, count and byte checks have already succeeded.
    fn validate_structure(&self, pole_count: usize) -> Result<(), BasisError> {
        if self.knots.iter().any(|u| !u.is_finite())
            || self
                .knots
                .windows(2)
                .any(|u| u[0] >= u[1] || !(u[1] - u[0]).is_finite())
        {
            return Err(BasisError::Knots);
        }
        let last = self.knots.len() - 1;
        let total = self
            .multiplicities
            .iter()
            .enumerate()
            .try_fold(0usize, |sum, (i, &m)| {
                let limit = self
                    .degree
                    .checked_add(usize::from(!self.periodic && (i == 0 || i == last)))
                    .ok_or(BasisError::CountOverflow)?;
                if m == 0 || m > limit {
                    return Err(BasisError::Multiplicity);
                }
                sum.checked_add(m).ok_or(BasisError::CountOverflow)
            })?;
        let (minimum, maximum) = if self.periodic {
            if self.multiplicities[0] != self.multiplicities[last]
                || total.checked_sub(self.multiplicities[last]) != Some(pole_count)
            {
                return Err(BasisError::PeriodicCardinality);
            }
            (self.knots[0], self.knots[last])
        } else {
            let expected = pole_count
                .checked_add(self.degree)
                .and_then(|n| n.checked_add(1))
                .ok_or(BasisError::CountOverflow)?;
            if total != expected {
                return Err(BasisError::Cardinality);
            }
            // Prefixes cannot overflow after the checked sum above. Do not
            // allocate or expand a potentially high-multiplicity knot vector.
            let at = |index: usize| {
                let mut end = 0;
                self.knots
                    .iter()
                    .zip(self.multiplicities)
                    .find_map(|(&u, &m)| {
                        end += m;
                        (index < end).then_some(u)
                    })
                    .unwrap_or(f64::NAN)
            };
            (at(self.degree), at(pole_count))
        };
        if self.domain.iter().any(|u| !u.is_finite())
            || self.domain[0] >= self.domain[1]
            || !(self.domain[1] - self.domain[0]).is_finite()
            || self.domain[0] < minimum
            || self.domain[1] > maximum
        {
            return Err(BasisError::Domain);
        }
        Ok(())
    }
}

impl CurveBasis<'_> {
    pub fn validate(&self, policy: BasisPolicy) -> Result<(), BasisError> {
        policy.validate_degree(self.axis.degree)?;
        preflight_curve_counts(
            self.poles.len(),
            self.weights.map(<[f64]>::len),
            self.axis.knots.len(),
            self.axis.multiplicities.len(),
            policy,
        )?;
        validate_poles(self.poles, self.weights, policy)?;
        self.axis.validate_structure(self.poles.len())
    }
}
impl SurfaceBasis<'_> {
    pub fn validate(&self, policy: BasisPolicy) -> Result<(), BasisError> {
        for axis in self.axes {
            policy.validate_degree(axis.degree)?;
        }
        preflight_surface_counts(
            self.u_count,
            self.v_count,
            self.poles.len(),
            self.weights.map(<[f64]>::len),
            self.axes.map(|a| a.knots.len()),
            self.axes.map(|a| a.multiplicities.len()),
            policy,
        )?;
        validate_poles(self.poles, self.weights, policy)?;
        self.axes[0].validate_structure(self.u_count)?;
        self.axes[1].validate_structure(self.v_count)
    }
}
