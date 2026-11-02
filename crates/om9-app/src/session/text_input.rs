// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed command-line input for pending geometry commands, including options and points.
//!
//! Extracted unchanged from `session.rs`; the pending-command state and helpers stay on
//! `Controller` in the parent module.
use super::*;

impl Controller {
    pub(super) fn submit_geometry_text(&mut self, text: &str) -> Result<()> {
        if text.len() > 4096 {
            return Err(Error::Budget("command input"));
        }
        let text = text.trim();
        if matches!(
            self.top11_pending,
            Some(
                super::top11::Top11Pending::Move { .. }
                    | super::top11::Top11Pending::Rotate { .. }
                    | super::top11::Top11Pending::SplitPoint { .. }
            )
        ) {
            let plane = self.input_plane();
            let input = om9_core::coords::parse_input(text, self.document().page_unit)?;
            let point =
                om9_core::coords::resolve_with_base(input, Some(&plane), self.relative_base())?;
            return self.accept_world_point(point.xyz());
        }
        if let Some(result) = self.submit_measure_text(text) {
            return result;
        }
        if let Some(result) = self.submit_annotation_text(text) {
            return result;
        }
        if let Some(result) = self.submit_transform_text(text) {
            return result;
        }
        if self.slash_pending.is_some() {
            let point_text = text
                .strip_prefix("First=")
                .or_else(|| text.strip_prefix("Second="))
                .or_else(|| text.strip_prefix("first="))
                .or_else(|| text.strip_prefix("second="))
                .unwrap_or(text);
            let values: Vec<f64> = point_text
                .split(',')
                .map(|value| value.trim().parse::<f64>())
                .collect::<std::result::Result<_, _>>()
                .map_err(|_| Error::Invalid("slash point"))?;
            if values.len() != 3 || values.iter().any(|value| !value.is_finite()) {
                return Err(Error::Invalid("slash point"));
            }
            return self.accept_world_point([values[0], values[1], values[2]]);
        }
        if self.history_options_pending.is_some() {
            return self.submit_history_options_text(text);
        }
        if self.fillet_pending.is_some() {
            return self.submit_fillet_text(text);
        }
        if self.variable_chamfer_pending.is_some() {
            return self.submit_variable_chamfer_text(text);
        }
        if self.chamfer_pending.is_some() {
            return self.submit_chamfer_text(text);
        }
        if self.variable_blend_pending.is_some() {
            return self.submit_variable_blend_text(text);
        }
        if self.shrink_pending.is_some() {
            return self.submit_shrink_text(text);
        }
        if self.variable_offset_pending.is_some() {
            return self.submit_variable_offset_text(text);
        }
        if self.extrude_all_pending.is_some() {
            return self.submit_extrude_all_text(text);
        }
        if self.tangent_pending.is_some() {
            return self.submit_tangent_text(text);
        }
        if self.split_pending.is_some() {
            return self.submit_split_text(text);
        }
        if self.extend_pending.is_some() {
            return self.submit_extend_text(text);
        }
        if self.tween_pending.is_some() {
            return self.submit_tween_text(text);
        }
        if self.soft_edit_pending.is_some() {
            return self.submit_soft_edit_text(text);
        }
        if self.seam_pending.is_some() {
            return self.submit_seam_text(text);
        }
        if self.surface_pair_pending.is_some() {
            return self.submit_surface_pair_text(text);
        }
        if self.curve_pair_pending.is_some() {
            return self.submit_curve_pair_text(text);
        }
        if self.surface_pending.is_some() {
            return self.submit_surface_text(text);
        }
        if let Some(command) = parse_gumball_placement(text) {
            match command {
                GumballPlacement::Origin(origin) => self.relocate_gumball(Some(origin)),
                GumballPlacement::Reset => self.relocate_gumball(None),
                GumballPlacement::World => {
                    self.set_gumball_alignment(super::gumball::GumballAlignment::World)
                }
                GumballPlacement::Object => {
                    self.set_gumball_alignment(super::gumball::GumballAlignment::Object)
                }
                GumballPlacement::CPlane => {
                    self.set_gumball_alignment(super::gumball::GumballAlignment::CPlane)
                }
            }
            self.events.emit(
                crate::command_events::EventKind::Finished,
                "Gumball",
                "gumball placement",
                None,
            );
            return Ok(());
        }
        if let Some(edit) = super::gumball_numeric::parse(text) {
            use super::gumball_numeric::NumericEdit;
            let result = match edit {
                NumericEdit::Move(delta) => self.numeric_move(delta),
                NumericEdit::Rotate(axis, degrees) => self.numeric_rotate(axis, degrees),
                NumericEdit::Scale(axis, factor) => self.numeric_scale(axis, factor),
            };
            let moved = result?;
            self.events.emit(
                crate::command_events::EventKind::Finished,
                "Gumball",
                &format!("applied to {moved} objects"),
                None,
            );
            return Ok(());
        }
        if let Some(count) = parse_mesh_stress(text) {
            let created = self.insert_stress_meshes(count)?;
            self.events.emit(
                crate::command_events::EventKind::Finished,
                "MeshStress",
                &format!("created {created} meshes"),
                None,
            );
            return Ok(());
        }
        if let Some(choice) = crate::layer_command::parse(text, self.document())? {
            return self.set_current_layer(choice);
        }
        if !self.is_running() {
            if text.is_empty() {
                let repeat = self
                    .repeatable_last()
                    .map(str::to_owned)
                    .ok_or(Error::Missing("previous repeatable command"))?;
                return self.dispatch(&repeat, InvocationOrigin::Cmd);
            }
            return match self.resolve_typed_command(text) {
                Some(LINE) => self.dispatch(LINE, InvocationOrigin::Cmd),
                Some(POINT) => self.dispatch(POINT, InvocationOrigin::Cmd),
                Some(super::circle::CIRCLE) => {
                    self.dispatch(super::circle::CIRCLE, InvocationOrigin::Cmd)
                }
                Some(super::rect::RECTANGLE) => {
                    self.dispatch(super::rect::RECTANGLE, InvocationOrigin::Cmd)
                }
                Some(super::polyline::POLYLINE) => {
                    self.dispatch(super::polyline::POLYLINE, InvocationOrigin::Cmd)
                }
                Some(super::interp::INTERP) => {
                    self.dispatch(super::interp::INTERP, InvocationOrigin::Cmd)
                }
                Some(super::ellipse::ELLIPSE) => {
                    self.dispatch(super::ellipse::ELLIPSE, InvocationOrigin::Cmd)
                }
                Some(super::arc::ARC) => self.dispatch(super::arc::ARC, InvocationOrigin::Cmd),
                Some(super::arc::ARC_START) => {
                    self.dispatch(super::arc::ARC_START, InvocationOrigin::Cmd)
                }
                Some(super::rebuild::REBUILD) => {
                    self.dispatch(super::rebuild::REBUILD, InvocationOrigin::Cmd)
                }
                Some(super::fit::FIT) => self.dispatch(super::fit::FIT, InvocationOrigin::Cmd),
                Some(super::refit_srf::REFIT) => {
                    self.dispatch(super::refit_srf::REFIT, InvocationOrigin::Cmd)
                }
                Some(surface_edit::SMASH) => {
                    self.dispatch(surface_edit::SMASH, InvocationOrigin::Cmd)
                }
                Some(super::offset::OFFSET) => {
                    self.dispatch(super::offset::OFFSET, InvocationOrigin::Cmd)
                }
                Some(super::project::PROJECT) => {
                    self.dispatch(super::project::PROJECT, InvocationOrigin::Cmd)
                }
                Some(super::blend::BLEND) => {
                    self.dispatch(super::blend::BLEND, InvocationOrigin::Cmd)
                }
                Some(super::blend::BLEND_CRV) => {
                    self.dispatch(super::blend::BLEND_CRV, InvocationOrigin::Cmd)
                }
                Some(super::blend::MATCH) => {
                    self.dispatch(super::blend::MATCH, InvocationOrigin::Cmd)
                }
                Some(super::isocurve::ISOCURVE) => {
                    self.dispatch(super::isocurve::ISOCURVE, InvocationOrigin::Cmd)
                }
                Some(super::pull::PULL) => self.dispatch(super::pull::PULL, InvocationOrigin::Cmd),
                Some(super::intersect::INTERSECT) => {
                    self.dispatch(super::intersect::INTERSECT, InvocationOrigin::Cmd)
                }
                Some(super::intersect_two_sets::INTERSECT_TWO_SETS) => self.dispatch(
                    super::intersect_two_sets::INTERSECT_TWO_SETS,
                    InvocationOrigin::Cmd,
                ),
                Some(super::crv2view::CRV2VIEW) => {
                    self.dispatch(super::crv2view::CRV2VIEW, InvocationOrigin::Cmd)
                }
                Some(super::crv2view::CRV2VIEW_HISTORY) => {
                    self.dispatch(super::crv2view::CRV2VIEW_HISTORY, InvocationOrigin::Cmd)
                }
                Some(super::through_points::THROUGH_POINTS) => {
                    self.dispatch(super::through_points::THROUGH_POINTS, InvocationOrigin::Cmd)
                }
                Some(super::polygon::POLYGON) => {
                    self.dispatch(super::polygon::POLYGON, InvocationOrigin::Cmd)
                }
                Some(super::helix::HELIX) => {
                    self.dispatch(super::helix::HELIX, InvocationOrigin::Cmd)
                }
                Some(super::spiral::SPIRAL) => {
                    self.dispatch(super::spiral::SPIRAL, InvocationOrigin::Cmd)
                }
                Some(super::divide::DIVIDE) => {
                    self.dispatch(super::divide::DIVIDE, InvocationOrigin::Cmd)
                }
                Some(super::to_lines::TO_LINES) => {
                    self.dispatch(super::to_lines::TO_LINES, InvocationOrigin::Cmd)
                }
                Some(super::extend_curve::EXTEND) => {
                    self.dispatch(super::extend_curve::EXTEND, InvocationOrigin::Cmd)
                }
                Some(super::tween_curves::TWEEN_CURVES) => {
                    self.dispatch(super::tween_curves::TWEEN_CURVES, InvocationOrigin::Cmd)
                }
                Some(super::continue_interp::CONTINUE_INTERP) => self.dispatch(
                    super::continue_interp::CONTINUE_INTERP,
                    InvocationOrigin::Cmd,
                ),
                Some(super::arc_blend::ARC_BLEND) => {
                    self.dispatch(super::arc_blend::ARC_BLEND, InvocationOrigin::Cmd)
                }
                Some(super::section::SECTION) => {
                    self.dispatch(super::section::SECTION, InvocationOrigin::Cmd)
                }
                Some(super::revolve::REVOLVE) => {
                    self.dispatch(super::revolve::REVOLVE, InvocationOrigin::Cmd)
                }
                Some(super::curve_boolean::CURVE_BOOLEAN) => {
                    self.dispatch(super::curve_boolean::CURVE_BOOLEAN, InvocationOrigin::Cmd)
                }
                Some(super::heightfield::HEIGHTFIELD) => {
                    self.dispatch(super::heightfield::HEIGHTFIELD, InvocationOrigin::Cmd)
                }
                Some(super::extrude_surface::EXTRUDE) => {
                    self.dispatch(super::extrude_surface::EXTRUDE, InvocationOrigin::Cmd)
                }
                Some(super::angle_isocurve::ANGLE_ISOCURVE) => {
                    self.dispatch(super::angle_isocurve::ANGLE_ISOCURVE, InvocationOrigin::Cmd)
                }
                Some(super::curve_soft_edit::CURVE_SOFT_EDIT) => self.dispatch(
                    super::curve_soft_edit::CURVE_SOFT_EDIT,
                    InvocationOrigin::Cmd,
                ),
                Some(super::cross_section::CROSS_SECTION) => {
                    self.dispatch(super::cross_section::CROSS_SECTION, InvocationOrigin::Cmd)
                }
                Some(super::fillet_corners::FILLET_CORNERS) => {
                    self.dispatch(super::fillet_corners::FILLET_CORNERS, InvocationOrigin::Cmd)
                }
                Some(super::silhouette::SILHOUETTE) => {
                    self.dispatch(super::silhouette::SILHOUETTE, InvocationOrigin::Cmd)
                }
                Some(super::mesh_curve::MESH_CURVE) => {
                    self.dispatch(super::mesh_curve::MESH_CURVE, InvocationOrigin::Cmd)
                }
                Some(super::uv_curves::CREATE_UV) => {
                    self.dispatch(super::uv_curves::CREATE_UV, InvocationOrigin::Cmd)
                }
                Some(super::apply_crv::APPLY_CRV) => {
                    self.dispatch(super::apply_crv::APPLY_CRV, InvocationOrigin::Cmd)
                }
                Some(super::offset_normal::OFFSET_NORMAL) => {
                    self.dispatch(super::offset_normal::OFFSET_NORMAL, InvocationOrigin::Cmd)
                }
                Some(super::corner_ops::FILLET) => {
                    self.dispatch(super::corner_ops::FILLET, InvocationOrigin::Cmd)
                }
                Some(super::corner_ops::CHAMFER) => {
                    self.dispatch(super::corner_ops::CHAMFER, InvocationOrigin::Cmd)
                }
                Some(super::offset_on_surface::OFFSET_ON_SURFACE) => self.dispatch(
                    super::offset_on_surface::OFFSET_ON_SURFACE,
                    InvocationOrigin::Cmd,
                ),
                Some(super::crv_seam::CRV_SEAM) => {
                    self.dispatch(super::crv_seam::CRV_SEAM, InvocationOrigin::Cmd)
                }
                Some(super::control_curve::CONTROL_CURVE) => {
                    self.dispatch(super::control_curve::CONTROL_CURVE, InvocationOrigin::Cmd)
                }
                Some(super::continue_curve::CONTINUE_CURVE) => {
                    self.dispatch(super::continue_curve::CONTINUE_CURVE, InvocationOrigin::Cmd)
                }
                Some(super::sketch::SKETCH) => {
                    self.dispatch(super::sketch::SKETCH, InvocationOrigin::Cmd)
                }
                Some(super::sketch_on_surface::SKETCH_ON_SURFACE) => self.dispatch(
                    super::sketch_on_surface::SKETCH_ON_SURFACE,
                    InvocationOrigin::Cmd,
                ),
                Some(super::polyline_on_surface::POLYLINE_ON_SURFACE) => self.dispatch(
                    super::polyline_on_surface::POLYLINE_ON_SURFACE,
                    InvocationOrigin::Cmd,
                ),
                Some(super::interp_on_surface::INTERP_ON_SURFACE) => self.dispatch(
                    super::interp_on_surface::INTERP_ON_SURFACE,
                    InvocationOrigin::Cmd,
                ),
                Some(super::wireframe::DUP_EDGE) => {
                    self.dispatch(super::wireframe::DUP_EDGE, InvocationOrigin::Cmd)
                }
                Some(super::wireframe::EXTRACT_WIRE) => {
                    self.dispatch(super::wireframe::EXTRACT_WIRE, InvocationOrigin::Cmd)
                }
                Some(super::crv_ends::CRV_START) => {
                    self.dispatch(super::crv_ends::CRV_START, InvocationOrigin::Cmd)
                }
                Some(super::crv_ends::CRV_END) => {
                    self.dispatch(super::crv_ends::CRV_END, InvocationOrigin::Cmd)
                }
                Some(super::dup_border::DUP_BORDER) => {
                    self.dispatch(super::dup_border::DUP_BORDER, InvocationOrigin::Cmd)
                }
                Some(command)
                    if matches!(
                        command,
                        ZOOM_EXTENTS | ZOOM_SELECTED | VIEW_HOME | VIEW_CENTER
                    ) =>
                {
                    self.dispatch(command, InvocationOrigin::Cmd)
                }
                Some(feature) if surface_edit::handles_command(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if super::measure::handles(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if super::annotation::handles(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some(feature) if super::transform::handles(feature) => {
                    self.dispatch(feature, InvocationOrigin::Cmd)
                }
                Some("undo") => self.undo(),
                Some("redo") => self.redo(),
                _ => Err(Error::Unsupported("unknown/unavailable command")),
            };
        }
        let s = self.validate_session()?.clone();
        if text.is_empty() {
            return Err(Error::Invalid("point required"));
        }
        // Rhino p22: an option can be activated by typing its underlined letter. BothSides is the only flag
        // the implemented commands declare, so `B` toggles it through the same assignment path.
        if s.feature == LINE && text.chars().count() == 1 && text.eq_ignore_ascii_case("b") {
            self.set_both_sides(!s.both_sides)?;
            return Ok(());
        }
        if let Some(parsed) = crate::option_parser::parse_option(
            self.document().page_unit,
            &Self::line_descriptor(),
            text,
        )? {
            if s.feature != LINE {
                // A command that declares no options refuses every assignment, by
                // name: POINT has none.
                return Err(Error::Unsupported("command option"));
            }
            match parsed.value {
                crate::option_parser::OptionValue::Flag(both_sides) => {
                    self.session.as_mut().expect("session").both_sides = both_sides;
                }
                crate::option_parser::OptionValue::Number { value_mm, .. } => {
                    // `Length` fixes the end point's distance from the start; the
                    // click then chooses the direction (OM9-L3-01-02-005).
                    self.session.as_mut().expect("session").length_mm = Some(value_mm);
                }
            }
            return Ok(());
        }
        // Typed world/local/relative input: the grammar, prompts and error labels are
        // unchanged, but a local point only becomes world through the captured frame.
        let input = om9_core::coords::parse_input(text, s.unit)?;
        let world =
            om9_core::coords::resolve_with_base(input, Some(&s.plane), self.relative_base())?;
        self.accept_world_point(world.xyz())
    }
}
