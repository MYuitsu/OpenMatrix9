// SPDX-License-Identifier: LGPL-2.1-or-later
//! NURBS cage deformation kernel (M08 jewelry modeling, wave W9).
//!
//! Ported from the legacy bridge (`rust/src/cage.rs`): the portable half is the owned cage
//! record, its admission rules (clamped ends, non-negative multiplicity, positive weights)
//! and the rational tensor evaluation that maps a cage parameter to a world point. The legacy
//! C entry points, the capture/binding lifecycle and the parameter inversion stay behind —
//! inversion needs the native solver, so it is refused by name rather than approximated.
use crate::{
    Error, Result,
    budget::Limits,
    curve_build::basis,
    geometry::{Point3, finite},
};

/// The owning scale of a cage trigger region. Only the region record is portable here; the
/// falloff weighting lives in the native adapter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Region {
    /// Every point in the document is influenced.
    Global,
    /// Only points inside the box are influenced, with a falloff distance outside it.
    Local {
        min: Point3,
        max: Point3,
        falloff: f64,
    },
}

impl Region {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Global => Ok(()),
            Self::Local { min, max, falloff } => {
                finite(*min)?;
                finite(*max)?;
                if !falloff.is_finite() || *falloff < 0.0 {
                    return Err(Error::Invalid("cage region falloff"));
                }
                if (0..3).any(|axis| min[axis] > max[axis]) {
                    return Err(Error::Invalid("cage region box"));
                }
                Ok(())
            }
        }
    }
}

/// An owned rational trivariate cage. Control points are ordered **U fastest**:
/// `index = ((w * counts[1]) + v) * counts[0] + u`.
#[derive(Debug, Clone, PartialEq)]
pub struct Cage {
    pub counts: [usize; 3],
    pub degrees: [usize; 3],
    pub knots: [Vec<f64>; 3],
    pub points: Vec<Point3>,
    pub weights: Vec<f64>,
}

impl Cage {
    /// Admit a cage. The rules are the legacy ones: full clamped knot vectors, no interior
    /// knot equal to an end, multiplicity at most `degree + 1`, positive weights, and a point
    /// grid that matches the counts.
    pub fn validate(&self, limits: &Limits) -> Result<()> {
        let mut total = 1usize;
        for axis in 0..3 {
            let count = self.counts[axis];
            let degree = self.degrees[axis];
            if degree == 0 || degree > limits.max_degree || count <= degree {
                return Err(Error::Invalid("cage degree/count"));
            }
            let knots = &self.knots[axis];
            if knots.len() != count + degree + 1
                || knots.iter().any(|value| !value.is_finite())
                || knots.windows(2).any(|pair| pair[0] > pair[1])
            {
                return Err(Error::Invalid("cage knots"));
            }
            let low = knots[degree];
            let high = knots[count];
            if low >= high
                || !(high - low).is_finite()
                // Clamped ends: the first and last `degree + 1` knots repeat the domain ends.
                || knots[..=degree].iter().any(|value| *value != low)
                || knots[count..].iter().any(|value| *value != high)
                // A clamped vector must have a real interior span at both ends.
                || knots[degree + 1] == low
                || knots[count - 1] == high
            {
                return Err(Error::Invalid("cage knots"));
            }
            let mut run = 1;
            for pair in knots.windows(2) {
                run = if pair[0] == pair[1] { run + 1 } else { 1 };
                if run > degree + 1 {
                    return Err(Error::Invalid("cage knot multiplicity"));
                }
            }
            total = total
                .checked_mul(count)
                .ok_or(Error::Overflow("cage control grid"))?;
        }
        if self.points.len() != total || self.weights.len() != total {
            return Err(Error::Invalid("cage control grid"));
        }
        if total as u64 > limits.max_points || total as u64 > limits.max_objects {
            return Err(Error::Budget("cage control grid"));
        }
        for point in &self.points {
            finite(*point)?;
        }
        if self
            .weights
            .iter()
            .any(|weight| !weight.is_finite() || *weight <= 0.0)
        {
            return Err(Error::Invalid("cage weights"));
        }
        Ok(())
    }

    /// The world point the cage maps parameter `[u, v, w]` to. Uniform weights make this an
    /// affine map of the parameter cube, which is what the identity cage test asserts.
    pub fn evaluate(&self, parameters: Point3, limits: &Limits) -> Result<Point3> {
        self.validate(limits)?;
        finite(parameters)?;
        let mut spans = [0usize; 3];
        let mut values = [Vec::new(), Vec::new(), Vec::new()];
        for axis in 0..3 {
            let count = self.counts[axis];
            let degree = self.degrees[axis];
            let knots = &self.knots[axis];
            let value = parameters[axis];
            if value < knots[degree] || value > knots[count] {
                return Err(Error::Invalid("cage parameter"));
            }
            // The closed domain end belongs to the last non-empty span, matching the curve
            // evaluator's half-open rule.
            let span = if value == knots[count] {
                count - 1
            } else {
                let mut found = None;
                for index in degree..count {
                    if value >= knots[index] && value < knots[index + 1] {
                        found = Some(index);
                        break;
                    }
                }
                found.ok_or(Error::Invalid("cage parameter"))?
            };
            spans[axis] = span;
            let mut row = Vec::new();
            for index in span - degree..=span {
                row.push(basis(degree, knots, index, value)?);
            }
            values[axis] = row;
        }
        let mut numerator = [0.0; 3];
        let mut denominator = 0.0;
        for (k, weight_k) in values[2].iter().enumerate() {
            for (j, weight_j) in values[1].iter().enumerate() {
                for (i, weight_i) in values[0].iter().enumerate() {
                    let coefficient = weight_i * weight_j * weight_k;
                    if coefficient == 0.0 {
                        continue;
                    }
                    let index = ((spans[2] - self.degrees[2] + k) * self.counts[1] + spans[1]
                        - self.degrees[1]
                        + j)
                        * self.counts[0]
                        + spans[0]
                        - self.degrees[0]
                        + i;
                    let weight = self.weights[index];
                    denominator += coefficient * weight;
                    let point = self.points[index];
                    for (axis, value) in numerator.iter_mut().enumerate() {
                        *value += coefficient * weight * point[axis];
                    }
                }
            }
        }
        if !denominator.is_finite() || denominator <= 0.0 {
            return Err(Error::Invalid("cage homogeneous denominator"));
        }
        let point = [
            numerator[0] / denominator,
            numerator[1] / denominator,
            numerator[2] / denominator,
        ];
        finite(point)?;
        Ok(point)
    }
}

/// Capture the parameters of a point grid once, so a later cage edit deforms the captured
/// originals instead of composing errors. Only the capture is portable: recovering the
/// parameters needs the native inversion.
pub fn capture_parameters(_source: &[Point3]) -> Result<()> {
    Err(Error::Unsupported("cage parameter inversion"))
}
