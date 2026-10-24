// SPDX-License-Identifier: LGPL-2.1-or-later
//! Transform and pattern operations (M03 `OM9-L2-01-11`).
//!
//! Safe Rust owns the affine matrix service (translation, rotation, uniform and non-uniform
//! scaling, mirror, inverse, composition), the placement frames (curve samples and surface UV/N),
//! the two/three-point orientation solver, pattern placement generation and the bake/instance
//! policy. C++ is limited to the OCCT transform adapter in `transform_bridge.cpp`.
//!
//! A transform is applied to an owned [`Brep`] and returns a new owned [`Brep`]; the input is
//! never mutated. A pattern in instance mode stores one definition plus placements (no geometry
//! duplication); baking creates one independent body per placement.
//!
//! Units are millimetres, angles are radians.
use crate::{
    brep::{snapshot_budget, snapshot_via},
    creation::{CreationError, PathFrame, polyline_frames},
};
use om9_core::{
    brep::Brep,
    budget::Limits,
    geometry::NurbsSurface,
    geometry::{Point3, Transform, finite},
    surface_kernel,
};
use std::ffi::{c_int, c_void};

/// The typed refusal contract is shared with the creation/editing owners.
pub type TransformError = CreationError;

/// The most occurrences one pattern may place; an unbounded count is a denial of service.
pub const MAX_PATTERN_INSTANCES: usize = 10_000;

type NativeSink = unsafe extern "C" fn(*mut c_void, *const crate::brep::Snapshot) -> c_int;

unsafe extern "C" {
    fn om9_occt_transform_brep(
        source: *const u8,
        length: usize,
        matrix: *const f64,
        deflection: f64,
        max_points: usize,
        max_nodes: usize,
        max_bytes: usize,
        sink: NativeSink,
        context: *mut c_void,
        message: *mut std::ffi::c_char,
        capacity: c_int,
    ) -> c_int;
}

fn map_core(error: om9_core::Error) -> TransformError {
    CreationError::Invalid(error.to_string())
}

fn checked_deflection(deflection_mm: f64) -> Result<f64, TransformError> {
    if !deflection_mm.is_finite() || deflection_mm <= 0.0 {
        return Err(CreationError::Invalid(
            "deflection must be finite and positive".into(),
        ));
    }
    Ok(deflection_mm)
}

fn checked_matrix(matrix: &Transform) -> Result<(), TransformError> {
    matrix.validate().map_err(map_core)
}

fn determinant3(matrix: &Transform) -> f64 {
    let m = matrix.0;
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/* ------------------------- matrix service ------------------------- */

/// Translation by `offset`.
pub fn translation(offset: Point3) -> Result<Transform, TransformError> {
    finite(offset).map_err(map_core)?;
    let matrix = Transform::translation(offset);
    checked_matrix(&matrix)?;
    Ok(matrix)
}

/// Right-handed rotation of `angle` radians about the unit-direction `axis` through `origin`.
pub fn rotation(origin: Point3, axis: Point3, angle: f64) -> Result<Transform, TransformError> {
    om9_core::gumball::rotation_about(origin, axis, angle).map_err(map_core)
}

/// Non-uniform scaling by `factors` about `origin`. A negative factor is a mirror in that axis and
/// is admitted (the determinant sign is reported); a zero factor is singular and refused.
pub fn scaling(origin: Point3, factors: Point3) -> Result<Transform, TransformError> {
    finite(origin).map_err(map_core)?;
    for (value, name) in factors.iter().zip(["x", "y", "z"]) {
        if !value.is_finite() || *value == 0.0 {
            return Err(CreationError::Invalid(format!(
                "scale factor {name} must be finite and non-zero"
            )));
        }
    }
    let mut m = Transform::identity().0;
    for (index, value) in factors.iter().enumerate() {
        m[index][index] = *value;
    }
    // p' = origin + S (p - origin)  =>  translation = origin - S origin
    for (row, line) in m.iter_mut().enumerate().take(3) {
        line[3] = origin[row] - line[row] * origin[row];
    }
    let matrix = Transform(m);
    checked_matrix(&matrix)?;
    Ok(matrix)
}

/// Mirror across the plane through `plane_origin` with normal `plane_normal`.
pub fn mirror(plane_origin: Point3, plane_normal: Point3) -> Result<Transform, TransformError> {
    finite(plane_origin).map_err(map_core)?;
    finite(plane_normal).map_err(map_core)?;
    let length = (plane_normal[0] * plane_normal[0]
        + plane_normal[1] * plane_normal[1]
        + plane_normal[2] * plane_normal[2])
        .sqrt();
    if !length.is_finite() || length <= 1.0e-12 {
        return Err(CreationError::Invalid(
            "mirror plane normal must be finite and non-zero".into(),
        ));
    }
    let n = [
        plane_normal[0] / length,
        plane_normal[1] / length,
        plane_normal[2] / length,
    ];
    let mut m = Transform::identity().0;
    for row in 0..3 {
        for column in 0..3 {
            m[row][column] = if row == column { 1.0 } else { 0.0 } - 2.0 * n[row] * n[column];
        }
        // p' = p - 2 n (n . (p - o)) = (I - 2 n n^T) p + 2 n (n . o)
        let projection = n[0] * plane_origin[0] + n[1] * plane_origin[1] + n[2] * plane_origin[2];
        m[row][3] = 2.0 * n[row] * projection;
    }
    let matrix = Transform(m);
    checked_matrix(&matrix)?;
    Ok(matrix)
}

/// The affine inverse. A singular matrix is refused; the inverse restores every mapped point.
pub fn inverse(matrix: &Transform) -> Result<Transform, TransformError> {
    checked_matrix(matrix)?;
    let m = matrix.0;
    let det = determinant3(matrix);
    if !det.is_finite() || det.abs() <= 1.0e-15 {
        return Err(CreationError::Invalid(
            "matrix is singular and has no inverse".into(),
        ));
    }
    let inv = 1.0 / det;
    let mut out = Transform::identity().0;
    out[0][0] = (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv;
    out[0][1] = (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv;
    out[0][2] = (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv;
    out[1][0] = (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv;
    out[1][1] = (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv;
    out[1][2] = (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv;
    out[2][0] = (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv;
    out[2][1] = (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv;
    out[2][2] = (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv;
    for line in out.iter_mut().take(3) {
        line[3] = -(line[0] * m[0][3] + line[1] * m[1][3] + line[2] * m[2][3]);
    }
    let result = Transform(out);
    checked_matrix(&result)?;
    Ok(result)
}

/// Compose two affine matrices: the result applies `local` first, then `outer`.
pub fn compose(outer: &Transform, local: &Transform) -> Result<Transform, TransformError> {
    outer.compose(*local).map_err(map_core)
}

/// Map one point through the matrix.
pub fn apply(matrix: &Transform, point: Point3) -> Result<Point3, TransformError> {
    matrix.point(point).map_err(map_core)
}

/// The determinant's sign: positive keeps handedness, negative mirrors it.
pub fn handedness(matrix: &Transform) -> Result<f64, TransformError> {
    checked_matrix(matrix)?;
    Ok(determinant3(matrix))
}

/// Apply one admissible affine matrix to a B-Rep, returning a new body. The input is immutable.
pub fn transform_brep(
    brep: &Brep,
    matrix: &Transform,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<Brep, TransformError> {
    let deflection = checked_deflection(deflection_mm)?;
    checked_matrix(matrix)?;
    brep.validate(limits)
        .map_err(|error| CreationError::Invalid(error.to_string()))?;
    let budget = snapshot_budget(limits);
    snapshot_via(deflection, limits, |sink, context, message, capacity| {
        // SAFETY: source, matrix and the callback context live through this synchronous call.
        unsafe {
            om9_occt_transform_brep(
                brep.source().as_ptr(),
                brep.source().len(),
                matrix.0.as_ptr().cast(),
                deflection,
                budget.points,
                budget.nodes,
                budget.bytes,
                sink,
                context,
                message,
                capacity,
            )
        }
    })
    .map_err(TransformError::from)
}

/* ------------------------- placement frames ------------------------- */

/// One right-handed placement frame: origin plus three orthonormal axes (`x × y = z`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacementFrame {
    pub origin: Point3,
    pub x: Point3,
    pub y: Point3,
    pub z: Point3,
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(value: Point3) -> f64 {
    dot(value, value).sqrt()
}

fn unit(value: Point3, what: &str) -> Result<Point3, TransformError> {
    let magnitude = length(value);
    if !magnitude.is_finite() || magnitude <= 1.0e-12 {
        return Err(CreationError::Invalid(format!("{what} is degenerate")));
    }
    Ok([
        value[0] / magnitude,
        value[1] / magnitude,
        value[2] / magnitude,
    ])
}

impl PlacementFrame {
    /// Validate orthonormality and right-handedness.
    pub fn validate(&self) -> Result<(), TransformError> {
        finite(self.origin).map_err(map_core)?;
        for axis in [self.x, self.y, self.z] {
            finite(axis).map_err(map_core)?;
            if (length(axis) - 1.0).abs() > 1.0e-9 {
                return Err(CreationError::Invalid(
                    "frame axis is not unit length".into(),
                ));
            }
        }
        if dot(self.x, self.y).abs() > 1.0e-9
            || dot(self.x, self.z).abs() > 1.0e-9
            || dot(self.y, self.z).abs() > 1.0e-9
        {
            return Err(CreationError::Invalid(
                "frame axes are not orthogonal".into(),
            ));
        }
        if dot(cross(self.x, self.y), self.z) < 1.0 - 1.0e-9 {
            return Err(CreationError::Invalid(
                "frame is not right-handed (x × y must equal z)".into(),
            ));
        }
        Ok(())
    }

    /// The local-to-world matrix of this frame.
    pub fn matrix(&self) -> Transform {
        let mut m = Transform::identity().0;
        for (row, line) in m.iter_mut().enumerate().take(3) {
            line[0] = self.x[row];
            line[1] = self.y[row];
            line[2] = self.z[row];
            line[3] = self.origin[row];
        }
        Transform(m)
    }
}

fn from_path_frame(frame: PathFrame) -> PlacementFrame {
    PlacementFrame {
        origin: frame.origin,
        x: frame.x,
        y: frame.y,
        z: frame.tangent,
    }
}

fn distance(a: Point3, b: Point3) -> f64 {
    length([a[0] - b[0], a[1] - b[1], a[2] - b[2]])
}

/// Resample a polyline by arc length at `count` equal-distance stations.
fn resample_polyline(points: &[Point3], count: usize) -> Result<Vec<Point3>, TransformError> {
    if points.len() < 2 {
        return Err(CreationError::Invalid(
            "frames need a rail with at least two points".into(),
        ));
    }
    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0);
    for index in 1..points.len() {
        let segment = distance(points[index - 1], points[index]);
        if segment <= 1.0e-9 {
            return Err(CreationError::Invalid(
                "rail contains a zero-length segment".into(),
            ));
        }
        lengths.push(lengths[index - 1] + segment);
    }
    let total = lengths[lengths.len() - 1];
    if !total.is_finite() || total <= 0.0 {
        return Err(CreationError::Invalid("rail is degenerate".into()));
    }
    let mut out = Vec::with_capacity(count);
    for index in 0..count {
        let target = total * index as f64 / (count - 1) as f64;
        let mut station = None;
        if target <= 0.0 {
            station = Some(points[0]);
        } else if target >= total {
            station = Some(points[points.len() - 1]);
        } else {
            for segment in 1..points.len() {
                if target <= lengths[segment] {
                    let span = lengths[segment] - lengths[segment - 1];
                    let alpha = if span > 0.0 {
                        (target - lengths[segment - 1]) / span
                    } else {
                        0.0
                    };
                    station = Some([
                        points[segment - 1][0]
                            + alpha * (points[segment][0] - points[segment - 1][0]),
                        points[segment - 1][1]
                            + alpha * (points[segment][1] - points[segment - 1][1]),
                        points[segment - 1][2]
                            + alpha * (points[segment][2] - points[segment - 1][2]),
                    ]);
                    break;
                }
            }
        }
        out.push(station.ok_or_else(|| CreationError::Invalid("rail resample".into()))?);
    }
    Ok(out)
}

/// Arc-length sampled placement frames along a polyline (rotation-minimizing transport).
pub fn frames_along_polyline(
    rail: &[Point3],
    count: usize,
) -> Result<Vec<PlacementFrame>, TransformError> {
    if count < 2 {
        return Err(CreationError::Invalid(
            "frames need at least two samples".into(),
        ));
    }
    let resampled = resample_polyline(rail, count)?;
    let frames = polyline_frames(&resampled)?;
    Ok(frames.into_iter().map(from_path_frame).collect())
}

/// Arc-length sampled frames along an exact NURBS curve, using the surface kernel's path frames.
pub fn frames_along_curve(
    curve: &om9_core::geometry::Curve,
    count: usize,
    limits: &Limits,
) -> Result<Vec<PlacementFrame>, TransformError> {
    if count < 2 {
        return Err(CreationError::Invalid(
            "frames need at least two samples".into(),
        ));
    }
    let frames = surface_kernel::path_frames(curve, count, limits).map_err(map_core)?;
    Ok(frames
        .into_iter()
        .map(|frame| PlacementFrame {
            origin: frame.origin,
            x: frame.x,
            y: frame.y,
            z: frame.tangent,
        })
        .collect())
}

/// Frame on a NURBS surface at parameter `(u, v)`: `x` along `dS/du`, `z` along the unit normal,
/// `y = z × x` so the frame is right-handed.
pub fn frame_on_surface(
    surface: &NurbsSurface,
    u: f64,
    v: f64,
    limits: &Limits,
) -> Result<PlacementFrame, TransformError> {
    let derivatives = surface_kernel::derivatives(surface, u, v, limits).map_err(map_core)?;
    let point = surface_kernel::evaluate(surface, u, v, limits).map_err(map_core)?;
    let normal = unit(
        cross(derivatives.du, derivatives.dv),
        "surface UV derivatives (singular point)",
    )?;
    let x = unit(derivatives.du, "surface dS/du")?;
    let y = cross(normal, x);
    let frame = PlacementFrame {
        origin: point,
        x,
        y,
        z: normal,
    };
    frame.validate()?;
    Ok(frame)
}

/* ------------------------- orientation solver ------------------------- */

fn rotation_between(source: Point3, target: Point3) -> Result<[[f64; 3]; 3], TransformError> {
    let u = unit(source, "source direction")?;
    let v = unit(target, "target direction")?;
    let axis = cross(u, v);
    let sine = length(axis);
    let cosine = dot(u, v).clamp(-1.0, 1.0);
    if sine <= 1.0e-12 {
        if cosine > 0.0 {
            return Ok([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
        }
        // Opposite directions: rotate 180° about any axis perpendicular to u.
        let helper = if u[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let perpendicular = unit(cross(u, helper), "opposite direction axis")?;
        return Ok(rotation_matrix(perpendicular, std::f64::consts::PI));
    }
    Ok(rotation_matrix(
        [axis[0] / sine, axis[1] / sine, axis[2] / sine],
        sine.atan2(cosine),
    ))
}

fn rotation_matrix(axis: Point3, angle: f64) -> [[f64; 3]; 3] {
    let (sine, cosine) = angle.sin_cos();
    let one_minus = 1.0 - cosine;
    let [x, y, z] = axis;
    [
        [
            cosine + x * x * one_minus,
            x * y * one_minus - z * sine,
            x * z * one_minus + y * sine,
        ],
        [
            y * x * one_minus + z * sine,
            cosine + y * y * one_minus,
            y * z * one_minus - x * sine,
        ],
        [
            z * x * one_minus - y * sine,
            z * y * one_minus + x * sine,
            cosine + z * z * one_minus,
        ],
    ]
}

fn matrix_from_rotation(rotation: [[f64; 3]; 3], translation: Point3) -> Transform {
    let mut m = Transform::identity().0;
    for row in 0..3 {
        for column in 0..3 {
            m[row][column] = rotation[row][column];
        }
        m[row][3] = translation[row];
    }
    Transform(m)
}

fn multiply3(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for row in 0..3 {
        for column in 0..3 {
            out[row][column] = (0..3).map(|k| a[row][k] * b[k][column]).sum();
        }
    }
    out
}

fn transpose3(a: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for row in 0..3 {
        for column in 0..3 {
            out[row][column] = a[column][row];
        }
    }
    out
}

fn frame_rows(primary: Point3, secondary: Point3) -> Result<[[f64; 3]; 3], TransformError> {
    let e1 = unit(primary, "orientation reference")?;
    let normal = cross(primary, secondary);
    let e3 = unit(normal, "orientation references are collinear")?;
    let e2 = cross(e3, e1);
    // Rows are the local axes in world space.
    Ok([
        [e1[0], e1[1], e1[2]],
        [e2[0], e2[1], e2[2]],
        [e3[0], e3[1], e3[2]],
    ])
}

/// Rigid transform mapping source A→target A′ and the source direction A→B onto target A′→B′.
///
/// Two points leave the roll undefined; the solver picks the minimal rotation between the two
/// directions, which preserves every distance.
pub fn orient_two_point(
    source_a: Point3,
    source_b: Point3,
    target_a: Point3,
    target_b: Point3,
) -> Result<Transform, TransformError> {
    for point in [source_a, source_b, target_a, target_b] {
        finite(point).map_err(map_core)?;
    }
    let source_direction = [
        source_b[0] - source_a[0],
        source_b[1] - source_a[1],
        source_b[2] - source_a[2],
    ];
    let target_direction = [
        target_b[0] - target_a[0],
        target_b[1] - target_a[1],
        target_b[2] - target_a[2],
    ];
    if length(source_direction) <= 1.0e-12 || length(target_direction) <= 1.0e-12 {
        return Err(CreationError::Invalid(
            "orientation points must be distinct".into(),
        ));
    }
    let rotation = rotation_between(source_direction, target_direction)?;
    let rotated_a = [
        (0..3)
            .map(|column| rotation[0][column] * source_a[column])
            .sum::<f64>(),
        (0..3)
            .map(|column| rotation[1][column] * source_a[column])
            .sum::<f64>(),
        (0..3)
            .map(|column| rotation[2][column] * source_a[column])
            .sum::<f64>(),
    ];
    let translation = [
        target_a[0] - rotated_a[0],
        target_a[1] - rotated_a[1],
        target_a[2] - rotated_a[2],
    ];
    let matrix = matrix_from_rotation(rotation, translation);
    checked_matrix(&matrix)?;
    Ok(matrix)
}

/// Rigid transform mapping three non-collinear source points onto three target points.
pub fn orient_three_point(
    source: [Point3; 3],
    target: [Point3; 3],
) -> Result<Transform, TransformError> {
    for point in source.iter().chain(target.iter()) {
        finite(*point).map_err(map_core)?;
    }
    let source_u = [
        source[1][0] - source[0][0],
        source[1][1] - source[0][1],
        source[1][2] - source[0][2],
    ];
    let source_v = [
        source[2][0] - source[0][0],
        source[2][1] - source[0][1],
        source[2][2] - source[0][2],
    ];
    let target_u = [
        target[1][0] - target[0][0],
        target[1][1] - target[0][1],
        target[1][2] - target[0][2],
    ];
    let target_v = [
        target[2][0] - target[0][0],
        target[2][1] - target[0][1],
        target[2][2] - target[0][2],
    ];
    if length(source_u) <= 1.0e-12
        || length(target_u) <= 1.0e-12
        || length(cross(source_u, source_v)) <= 1.0e-12
        || length(cross(target_u, target_v)) <= 1.0e-12
    {
        return Err(CreationError::Invalid(
            "orientation points are coincident or collinear".into(),
        ));
    }
    let source_frame = frame_rows(source_u, source_v)?;
    let target_frame = frame_rows(target_u, target_v)?;
    let rotation = multiply3(transpose3(target_frame), source_frame);
    let rotated_a = [
        (0..3)
            .map(|column| rotation[0][column] * source[0][column])
            .sum::<f64>(),
        (0..3)
            .map(|column| rotation[1][column] * source[0][column])
            .sum::<f64>(),
        (0..3)
            .map(|column| rotation[2][column] * source[0][column])
            .sum::<f64>(),
    ];
    let translation = [
        target[0][0] - rotated_a[0],
        target[0][1] - rotated_a[1],
        target[0][2] - rotated_a[2],
    ];
    let matrix = matrix_from_rotation(rotation, translation);
    checked_matrix(&matrix)?;
    Ok(matrix)
}

/* ------------------------- patterns ------------------------- */

/// Linear pattern placements: `count` instances spaced by `spacing`. With `include_source` the
/// first placement is the identity; without it the source is skipped.
pub fn linear_pattern(
    count: usize,
    spacing: Point3,
    include_source: bool,
) -> Result<Vec<Transform>, TransformError> {
    if count == 0 {
        return Err(CreationError::Invalid(
            "pattern count must be positive".into(),
        ));
    }
    if count > MAX_PATTERN_INSTANCES {
        return Err(CreationError::Budget(format!(
            "pattern count above {MAX_PATTERN_INSTANCES}"
        )));
    }
    finite(spacing).map_err(map_core)?;
    if spacing.iter().any(|value| *value < 0.0) {
        return Err(CreationError::Invalid(
            "linear spacing must not be negative".into(),
        ));
    }
    if length(spacing) <= 1.0e-12 {
        return Err(CreationError::Invalid("linear spacing is zero".into()));
    }
    let first = usize::from(!include_source);
    let mut placements = Vec::new();
    placements
        .try_reserve_exact(count.saturating_sub(first))
        .map_err(|_| CreationError::Budget("pattern placements".into()))?;
    for index in first..count {
        placements.push(translation([
            spacing[0] * index as f64,
            spacing[1] * index as f64,
            spacing[2] * index as f64,
        ])?);
    }
    Ok(placements)
}

/// Polar pattern placements around `axis` through `center`. A full turn (`total_angle` = 2π)
/// divides by `count` so the first and last instances do not coincide.
pub fn polar_pattern(
    count: usize,
    center: Point3,
    axis: Point3,
    total_angle: f64,
    include_source: bool,
) -> Result<Vec<Transform>, TransformError> {
    if count < 2 {
        return Err(CreationError::Invalid(
            "polar pattern needs at least two instances".into(),
        ));
    }
    if count > MAX_PATTERN_INSTANCES {
        return Err(CreationError::Budget(format!(
            "pattern count above {MAX_PATTERN_INSTANCES}"
        )));
    }
    if !total_angle.is_finite() || total_angle == 0.0 {
        return Err(CreationError::Invalid(
            "polar pattern angle must be finite and non-zero".into(),
        ));
    }
    let full_turn = (total_angle.abs() - std::f64::consts::TAU).abs() <= 1.0e-9;
    let first = usize::from(!include_source);
    let divisor = if full_turn {
        count as f64
    } else if include_source {
        (count - 1) as f64
    } else {
        count as f64
    };
    let mut placements = Vec::new();
    placements
        .try_reserve_exact(count.saturating_sub(first))
        .map_err(|_| CreationError::Budget("pattern placements".into()))?;
    for index in first..count {
        placements.push(rotation(
            center,
            axis,
            total_angle * index as f64 / divisor,
        )?);
    }
    Ok(placements)
}

/// How a pattern is materialised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternMode {
    /// One shared definition plus placements; no geometry duplication.
    Instance,
    /// One independent transformed B-Rep per placement.
    Bake,
}

/// An instanced pattern: one definition and the placements that reference it.
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub definition: Brep,
    pub placements: Vec<Transform>,
}

/// The materialised pattern result.
///
/// The instanced definition is inline: it is the normal outcome and boxing it would allocate on
/// every pattern; the baked list is the alternative.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum PatternResult {
    Instanced(Pattern),
    Baked(Vec<Brep>),
}

impl PatternResult {
    pub fn placements(&self) -> usize {
        match self {
            Self::Instanced(pattern) => pattern.placements.len(),
            Self::Baked(bodies) => bodies.len(),
        }
    }
}

/// Materialise a pattern in instance or baked mode.
///
/// Instance mode keeps exactly one definition; bake mode produces one independent body per
/// placement. Every placement is validated before any body is transformed, so a bad placement
/// never leaves a partial result.
pub fn apply_pattern(
    definition: &Brep,
    placements: &[Transform],
    mode: PatternMode,
    deflection_mm: f64,
    limits: &Limits,
) -> Result<PatternResult, TransformError> {
    if placements.is_empty() {
        return Err(CreationError::Invalid(
            "pattern needs at least one placement".into(),
        ));
    }
    if placements.len() > MAX_PATTERN_INSTANCES {
        return Err(CreationError::Budget(format!(
            "pattern placements above {MAX_PATTERN_INSTANCES}"
        )));
    }
    for placement in placements {
        checked_matrix(placement)?;
    }
    definition
        .validate(limits)
        .map_err(|error| CreationError::Invalid(error.to_string()))?;
    match mode {
        PatternMode::Instance => Ok(PatternResult::Instanced(Pattern {
            definition: definition.clone(),
            placements: placements.to_vec(),
        })),
        PatternMode::Bake => {
            let mut bodies = Vec::new();
            bodies
                .try_reserve_exact(placements.len())
                .map_err(|_| CreationError::Budget("baked bodies".into()))?;
            for placement in placements {
                bodies.push(transform_brep(
                    definition,
                    placement,
                    deflection_mm,
                    limits,
                )?);
            }
            Ok(PatternResult::Baked(bodies))
        }
    }
}
