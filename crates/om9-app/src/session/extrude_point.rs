// SPDX-License-Identifier: LGPL-2.1-or-later
use super::{Controller, context_presets::toggle};
use om9_addon_abi::session::{Event, Geo, Input, OptionToken, Reply, Want};
use om9_core::{
    Error, Result,
    brep::CanonicalBrep,
    budget::Usage,
    curve_ops::extrude_lines::ControlProfiles,
    geometry::{Geometry, Point3},
    identity::{DocumentId, ObjectId},
};

pub(crate) const CODE: &str = "ExtrudeCrvToPoint";
const LABEL: &str = "Extrude to Point";

#[derive(Clone, Copy, Default)]
pub(super) struct Defaults {
    solid: bool,
    delete: bool,
    split: bool,
}

pub(super) struct ExtrudePoint {
    stamp: (DocumentId, u64),
    ids: Vec<ObjectId>,
    pub(super) profiles: ControlProfiles,
    options: Defaults,
    pub(super) picking_boundary: bool,
    boundary: Option<CanonicalBrep>,
}

impl ExtrudePoint {
    pub(super) fn current(&self, controller: &Controller) -> bool {
        self.stamp == (controller.document, controller.coordinator.epoch())
    }

    fn capture(c: &Controller) -> Result<Option<Self>> {
        let ids = c.selected_ordered();
        if ids.is_empty() {
            return Ok(None);
        }
        let doc = c.document();
        let mut inputs = Vec::with_capacity(ids.len());
        for id in &ids {
            let source = doc
                .entities
                .get(id)
                .ok_or(Error::Stale("Extrude to Point source"))?;
            let Geometry::Curve(curve) = &source.geometry else {
                return Err(Error::Invalid("Select curves to extrude"));
            };
            inputs.push((curve, source.placement));
        }
        let tolerance = doc.effective_tolerance()?.absolute_mm / doc.page_unit.factor();
        let profiles =
            ControlProfiles::capture(inputs, tolerance, c.coordinator.limits(), &mut || Ok(()))?;
        Ok(Some(Self {
            stamp: (c.document, c.coordinator.epoch()),
            ids,
            profiles,
            options: c.extrude_point_defaults,
            picking_boundary: false,
            boundary: None,
        }))
    }

    fn reply(&self) -> Reply {
        if self.picking_boundary {
            return Reply {
                prompt: "Select a boundary surface".into(),
                want: Want::Objects,
                ..Default::default()
            };
        }
        let mut options = vec![
            OptionToken::toggle("Solid", self.options.solid),
            OptionToken::toggle("DeleteInput", self.options.delete),
        ];
        if self.boundary.is_none() {
            options.push(OptionToken::action("ToBoundary"));
        }
        options.push(OptionToken::toggle("SplitAtTangents", self.options.split));
        Reply {
            prompt: "Point to extrude to".into(),
            options,
            want: Want::Point,
            ..Default::default()
        }
    }

    fn step(&mut self, c: &mut Controller, input: &Input) -> Result<Reply> {
        let mut reply = self.reply();
        if self.picking_boundary {
            #[cfg(feature = "occt-brep")]
            if matches!(input, Input::Selection | Input::Enter) && !c.selected().is_empty() {
                match super::extrude_lines::boundary::boundary_face(c) {
                    Ok(face) => {
                        self.boundary = Some(face);
                        self.picking_boundary = false;
                        c.select_objects(&self.ids)?;
                        return Ok(self.reply());
                    }
                    Err(e) => {
                        c.clear_selection();
                        reply.messages.push(e.to_string());
                    }
                }
            }
            return Ok(reply);
        }
        match input {
            Input::Option { name, value } => {
                let v = value.as_deref();
                match name.as_str() {
                    "Solid" => self.options.solid = toggle(self.options.solid, v)?,
                    "DeleteInput" => self.options.delete = toggle(self.options.delete, v)?,
                    "SplitAtTangents" => self.options.split = toggle(self.options.split, v)?,
                    "ToBoundary" if v.is_none() && self.boundary.is_none() => {
                        self.picking_boundary = true;
                        c.clear_selection();
                    }
                    _ => return Err(Error::Invalid("Extrude to Point option")),
                }
                Ok(self.reply())
            }
            Input::Hover { point } => {
                let scale = c.document().page_unit.factor();
                let limits = c.coordinator.limits();
                for station in self.profiles.apex_preview_stations() {
                    if reply.preview.len() as u64 >= limits.max_outputs {
                        return Err(Error::Budget("Extrude to Point preview"));
                    }
                    reply.preview.push(Geo::Line {
                        start: station.map(|v| v * scale),
                        end: *point,
                    });
                }
                Ok(reply)
            }
            Input::Point { point } => {
                self.finish(c, point.map(|v| v / c.document().page_unit.factor()))
            }
            Input::Selection | Input::Start => Ok(reply),
            _ => Err(Error::Invalid("Pick a point to extrude to")),
        }
    }

    #[cfg(feature = "occt-brep")]
    fn finish(&self, c: &mut Controller, apex: Point3) -> Result<Reply> {
        use om9_core::{
            curve_ops, geometry::Transform, top11::inverse_affine, transaction::Mutation,
        };
        use om9_kernel::{creation::DEFAULT_DEFLECTION_MM, extrusion, transform_ops};
        let limits = c.coordinator.limits();
        if self.ids.len() as u64 > limits.max_outputs {
            return Err(Error::Budget("Extrude to Point outputs"));
        }
        let count = self.ids.len() * (1 + usize::from(self.options.delete));
        let mut usage = Usage::array::<Mutation>(count)?;
        if let Some(face) = &self.boundary {
            usage.add(face.storage_usage(limits)?, limits)?;
        }
        usage.check(limits)?;
        let mut changes = Vec::with_capacity(count);
        if self.options.delete {
            for id in &self.ids {
                changes.push(Mutation::Delete {
                    source: c.document().stamp(*id)?,
                });
            }
        }
        let tol = c.document().effective_tolerance()?;
        let distance = tol.absolute_mm / c.document().page_unit.factor();
        let native_error =
            |e: om9_kernel::creation::CreationError| Error::Io(format!("Extrude to Point: {e}"));
        c.publish_extrude_solids(
            (CODE, LABEL),
            &self.ids,
            changes,
            Vec::with_capacity(self.ids.len()),
            usage,
            |c, id, room| {
                let source = &c.document().entities[id];
                let Geometry::Curve(curve) = &source.geometry else {
                    return Err(Error::Stale("Extrude to Point source"));
                };
                let placement = source.placement;
                let inverse = inverse_affine(placement)?;
                let profile = curve_ops::extrude_lines::prism_profile(
                    curve,
                    placement,
                    self.options.split,
                    distance,
                    tol.angular_rad,
                    room,
                )?;
                let capped = self.options.solid
                    && curve_ops::is_closed(curve, distance, room)?
                    && curve_ops::extrude_lines::profile_plane([(curve, placement)], room)?
                        .is_some();
                let local;
                let boundary = match &self.boundary {
                    Some(face) if placement != Transform::identity() => {
                        local = transform_ops::transform_brep(
                            face,
                            &inverse,
                            DEFAULT_DEFLECTION_MM,
                            room,
                        )
                        .map_err(|e| Error::Io(format!("Extrude to Point boundary: {e}")))?
                        .into_canonical_and_representation()
                        .0;
                        Some(&local)
                    }
                    other => other.as_ref(),
                };
                let body = extrusion::to_point(
                    &profile,
                    inverse.point(apex)?,
                    capped,
                    boundary,
                    DEFAULT_DEFLECTION_MM,
                    room,
                )
                .map_err(native_error)?;
                Ok((body, placement))
            },
        )?;
        c.extrude_point_defaults = self.options;
        Ok(Reply {
            done: true,
            state: serde_json::json!(self.ids.len()),
            ..Default::default()
        })
    }
    #[cfg(not(feature = "occt-brep"))]
    fn finish(&self, _: &mut Controller, _: Point3) -> Result<Reply> {
        Err(Error::Missing("exact BRep kernel"))
    }
}

impl Controller {
    pub(super) fn start_extrude_point(&mut self) -> Result<()> {
        if !cfg!(feature = "occt-brep") {
            return Err(Error::Missing("exact BRep kernel"));
        }
        self.start_interactive_session(CODE, LABEL, "om9".into(), false, false)
    }

    pub(super) fn extrude_point_step(&mut self, event: &Event) -> Result<Reply> {
        let opening = self.extrude_point.is_none();
        if opening && matches!(event.input, Input::Start | Input::Enter) {
            self.extrude_point = ExtrudePoint::capture(self)?;
        }
        let Some(mut state) = self.extrude_point.take() else {
            return Ok(Reply {
                prompt: "Select curves to extrude, Enter when done".into(),
                want: Want::Objects,
                ..Default::default()
            });
        };
        if !state.current(self) {
            self.cancel_addon_session();
            return Err(Error::Stale("Extrude to Point inputs changed"));
        }
        let result = if opening {
            Ok(state.reply())
        } else {
            state.step(self, &event.input)
        };
        if !result.as_ref().is_ok_and(|r| r.done) {
            self.extrude_point = Some(state);
        }
        result
    }
    pub(super) fn restore_extrude_point_inputs(&mut self) {
        if let Some(state) = self.extrude_point.take()
            && state.picking_boundary
            && state.current(self)
        {
            let _ = self.select_objects(&state.ids);
        }
    }
}
