// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed command-name and alias registry.
//!
//! Extracted unchanged from `session.rs`: the string tables that map what a user types
//! (`Ln`, `Circle`, `ViewCenter`, a feature id) to one stable command. Feature modules own their
//! constants; this file owns only the lookup order.
use super::*;

impl Controller {
    /// Resolves a typed command name through the declared aliases of the implemented commands.
    /// Registry aliases such as `Ln` are part of the record, so typing them must reach the same command.
    pub fn resolve_alias(text: &str) -> Option<&'static str> {
        let text = text.trim();
        if text.eq_ignore_ascii_case("undo") {
            return Some("undo");
        }
        if text.eq_ignore_ascii_case("redo") {
            return Some("redo");
        }
        if let Some(command) = Self::view_command(text) {
            return Some(command);
        }
        if let Some(command) = super::measure::from_alias(text) {
            return Some(command);
        }
        if let Some(command) = super::annotation::from_alias(text) {
            return Some(command);
        }
        if let Some(command) = super::transform::from_alias(text) {
            return Some(command);
        }
        if let Some(command) = super::head_commands::from_alias(text) {
            return Some(command);
        }
        if let Some(command) = super::settings_commands::from_alias(text) {
            return Some(command);
        }
        if let Some(command) = super::gem_cutters::CutterTool::command_name(text) {
            return Some(command);
        }
        if let Some(command) = super::channel_cutters::RowTool::command_name(text) {
            return Some(command);
        }
        if text.eq_ignore_ascii_case("circle") || text.eq_ignore_ascii_case(super::circle::CIRCLE) {
            return Some(super::circle::CIRCLE);
        }
        if text.eq_ignore_ascii_case("rectangle")
            || text.eq_ignore_ascii_case(super::rect::RECTANGLE)
        {
            return Some(super::rect::RECTANGLE);
        }
        if text.eq_ignore_ascii_case("polyline")
            || text.eq_ignore_ascii_case(super::polyline::POLYLINE)
        {
            return Some(super::polyline::POLYLINE);
        }
        if text.eq_ignore_ascii_case("interpcrv")
            || text.eq_ignore_ascii_case(super::interp::INTERP)
        {
            return Some(super::interp::INTERP);
        }
        if text.eq_ignore_ascii_case("ellipse")
            || text.eq_ignore_ascii_case(super::ellipse::ELLIPSE)
        {
            return Some(super::ellipse::ELLIPSE);
        }
        if text.eq_ignore_ascii_case("arc") || text.eq_ignore_ascii_case(super::arc::ARC) {
            return Some(super::arc::ARC);
        }
        if text.eq_ignore_ascii_case("arc_startpoint")
            || text.eq_ignore_ascii_case("arc startpoint")
            || text.eq_ignore_ascii_case("arcdirection")
            || text.eq_ignore_ascii_case(super::arc::ARC_START)
        {
            return Some(super::arc::ARC_START);
        }
        if text.eq_ignore_ascii_case("rebuild")
            || text.eq_ignore_ascii_case(super::rebuild::REBUILD)
        {
            return Some(super::rebuild::REBUILD);
        }
        if text.eq_ignore_ascii_case("fitcrv") || text.eq_ignore_ascii_case(super::fit::FIT) {
            return Some(super::fit::FIT);
        }
        if text.eq_ignore_ascii_case("offset") || text.eq_ignore_ascii_case(super::offset::OFFSET) {
            return Some(super::offset::OFFSET);
        }
        if text.eq_ignore_ascii_case("project")
            || text.eq_ignore_ascii_case(super::project::PROJECT)
        {
            return Some(super::project::PROJECT);
        }
        if text.eq_ignore_ascii_case("blend") || text.eq_ignore_ascii_case(super::blend::BLEND) {
            return Some(super::blend::BLEND);
        }
        if text.eq_ignore_ascii_case("blendcrv")
            || text.eq_ignore_ascii_case(super::blend::BLEND_CRV)
        {
            return Some(super::blend::BLEND_CRV);
        }
        if text.eq_ignore_ascii_case("match") || text.eq_ignore_ascii_case(super::blend::MATCH) {
            return Some(super::blend::MATCH);
        }
        if text.eq_ignore_ascii_case("extractisocurve")
            || text.eq_ignore_ascii_case(super::isocurve::ISOCURVE)
        {
            return Some(super::isocurve::ISOCURVE);
        }
        if text.eq_ignore_ascii_case("pull") || text.eq_ignore_ascii_case(super::pull::PULL) {
            return Some(super::pull::PULL);
        }
        if text.eq_ignore_ascii_case("intersect")
            || text.eq_ignore_ascii_case(super::intersect::INTERSECT)
        {
            return Some(super::intersect::INTERSECT);
        }
        if text.eq_ignore_ascii_case("intersecttwosets")
            || text.eq_ignore_ascii_case(super::intersect_two_sets::INTERSECT_TWO_SETS)
        {
            return Some(super::intersect_two_sets::INTERSECT_TWO_SETS);
        }
        if text.eq_ignore_ascii_case("crv2view")
            || text.eq_ignore_ascii_case(super::crv2view::CRV2VIEW)
        {
            return Some(super::crv2view::CRV2VIEW);
        }
        if text.eq_ignore_ascii_case("crv2viewhistory")
            || text.eq_ignore_ascii_case(super::crv2view::CRV2VIEW_HISTORY)
        {
            return Some(super::crv2view::CRV2VIEW_HISTORY);
        }
        if text.eq_ignore_ascii_case("curvethroughpt")
            || text.eq_ignore_ascii_case(super::through_points::THROUGH_POINTS)
        {
            return Some(super::through_points::THROUGH_POINTS);
        }
        if text.eq_ignore_ascii_case("polygon")
            || text.eq_ignore_ascii_case(super::polygon::POLYGON)
        {
            return Some(super::polygon::POLYGON);
        }
        if text.eq_ignore_ascii_case("helix") || text.eq_ignore_ascii_case(super::helix::HELIX) {
            return Some(super::helix::HELIX);
        }
        if text.eq_ignore_ascii_case("spiral") || text.eq_ignore_ascii_case(super::spiral::SPIRAL) {
            return Some(super::spiral::SPIRAL);
        }
        if text.eq_ignore_ascii_case("divide") || text.eq_ignore_ascii_case(super::divide::DIVIDE) {
            return Some(super::divide::DIVIDE);
        }
        if text.eq_ignore_ascii_case("convert")
            || text.eq_ignore_ascii_case(super::to_lines::TO_LINES)
        {
            return Some(super::to_lines::TO_LINES);
        }
        if text.eq_ignore_ascii_case("extend")
            || text.eq_ignore_ascii_case(super::extend_curve::EXTEND)
        {
            return Some(super::extend_curve::EXTEND);
        }
        if text.eq_ignore_ascii_case("tweencurves")
            || text.eq_ignore_ascii_case(super::tween_curves::TWEEN_CURVES)
        {
            return Some(super::tween_curves::TWEEN_CURVES);
        }
        if text.eq_ignore_ascii_case("continueinterpcrv")
            || text.eq_ignore_ascii_case(super::continue_interp::CONTINUE_INTERP)
        {
            return Some(super::continue_interp::CONTINUE_INTERP);
        }
        if text.eq_ignore_ascii_case("arcblend")
            || text.eq_ignore_ascii_case(super::arc_blend::ARC_BLEND)
        {
            return Some(super::arc_blend::ARC_BLEND);
        }
        if text.eq_ignore_ascii_case("dupedge")
            || text.eq_ignore_ascii_case(super::wireframe::DUP_EDGE)
        {
            return Some(super::wireframe::DUP_EDGE);
        }
        if text.eq_ignore_ascii_case("extractwire")
            || text.eq_ignore_ascii_case("extractwireframe")
            || text.eq_ignore_ascii_case(super::wireframe::EXTRACT_WIRE)
        {
            return Some(super::wireframe::EXTRACT_WIRE);
        }
        if text.eq_ignore_ascii_case("crvstart")
            || text.eq_ignore_ascii_case(super::crv_ends::CRV_START)
        {
            return Some(super::crv_ends::CRV_START);
        }
        if text.eq_ignore_ascii_case("crvend")
            || text.eq_ignore_ascii_case(super::crv_ends::CRV_END)
        {
            return Some(super::crv_ends::CRV_END);
        }
        if text.eq_ignore_ascii_case("interpcrvonsrf")
            || text.eq_ignore_ascii_case(super::interp_on_surface::INTERP_ON_SURFACE)
        {
            return Some(super::interp_on_surface::INTERP_ON_SURFACE);
        }
        if text.eq_ignore_ascii_case("section")
            || text.eq_ignore_ascii_case(super::section::SECTION)
        {
            return Some(super::section::SECTION);
        }
        if text.eq_ignore_ascii_case("revolve")
            || text.eq_ignore_ascii_case(super::revolve::REVOLVE)
        {
            return Some(super::revolve::REVOLVE);
        }
        if text.eq_ignore_ascii_case("curveboolean")
            || text.eq_ignore_ascii_case(super::curve_boolean::CURVE_BOOLEAN)
        {
            return Some(super::curve_boolean::CURVE_BOOLEAN);
        }
        if text.eq_ignore_ascii_case("loft") {
            return Some(super::surface_edit::LOFT);
        }
        if text.eq_ignore_ascii_case("sweep1") {
            return Some(super::surface_edit::SWEEP1);
        }
        if text.eq_ignore_ascii_case("sweep2") {
            return Some(super::surface_edit::SWEEP2);
        }
        if text.eq_ignore_ascii_case("networksrf") {
            return Some(super::surface_edit::NETWORK);
        }
        if text.eq_ignore_ascii_case("patch") {
            return Some(super::surface_edit::PATCH);
        }
        if text.eq_ignore_ascii_case("blendsrf") {
            return Some(super::surface_edit::BLEND);
        }
        if text.eq_ignore_ascii_case("seamsrf") || text.eq_ignore_ascii_case("seam") {
            return Some(super::surface_edit::SEAM);
        }
        if text.eq_ignore_ascii_case("mergesrf") {
            return Some(super::surface_edit::MERGE);
        }
        if text.eq_ignore_ascii_case("matchsrf") {
            return Some(super::surface_edit::MATCH);
        }
        if text.eq_ignore_ascii_case("tweensurfaces") || text.eq_ignore_ascii_case("tween") {
            return Some(super::surface_edit::TWEEN);
        }
        if text.eq_ignore_ascii_case("railrevolve") {
            return Some(super::surface_edit::RAIL_REVOLVE);
        }
        if text.eq_ignore_ascii_case("extendsrf") {
            return Some(super::surface_edit::EXTEND);
        }
        if text.eq_ignore_ascii_case("shrinksrf") {
            return Some(super::surface_edit::SHRINK);
        }
        if text.eq_ignore_ascii_case("unrollsrf") {
            return Some(super::surface_edit::UNROLL);
        }
        if text.eq_ignore_ascii_case("smash")
            || text.eq_ignore_ascii_case(super::surface_edit::SMASH)
        {
            return Some(super::surface_edit::SMASH);
        }
        if text.eq_ignore_ascii_case("refitsrf")
            || text.eq_ignore_ascii_case(super::refit_srf::REFIT)
        {
            return Some(super::refit_srf::REFIT);
        }
        if text.eq_ignore_ascii_case("offsetsrf") {
            return Some(super::surface_edit::OFFSET);
        }
        if text.eq_ignore_ascii_case("planarsrf") {
            return Some(super::surface_edit::PLANAR_CURVES);
        }
        if text.eq_ignore_ascii_case("edgesrf") {
            return Some(super::surface_edit::EDGE_CURVES);
        }
        if text.eq_ignore_ascii_case("heightfield")
            || text.eq_ignore_ascii_case(super::heightfield::HEIGHTFIELD)
        {
            return Some(super::heightfield::HEIGHTFIELD);
        }
        if text.eq_ignore_ascii_case("extrudesurface")
            || text.eq_ignore_ascii_case(super::extrude_surface::EXTRUDE)
        {
            return Some(super::extrude_surface::EXTRUDE);
        }
        if text.eq_ignore_ascii_case("angleisocurve")
            || text.eq_ignore_ascii_case("gvextractisocurve")
            || text.eq_ignore_ascii_case(super::angle_isocurve::ANGLE_ISOCURVE)
        {
            return Some(super::angle_isocurve::ANGLE_ISOCURVE);
        }
        if text.eq_ignore_ascii_case("curvesoftedit")
            || text.eq_ignore_ascii_case("softeditcrv")
            || text.eq_ignore_ascii_case(super::curve_soft_edit::CURVE_SOFT_EDIT)
        {
            return Some(super::curve_soft_edit::CURVE_SOFT_EDIT);
        }
        if text.eq_ignore_ascii_case("crosssectionprofiles")
            || text.eq_ignore_ascii_case("csec")
            || text.eq_ignore_ascii_case(super::cross_section::CROSS_SECTION)
        {
            return Some(super::cross_section::CROSS_SECTION);
        }
        if text.eq_ignore_ascii_case("filletcorners")
            || text.eq_ignore_ascii_case(super::fillet_corners::FILLET_CORNERS)
        {
            return Some(super::fillet_corners::FILLET_CORNERS);
        }
        if text.eq_ignore_ascii_case("silhouette")
            || text.eq_ignore_ascii_case(super::silhouette::SILHOUETTE)
        {
            return Some(super::silhouette::SILHOUETTE);
        }
        if text.eq_ignore_ascii_case("mesh")
            || text.eq_ignore_ascii_case("sketchonmesh")
            || text.eq_ignore_ascii_case("sketch on mesh")
            || text.eq_ignore_ascii_case(super::mesh_curve::MESH_CURVE)
        {
            return Some(super::mesh_curve::MESH_CURVE);
        }
        if text.eq_ignore_ascii_case("createuvcrv")
            || text.eq_ignore_ascii_case(super::uv_curves::CREATE_UV)
        {
            return Some(super::uv_curves::CREATE_UV);
        }
        if text.eq_ignore_ascii_case("applycrv")
            || text.eq_ignore_ascii_case(super::apply_crv::APPLY_CRV)
        {
            return Some(super::apply_crv::APPLY_CRV);
        }
        if text.eq_ignore_ascii_case("offsetcrvonsrf")
            || text.eq_ignore_ascii_case(super::offset_on_surface::OFFSET_ON_SURFACE)
        {
            return Some(super::offset_on_surface::OFFSET_ON_SURFACE);
        }
        if text.eq_ignore_ascii_case("offset normal")
            || text.eq_ignore_ascii_case("offsetnormal")
            || text.eq_ignore_ascii_case(super::offset_normal::OFFSET_NORMAL)
        {
            return Some(super::offset_normal::OFFSET_NORMAL);
        }
        if text.eq_ignore_ascii_case("curvefillet")
            || text.eq_ignore_ascii_case("fillet")
            || text.eq_ignore_ascii_case(super::corner_ops::FILLET)
        {
            return Some(super::corner_ops::FILLET);
        }
        if text.eq_ignore_ascii_case("curvechamfer")
            || text.eq_ignore_ascii_case("chamfer")
            || text.eq_ignore_ascii_case(super::corner_ops::CHAMFER)
        {
            return Some(super::corner_ops::CHAMFER);
        }
        if text.eq_ignore_ascii_case("crvseam")
            || text.eq_ignore_ascii_case(super::crv_seam::CRV_SEAM)
        {
            return Some(super::crv_seam::CRV_SEAM);
        }
        if text.eq_ignore_ascii_case("curve")
            || text.eq_ignore_ascii_case(super::control_curve::CONTROL_CURVE)
        {
            return Some(super::control_curve::CONTROL_CURVE);
        }
        if text.eq_ignore_ascii_case("continuecurve")
            || text.eq_ignore_ascii_case(super::continue_curve::CONTINUE_CURVE)
        {
            return Some(super::continue_curve::CONTINUE_CURVE);
        }
        if text.eq_ignore_ascii_case("polylineonsrf")
            || text.eq_ignore_ascii_case("polylineonsurface")
            || text.eq_ignore_ascii_case("polyline on surface")
            || text.eq_ignore_ascii_case(super::polyline_on_surface::POLYLINE_ON_SURFACE)
        {
            return Some(super::polyline_on_surface::POLYLINE_ON_SURFACE);
        }
        if text.eq_ignore_ascii_case("sketch") || text.eq_ignore_ascii_case(super::sketch::SKETCH) {
            return Some(super::sketch::SKETCH);
        }
        if text.eq_ignore_ascii_case("sketchonsrf")
            || text.eq_ignore_ascii_case("sketchonsurface")
            || text.eq_ignore_ascii_case("sketch on surface")
            || text.eq_ignore_ascii_case(super::sketch_on_surface::SKETCH_ON_SURFACE)
        {
            return Some(super::sketch_on_surface::SKETCH_ON_SURFACE);
        }
        if text.eq_ignore_ascii_case("dupborder")
            || text.eq_ignore_ascii_case(super::dup_border::DUP_BORDER)
        {
            return Some(super::dup_border::DUP_BORDER);
        }
        for descriptor in [Self::line_descriptor(), Self::point_descriptor()] {
            if text.eq_ignore_ascii_case(&descriptor.feature)
                || descriptor
                    .aliases
                    .iter()
                    .any(|alias| text.eq_ignore_ascii_case(alias))
            {
                return Some(if descriptor.feature == LINE {
                    LINE
                } else {
                    POINT
                });
            }
        }
        // The Curve menu shows labels; typing the visible label must reach the same command
        // as clicking the row (OM9-L2-03-16), including the two rows whose documented script
        // is a shared Matrix sequence (`Curve 2 Views with History`, `Sketch on Mesh`).
        // `Soft Edit` is also a Surface row: the Curve row keeps the shared label here and
        // `resolve_typed_command` hands it to Surface when a surface is selected.
        if let Some(binding) = crate::curve_menu::by_label(text) {
            return Some(binding.feature);
        }
        // Matrix Surface Tools: accept the authored script name (`_Sweep1`, `gvSweepHistory`,
        // ...) and the visible row label, so the menu and the command line reach one row.
        // `Rebuild` is deliberately left to the curve command above: Edit ▸ Rebuild and
        // Others ▸ Rebuild share `_Rebuild`, and the typed path picks by selection.
        if let Some(binding) = crate::surface_menu::by_command_line(text)
            && binding.feature != surface_edit::REBUILD
        {
            return Some(binding.feature);
        }
        if let Some(binding) = crate::surface_menu::by_label(text)
            && binding.feature != surface_edit::REBUILD
        {
            return Some(binding.feature);
        }
        None
    }

    /// Resolve a typed name against the current selection.
    ///
    /// The Matrix Curve menu and Surface menu share the labels `Rebuild` and `Soft Edit`
    /// (and the script `_Rebuild`). A surface in the selection gives the Surface row the
    /// command, exactly as clicking that menu row would; a curve in the selection gives the
    /// Curve row the command. Without a matching selection the registry keeps the
    /// selection-free precedence of [`Self::resolve_alias`].
    pub fn resolve_typed_command(&self, text: &str) -> Option<&'static str> {
        let selected = |wanted: fn(&Geometry) -> bool| {
            self.selection.iter().any(|id| {
                self.document()
                    .entities
                    .get(id)
                    .is_some_and(|entity| wanted(&entity.geometry))
            })
        };
        let surface_selected = selected(|geometry| matches!(geometry, Geometry::Surface(_)));
        let curve_selected = selected(|geometry| matches!(geometry, Geometry::Curve(_)));
        if surface_selected && let Some(binding) = crate::surface_menu::by_label(text) {
            return Some(binding.feature);
        }
        if curve_selected && let Some(binding) = crate::curve_menu::by_label(text) {
            return Some(binding.feature);
        }
        Self::resolve_alias(text)
    }

    /// Canonical immediate view commands accepted by the command line.
    pub fn view_command(text: &str) -> Option<&'static str> {
        let text = text.trim();
        let aliases: &[&str] = match text {
            t if t.eq_ignore_ascii_case("ZoomExtents")
                || t.eq_ignore_ascii_case("ZE")
                || t.eq_ignore_ascii_case("ViewZoomZoomExtents") =>
            {
                &["ViewZoomZoomExtents"]
            }
            t if t.eq_ignore_ascii_case("ZoomSelected")
                || t.eq_ignore_ascii_case("ZS")
                || t.eq_ignore_ascii_case("ViewZoomZoomSelected") =>
            {
                &["ViewZoomZoomSelected"]
            }
            t if t.eq_ignore_ascii_case("Home") || t.eq_ignore_ascii_case("ViewHome") => {
                &["ViewHome"]
            }
            t if t.eq_ignore_ascii_case("Center") || t.eq_ignore_ascii_case("ViewCenter") => {
                &["ViewCenter"]
            }
            _ => return None,
        };
        aliases.first().copied()
    }
}
