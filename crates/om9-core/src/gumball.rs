// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gumball and occurrence-array kernel (M02 `OM9-L2-03-03`, wave W4).
//!
//! Ported from the legacy bridge (`rust/src/{gumball,gumball_component,array_occurrences}.rs`)
//! into safe Rust. The legacy C ABI, the Qt view glue and the session/owner bookkeeping stay
//! behind; the portable part — handle model, constrained drag arithmetic, rotation/scale about
//! a frame origin and occurrence arrays — is re-implemented here.
use crate::{
    Error, Result,
    geometry::{Point3, Transform, finite},
    identity::ObjectId,
    model::Document,
    transaction::Mutation,
};

/// The three gumball axes. Codes match the legacy wire values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn code(self) -> u32 {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::X),
            1 => Some(Self::Y),
            2 => Some(Self::Z),
            _ => None,
        }
    }
    pub fn index(self) -> usize {
        self.code() as usize
    }
}

/// The three gumball planes. Codes match the legacy wire values (`XY` = 3 ... `ZX` = 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plane {
    XY,
    YZ,
    ZX,
}

impl Plane {
    pub fn code(self) -> u32 {
        3 + self as u32
    }
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            3 => Some(Self::XY),
            4 => Some(Self::YZ),
            5 => Some(Self::ZX),
            _ => None,
        }
    }
    /// The plane's normal axis: `XY -> Z`, `YZ -> X`, `ZX -> Y`.
    pub fn normal_axis(self) -> Axis {
        match self {
            Self::XY => Axis::Z,
            Self::YZ => Axis::X,
            Self::ZX => Axis::Y,
        }
    }
}

/// Which gumball handle a drag started on. Codes and order follow the legacy model, so the
/// UI layer can keep sending the same numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Handle {
    MoveAxis(Axis),
    MovePlane(Plane),
    MoveFree,
    Rotate(Axis),
    Scale(Axis),
}

impl Handle {
    pub fn code(self) -> u32 {
        match self {
            Self::MoveAxis(axis) => axis.code(),
            Self::MovePlane(plane) => plane.code(),
            Self::MoveFree => 6,
            Self::Rotate(axis) => 7 + axis.code(),
            Self::Scale(axis) => 10 + axis.code(),
        }
    }
    pub fn from_code(code: u32) -> Option<Self> {
        Some(match code {
            0..=2 => Self::MoveAxis(Axis::from_code(code)?),
            3..=5 => Self::MovePlane(Plane::from_code(code)?),
            6 => Self::MoveFree,
            7..=9 => Self::Rotate(Axis::from_code(code - 7)?),
            10..=12 => Self::Scale(Axis::from_code(code - 10)?),
            _ => return None,
        })
    }
}

/// The gumball frame: an orthonormal frame placed at the selection pivot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub origin: Point3,
    pub x: Point3,
    pub y: Point3,
    pub z: Point3,
}

impl Frame {
    pub fn world() -> Self {
        Self {
            origin: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            z: [0.0, 0.0, 1.0],
        }
    }
    /// A frame at `origin` whose third axis is `x cross y`, as the legacy relocation
    /// (`gumball_component`) built it.
    pub fn new(origin: Point3, x: Point3, y: Point3) -> Result<Self> {
        let z = cross(x, y);
        let frame = Self { origin, x, y, z };
        frame.validate()?;
        Ok(frame)
    }
    pub fn validate(&self) -> Result<()> {
        for point in [self.origin, self.x, self.y, self.z] {
            finite(point)?;
        }
        if (dot(self.x, self.x) - 1.0).abs() > 1e-9
            || (dot(self.y, self.y) - 1.0).abs() > 1e-9
            || (dot(self.z, self.z) - 1.0).abs() > 1e-9
            || dot(self.x, self.y).abs() > 1e-9
            || dot(self.x, self.z).abs() > 1e-9
            || dot(self.y, self.z).abs() > 1e-9
        {
            return Err(Error::Invalid("gumball frame"));
        }
        Ok(())
    }
    pub fn axis(&self, axis: Axis) -> Point3 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }
    pub fn plane_basis(&self, plane: Plane) -> (Point3, Point3) {
        match plane {
            Plane::XY => (self.x, self.y),
            Plane::YZ => (self.y, self.z),
            Plane::ZX => (self.z, self.x),
        }
    }
    pub fn plane_normal(&self, plane: Plane) -> Point3 {
        self.axis(plane.normal_axis())
    }
}

/// What the pointer produced for this drag. Translation handles consume a world delta;
/// rotation and scale handles consume an angle (radians) or a factor, because the angle and
/// the factor come from the screen projection in the scene layer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Drag {
    WorldDelta(Point3),
    Angle(f64),
    Factor(f64),
}

/// The transform a drag produces. A constrained handle only moves along what it constrains:
/// an axis handle keeps the component along its axis, a plane handle drops the normal
/// component, and a free handle keeps the whole delta.
pub fn transform(frame: &Frame, handle: Handle, drag: Drag) -> Result<Transform> {
    frame.validate()?;
    match (handle, drag) {
        (Handle::MoveAxis(axis), Drag::WorldDelta(delta)) => {
            finite(delta)?;
            let axis = frame.axis(axis);
            let amount = dot(delta, axis);
            Ok(Transform::translation([
                axis[0] * amount,
                axis[1] * amount,
                axis[2] * amount,
            ]))
        }
        (Handle::MovePlane(plane), Drag::WorldDelta(delta)) => {
            finite(delta)?;
            let normal = frame.plane_normal(plane);
            let along = dot(delta, normal);
            let projected = [
                delta[0] - normal[0] * along,
                delta[1] - normal[1] * along,
                delta[2] - normal[2] * along,
            ];
            Ok(Transform::translation(projected))
        }
        (Handle::MoveFree, Drag::WorldDelta(delta)) => {
            finite(delta)?;
            Ok(Transform::translation(delta))
        }
        (Handle::Rotate(axis), Drag::Angle(angle)) => {
            rotation_about(frame.origin, frame.axis(axis), angle)
        }
        (Handle::Scale(axis), Drag::Factor(factor)) => {
            scaling_along(frame.origin, frame.axis(axis), factor)
        }
        _ => Err(Error::Invalid("gumball drag input")),
    }
}

/// Rotation about a unit axis through `origin` (right-handed, radians).
pub fn rotation_about(origin: Point3, axis: Point3, angle: f64) -> Result<Transform> {
    finite(origin)?;
    finite(axis)?;
    if !angle.is_finite() {
        return Err(Error::Invalid("gumball rotation angle"));
    }
    let length = dot(axis, axis).sqrt();
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("gumball rotation axis"));
    }
    let [x, y, z] = [axis[0] / length, axis[1] / length, axis[2] / length];
    let (sine, cosine) = angle.sin_cos();
    let one_minus = 1.0 - cosine;
    let linear = [
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
    ];
    // p' = origin + R (p - origin)  =>  translation = origin - R origin
    let rotated_origin = [
        linear[0][0] * origin[0] + linear[0][1] * origin[1] + linear[0][2] * origin[2],
        linear[1][0] * origin[0] + linear[1][1] * origin[1] + linear[1][2] * origin[2],
        linear[2][0] * origin[0] + linear[2][1] * origin[1] + linear[2][2] * origin[2],
    ];
    let mut matrix = Transform::identity().0;
    for row in 0..3 {
        for column in 0..3 {
            matrix[row][column] = linear[row][column];
        }
        matrix[row][3] = origin[row] - rotated_origin[row];
    }
    let transform = Transform(matrix);
    transform.validate()?;
    Ok(transform)
}

/// Scale along one axis through `origin`; the other two axes keep their length.
pub fn scaling_along(origin: Point3, axis: Point3, factor: f64) -> Result<Transform> {
    finite(origin)?;
    finite(axis)?;
    if !factor.is_finite() || factor <= 0.0 {
        return Err(Error::Invalid("gumball scale factor"));
    }
    let length = dot(axis, axis).sqrt();
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("gumball scale axis"));
    }
    let unit = [axis[0] / length, axis[1] / length, axis[2] / length];
    let mut matrix = Transform::identity().0;
    for (row, line) in matrix.iter_mut().enumerate().take(3) {
        for (column, value) in line.iter_mut().enumerate().take(3) {
            // I + (factor - 1) * u u^T
            let identity = if row == column { 1.0 } else { 0.0 };
            *value = identity + (factor - 1.0) * unit[row] * unit[column];
        }
    }
    // p' = origin + M (p - origin)  =>  translation = origin - M origin
    for (row, line) in matrix.iter_mut().enumerate().take(3) {
        let moved = line[0] * origin[0] + line[1] * origin[1] + line[2] * origin[2];
        line[3] = origin[row] - moved;
    }
    let transform = Transform(matrix);
    transform.validate()?;
    Ok(transform)
}

/// How many occurrences an array may create. A gumball array is a UI action, so the bound is
/// generous but finite; an unbounded count would be a denial of service.
pub const MAX_ARRAY_OCCURRENCES: usize = 10_000;

/// Rectangular (two-dimensional) array of occurrences, excluding the source by default.
pub fn rectangular_array(
    frame: &Frame,
    counts: [usize; 2],
    spacing: [f64; 2],
    include_source: bool,
) -> Result<Vec<Transform>> {
    frame.validate()?;
    if counts[0] == 0 || counts[1] == 0 {
        return Err(Error::Invalid("gumball array count"));
    }
    let total = counts[0]
        .checked_mul(counts[1])
        .ok_or(Error::Overflow("array occurrences"))?;
    if total > MAX_ARRAY_OCCURRENCES {
        return Err(Error::Invalid("gumball array count"));
    }
    for value in spacing {
        if !value.is_finite() {
            return Err(Error::Invalid("gumball array spacing"));
        }
    }
    if counts[0] == 1 && counts[1] == 1 {
        return Err(Error::Invalid("gumball array count"));
    }
    let mut occurrences = Vec::new();
    let keep = if include_source { total } else { total - 1 };
    occurrences
        .try_reserve_exact(keep)
        .map_err(|_| Error::Budget("array occurrences"))?;
    for i in 0..counts[0] {
        for j in 0..counts[1] {
            if !include_source && i == 0 && j == 0 {
                continue;
            }
            let delta = [
                frame.x[0] * spacing[0] * i as f64 + frame.y[0] * spacing[1] * j as f64,
                frame.x[1] * spacing[0] * i as f64 + frame.y[1] * spacing[1] * j as f64,
                frame.x[2] * spacing[0] * i as f64 + frame.y[2] * spacing[1] * j as f64,
            ];
            occurrences.push(Transform::translation(delta));
        }
    }
    Ok(occurrences)
}

/// Polar array of occurrences about the frame `z` axis through the frame origin, spread over
/// `total_angle` radians (a full turn when `total_angle` is `TAU`), excluding the source.
pub fn polar_array(
    frame: &Frame,
    count: usize,
    total_angle: f64,
    include_source: bool,
) -> Result<Vec<Transform>> {
    frame.validate()?;
    if !(2..=MAX_ARRAY_OCCURRENCES).contains(&count) {
        return Err(Error::Invalid("gumball array count"));
    }
    if !total_angle.is_finite() || total_angle == 0.0 {
        return Err(Error::Invalid("gumball array angle"));
    }
    // A full turn must not duplicate the first item on the last step.
    let closed = (total_angle.abs() - std::f64::consts::TAU).abs() <= 1e-12;
    let step = if closed {
        total_angle / count as f64
    } else {
        total_angle / (count - 1) as f64
    };
    let mut occurrences = Vec::new();
    let keep = if include_source { count } else { count - 1 };
    occurrences
        .try_reserve_exact(keep)
        .map_err(|_| Error::Budget("array occurrences"))?;
    for index in 0..count {
        if !include_source && index == 0 {
            continue;
        }
        occurrences.push(rotation_about(frame.origin, frame.z, step * index as f64)?);
    }
    Ok(occurrences)
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

/* ------------------------- drag session ------------------------- */

/// A gumball drag session: it collects the drags of one gesture and turns them into the
/// mutations of **one undoable step**. Move rewrites `Entity.placement` (geometry is never
/// baked), copy inserts clones with fresh identities and the transformed placement.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    frame: Frame,
    targets: Vec<ObjectId>,
    copy: bool,
    accumulated: Transform,
    drags: usize,
}

impl Session {
    pub fn begin(frame: Frame, targets: Vec<ObjectId>, copy: bool) -> Result<Self> {
        frame.validate()?;
        if targets.is_empty() {
            return Err(Error::Invalid("gumball targets"));
        }
        let mut seen = std::collections::BTreeSet::new();
        for target in &targets {
            if target.is_nil() || !seen.insert(*target) {
                return Err(Error::Invalid("gumball targets"));
            }
        }
        Ok(Self {
            frame,
            targets,
            copy,
            accumulated: Transform::identity(),
            drags: 0,
        })
    }
    pub fn frame(&self) -> &Frame {
        &self.frame
    }
    pub fn targets(&self) -> &[ObjectId] {
        &self.targets
    }
    pub fn is_copy(&self) -> bool {
        self.copy
    }
    pub fn set_copy(&mut self, copy: bool) {
        self.copy = copy;
    }
    /// The transform applied so far. Successive drags compose in world order, so a second
    /// drag is applied after the first.
    pub fn accumulated(&self) -> Transform {
        self.accumulated
    }
    pub fn drag_count(&self) -> usize {
        self.drags
    }
    /// Add one drag to the gesture. The frame stays where the gesture began.
    pub fn drag(&mut self, handle: Handle, drag: Drag) -> Result<()> {
        let step = transform(&self.frame, handle, drag)?;
        self.accumulated = step.compose(self.accumulated)?;
        self.drags += 1;
        Ok(())
    }
    /// Drop the accumulated transform but keep the gesture (Esc while dragging).
    pub fn reset(&mut self) {
        self.accumulated = Transform::identity();
        self.drags = 0;
    }
    /// Move mutations: one `Placement` per target, stamped against the document as it is now.
    pub fn move_mutations(&self, document: &Document) -> Result<Vec<Mutation>> {
        if self.copy {
            return Err(Error::Invalid("gumball session mode"));
        }
        let mut mutations = Vec::new();
        mutations
            .try_reserve_exact(self.targets.len())
            .map_err(|_| Error::Budget("gumball mutations"))?;
        for target in &self.targets {
            let entity = document
                .entities
                .get(target)
                .ok_or(Error::Missing("object"))?;
            let placement = self.accumulated.compose(entity.placement)?;
            mutations.push(Mutation::Placement {
                source: document.stamp(*target)?,
                placement,
            });
        }
        Ok(mutations)
    }
    /// Copy mutations: one `Insert` per target with a fresh identity, the same geometry and
    /// the transformed placement.
    pub fn copy_mutations(&self, document: &Document) -> Result<Vec<Mutation>> {
        if !self.copy {
            return Err(Error::Invalid("gumball session mode"));
        }
        let mut mutations = Vec::new();
        mutations
            .try_reserve_exact(self.targets.len())
            .map_err(|_| Error::Budget("gumball mutations"))?;
        for target in &self.targets {
            let entity = document
                .entities
                .get(target)
                .ok_or(Error::Missing("object"))?;
            let mut clone = entity.clone();
            clone.id = ObjectId::new();
            clone.placement = self.accumulated.compose(entity.placement)?;
            mutations.push(Mutation::Insert {
                document: document.id,
                entity: clone,
            });
        }
        Ok(mutations)
    }
}
