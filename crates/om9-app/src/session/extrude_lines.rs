// SPDX-License-Identifier: LGPL-2.1-or-later
use super::{Controller, context_presets::toggle};
use om9_addon_abi::session::{Event, Geo, Input, OptionToken, Reply, Want, dot, scale, sub};
use om9_core::{
    Error, Result,
    budget::Usage,
    curve_ops::extrude_lines::ControlProfiles,
    geometry::{Curve, Geometry, Point3, Transform},
    identity::{DocumentId, ObjectId},
    model::Entity,
    transaction::{Mutation, TransactionRequest},
};

#[cfg(feature = "occt-brep")]
pub(super) mod boundary;
pub(crate) mod tapered;

pub(crate) const CODE: &str = "tsExtrudeLines";
const LABEL: &str = "Extrude Lines";

#[derive(Clone, Copy)]
enum Stage {
    Distance,
    Boundary,
    BasePoint,
    DraftAngle,
    DirectionStart,
    DirectionEnd(Point3),
}

#[derive(Default)]
pub(super) struct Defaults {
    distance_mm: Option<f64>,
    both: bool,
    solid: bool,
    split_at_tangents: bool,
    delete: bool,
}

pub(super) struct ExtrudeLines {
    stamp: (DocumentId, u64),
    ids: Vec<ObjectId>,
    profiles: ControlProfiles,
    taper: Option<tapered::Taper>,
    base: Point3,
    axis: Point3,
    stage: Stage,
    to_boundary: bool,
    default_distance: Option<f64>,
    both: bool,
    solid: bool,
    split_at_tangents: bool,
    delete: bool,
}

impl ExtrudeLines {
    #[cfg(feature = "occt-brep")]
    fn solid_profile<'a>(
        &self,
        c: &Controller,
        curve: &'a Curve,
        world: Transform,
        limits: &om9_core::budget::Limits,
    ) -> Result<std::borrow::Cow<'a, Curve>> {
        let tolerance = c.document().effective_tolerance()?;
        om9_core::curve_ops::extrude_lines::prism_profile(
            curve,
            world,
            self.split_at_tangents,
            tolerance.absolute_mm / c.document().page_unit.factor(),
            tolerance.angular_rad,
            limits,
        )
    }

    pub(super) fn current(&self, c: &Controller) -> bool {
        self.stamp == (c.document, c.coordinator.epoch())
    }

    fn capture(c: &Controller, axis: Point3, is_tapered: bool) -> Result<Option<Self>> {
        let ids = c.selected_ordered();
        if ids.is_empty() {
            return Ok(None);
        }
        let doc = c.document();
        let mut inputs = Vec::with_capacity(ids.len());
        for id in &ids {
            let entity = doc
                .entities
                .get(id)
                .ok_or(Error::Stale("Extrude Lines source"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("Select curves for Extrude Lines"));
            };
            inputs.push((curve, entity.placement));
        }
        let tol = doc.effective_tolerance()?.absolute_mm / doc.page_unit.factor();
        let axis = om9_core::curve_ops::extrude_lines::default_axis(
            inputs.iter().copied(),
            axis,
            c.coordinator.limits(),
        )?;
        let profiles = ControlProfiles::capture(
            inputs.iter().copied(),
            tol,
            c.coordinator.limits(),
            &mut || Ok(()),
        )?;
        let taper = if is_tapered {
            Some(tapered::Taper::capture(c, &inputs, &profiles, tol)?)
        } else {
            None
        };
        let defaults = if is_tapered {
            &c.taper_defaults.common
        } else {
            &c.extrude_lines_defaults
        };
        Ok(Some(Self {
            stamp: (c.document, c.coordinator.epoch()),
            ids,
            base: profiles.start,
            profiles,
            taper,
            axis,
            stage: Stage::Distance,
            to_boundary: false,
            default_distance: defaults
                .distance_mm
                .map(|d| d / doc.page_unit.factor())
                .or(is_tapered.then_some(1.)),
            solid: defaults.solid,
            split_at_tangents: defaults.split_at_tangents,
            both: defaults.both,
            delete: defaults.delete,
        }))
    }

    fn reply(&self) -> Reply {
        let prompt = match self.stage {
            Stage::Distance => "Extrusion distance: enter a length or pick a point",
            Stage::Boundary => "Select boundary surface",
            Stage::DraftAngle => "Draft angle in degrees",
            Stage::BasePoint => "Pick the extrusion base point",
            Stage::DirectionStart => "Start of extrusion direction",
            Stage::DirectionEnd(_) => "End of extrusion direction",
        };
        let options = if matches!(self.stage, Stage::Distance) {
            if let Some(taper) = &self.taper {
                taper.options(self.solid, self.delete)
            } else {
                let mut options = vec![
                    OptionToken::action("Direction"),
                    OptionToken::toggle("BothSides", self.both),
                    OptionToken::toggle("DeleteInput", self.delete),
                ];
                if cfg!(feature = "occt-brep") {
                    options.push(OptionToken::toggle("Solid", self.solid));
                    options.push(OptionToken::action("ToBoundary"));
                    options.push(OptionToken::toggle(
                        "SplitAtTangents",
                        self.split_at_tangents,
                    ));
                }
                options.push(OptionToken::action("SetBasePoint"));
                options
            }
        } else if matches!(self.stage, Stage::Boundary) {
            if self.taper.is_some() {
                Vec::new()
            } else {
                vec![OptionToken::action("Direction")]
            }
        } else {
            Vec::new()
        };
        Reply {
            prompt: match (self.stage, self.default_distance) {
                (Stage::Distance, Some(d)) => {
                    format!("Extrusion distance <{}>", om9_addon_abi::session::fmt(d))
                }
                _ => prompt.into(),
            },
            options,
            want: if matches!(self.stage, Stage::Boundary) {
                Want::Objects
            } else if matches!(self.stage, Stage::DraftAngle) {
                Want::Number
            } else {
                Want::Point
            },
            ..Default::default()
        }
    }

    fn step(&mut self, c: &mut Controller, input: &Input) -> Result<Reply> {
        let k = c.document().page_unit.factor();
        let mut reply = self.reply();
        #[cfg(feature = "occt-brep")]
        if matches!(self.stage, Stage::Boundary) && matches!(input, Input::Selection | Input::Enter)
        {
            if c.selected().is_empty() && matches!(input, Input::Selection) {
                return Ok(reply);
            }
            let result = if self.taper.is_some() {
                self.finish_taper_boundary(c)
            } else {
                self.finish_boundary(c)
            };
            return match result {
                Err(error) if matches!(input, Input::Selection) => {
                    c.clear_selection();
                    reply.messages.push(error.to_string());
                    Ok(reply)
                }
                other => other,
            };
        }
        if matches!(self.stage, Stage::DraftAngle) {
            if let Input::Text { text } = input {
                self.taper
                    .as_mut()
                    .ok_or(Error::Missing("draft options"))?
                    .set_angle(text)?;
                self.stage = Stage::Distance;
            } else if matches!(input, Input::Enter) {
                self.stage = Stage::Distance;
            }
            return Ok(self.reply());
        }
        let offset = match input {
            Input::Start | Input::Selection => return Ok(reply),
            Input::Enter if matches!(self.stage, Stage::Distance) => scale(
                self.axis,
                self.default_distance.ok_or(Error::Missing(
                    "enter an extrusion distance or pick a point",
                ))?,
            ),
            Input::Enter => return Err(Error::Missing("pick an extrusion point")),
            Input::Option { name, value } => {
                match (name.as_str(), value.as_deref()) {
                    ("DraftAngle", value) if self.taper.is_some() => {
                        if let Some(s) = value {
                            self.taper.as_mut().unwrap().set_angle(s)?;
                        } else {
                            self.stage = Stage::DraftAngle;
                        }
                    }
                    ("FlipAngle", None) if self.taper.is_some() => {
                        let t = self.taper.as_mut().unwrap();
                        t.angle = -t.angle;
                    }
                    ("Corners", value) if self.taper.is_some() => {
                        self.taper.as_mut().unwrap().set_corners(value)?
                    }
                    ("ToBoundary", None) if cfg!(feature = "occt-brep") => {
                        self.both = false;
                        self.stage = Stage::Boundary;
                        self.to_boundary = true;
                        c.clear_selection();
                    }
                    ("BothSides", v) => self.both = toggle(self.both, v)?,
                    ("SplitAtTangents", v) if cfg!(feature = "occt-brep") => {
                        self.split_at_tangents = toggle(self.split_at_tangents, v)?
                    }
                    ("Solid", v) if cfg!(feature = "occt-brep") => {
                        self.solid = toggle(self.solid, v)?
                    }
                    ("DeleteInput", v) => self.delete = toggle(self.delete, v)?,
                    ("SetBasePoint", None) => self.stage = Stage::BasePoint,
                    ("Direction", None) => self.stage = Stage::DirectionStart,
                    _ => return Err(Error::Invalid("Extrude Lines option")),
                }
                return Ok(self.reply());
            }
            Input::Text { text } if matches!(self.stage, Stage::Distance) => {
                let mut d = om9_addon_abi::session::parse_number(text)
                    .ok_or(Error::Invalid("enter an extrusion distance"))?;
                if text.trim_end().ends_with("mm") {
                    d /= k;
                }
                scale(self.axis, d)
            }
            Input::Hover { point } | Input::Point { point } => {
                let at = point.map(|x| x / k);
                let click = matches!(input, Input::Point { .. });
                match self.stage {
                    Stage::DirectionEnd(start) => {
                        if click {
                            let delta = sub(at, start);
                            let length = delta[0].hypot(delta[1]).hypot(delta[2]);
                            if !length.is_finite() || length == 0. {
                                return Err(Error::Invalid("pick two distinct direction points"));
                            }
                            self.axis = delta.map(|v| v / length);
                            self.stage = if self.to_boundary {
                                Stage::Boundary
                            } else {
                                Stage::Distance
                            };
                            return Ok(self.reply());
                        }
                        reply.rubber_from = Some(start.map(|v| v * k));
                        return Ok(reply);
                    }
                    Stage::Boundary | Stage::DraftAngle => return Ok(reply),
                    Stage::Distance => scale(self.axis, dot(sub(at, self.base), self.axis)),
                    Stage::BasePoint | Stage::DirectionStart => {
                        if click {
                            if matches!(self.stage, Stage::BasePoint) {
                                self.base = at;
                                self.stage = Stage::Distance;
                            } else {
                                self.stage = Stage::DirectionEnd(at);
                            }
                        }
                        let mut reply = self.reply();
                        if let Stage::DirectionEnd(start) = self.stage {
                            reply.rubber_from = Some(start.map(|v| v * k));
                        }
                        return Ok(reply);
                    }
                }
            }
            _ => return Err(Error::Invalid("pick an extrusion point")),
        };
        if let Some(taper) = &self.taper {
            let h = dot(offset, self.axis);
            if matches!(input, Input::Hover { .. }) {
                return taper.preview(self, c, h, reply);
            }
            #[cfg(feature = "occt-brep")]
            return self.finish_taper(c, h, None);
            #[cfg(not(feature = "occt-brep"))]
            return Err(Error::Missing("exact BRep kernel"));
        }
        self.build(c, offset, matches!(input, Input::Hover { .. }), reply)
    }

    fn build(
        &self,
        c: &mut Controller,
        offset: Point3,
        preview: bool,
        mut reply: Reply,
    ) -> Result<Reply> {
        let k = c.document().page_unit.factor();
        if preview && offset == [0.; 3] {
            return Ok(reply);
        }
        let limits = c.coordinator.limits();
        let mut capped = Vec::new();
        let mut wire_ids = Vec::new();
        let mut wire_inputs = Vec::new();
        // The live wire outline is common to capped and uncapped extrusions.
        // Partition only for publication so hovering never calls the solid kernel.
        if self.solid && !preview {
            for id in &self.ids {
                let source = &c.document().entities[id];
                let Geometry::Curve(curve) = &source.geometry else {
                    return Err(Error::Stale("Extrude Lines source"));
                };
                let input = (curve, source.placement);
                if om9_core::curve_ops::is_closed(curve, 1e-7, limits)?
                    && let Some(normal) =
                        om9_core::curve_ops::extrude_lines::profile_plane([input], limits)?
                {
                    capped.push((*id, normal));
                } else {
                    wire_ids.push(*id);
                    wire_inputs.push(input);
                }
            }
        }
        let filtered;
        let (profiles, sources) = if capped.is_empty() {
            (Some(&self.profiles), self.ids.as_slice())
        } else if wire_inputs.is_empty() {
            (None, wire_ids.as_slice())
        } else {
            let tolerance = c.document().effective_tolerance()?.absolute_mm / k;
            filtered = ControlProfiles::capture(wire_inputs, tolerance, limits, &mut || Ok(()))?;
            (Some(&filtered), wire_ids.as_slice())
        };
        let wires = match profiles {
            Some(profiles) => {
                profiles.wires(offset, self.both, !self.delete, limits, &mut || Ok(()))?
            }
            None => Vec::new(),
        };
        let curved = profiles.map_or(&[][..], |p| p.curved.as_slice());
        let lo = if self.both {
            Some(offset.map(|v| -v))
        } else if self.delete {
            Some([0.; 3])
        } else {
            None
        };
        let shifts = [Some(offset), lo];
        let rail_count = curved
            .len()
            .checked_mul(1 + usize::from(lo.is_some()))
            .ok_or(Error::Overflow("Extrude Lines rails"))?;
        let count = wires
            .len()
            .checked_add(rail_count)
            .and_then(|n| n.checked_add(capped.len()))
            .ok_or(Error::Overflow("Extrude Lines outputs"))?;
        if count as u64 > limits.max_outputs {
            return Err(Error::Budget("Extrude Lines outputs"));
        }
        if preview {
            let mut held = Usage::array::<Geo>(count)?;
            held.check(limits)?;
            reply.preview = wires
                .into_iter()
                .map(|[p, q]| Geo::Line {
                    start: p.map(|v| v * k),
                    end: q.map(|v| v * k),
                })
                .collect();
            for rail in curved {
                for shift in shifts.into_iter().flatten() {
                    let t = Transform::translation(shift);
                    held.add(Usage::array::<Point3>(rail.display.len())?, limits)?;
                    let points = rail
                        .display
                        .iter()
                        .map(|p| Ok(t.point(*p)?.map(|v| v * k)))
                        .collect::<Result<Vec<_>>>()?;
                    reply.preview.push(Geo::Polyline {
                        points,
                        closed: false,
                    });
                }
            }
            return Ok(reply);
        }

        let mutations = count
            .checked_add(if self.delete { self.ids.len() } else { 0 })
            .ok_or(Error::Overflow("Extrude Lines transaction"))?;
        Usage::array::<Mutation>(mutations)?.check(limits)?;
        let mut changes = Vec::with_capacity(mutations);
        let mut ids = Vec::with_capacity(count);
        for [start, end] in wires {
            let mut entity = Entity::new(
                LABEL,
                Geometry::Curve(Curve::Line {
                    start,
                    end,
                    domain: [0., 1.],
                }),
                c.document().current_layer,
            );
            if entity.attributes.layer.is_some() {
                entity.attributes.color = None;
            }
            ids.push(entity.id);
            changes.push(Mutation::Insert {
                document: c.document,
                entity,
            });
        }
        let mut usage = Usage::array::<Mutation>(mutations)?;
        for rail in curved {
            let source = &c.document().entities[&sources[rail.source]];
            let cost = source.geometry.validate(limits)?;
            for shift in shifts.into_iter().flatten() {
                usage.add(cost, limits)?;
                let mut entity =
                    Entity::new(LABEL, source.geometry.clone(), c.document().current_layer);
                entity.placement = Transform::translation(shift).compose(source.placement)?;
                if entity.attributes.layer.is_some() {
                    entity.attributes.color = None;
                }
                ids.push(entity.id);
                changes.push(Mutation::Insert {
                    document: c.document,
                    entity,
                });
            }
        }
        if self.delete {
            for id in &self.ids {
                changes.push(Mutation::Delete {
                    source: c.document().stamp(*id)?,
                });
            }
        }
        if capped.is_empty() {
            c.publish_request_with_selection(TransactionRequest::new(LABEL, changes), Some(&ids))?;
        } else {
            #[cfg(feature = "occt-brep")]
            {
                let direction = if self.both { scale(offset, 2.) } else { offset };
                let start = Transform::translation(if self.both {
                    scale(offset, -1.)
                } else {
                    [0.; 3]
                });
                c.publish_extrude_solids(
                    (CODE, LABEL),
                    &capped,
                    changes,
                    ids,
                    usage,
                    |c, (id, normal), available| {
                        if dot(*normal, direction) == 0. {
                            return Err(Error::Invalid(
                                "extrusion direction lies in the profile plane",
                            ));
                        }
                        let source = &c.document().entities[id];
                        let Geometry::Curve(curve) = &source.geometry else {
                            return Err(Error::Stale("Extrude Lines source"));
                        };
                        let placement = start.compose(source.placement)?;
                        let profile = self.solid_profile(c, curve, source.placement, available)?;
                        let body = om9_kernel::extrusion::placed_solid(
                            &profile,
                            source.placement,
                            direction,
                            om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                            available,
                        )
                        .map_err(|e| Error::Io(format!("Extrude Lines: {e}")))?;
                        Ok((body, placement))
                    },
                )?;
            }
            #[cfg(not(feature = "occt-brep"))]
            return Err(Error::Invalid("Solid requires the exact BRep kernel"));
        }
        c.extrude_lines_defaults = Defaults {
            distance_mm: Some(dot(offset, self.axis) * k),
            solid: self.solid,
            split_at_tangents: self.split_at_tangents,
            both: self.both,
            delete: self.delete,
        };
        Ok(Reply {
            done: true,
            state: serde_json::json!(count),
            ..Default::default()
        })
    }
}

impl Controller {
    #[cfg(feature = "occt-brep")]
    pub(super) fn publish_extrude_solids<T>(
        &mut self,
        command: (&str, &str),
        sources: &[T],
        mut changes: Vec<Mutation>,
        mut ids: Vec<ObjectId>,
        mut usage: Usage,
        mut construct: impl FnMut(
            &Controller,
            &T,
            &om9_core::budget::Limits,
        ) -> Result<(om9_core::brep::Brep, Transform)>,
    ) -> Result<()> {
        let c = self;
        let result = (|| {
            let (mut guards, _storage) = c.reserve_solid_guard_vector(sources.len())?;
            let (mut proofs, _proof_storage) = c.reserve_solid_guard_vector(sources.len())?;
            for source in sources {
                let available = c.coordinator.limits().remaining_after(usage)?;
                let (body, placement) = construct(c, source, &available)?;
                usage.add(body.storage_usage(&available)?, c.coordinator.limits())?;
                let (candidate, guard, proof, storage) = c.reserve_attested_brep_candidate(body)?;
                guards.push(guard);
                let mut entity = Entity::new(
                    command.1,
                    Geometry::Brep(candidate),
                    c.document().current_layer,
                );
                entity.placement = placement;
                if entity.attributes.layer.is_some() {
                    entity.attributes.color = None;
                }
                ids.push(entity.id);
                proofs.push((entity.id, proof, storage));
                changes.push(Mutation::Insert {
                    document: c.document,
                    entity,
                });
            }
            c.publish_request_with_reserved_selection(
                TransactionRequest::new(command.1, changes),
                Some(&ids),
                Some(&mut guards),
            )?;
            for (id, proof, storage) in proofs {
                if let Err(e) = c.retain_published_brep_proof(id, proof, storage) {
                    c.events.emit(
                        crate::command_events::EventKind::Warn,
                        command.0,
                        &format!("Could not retain BRep save evidence: {e}"),
                        None,
                    );
                }
            }
            Ok(())
        })();
        if result.is_err() {
            c.coordinator.prune_orphaned_brep_representations();
        }
        result
    }

    pub(super) fn start_extrude_lines(&mut self) -> Result<()> {
        self.start_interactive_session(CODE, LABEL, "om9".into(), false, false)
    }
    pub(super) fn extrude_lines_step(&mut self, event: &Event) -> Result<Reply> {
        let opening = self.extrude_lines.is_none();
        if opening && matches!(event.input, Input::Start | Input::Enter) {
            self.extrude_lines =
                ExtrudeLines::capture(self, event.plane.normal(), event.command == tapered::CODE)?;
        }
        let Some(mut state) = self.extrude_lines.take() else {
            return Ok(Reply {
                prompt: "Select curves to extrude, Enter when done".into(),
                want: Want::Objects,
                ..Default::default()
            });
        };
        if !state.current(self) {
            self.cancel_addon_session();
            return Err(Error::Stale("Extrude Lines inputs changed"));
        }
        let result = if opening {
            Ok(state.reply())
        } else {
            state.step(self, &event.input)
        };
        if !result.as_ref().is_ok_and(|r| r.done) {
            self.extrude_lines = Some(state);
        }
        result
    }

    pub(crate) fn extrude_profile_wires(&self) -> Option<impl Iterator<Item = [Point3; 2]> + '_> {
        let profiles = self
            .extrude_lines
            .as_ref()
            .map(|s| &s.profiles)
            .or_else(|| self.extrude_point.as_ref().map(|s| &s.profiles))?;
        Some(profiles.source_wires())
    }
    pub(crate) fn extrude_picking_boundary(&self) -> bool {
        self.extrude_lines
            .as_ref()
            .is_some_and(|s| matches!(s.stage, Stage::Boundary))
            || self
                .extrude_point
                .as_ref()
                .is_some_and(|s| s.picking_boundary)
    }

    pub(super) fn restore_extrude_inputs(&mut self) {
        if let Some(s) = self.extrude_lines.take()
            && s.to_boundary
            && s.current(self)
        {
            let _ = self.select_objects(&s.ids);
        }
    }

    pub(crate) fn extrude_distance_axis(&self) -> Option<(Point3, Point3)> {
        let state = self.extrude_lines.as_ref()?;
        matches!(state.stage, Stage::Distance).then_some((state.base, state.axis))
    }
}
