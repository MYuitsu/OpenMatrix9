// SPDX-License-Identifier: LGPL-2.1-or-later
//! Feature-id dispatch for geometry commands (menu, CMD, F6 and builder starts).
//!
//! Extracted unchanged from `session.rs`; the pending-command state and helpers stay on`n//! `Controller` in the parent module.
use super::*;

impl Controller {
    pub(super) fn dispatch_geometry(
        &mut self,
        feature: &str,
        _origin: InvocationOrigin,
    ) -> Result<()> {
        Self::availability(feature)?;
        if self.is_running() {
            return Err(Error::Busy);
        }
        if super::metal::handles_command(feature) {
            return self.dispatch_metal_tool(feature);
        }
        if super::head_commands::handles_command(feature) {
            return self.dispatch_head_tool(feature);
        }
        if super::settings_commands::handles_command(feature) {
            return self.dispatch_settings_tool(feature);
        }
        if super::gem::handles(feature) {
            return self.dispatch_gem(feature);
        }
        if super::gem_placement::handles(feature) {
            return self.dispatch_gem_placement(feature);
        }
        if super::gem_edits::handles(feature) || super::gem_edits::command_name(feature).is_some() {
            return self.dispatch_gem_edits(feature);
        }
        if super::gem_cluster::handles(feature) {
            return self.dispatch_gem_cluster(feature);
        }
        if super::gem_pave::PaveTool::handles(feature) {
            return self.dispatch_gem_pave(feature);
        }
        if super::gem_cutters::CutterTool::handles(feature) {
            return self.dispatch_gem_cutter(feature);
        }
        if super::channel_cutters::RowTool::handles(feature) {
            return self.dispatch_row_cutter(feature);
        }
        if super::ring_tools::handles_command(feature) {
            return self.dispatch_ring_tool(feature);
        }
        let feature = match feature {
            _ if super::annotation::handles(feature) => {
                self.begin_annotation(feature)?;
                return Ok(());
            }
            _ if super::transform::handles(feature) => {
                self.begin_transform(feature)?;
                return Ok(());
            }
            _ if super::measure::handles(feature) => {
                self.begin_measure(feature)?;
                return Ok(());
            }
            ZOOM_EXTENTS => {
                self.zoom_extents(1.6)?;
                return Ok(());
            }
            ZOOM_SELECTED => {
                self.zoom_selected(1.6)?;
                return Ok(());
            }
            VIEW_HOME => {
                self.view_home()?;
                return Ok(());
            }
            VIEW_CENTER => {
                self.view_center()?;
                return Ok(());
            }
            LINE => LINE,
            POINT => POINT,
            "Circle" | super::circle::CIRCLE => {
                return self.dispatch_circle();
            }
            "Rectangle" | super::rect::RECTANGLE => {
                return self.dispatch_rect();
            }
            "Polyline" | super::polyline::POLYLINE => {
                return self.dispatch_polyline();
            }
            "InterpCrv" | super::interp::INTERP => {
                return self.dispatch_interp();
            }
            "Ellipse" | super::ellipse::ELLIPSE => {
                return self.dispatch_ellipse();
            }
            "Arc" | super::arc::ARC => {
                return self.dispatch_arc();
            }
            "Arc_StartPoint" | super::arc::ARC_START => {
                return self.dispatch_arc_start();
            }
            "Rebuild" | super::rebuild::REBUILD => {
                return self.dispatch_rebuild();
            }
            "FitCrv" | super::fit::FIT => {
                return self.dispatch_fit();
            }
            "RefitSrf" | super::refit_srf::REFIT => {
                return self.dispatch_refit_srf();
            }
            "Offset" | super::offset::OFFSET => {
                return self.dispatch_offset_curve();
            }
            "Project" | super::project::PROJECT => {
                return self.dispatch_project();
            }
            "Blend" | super::blend::BLEND => {
                return self.dispatch_blend();
            }
            "BlendCrv" | super::blend::BLEND_CRV => {
                return self.dispatch_blend();
            }
            "Match" | super::blend::MATCH => {
                return self.dispatch_match();
            }
            "ExtractIsocurve" | super::isocurve::ISOCURVE => {
                return self.dispatch_isocurve();
            }
            "Pull" | super::pull::PULL => {
                return self.dispatch_pull();
            }
            "Intersect" | super::intersect::INTERSECT => {
                return self.dispatch_intersect();
            }
            "IntersectTwoSets" | super::intersect_two_sets::INTERSECT_TWO_SETS => {
                return self.dispatch_intersect_two_sets();
            }
            "Crv2View" | super::crv2view::CRV2VIEW => {
                return self.dispatch_crv2view(false);
            }
            "Crv2ViewHistory" | super::crv2view::CRV2VIEW_HISTORY => {
                return self.dispatch_crv2view(true);
            }
            "CurveThroughPt" | super::through_points::THROUGH_POINTS => {
                return self.dispatch_through_points();
            }
            "Polygon" | super::polygon::POLYGON => {
                return self.dispatch_polygon();
            }
            "Helix" | super::helix::HELIX => {
                return self.dispatch_helix();
            }
            "Spiral" | super::spiral::SPIRAL => {
                return self.dispatch_spiral();
            }
            "Divide" | super::divide::DIVIDE => {
                return self.dispatch_divide();
            }
            "CrvStart" | super::crv_ends::CRV_START => {
                return self.dispatch_crv_mark(false);
            }
            "CrvEnd" | super::crv_ends::CRV_END => {
                return self.dispatch_crv_mark(true);
            }
            "DupBorder" | super::dup_border::DUP_BORDER => {
                return self.dispatch_dup_border();
            }
            "Convert" | super::to_lines::TO_LINES => {
                return self.dispatch_to_lines();
            }
            "Extend" | super::extend_curve::EXTEND => {
                return self.dispatch_extend_curve();
            }
            "TweenCurves" | super::tween_curves::TWEEN_CURVES => {
                return self.dispatch_tween_curves();
            }
            "ContinueinterpCrv" | super::continue_interp::CONTINUE_INTERP => {
                return self.dispatch_continue_interp();
            }
            "ArcBlend" | super::arc_blend::ARC_BLEND => {
                return self.dispatch_arc_blend();
            }
            "InterpCrvOnSrf" | super::interp_on_surface::INTERP_ON_SURFACE => {
                return self.dispatch_interp_on_surface();
            }
            "Section" | super::section::SECTION => {
                return self.dispatch_section();
            }
            "Revolve" | super::revolve::REVOLVE => {
                return self.dispatch_revolve();
            }
            "CurveBoolean" | super::curve_boolean::CURVE_BOOLEAN => {
                return self.dispatch_curve_boolean();
            }
            "Heightfield" | super::heightfield::HEIGHTFIELD => {
                return self.dispatch_heightfield();
            }
            "ExtrudeSurface" | super::extrude_surface::EXTRUDE => {
                return self.dispatch_extrude_surface();
            }
            "AngleIsocurve" | super::angle_isocurve::ANGLE_ISOCURVE => {
                return self.dispatch_angle_isocurve();
            }
            "CurveSoftEdit" | super::curve_soft_edit::CURVE_SOFT_EDIT => {
                return self.dispatch_curve_soft_edit();
            }
            "CrossSectionProfiles" | super::cross_section::CROSS_SECTION => {
                return self.dispatch_cross_section();
            }
            "FilletCorners" | super::fillet_corners::FILLET_CORNERS => {
                return self.dispatch_fillet_corners();
            }
            "Silhouette" | super::silhouette::SILHOUETTE => {
                return self.dispatch_silhouette();
            }
            "Mesh" | super::mesh_curve::MESH_CURVE => {
                return self.dispatch_mesh_curve();
            }
            "CreateUVCrv" | super::uv_curves::CREATE_UV => {
                return self.dispatch_create_uv();
            }
            "ApplyCrv" | super::apply_crv::APPLY_CRV => {
                return self.dispatch_apply_crv();
            }
            "OffsetCrvOnSrf" | super::offset_on_surface::OFFSET_ON_SURFACE => {
                return self.dispatch_offset_on_surface();
            }
            "Offset Normal" | super::offset_normal::OFFSET_NORMAL => {
                return self.dispatch_offset_normal();
            }
            "CurveFillet" | super::corner_ops::FILLET => {
                return self.dispatch_corner(super::corner_ops::FILLET);
            }
            "CurveChamfer" | super::corner_ops::CHAMFER => {
                return self.dispatch_corner(super::corner_ops::CHAMFER);
            }
            "CrvSeam" | super::crv_seam::CRV_SEAM => {
                return self.dispatch_crv_seam();
            }
            "Curve" | super::control_curve::CONTROL_CURVE => {
                return self.dispatch_control_curve();
            }
            "ContinueCurve" | super::continue_curve::CONTINUE_CURVE => {
                return self.dispatch_continue_curve();
            }
            "PolylineOnSrf" | super::polyline_on_surface::POLYLINE_ON_SURFACE => {
                return self.dispatch_polyline_on_surface();
            }
            "Sketch" | super::sketch::SKETCH => {
                return self.dispatch_sketch();
            }
            "SketchOnSrf" | super::sketch_on_surface::SKETCH_ON_SURFACE => {
                return self.dispatch_sketch_on_surface();
            }
            "DupEdge" | super::wireframe::DUP_EDGE => {
                return self.dispatch_wireframe(super::wireframe::DUP_EDGE);
            }
            "ExtractWire" | super::wireframe::EXTRACT_WIRE => {
                return self.dispatch_wireframe(super::wireframe::EXTRACT_WIRE);
            }
            "MeshStress" => {
                // Menu or search invocation: the same command as typing `MeshStress` in the line.
                self.insert_stress_meshes(super::stress::DEFAULT_STRESS_MESHES)?;
                return Ok(());
            }
            _ if solid::handles(feature) => {
                self.dispatch_solid(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_smash(feature) => {
                self.dispatch_smash()?;
                return Ok(());
            }
            _ if surface_edit::handles(feature) => {
                let geometry = surface_edit::apply(
                    feature,
                    self.document(),
                    &self.selection,
                    self.coordinator.limits(),
                )?;
                self.events.emit(
                    crate::command_events::EventKind::Started,
                    surface_edit::static_feature(feature),
                    "selected surface",
                    None,
                );
                self.commit_geometry(
                    surface_edit::static_feature(feature),
                    surface_edit::label(feature),
                    geometry,
                )?;
                return Ok(());
            }
            _ if surface_edit::handles_numeric(feature) => {
                self.dispatch_surface_distance(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_drape(feature) => {
                self.dispatch_drape(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_fillet(feature) => {
                self.dispatch_fillet(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_variable_chamfer(feature) => {
                self.dispatch_variable_chamfer(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_chamfer(feature) => {
                self.dispatch_chamfer(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_untrim(feature) => {
                self.dispatch_untrim(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_variable_blend(feature) => {
                self.dispatch_variable_blend(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_shrink(feature) => {
                self.dispatch_shrink(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_variable_offset(feature) => {
                self.dispatch_variable_offset(feature)?;
                return Ok(());
            }
            "EdgeSrf" => {
                let geometry = surface_edit::apply_curves(
                    super::surface_edit::EDGE_CURVES,
                    self.document(),
                    self.selected_ordered().as_slice(),
                    self.coordinator.limits(),
                )?;
                self.commit_geometry(super::surface_edit::EDGE_CURVES, "EdgeSrf", geometry)?;
                return Ok(());
            }
            "PlanarSrf" => {
                self.dispatch_planar_curves(super::surface_edit::PLANAR_CURVES)?;
                return Ok(());
            }
            _ if surface_edit::handles_planar(feature) => {
                self.dispatch_planar_curves(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_extrude_all(feature) => {
                self.dispatch_extrude_all(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_tangent(feature) => {
                self.dispatch_tangent(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_split(feature) => {
                self.dispatch_split(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_extend(feature) => {
                self.dispatch_extend(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_tween(feature) => {
                self.dispatch_tween(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_soft_edit(feature) => {
                self.dispatch_soft_edit(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_seam(feature) => {
                self.dispatch_seam(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_surface_pair(feature) => {
                self.dispatch_surface_pair(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_merge(feature) => {
                self.dispatch_merge(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_sweep_multi(feature) => {
                self.dispatch_sweep_multi(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_curve_pair(feature) => {
                self.dispatch_curve_pair(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_four_rail_profile(feature) => {
                self.dispatch_four_rail_profile(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_sweep4(feature) => {
                self.dispatch_sweep4(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_history(feature) => {
                self.dispatch_sweep_history(feature)?;
                return Ok(());
            }
            _ if surface_edit::handles_curves(feature) => {
                let geometry = surface_edit::apply_curves(
                    feature,
                    self.document(),
                    &self.selected_ordered(),
                    self.coordinator.limits(),
                )?;
                self.events.emit(
                    crate::command_events::EventKind::Started,
                    surface_edit::static_feature(feature),
                    "selected curves",
                    None,
                );
                self.commit_geometry(
                    surface_edit::static_feature(feature),
                    surface_edit::label(feature),
                    geometry,
                )?;
                return Ok(());
            }
            surface_edit::PLANE_CORNERS | surface_edit::CUTTING_PLANE => {
                surface_edit::static_feature(feature)
            }
            _ => unreachable!(),
        };
        let view = &self.workspace.views[&self.workspace.active];
        let context = CommandContext {
            document: self.document,
            view: self.workspace.active,
            selection: self.selection.iter().copied().collect(),
            revision: self.document().context_revision,
            epoch: self.coordinator.epoch(),
        };
        self.session = Some(CommandSession {
            feature,
            context,
            plane: view.cplane,
            unit: self.document().page_unit,
            start: None,
            both_sides: false,
            length_mm: None,
            points: Vec::new(),
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            feature,
            "started",
            None,
        );
        Ok(())
    }
}
