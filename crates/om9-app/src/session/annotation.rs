// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-03-21: thirteen catalog commands share one typed model-space owner.
use super::Controller;
use om9_core::{
    Error, Result,
    annotation::{Annotation, Kind, SLOT},
    geometry::{Curve, Point3},
    identity::SourceStamp,
    model::Entity,
    transaction::Mutation,
};
pub const FEATURES: [&str; 13] = [
    "OM9-MEASURE-004",
    "OM9-MEASURE-006",
    "OM9-MEASURE-007",
    "OM9-MEASURE-008",
    "OM9-MEASURE-009",
    "OM9-MEASURE-010",
    "OM9-MEASURE-011",
    "OM9-MEASURE-012",
    "OM9-MEASURE-013",
    "OM9-MEASURE-014",
    "OM9-MEASURE-015",
    "OM9-MEASURE-016",
    "OM9-MEASURE-017",
];
pub fn handles(feature: &str) -> bool {
    FEATURES.contains(&feature)
}
pub fn from_alias(text: &str) -> Option<&'static str> {
    let aliases = [
        "Dim",
        "DimVertical",
        "DimAngle",
        "DimRotated",
        "DimAligned",
        "DimDiameter",
        "DimRadius",
        "Leader",
        "TextBlock",
        "DimEdit",
        "gvEditText",
        "DimOptions",
        "DimRecenter",
    ];
    aliases
        .iter()
        .position(|a| a.eq_ignore_ascii_case(text))
        .or_else(|| FEATURES.iter().position(|f| f.eq_ignore_ascii_case(text)))
        .map(|i| FEATURES[i])
        .or_else(|| {
            if text.eq_ignore_ascii_case("DimText") {
                Some(FEATURES[8])
            } else {
                None
            }
        })
}
#[derive(Debug, Clone)]
pub struct Pending {
    pub feature: &'static str,
    pub record: Annotation,
    pub points: Vec<Point3>,
    pub source: Option<SourceStamp>,
    pub plane: om9_scene::camera::CPlane,
    pub unit: om9_core::geometry::Unit,
}
impl Pending {
    fn editing(&self) -> bool {
        self.feature == FEATURES[9] || self.feature == FEATURES[10] || self.feature == FEATURES[11]
    }
    fn expected(&self) -> usize {
        match self.record.kind {
            Kind::Text => 1,
            Kind::Radius | Kind::Diameter => 1,
            Kind::Angular => 4,
            _ => 3,
        }
    }
    fn text_stage(&self) -> bool {
        self.feature == FEATURES[10]
            || (!self.editing()
                && matches!(self.record.kind, Kind::Leader | Kind::Text)
                && self.points.len() == self.expected())
    }
    pub fn prompt(&self) -> &'static str {
        if self.text_stage() {
            return "Annotation text: ";
        }
        if self.editing() {
            return "Edit annotation options; Enter to apply: ";
        }
        match (self.record.kind, self.points.len()) {
            (Kind::Text, _) | (Kind::Radius | Kind::Diameter, _) => "Annotation text position: ",
            (Kind::Angular, 0) => "Angle vertex: ",
            (Kind::Angular, 1) => "First ray point: ",
            (Kind::Angular, 2) => "Second ray point: ",
            (_, 0) => "First witness/leader tip: ",
            (_, 1) => "Second witness/leader elbow: ",
            _ => "Annotation text position: ",
        }
    }
    fn candidate(&self, points: &[Point3]) -> Annotation {
        let mut r = self.record.clone();
        if points.len() == self.expected() {
            r.text_point = *points.last().unwrap();
            if !matches!(r.kind, Kind::Radius | Kind::Diameter) {
                r.points = points[..points.len() - 1].to_vec();
            }
        }
        r
    }
}
impl Controller {
    pub fn begin_annotation(&mut self, feature: &str) -> Result<()> {
        let index = FEATURES
            .iter()
            .position(|f| *f == feature)
            .ok_or(Error::Unsupported("annotation command"))?;
        let feature = FEATURES[index];
        let plane = self.workspace.views[&self.workspace.active].cplane;
        let kind = [
            Kind::Horizontal,
            Kind::Vertical,
            Kind::Angular,
            Kind::Rotated,
            Kind::Aligned,
            Kind::Diameter,
            Kind::Radius,
            Kind::Leader,
            Kind::Text,
        ];
        let mut record = Annotation {
            kind: kind.get(index).copied().unwrap_or(Kind::Text),
            points: vec![],
            x: plane.x,
            y: plane.y,
            text_point: plane.origin,
            text: None,
            rotation_degrees: 0.,
            style: self.annotation_style.clone(),
        };
        let mut source = None;
        if index >= 9 {
            let ids = self.selected_ordered();
            if ids.is_empty() && index == 11 {
            } else {
                if ids.len() != 1 {
                    return Err(Error::Invalid("select one annotation"));
                }
                if !self.document().permissions(ids[0], false)?.editable {
                    return Err(Error::Permission("annotation edit"));
                }
                record = Annotation::read(&self.document().entities[&ids[0]])?;
                record.layout(self.coordinator.limits())?;
                source = Some(self.document().stamp(ids[0])?);
                if (index == 9 || index == 12) && matches!(record.kind, Kind::Text | Kind::Leader) {
                    return Err(Error::Invalid("select a dimension"));
                }
                if index == 10 && !matches!(record.kind, Kind::Text | Kind::Leader) {
                    return Err(Error::Invalid("select a Text Block or Leader"));
                }
            }
        } else if matches!(record.kind, Kind::Radius | Kind::Diameter) {
            let ids = self.selected_ordered();
            if ids.len() != 1 {
                return Err(Error::Invalid("select one circle or arc"));
            }
            let curves = super::measure::selected_curves(self.document(), &ids)?;
            let Curve::Circle {
                center,
                x,
                y,
                radius,
                ..
            } = &curves[0]
            else {
                return Err(Error::Invalid("radial dimension needs a circle/arc"));
            };
            record.x = *x;
            record.y = *y;
            record.points = vec![*center, std::array::from_fn(|i| center[i] + x[i] * radius)];
        }
        if index == 12 {
            record.text_point = record.layout(self.coordinator.limits())?.center;
            self.commit_annotation_edit(feature, source.unwrap(), record)?;
            return Ok(());
        }
        self.annotation_pending = Some(Pending {
            feature,
            record,
            points: vec![],
            source,
            plane,
            unit: self.document().page_unit,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "started",
            None,
        );
        Ok(())
    }
    fn commit_annotation_edit(
        &mut self,
        feature: &'static str,
        source: SourceStamp,
        record: Annotation,
    ) -> Result<()> {
        self.document().resolve(&source)?;
        let geometry = record.geometry(self.coordinator.limits())?;
        let extension = record.extension(self.coordinator.limits())?;
        let mut extensions = self.document().entities[&source.object].extensions.clone();
        extensions.insert(SLOT.into(), extension);
        self.commit_utility_mutations(
            "Edit Annotation",
            vec![
                Mutation::Geometry { source, geometry },
                Mutation::Extensions { source, extensions },
            ],
        )?;
        self.events.emit(
            crate::command_events::EventKind::Finished,
            feature,
            "annotation edited",
            Some(crate::command_events::EventOutcome::Committed),
        );
        Ok(())
    }
    fn finish_annotation(&mut self, p: &Pending, record: Annotation) -> Result<()> {
        if let Some(source) = p.source {
            return self.commit_annotation_edit(p.feature, source, record);
        }
        if p.feature == FEATURES[11] {
            record.style.validate()?;
            self.annotation_style = record.style;
            self.events.emit(
                crate::command_events::EventKind::Finished,
                p.feature,
                "session dimension defaults updated",
                Some(crate::command_events::EventOutcome::Committed),
            );
            return Ok(());
        }
        let geometry = record.geometry(self.coordinator.limits())?;
        let extension = record.extension(self.coordinator.limits())?;
        let mut entity = Entity::new("Annotation", geometry, self.document().current_layer);
        entity.extensions.insert(SLOT.into(), extension);
        self.commit_entity(p.feature, "Annotation", entity)
    }
    pub fn accept_annotation_point(&mut self, point: Point3) -> Result<()> {
        let mut p = self
            .annotation_pending
            .take()
            .ok_or(Error::Missing("annotation run"))?;
        if p.editing() || p.text_stage() {
            self.annotation_pending = Some(p);
            return Err(Error::Invalid("annotation expects text/options"));
        }
        p.points.push(point);
        if p.points.len() < p.expected() {
            self.annotation_pending = Some(p);
            return Ok(());
        }
        let record = p.candidate(&p.points);
        if matches!(record.kind, Kind::Text | Kind::Leader) {
            p.record = record;
            self.annotation_pending = Some(p);
            return Ok(());
        }
        match self.finish_annotation(&p, record) {
            Ok(()) => Ok(()),
            Err(e) => {
                p.points.pop();
                self.annotation_pending = Some(p);
                Err(e)
            }
        }
    }
    pub fn submit_annotation_text(&mut self, text: &str) -> Option<Result<()>> {
        let mut p = self.annotation_pending.take()?;
        if text.eq_ignore_ascii_case("cancel") || text.eq_ignore_ascii_case("esc") {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                p.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
            return Some(Ok(()));
        }
        let result = (|| {
            if p.text_stage() {
                if text.trim().is_empty() {
                    return Err(Error::Invalid("empty annotation text"));
                }
                let mut r = p.record.clone();
                r.text = Some(text.to_owned());
                self.finish_annotation(&p, r)?;
                return Ok(true);
            }
            if text.trim().is_empty() && p.editing() {
                self.finish_annotation(&p, p.record.clone())?;
                return Ok(true);
            }
            if let Some((key, value)) = text.split_once('=') {
                let mut r = p.record.clone();
                match key.to_ascii_lowercase().as_str() {
                    "height" => r.style.height_mm = super::transform::parse_number(value)?,
                    "arrow" => r.style.arrow_mm = super::transform::parse_number(value)?,
                    "scale" => r.style.scale = super::transform::parse_number(value)?,
                    "precision" => {
                        r.style.precision = value
                            .parse()
                            .map_err(|_| Error::Invalid("dimension precision 0..8"))?
                    }
                    "angle" => r.rotation_degrees = super::transform::parse_number(value)?,
                    "text" => {
                        r.text = if value.eq_ignore_ascii_case("auto") {
                            None
                        } else {
                            Some(value.into())
                        }
                    }
                    "position" | "point1" | "point2" | "point3" => {
                        let input = om9_core::coords::parse_input(value, p.unit)?;
                        let mut point =
                            om9_core::coords::resolve_with_base(input, Some(&p.plane), None)?.xyz();
                        if let Some(source) = p.source {
                            point = om9_core::top11::inverse_affine(
                                self.document().entities[&source.object].placement,
                            )?
                            .point(point)?;
                        }
                        if key.eq_ignore_ascii_case("position") {
                            r.text_point = point;
                        } else {
                            let i = key[5..].parse::<usize>().unwrap() - 1;
                            let dest = r
                                .points
                                .get_mut(i)
                                .ok_or(Error::Invalid("dimension witness index"))?;
                            *dest = point;
                        }
                    }
                    _ => return Err(Error::Invalid("annotation option")),
                }
                r.style.validate()?;
                if !r.rotation_degrees.is_finite() {
                    return Err(Error::Invalid("dimension angle"));
                }
                p.record = r;
                return Ok(false);
            }
            if p.editing() {
                return Err(Error::Invalid("use annotation option=value or Enter"));
            }
            let point = om9_core::coords::resolve_with_base(
                om9_core::coords::parse_input(text, p.unit)?,
                Some(&p.plane),
                self.relative_base(),
            )?
            .xyz();
            self.annotation_pending = Some(p.clone());
            self.accept_annotation_point(point)?;
            Ok(true)
        })();
        match result {
            Ok(true) => Some(Ok(())),
            Ok(false) => {
                self.annotation_pending = Some(p);
                Some(Ok(()))
            }
            Err(e) => {
                self.annotation_pending = Some(p);
                Some(Err(e))
            }
        }
    }
    pub fn annotation_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(p) = &self.annotation_pending else {
            return vec![];
        };
        if p.text_stage() {
            return vec![("Cancel", "Cancel")];
        }
        let mut result = vec![
            ("Height=", "Height="),
            ("Arrow=", "Arrow="),
            ("Precision=", "Precision="),
            ("Scale=", "Scale="),
        ];
        if p.record.kind == Kind::Rotated {
            result.push(("Angle=", "Angle="));
        }
        if p.editing() && p.feature != FEATURES[11] {
            result.extend([
                ("Text=", "Text="),
                ("Position=", "Position="),
                ("Point1=", "Point1="),
                ("Point2=", "Point2="),
            ]);
            if p.record.kind == Kind::Angular {
                result.push(("Point3=", "Point3="));
            }
        }
        result.push(("Cancel", "Cancel"));
        result
    }
    pub fn annotation_preview(
        &self,
        point: Point3,
    ) -> Result<Option<(Annotation, om9_core::annotation::Layout)>> {
        let Some(p) = &self.annotation_pending else {
            return Ok(None);
        };
        let mut points = p.points.clone();
        if !p.editing() && !p.text_stage() {
            points.push(point);
            if points.len() != p.expected() {
                return Ok(None);
            }
        }
        let mut r = if p.editing() || p.text_stage() {
            p.record.clone()
        } else {
            p.candidate(&points)
        };
        if matches!(r.kind, Kind::Text | Kind::Leader) && r.text.is_none() {
            r.text = Some("Text".into());
        }
        if let Some(source) = p.source {
            r = r.placed(self.document().entities[&source.object].placement)?;
        }
        let layout = r.layout(self.coordinator.limits())?;
        Ok(Some((r, layout)))
    }
}
