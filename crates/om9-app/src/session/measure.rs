// SPDX-License-Identifier: LGPL-2.1-or-later
//! Measure menu read-outs (OM9-L2-03-20).
//!
//! The Matrix Measure menu reports values in the Command Line and never edits the
//! model: `Angle`, `Distance`, `Length` and `Radius` are read-only analyses, and
//! `Utilities > Bounding Box` with its `Output=None` contract reports the size and
//! center instead of drawing a box. This module owns the arithmetic, the label
//! text and the type/count summary; `session.rs` owns the interactive point input
//! (Distance/Angle) and the menu/Command-Line dispatch.
//!
//! Every value is canonical millimetres (angles in degrees), so a read-out does
//! not depend on the display unit of the document.
use om9_core::{
    Error, Result,
    budget::Limits,
    curve_ops,
    geometry::Unit,
    geometry::{Curve, Geometry, Point3, Transform},
    identity::ObjectId,
    measure,
    model::{
        CUTTER_EXTENSION_SLOT, Document, GEM_EXTENSION_SLOT, HEAD_EXTENSION_SLOT,
        RAIL_EXTENSION_SLOT, extension_slot_known,
    },
    utilities::{self, Bounds},
};
use std::collections::BTreeMap;

use super::Controller;
mod extended;
pub use extended::MeasureOptions;

/// Matrix `Measure > Distance` (OM9-MEASURE-002).
pub const DISTANCE: &str = "OM9-MEASURE-002";
/// Matrix `Measure > Angle` (OM9-MEASURE-001).
pub const ANGLE: &str = "OM9-MEASURE-001";
/// Matrix `Measure > Length` (OM9-MEASURE-003).
pub const LENGTH: &str = "OM9-MEASURE-003";
/// Matrix `Measure > Radius` (OM9-MEASURE-005).
pub const RADIUS: &str = "OM9-MEASURE-005";
/// Matrix `Utilities > Bounding Box` with `Output=None` (OM9-UTIL-010).
pub const BOUNDING_BOX: &str = "OM9-UTIL-010";
/// The jewellery type/count summary (OM9-L3-03-20-004). Matrix has no menu row
/// for a whole-document count, so it is reached by its typed name only.
pub const OBJECT_SUMMARY: &str = "ObjectSummary";
pub const AREA: &str = "Area";
pub const VOLUME: &str = "Volume";

/// The features this module answers for, in menu order.
pub const FEATURES: [&str; 8] = [
    ANGLE,
    DISTANCE,
    LENGTH,
    RADIUS,
    BOUNDING_BOX,
    OBJECT_SUMMARY,
    AREA,
    VOLUME,
];

pub fn handles(feature: &str) -> bool {
    FEATURES.contains(&feature)
}

/// The original Matrix names and the short names a user types at the Command
/// Line, resolved to the same feature the menu row dispatches.
pub fn from_alias(text: &str) -> Option<&'static str> {
    let text = text.trim();
    let name = match text {
        t if t.eq_ignore_ascii_case("distance") || t.eq_ignore_ascii_case("dist") => DISTANCE,
        t if t.eq_ignore_ascii_case("angle") || t.eq_ignore_ascii_case("ang") => ANGLE,
        t if t.eq_ignore_ascii_case("length") || t.eq_ignore_ascii_case("len") => LENGTH,
        t if t.eq_ignore_ascii_case("radius") || t.eq_ignore_ascii_case("rad") => RADIUS,
        t if t.eq_ignore_ascii_case("area") => AREA,
        t if t.eq_ignore_ascii_case("volume") => VOLUME,
        t if t.eq_ignore_ascii_case("boundingbox")
            || t.eq_ignore_ascii_case("bounding box")
            || t.eq_ignore_ascii_case("bbox") =>
        {
            BOUNDING_BOX
        }
        t if t.eq_ignore_ascii_case("objectsummary")
            || t.eq_ignore_ascii_case("object summary")
            || t.eq_ignore_ascii_case("objectcount")
            || t.eq_ignore_ascii_case("count") =>
        {
            OBJECT_SUMMARY
        }
        _ => return None,
    };
    Some(name)
}

/// Distance and Angle take their operands by picking points; the other rows
/// analyse the current selection immediately.
pub fn collects_points(feature: &str) -> bool {
    matches!(feature, DISTANCE | ANGLE)
}

/// How many world points the point-driven commands need before they report.
pub fn points_needed(feature: &str) -> usize {
    match feature {
        DISTANCE => 2,
        ANGLE => 4,
        _ => 0,
    }
}

/// The prompt for the next point of a running Distance/Angle measurement.
pub fn point_prompt(feature: &str, collected: usize) -> &'static str {
    match (feature, collected) {
        (DISTANCE, 0) => "First distance point: ",
        (DISTANCE, _) => "Second distance point: ",
        (ANGLE, 0) => "First angle line start: ",
        (ANGLE, 1) => "First angle line end: ",
        (ANGLE, 2) => "Second angle line start: ",
        (ANGLE, _) => "Second angle line end: ",
        _ => "Command: ",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasureUnit {
    Millimetre,
    SquareMillimetre,
    CubicMillimetre,
    Degree,
    /// A count of objects, not a length: the summary never claims millimetres.
    Count,
    /// A jewellery mass read-out. The report also carries pennyweights in its text.
    Gram,
    Pennyweight,
}

impl MeasureUnit {
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Millimetre => "mm",
            Self::SquareMillimetre => "mm²",
            Self::CubicMillimetre => "mm³",
            Self::Degree => "deg",
            Self::Count => "objects",
            Self::Gram => "g",
            Self::Pennyweight => "dwt",
        }
    }
}

/// One read-out: the value the command measured, its unit and the exact label the
/// Command Line shows. Construction is the only way to build one, so a label can
/// never disagree with the number it reports.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    pub feature: &'static str,
    pub value: f64,
    pub unit: MeasureUnit,
    text: String,
}

impl Measurement {
    pub fn new(feature: &'static str, name: &str, value: f64, unit: MeasureUnit) -> Self {
        let text = format!("{name} = {} {}", Self::format_value(value), unit.suffix());
        Self {
            feature,
            value,
            unit,
            text,
        }
    }

    /// A label whose shape is not `name = value unit`, e.g. the size and center of
    /// a bounding box or the per-type counts of a document.
    pub fn with_text(feature: &'static str, value: f64, unit: MeasureUnit, text: String) -> Self {
        Self {
            feature,
            value,
            unit,
            text,
        }
    }

    pub fn label(&self) -> &str {
        &self.text
    }

    /// Three decimals, trailing zeros trimmed, so `5` prints exactly as `5 mm`
    /// and a rounded sub-thousandth never prints as `-0`.
    pub fn format_value(value: f64) -> String {
        let mut text = format!("{value:.3}");
        while text.contains('.') && text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
        if text == "-0" { "0".to_owned() } else { text }
    }
}

/// The label of the last measurement, tagged with the selection revision it was
/// measured against, so a stale read-out can never be shown after a selection
/// change (OM9-L3-03-20-005).
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementLabel {
    pub text: String,
    pub selection_revision: u64,
}

/// A typed document summary for the jewellery workflow: geometry counts plus the
/// semantic jewellery slots. Unknown slots are counted separately and never
/// guessed as gems; block definitions are reported but never added to `total`,
/// so an instance is counted once through its placed entity.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectSummary {
    pub total: u64,
    pub by_kind: BTreeMap<String, u64>,
    pub gems: u64,
    pub heads: u64,
    pub rails: u64,
    pub cutters: u64,
    pub unknown_slots: u64,
    pub definitions: u64,
    pub by_layer: BTreeMap<String, u64>,
    pub by_definition: BTreeMap<String, u64>,
    pub known_carats: f64,
    pub gems_without_weight: u64,
}

impl ObjectSummary {
    pub fn label(&self) -> String {
        let kinds = self
            .by_kind
            .iter()
            .map(|(kind, count)| format!("{kind} {count}"))
            .collect::<Vec<_>>()
            .join(", ");
        let mut text = format!(
            "Objects: {} total ({kinds}) · gems {} · heads {} · rails {} · cutters {} · unknown {}",
            self.total, self.gems, self.heads, self.rails, self.cutters, self.unknown_slots
        );
        if self.definitions > 0 {
            text.push_str(&format!(" · definitions {}", self.definitions));
        }
        text.push_str(&format!(
            " · gem weight {} ct ({} unknown) · layers {:?} · placed definitions {:?}",
            Measurement::format_value(self.known_carats),
            self.gems_without_weight,
            self.by_layer,
            self.by_definition
        ));
        text
    }
}

/// Distance between two world points, or the undirected angle between two
/// directed lines (`from -> to`) in degrees.
pub fn measure_points(feature: &str, points: &[Point3]) -> Result<Measurement> {
    match feature {
        DISTANCE => {
            let [first, second] = points else {
                return Err(Error::Missing("distance needs two points"));
            };
            let millimetres = measure::distance_mm(*first, *second)?;
            Ok(Measurement::new(
                DISTANCE,
                "Distance",
                millimetres,
                MeasureUnit::Millimetre,
            ))
        }
        ANGLE => {
            let [a0, a1, b0, b1] = points else {
                return Err(Error::Missing("angle needs four points"));
            };
            let degrees = measure::angle_degrees(*a0, *a1, *b0, *b1)?;
            Ok(Measurement::new(
                ANGLE,
                "Angle",
                degrees,
                MeasureUnit::Degree,
            ))
        }
        _ => Err(Error::Unsupported("point measurement")),
    }
}

/// Total exact arc length of the given curves, in millimetres. The sum is the
/// documented Length read-out for a curve or a multi-curve selection.
pub fn measure_length(curves: &[Curve], limits: &Limits) -> Result<Measurement> {
    if curves.is_empty() {
        return Err(Error::Missing("curve selection"));
    }
    let mut total = 0.0;
    for curve in curves {
        let length = curve_ops::arc_length(curve, limits)?;
        if !length.is_finite() || length < 0.0 {
            return Err(Error::Invalid("curve length"));
        }
        total += length;
        if !total.is_finite() {
            return Err(Error::Invalid("curve length"));
        }
    }
    Ok(Measurement::new(
        LENGTH,
        "Length",
        total,
        MeasureUnit::Millimetre,
    ))
}

/// Radius of a circle curve (a full circle or an arc) in millimetres. A curve
/// without a single radius is refused instead of reporting spline curvature.
pub fn measure_radius(curve: &Curve) -> Result<Measurement> {
    match curve {
        Curve::Circle { radius, .. } => {
            if !radius.is_finite() || *radius <= 0.0 {
                return Err(Error::Invalid("circle radius"));
            }
            Ok(Measurement::new(
                RADIUS,
                "Radius",
                *radius,
                MeasureUnit::Millimetre,
            ))
        }
        _ => Err(Error::Unsupported("radius needs a circle curve")),
    }
}

/// World bounding box of the selected objects. A non-identity placement is
/// applied to the eight local corners, so a moved object reports its moved box.
pub fn world_bounds(document: &Document, ids: &[ObjectId]) -> Result<Bounds> {
    if ids.is_empty() {
        return Err(Error::Missing("bounding box selection"));
    }
    let mut result: Option<Bounds> = None;
    for id in ids {
        let entity = document
            .entities
            .get(id)
            .ok_or(Error::Missing("bounding box object"))?;
        let Some(local) = utilities::bounds(&entity.geometry) else {
            return Err(Error::Unsupported("bounding box geometry"));
        };
        if entity.placement == Transform::identity() {
            add_bounds(&mut result, local);
            continue;
        }
        for corner in corners(local) {
            let world = entity.placement.point(corner)?;
            add_point(&mut result, world);
        }
    }
    result.ok_or(Error::Invalid("bounding box has no finite points"))
}

/// `Utilities > Bounding Box` with `Output=None`: report the size and center.
pub fn bounds_measurement(bounds: Bounds) -> Measurement {
    let size = bounds.size();
    let center = bounds.center();
    let value = size[0].max(size[1]).max(size[2]);
    let text = format!(
        "Bounding Box = {} x {} x {} mm, center ({}, {}, {})",
        Measurement::format_value(size[0]),
        Measurement::format_value(size[1]),
        Measurement::format_value(size[2]),
        Measurement::format_value(center[0]),
        Measurement::format_value(center[1]),
        Measurement::format_value(center[2])
    );
    Measurement::with_text(BOUNDING_BOX, value, MeasureUnit::Millimetre, text)
}

/// Count the document's objects by geometry kind and by jewellery semantic slot.
pub fn summarize(document: &Document) -> ObjectSummary {
    let mut summary = ObjectSummary {
        total: 0,
        by_kind: BTreeMap::new(),
        gems: 0,
        heads: 0,
        rails: 0,
        cutters: 0,
        unknown_slots: 0,
        definitions: document.definitions.len() as u64,
        by_layer: BTreeMap::new(),
        by_definition: BTreeMap::new(),
        known_carats: 0.,
        gems_without_weight: 0,
    };
    for entity in document.entities.values() {
        summary.total += 1;
        let layer = entity
            .attributes
            .layer
            .map(|id| {
                document
                    .layers
                    .get(&id)
                    .map(|l| format!("{} ({id})", l.name))
                    .unwrap_or_else(|| id.to_string())
            })
            .unwrap_or_else(|| "Unassigned".into());
        *summary.by_layer.entry(layer).or_default() += 1;
        if let Geometry::Instance(id) = entity.geometry {
            let name = document
                .definitions
                .get(&id)
                .map(|d| format!("{} ({id})", d.label))
                .unwrap_or_else(|| id.to_string());
            *summary.by_definition.entry(name).or_default() += 1;
        }
        extended::gem_weights(document, entity, &mut summary, &mut Vec::new());
        *summary
            .by_kind
            .entry(entity.geometry.kind().tag().to_owned())
            .or_default() += 1;
        for slot in entity.extensions.keys() {
            match slot.as_str() {
                GEM_EXTENSION_SLOT => {}
                HEAD_EXTENSION_SLOT => summary.heads += 1,
                RAIL_EXTENSION_SLOT => summary.rails += 1,
                CUTTER_EXTENSION_SLOT => summary.cutters += 1,
                other if !extension_slot_known(other) => summary.unknown_slots += 1,
                _ => {}
            }
        }
    }
    summary
}

/// The curves of the selection in pick order; a non-curve refuses the read-out
/// instead of silently measuring a subset.
pub fn selected_curves(document: &Document, ids: &[ObjectId]) -> Result<Vec<Curve>> {
    if ids.is_empty() {
        return Err(Error::Missing("curve selection"));
    }
    let mut curves = Vec::with_capacity(ids.len());
    for id in ids {
        let entity = document
            .entities
            .get(id)
            .ok_or(Error::Missing("selected object"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("curve selection"));
        };
        curves.push(om9_core::top11::bake_curve(curve, entity.placement)?);
    }
    Ok(curves)
}

/// A Distance/Angle run waiting for its picked points.
#[derive(Debug, Clone)]
pub struct MeasurePending {
    pub feature: &'static str,
    pub points: Vec<Point3>,
    /// The construction plane latched when the command started: typed local
    /// coordinates resolve through it, exactly as the Line command does.
    pub plane: om9_scene::camera::CPlane,
    pub unit: Unit,
    pub options: MeasureOptions,
}

impl MeasurePending {
    pub fn prompt(&self) -> &'static str {
        if self.feature == RADIUS {
            return "Point on curve for local curvature radius: ";
        }
        if self.feature == LENGTH {
            return "Isocurve U=value or V=value; Enter to measure: ";
        }
        point_prompt(self.feature, self.points.len())
    }
}

impl Controller {
    /// Start the point-driven read-out. The selection rows (Length, Radius,
    /// Bounding Box, ObjectSummary) analyse immediately through
    /// [`Controller::measure_selection`].
    pub fn begin_measure(&mut self, feature: &str) -> Result<()> {
        let feature = match feature {
            DISTANCE => DISTANCE,
            ANGLE => ANGLE,
            RADIUS if !self.selected_circle_radius_available() => {
                self.selected_single_curve()?;
                RADIUS
            }
            LENGTH if self.selected_surface_available() => LENGTH,
            other => return self.measure_selection(other),
        };
        let view = &self.workspace.views[&self.workspace.active];
        self.measure_pending = Some(MeasurePending {
            feature,
            points: Vec::new(),
            plane: view.cplane,
            unit: self.document().page_unit,
            options: MeasureOptions::default(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "started",
            None,
        );
        Ok(())
    }

    /// Take one picked/typed world point. The command reports only after its
    /// last point; a refused measurement closes the command with the typed error
    /// and never leaves a partial read-out behind.
    pub fn accept_measure_point(
        &mut self,
        mut pending: MeasurePending,
        point: Point3,
    ) -> Result<()> {
        self.last_point = Some(om9_core::coords::WorldPoint::try_new(point)?);
        let retry = pending.clone();
        pending
            .points
            .push(extended::constrain_point(&pending, point)?);
        if pending.feature == RADIUS {
            let result = self.curvature_at_pick(point);
            return match result {
                Ok(m) => {
                    self.report_measurement(m);
                    Ok(())
                }
                Err(e) => {
                    self.measure_pending = Some(retry);
                    Err(e)
                }
            };
        }
        if pending.points.len() < points_needed(pending.feature) {
            self.events.emit(
                crate::command_events::EventKind::Progress,
                pending.feature,
                "point",
                None,
            );
            self.measure_pending = Some(pending);
            return Ok(());
        }
        let measurement = match measure_points(pending.feature, &pending.points) {
            Ok(m) => m,
            Err(e) => {
                self.measure_pending = Some(retry);
                return Err(e);
            }
        };
        let continuous = pending.options.continuous;
        self.report_measurement_event(measurement, !continuous);
        if continuous {
            pending.points = if pending.feature == DISTANCE {
                vec![*pending.points.last().unwrap()]
            } else {
                Vec::new()
            };
            self.measure_pending = Some(pending);
        }
        Ok(())
    }

    /// Typed answer for a running Distance/Angle read-out: a coordinate or
    /// `Cancel`. `None` means no read-out is waiting for input. A refused point
    /// keeps the points already accepted, as the Line command does.
    pub fn submit_measure_text(&mut self, text: &str) -> Option<Result<()>> {
        let mut pending = self.measure_pending.take()?;
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
        let action = if trimmed.eq_ignore_ascii_case("objects")
            && matches!(pending.feature, DISTANCE | ANGLE)
        {
            Some(self.measure_selected_objects(pending.feature))
        } else if trimmed.is_empty() && pending.feature == LENGTH {
            Some(
                pending
                    .options
                    .iso
                    .ok_or(Error::Missing("choose U=value or V=value"))
                    .and_then(|(u, v)| self.measure_isocurve(u, v)),
            )
        } else if pending.feature == RADIUS && trimmed.to_ascii_lowercase().starts_with("station=")
        {
            Some(
                trimmed
                    .split_once('=')
                    .unwrap()
                    .1
                    .parse::<f64>()
                    .map_err(|_| Error::Invalid("curvature station"))
                    .and_then(|f| self.curvature_at_fraction(f)),
            )
        } else {
            None
        };
        if let Some(result) = action {
            return Some(match result {
                Ok(m) => {
                    self.report_measurement(m);
                    Ok(())
                }
                Err(e) => {
                    self.measure_pending = Some(pending);
                    Err(e)
                }
            });
        }
        if let Some(result) = self.measure_option_input(&mut pending, trimmed) {
            self.measure_pending = Some(pending);
            return Some(result);
        }
        let input = match om9_core::coords::parse_input(trimmed, pending.unit) {
            Ok(input) => input,
            Err(error) => {
                self.measure_pending = Some(pending);
                return Some(Err(error));
            }
        };
        let world = match om9_core::coords::resolve_with_base(
            input,
            Some(&pending.plane),
            self.relative_base(),
        ) {
            Ok(world) => world,
            Err(error) => {
                self.measure_pending = Some(pending);
                return Some(Err(error));
            }
        };
        Some(self.accept_measure_point(pending, world.xyz()))
    }

    /// Publish one read-out: it becomes the current label and one
    /// Started/Progress/Finished run in the event log. The document is untouched.
    pub fn report_measurement(&mut self, measurement: Measurement) {
        self.report_measurement_event(measurement, true);
    }
    fn report_measurement_event(&mut self, measurement: Measurement, finished: bool) {
        let label = measurement.label().to_owned();
        self.last_measurement = Some(MeasurementLabel {
            text: label.clone(),
            selection_revision: self.selection_revision,
        });
        self.events.emit(
            crate::command_events::EventKind::Progress,
            measurement.feature,
            &label,
            None,
        );
        if finished {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                measurement.feature,
                &label,
                Some(crate::command_events::EventOutcome::Committed),
            );
        }
    }

    /// Run a selection-driven read-out by its feature id (menu and Command Line
    /// reach the same body).
    pub fn measure_selection(&mut self, feature: &str) -> Result<()> {
        match feature {
            LENGTH => {
                self.measure_selected_length()?;
            }
            RADIUS => {
                self.measure_selected_radius()?;
            }
            BOUNDING_BOX => {
                self.measure_selected_bounds()?;
            }
            OBJECT_SUMMARY => {
                self.object_summary()?;
            }
            AREA | VOLUME => {
                let m = self.measure_selected_integral(feature)?;
                self.report_measurement(m);
            }
            _ => return Err(Error::Unsupported("measurement command")),
        }
        Ok(())
    }

    pub fn measure_distance(&mut self, first: Point3, second: Point3) -> Result<Measurement> {
        let measurement = measure_points(DISTANCE, &[first, second])?;
        self.report_measurement(measurement.clone());
        Ok(measurement)
    }

    pub fn measure_angle(
        &mut self,
        a0: Point3,
        a1: Point3,
        b0: Point3,
        b1: Point3,
    ) -> Result<Measurement> {
        let measurement = measure_points(ANGLE, &[a0, a1, b0, b1])?;
        self.report_measurement(measurement.clone());
        Ok(measurement)
    }

    pub fn measure_selected_length(&mut self) -> Result<Measurement> {
        let mut ids = self.selected_ordered();
        for s in &self.subselection {
            if !ids.contains(&s.root) {
                ids.push(s.root);
            }
        }
        let measurement = self.selected_length(&ids)?;
        self.report_measurement(measurement.clone());
        Ok(measurement)
    }

    pub fn measure_selected_radius(&mut self) -> Result<Measurement> {
        let ids = self.selected_ordered();
        if ids.len() != 1 {
            return Err(Error::Invalid("radius needs one circle"));
        }
        let curves = selected_curves(self.document(), &ids)?;
        let measurement = measure_radius(&curves[0])?;
        self.report_measurement(measurement.clone());
        Ok(measurement)
    }

    pub fn measure_selected_bounds(&mut self) -> Result<Measurement> {
        let ids = self.selected_ordered();
        let bounds = world_bounds(self.document(), &ids)?;
        let measurement = bounds_measurement(bounds);
        self.report_measurement(measurement.clone());
        Ok(measurement)
    }

    pub fn object_summary(&mut self) -> Result<Measurement> {
        let summary = summarize(self.document());
        let measurement = Measurement::with_text(
            OBJECT_SUMMARY,
            summary.total as f64,
            MeasureUnit::Count,
            summary.label(),
        );
        self.report_measurement(measurement.clone());
        Ok(measurement)
    }

    /// The current read-out, or `None` when the selection changed since it was
    /// measured (a stale label is never returned).
    pub fn last_measurement(&self) -> Option<&str> {
        self.last_measurement
            .as_ref()
            .filter(|label| label.selection_revision == self.selection_revision)
            .map(|label| label.text.as_str())
    }

    /// Take the current read-out for display. It is consumed exactly once, so the
    /// status line cannot repeat an old value.
    pub fn take_measurement_label(&mut self) -> Option<String> {
        let label = self.last_measurement.take()?;
        (label.selection_revision == self.selection_revision).then_some(label.text)
    }

    /// The summary a caller can render without publishing a read-out.
    pub fn document_summary(&self) -> ObjectSummary {
        summarize(self.document())
    }
}

fn corners(bounds: Bounds) -> [Point3; 8] {
    std::array::from_fn(|index| {
        std::array::from_fn(|axis| {
            if index & (1 << axis) == 0 {
                bounds.min[axis]
            } else {
                bounds.max[axis]
            }
        })
    })
}

fn add_point(bounds: &mut Option<Bounds>, point: Point3) {
    if !point.iter().all(|value| value.is_finite()) {
        return;
    }
    match bounds {
        Some(current) => {
            for ((min, max), value) in current.min.iter_mut().zip(&mut current.max).zip(point) {
                *min = min.min(value);
                *max = max.max(value);
            }
        }
        None => {
            *bounds = Some(Bounds {
                min: point,
                max: point,
            })
        }
    }
}

fn add_bounds(bounds: &mut Option<Bounds>, extra: Bounds) {
    for corner in corners(extra) {
        add_point(bounds, corner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_trim_trailing_zeros_and_keep_millimetres() {
        assert_eq!(Measurement::format_value(5.0), "5");
        assert_eq!(Measurement::format_value(5.5), "5.5");
        assert_eq!(Measurement::format_value(5.0004), "5");
        assert_eq!(Measurement::format_value(-0.0), "0");
        assert_eq!(
            Measurement::new(DISTANCE, "Distance", 5.0, MeasureUnit::Millimetre).label(),
            "Distance = 5 mm"
        );
        assert_eq!(
            Measurement::new(ANGLE, "Angle", 90.0, MeasureUnit::Degree).label(),
            "Angle = 90 deg"
        );
    }

    #[test]
    fn a_degenerate_direction_and_a_spline_radius_are_refused() {
        assert!(
            measure::angle_degrees([0., 0., 0.], [0., 0., 0.], [0., 0., 0.], [1., 0., 0.]).is_err()
        );
        let spline = Curve::Nurbs(om9_core::geometry::NurbsCurve {
            degree: 2,
            poles: vec![[0., 0., 0.], [1., 1., 0.], [2., 0., 0.]],
            weights: vec![1., 1., 1.],
            knots: vec![0., 0., 0., 1., 1., 1.],
            domain: [0., 1.],
            periodic: false,
            reversed: false,
            seam: None,
        });
        assert!(measure_radius(&spline).is_err());
    }
}
