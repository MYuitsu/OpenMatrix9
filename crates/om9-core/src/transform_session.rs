// SPDX-License-Identifier: LGPL-2.1-or-later
//! Interactive Rhino-style transform command sessions (`OM9-L2-02-06`).
//!
//! The gumball owns direct handle drags; this module owns the typed command state used by
//! Move/Copy, Rotate and Scale 1D/2D/3D: a base/reference point, a live preview transform,
//! Copy, Cancel and commit. The session is a pure value: the host applies the preview to the
//! scene and commits the same transform as one undoable transaction.
use crate::{
    Error, Result,
    geometry::{Point3, Transform, finite},
    gumball::{Axis, Plane},
};

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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

fn length(a: Point3) -> f64 {
    dot(a, a).sqrt()
}

fn unit(a: Point3, label: &'static str) -> Result<Point3> {
    let magnitude = length(a);
    if !magnitude.is_finite() || magnitude <= 1e-12 {
        return Err(Error::Invalid(label));
    }
    Ok([a[0] / magnitude, a[1] / magnitude, a[2] / magnitude])
}

fn checked_point(point: Point3, label: &'static str) -> Result<()> {
    finite(point).map_err(|_| Error::Invalid(label))
}

fn translation(offset: Point3) -> Transform {
    let mut matrix = Transform::identity();
    matrix.0[0][3] = offset[0];
    matrix.0[1][3] = offset[1];
    matrix.0[2][3] = offset[2];
    matrix
}

fn rotation(origin: Point3, axis: Point3, angle: f64) -> Result<Transform> {
    let axis = unit(axis, "transform rotation axis")?;
    let (sin, cos) = angle.sin_cos();
    let one_minus = 1.0 - cos;
    let [x, y, z] = axis;
    let mut matrix = Transform::identity();
    matrix.0[0][0] = cos + x * x * one_minus;
    matrix.0[0][1] = x * y * one_minus - z * sin;
    matrix.0[0][2] = x * z * one_minus + y * sin;
    matrix.0[1][0] = y * x * one_minus + z * sin;
    matrix.0[1][1] = cos + y * y * one_minus;
    matrix.0[1][2] = y * z * one_minus - x * sin;
    matrix.0[2][0] = z * x * one_minus - y * sin;
    matrix.0[2][1] = z * y * one_minus + x * sin;
    matrix.0[2][2] = cos + z * z * one_minus;
    let rotated = matrix.point(origin)?;
    matrix.0[0][3] = origin[0] - rotated[0];
    matrix.0[1][3] = origin[1] - rotated[1];
    matrix.0[2][3] = origin[2] - rotated[2];
    matrix.validate()?;
    Ok(matrix)
}

fn scaling(origin: Point3, factors: Point3) -> Result<Transform> {
    if factors
        .iter()
        .any(|factor| !factor.is_finite() || *factor == 0.0)
    {
        return Err(Error::Invalid("transform scale factor"));
    }
    let mut matrix = Transform::identity();
    for axis in 0..3 {
        matrix.0[axis][axis] = factors[axis];
        matrix.0[axis][3] = origin[axis] * (1.0 - factors[axis]);
    }
    matrix.validate()?;
    Ok(matrix)
}

fn project_perpendicular(vector: Point3, axis: Point3) -> Point3 {
    let along = dot(vector, axis);
    [
        vector[0] - axis[0] * along,
        vector[1] - axis[1] * along,
        vector[2] - axis[2] * along,
    ]
}

/* ------------------------------------ move / copy ------------------------------------ */

/// A Move/Copy reference-point command.
#[derive(Debug, Clone, PartialEq)]
pub struct MoveSession {
    base: Point3,
    reference: Option<Point3>,
    copy: bool,
}

impl MoveSession {
    pub fn begin(base: Point3, copy: bool) -> Result<Self> {
        checked_point(base, "move base point")?;
        Ok(Self {
            base,
            reference: None,
            copy,
        })
    }

    pub fn set_reference(&mut self, point: Point3) -> Result<()> {
        checked_point(point, "move reference point")?;
        self.reference = Some(point);
        Ok(())
    }

    pub fn copy(&self) -> bool {
        self.copy
    }

    pub fn set_copy(&mut self, copy: bool) {
        self.copy = copy;
    }

    pub fn base(&self) -> Point3 {
        self.base
    }

    pub fn reference(&self) -> Option<Point3> {
        self.reference
    }

    /// The live preview. Before the reference point is supplied this is identity.
    pub fn preview(&self) -> Result<Transform> {
        match self.reference {
            Some(reference) => Ok(translation(sub(reference, self.base))),
            None => Ok(Transform::identity()),
        }
    }

    pub fn cancel(&mut self) {
        self.reference = None;
    }
}

/* ------------------------------------ rotate ------------------------------------ */

/// A Rotate command around a fixed axis and centre.
#[derive(Debug, Clone, PartialEq)]
pub struct RotateSession {
    center: Point3,
    axis: Point3,
    reference: Option<Point3>,
    current: Option<Point3>,
    copy: bool,
}

impl RotateSession {
    pub fn begin(center: Point3, axis: Point3, copy: bool) -> Result<Self> {
        checked_point(center, "rotate center")?;
        let axis = unit(axis, "rotate axis")?;
        Ok(Self {
            center,
            axis,
            reference: None,
            current: None,
            copy,
        })
    }

    pub fn set_reference(&mut self, point: Point3) -> Result<()> {
        checked_point(point, "rotate reference point")?;
        self.reference = Some(point);
        Ok(())
    }

    pub fn set_current(&mut self, point: Point3) -> Result<()> {
        checked_point(point, "rotate current point")?;
        self.current = Some(point);
        Ok(())
    }

    pub fn copy(&self) -> bool {
        self.copy
    }

    pub fn set_copy(&mut self, copy: bool) {
        self.copy = copy;
    }

    /// The signed angle in radians from the reference vector to the current vector around
    /// the axis, projected perpendicular to the axis.
    pub fn angle(&self) -> Result<f64> {
        let Some(reference) = self.reference else {
            return Ok(0.0);
        };
        let Some(current) = self.current else {
            return Ok(0.0);
        };
        let a = project_perpendicular(sub(reference, self.center), self.axis);
        let b = project_perpendicular(sub(current, self.center), self.axis);
        if length(a) <= 1e-12 || length(b) <= 1e-12 {
            return Err(Error::Invalid("rotate reference vector"));
        }
        Ok(dot(self.axis, cross(a, b)).atan2(dot(a, b)))
    }

    pub fn preview(&self) -> Result<Transform> {
        rotation(self.center, self.axis, self.angle()?)
    }

    pub fn cancel(&mut self) {
        self.reference = None;
        self.current = None;
    }
}

/* ------------------------------------ scale ------------------------------------ */

/// The scale dimensionality a command asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleAxes {
    OneD(Axis),
    TwoD(Plane),
    ThreeD,
}

impl ScaleAxes {
    fn factors(self, factor: f64) -> Point3 {
        match self {
            ScaleAxes::OneD(Axis::X) => [factor, 1.0, 1.0],
            ScaleAxes::OneD(Axis::Y) => [1.0, factor, 1.0],
            ScaleAxes::OneD(Axis::Z) => [1.0, 1.0, factor],
            ScaleAxes::TwoD(Plane::XY) => [factor, factor, 1.0],
            ScaleAxes::TwoD(Plane::YZ) => [1.0, factor, factor],
            ScaleAxes::TwoD(Plane::ZX) => [factor, 1.0, factor],
            ScaleAxes::ThreeD => [factor, factor, factor],
        }
    }

    fn projected_distance(self, vector: Point3) -> f64 {
        match self {
            ScaleAxes::OneD(Axis::X) => vector[0].abs(),
            ScaleAxes::OneD(Axis::Y) => vector[1].abs(),
            ScaleAxes::OneD(Axis::Z) => vector[2].abs(),
            ScaleAxes::TwoD(Plane::XY) => (vector[0] * vector[0] + vector[1] * vector[1]).sqrt(),
            ScaleAxes::TwoD(Plane::YZ) => (vector[1] * vector[1] + vector[2] * vector[2]).sqrt(),
            ScaleAxes::TwoD(Plane::ZX) => (vector[2] * vector[2] + vector[0] * vector[0]).sqrt(),
            ScaleAxes::ThreeD => length(vector),
        }
    }
}

/// A Scale 1D/2D/3D command with an origin and a reference distance.
#[derive(Debug, Clone, PartialEq)]
pub struct ScaleSession {
    origin: Point3,
    axes: ScaleAxes,
    reference: Option<Point3>,
    current: Option<Point3>,
    copy: bool,
}

impl ScaleSession {
    pub fn begin(origin: Point3, axes: ScaleAxes, copy: bool) -> Result<Self> {
        checked_point(origin, "scale origin")?;
        Ok(Self {
            origin,
            axes,
            reference: None,
            current: None,
            copy,
        })
    }

    pub fn set_reference(&mut self, point: Point3) -> Result<()> {
        checked_point(point, "scale reference point")?;
        self.reference = Some(point);
        Ok(())
    }

    pub fn set_current(&mut self, point: Point3) -> Result<()> {
        checked_point(point, "scale current point")?;
        self.current = Some(point);
        Ok(())
    }

    pub fn copy(&self) -> bool {
        self.copy
    }

    pub fn set_copy(&mut self, copy: bool) {
        self.copy = copy;
    }

    /// The scalar factor from the reference to the current projected distance. A zero or
    /// nearly zero reference distance has no finite factor and is refused.
    pub fn factor(&self) -> Result<f64> {
        let Some(reference) = self.reference else {
            return Ok(1.0);
        };
        let Some(current) = self.current else {
            return Ok(1.0);
        };
        let reference = self.axes.projected_distance(sub(reference, self.origin));
        let current = self.axes.projected_distance(sub(current, self.origin));
        if !reference.is_finite() || reference <= 1e-12 || !current.is_finite() {
            return Err(Error::Invalid("scale reference distance"));
        }
        Ok(current / reference)
    }

    pub fn preview(&self) -> Result<Transform> {
        let factor = self.factor()?;
        scaling(self.origin, self.axes.factors(factor))
    }

    pub fn cancel(&mut self) {
        self.reference = None;
        self.current = None;
    }
}

/* --------------------------------- align / set point --------------------------------- */

/// Which components an Align command applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignMode {
    Translate,
    Rotate,
    Scale,
    RotateScale,
}

/// A complete Align request: move `source_origin` onto `target_origin`, optionally rotate the
/// source axis onto the target axis and/or scale the source extent to the target extent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignRequest {
    pub source_origin: Point3,
    pub target_origin: Point3,
    pub source_axis: Option<Point3>,
    pub target_axis: Option<Point3>,
    pub source_extent: Option<f64>,
    pub target_extent: Option<f64>,
    pub mode: AlignMode,
}

impl AlignRequest {
    pub fn translate(source_origin: Point3, target_origin: Point3) -> Self {
        Self {
            source_origin,
            target_origin,
            source_axis: None,
            target_axis: None,
            source_extent: None,
            target_extent: None,
            mode: AlignMode::Translate,
        }
    }

    pub fn rotate(
        source_origin: Point3,
        target_origin: Point3,
        source_axis: Point3,
        target_axis: Point3,
    ) -> Self {
        Self {
            source_axis: Some(source_axis),
            target_axis: Some(target_axis),
            mode: AlignMode::Rotate,
            ..Self::translate(source_origin, target_origin)
        }
    }

    pub fn scale(
        source_origin: Point3,
        target_origin: Point3,
        source_extent: f64,
        target_extent: f64,
    ) -> Self {
        Self {
            source_extent: Some(source_extent),
            target_extent: Some(target_extent),
            mode: AlignMode::Scale,
            ..Self::translate(source_origin, target_origin)
        }
    }

    pub fn rotate_scale(
        source_origin: Point3,
        target_origin: Point3,
        source_axis: Point3,
        target_axis: Point3,
        source_extent: f64,
        target_extent: f64,
    ) -> Self {
        Self {
            source_axis: Some(source_axis),
            target_axis: Some(target_axis),
            source_extent: Some(source_extent),
            target_extent: Some(target_extent),
            mode: AlignMode::RotateScale,
            ..Self::translate(source_origin, target_origin)
        }
    }
}

/// Rotation that maps `from` onto `to` about `origin`. Opposite vectors use a deterministic
/// perpendicular axis, so a 180-degree align is still a single rigid transform.
pub fn align_rotation(from: Point3, to: Point3, origin: Point3) -> Result<Transform> {
    checked_point(origin, "align rotation origin")?;
    let from = unit(from, "align source axis")?;
    let to = unit(to, "align target axis")?;
    let axis = cross(from, to);
    let sine = length(axis);
    let cosine = dot(from, to).clamp(-1.0, 1.0);
    if sine <= 1e-12 {
        if cosine > 0.0 {
            return Ok(Transform::identity());
        }
        // 180 degrees: any axis perpendicular to `from` is a valid deterministic choice.
        let helper = if from[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let axis = unit(cross(from, helper), "align opposite axis")?;
        return rotation(origin, axis, std::f64::consts::PI);
    }
    rotation(origin, axis, sine.atan2(cosine))
}

/// Apply a complete Align request: scale and rotate about the source origin, then translate the
/// origin onto the target origin. The returned Transform is the one the host commits.
pub fn align_transform(request: &AlignRequest) -> Result<Transform> {
    checked_point(request.source_origin, "align source origin")?;
    checked_point(request.target_origin, "align target origin")?;
    let needs_rotation = matches!(request.mode, AlignMode::Rotate | AlignMode::RotateScale);
    let needs_scale = matches!(request.mode, AlignMode::Scale | AlignMode::RotateScale);
    let scale_factor = if needs_scale {
        let source = request
            .source_extent
            .ok_or(Error::Missing("align source extent"))?;
        let target = request
            .target_extent
            .ok_or(Error::Missing("align target extent"))?;
        if !source.is_finite() || source <= 1e-12 || !target.is_finite() || target <= 0.0 {
            return Err(Error::Invalid("align extent"));
        }
        target / source
    } else {
        1.0
    };
    let rotation = if needs_rotation {
        let source_axis = request
            .source_axis
            .ok_or(Error::Missing("align source axis"))?;
        let target_axis = request
            .target_axis
            .ok_or(Error::Missing("align target axis"))?;
        align_rotation(source_axis, target_axis, request.source_origin)?
    } else {
        Transform::identity()
    };
    let scaling = scaling(
        request.source_origin,
        [scale_factor, scale_factor, scale_factor],
    )?;
    let translation = translation(sub(request.target_origin, request.source_origin));
    // Apply the scale first, then the rotation, then the origin translation.
    translation.compose(rotation)?.compose(scaling)
}

/// SetPoint: translate the selected reference point to a world target on the named axes.
pub fn set_point_transform(current: Point3, target: Point3, axes: [bool; 3]) -> Result<Transform> {
    checked_point(current, "set point current")?;
    checked_point(target, "set point target")?;
    let delta = sub(target, current);
    let delta = [
        if axes[0] { delta[0] } else { 0.0 },
        if axes[1] { delta[1] } else { 0.0 },
        if axes[2] { delta[2] } else { 0.0 },
    ];
    Ok(translation(delta))
}

/// Align the selected point to a construction plane: translate it along the plane normal.
pub fn align_point_to_plane(
    point: Point3,
    plane_origin: Point3,
    plane_normal: Point3,
) -> Result<Transform> {
    checked_point(point, "align point")?;
    checked_point(plane_origin, "align plane origin")?;
    let normal = unit(plane_normal, "align plane normal")?;
    let distance = dot(sub(point, plane_origin), normal);
    Ok(translation([
        -normal[0] * distance,
        -normal[1] * distance,
        -normal[2] * distance,
    ]))
}

/// Align the selected point to a world axis: translate it onto the axis line.
pub fn align_point_to_axis(
    point: Point3,
    axis_origin: Point3,
    axis_direction: Point3,
) -> Result<Transform> {
    checked_point(point, "align point")?;
    checked_point(axis_origin, "align axis origin")?;
    let axis = unit(axis_direction, "align axis direction")?;
    let relative = sub(point, axis_origin);
    let along = dot(relative, axis);
    let projected = [
        axis_origin[0] + axis[0] * along,
        axis_origin[1] + axis[1] * along,
        axis_origin[2] + axis[2] * along,
    ];
    Ok(translation(sub(projected, point)))
}
