// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix Top 11 command service (OM9-L2-03-08).
//!
//! The Top 11 are command adapters, not a second geometry kernel: duplicate, mirror,
//! move/rotate, point display, explode/join/split/trim all call the existing curve,
//! placement and document records. This module owns the action catalogue, the
//! availability rule and the pure entity/geometry builders; the host commits them in
//! one transaction.
use crate::{
    Error, Result,
    budget::Limits,
    curve_ops, curve_surface_ops,
    geometry::{Curve, Geometry, GeometryKind, NurbsCurve, Point3, Transform, finite},
    identity::ObjectId,
    model::{Document, Entity},
    surface_kernel,
};
use std::collections::BTreeSet;

/// The eleven Matrix Top 11 actions in toolbar order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Top11Action {
    Duplicate,
    Mirror,
    Move,
    Rotate,
    EditPoints,
    ControlPoints,
    Explode,
    Join,
    Split,
    Trim,
    RingRail,
}

impl Top11Action {
    pub const ALL: [Self; 11] = [
        Self::Duplicate,
        Self::Mirror,
        Self::Move,
        Self::Rotate,
        Self::EditPoints,
        Self::ControlPoints,
        Self::Explode,
        Self::Join,
        Self::Split,
        Self::Trim,
        Self::RingRail,
    ];

    /// Original Matrix command name.
    pub fn command(self) -> &'static str {
        match self {
            Self::Duplicate => "Copy",
            Self::Mirror => "Mirror",
            Self::Move => "Move",
            Self::Rotate => "Rotate",
            Self::EditPoints => "EditPtOn",
            Self::ControlPoints => "PointsOn",
            Self::Explode => "Explode",
            Self::Join => "Join",
            Self::Split => "Split",
            Self::Trim => "Trim",
            Self::RingRail => "gvRingRail",
        }
    }

    /// Icon key registered by the application.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Duplicate => "Duplicate",
            Self::Mirror => "Mirror",
            Self::Move => "Move",
            Self::Rotate => "Rotate",
            Self::EditPoints => "EditPoints",
            Self::ControlPoints => "ControlPoints",
            Self::Explode => "Explode",
            Self::Join => "Join",
            Self::Split => "Split",
            Self::Trim => "Trim",
            Self::RingRail => "RingRail",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Duplicate => "Duplicate",
            Self::Mirror => "Mirror",
            Self::Move => "Move",
            Self::Rotate => "Rotate",
            Self::EditPoints => "Edit Points On",
            Self::ControlPoints => "Control Points On",
            Self::Explode => "Explode",
            Self::Join => "Join",
            Self::Split => "Split",
            Self::Trim => "Trim",
            Self::RingRail => "Ring Rail",
        }
    }

    /// Core requirement id from `docs/core-requirements.json`.
    pub fn feature_id(self) -> &'static str {
        match self {
            Self::ControlPoints => "OM9-TOP11-001",
            Self::Duplicate => "OM9-TOP11-002",
            Self::EditPoints => "OM9-TOP11-003",
            Self::Mirror => "OM9-TOP11-004",
            Self::Explode => "OM9-TOP11-005",
            Self::Move => "OM9-TOP11-006",
            Self::Rotate => "OM9-TOP11-007",
            Self::Join => "OM9-TOP11-008",
            Self::Split => "OM9-TOP11-009",
            Self::Trim => "OM9-TOP11-010",
            Self::RingRail => "OM9-TOP11-011",
        }
    }

    /// Only `PointsOn` has a documented Matrix9 shortcut (F10).
    pub fn shortcut(self) -> Option<&'static str> {
        match self {
            Self::ControlPoints => Some("F10"),
            _ => None,
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|action| action.label().eq_ignore_ascii_case(label))
    }

    pub fn from_icon(icon: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|action| action.icon().eq_ignore_ascii_case(icon))
    }
}

/// Point display mode selected by the two `*PtOn` actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GripMode {
    Edit,
    Control,
}

/// What the selection currently offers to the Top 11.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Top11Context {
    pub running: bool,
    pub selected: usize,
    pub curves: usize,
    pub points: usize,
    pub meshes: usize,
    pub surfaces: usize,
    pub breps: usize,
    pub other: usize,
}

impl Top11Context {
    pub fn capture(document: &Document, selection: &BTreeSet<ObjectId>, running: bool) -> Self {
        let mut context = Self {
            running,
            selected: selection.len(),
            curves: 0,
            points: 0,
            meshes: 0,
            surfaces: 0,
            breps: 0,
            other: 0,
        };
        for id in selection {
            match document
                .entities
                .get(id)
                .map(|entity| entity.geometry.kind())
            {
                Some(GeometryKind::Curve) => context.curves += 1,
                Some(GeometryKind::Point) => context.points += 1,
                Some(GeometryKind::Mesh) => context.meshes += 1,
                Some(GeometryKind::Surface) => context.surfaces += 1,
                Some(GeometryKind::Brep) => context.breps += 1,
                Some(_) => context.other += 1,
                None => context.selected = context.selected.saturating_sub(1),
            }
        }
        context
    }
}

/// One toolbar button's admitted state and the reason a disabled button exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Top11Availability {
    pub action: Top11Action,
    pub enabled: bool,
    pub reason: &'static str,
}

impl Top11Availability {
    fn enabled(action: Top11Action) -> Self {
        Self {
            action,
            enabled: true,
            reason: "",
        }
    }

    fn disabled(action: Top11Action, reason: &'static str) -> Self {
        Self {
            action,
            enabled: false,
            reason,
        }
    }
}

/// Admission rule for every Top 11 button.
pub fn availability(action: Top11Action, context: &Top11Context) -> Top11Availability {
    if action == Top11Action::RingRail {
        return Top11Availability::disabled(action, "Ring Rail builder belongs to M04");
    }
    if context.running {
        return Top11Availability::disabled(action, "a command is already running");
    }
    if context.selected == 0 {
        return Top11Availability::disabled(action, "select objects first");
    }
    match action {
        Top11Action::Duplicate => Top11Availability::enabled(action),
        Top11Action::Mirror => {
            if context.curves + context.points + context.meshes + context.surfaces + context.breps
                > 0
            {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(
                    action,
                    "mirror needs points, curves, surfaces, meshes or BReps",
                )
            }
        }
        Top11Action::Move | Top11Action::Rotate => Top11Availability::enabled(action),
        Top11Action::EditPoints => {
            if context.curves + context.surfaces > 0 {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(action, "select a curve or surface first")
            }
        }
        Top11Action::ControlPoints => {
            if context.curves + context.surfaces > 0 {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(action, "select a curve or surface first")
            }
        }
        Top11Action::Explode => {
            if context.curves > 0 && context.other == 0 {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(action, "explode needs curves only")
            }
        }
        Top11Action::Join => {
            if context.curves >= 2 && context.other == 0 {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(action, "join needs two or more curves")
            }
        }
        Top11Action::Split | Top11Action::Trim => {
            if context.curves == 1 && context.selected == 1 {
                Top11Availability::enabled(action)
            } else {
                Top11Availability::disabled(action, "split/trim need exactly one curve")
            }
        }
        Top11Action::RingRail => unreachable!("handled first"),
    }
}

pub fn toolbar(context: &Top11Context) -> [Top11Availability; 11] {
    std::array::from_fn(|index| availability(Top11Action::ALL[index], context))
}

/* ------------------------------ entity helpers ------------------------------ */

fn clone_entity(source: &Entity, geometry: Geometry, placement: Transform) -> Entity {
    let mut entity = source.clone();
    entity.id = ObjectId::new();
    entity.generation = 1;
    entity.geometry_revision = 1;
    entity.attributes_revision = 1;
    entity.geometry = geometry;
    entity.placement = placement;
    entity
}

fn curve_of(document: &Document, id: ObjectId) -> Result<&Curve> {
    match &document
        .entities
        .get(&id)
        .ok_or(Error::Missing("object"))?
        .geometry
    {
        Geometry::Curve(curve) => Ok(curve),
        _ => Err(Error::Unsupported("curve operation needs a curve")),
    }
}

/// Duplicate selected objects in place (the command's `In Place` result).
pub fn duplicate_entities(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    limits: &Limits,
) -> Result<Vec<Entity>> {
    if ids.is_empty() {
        return Err(Error::Invalid("duplicate selection"));
    }
    let mut copies = Vec::new();
    copies
        .try_reserve_exact(ids.len())
        .map_err(|_| Error::Budget("duplicate objects"))?;
    for id in ids {
        let source = document.entities.get(id).ok_or(Error::Missing("object"))?;
        source.geometry.validate(limits)?;
        source.placement.validate()?;
        copies.push(clone_entity(
            source,
            source.geometry.clone(),
            source.placement,
        ));
    }
    Ok(copies)
}

/// Duplicate selected objects with one world translation (the base-point flow).
pub fn duplicate_entities_offset(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    delta: Point3,
    limits: &Limits,
) -> Result<Vec<Entity>> {
    finite(delta)?;
    let mut copies = duplicate_entities(document, ids, limits)?;
    let translation = Transform::translation(delta);
    for copy in &mut copies {
        copy.placement = translation.compose(copy.placement)?;
    }
    Ok(copies)
}

/// Explode every selected polycurve into its members.
pub fn explode_entities(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    limits: &Limits,
) -> Result<Vec<Entity>> {
    let mut pieces = Vec::new();
    for id in ids {
        let source = document.entities.get(id).ok_or(Error::Missing("object"))?;
        let curve = curve_of(document, *id)?;
        let members = curve_ops::explode_curve(curve, limits)?;
        if members.len() < 2 {
            return Err(Error::Unsupported("explode needs a polycurve"));
        }
        pieces
            .try_reserve(members.len())
            .map_err(|_| Error::Budget("exploded objects"))?;
        for member in members {
            member.validate(limits)?;
            pieces.push(clone_entity(
                source,
                Geometry::Curve(member),
                source.placement,
            ));
        }
    }
    if pieces.is_empty() {
        return Err(Error::Invalid("explode selection"));
    }
    Ok(pieces)
}

/// Join selected curves into one curve in the first object's layer/attributes.
pub fn join_entities(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    tolerance: f64,
    limits: &Limits,
) -> Result<Entity> {
    if ids.len() < 2 {
        return Err(Error::Invalid("join selection"));
    }
    let mut source: Option<&Entity> = None;
    let mut placement: Option<Transform> = None;
    let mut curves = Vec::new();
    for id in ids {
        let entity = document.entities.get(id).ok_or(Error::Missing("object"))?;
        let curve = curve_of(document, *id)?.clone();
        if let Some(existing) = placement {
            if existing != entity.placement {
                return Err(Error::Unsupported("join across different placements"));
            }
        } else {
            placement = Some(entity.placement);
            source = Some(entity);
        }
        curves.push(curve);
    }
    let curves = chain_curves(curves, tolerance, limits)?;
    let joined = curve_ops::join_curves(&curves, tolerance, limits)?;
    joined.validate(limits)?;
    let source = source.ok_or(Error::Missing("join source"))?;
    Ok(clone_entity(
        source,
        Geometry::Curve(joined),
        placement.unwrap_or_else(Transform::identity),
    ))
}

fn distance(a: Point3, b: Point3) -> f64 {
    let delta = sub(a, b);
    dot(delta, delta).sqrt()
}

/// Order a selection into one continuous chain, regardless of selection order.
fn chain_curves(mut curves: Vec<Curve>, tolerance: f64, limits: &Limits) -> Result<Vec<Curve>> {
    if curves.len() < 2 {
        return Err(Error::Invalid("join selection"));
    }
    let first = curves.remove(0);
    let mut ordered = vec![first];
    while !curves.is_empty() {
        let end = curve_ops::curve_endpoints(ordered.last().expect("chain"), limits)?[1];
        let mut found = None;
        for (index, curve) in curves.iter().enumerate() {
            let endpoints = curve_ops::curve_endpoints(curve, limits)?;
            if distance(end, endpoints[0]) <= tolerance {
                found = Some((index, false));
                break;
            }
            if distance(end, endpoints[1]) <= tolerance {
                found = Some((index, true));
                break;
            }
        }
        let Some((index, reverse)) = found else {
            break;
        };
        let curve = curves.remove(index);
        ordered.push(if reverse {
            curve_ops::reverse_curve(&curve, limits)?
        } else {
            curve
        });
    }
    // An open chain may have been entered in the middle: extend at the start too.
    while !curves.is_empty() {
        let start = curve_ops::curve_endpoints(ordered.first().expect("chain"), limits)?[0];
        let mut found = None;
        for (index, curve) in curves.iter().enumerate() {
            let endpoints = curve_ops::curve_endpoints(curve, limits)?;
            if distance(start, endpoints[1]) <= tolerance {
                found = Some((index, false));
                break;
            }
            if distance(start, endpoints[0]) <= tolerance {
                found = Some((index, true));
                break;
            }
        }
        let Some((index, reverse)) = found else {
            break;
        };
        let curve = curves.remove(index);
        let curve = if reverse {
            curve_ops::reverse_curve(&curve, limits)?
        } else {
            curve
        };
        ordered.insert(0, curve);
    }
    if !curves.is_empty() {
        return Err(Error::Invalid("join continuity"));
    }
    Ok(ordered)
}

/// Split one curve at a fraction into two objects.
pub fn split_entity(
    document: &Document,
    id: ObjectId,
    fraction: f64,
    limits: &Limits,
) -> Result<Vec<Entity>> {
    let source = document.entities.get(&id).ok_or(Error::Missing("object"))?;
    let curve = curve_of(document, id)?;
    let (left, right) = curve_ops::split_curve(curve, fraction, limits)?;
    left.validate(limits)?;
    right.validate(limits)?;
    Ok(vec![
        clone_entity(source, Geometry::Curve(left), source.placement),
        clone_entity(source, Geometry::Curve(right), source.placement),
    ])
}

/// Trim one curve to the fraction range; the same object identity is kept.
pub fn trim_entity(
    document: &Document,
    id: ObjectId,
    from: f64,
    to: f64,
    limits: &Limits,
) -> Result<Geometry> {
    let curve = curve_of(document, id)?;
    let trimmed = curve_ops::trim_curve(curve, from, to, limits)?;
    trimmed.validate(limits)?;
    Ok(Geometry::Curve(trimmed))
}

/* ------------------------------- transforms ------------------------------- */

fn unit(value: Point3) -> Result<Point3> {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    if !length.is_finite() || length <= 1e-12 {
        return Err(Error::Invalid("direction"));
    }
    Ok([value[0] / length, value[1] / length, value[2] / length])
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Point3, factor: f64) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn translation(offset: Point3) -> Transform {
    Transform::translation(offset)
}

/// Rotation about an arbitrary axis through `center`, right-handed, degrees.
pub fn rotation(center: Point3, axis: Point3, degrees: f64) -> Result<Transform> {
    finite(center)?;
    if !degrees.is_finite() {
        return Err(Error::Invalid("rotation angle"));
    }
    let axis = unit(axis)?;
    let radians = degrees.to_radians();
    let (sine, cosine) = radians.sin_cos();
    let one = 1.0 - cosine;
    let (x, y, z) = (axis[0], axis[1], axis[2]);
    let linear = [
        [
            cosine + x * x * one,
            x * y * one - z * sine,
            x * z * one + y * sine,
        ],
        [
            y * x * one + z * sine,
            cosine + y * y * one,
            y * z * one - x * sine,
        ],
        [
            z * x * one - y * sine,
            z * y * one + x * sine,
            cosine + z * z * one,
        ],
    ];
    let mut matrix = Transform::identity();
    for (row, values) in linear.iter().enumerate() {
        matrix.0[row][..3].copy_from_slice(values);
        matrix.0[row][3] = center[row]
            - (0..3)
                .map(|column| values[column] * center[column])
                .sum::<f64>();
    }
    matrix.validate()?;
    Ok(matrix)
}

/// Move every selected object by one world translation.
pub fn translate_placements(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    delta: Point3,
) -> Result<Vec<(ObjectId, Transform)>> {
    finite(delta)?;
    let mut moved = Vec::new();
    moved
        .try_reserve_exact(ids.len())
        .map_err(|_| Error::Budget("move objects"))?;
    for id in ids {
        let entity = document.entities.get(id).ok_or(Error::Missing("object"))?;
        moved.push((*id, translation(delta).compose(entity.placement)?));
    }
    Ok(moved)
}

/// Rotate every selected object about a world axis through `center`.
pub fn rotate_placements(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    center: Point3,
    axis: Point3,
    degrees: f64,
) -> Result<Vec<(ObjectId, Transform)>> {
    let rotation = rotation(center, axis, degrees)?;
    let mut moved = Vec::new();
    moved
        .try_reserve_exact(ids.len())
        .map_err(|_| Error::Budget("rotate objects"))?;
    for id in ids {
        let entity = document.entities.get(id).ok_or(Error::Missing("object"))?;
        moved.push((*id, rotation.compose(entity.placement)?));
    }
    Ok(moved)
}

fn is_rigid(placement: Transform) -> bool {
    let m = placement.0;
    let columns = [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ];
    for column in columns {
        if (dot(column, column) - 1.0).abs() > 1e-9 {
            return false;
        }
    }
    dot(columns[0], columns[1]).abs() <= 1e-9
        && dot(columns[0], columns[2]).abs() <= 1e-9
        && dot(columns[1], columns[2]).abs() <= 1e-9
}

/// Inverse of a rigid placement (orthonormal rotation/reflection plus translation).
pub fn inverse_affine(placement: Transform) -> Result<Transform> {
    placement.validate()?;
    let m = placement.0;
    let determinant = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if !determinant.is_finite() || determinant.abs() <= 1e-300 {
        return Err(Error::Invalid("singular placement"));
    }
    let inverse_det = 1.0 / determinant;
    let mut inverse = Transform::identity();
    inverse.0[0][0] = (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inverse_det;
    inverse.0[0][1] = (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inverse_det;
    inverse.0[0][2] = (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inverse_det;
    inverse.0[1][0] = (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inverse_det;
    inverse.0[1][1] = (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inverse_det;
    inverse.0[1][2] = (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inverse_det;
    inverse.0[2][0] = (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inverse_det;
    inverse.0[2][1] = (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inverse_det;
    inverse.0[2][2] = (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inverse_det;
    for row in 0..3 {
        inverse.0[row][3] = -(0..3)
            .map(|column| inverse.0[row][column] * m[column][3])
            .sum::<f64>();
    }
    inverse.validate()?;
    Ok(inverse)
}

/// Exact affine image of a circle: a rational quadratic NURBS ellipse. The weights and
/// knots are unchanged by an affine map, so only the Cartesian poles are transformed.
fn affine_circle(
    center: Point3,
    x: Point3,
    y: Point3,
    radius: f64,
    angles: [f64; 2],
    placement: Transform,
) -> Result<Curve> {
    let span = angles[1] - angles[0];
    if !span.is_finite() || span <= 0. || span > std::f64::consts::TAU + 1e-10 {
        return Err(Error::Invalid("affine arc angular domain"));
    }
    let segments = (span / std::f64::consts::FRAC_PI_2).ceil() as usize;
    let mut poles = Vec::with_capacity(segments * 2 + 1);
    let mut weights = Vec::with_capacity(segments * 2 + 1);
    let mut knots = vec![0., 0., 0.];
    for i in 0..segments {
        let a = angles[0] + span * i as f64 / segments as f64;
        let b = angles[0] + span * (i + 1) as f64 / segments as f64;
        let weight = ((b - a) * 0.5).cos();
        for (angle, w) in [(a, 1.), ((a + b) * 0.5, weight), (b, 1.)].into_iter().skip(usize::from(i > 0)) {
            let point = std::array::from_fn(|axis| center[axis] + radius * (angle.cos() * x[axis] + angle.sin() * y[axis]) / w);
            poles.push(placement.point(point)?);
            weights.push(w);
        }
        if i + 1 < segments {
            knots.extend([ (i + 1) as f64 / segments as f64; 2]);
        }
    }
    knots.extend([1., 1., 1.]);
    let curve = NurbsCurve {
        degree: 2,
        poles,
        weights,
        knots,
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    };
    curve.validate(&Limits::default())?;
    Ok(Curve::Nurbs(curve))
}

fn transform_normal(placement: Transform, normal: Point3) -> Result<Point3> {
    let inverse = inverse_affine(placement)?;
    let mut out = [0.0; 3];
    for (row, value) in out.iter_mut().enumerate() {
        *value = (0..3)
            .map(|column| inverse.0[column][row] * normal[column])
            .sum::<f64>();
    }
    finite(out)?;
    let determinant = placement.0[0][0]
        * (placement.0[1][1] * placement.0[2][2] - placement.0[1][2] * placement.0[2][1])
        - placement.0[0][1]
            * (placement.0[1][0] * placement.0[2][2] - placement.0[1][2] * placement.0[2][0])
        + placement.0[0][2]
            * (placement.0[1][0] * placement.0[2][1] - placement.0[1][1] * placement.0[2][0]);
    if determinant < 0.0 {
        out = out.map(|value| -value);
    }
    unit(out)
}

pub fn bake_curve(curve: &Curve, placement: Transform) -> Result<Curve> {
    let vector = |v: Point3| -> Result<Point3> {
        let mut out = [0.0; 3];
        for (row, value) in out.iter_mut().enumerate() {
            *value = (0..3)
                .map(|column| placement.0[row][column] * v[column])
                .sum::<f64>();
        }
        finite(out)?;
        Ok(out)
    };
    Ok(match curve {
        Curve::Line { start, end, .. } => {
            let start = placement.point(*start)?;
            let end = placement.point(*end)?;
            let length = (0..3)
                .map(|axis| (end[axis] - start[axis]).powi(2))
                .sum::<f64>()
                .sqrt();
            Curve::Line {
                start,
                end,
                domain: [0.0, length],
            }
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            if is_rigid(placement) {
                Curve::Circle {
                    center: placement.point(*center)?,
                    x: unit(vector(*x)?)?,
                    y: unit(vector(*y)?)?,
                    radius: *radius,
                    domain: *domain,
                    angle_domain: *angle_domain,
                }
            } else {
                affine_circle(*center, *x, *y, *radius, angle_domain.unwrap_or(*domain), placement)?
            }
        }
        Curve::Nurbs(nurbs) => {
            let mut baked = nurbs.clone();
            for pole in &mut baked.poles {
                *pole = placement.point(*pole)?;
            }
            Curve::Nurbs(baked)
        }
        Curve::Polycurve { segments, reversed } => {
            let mut baked = Vec::new();
            baked
                .try_reserve_exact(segments.len())
                .map_err(|_| Error::Budget("baked polycurve"))?;
            for segment in segments {
                baked.push(bake_curve(segment, placement)?);
            }
            Curve::Polycurve {
                segments: baked,
                reversed: *reversed,
            }
        }
    })
}

/// The geometry with the placement applied, so callers that work in world space (mirror,
/// deformers) can write one Geometry mutation back into the object's own frame.
pub fn bake_geometry(
    geometry: &Geometry,
    placement: Transform,
    limits: &Limits,
) -> Result<Geometry> {
    geometry.validate(limits)?;
    Ok(match geometry {
        Geometry::Point(point) => Geometry::Point(placement.point(*point)?),
        Geometry::Curve(curve) => Geometry::Curve(bake_curve(curve, placement)?),
        Geometry::Surface(surface) => {
            let mut baked = surface.clone();
            for pole in &mut baked.poles {
                *pole = placement.point(*pole)?;
            }
            Geometry::Surface(baked)
        }
        Geometry::Mesh(mesh) => {
            let mut baked = mesh.clone();
            for vertex in &mut baked.vertices {
                *vertex = placement.point(*vertex)?;
            }
            for normal in &mut baked.normals {
                *normal = transform_normal(placement, *normal)?;
            }
            Geometry::Mesh(baked)
        }
        Geometry::Cloud(cloud) => {
            let mut baked = cloud.clone();
            for point in &mut baked.points {
                *point = placement.point(*point)?;
            }
            for normal in &mut baked.normals {
                *normal = transform_normal(placement, *normal)?;
            }
            Geometry::Cloud(baked)
        }
        _ => return Err(Error::Unsupported("mirror for this geometry kind")),
    })
}

fn reflect_point(point: Point3, origin: Point3, normal: Point3) -> Point3 {
    sub(point, scale(normal, 2.0 * dot(sub(point, origin), normal)))
}

fn reflect_direction(direction: Point3, normal: Point3) -> Point3 {
    sub(direction, scale(normal, 2.0 * dot(direction, normal)))
}

fn reflect_geometry(geometry: Geometry, origin: Point3, normal: Point3) -> Result<Geometry> {
    Ok(match geometry {
        Geometry::Point(point) => Geometry::Point(reflect_point(point, origin, normal)),
        Geometry::Curve(curve) => Geometry::Curve(match curve {
            Curve::Line { start, end, .. } => {
                let start = reflect_point(start, origin, normal);
                let end = reflect_point(end, origin, normal);
                let length = (0..3)
                    .map(|axis| (end[axis] - start[axis]).powi(2))
                    .sum::<f64>()
                    .sqrt();
                Curve::Line {
                    start,
                    end,
                    domain: [0.0, length],
                }
            }
            Curve::Circle {
                center,
                x,
                y,
                radius,
                domain,
                angle_domain,
            } => Curve::Circle {
                center: reflect_point(center, origin, normal),
                x: sub(x, scale(normal, 2.0 * dot(x, normal))),
                y: sub(y, scale(normal, 2.0 * dot(y, normal))),
                radius,
                domain,
                angle_domain,
            },
            Curve::Nurbs(mut nurbs) => {
                for pole in &mut nurbs.poles {
                    *pole = reflect_point(*pole, origin, normal);
                }
                Curve::Nurbs(nurbs)
            }
            Curve::Polycurve { segments, reversed } => {
                let mut reflected = Vec::new();
                for segment in segments {
                    reflected.push(
                        match reflect_geometry(Geometry::Curve(segment), origin, normal)? {
                            Geometry::Curve(curve) => curve,
                            _ => return Err(Error::Invalid("mirror polycurve segment")),
                        },
                    );
                }
                Curve::Polycurve {
                    segments: reflected,
                    reversed,
                }
            }
        }),
        Geometry::Surface(mut surface) => {
            for pole in &mut surface.poles {
                *pole = reflect_point(*pole, origin, normal);
            }
            Geometry::Surface(surface)
        }
        Geometry::Mesh(mut mesh) => {
            for vertex in &mut mesh.vertices {
                *vertex = reflect_point(*vertex, origin, normal);
            }
            for direction in &mut mesh.normals {
                *direction = reflect_direction(*direction, normal);
            }
            // A reflection reverses orientation; swapping two corners keeps the
            // outward winding.
            for face in &mut mesh.faces {
                if face.len() >= 3 {
                    face.swap(1, 2);
                }
            }
            Geometry::Mesh(mesh)
        }
        Geometry::Cloud(mut cloud) => {
            for point in &mut cloud.points {
                *point = reflect_point(*point, origin, normal);
            }
            for direction in &mut cloud.normals {
                *direction = reflect_direction(*direction, normal);
            }
            Geometry::Cloud(cloud)
        }
        _ => return Err(Error::Unsupported("mirror for this geometry kind")),
    })
}

/// Result of a mirror: either fresh copies (default) or replacement geometry on the
/// same identities (`copy = false`).
#[derive(Debug, Clone, PartialEq)]
pub struct MirrorOutcome {
    pub copies: Vec<Entity>,
    pub replacements: Vec<(ObjectId, Geometry)>,
}

/// Mirror selected objects about a world plane.
pub fn mirror_entities(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    origin: Point3,
    normal: Point3,
    copy: bool,
    limits: &Limits,
) -> Result<MirrorOutcome> {
    finite(origin)?;
    let normal = unit(normal)?;
    let mut outcome = MirrorOutcome {
        copies: Vec::new(),
        replacements: Vec::new(),
    };
    for id in ids {
        let entity = document.entities.get(id).ok_or(Error::Missing("object"))?;
        let baked = bake_geometry(&entity.geometry, entity.placement, limits)?;
        let reflected = reflect_geometry(baked, origin, normal)?;
        reflected.validate(limits)?;
        if copy {
            outcome
                .copies
                .push(clone_entity(entity, reflected, Transform::identity()));
        } else {
            // Keep the object's placement: express the reflected world geometry back in
            // its local frame so one Geometry write is enough.
            let inverse = inverse_affine(entity.placement)?;
            let local = bake_geometry(&reflected, inverse, limits)?;
            outcome.replacements.push((*id, local));
        }
    }
    Ok(outcome)
}

/* -------------------------------- grips -------------------------------- */

/// Surface edit points: the surface point at each Greville abscissa pair. Periodic and
/// reversed surfaces are included because the grip moves the corresponding pole, not a
/// re-interpolated replacement record.
const MAX_SURFACE_EDIT_GRIPS: usize = 10_000;

fn surface_edit_points(
    surface: &crate::geometry::NurbsSurface,
    limits: &Limits,
) -> Result<Vec<Point3>> {
    let count = surface.size[0]
        .checked_mul(surface.size[1])
        .ok_or(Error::Overflow("surface edit grips"))?;
    if count > MAX_SURFACE_EDIT_GRIPS {
        return Err(Error::Budget(
            "surface edit grips are limited to 10,000 points; use control points",
        ));
    }
    surface_kernel::edit_points(surface, limits)
}

/// The rational basis coefficient of pole `(i, j)` at its own Greville point:
/// `N_i(u_i) N_j(v_j) w_ij / W(u_i, v_j)`. Moving only that pole by `delta / alpha`
/// moves the edit point by exactly `delta` because the weights and denominator do not
/// change.
fn surface_edit_influence(
    surface: &crate::geometry::NurbsSurface,
    i: usize,
    j: usize,
    limits: &Limits,
) -> Result<f64> {
    surface_kernel::edit_influence(surface, i, j, limits)
}

/// One addressable grip: the object, its index in the selected mode and the world point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GripHandle {
    pub object: ObjectId,
    pub index: usize,
    pub mode: GripMode,
    pub point: Point3,
}

/// World-space grips for the selected curves.
pub fn grip_handles(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    mode: GripMode,
    limits: &Limits,
) -> Result<Vec<GripHandle>> {
    let mut handles = Vec::new();
    for id in ids {
        let entity = document.entities.get(id).ok_or(Error::Missing("object"))?;
        let source: Vec<Point3> = match &entity.geometry {
            Geometry::Curve(curve) => {
                let grips = curve_ops::curve_grips(curve, 1e-9, limits)?;
                match mode {
                    GripMode::Edit => grips.edit_points,
                    GripMode::Control => grips.poles,
                }
            }
            Geometry::Surface(surface) => match mode {
                GripMode::Edit => surface_edit_points(surface, limits)?,
                GripMode::Control => surface.poles.clone(),
            },
            _ => return Err(Error::Unsupported("grips need a curve or surface")),
        };
        handles
            .try_reserve(source.len())
            .map_err(|_| Error::Budget("grip points"))?;
        for (index, point) in source.iter().enumerate() {
            handles.push(GripHandle {
                object: *id,
                index,
                mode,
                point: entity.placement.point(*point)?,
            });
        }
    }
    Ok(handles)
}

/// World-space grip points for the selected curves.
pub fn grip_points(
    document: &Document,
    ids: &BTreeSet<ObjectId>,
    mode: GripMode,
    limits: &Limits,
) -> Result<Vec<Point3>> {
    Ok(grip_handles(document, ids, mode, limits)?
        .into_iter()
        .map(|handle| handle.point)
        .collect())
}

/// Move one grip to a world point and return the edited geometry.
pub fn move_grip_geometry(
    document: &Document,
    handle: GripHandle,
    world: Point3,
    limits: &Limits,
) -> Result<Geometry> {
    finite(world)?;
    let entity = document
        .entities
        .get(&handle.object)
        .ok_or(Error::Missing("object"))?;
    let local = inverse_affine(entity.placement)?.point(world)?;
    match &entity.geometry {
        Geometry::Curve(curve) => {
            let edited = match handle.mode {
                GripMode::Control => curve_ops::move_grip(curve, handle.index, local, limits)?,
                GripMode::Edit => curve_ops::move_edit_point(curve, handle.index, local, limits)?,
            };
            edited.validate(limits)?;
            Ok(Geometry::Curve(edited))
        }
        Geometry::Surface(surface) => {
            if handle.index >= surface.poles.len() {
                return Err(Error::Invalid("surface grip index"));
            }
            let mut edited = surface.clone();
            match handle.mode {
                GripMode::Control => {
                    edited.poles[handle.index] = local;
                }
                GripMode::Edit => {
                    let row = handle.index / surface.size[1];
                    let column = handle.index % surface.size[1];
                    let influence = surface_edit_influence(surface, row, column, limits)?;
                    let current = surface_kernel::edit_point(surface, row, column, limits)?;
                    let delta = [
                        (local[0] - current[0]) / influence,
                        (local[1] - current[1]) / influence,
                        (local[2] - current[2]) / influence,
                    ];
                    edited.poles[handle.index] = [
                        surface.poles[handle.index][0] + delta[0],
                        surface.poles[handle.index][1] + delta[1],
                        surface.poles[handle.index][2] + delta[2],
                    ];
                }
            }
            edited.validate(limits)?;
            Ok(Geometry::Surface(edited))
        }
        _ => Err(Error::Unsupported("grips need a curve or surface")),
    }
}

/* ------------------------- split / trim by cutter ------------------------- */

fn intersection_fractions(
    target: &Curve,
    cutter: &Curve,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<f64>> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(Error::Invalid("cutting tolerance"));
    }
    let hits = curve_surface_ops::curve_curve_intersections(target, cutter, tolerance, limits)?;
    let mut fractions: Vec<f64> = hits
        .into_iter()
        .map(|hit| hit.first_fraction)
        .filter(|fraction| *fraction > 1e-7 && *fraction < 1.0 - 1e-7)
        .collect();
    fractions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    fractions.dedup_by(|a, b| (*a - *b).abs() <= 1e-7);
    if fractions.is_empty() {
        return Err(Error::Invalid("cutting curve has no intersection"));
    }
    Ok(fractions)
}

fn cut_pieces(target: &Curve, fractions: &[f64], limits: &Limits) -> Result<Vec<Curve>> {
    let mut boundaries = Vec::with_capacity(fractions.len() + 2);
    boundaries.push(0.0);
    boundaries.extend_from_slice(fractions);
    boundaries.push(1.0);
    let mut pieces = Vec::new();
    for pair in boundaries.windows(2) {
        if pair[1] - pair[0] <= 1e-9 {
            continue;
        }
        let piece = curve_ops::trim_curve(target, pair[0], pair[1], limits)?;
        if curve_ops::arc_length(&piece, limits)? > 1e-9 {
            pieces.push(piece);
        }
    }
    if pieces.len() < 2 {
        return Err(Error::Invalid("cut did not split the curve"));
    }
    Ok(pieces)
}

/// Split a curve at explicit interior fractions.
pub fn split_at_fractions(
    target: &Curve,
    fractions: &[f64],
    limits: &Limits,
) -> Result<Vec<Curve>> {
    target.validate(limits)?;
    let mut clean: Vec<f64> = fractions
        .iter()
        .copied()
        .filter(|fraction| fraction.is_finite() && *fraction > 1e-7 && *fraction < 1.0 - 1e-7)
        .collect();
    clean.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    clean.dedup_by(|a, b| (*a - *b).abs() <= 1e-7);
    if clean.is_empty() {
        return Err(Error::Invalid("no cutting fraction"));
    }
    cut_pieces(target, &clean, limits)
}

/// Split a target curve at every intersection with a cutting curve.
pub fn split_by_cutter(
    target: &Curve,
    cutter: &Curve,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<Curve>> {
    let fractions = intersection_fractions(target, cutter, tolerance, limits)?;
    split_at_fractions(target, &fractions, limits)
}

/// Remove the interval nearest `pick` and return the surviving pieces.
pub fn trim_by_fractions(
    target: &Curve,
    fractions: &[f64],
    pick: Point3,
    limits: &Limits,
) -> Result<Vec<Curve>> {
    finite(pick)?;
    target.validate(limits)?;
    let mut fractions: Vec<f64> = fractions
        .iter()
        .copied()
        .filter(|fraction| fraction.is_finite() && *fraction > 1e-7 && *fraction < 1.0 - 1e-7)
        .collect();
    fractions.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    fractions.dedup_by(|a, b| (*a - *b).abs() <= 1e-7);
    if fractions.is_empty() {
        return Err(Error::Invalid("no cutting fraction"));
    }
    let mut boundaries = Vec::with_capacity(fractions.len() + 2);
    boundaries.push(0.0);
    boundaries.extend_from_slice(&fractions);
    boundaries.push(1.0);
    let mut remove = None;
    let mut best = f64::INFINITY;
    for (index, pair) in boundaries.windows(2).enumerate() {
        let middle = 0.5 * (pair[0] + pair[1]);
        let point = curve_ops::point_at(target, middle, limits)?;
        let measured = distance(point, pick);
        if measured < best {
            best = measured;
            remove = Some(index);
        }
    }
    let remove = remove.ok_or(Error::Invalid("trim interval"))?;
    let mut pieces = Vec::new();
    for (index, pair) in boundaries.windows(2).enumerate() {
        if index == remove || pair[1] - pair[0] <= 1e-9 {
            continue;
        }
        let piece = curve_ops::trim_curve(target, pair[0], pair[1], limits)?;
        if curve_ops::arc_length(&piece, limits)? > 1e-9 {
            pieces.push(piece);
        }
    }
    if pieces.is_empty() {
        return Err(Error::Invalid("trim removed the whole curve"));
    }
    Ok(pieces)
}

/// Remove the interval nearest `pick` after intersecting with a cutting curve.
pub fn trim_by_cutter(
    target: &Curve,
    cutter: &Curve,
    pick: Point3,
    tolerance: f64,
    limits: &Limits,
) -> Result<Vec<Curve>> {
    let fractions = intersection_fractions(target, cutter, tolerance, limits)?;
    trim_by_fractions(target, &fractions, pick, limits)
}

/* ------------------------------- defaults ------------------------------- */

/// The duration unit used by the quick trim variant (10% off each end).
pub const QUICK_TRIM: [f64; 2] = [0.1, 0.9];

/// Split fraction used by the quick Split button.
pub const QUICK_SPLIT: f64 = 0.5;
