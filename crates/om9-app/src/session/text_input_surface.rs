// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed text handling for pending surface commands.
//!
//! One method per pending family: option tokens, point/parameter input and the atomic commit.
//! Extracted unchanged from `text_input.rs`; each caller returns the stage result immediately.
use super::*;

impl Controller {
    pub(super) fn submit_history_options_text(&mut self, text: &str) -> Result<()> {
        let pending = self
            .history_options_pending
            .as_ref()
            .expect("history options")
            .clone();
        let mut closed = pending.closed;
        let mut flip = pending.flip;
        let mut slash = false;
        let mut option_only = false;
        let mut closed_requested = false;
        if !text.is_empty() {
            for token in text.to_ascii_lowercase().split([',', ' ']) {
                match token.trim() {
                    "closed" | "yes" => {
                        closed = true;
                        closed_requested = true;
                    }
                    "closed=yes" => {
                        closed = true;
                        closed_requested = true;
                        option_only = true;
                    }
                    "closed=no" => {
                        closed = false;
                        closed_requested = true;
                        option_only = true;
                    }
                    "flip" => flip = true,
                    "flip=yes" => {
                        flip = true;
                        option_only = true;
                    }
                    "flip=no" => {
                        flip = false;
                        option_only = true;
                    }
                    "slash" => slash = true,
                    "open" | "no" | "simple" => {}
                    "" => {}
                    _ => return Err(Error::Invalid("sweep option")),
                }
            }
        }
        if closed_requested && !pending.closed_available {
            return Err(Error::Unsupported(
                "closed sweep needs a closed rail and two profiles",
            ));
        }
        if option_only && !slash {
            self.history_options_pending = Some(HistoryOptionsPending {
                closed,
                flip,
                ..pending
            });
            return Ok(());
        }
        if slash {
            if pending.rail_count != 2 {
                return Err(Error::Invalid("slash needs two rails"));
            }
            self.history_options_pending = None;
            self.slash_pending = Some(SlashPending {
                feature: pending.feature,
                curves: pending.curves,
                ids: pending.ids,
                rail_count: pending.rail_count,
                closed,
                flip,
                points: Vec::new(),
            });
            return Ok(());
        }
        let brep = self.build_sweep_history(
            pending.feature,
            &pending.curves,
            pending.rail_count,
            closed,
            flip,
            None,
        )?;
        let entity = Entity::new(
            surface_edit::label(pending.feature),
            Geometry::Brep(brep),
            self.document().current_layer,
        );
        let mut inputs = Vec::with_capacity(pending.ids.len());
        for (index, id) in pending.ids.iter().enumerate() {
            inputs.push(SourceBinding {
                role: if index < pending.rail_count {
                    "rail"
                } else {
                    "profile"
                }
                .into(),
                source: self.document().stamp(*id)?,
                selection: Selection::Whole,
            });
        }
        self.history_options_pending = None;
        return self.commit_entity_with_history(
            pending.feature,
            surface_edit::label(pending.feature),
            entity,
            if pending.feature == surface_edit::SWEEP2_HISTORY {
                "Sweep2History"
            } else {
                "Sweep1History"
            },
            inputs,
        );
    }

    pub(super) fn submit_fillet_text(&mut self, text: &str) -> Result<()> {
        let mut pending = self.fillet_pending.take().expect("fillet");
        let trimmed = text.trim();
        if pending.feature == surface_edit::VARIABLE_FILLET {
            let (start_input, end_input) = if let Some(rest) = trimmed.strip_prefix("Start=") {
                (Some(parse_fillet_radius(rest)?), None)
            } else if let Some(rest) = trimmed.strip_prefix("start=") {
                (Some(parse_fillet_radius(rest)?), None)
            } else if let Some(rest) = trimmed.strip_prefix("End=") {
                (None, Some(parse_fillet_radius(rest)?))
            } else if let Some(rest) = trimmed.strip_prefix("end=") {
                (None, Some(parse_fillet_radius(rest)?))
            } else {
                let mut parts = trimmed.split(',');
                let start = parts
                    .next()
                    .ok_or(Error::Invalid("fillet radius"))
                    .and_then(parse_fillet_radius)?;
                let end = parts
                    .next()
                    .ok_or(Error::Invalid("fillet radius"))
                    .and_then(parse_fillet_radius)?;
                if parts.next().is_some() {
                    return Err(Error::Invalid("fillet radii"));
                }
                (Some(start), Some(end))
            };
            if let Some(start) = start_input {
                pending.start_radius = Some(start);
            }
            if let Some(end) = end_input {
                pending.end_radius = Some(end);
            }
            let (Some(start), Some(end)) = (pending.start_radius, pending.end_radius) else {
                self.fillet_pending = Some(pending);
                return Ok(());
            };
            let geometry = surface_edit::apply_fillet(
                &pending.first,
                &pending.second,
                start,
                end,
                self.coordinator.limits(),
            )?;
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        let radius_text = trimmed
            .strip_prefix("Radius=")
            .or_else(|| trimmed.strip_prefix("radius="))
            .unwrap_or(trimmed);
        let radius = parse_fillet_radius(radius_text)?;
        let geometry = match surface_edit::apply_fillet(
            &pending.first,
            &pending.second,
            radius,
            radius,
            self.coordinator.limits(),
        ) {
            Ok(geometry) => geometry,
            Err(error) => {
                self.fillet_pending = Some(pending);
                return Err(error);
            }
        };
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_variable_chamfer_text(&mut self, text: &str) -> Result<()> {
        let trimmed = text.trim();
        let (start_input, end_input) = if let Some(rest) = trimmed.strip_prefix("Start=") {
            (Some(parse_chamfer_setback(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("start=") {
            (Some(parse_chamfer_setback(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("End=") {
            (None, Some(parse_chamfer_setback(rest)?))
        } else if let Some(rest) = trimmed.strip_prefix("end=") {
            (None, Some(parse_chamfer_setback(rest)?))
        } else {
            let mut parts = trimmed.split(',');
            let start = parts
                .next()
                .ok_or(Error::Invalid("chamfer setbacks"))
                .and_then(parse_chamfer_setback)?;
            let end = parts
                .next()
                .ok_or(Error::Invalid("chamfer setbacks"))
                .and_then(parse_chamfer_setback)?;
            if parts.next().is_some() {
                return Err(Error::Invalid("chamfer setbacks"));
            }
            (Some(start), Some(end))
        };
        let mut pending = self
            .variable_chamfer_pending
            .take()
            .expect("variable chamfer");
        if let Some(start) = start_input {
            pending.start_setback = Some(start);
        }
        if let Some(end) = end_input {
            pending.end_setback = Some(end);
        }
        let (Some(start), Some(end)) = (pending.start_setback, pending.end_setback) else {
            self.variable_chamfer_pending = Some(pending);
            return Ok(());
        };
        let geometry = match surface_edit::apply_variable_chamfer(
            &pending.first,
            &pending.second,
            start,
            end,
            self.coordinator.limits(),
        ) {
            Ok(geometry) => geometry,
            Err(error) => {
                self.variable_chamfer_pending = Some(pending);
                return Err(error);
            }
        };
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_chamfer_text(&mut self, text: &str) -> Result<()> {
        let setback_text = text
            .strip_prefix("Setback=")
            .or_else(|| text.strip_prefix("setback="))
            .unwrap_or(text);
        let setback: f64 = setback_text
            .parse()
            .map_err(|_| Error::Invalid("chamfer setback"))?;
        if !setback.is_finite() || setback <= 0.0 {
            return Err(Error::Invalid("chamfer setback"));
        }
        let pending = self.chamfer_pending.as_ref().expect("chamfer").clone();
        let geometry = surface_edit::apply_chamfer(
            &pending.first,
            &pending.second,
            setback,
            self.coordinator.limits(),
        )?;
        self.chamfer_pending = None;
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_variable_blend_text(&mut self, text: &str) -> Result<()> {
        let trimmed = text.trim();
        let (start_input, end_input) = if let Some(rest) = trimmed.strip_prefix("Start=") {
            (Some(parse_blend_scale(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("start=") {
            (Some(parse_blend_scale(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("End=") {
            (None, Some(parse_blend_scale(rest)?))
        } else if let Some(rest) = trimmed.strip_prefix("end=") {
            (None, Some(parse_blend_scale(rest)?))
        } else {
            let mut parts = trimmed.split(',');
            let start = parts
                .next()
                .ok_or(Error::Invalid("variable blend scale"))
                .and_then(parse_blend_scale)?;
            let end = parts
                .next()
                .ok_or(Error::Invalid("variable blend scale"))
                .and_then(parse_blend_scale)?;
            if parts.next().is_some() {
                return Err(Error::Invalid("variable blend scale"));
            }
            (Some(start), Some(end))
        };
        let mut pending = self.variable_blend_pending.take().expect("variable blend");
        if let Some(start) = start_input {
            pending.start_scale = Some(start);
        }
        if let Some(end) = end_input {
            pending.end_scale = Some(end);
        }
        let (Some(start), Some(end)) = (pending.start_scale, pending.end_scale) else {
            self.variable_blend_pending = Some(pending);
            return Ok(());
        };
        let geometry = match surface_edit::apply_variable_blend(
            &pending.first,
            &pending.second,
            start,
            end,
            self.coordinator.limits(),
        ) {
            Ok(geometry) => geometry,
            Err(error) => {
                self.variable_blend_pending = Some(pending);
                return Err(error);
            }
        };
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_shrink_text(&mut self, text: &str) -> Result<()> {
        let trimmed = text.trim();
        let mut u0 = None;
        let mut u1 = None;
        let mut v0 = None;
        let mut v1 = None;
        if let Some(rest) = trimmed.strip_prefix("U0=") {
            u0 = Some(parse_shrink_value(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("u0=") {
            u0 = Some(parse_shrink_value(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("U1=") {
            u1 = Some(parse_shrink_value(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("u1=") {
            u1 = Some(parse_shrink_value(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("V0=") {
            v0 = Some(parse_shrink_value(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("v0=") {
            v0 = Some(parse_shrink_value(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("V1=") {
            v1 = Some(parse_shrink_value(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("v1=") {
            v1 = Some(parse_shrink_value(rest)?);
        } else {
            let values: Vec<f64> = trimmed
                .split(',')
                .map(|value| value.trim().parse::<f64>())
                .collect::<std::result::Result<_, _>>()
                .map_err(|_| Error::Invalid("shrink domain"))?;
            if values.len() != 4 || values.iter().any(|value| !value.is_finite()) {
                return Err(Error::Invalid("shrink domain"));
            }
            u0 = Some(values[0]);
            u1 = Some(values[1]);
            v0 = Some(values[2]);
            v1 = Some(values[3]);
        }
        let mut pending = self.shrink_pending.take().expect("shrink");
        if let Some(value) = u0 {
            pending.u0 = Some(value);
        }
        if let Some(value) = u1 {
            pending.u1 = Some(value);
        }
        if let Some(value) = v0 {
            pending.v0 = Some(value);
        }
        if let Some(value) = v1 {
            pending.v1 = Some(value);
        }
        let (Some(u0), Some(u1), Some(v0), Some(v1)) =
            (pending.u0, pending.u1, pending.v0, pending.v1)
        else {
            self.shrink_pending = Some(pending);
            return Ok(());
        };
        let geometry = match surface_edit::apply_shrink(
            &pending.surface,
            [u0, u1],
            [v0, v1],
            self.coordinator.limits(),
        ) {
            Ok(geometry) => geometry,
            Err(error) => {
                self.shrink_pending = Some(pending);
                return Err(error);
            }
        };
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_variable_offset_text(&mut self, text: &str) -> Result<()> {
        let trimmed = text.trim();
        let mut start_input = None;
        let mut end_input = None;
        let mut axis_input = None;
        if let Some(rest) = trimmed.strip_prefix("Start=") {
            start_input = Some(parse_offset_distance(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("start=") {
            start_input = Some(parse_offset_distance(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("End=") {
            end_input = Some(parse_offset_distance(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("end=") {
            end_input = Some(parse_offset_distance(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("Axis=") {
            axis_input = Some(surface_edit::parse_iso(rest)?);
        } else if let Some(rest) = trimmed.strip_prefix("axis=") {
            axis_input = Some(surface_edit::parse_iso(rest)?);
        } else {
            let mut parts = trimmed.split(',');
            start_input = Some(
                parts
                    .next()
                    .ok_or(Error::Invalid("variable offset start"))
                    .and_then(parse_offset_distance)?,
            );
            end_input = Some(
                parts
                    .next()
                    .ok_or(Error::Invalid("variable offset end"))
                    .and_then(parse_offset_distance)?,
            );
            axis_input = Some(
                parts
                    .next()
                    .ok_or(Error::Invalid("variable offset axis"))
                    .and_then(surface_edit::parse_iso)?,
            );
            if parts.next().is_some() {
                return Err(Error::Invalid("variable offset input"));
            }
        }
        let mut pending = self
            .variable_offset_pending
            .take()
            .expect("variable offset");
        if let Some(start) = start_input {
            pending.start = Some(start);
        }
        if let Some(end) = end_input {
            pending.end = Some(end);
        }
        if let Some(axis) = axis_input {
            pending.axis = Some(axis);
        }
        let (Some(start), Some(end), Some(axis)) = (pending.start, pending.end, pending.axis)
        else {
            self.variable_offset_pending = Some(pending);
            return Ok(());
        };
        let geometry = match surface_edit::apply_variable_offset(
            &pending.surface,
            start,
            end,
            axis,
            self.coordinator.limits(),
        ) {
            Ok(geometry) => geometry,
            Err(error) => {
                self.variable_offset_pending = Some(pending);
                return Err(error);
            }
        };
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_extrude_all_text(&mut self, text: &str) -> Result<()> {
        let height_text = text
            .strip_prefix("Height=")
            .or_else(|| text.strip_prefix("height="))
            .unwrap_or(text);
        let height: f64 = height_text
            .parse()
            .map_err(|_| Error::Invalid("extrude height"))?;
        if !height.is_finite() || height == 0.0 {
            return Err(Error::Invalid("extrude height"));
        }
        let pending = self
            .extrude_all_pending
            .as_ref()
            .expect("extrude all")
            .clone();
        let geometries =
            surface_edit::apply_extrude_all(&pending.curves, height, self.coordinator.limits())?;
        self.extrude_all_pending = None;
        let items = geometries
            .into_iter()
            .map(|geometry| (surface_edit::label(pending.feature), geometry))
            .collect();
        return self.commit_geometries(
            pending.feature,
            surface_edit::label(pending.feature),
            items,
        );
    }

    pub(super) fn submit_tangent_text(&mut self, text: &str) -> Result<()> {
        let trimmed = text.trim();
        let (edge_input, direction_input) = if let Some(rest) = trimmed.strip_prefix("Edge=") {
            (Some(surface_edit::parse_surface_edge(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("edge=") {
            (Some(surface_edit::parse_surface_edge(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("Direction=") {
            (None, Some(parse_tangent_direction(rest)?))
        } else if let Some(rest) = trimmed.strip_prefix("direction=") {
            (None, Some(parse_tangent_direction(rest)?))
        } else {
            let mut parts = trimmed.split(',');
            let edge = parts
                .next()
                .ok_or(Error::Invalid("surface tangent edge"))
                .and_then(surface_edit::parse_surface_edge)?;
            let rest: Vec<_> = parts.collect();
            let direction = parse_tangent_direction(&rest.join(","))?;
            (Some(edge), Some(direction))
        };
        let mut pending = self.tangent_pending.take().expect("tangent");
        if let Some(edge) = edge_input {
            pending.edge = Some(edge);
        }
        if let Some(direction) = direction_input {
            pending.direction = Some(direction);
        }
        let (Some(edge), Some(direction)) = (pending.edge, pending.direction) else {
            self.tangent_pending = Some(pending);
            return Ok(());
        };
        let geometry = surface_edit::apply_tangent(
            &pending.surface,
            edge,
            direction,
            self.coordinator.limits(),
        )?;
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_split_text(&mut self, text: &str) -> Result<()> {
        let trimmed = text.trim();
        let (axis_input, parameter_input) = if let Some(rest) = trimmed.strip_prefix("Axis=") {
            (Some(surface_edit::parse_iso(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("axis=") {
            (Some(surface_edit::parse_iso(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("Parameter=") {
            (None, Some(parse_split_parameter(rest)?))
        } else if let Some(rest) = trimmed.strip_prefix("parameter=") {
            (None, Some(parse_split_parameter(rest)?))
        } else {
            let mut parts = trimmed.split(',');
            let axis = parts
                .next()
                .ok_or(Error::Invalid("surface split axis"))
                .and_then(surface_edit::parse_iso)?;
            let parameter = parts
                .next()
                .ok_or(Error::Invalid("surface split parameter"))
                .and_then(parse_split_parameter)?;
            if parts.next().is_some() {
                return Err(Error::Invalid("surface split input"));
            }
            (Some(axis), Some(parameter))
        };
        let mut pending = self.split_pending.take().expect("split");
        if let Some(axis) = axis_input {
            pending.axis = Some(axis);
        }
        if let Some(parameter) = parameter_input {
            pending.parameter = Some(parameter);
        }
        let (Some(axis), Some(parameter)) = (pending.axis, pending.parameter) else {
            self.split_pending = Some(pending);
            return Ok(());
        };
        let geometries = surface_edit::apply_split(
            &pending.surface,
            axis,
            parameter,
            self.coordinator.limits(),
        )?;
        self.events.emit(
            crate::command_events::EventKind::Started,
            pending.feature,
            "surface split",
            None,
        );
        let items = geometries
            .into_iter()
            .map(|geometry| (surface_edit::label(pending.feature), geometry))
            .collect();
        return self.commit_geometries(
            pending.feature,
            surface_edit::label(pending.feature),
            items,
        );
    }

    pub(super) fn submit_extend_text(&mut self, text: &str) -> Result<()> {
        let trimmed = text.trim();
        let (edge_input, distance_input) = if let Some(rest) = trimmed.strip_prefix("Edge=") {
            (Some(surface_edit::parse_surface_edge(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("edge=") {
            (Some(surface_edit::parse_surface_edge(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("Distance=") {
            (None, Some(parse_extend_distance(rest)?))
        } else if let Some(rest) = trimmed.strip_prefix("distance=") {
            (None, Some(parse_extend_distance(rest)?))
        } else {
            let mut parts = trimmed.split(',');
            let edge = parts
                .next()
                .ok_or(Error::Invalid("surface extend edge"))
                .and_then(surface_edit::parse_surface_edge)?;
            let distance = parts
                .next()
                .ok_or(Error::Invalid("surface extend distance"))
                .and_then(parse_extend_distance)?;
            if parts.next().is_some() {
                return Err(Error::Invalid("surface extend input"));
            }
            (Some(edge), Some(distance))
        };
        let mut pending = self.extend_pending.take().expect("extend");
        if let Some(edge) = edge_input {
            pending.edge = Some(edge);
        }
        if let Some(distance) = distance_input {
            pending.distance_mm = Some(distance);
        }
        let (Some(edge), Some(distance)) = (pending.edge, pending.distance_mm) else {
            self.extend_pending = Some(pending);
            return Ok(());
        };
        let geometry = surface_edit::apply_extend(
            &pending.surface,
            edge,
            distance,
            self.coordinator.limits(),
        )?;
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_tween_text(&mut self, text: &str) -> Result<()> {
        let fraction_text = text
            .strip_prefix("Fraction=")
            .or_else(|| text.strip_prefix("fraction="))
            .unwrap_or(text);
        let fraction: f64 = fraction_text
            .parse()
            .map_err(|_| Error::Invalid("tween fraction"))?;
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            return Err(Error::Invalid("tween fraction"));
        }
        let pending = self.tween_pending.take().expect("tween");
        let geometry = surface_edit::apply_tween(
            &pending.first,
            &pending.second,
            fraction,
            self.coordinator.limits(),
        )?;
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_seam_text(&mut self, text: &str) -> Result<()> {
        let trimmed = text.trim();
        let (axis_input, tolerance_input) = if let Some(rest) = trimmed.strip_prefix("Axis=") {
            (Some(surface_edit::parse_iso(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("axis=") {
            (Some(surface_edit::parse_iso(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("Tolerance=") {
            (None, Some(parse_seam_tolerance(rest)?))
        } else if let Some(rest) = trimmed.strip_prefix("tolerance=") {
            (None, Some(parse_seam_tolerance(rest)?))
        } else {
            let mut parts = trimmed.split(',');
            let axis = parts
                .next()
                .ok_or(Error::Invalid("surface seam axis"))
                .and_then(surface_edit::parse_iso)?;
            let tolerance = parts
                .next()
                .ok_or(Error::Invalid("surface seam tolerance"))
                .and_then(parse_seam_tolerance)?;
            if parts.next().is_some() {
                return Err(Error::Invalid("surface seam input"));
            }
            (Some(axis), Some(tolerance))
        };
        let mut pending = self.seam_pending.take().expect("seam");
        if let Some(axis) = axis_input {
            pending.axis = Some(axis);
        }
        if let Some(tolerance) = tolerance_input {
            pending.tolerance_mm = Some(tolerance);
        }
        let (Some(axis), Some(tolerance)) = (pending.axis, pending.tolerance_mm) else {
            self.seam_pending = Some(pending);
            return Ok(());
        };
        let geometry =
            surface_edit::apply_seam(&pending.surface, axis, tolerance, self.coordinator.limits())?;
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_surface_pair_text(&mut self, text: &str) -> Result<()> {
        let trimmed = text.trim();
        let mut pending = self.surface_pair_pending.take().expect("surface pair");
        macro_rules! restore {
            ($error:expr) => {{
                self.surface_pair_pending = Some(pending);
                return Err($error);
            }};
        }
        if trimmed.is_empty() {
            let (Some(source), Some(target)) = (pending.source_edge, pending.target_edge) else {
                restore!(Error::Invalid("match needs a source and a target edge"));
            };
            let geometry = match surface_edit::apply_match(
                &pending.source,
                &pending.target,
                source,
                target,
                pending.continuity,
                self.coordinator.limits(),
            ) {
                Ok(geometry) => geometry,
                Err(error) => restore!(error),
            };
            return self.commit_geometry(
                surface_edit::static_feature(pending.feature),
                surface_edit::label(pending.feature),
                geometry,
            );
        }
        let continuity_input = trimmed
            .strip_prefix("Continuity=")
            .or_else(|| trimmed.strip_prefix("continuity="));
        if let Some(rest) = continuity_input {
            match surface_edit::parse_match_continuity(rest) {
                Ok(continuity) => {
                    pending.continuity = continuity;
                    self.surface_pair_pending = Some(pending);
                    return Ok(());
                }
                Err(error) => restore!(error),
            }
        }
        let (source_input, target_input) = if let Some(rest) = trimmed.strip_prefix("Source=") {
            (Some(surface_edit::parse_surface_edge(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("source=") {
            (Some(surface_edit::parse_surface_edge(rest)?), None)
        } else if let Some(rest) = trimmed.strip_prefix("Target=") {
            (None, Some(surface_edit::parse_surface_edge(rest)?))
        } else if let Some(rest) = trimmed.strip_prefix("target=") {
            (None, Some(surface_edit::parse_surface_edge(rest)?))
        } else {
            let mut parts = trimmed.split(',');
            let source = parts
                .next()
                .ok_or(Error::Invalid("surface source edge"))
                .and_then(surface_edit::parse_surface_edge)?;
            let target = parts
                .next()
                .ok_or(Error::Invalid("surface target edge"))
                .and_then(surface_edit::parse_surface_edge)?;
            if parts.next().is_some() {
                restore!(Error::Invalid("surface match edges"));
            }
            (Some(source), Some(target))
        };
        if let Some(source) = source_input {
            pending.source_edge = Some(source);
        }
        if let Some(target) = target_input {
            pending.target_edge = Some(target);
        }
        self.surface_pair_pending = Some(pending);
        return Ok(());
    }

    pub(super) fn submit_curve_pair_text(&mut self, text: &str) -> Result<()> {
        let axis_text = text
            .strip_prefix("Axis=")
            .or_else(|| text.strip_prefix("axis="))
            .unwrap_or(text);
        let values: Vec<f64> = axis_text
            .split(',')
            .map(|value| value.trim().parse::<f64>())
            .collect::<std::result::Result<_, _>>()
            .map_err(|_| Error::Invalid("rail revolve axis"))?;
        if values.len() != 6 || values.iter().any(|value| !value.is_finite()) {
            return Err(Error::Invalid("rail revolve axis"));
        }
        let pending = self.curve_pair_pending.take().expect("curve pair");
        let geometry = surface_edit::apply_rail_revolve(
            &pending.profile,
            &pending.rail,
            [values[0], values[1], values[2]],
            [values[3], values[4], values[5]],
            self.coordinator.limits(),
        )?;
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }

    pub(super) fn submit_surface_text(&mut self, text: &str) -> Result<()> {
        let distance_text = text
            .strip_prefix("Distance=")
            .or_else(|| text.strip_prefix("distance="))
            .unwrap_or(text);
        let distance: f64 = distance_text
            .parse()
            .map_err(|_| Error::Invalid("surface distance"))?;
        if !distance.is_finite() || distance == 0.0 {
            return Err(Error::Invalid("surface distance"));
        }
        let pending = self.surface_pending.take().expect("surface pending");
        let geometry = surface_edit::apply_numeric(
            pending.feature,
            &pending.surface,
            distance,
            self.coordinator.limits(),
        )?;
        return self.commit_geometry(
            surface_edit::static_feature(pending.feature),
            surface_edit::label(pending.feature),
            geometry,
        );
    }
}
