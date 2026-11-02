// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix Transform commands (`OM9-L2-04-01`).
//!
//! The Transform menu is a bridge: every row is a documented Matrix command that must
//! reach the kernel transform service instead of re-implementing matrix math inside a
//! toolbar adapter. This module owns the interactive input (base point, factor, angle,
//! array count) and turns one finished command into **one** transaction, so a transform
//! is a single Undo entry and the scene is rebuilt once.
//!
//! `om9_core::gumball` owns the arithmetic (`scaling_along`, `rotation_about`,
//! `rectangular_array`, `polar_array`) and `om9_core::transform_session` owns the align
//! solve (`AlignRequest`/`align_transform`); nothing here builds a matrix by hand.
//!
//! A row whose owner service is missing keeps its menu row disabled with a recorded
//! reason (`transform_menu::BINDINGS`), so a click can never silently do nothing.
use om9_core::{
    Error, Result,
    budget::Limits,
    curve_ops,
    geometry::{Curve, Geometry, Point3, Transform, Unit},
    gumball::{self, Frame},
    identity::ObjectId,
    transaction::{Mutation, TransactionRequest},
};
use om9_scene::camera::CPlane;

use super::Controller;
mod advanced;
pub use advanced::{CurveOrientation, TransformOptions};
mod cage;
mod flow;
mod rows;
pub use cage::{CAGE, CAGE_EDIT, RELEASE_CAGE};
pub use flow::{SMART_FLOW, SMART_FLOW_BREAK, SMART_FLOW_PULLDOWN, SMART_FLOW_RIGID};
pub use rows::{
    ARRAY_CURVE_SURFACE, ARRAY_SURFACE, BEND, FLOW_CURVE, FLOW_SURFACE, MAELSTROM, MOVE_UVN,
    ORIENT_ON_CURVE, ORIENT_SURFACE, SHEAR, SOFT_MOVE, SPLOP, STRETCH, SYMMETRY, TAPER, TWIST,
};

/// Matrix `Transform > Scale 3-D` (OM9-TRANSFORM-001).
pub const SCALE_3D: &str = "OM9-TRANSFORM-001";
/// Matrix `Transform > Scale 2-D` (OM9-TRANSFORM-002).
pub const SCALE_2D: &str = "OM9-TRANSFORM-002";
/// Matrix `Transform > Scale Non-Uniform` (OM9-TRANSFORM-003).
pub const SCALE_NU: &str = "OM9-TRANSFORM-003";
/// Matrix `Transform > Scale 1-D` (OM9-TRANSFORM-004).
pub const SCALE_1D: &str = "OM9-TRANSFORM-004";
/// Matrix `Transform > Orient 2 Points` (OM9-TRANSFORM-005).
pub const ORIENT_2: &str = "OM9-TRANSFORM-005";
/// Matrix `Transform > Orient 3 Points` (OM9-TRANSFORM-006).
pub const ORIENT_3: &str = "OM9-TRANSFORM-006";
/// Matrix `Transform > Rotate 3-D` (OM9-TRANSFORM-008).
pub const ROTATE_3D: &str = "OM9-TRANSFORM-008";
/// Matrix `Transform > Array Polar` (OM9-TRANSFORM-009).
pub const ARRAY_POLAR: &str = "OM9-TRANSFORM-009";
/// Matrix `Transform > Project to C-Plane` (OM9-TRANSFORM-010).
pub const PROJECT_CPLANE: &str = "OM9-TRANSFORM-010";
/// Matrix `Transform > Array Linear` (OM9-TRANSFORM-024).
pub const ARRAY_LINEAR: &str = "OM9-TRANSFORM-024";
/// Matrix `Transform > Array` (OM9-TRANSFORM-025).
pub const ARRAY: &str = "OM9-TRANSFORM-025";
/// Matrix `Transform > Set Points` (OM9-TRANSFORM-030).
pub const SET_POINTS: &str = "OM9-TRANSFORM-030";
/// Matrix `Transform > Scale By Plane` (OM9-TRANSFORM-031).
pub const SCALE_BY_PLANE: &str = "OM9-TRANSFORM-031";
/// Matrix `Transform > Remap to C-Plane` (OM9-TRANSFORM-038).
pub const REMAP_CPLANE: &str = "OM9-TRANSFORM-038";
/// Matrix `Transform > Orient Perpendicular to Curve` (OM9-TRANSFORM-016). The path curve
/// comes from the selection, because this shell has no mid-command object pick.
pub const ORIENT_CURVE: &str = "OM9-TRANSFORM-016";
/// Matrix `Transform > Array Along Curve` (the row is `OM9-L3-04-01-022`, its feature id is
/// `OM9-TRANSFORM-021`). Same selection rule as `ORIENT_CURVE`.
pub const ARRAY_CURVE: &str = "OM9-TRANSFORM-021";

/// One array-along-curve run is bounded: every station is found by bisection on the owner
/// length service, so an unbounded count would turn one click into a very long solve.
pub const MAX_CURVE_ITEMS: usize = 512;

/// The commands this module runs, in menu order.
pub const FEATURES: [&str; 16] = [
    SCALE_3D,
    SCALE_2D,
    SCALE_NU,
    SCALE_1D,
    ORIENT_2,
    ORIENT_3,
    ORIENT_CURVE,
    ROTATE_3D,
    ARRAY_POLAR,
    PROJECT_CPLANE,
    ARRAY_CURVE,
    ARRAY_LINEAR,
    ARRAY,
    SET_POINTS,
    SCALE_BY_PLANE,
    REMAP_CPLANE,
];

/// The rows whose path comes from the selection: exactly one selected curve is the path and
/// every other selected object is transformed along it.
pub fn curve_path_row(feature: &str) -> bool {
    matches!(feature, ORIENT_CURVE | ARRAY_CURVE)
}

pub fn handles(feature: &str) -> bool {
    FEATURES.contains(&feature) || self::rows::handles(feature)
}

/// Every row that runs a body, including the reference-frame rows.
pub fn handled_rows() -> Vec<&'static str> {
    FEATURES
        .iter()
        .chain(self::rows::FEATURES.iter())
        .copied()
        .collect()
}

/// The typed names that reach these rows, so the menu click and the Command Line run one
/// implementation. Both the authored script name (`Scale`, `Scale2D`) and the documented
/// row label (`Scale 3-D`, `Scale Non-Uniform`) resolve; only enabled rows resolve, so a
/// disabled row stays a visible refusal with its recorded reason.
pub fn from_alias(text: &str) -> Option<&'static str> {
    let binding = crate::transform_menu::by_command(text)
        .or_else(|| crate::transform_menu::by_label(text))?;
    binding.enabled().then_some(binding.feature)
}

/// One typed input a running Transform command still needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Point(&'static str),
    Number(&'static str),
}

/// The documented input sequence of one command. A point is resolved through the
/// construction plane captured when the command started, exactly like Line; a number is a
/// plain scalar (a factor, a count, an angle in degrees, a distance in millimetres).
pub fn fields(feature: &str) -> &'static [Field] {
    const SCALE_3D_FIELDS: &[Field] = &[
        Field::Point("Base point for scale: "),
        Field::Number("Scale factor: "),
    ];
    const SCALE_2D_FIELDS: &[Field] = &[
        Field::Point("Base point for scale2D: "),
        Field::Number("Scale factor: "),
    ];
    const SCALE_1D_FIELDS: &[Field] = &[
        Field::Point("Base point for scale1D: "),
        Field::Number("Scale factor: "),
    ];
    const SCALE_NU_FIELDS: &[Field] = &[
        Field::Point("Base point for scaleNU: "),
        Field::Number("X scale factor: "),
        Field::Number("Y scale factor: "),
        Field::Number("Z scale factor: "),
    ];
    const ORIENT_2_FIELDS: &[Field] =
        &[Field::Point("Base point: "), Field::Point("Target point: ")];
    const ORIENT_3_FIELDS: &[Field] = &[
        Field::Point("Base point: "),
        Field::Point("Target point: "),
        Field::Point("Reference direction point: "),
        Field::Point("Target direction point: "),
    ];
    const ROTATE_3D_FIELDS: &[Field] = &[
        Field::Point("Center of rotation: "),
        Field::Number("Rotation angle in degrees: "),
    ];
    const ARRAY_POLAR_FIELDS: &[Field] = &[
        Field::Point("Center of array: "),
        Field::Number("Number of items: "),
        Field::Number("Total angle in degrees: "),
    ];
    const ARRAY_LINEAR_FIELDS: &[Field] = &[
        Field::Number("Number of items: "),
        Field::Number("Spacing in mm: "),
    ];
    const ARRAY_FIELDS: &[Field] = &[
        Field::Number("Number of columns: "),
        Field::Number("Number of rows: "),
        Field::Number("Column spacing in mm: "),
        Field::Number("Row spacing in mm: "),
    ];
    const ORIENT_CURVE_FIELDS: &[Field] = &[Field::Point("Point on curve to orient at: ")];
    const ARRAY_CURVE_FIELDS: &[Field] = &[Field::Number("Number of items: ")];
    const SET_POINTS_FIELDS: &[Field] = &[Field::Point("Target coordinates for selected points: ")];
    const SCALE_BY_PLANE_FIELDS: &[Field] = &[
        Field::Point("Base point for scale by plane: "),
        Field::Number("Scale factor: "),
    ];
    const REMAP_CPLANE_FIELDS: &[Field] = &[
        Field::Point("Source plane origin: "),
        Field::Point("Source plane X point: "),
    ];
    const NO_FIELDS: &[Field] = &[];
    match feature {
        SCALE_3D => SCALE_3D_FIELDS,
        SCALE_2D => SCALE_2D_FIELDS,
        SCALE_1D => SCALE_1D_FIELDS,
        SCALE_NU => SCALE_NU_FIELDS,
        ORIENT_2 => ORIENT_2_FIELDS,
        ORIENT_3 => ORIENT_3_FIELDS,
        ORIENT_CURVE => ORIENT_CURVE_FIELDS,
        ROTATE_3D => ROTATE_3D_FIELDS,
        ARRAY_POLAR => ARRAY_POLAR_FIELDS,
        ARRAY_CURVE => ARRAY_CURVE_FIELDS,
        ARRAY_LINEAR => ARRAY_LINEAR_FIELDS,
        ARRAY => ARRAY_FIELDS,
        SET_POINTS => SET_POINTS_FIELDS,
        SCALE_BY_PLANE => SCALE_BY_PLANE_FIELDS,
        REMAP_CPLANE => REMAP_CPLANE_FIELDS,
        PROJECT_CPLANE => NO_FIELDS,
        _ if self::rows::handles(feature) => self::rows::fields(feature),
        _ => NO_FIELDS,
    }
}

/// A command run that is still collecting typed input.
#[derive(Debug, Clone)]
pub struct TransformPending {
    pub feature: &'static str,
    fields: &'static [Field],
    field: usize,
    pub points: Vec<Point3>,
    pub numbers: Vec<f64>,
    pub plane: CPlane,
    pub unit: Unit,
    pub options: TransformOptions,
}

struct TransformPlan {
    targets: Vec<ObjectId>,
    steps: Vec<Transform>,
    copies: bool,
    replacements: Vec<(ObjectId, Geometry)>,
}

impl TransformPending {
    /// The prompt for the next input.
    pub fn prompt(&self) -> &'static str {
        match self.fields.get(self.field) {
            Some(Field::Point(text)) | Some(Field::Number(text)) => text,
            None => "Command: ",
        }
    }

    /// The input currently expected, or `None` when the run is ready to commit.
    pub fn next(&self) -> Option<Field> {
        self.fields.get(self.field).copied()
    }

    /// How many typed inputs the command still needs.
    pub fn remaining(&self) -> usize {
        self.fields.len().saturating_sub(self.field)
    }
}

/// The construction plane as a normalised transform frame: origin at the plane origin and
/// the three axes of the active CPlane.
fn plane_frame(plane: &CPlane) -> Frame {
    let normal = [
        plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
        plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
        plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
    ];
    Frame {
        origin: plane.origin,
        x: plane.x,
        y: plane.y,
        z: normal,
    }
}

/// Scale by `factors` about `origin` along the given (unit) axes. Composition of kernel
/// `scaling_along` transforms: for orthogonal axes this is exactly the diagonal scale in
/// that frame, and no matrix is written here.
fn scale_about(origin: Point3, axes: &[Point3], factors: &[f64]) -> Result<Transform> {
    let mut result = Transform::identity();
    for (axis, factor) in axes.iter().zip(factors.iter()) {
        if (factor - 1.0).abs() <= f64::EPSILON {
            continue;
        }
        result = gumball::scaling_along(origin, *axis, *factor)?.compose(result)?;
    }
    Ok(result)
}

/// Parse one scalar input. A trailing `mm`, `deg` or `°` is accepted because the manual
/// prints angles with a degree sign, and thousands separators are refused rather than
/// guessed.
pub(super) fn parse_number(text: &str) -> Result<f64> {
    let trimmed = text.trim();
    let trimmed = trimmed
        .strip_suffix("mm")
        .or_else(|| trimmed.strip_suffix("deg"))
        .or_else(|| trimmed.strip_suffix('°'))
        .unwrap_or(trimmed)
        .trim();
    let value: f64 = trimmed
        .parse()
        .map_err(|_| Error::Invalid("transform number"))?;
    if !value.is_finite() {
        return Err(Error::Invalid("transform number"));
    }
    Ok(value)
}

impl Controller {
    /// Start one Transform command. The selection must already hold at least one unlocked
    /// object, because every row of this menu acts on the selection.
    pub fn begin_transform(&mut self, feature: &str) -> Result<()> {
        let binding = crate::transform_menu::by_feature(feature)
            .ok_or(Error::Unsupported("transform command is not catalogued"))?;
        if !binding.enabled() {
            return Err(Error::Unsupported(binding.disabled_reason));
        }
        self.validate_transform_selection()?;
        if curve_path_row(feature) {
            // A curve row needs its path and at least one object before the first prompt,
            // so the command never asks a question and then refuses the answer.
            self.curve_path()?;
        }
        if self::rows::curve_row(feature) {
            self.curve_path()?;
        }
        if self::rows::surface_row(feature) {
            self::rows::surface_selection(self)?;
        }
        if self::rows::flow_row(feature) {
            self::rows::flow_selection(self, feature)?;
        }
        if self::rows::surface_edit_row(feature) {
            self::rows::edit_selection(self, feature)?;
        }
        if self::rows::cage_row(feature) {
            self::cage::validate(self, feature)?;
        }
        if self::rows::flow_link_row(feature) {
            self::flow::validate(self, feature)?;
        }
        let view = &self.workspace.views[&self.workspace.active];
        let pending = TransformPending {
            feature: binding.feature,
            fields: fields(binding.feature),
            field: 0,
            points: Vec::new(),
            numbers: Vec::new(),
            plane: view.cplane,
            unit: self.document().page_unit,
            options: TransformOptions::default(),
        };
        self.events.emit(
            crate::command_events::EventKind::Started,
            binding.feature,
            "started",
            None,
        );
        if pending.next().is_none() {
            return self.finish_transform(&pending);
        }
        self.transform_pending = Some(pending);
        Ok(())
    }

    /// Take one picked/typed world point for the running Transform command.
    pub fn accept_transform_point(
        &mut self,
        mut pending: TransformPending,
        point: Point3,
    ) -> Result<()> {
        match pending.next() {
            Some(Field::Point(_)) => {
                pending.points.push(point);
                pending.field += 1;
            }
            Some(Field::Number(_)) => {
                self.transform_pending = Some(pending);
                return Err(Error::Invalid("transform expects a number"));
            }
            None => return Err(Error::Invalid("transform input")),
        }
        if pending.next().is_none() {
            return match self.finish_transform(&pending) {
                Ok(()) => Ok(()),
                Err(error) => {
                    self.transform_pending = Some(pending);
                    Err(error)
                }
            };
        }
        self.events.emit(
            crate::command_events::EventKind::Progress,
            pending.feature,
            "point",
            None,
        );
        self.transform_pending = Some(pending);
        Ok(())
    }

    /// Typed input for the running Transform command, or `None` when this module does not
    /// own the current session. `Cancel`/`Esc` ends the run without touching the document.
    pub fn submit_transform_text(&mut self, text: &str) -> Option<Result<()>> {
        let mut pending = self.transform_pending.take()?;
        let trimmed = text.trim();
        if trimmed.eq_ignore_ascii_case("cancel") || trimmed.eq_ignore_ascii_case("esc") {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
            return Some(Ok(()));
        }
        let retry = pending.clone();
        if let Some(result) = advanced::option_input(&mut pending, trimmed) {
            self.transform_pending = Some(pending);
            return Some(result);
        }
        if trimmed.is_empty()
            && pending.feature == ARRAY_CURVE
            && (pending.options.spacing.is_some() || !pending.numbers.is_empty())
        {
            return Some(match self.finish_transform(&pending) {
                Ok(()) => Ok(()),
                Err(error) => {
                    self.transform_pending = Some(pending);
                    Err(error)
                }
            });
        }
        let result = self.take_transform_input(&mut pending, trimmed);
        match result {
            Ok(()) if pending.next().is_none() => match self.finish_transform(&pending) {
                Ok(()) => Some(Ok(())),
                Err(error) => {
                    // A refused value keeps the command open on the same field, so the
                    // user can retype it; the document was never touched.
                    self.transform_pending = Some(retry);
                    Some(Err(error))
                }
            },
            Ok(()) => {
                self.transform_pending = Some(pending);
                Some(Ok(()))
            }
            Err(error) => {
                self.transform_pending = Some(pending);
                Some(Err(error))
            }
        }
    }

    fn take_transform_input(&mut self, pending: &mut TransformPending, text: &str) -> Result<()> {
        match pending.next() {
            Some(Field::Point(_)) => {
                let input = om9_core::coords::parse_input(text, pending.unit)?;
                let world = om9_core::coords::resolve_with_base(input, Some(&pending.plane), None)?;
                pending.points.push(world.xyz());
                pending.field += 1;
                Ok(())
            }
            Some(Field::Number(_)) => {
                let value = parse_number(text)?;
                check_number(pending.feature, pending.numbers.len(), value)?;
                pending.numbers.push(value);
                pending.field += 1;
                Ok(())
            }
            None => Err(Error::Invalid("transform input")),
        }
    }

    /// The transform a finished run applies, as a list of `(object, world transform)`.
    /// Placement rows return one entry per selected object; array rows return the copies
    /// and leave the source where it is.
    fn finish_transform(&mut self, pending: &TransformPending) -> Result<()> {
        if self::rows::finish(self, pending)? {
            return Ok(());
        }
        let plan = self.plan_transform(pending)?;
        let name = crate::transform_menu::by_feature(pending.feature)
            .map(|binding| binding.label)
            .unwrap_or("Transform");
        if plan.replacements.is_empty() {
            self.commit_transform_mutations(name, plan.targets, plan.steps, plan.copies)?;
        } else {
            let mutations = plan
                .replacements
                .into_iter()
                .map(|(id, geometry)| {
                    Ok(Mutation::Geometry {
                        source: self.document().stamp(id)?,
                        geometry,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            self.commit_utility_mutations("Set Points", mutations)?;
        }
        self.finish_transform_events(pending)
    }

    fn plan_transform(&self, pending: &TransformPending) -> Result<TransformPlan> {
        let frame = plane_frame(&pending.plane);
        let mut copies = false;
        let (path, targets) = if curve_path_row(pending.feature) {
            let (curve, targets) = self.curve_path()?;
            (Some(curve), targets)
        } else {
            (None, self.transform_targets()?)
        };
        let steps: Vec<Transform> = match pending.feature {
            SCALE_3D => {
                let factor = positive(pending.numbers.first().copied(), "scale factor")?;
                vec![scale_about(
                    point(pending.points.first(), "scale base point")?,
                    &[frame.x, frame.y, frame.z],
                    &[factor, factor, factor],
                )?]
            }
            SCALE_2D => {
                let factor = positive(pending.numbers.first().copied(), "scale factor")?;
                vec![scale_about(
                    point(pending.points.first(), "scale base point")?,
                    &[frame.x, frame.y],
                    &[factor, factor],
                )?]
            }
            SCALE_1D => {
                let factor = positive(pending.numbers.first().copied(), "scale factor")?;
                vec![scale_about(
                    point(pending.points.first(), "scale base point")?,
                    &[frame.x],
                    &[factor],
                )?]
            }
            SCALE_NU => {
                let factors = [
                    positive(pending.numbers.first().copied(), "X scale factor")?,
                    positive(pending.numbers.get(1).copied(), "Y scale factor")?,
                    positive(pending.numbers.get(2).copied(), "Z scale factor")?,
                ];
                vec![scale_about(
                    point(pending.points.first(), "scale base point")?,
                    &[frame.x, frame.y, frame.z],
                    &factors,
                )?]
            }
            SCALE_BY_PLANE => {
                let factor = positive(pending.numbers.first().copied(), "scale factor")?;
                vec![gumball::scaling_along(
                    point(pending.points.first(), "scale base point")?,
                    frame.z,
                    factor,
                )?]
            }
            ORIENT_2 => {
                let from = point(pending.points.first(), "base point")?;
                let to = point(pending.points.get(1), "target point")?;
                vec![om9_core::transform_session::align_transform(
                    &om9_core::transform_session::AlignRequest::translate(from, to),
                )?]
            }
            ORIENT_3 => {
                let from = point(pending.points.first(), "base point")?;
                let to = point(pending.points.get(1), "target point")?;
                let direction = point(pending.points.get(2), "reference direction point")?;
                let target_direction = point(pending.points.get(3), "target direction point")?;
                vec![om9_core::transform_session::align_transform(
                    &om9_core::transform_session::AlignRequest::rotate(
                        from,
                        to,
                        subtract(direction, from),
                        subtract(target_direction, to),
                    ),
                )?]
            }
            ROTATE_3D => {
                let center = point(pending.points.first(), "center of rotation")?;
                let degrees = pending
                    .numbers
                    .first()
                    .copied()
                    .ok_or(Error::Missing("rotation angle"))?;
                if degrees.abs() <= f64::EPSILON {
                    return Err(Error::Invalid("rotation angle"));
                }
                vec![gumball::rotation_about(
                    center,
                    frame.z,
                    degrees.to_radians(),
                )?]
            }
            ARRAY_POLAR => {
                let center = point(pending.points.first(), "center of array")?;
                let count = count(pending.numbers.first().copied(), "array count")?;
                let degrees = pending
                    .numbers
                    .get(1)
                    .copied()
                    .ok_or(Error::Missing("array angle"))?;
                if degrees.abs() <= f64::EPSILON {
                    return Err(Error::Invalid("array angle"));
                }
                let frame = Frame {
                    origin: center,
                    ..frame
                };
                copies = true;
                gumball::polar_array(&frame, count, degrees.to_radians(), false)?
            }
            ORIENT_CURVE => {
                let curve = path.as_ref().ok_or(Error::Missing("path curve"))?;
                let limits = self.coordinator.limits();
                let picked = point(pending.points.first(), "point on curve")?;
                let fraction = nearest_fraction(curve, picked, limits)?;
                let snapped = curve_ops::point_at(curve, fraction, limits)?;
                let tangent = curve_ops::tangent_at(curve, fraction, limits)?;
                // The object's own construction-plane normal stands on the curve, so the
                // row orients it perpendicular to the path at the picked station.
                let mut steps = Vec::new();
                for target in &targets {
                    steps.push(om9_core::transform_session::align_transform(
                        &om9_core::transform_session::AlignRequest::rotate(
                            self.object_center(*target)?,
                            snapped,
                            frame.z,
                            tangent,
                        ),
                    )?);
                }
                steps
            }
            ARRAY_CURVE => {
                let curve = path.as_ref().ok_or(Error::Missing("path curve"))?;
                copies = true;
                advanced::curve_steps(curve, pending, self.coordinator.limits())?
            }
            ARRAY_LINEAR => {
                let count = count(pending.numbers.first().copied(), "array count")?;
                let spacing = pending
                    .numbers
                    .get(1)
                    .copied()
                    .ok_or(Error::Missing("array spacing"))?;
                if spacing.abs() <= f64::EPSILON {
                    return Err(Error::Invalid("array spacing"));
                }
                copies = true;
                gumball::rectangular_array(&frame, [count, 1], [spacing, 0.0], false)?
            }
            ARRAY => {
                let columns = count(pending.numbers.first().copied(), "array columns")?;
                let rows = count(pending.numbers.get(1).copied(), "array rows")?;
                let column_spacing = pending
                    .numbers
                    .get(2)
                    .copied()
                    .ok_or(Error::Missing("array column spacing"))?;
                let row_spacing = pending
                    .numbers
                    .get(3)
                    .copied()
                    .ok_or(Error::Missing("array row spacing"))?;
                copies = true;
                gumball::rectangular_array(
                    &frame,
                    [columns, rows],
                    [column_spacing, row_spacing],
                    false,
                )?
            }
            SET_POINTS => {
                let target = point(pending.points.first(), "object position")?;
                let replacements = targets
                    .iter()
                    .map(|id| {
                        let entity = self
                            .document()
                            .entities
                            .get(id)
                            .ok_or(Error::Missing("SetPt object"))?;
                        let frame: Option<&dyn om9_core::coords::LocalFrame> =
                            if pending.options.align_cplane {
                                Some(&pending.plane)
                            } else {
                                None
                            };
                        Ok((
                            *id,
                            om9_core::point_alignment::set_points(
                                entity,
                                target,
                                pending.options.axes,
                                &pending.options.indices,
                                frame,
                                self.coordinator.limits(),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                return Ok(TransformPlan {
                    targets,
                    steps: Vec::new(),
                    copies: false,
                    replacements,
                });
            }
            REMAP_CPLANE => {
                let origin = point(pending.points.first(), "source plane origin")?;
                let x_point = point(pending.points.get(1), "source plane X point")?;
                let source_x = subtract(x_point, origin);
                vec![om9_core::transform_session::align_transform(
                    &om9_core::transform_session::AlignRequest::rotate(
                        origin,
                        frame.origin,
                        source_x,
                        frame.x,
                    ),
                )?]
            }
            PROJECT_CPLANE => {
                // Project to the CPlane: each object moves along the plane normal by its
                // signed height, so its position lands on the plane. Per-object, because
                // two objects at different heights need different distances.
                let normal = frame.z;
                let mut steps = Vec::new();
                for target in &targets {
                    let center = self.object_center(*target)?;
                    steps.push(Transform::translation(scale(
                        normal,
                        -dot(subtract(center, frame.origin), normal),
                    )));
                }
                return Ok(TransformPlan {
                    targets,
                    steps,
                    copies: false,
                    replacements: Vec::new(),
                });
            }
            other => return Err(Error::Unsupported(other)),
        };
        Ok(TransformPlan {
            targets,
            steps,
            copies,
            replacements: Vec::new(),
        })
    }

    fn finish_transform_events(&mut self, pending: &TransformPending) -> Result<()> {
        self.events.emit(
            crate::command_events::EventKind::Finished,
            pending.feature,
            "committed",
            Some(crate::command_events::EventOutcome::Committed),
        );
        Ok(())
    }

    /// The selected objects a Transform row acts on, in document order.
    fn transform_targets(&self) -> Result<Vec<ObjectId>> {
        let mut targets: Vec<ObjectId> = self.selection.iter().copied().collect();
        targets.sort();
        Ok(targets)
    }

    /// Refuse a run with no selection, or with an object the user cannot edit. The refusal
    /// happens before any input is collected so the command never asks two questions and
    /// then fails.
    fn validate_transform_selection(&self) -> Result<()> {
        if self.selection.is_empty() {
            return Err(Error::Missing("transform selection"));
        }
        let document = self.document();
        for id in &self.selection {
            let entity = document
                .entities
                .get(id)
                .ok_or(Error::Missing("transform object"))?;
            if entity.attributes.locked {
                return Err(Error::Permission("transform selection is locked"));
            }
            if entity.attributes.hidden {
                return Err(Error::Permission("transform selection is hidden"));
            }
        }
        Ok(())
    }

    /// One transaction for the whole command: placements for a transform in place, inserts
    /// for the copies of an array. The scene is rebuilt because placements changed.
    fn commit_transform_mutations(
        &mut self,
        name: &str,
        targets: Vec<ObjectId>,
        steps: Vec<Transform>,
        copies: bool,
    ) -> Result<()> {
        if steps.is_empty() {
            return Ok(());
        }
        let document = self.document();
        let mut mutations: Vec<Mutation> = Vec::new();
        if copies {
            for step in &steps {
                for target in &targets {
                    let entity = document
                        .entities
                        .get(target)
                        .ok_or(Error::Missing("transform object"))?;
                    let mut clone = entity.clone();
                    clone.id = ObjectId::new();
                    clone.placement = step.compose(entity.placement)?;
                    mutations.push(Mutation::Insert {
                        document: document.id,
                        entity: clone,
                    });
                }
            }
        } else if steps.len() == targets.len() {
            // A row that reasons per object (`ProjectToCPlane` by height, `OrientOnCrv` by
            // each object's own centre) hands in one step per target; a shared step is used
            // only when the row computes a single transform for the whole selection.
            for (step, target) in steps.iter().zip(targets.iter()) {
                let entity = document
                    .entities
                    .get(target)
                    .ok_or(Error::Missing("transform object"))?;
                let placement = step.compose(entity.placement)?;
                mutations.push(Mutation::Placement {
                    source: document.stamp(*target)?,
                    placement,
                });
            }
        } else {
            let step = steps[0];
            for target in &targets {
                let entity = document
                    .entities
                    .get(target)
                    .ok_or(Error::Missing("transform object"))?;
                let placement = step.compose(entity.placement)?;
                mutations.push(Mutation::Placement {
                    source: document.stamp(*target)?,
                    placement,
                });
            }
        }
        let held = self.scene_cache_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(name, mutations), held)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }

    /// Split a curve row's selection into its path and its objects: exactly one selected
    /// curve is the path, every other selected object is transformed along it.
    fn curve_path(&self) -> Result<(Curve, Vec<ObjectId>)> {
        let document = self.document();
        let mut path: Option<Curve> = None;
        let mut targets: Vec<ObjectId> = Vec::new();
        let mut ids: Vec<ObjectId> = self.selection.iter().copied().collect();
        ids.sort();
        for id in ids {
            let entity = document
                .entities
                .get(&id)
                .ok_or(Error::Missing("transform object"))?;
            match &entity.geometry {
                Geometry::Curve(curve) => {
                    if path.is_some() {
                        return Err(Error::Invalid("one path curve"));
                    }
                    path = Some(om9_core::top11::bake_curve(curve, entity.placement)?);
                }
                _ => targets.push(id),
            }
        }
        let path = path.ok_or(Error::Missing("path curve selection"))?;
        if targets.is_empty() {
            return Err(Error::Missing(
                "select the objects and the path curve together",
            ));
        }
        Ok((path, targets))
    }

    /// The world centre of one object, from its placement origin (the value `SetPt` and
    /// `ProjectToCPlane` reason about).
    fn object_center(&self, id: ObjectId) -> Result<Point3> {
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("transform object"))?;
        entity.placement.point([0.0, 0.0, 0.0])
    }
}

/// The rule for the `index`-th scalar of one command, applied to every typed value so a
/// refused number never starts a half-valid run. The same rule is re-applied when the run
/// commits, because a point can also arrive from a pick.
fn check_number(feature: &str, index: usize, value: f64) -> Result<()> {
    if let Some(result) = self::rows::check_number(feature, index, value) {
        return result;
    }
    if let Some(result) = self::cage::check_number(feature, index, value) {
        return result;
    }
    if let Some(result) = self::flow::check_number(feature, index, value) {
        return result;
    }
    let name = match (feature, index) {
        (SCALE_3D | SCALE_2D | SCALE_1D | SCALE_BY_PLANE, 0) => "scale factor",
        (SCALE_NU, 0) => "X scale factor",
        (SCALE_NU, 1) => "Y scale factor",
        (SCALE_NU, 2) => "Z scale factor",
        (ROTATE_3D, 0) => "rotation angle",
        (ARRAY_POLAR, 0) => "array count",
        (ARRAY_POLAR, 1) => "array angle",
        (ARRAY_CURVE, 0) | (ARRAY_LINEAR, 0) => "array count",
        (rows::ARRAY_SURFACE, 0) => "array rows",
        (rows::ARRAY_SURFACE, 1) => "array columns",
        (rows::ARRAY_CURVE_SURFACE, 0) => "array count",
        (rows::STRETCH, 0) => "scale factor",
        (rows::ARRAY_SURFACE, 2) | (rows::ARRAY_CURVE_SURFACE, 1) | (rows::SYMMETRY, 0) => "flag",
        (ARRAY_LINEAR, 1) => "array spacing",
        (ARRAY, 0) => "array columns",
        (ARRAY, 1) => "array rows",
        (ARRAY, 2) => "array column spacing",
        (ARRAY, 3) => "array row spacing",
        _ => "transform number",
    };
    match name {
        "scale factor" | "X scale factor" | "Y scale factor" | "Z scale factor" => {
            positive(Some(value), "scale factor").map(|_| ())
        }
        "array count" | "array columns" | "array rows" => count(Some(value), name).map(|_| ()),
        // The reference-frame rows ask their switches as numbers so the typed contract stays
        // points and scalars; only an exact 0 or 1 is a switch.
        "flag" => {
            if (value - 0.0).abs() <= f64::EPSILON || (value - 1.0).abs() <= f64::EPSILON {
                Ok(())
            } else {
                Err(Error::Invalid("0 or 1 switch"))
            }
        }
        _ => {
            if !value.is_finite() || value == 0.0 {
                return Err(Error::Invalid(name));
            }
            Ok(())
        }
    }
}

fn point(value: Option<&Point3>, name: &'static str) -> Result<Point3> {
    value.copied().ok_or(Error::Missing(name))
}

fn positive(value: Option<f64>, name: &'static str) -> Result<f64> {
    let value = value.ok_or(Error::Missing(name))?;
    if !value.is_finite() || value <= 0.0 {
        return Err(Error::Invalid(name));
    }
    Ok(value)
}

fn count(value: Option<f64>, name: &'static str) -> Result<usize> {
    let value = value.ok_or(Error::Missing(name))?;
    if !value.is_finite() || value < 2.0 || value.fract().abs() > 1e-9 {
        return Err(Error::Invalid(name));
    }
    if value > 10_000.0 {
        return Err(Error::Invalid(name));
    }
    Ok(value as usize)
}

/// The nearest station on a curve, refined from a coarse sample so a typed or picked
/// point snaps to the path instead of being accepted at a distance.
fn nearest_fraction(curve: &Curve, target: Point3, limits: &Limits) -> Result<f64> {
    om9_core::curve_surface_ops::closest_point_on_curve(curve, target, limits)
        .map(|result| result.0)
}

/// The point at an arc-length distance from the start of the curve, through the owner
/// `parameter_at_length`, so spacing along the path is measured, not guessed. The rows use
/// `curve_ops` directly; this wrapper is what the station unit tests measure.
#[cfg(test)]
fn point_at_length(curve: &Curve, distance: f64, limits: &Limits) -> Result<Point3> {
    let fraction = om9_core::curve_ops::parameter_at_length(curve, distance, limits)?;
    om9_core::curve_ops::point_at(curve, fraction, limits)
}

fn subtract(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(vector: Point3, factor: f64) -> Point3 {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length(vector: Point3) -> f64 {
    dot(vector, vector).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: Point3, expected: Point3) -> bool {
        (0..3).all(|axis| (actual[axis] - expected[axis]).abs() < 1e-9)
    }

    fn line(start: Point3, end: Point3) -> Curve {
        let length = ((end[0] - start[0]).powi(2)
            + (end[1] - start[1]).powi(2)
            + (end[2] - start[2]).powi(2))
        .sqrt();
        Curve::Line {
            start,
            end,
            domain: [0.0, length],
        }
    }

    #[test]
    fn a_station_snaps_to_the_nearest_point_on_the_path() {
        let limits = Limits::default();
        let curve = line([0.0; 3], [10.0, 0.0, 0.0]);
        let fraction = nearest_fraction(&curve, [5.0, 2.0, 0.0], &limits).expect("station");
        let station = curve_ops::point_at(&curve, fraction, &limits).expect("point");
        assert!(close(station, [5.0, 0.0, 0.0]), "got {station:?}");
        // A point before the start and past the end clamps to the curve ends.
        let before = nearest_fraction(&curve, [-3.0, 1.0, 0.0], &limits).expect("start");
        assert!(before.abs() < 1e-9, "got {before}");
        let after = nearest_fraction(&curve, [12.0, 0.0, 0.0], &limits).expect("end");
        assert!((after - 1.0).abs() < 1e-9, "got {after}");
    }

    #[test]
    fn a_station_is_measured_along_the_curve_length_not_along_the_axis() {
        let limits = Limits::default();
        // An L-shaped path is 20 mm long but only 10 mm wide, so a station at 15 mm is
        // halfway up the second leg: measuring the axis instead would land elsewhere.
        let curve = Curve::Polycurve {
            segments: vec![
                line([0.0; 3], [10.0, 0.0, 0.0]),
                line([10.0, 0.0, 0.0], [10.0, 10.0, 0.0]),
            ],
            reversed: false,
        };
        let total = curve_ops::arc_length(&curve, &limits).expect("length");
        assert!((total - 20.0).abs() < 1e-9, "got {total}");
        let station = point_at_length(&curve, 15.0, &limits).expect("station");
        assert!(close(station, [10.0, 5.0, 0.0]), "got {station:?}");
        let start = point_at_length(&curve, 0.0, &limits).expect("start");
        assert!(close(start, [0.0; 3]));
    }

    #[test]
    fn a_number_accepts_the_documented_units_and_refuses_junk() {
        assert_eq!(parse_number("2").expect("plain"), 2.0);
        assert_eq!(parse_number(" 2.5 ").expect("padded"), 2.5);
        assert_eq!(parse_number("90deg").expect("degree suffix"), 90.0);
        assert_eq!(parse_number("90°").expect("degree sign"), 90.0);
        assert_eq!(parse_number("10mm").expect("millimetre suffix"), 10.0);
        assert!(parse_number("two").is_err());
        assert!(parse_number("1,000").is_err(), "no invented separators");
        assert!(parse_number("inf").is_err());
    }

    #[test]
    fn a_count_is_a_whole_number_of_at_least_two() {
        assert_eq!(count(Some(2.0), "count").expect("two"), 2);
        assert_eq!(count(Some(10000.0), "count").expect("upper bound"), 10_000);
        assert!(count(Some(1.0), "count").is_err());
        assert!(count(Some(2.5), "count").is_err());
        assert!(count(Some(f64::NAN), "count").is_err());
        assert!(count(None, "count").is_err());
    }

    #[test]
    fn a_factor_must_be_positive_and_finite() {
        assert_eq!(positive(Some(2.0), "factor").expect("two"), 2.0);
        assert!(positive(Some(0.0), "factor").is_err());
        assert!(positive(Some(-1.0), "factor").is_err());
        assert!(positive(Some(f64::INFINITY), "factor").is_err());
    }

    #[test]
    fn scale_about_composes_the_kernel_axes() {
        let origin = [1.0, 2.0, 3.0];
        let axes = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let transform = scale_about(origin, &axes, &[2.0, 3.0, 4.0]).expect("scale");
        assert!(close(
            transform.point([2.0, 3.0, 4.0]).expect("point"),
            [3.0, 5.0, 7.0]
        ));
        // A factor of 1 leaves that axis alone.
        let partial = scale_about(origin, &axes[..1], &[1.0]).expect("identity");
        assert!(close(
            partial.point([9.0, 9.0, 9.0]).expect("point"),
            [9.0, 9.0, 9.0]
        ));
    }

    #[test]
    fn the_cplane_frame_is_right_handed() {
        let plane = CPlane::world_xy();
        let frame = plane_frame(&plane);
        assert!(close(frame.origin, [0.0; 3]));
        assert!(close(frame.x, [1.0, 0.0, 0.0]));
        assert!(close(frame.y, [0.0, 1.0, 0.0]));
        assert!(close(frame.z, [0.0, 0.0, 1.0]));
    }

    #[test]
    fn every_command_asks_for_its_documented_inputs() {
        for feature in FEATURES {
            let binding = crate::transform_menu::by_feature(feature).expect("catalogued");
            assert!(binding.enabled(), "{feature}");
            let fields = fields(feature);
            let points = fields
                .iter()
                .filter(|field| matches!(field, Field::Point(_)))
                .count();
            let numbers = fields.len() - points;
            if feature == SCALE_3D
                || feature == SCALE_2D
                || feature == SCALE_1D
                || feature == SCALE_BY_PLANE
            {
                assert_eq!((points, numbers), (1, 1), "{feature}");
            }
            if feature == SCALE_NU {
                assert_eq!((points, numbers), (1, 3), "{feature}");
            }
            if feature == ROTATE_3D {
                assert_eq!((points, numbers), (1, 1), "{feature}");
            }
            if feature == ARRAY_LINEAR {
                assert_eq!((points, numbers), (0, 2), "{feature}");
            }
            if feature == ARRAY {
                assert_eq!((points, numbers), (0, 4), "{feature}");
            }
            if feature == PROJECT_CPLANE {
                assert!(fields.is_empty(), "{feature} needs no typed input");
            }
            for field in fields {
                let prompt = match field {
                    Field::Point(text) | Field::Number(text) => text,
                };
                assert!(prompt.ends_with(": "), "{feature}: {prompt}");
            }
        }
    }

    #[test]
    fn the_prompt_advances_through_the_fields() {
        let mut pending = TransformPending {
            feature: SCALE_3D,
            fields: fields(SCALE_3D),
            field: 0,
            points: Vec::new(),
            numbers: Vec::new(),
            plane: CPlane::world_xy(),
            unit: Unit::Millimeter,
            options: TransformOptions::default(),
        };
        assert_eq!(pending.prompt(), "Base point for scale: ");
        assert_eq!(pending.remaining(), 2);
        pending.field = 1;
        assert_eq!(pending.prompt(), "Scale factor: ");
        assert_eq!(pending.remaining(), 1);
        pending.field = 2;
        assert_eq!(pending.prompt(), "Command: ");
        assert!(pending.next().is_none());
    }
}
