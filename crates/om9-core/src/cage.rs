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

/// The eight corners of a box cage, in the cage's own point order `index = (w * 2 + v) * 2 + u`.
pub fn box_corners(min: Point3, max: Point3) -> [Point3; 8] {
    let mut corners = [[0.0; 3]; 8];
    for (index, corner) in corners.iter_mut().enumerate() {
        let u = index & 1;
        let v = (index >> 1) & 1;
        let w = (index >> 2) & 1;
        for axis in 0..3 {
            corner[axis] = if [u, v, w][axis] == 1 {
                max[axis]
            } else {
                min[axis]
            };
        }
    }
    corners
}

/// A trilinear cage over a box: degree one, two poles per axis, uniform weights, unit domain.
pub fn box_cage(min: Point3, max: Point3, limits: &Limits) -> Result<Cage> {
    finite(min)?;
    finite(max)?;
    if (0..3).any(|axis| min[axis] > max[axis]) {
        return Err(Error::Invalid("cage box"));
    }
    let cage = Cage {
        counts: [2, 2, 2],
        degrees: [1, 1, 1],
        knots: [
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        ],
        points: box_corners(min, max).to_vec(),
        weights: vec![1.0; 8],
    };
    cage.validate(limits)?;
    Ok(cage)
}

/// A trilinear cage through eight corners in the cage point order. A cage edit moves one of
/// these, so the live cage is rebuilt from the mesh it presents.
pub fn cage_from_corners(corners: [Point3; 8], limits: &Limits) -> Result<Cage> {
    let mut points = Vec::with_capacity(8);
    for corner in corners {
        finite(corner)?;
        points.push(corner);
    }
    let cage = Cage {
        counts: [2, 2, 2],
        degrees: [1, 1, 1],
        knots: [
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        ],
        points,
        weights: vec![1.0; 8],
    };
    cage.validate(limits)?;
    Ok(cage)
}

/// The corner of one control cell, `corner` in `0..8`, in the cage point order.
pub fn corner_index(corner: usize) -> Result<(usize, usize, usize)> {
    if corner >= 8 {
        return Err(Error::Invalid("cage corner"));
    }
    Ok((corner & 1, (corner >> 1) & 1, (corner >> 2) & 1))
}

fn is_box(cage: &Cage) -> bool {
    cage.counts == [2, 2, 2]
        && cage.degrees == [1, 1, 1]
        && cage.weights.iter().all(|weight| *weight == 1.0)
}

impl Cage {
    /// The eight corners of a box cage, in the cage point order.
    pub fn corners(&self) -> Result<[Point3; 8]> {
        if !is_box(self) {
            return Err(Error::Unsupported("corners of a non-box cage"));
        }
        let mut corners = [[0.0; 3]; 8];
        corners.copy_from_slice(&self.points);
        Ok(corners)
    }

    /// The same box cage with one corner moved. The corner order is the cage point order, so a
    /// cage edit is one index and one offset.
    pub fn with_corner(&self, corner: usize, offset: Point3) -> Result<Self> {
        corner_index(corner)?;
        finite(offset)?;
        if !is_box(self) {
            return Err(Error::Unsupported("corner edit of a non-box cage"));
        }
        let mut moved = self.clone();
        moved.points[corner] = [
            moved.points[corner][0] + offset[0],
            moved.points[corner][1] + offset[1],
            moved.points[corner][2] + offset[2],
        ];
        Ok(moved)
    }

    /// The parameter of `point` in a box cage: the trilinear inverse `S(u, v, w) = point`,
    /// solved with Newton steps because a moved cage is a hexahedron, not a box. Only box cages
    /// are solvable in the portable kernel; anything else is refused by name.
    pub fn invert(&self, point: Point3, limits: &Limits) -> Result<Point3> {
        self.validate(limits)?;
        finite(point)?;
        if !is_box(self) {
            return Err(Error::Unsupported("cage parameter inversion"));
        }
        let scale = self
            .points
            .iter()
            .flat_map(|corner| corner.iter())
            .fold(0.0f64, |largest, value| largest.max(value.abs()))
            .max(1.0);
        let mut parameters = [0.5; 3];
        for _ in 0..48 {
            let (value, jacobian) = self.trilinear(parameters)?;
            let residual = [
                value[0] - point[0],
                value[1] - point[1],
                value[2] - point[2],
            ];
            let error = residual
                .iter()
                .map(|value| value * value)
                .sum::<f64>()
                .sqrt();
            if error <= 1.0e-12 * scale {
                return Ok(parameters);
            }
            let step = solve3(jacobian, residual).ok_or(Error::Invalid("cage inversion"))?;
            let mut moved = [0.0; 3];
            for axis in 0..3 {
                moved[axis] = parameters[axis] - step[axis];
            }
            if moved.iter().any(|value| !value.is_finite()) {
                return Err(Error::Invalid("cage inversion"));
            }
            parameters = moved;
        }
        Err(Error::Invalid("cage inversion did not converge"))
    }

    /// The trilinear map and its Jacobian at one parameter triple.
    fn trilinear(&self, parameters: Point3) -> Result<(Point3, [[f64; 3]; 3])> {
        let mut value = [0.0; 3];
        let mut jacobian = [[0.0; 3]; 3];
        for index in 0..8 {
            let u = index & 1;
            let v = (index >> 1) & 1;
            let w = (index >> 2) & 1;
            let bu = if u == 1 {
                parameters[0]
            } else {
                1.0 - parameters[0]
            };
            let bv = if v == 1 {
                parameters[1]
            } else {
                1.0 - parameters[1]
            };
            let bw = if w == 1 {
                parameters[2]
            } else {
                1.0 - parameters[2]
            };
            let du = if u == 1 { 1.0 } else { -1.0 };
            let dv = if v == 1 { 1.0 } else { -1.0 };
            let dw = if w == 1 { 1.0 } else { -1.0 };
            let point = self.points[index];
            for axis in 0..3 {
                value[axis] += bu * bv * bw * point[axis];
                jacobian[axis][0] += du * bv * bw * point[axis];
                jacobian[axis][1] += bu * dv * bw * point[axis];
                jacobian[axis][2] += bu * bv * dw * point[axis];
            }
        }
        Ok((value, jacobian))
    }
}

/// Solve `matrix * step = residual` by Cramer's rule; `None` when the matrix is singular.
fn solve3(matrix: [[f64; 3]; 3], residual: [f64; 3]) -> Option<[f64; 3]> {
    let determinant = determinant3(matrix);
    if !determinant.is_finite() || determinant.abs() <= 1.0e-18 {
        return None;
    }
    let mut step = [0.0; 3];
    for column in 0..3 {
        let mut replaced = matrix;
        for row in 0..3 {
            replaced[row][column] = residual[row];
        }
        step[column] = determinant3(replaced) / determinant;
    }
    Some(step)
}

fn determinant3(matrix: [[f64; 3]; 3]) -> f64 {
    matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1])
        - matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0])
        + matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0])
}

/// The displacement a cage edit applies at `point`: the point keeps its parameter in `before`
/// and moves to the same parameter of `after`.
pub fn displacement(before: &Cage, after: &Cage, point: Point3, limits: &Limits) -> Result<Point3> {
    let parameters = before.invert(point, limits)?;
    let landing = after.evaluate(parameters, limits)?;
    let origin = before.evaluate(parameters, limits)?;
    Ok([
        point[0] + landing[0] - origin[0],
        point[1] + landing[1] - origin[1],
        point[2] + landing[2] - origin[2],
    ])
}

/// The visible box of a cage: the eight corners as one mesh with six quad faces, so a cage is an
/// ordinary document object that moves and undoes like any other.
pub fn box_mesh(corners: [Point3; 8]) -> crate::geometry::Mesh {
    const FACES: [[u32; 4]; 6] = [
        [0, 2, 3, 1],
        [4, 5, 7, 6],
        [0, 1, 5, 4],
        [2, 6, 7, 3],
        [0, 4, 6, 2],
        [1, 3, 7, 5],
    ];
    crate::geometry::Mesh {
        vertices: corners.to_vec(),
        faces: FACES
            .iter()
            .map(|face| face.to_vec())
            .collect(),
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    }
}

/// The corners of a cage mesh written by [`box_mesh`]: eight vertices in the cage point order.
pub fn mesh_corners(mesh: &crate::geometry::Mesh) -> Result<[Point3; 8]> {
    if mesh.vertices.len() != 8 {
        return Err(Error::Invalid("cage mesh needs eight corners"));
    }
    let mut corners = [[0.0; 3]; 8];
    corners.copy_from_slice(&mesh.vertices);
    Ok(corners)
}

/// Capture the parameters of a point grid once, so a later cage edit deforms the captured
/// originals instead of composing errors.
pub fn capture_parameters(cage: &Cage, points: &[Point3], limits: &Limits) -> Result<Vec<Point3>> {
    let mut parameters = Vec::with_capacity(points.len());
    for point in points {
        parameters.push(cage.invert(*point, limits)?);
    }
    Ok(parameters)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        Limits::default()
    }

    #[test]
    fn a_unit_box_cage_inverts_its_own_parameters() {
        let cage = box_cage([0.0; 3], [10.0; 3], &limits()).expect("box cage");
        for parameters in [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.25, 0.5, 0.75]] {
            let point = cage.evaluate(parameters, &limits()).expect("evaluate");
            let found = cage.invert(point, &limits()).expect("invert");
            for axis in 0..3 {
                assert!((found[axis] - parameters[axis]).abs() < 1.0e-9, "{found:?}");
            }
        }
        let middle = cage.invert([5.0, 5.0, 5.0], &limits()).expect("centre");
        assert!((middle[0] - 0.5).abs() < 1.0e-9);
    }

    #[test]
    fn moving_a_corner_carries_the_interior_toward_it() {
        let before = box_cage([0.0; 3], [10.0; 3], &limits()).expect("box cage");
        let after = before
            .with_corner(7, [10.0, 0.0, 0.0])
            .expect("corner edit");
        let moved = displacement(&before, &after, [10.0, 10.0, 10.0], &limits()).expect("shift");
        assert!((moved[0] - 20.0).abs() < 1.0e-9, "{moved:?}");
        assert!((moved[1] - 10.0).abs() < 1.0e-9 && (moved[2] - 10.0).abs() < 1.0e-9);
        // The opposite corner keeps its place, so the field is anchored at rest.
        let held = displacement(&before, &after, [0.0; 3], &limits()).expect("shift");
        assert!((held[0]).abs() < 1.0e-9, "{held:?}");
    }

    #[test]
    fn a_cage_mesh_round_trips_its_corners() {
        let corners = box_corners([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let mesh = box_mesh(corners);
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.faces.len(), 6);
        let found = mesh_corners(&mesh).expect("corners");
        for index in 0..8 {
            for axis in 0..3 {
                assert!((found[index][axis] - corners[index][axis]).abs() < 1.0e-12);
            }
        }
        let cage = box_cage([0.0; 3], [1.0; 3], &limits()).expect("box cage");
        let parameters = capture_parameters(&cage, &[corners[7]], &limits()).expect("capture");
        assert_eq!(parameters.len(), 1);
    }

    #[test]
    fn a_corner_outside_the_range_is_refused() {
        let cage = box_cage([0.0; 3], [1.0; 3], &limits()).expect("box cage");
        assert!(cage.with_corner(8, [0.0; 3]).is_err());
    }
}
