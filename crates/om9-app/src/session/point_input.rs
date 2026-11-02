// SPDX-License-Identifier: LGPL-2.1-or-later
//! Viewport-click input: one world point is routed to the pending command.
//!
//! Extracted unchanged from `session.rs`; the pending-command state and helpers stay on`n//! `Controller` in the parent module.
use super::*;

impl Controller {
    pub fn accept_world_point(&mut self, point: Point3) -> Result<()> {
        finite(point)?;
        #[cfg(feature = "occt-brep")]
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|pending| pending.feature == "SolidSlab")
        {
            return self.slab_accept_point(point);
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|pending| pending.feature == "OM9-SOLID-015")
        {
            return self.solid_torus_accept_point(point);
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|pending| pending.feature == "OM9-SOLID-013")
        {
            return self.solid_ellipsoid_accept_point(point);
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|pending| pending.feature == "OM9-SOLID-018")
        {
            return self.solid_pyramid_accept_point(point);
        }
        if self.solid_pending.as_ref().is_some_and(|pending| {
            matches!(
                pending.feature,
                "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
            )
        }) {
            return self.solid_cone_accept_point(point);
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|pending| pending.feature == "OM9-SOLID-011")
        {
            return self.solid_pipe_accept_radius_point(point);
        }
        if self.annotation_pending.is_some() {
            return self.accept_annotation_point(point);
        }
        if let Some(pending) = self.measure_pending.take() {
            return self.accept_measure_point(pending, point);
        }
        if let Some(pending) = self.transform_pending.take() {
            return self.accept_transform_point(pending, point);
        }
        if self.circle_pending.is_some() {
            return self.accept_circle_point(point);
        }
        if self.control_curve_pending.is_some() {
            return self.accept_control_curve_point(point);
        }
        if self.continue_curve_pending.is_some() {
            return self.accept_continue_curve_point(point);
        }
        if self.rect_pending.is_some() {
            return self.accept_rect_point(point);
        }
        if self.polyline_pending.is_some() {
            return self.accept_polyline_point(point);
        }
        if self.interp_pending.is_some() {
            return self.accept_interp_point(point);
        }
        if self.polygon_pending.is_some() {
            return self.accept_polygon_point(point);
        }
        if self.continue_interp_pending.is_some() {
            return self.accept_continue_interp_point(point);
        }
        if self.helix_pending.is_some() {
            return self.accept_helix_point(point);
        }
        if self.spiral_pending.is_some() {
            return self.accept_spiral_point(point);
        }
        if self.ellipse_pending.is_some() {
            return self.accept_ellipse_point(point);
        }
        if self.arc_pending.is_some() {
            return self.accept_arc_point(point);
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|pending| pending.sphere_mode)
        {
            return self.accept_sphere_point(point);
        }
        if self.solid_pending.as_ref().is_some_and(|pending| {
            pending.box_mode
                && pending.box_kind == "Center"
                && pending.points.len() == 1
                && pending.box_length > 0.0
                && pending.box_width.is_some()
        }) {
            let pending = self.solid_pending.take().expect("center box pending");
            let center = pending.points[0];
            let plane = pending.box_plane.unwrap_or_else(|| self.input_plane());
            let normal = plane.normal();
            let height: f64 = (0..3).map(|i| (point[i] - center[i]) * normal[i]).sum();
            if !height.is_finite() || height.abs() <= 1e-9 {
                self.solid_pending = Some(pending);
                return Err(Error::Invalid("box height must be non-zero"));
            }
            let result = self.solid_box_dimensions(
                center,
                pending.box_length,
                pending.box_width.unwrap_or(pending.box_length),
                height,
                plane,
            );
            if result.is_err() {
                self.solid_pending = Some(pending);
            }
            return result;
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|p| p.box_mode && p.box_kind == "Cube" && p.points.len() == 1)
        {
            let pending = self.solid_pending.take().expect("cube pending");
            let plane = pending.box_plane.unwrap_or_else(|| self.input_plane());
            let result = self.solid_box_points(pending.points[0], point, 1.0, "Cube", None, plane);
            if result.is_err() {
                self.solid_pending = Some(pending);
            }
            return result;
        }
        let box_input_plane = self.input_plane();
        if let Some(pending) = self.solid_pending.as_mut()
            && pending.box_mode
            && pending.points.len() < 2
        {
            if pending.points.is_empty() {
                pending.box_plane = Some(box_input_plane);
            } else if matches!(pending.box_kind, "ThreePoint" | "Vertical") {
                solid::box_edge_axes(
                    pending.points[0],
                    point,
                    pending.box_plane.unwrap_or(box_input_plane),
                    pending.box_kind == "Vertical",
                )?;
            }
            pending.points.push(point);
            return Ok(());
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|pending| pending.box_mode && pending.points.len() >= 2)
        {
            let pending = self.solid_pending.take().expect("box pending");
            let first = pending.points[0];
            let second = pending.points[1];
            let plane = pending.box_plane.unwrap_or_else(|| self.input_plane());
            let axes = if matches!(pending.box_kind, "ThreePoint" | "Vertical") {
                match solid::box_edge_axes(first, second, plane, pending.box_kind == "Vertical") {
                    Ok((_, axes)) => Some(axes),
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Err(e);
                    }
                }
            } else {
                None
            };
            if let Some(axes) = axes
                && pending.box_width.is_none()
            {
                let width: f64 = (0..3).map(|i| (point[i] - first[i]) * axes[1][i]).sum();
                if !width.is_finite() || width.abs() <= 1e-9 {
                    self.solid_pending = Some(pending);
                    return Err(Error::Invalid("box width must be non-zero"));
                }
                let mut pending = pending;
                pending.box_width = Some(width);
                self.solid_pending = Some(pending);
                return Ok(());
            }
            let normal = axes.map_or_else(|| plane.normal(), |v| v[2]);
            let height: f64 = (0..3).map(|i| (point[i] - first[i]) * normal[i]).sum();
            if !height.is_finite() || height.abs() <= 1e-9 {
                self.solid_pending = Some(pending);
                return Err(Error::Invalid("box height must be non-zero"));
            }
            let result = self.solid_box_points(
                first,
                second,
                height,
                pending.box_kind,
                pending.box_width,
                plane,
            );
            if result.is_err() {
                self.solid_pending = Some(pending);
            }
            return result;
        }
        if self.slash_pending.is_some() {
            let mut pending = self.slash_pending.as_ref().expect("slash").clone();
            pending.points.push(point);
            if pending.points.len() < 2 {
                self.slash_pending = Some(pending);
                self.last_point = Some(om9_core::coords::WorldPoint::try_new(point)?);
                return Ok(());
            }
            let slash = [pending.points[0], pending.points[1]];
            self.slash_pending = None;
            let brep = self.build_sweep_history(
                pending.feature,
                &pending.curves,
                pending.rail_count,
                pending.closed,
                pending.flip,
                Some(slash),
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
            return self.commit_entity_with_history(
                pending.feature,
                surface_edit::label(pending.feature),
                entity,
                "Sweep2History",
                inputs,
            );
        }
        if self.top11_pending.is_some() {
            self.last_point = Some(om9_core::coords::WorldPoint::try_new(point)?);
            self.top11_accept_point(point)?;
            return Ok(());
        }
        let s = self.validate_session()?.clone();
        self.last_point = Some(om9_core::coords::WorldPoint::try_new(point)?);
        if surface_edit::plane_like(s.feature) {
            let collected = {
                let session = self.session.as_mut().expect("session");
                session.points.push(point);
                session.points.clone()
            };
            if collected.len() < 3 {
                self.events.emit(
                    crate::command_events::EventKind::Progress,
                    s.feature,
                    "corner",
                    None,
                );
                return Ok(());
            }
            let corners = [collected[0], collected[1], collected[2]];
            let surface = surface_edit::plane_from_corners(corners, self.coordinator.limits())?;
            return self.commit_geometry(
                s.feature,
                surface_edit::label(s.feature),
                Geometry::Surface(surface),
            );
        }
        let geometry = if s.feature == POINT {
            Geometry::Point(point)
        } else {
            let Some(center) = s.start else {
                self.session.as_mut().expect("session").start = Some(point);
                self.events.emit(
                    crate::command_events::EventKind::Progress,
                    s.feature,
                    "first point",
                    None,
                );
                return Ok(());
            };
            let start = if s.both_sides {
                std::array::from_fn(|i| 2. * center[i] - point[i])
            } else {
                center
            };
            finite(start)?;
            // A fixed length keeps the clicked direction and scales it.
            let end = match s.length_mm {
                None => point,
                Some(fixed) => {
                    let d = [
                        point[0] - start[0],
                        point[1] - start[1],
                        point[2] - start[2],
                    ];
                    let span = d[0].hypot(d[1]).hypot(d[2]);
                    if !span.is_finite() || span <= 0. {
                        return Err(Error::Invalid("direction for the fixed length"));
                    }
                    let scale = fixed / span;
                    let end = std::array::from_fn(|i| start[i] + d[i] * scale);
                    finite(end)?;
                    end
                }
            };
            let length = (end[0] - start[0])
                .hypot(end[1] - start[1])
                .hypot(end[2] - start[2]);
            if !length.is_finite() || length <= 0. {
                return Err(Error::Invalid("nonzero finite line length"));
            }
            Geometry::Curve(Curve::Line {
                start,
                end,
                domain: [0., length],
            })
        };
        let name = if s.feature == POINT { "Point" } else { "Line" };
        self.commit_geometry(s.feature, name, geometry)
    }
}
