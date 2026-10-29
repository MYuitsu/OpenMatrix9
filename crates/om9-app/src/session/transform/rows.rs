// SPDX-License-Identifier: LGPL-2.1-or-later
//! Transform rows whose frame comes from a curve or a surface, plus mirror, stretch and the
//! nonlinear deformers (bend, taper, twist, shear, maelstrom, soft move).
//!
//! These rows were still visible-and-disabled while their owner services sat unused in
//! `om9-core`: `curve_ops` for the curve frame, `surface_kernel`/`curve_surface_ops` for the
//! surface frame and projection, `top11::mirror_entities` for the mirror plane and
//! `gumball::scaling_along` for the one-direction stretch. Nothing here invents geometry: a
//! row is only enabled when the owner it calls can answer, and the selection rule for a
//! reference row is the same one `OrientOnCrv` uses (exactly one reference curve or surface,
//! plus the objects to transform).
use super::*;
use om9_core::{curve_surface_ops, geometry::NurbsSurface, surface_kernel};

/// Matrix `Transform > Orient on Surface` (OM9-TRANSFORM-013, the row is `L3-04-01-015`).
pub const ORIENT_SURFACE: &str = "OM9-TRANSFORM-013";
/// Matrix `Transform > Orient on Curve` (OM9-TRANSFORM-017): the object's X axis follows
/// the path tangent, where `OrientPerp` stands its normal on the tangent.
pub const ORIENT_ON_CURVE: &str = "OM9-TRANSFORM-017";
/// Matrix `Transform > Symmetry` (OM9-TRANSFORM-018).
pub const SYMMETRY: &str = "OM9-TRANSFORM-018";
/// Matrix `Transform > Array Along Surface` (OM9-TRANSFORM-023, the row is `L3-04-01-024`).
pub const ARRAY_SURFACE: &str = "OM9-TRANSFORM-023";
/// Matrix `Transform > Array Curve on Surface` (OM9-TRANSFORM-026, `L3-04-01-025`).
pub const ARRAY_CURVE_SURFACE: &str = "OM9-TRANSFORM-026";
/// Matrix `Transform > Stretch` (OM9-TRANSFORM-032).
pub const STRETCH: &str = "OM9-TRANSFORM-032";
/// Matrix `Transform > Bend` (OM9-TRANSFORM-007, the row is `L3-04-01-008`).
pub const BEND: &str = "OM9-TRANSFORM-007";
/// Matrix `Transform > Taper` (OM9-TRANSFORM-033).
pub const TAPER: &str = "OM9-TRANSFORM-033";
/// Matrix `Transform > Shear` (OM9-TRANSFORM-034).
pub const SHEAR: &str = "OM9-TRANSFORM-034";
/// Matrix `Transform > Maelstrom` (OM9-TRANSFORM-035, the row is `L3-04-01-036`).
pub const MAELSTROM: &str = "OM9-TRANSFORM-035";
/// Matrix `Transform > Twist` (OM9-TRANSFORM-036, the row is `L3-04-01-035`).
pub const TWIST: &str = "OM9-TRANSFORM-036";
/// Matrix `Transform > Soft Move` (OM9-TRANSFORM-040).
pub const SOFT_MOVE: &str = "OM9-TRANSFORM-040";
/// Matrix `Transform > Flow Along Curve` (OM9-TRANSFORM-027).
pub const FLOW_CURVE: &str = "OM9-TRANSFORM-027";
/// Matrix `Transform > Flow Along Surface` (OM9-TRANSFORM-028).
pub const FLOW_SURFACE: &str = "OM9-TRANSFORM-028";
/// Matrix `Transform > Splop` (OM9-TRANSFORM-029).
pub const SPLOP: &str = "OM9-TRANSFORM-029";
/// Matrix `Transform > MoveUVN` (OM9-TRANSFORM-039).
pub const MOVE_UVN: &str = "OM9-TRANSFORM-039";

/// The rows this module runs, in menu order.
pub const FEATURES: [&str; 23] = [
    BEND,
    ORIENT_SURFACE,
    ORIENT_ON_CURVE,
    SYMMETRY,
    ARRAY_SURFACE,
    ARRAY_CURVE_SURFACE,
    STRETCH,
    TAPER,
    SHEAR,
    TWIST,
    MAELSTROM,
    SOFT_MOVE,
    FLOW_CURVE,
    FLOW_SURFACE,
    SPLOP,
    MOVE_UVN,
    CAGE,
    CAGE_EDIT,
    RELEASE_CAGE,
    SMART_FLOW,
    SMART_FLOW_RIGID,
    SMART_FLOW_BREAK,
    SMART_FLOW_PULLDOWN,
];

/// The rows that own or read a Smart Flow link (owner `om9_core::deform`).
pub fn flow_link_row(feature: &str) -> bool {
    self::flow::handles(feature)
}

/// The rows that create, edit and release a cage (owner `om9_core::cage`).
pub fn cage_row(feature: &str) -> bool {
    self::cage::handles(feature)
}

/// The rows that edit a surface's own control net: the surface is the object, not a frame.
pub fn surface_edit_row(feature: &str) -> bool {
    feature == MOVE_UVN
}

/// The rows whose two frames come from the selection (source and target).
pub fn flow_row(feature: &str) -> bool {
    matches!(feature, FLOW_CURVE | FLOW_SURFACE)
}

pub fn handles(feature: &str) -> bool {
    FEATURES.contains(&feature)
}

/// The rows whose reference surface comes from the selection.
pub fn surface_row(feature: &str) -> bool {
    matches!(
        feature,
        ORIENT_SURFACE | ARRAY_SURFACE | ARRAY_CURVE_SURFACE | SPLOP
    )
}

/// The rows whose reference curve comes from the selection.
pub fn curve_row(feature: &str) -> bool {
    matches!(feature, ORIENT_ON_CURVE | ARRAY_CURVE_SURFACE)
}

/// The documented input sequence of the rows in this module.
pub fn fields(feature: &str) -> &'static [Field] {
    const ORIENT_SURFACE_FIELDS: &[Field] = &[Field::Point("Point on surface to orient at: ")];
    const ARRAY_SURFACE_FIELDS: &[Field] = &[
        Field::Number("Number of rows: "),
        Field::Number("Number of columns: "),
        Field::Number("Orient copies to the surface normal (0 = no, 1 = yes): "),
    ];
    const ARRAY_CURVE_SURFACE_FIELDS: &[Field] = &[
        Field::Number("Number of items: "),
        Field::Number("Orient copies to the surface normal (0 = no, 1 = yes): "),
    ];
    const STRETCH_FIELDS: &[Field] = &[
        Field::Point("Base point for stretch: "),
        Field::Point("Stretch direction: "),
        Field::Number("Stretch factor: "),
    ];
    const ORIENT_ON_CURVE_FIELDS: &[Field] = &[Field::Point("Point on curve to orient at: ")];
    const BEND_FIELDS: &[Field] = &[
        Field::Point("Base point for bend: "),
        Field::Point("Bend axis end point: "),
        Field::Point("Point in the bend direction: "),
        Field::Number("Bend angle in degrees: "),
    ];
    const TAPER_FIELDS: &[Field] = &[
        Field::Point("Base point for taper: "),
        Field::Point("Taper axis end point: "),
        Field::Number("Taper factor at the far end: "),
    ];
    const SHEAR_FIELDS: &[Field] = &[
        Field::Point("Base point for shear: "),
        Field::Point("Point along the measure direction: "),
        Field::Point("Point along the shear direction: "),
        Field::Number("Shear factor: "),
    ];
    const TWIST_FIELDS: &[Field] = &[
        Field::Point("Base point for twist: "),
        Field::Point("Twist axis end point: "),
        Field::Number("Twist angle in degrees: "),
    ];
    const MAELSTROM_FIELDS: &[Field] = &[
        Field::Point("Base point for maelstrom: "),
        Field::Point("Maelstrom axis end point: "),
        Field::Number("Maelstrom angle in degrees: "),
        Field::Number("Pull between 0 and 1: "),
    ];
    const SOFT_MOVE_FIELDS: &[Field] = &[
        Field::Point("Base point for soft move: "),
        Field::Point("Displacement point: "),
        Field::Number("Falloff radius: "),
    ];
    const SYMMETRY_FIELDS: &[Field] = &[
        Field::Point("Mirror plane origin: "),
        Field::Point("Mirror plane normal point: "),
        Field::Number("Copy mirrored objects (0 = no, 1 = yes): "),
    ];
    const NO_FIELDS: &[Field] = &[];
    const MOVE_UVN_FIELDS: &[Field] = &[
        Field::Number("Move in the surface normal direction: "),
        Field::Number("Move along the U direction: "),
        Field::Number("Move along the V direction: "),
    ];
    const FLOW_CURVE_FIELDS: &[Field] = &[Field::Point("Point on the source curve: ")];
    const FLOW_SURFACE_FIELDS: &[Field] = &[Field::Point("Point on the source surface: ")];
    match feature {
        ORIENT_SURFACE => ORIENT_SURFACE_FIELDS,
        ORIENT_ON_CURVE => ORIENT_ON_CURVE_FIELDS,
        ARRAY_SURFACE => ARRAY_SURFACE_FIELDS,
        ARRAY_CURVE_SURFACE => ARRAY_CURVE_SURFACE_FIELDS,
        STRETCH => STRETCH_FIELDS,
        SYMMETRY => SYMMETRY_FIELDS,
        BEND => BEND_FIELDS,
        TAPER => TAPER_FIELDS,
        SHEAR => SHEAR_FIELDS,
        TWIST => TWIST_FIELDS,
        MAELSTROM => MAELSTROM_FIELDS,
        SOFT_MOVE => SOFT_MOVE_FIELDS,
        FLOW_CURVE => FLOW_CURVE_FIELDS,
        FLOW_SURFACE => FLOW_SURFACE_FIELDS,
        SPLOP => NO_FIELDS,
        MOVE_UVN => MOVE_UVN_FIELDS,
        CAGE | CAGE_EDIT | RELEASE_CAGE => self::cage::fields(feature),
        SMART_FLOW | SMART_FLOW_RIGID | SMART_FLOW_BREAK | SMART_FLOW_PULLDOWN => {
            self::flow::fields(feature)
        }
        _ => &[],
    }
}

/// A reflection about the plane through `origin` with unit `normal`, written as the affine
/// map `x -> x - 2 (n . (x - origin)) n`. A placement-only mirror keeps every geometry kind,
/// which is why the row does not depend on a geometry-specific mirror routine.
fn reflection(origin: Point3, normal: Point3) -> Result<Transform> {
    let mut matrix = Transform::identity().0;
    for row in 0..3 {
        for column in 0..3 {
            matrix[row][column] -= 2.0 * normal[row] * normal[column];
        }
    }
    let offset = 2.0 * dot(normal, origin);
    for row in 0..3 {
        matrix[row][3] = offset * normal[row];
    }
    let transform = Transform(matrix);
    transform.validate()?;
    Ok(transform)
}

/// One documented `0`/`1` switch, accepted as a number because the typed contract of this
/// module is points and scalars only.
fn flag(value: Option<f64>, name: &'static str) -> Result<bool> {
    let value = value.ok_or(Error::Missing(name))?;
    if (value - 0.0).abs() <= f64::EPSILON {
        return Ok(false);
    }
    if (value - 1.0).abs() <= f64::EPSILON {
        return Ok(true);
    }
    Err(Error::Invalid(name))
}

/// The one selected surface plus the objects to transform, in document order.
pub(super) fn surface_selection(controller: &Controller) -> Result<(NurbsSurface, Vec<ObjectId>)> {
    let document = controller.document();
    let mut surface: Option<NurbsSurface> = None;
    let mut targets: Vec<ObjectId> = Vec::new();
    let mut ids: Vec<ObjectId> = controller.selection.iter().copied().collect();
    ids.sort();
    for id in ids {
        let entity = document
            .entities
            .get(&id)
            .ok_or(Error::Missing("transform object"))?;
        match &entity.geometry {
            Geometry::Surface(found) => {
                if surface.is_some() {
                    return Err(Error::Invalid("one reference surface"));
                }
                surface = Some(found.clone());
            }
            _ => targets.push(id),
        }
    }
    let surface = surface.ok_or(Error::Missing("reference surface selection"))?;
    if targets.is_empty() {
        return Err(Error::Missing(
            "select the objects and the reference surface together",
        ));
    }
    Ok((surface, targets))
}

/// The one selected surface, the one selected curve and the objects to transform.
fn curve_and_surface(controller: &Controller) -> Result<(Curve, NurbsSurface, Vec<ObjectId>)> {
    let (curve, others) = controller.curve_path()?;
    let document = controller.document();
    let mut surface: Option<NurbsSurface> = None;
    let mut targets: Vec<ObjectId> = Vec::new();
    for id in others {
        let entity = document
            .entities
            .get(&id)
            .ok_or(Error::Missing("transform object"))?;
        match &entity.geometry {
            Geometry::Surface(found) => {
                if surface.is_some() {
                    return Err(Error::Invalid("one reference surface"));
                }
                surface = Some(found.clone());
            }
            _ => targets.push(id),
        }
    }
    let surface = surface.ok_or(Error::Missing("reference surface selection"))?;
    if targets.is_empty() {
        return Err(Error::Missing(
            "select the objects, the curve and the reference surface together",
        ));
    }
    Ok((curve, surface, targets))
}

fn unit(vector: Point3, name: &'static str) -> Result<Point3> {
    let magnitude = length(vector);
    if !magnitude.is_finite() || magnitude <= 1.0e-12 {
        return Err(Error::Invalid(name));
    }
    Ok(scale(vector, 1.0 / magnitude))
}

/// The rotation about `origin` that turns the CPlane normal onto a reference normal. The
/// caller composes the move to the station, so an object is oriented without being moved
/// twice.
fn stand_at(origin: Point3, up: Point3, target: Point3) -> Result<Transform> {
    om9_core::transform_session::align_transform(
        &om9_core::transform_session::AlignRequest::rotate(origin, origin, up, target),
    )
}

/// Run one row of this module, or answer `false` when the feature belongs to another module.
pub(super) fn finish(controller: &mut Controller, pending: &TransformPending) -> Result<bool> {
    if !handles(pending.feature) {
        return Ok(false);
    }
    let frame = plane_frame(&pending.plane);
    let name = crate::transform_menu::by_feature(pending.feature)
        .map(|binding| binding.label)
        .unwrap_or("Transform");
    match pending.feature {
        ORIENT_ON_CURVE => {
            let (curve, targets) = controller.curve_path()?;
            let limits = controller.coordinator.limits();
            let picked = point(pending.points.first(), "point on curve")?;
            let fraction = nearest_fraction(&curve, picked, limits)?;
            let station = curve_ops::point_at(&curve, fraction, limits)?;
            let tangent = curve_ops::tangent_at(&curve, fraction, limits)?;
            let mut steps = Vec::new();
            for target in &targets {
                let centre = controller.object_center(*target)?;
                steps.push(om9_core::transform_session::align_transform(
                    &om9_core::transform_session::AlignRequest::rotate(
                        centre,
                        station,
                        frame.x,
                        unit(tangent, "curve tangent")?,
                    ),
                )?);
            }
            controller.commit_transform_mutations(name, targets, steps, false)?;
        }
        ORIENT_SURFACE => {
            let (surface, targets) = surface_selection(controller)?;
            let limits = controller.coordinator.limits();
            let picked = point(pending.points.first(), "point on surface")?;
            let projection = curve_surface_ops::closest_point_on_surface(&surface, picked, limits)?;
            let normal = unit(
                surface_kernel::normal(
                    &surface,
                    projection.parameters[0],
                    projection.parameters[1],
                    limits,
                )?,
                "surface normal",
            )?;
            let mut steps = Vec::new();
            for target in &targets {
                let centre = controller.object_center(*target)?;
                steps.push(om9_core::transform_session::align_transform(
                    &om9_core::transform_session::AlignRequest::rotate(
                        centre,
                        projection.point,
                        frame.z,
                        normal,
                    ),
                )?);
            }
            controller.commit_transform_mutations(name, targets, steps, false)?;
        }
        ARRAY_SURFACE => {
            let (surface, _) = surface_selection(controller)?;
            let limits = controller.coordinator.limits();
            let rows = count(pending.numbers.first().copied(), "array rows")?;
            let columns = count(pending.numbers.get(1).copied(), "array columns")?;
            let orient = flag(
                pending.numbers.get(2).copied(),
                "surface normal orientation",
            )?;
            if rows.saturating_mul(columns) > MAX_CURVE_ITEMS {
                return Err(Error::Invalid("array count"));
            }
            let domain = surface.domain;
            let anchor = surface_kernel::evaluate(&surface, domain[0][0], domain[1][0], limits)?;
            let mut steps = Vec::new();
            for row in 0..rows {
                for column in 0..columns {
                    if row == 0 && column == 0 {
                        continue;
                    }
                    let u = domain[0][0]
                        + (domain[0][1] - domain[0][0]) * row as f64 / (rows - 1) as f64;
                    let v = domain[1][0]
                        + (domain[1][1] - domain[1][0]) * column as f64 / (columns - 1) as f64;
                    let station = surface_kernel::evaluate(&surface, u, v, limits)?;
                    let mut step = Transform::translation(subtract(station, anchor));
                    if orient {
                        let normal = unit(
                            surface_kernel::normal(&surface, u, v, limits)?,
                            "surface normal",
                        )?;
                        let rotation = stand_at(anchor, frame.z, normal)?;
                        step = step.compose(rotation)?;
                    }
                    steps.push(step);
                }
            }
            let targets = controller.transform_targets()?;
            controller.commit_transform_mutations(name, targets, steps, true)?;
        }
        ARRAY_CURVE_SURFACE => {
            let (curve, surface, _) = curve_and_surface(controller)?;
            let limits = controller.coordinator.limits();
            let count = count(pending.numbers.first().copied(), "array count")?;
            let orient = flag(
                pending.numbers.get(1).copied(),
                "surface normal orientation",
            )?;
            if count > MAX_CURVE_ITEMS {
                return Err(Error::Invalid("array count"));
            }
            let total = curve_ops::arc_length(&curve, limits)?;
            if !total.is_finite() || total <= 1.0e-9 {
                return Err(Error::Invalid("array path length"));
            }
            let anchor = curve_surface_ops::closest_point_on_surface(
                &surface,
                curve_ops::point_at(&curve, 0.0, limits)?,
                limits,
            )?
            .point;
            let mut steps = Vec::new();
            for index in 1..count {
                let station = total * index as f64 / (count - 1) as f64;
                let fraction = curve_ops::parameter_at_length(&curve, station, limits)?;
                let on_curve = curve_ops::point_at(&curve, fraction, limits)?;
                let projection =
                    curve_surface_ops::closest_point_on_surface(&surface, on_curve, limits)?;
                let mut step = Transform::translation(subtract(projection.point, anchor));
                if orient {
                    let normal = unit(
                        surface_kernel::normal(
                            &surface,
                            projection.parameters[0],
                            projection.parameters[1],
                            limits,
                        )?,
                        "surface normal",
                    )?;
                    let rotation = stand_at(anchor, frame.z, normal)?;
                    step = step.compose(rotation)?;
                }
                steps.push(step);
            }
            let targets = controller.transform_targets()?;
            controller.commit_transform_mutations(name, targets, steps, true)?;
        }
        STRETCH => {
            let base = point(pending.points.first(), "stretch base point")?;
            let direction = unit(
                subtract(point(pending.points.get(1), "stretch direction")?, base),
                "stretch direction",
            )?;
            let factor = positive(pending.numbers.first().copied(), "stretch factor")?;
            let targets = controller.transform_targets()?;
            controller.commit_transform_mutations(
                name,
                targets,
                vec![gumball::scaling_along(base, direction, factor)?],
                false,
            )?;
        }
        SYMMETRY => {
            let origin = point(pending.points.first(), "mirror plane origin")?;
            let normal_point = point(pending.points.get(1), "mirror plane normal point")?;
            let normal = unit(subtract(normal_point, origin), "mirror plane normal")?;
            let copy = flag(pending.numbers.first().copied(), "mirror copy switch")?;
            let targets = controller.transform_targets()?;
            controller.commit_transform_mutations(
                name,
                targets,
                vec![reflection(origin, normal)?],
                copy,
            )?;
        }
        BEND | TAPER | TWIST | SHEAR | MAELSTROM | SOFT_MOVE => {
            let deform = deform_from(pending)?;
            let limits = *controller.coordinator.limits();
            let targets = controller.transform_targets()?;
            commit_geometry(controller, name, targets, &mut |geometry, placement| {
                om9_core::deform::apply_placed(&deform, geometry, placement, &limits)
            })?;
        }
        FLOW_CURVE => {
            let limits = *controller.coordinator.limits();
            let picked = point(pending.points.first(), "point on the source curve")?;
            let (source, target, targets) = flow_curves(controller, picked, &limits)?;
            commit_geometry(controller, name, targets, &mut |geometry, placement| {
                om9_core::deform::flow_along_curve(&source, &target, geometry, placement, &limits)
            })?;
        }
        FLOW_SURFACE => {
            let limits = *controller.coordinator.limits();
            let picked = point(pending.points.first(), "point on the source surface")?;
            let (source, target, targets) = flow_surfaces(controller, picked, &limits)?;
            commit_geometry(controller, name, targets, &mut |geometry, placement| {
                om9_core::deform::flow_along_surface(&source, &target, geometry, placement, &limits)
            })?;
        }
        MOVE_UVN => {
            let normal = offset(pending.numbers.first().copied(), "normal offset")?;
            let u = offset(pending.numbers.get(1).copied(), "U offset")?;
            let v = offset(pending.numbers.get(2).copied(), "V offset")?;
            if normal == 0.0 && u == 0.0 && v == 0.0 {
                return Err(Error::Invalid("MoveUVN needs one non-zero offset"));
            }
            let limits = *controller.coordinator.limits();
            let surfaces = edit_surfaces(controller)?;
            let mut mutations = Vec::with_capacity(surfaces.len());
            for (id, surface) in surfaces {
                let source = controller.document().stamp(id)?;
                mutations.push(Mutation::Geometry {
                    source,
                    geometry: Geometry::Surface(om9_core::deform::move_uvn(
                        &surface, normal, u, v, &limits,
                    )?),
                });
            }
            controller.commit_utility_mutations(name, mutations)?;
        }
        SPLOP => {
            let (surface, targets) = surface_selection(controller)?;
            let limits = *controller.coordinator.limits();
            commit_geometry(controller, name, targets, &mut |geometry, placement| {
                om9_core::deform::splop(&surface, geometry, placement, &limits)
            })?;
        }
        CAGE => self::cage::create(controller, name)?,
        CAGE_EDIT => {
            let index = corner_index(pending)?;
            let offset = point(pending.points.first(), "corner offset")?;
            self::cage::edit(controller, name, index, offset)?;
        }
        RELEASE_CAGE => self::cage::release(controller, name)?,
        SMART_FLOW | SMART_FLOW_RIGID => {
            let picked = point(pending.points.first(), "point on the base surface")?;
            let rigid = pending.feature == SMART_FLOW_RIGID;
            self::flow::flow(controller, name, picked, rigid)?;
        }
        SMART_FLOW_BREAK => self::flow::break_flow(controller, name)?,
        SMART_FLOW_PULLDOWN => {
            let picked = point(pending.points.first(), "point on the base surface")?;
            self::flow::pulldown(controller, name, picked)?;
        }
        other => return Err(Error::Unsupported(other)),
    }
    controller.finish_transform_events(pending)?;
    Ok(true)
}

/// The deform a finished run asks for, built from the points and numbers it collected. The
/// points are already world coordinates through the captured CPlane, so an axis is simply the
/// vector between the two points the row asked for.
fn deform_from(pending: &TransformPending) -> Result<om9_core::deform::Deform> {
    use om9_core::deform::Deform;
    let at = |index: usize, name: &'static str| -> Result<Point3> {
        point(pending.points.get(index), name)
    };
    let deform = match pending.feature {
        BEND => {
            let base = at(0, "bend base point")?;
            let axis = unit(subtract(at(1, "bend axis end point")?, base), "bend axis")?;
            let bend = unit(
                subtract(at(2, "bend direction point")?, base),
                "bend direction",
            )?;
            Deform::Bend {
                base,
                axis,
                bend,
                angle_degrees: finite(pending.numbers.first().copied(), "bend angle")?,
            }
        }
        TAPER => Deform::Taper {
            base: at(0, "taper base point")?,
            axis: unit(
                subtract(at(1, "taper axis end point")?, at(0, "taper base point")?),
                "taper axis",
            )?,
            factor: positive(pending.numbers.first().copied(), "taper factor")?,
        },
        TWIST => Deform::Twist {
            base: at(0, "twist base point")?,
            axis: unit(
                subtract(at(1, "twist axis end point")?, at(0, "twist base point")?),
                "twist axis",
            )?,
            angle_degrees: finite(pending.numbers.first().copied(), "twist angle")?,
        },
        SHEAR => Deform::Shear {
            base: at(0, "shear base point")?,
            measure: unit(
                subtract(at(1, "shear measure point")?, at(0, "shear base point")?),
                "shear measure direction",
            )?,
            axis: unit(
                subtract(at(2, "shear direction point")?, at(0, "shear base point")?),
                "shear direction",
            )?,
            factor: finite(pending.numbers.first().copied(), "shear factor")?,
        },
        MAELSTROM => Deform::Maelstrom {
            base: at(0, "maelstrom base point")?,
            axis: unit(
                subtract(
                    at(1, "maelstrom axis end point")?,
                    at(0, "maelstrom base point")?,
                ),
                "maelstrom axis",
            )?,
            angle_degrees: finite(pending.numbers.first().copied(), "maelstrom angle")?,
            pull: pull(pending.numbers.get(1).copied())?,
        },
        SOFT_MOVE => {
            let base = at(0, "soft move base point")?;
            Deform::SoftMove {
                base,
                displacement: subtract(at(1, "soft move displacement point")?, base),
                radius: positive(pending.numbers.first().copied(), "falloff radius")?,
            }
        }
        other => return Err(Error::Unsupported(other)),
    };
    deform.validate()?;
    Ok(deform)
}

/// A number that may be zero but must be finite is not useful for an angle or a factor, so
/// the rows reject zero and a NaN the same way the built-in rows do.
fn finite(value: Option<f64>, name: &'static str) -> Result<f64> {
    let value = value.ok_or(Error::Missing(name))?;
    if !value.is_finite() || value == 0.0 {
        return Err(Error::Invalid(name));
    }
    Ok(value)
}

/// The maelstrom's radial pull: a share between 0 and 1, where 1 would collapse the section.
fn pull(value: Option<f64>) -> Result<f64> {
    let value = value.ok_or(Error::Missing("pull"))?;
    if !value.is_finite() || !(0.0..1.0).contains(&value) {
        return Err(Error::Invalid("pull between 0 and 1"));
    }
    Ok(value)
}

/// Validate one typed number of a row in this module; `None` when the row is not ours.
pub(super) fn check_number(feature: &str, index: usize, value: f64) -> Option<Result<()>> {
    let name = match (feature, index) {
        (BEND, 0) => "bend angle",
        (TWIST, 0) => "twist angle",
        (MAELSTROM, 0) => "maelstrom angle",
        (SHEAR, 0) => "shear factor",
        (TAPER, 0) => "taper factor",
        (SOFT_MOVE, 0) => "falloff radius",
        (MAELSTROM, 1) => "pull between 0 and 1",
        (MOVE_UVN, 0..=2) => "uvn offset",
        _ => return None,
    };
    Some(match name {
        "taper factor" | "falloff radius" => positive(Some(value), name).map(|_| ()),
        "pull between 0 and 1" => pull(Some(value)).map(|_| ()),
        "uvn offset" => finite_any(Some(value), "uvn offset").map(|_| ()),
        _ => finite(Some(value), name).map(|_| ()),
    })
}

/// Write one deformed geometry per object in a single transaction: the map runs in world space
/// and `Mutation::Geometry` stores the result back in the object's own frame.
fn commit_geometry(
    controller: &mut Controller,
    name: &'static str,
    targets: Vec<ObjectId>,
    map: &mut dyn FnMut(&Geometry, &Transform) -> Result<Geometry>,
) -> Result<()> {
    let mut mutations = Vec::with_capacity(targets.len());
    for target in targets {
        let source = controller.document().stamp(target)?;
        let entity = controller
            .document()
            .entities
            .get(&target)
            .ok_or(Error::Missing("deform object"))?;
        let geometry = entity.geometry.clone();
        let placement = entity.placement;
        mutations.push(Mutation::Geometry {
            source,
            geometry: map(&geometry, &placement)?,
        });
    }
    controller.commit_utility_mutations(name, mutations)
}

/// The selected curves and the objects to flow: exactly two curves are the frames.
fn flow_frames_curves(controller: &Controller) -> Result<(Vec<Curve>, Vec<ObjectId>)> {
    let mut curves = Vec::new();
    let mut targets = Vec::new();
    let mut ids: Vec<ObjectId> = controller.selection.iter().copied().collect();
    ids.sort();
    for id in ids {
        let entity = controller
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("transform object"))?;
        match &entity.geometry {
            Geometry::Curve(curve) => curves.push(curve.clone()),
            _ => targets.push(id),
        }
    }
    if curves.len() != 2 {
        return Err(Error::Missing(
            "select the source curve, the target curve and the objects together",
        ));
    }
    if targets.is_empty() {
        return Err(Error::Missing(
            "select the objects to flow with the two curves",
        ));
    }
    Ok((curves, targets))
}

/// The two selected curves with the source first: the user names the source by picking a point
/// on it, because the selection is a set and has no pick order.
pub(super) fn flow_curves(
    controller: &Controller,
    picked: Point3,
    limits: &Limits,
) -> Result<(Curve, Curve, Vec<ObjectId>)> {
    let (mut curves, targets) = flow_frames_curves(controller)?;
    let first = curve_surface_ops::closest_point_on_curve(&curves[0], picked, limits)?.2;
    let second = curve_surface_ops::closest_point_on_curve(&curves[1], picked, limits)?.2;
    if second < first {
        curves.swap(0, 1);
    }
    let mut curves = curves.into_iter();
    let source = curves.next().ok_or(Error::Missing("source curve"))?;
    let target = curves.next().ok_or(Error::Missing("target curve"))?;
    Ok((source, target, targets))
}

/// The selected surfaces and the objects to flow: exactly two surfaces are the frames.
fn flow_frames_surfaces(controller: &Controller) -> Result<(Vec<NurbsSurface>, Vec<ObjectId>)> {
    let mut surfaces = Vec::new();
    let mut targets = Vec::new();
    let mut ids: Vec<ObjectId> = controller.selection.iter().copied().collect();
    ids.sort();
    for id in ids {
        let entity = controller
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("transform object"))?;
        match &entity.geometry {
            Geometry::Surface(surface) => surfaces.push(surface.clone()),
            _ => targets.push(id),
        }
    }
    if surfaces.len() != 2 {
        return Err(Error::Missing(
            "select the source surface, the target surface and the objects together",
        ));
    }
    if targets.is_empty() {
        return Err(Error::Missing(
            "select the objects to flow with the two surfaces",
        ));
    }
    Ok((surfaces, targets))
}

/// The two selected surfaces with the source first, named by a point the user picked on it.
pub(super) fn flow_surfaces(
    controller: &Controller,
    picked: Point3,
    limits: &Limits,
) -> Result<(NurbsSurface, NurbsSurface, Vec<ObjectId>)> {
    let (mut surfaces, targets) = flow_frames_surfaces(controller)?;
    let first = curve_surface_ops::closest_point_on_surface(&surfaces[0], picked, limits)?.distance;
    let second =
        curve_surface_ops::closest_point_on_surface(&surfaces[1], picked, limits)?.distance;
    if second < first {
        surfaces.swap(0, 1);
    }
    let mut surfaces = surfaces.into_iter();
    let source = surfaces.next().ok_or(Error::Missing("source surface"))?;
    let target = surfaces.next().ok_or(Error::Missing("target surface"))?;
    Ok((source, target, targets))
}

/// Validate the reference selection of a flow row before the first (and only) step runs.
pub(super) fn flow_selection(controller: &Controller, feature: &str) -> Result<()> {
    match feature {
        FLOW_CURVE => {
            flow_frames_curves(controller)?;
            Ok(())
        }
        FLOW_SURFACE => {
            flow_frames_surfaces(controller)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// A MoveUVN offset: any finite number, and zero is meaningful because the three offsets are
/// independent; the row refuses only the step where all three are zero.
fn offset(value: Option<f64>, name: &'static str) -> Result<f64> {
    finite_any(value, name)
}

/// A finite number that may be zero.
fn finite_any(value: Option<f64>, name: &'static str) -> Result<f64> {
    let value = value.ok_or(Error::Missing(name))?;
    if !value.is_finite() {
        return Err(Error::Invalid(name));
    }
    Ok(value)
}

/// The selected surfaces MoveUVN edits, in document order. The row edits surfaces only, so a
/// solid or a curve in the selection is refused instead of being silently ignored.
fn edit_surfaces(controller: &Controller) -> Result<Vec<(ObjectId, NurbsSurface)>> {
    if controller.selection.is_empty() {
        return Err(Error::Missing("select at least one surface to move"));
    }
    let mut ids: Vec<ObjectId> = controller.selection.iter().copied().collect();
    ids.sort();
    let mut surfaces = Vec::with_capacity(ids.len());
    for id in ids {
        let entity = controller
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("transform object"))?;
        match &entity.geometry {
            Geometry::Surface(surface) => surfaces.push((id, surface.clone())),
            _ => {
                return Err(Error::Invalid(
                    "MoveUVN edits surfaces; select surfaces only",
                ));
            }
        }
    }
    Ok(surfaces)
}

/// Validate the MoveUVN selection before the three questions are asked.
pub(super) fn edit_selection(controller: &Controller, feature: &str) -> Result<()> {
    if surface_edit_row(feature) {
        edit_surfaces(controller)?;
    }
    Ok(())
}

/// The cage corner a `Cage Edit` step names: one-based, as the prompt reads.
fn corner_index(pending: &TransformPending) -> Result<usize> {
    let value = pending
        .numbers
        .first()
        .copied()
        .ok_or(Error::Missing("cage corner"))?;
    if !value.is_finite() || value.fract() != 0.0 || !(1.0..=8.0).contains(&value) {
        return Err(Error::Invalid("cage corner between 1 and 8"));
    }
    Ok(value as usize - 1)
}
