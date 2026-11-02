// SPDX-License-Identifier: LGPL-2.1-or-later
//! Cone, truncated-cone, torus and ellipsoid solid stages: point/value input and typed text.
//!
//! Moved unchanged from `solid.rs`; shares `SolidPending` with the parent session module.
use super::solid::{ConeFrame, SolidPending};
use super::*;

impl Controller {
    pub fn solid_cone_input_plane(&self) -> Option<super::CPlane> {
        let pending = self.solid_pending.as_ref()?;
        if pending.feature == "OM9-SOLID-015" {
            return pending
                .cone_plane
                .as_deref()?
                .plane
                .with_origin(*pending.points.first()?)
                .ok();
        }
        if pending.feature == "OM9-SOLID-013" {
            return pending
                .cone_plane
                .as_deref()?
                .plane
                .with_origin(pending.ellipsoid_center())
                .ok();
        }
        if pending.feature == "OM9-SOLID-018" {
            return pending.cone_plane.as_deref().map(|f| f.plane);
        }
        if matches!(
            pending.feature,
            "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
        ) {
            return pending.cone_plane.as_deref().map(|f| f.plane);
        }
        None
    }

    pub(super) fn solid_torus_accept_point(&mut self, mut point: [f64; 3]) -> Result<()> {
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|p| p.torus_around && p.points.is_empty())
        {
            return self.solid_cone_around_pick(point);
        }
        if let Some(stamp) = self
            .solid_pending
            .as_ref()
            .and_then(|p| p.cone_plane.as_deref())
            .and_then(|f| f.around_source)
        {
            if self.document().stamp(stamp.object).ok().as_ref() != Some(&stamp) {
                return Err(Error::Stale("AroundCurve source"));
            }
        }
        if let Some(pending) = self
            .solid_pending
            .as_ref()
            .filter(|p| p.torus_three_point && p.cone_flags.radius_phase() == 1)
        {
            let projected = pending.cone_project_base_point(point)?;
            let delta = std::array::from_fn::<_, 3, _>(|i| projected[i] - pending.points[0][i]);
            return self.solid_cone_radius_value(delta[0].hypot(delta[1]).hypot(delta[2]), false);
        }
        if let Some(pending) = self
            .solid_pending
            .as_ref()
            .filter(|p| p.points.len() < if p.torus_three_point { 3 } else { 2 })
        {
            let mut held = self.scene_cache_usage()?;
            let mut extra = om9_core::budget::Usage::array::<om9_core::geometry::Point3>(1)?;
            extra.points = 1;
            if pending.points.is_empty() {
                extra.bytes = extra
                    .bytes
                    .checked_add(std::mem::size_of::<ConeFrame>() as u64)
                    .ok_or(Error::Overflow("torus frame bytes"))?;
            }
            held.add(extra, self.coordinator.limits())?;
            self.coordinator.retained_usage_with_held_usage(held)?;
        }
        let plane = self.input_plane();
        let mut pending = self
            .solid_pending
            .take()
            .ok_or(Error::Missing("Torus session"))?;
        if !pending.values.is_empty() {
            self.solid_pending = Some(pending);
            return Err(Error::Invalid("finish entering radii or restart Torus"));
        }
        if pending.torus_three_point && pending.cone_flags.radius_phase() == 2 {
            let (center, radius) = match pending.cone_radius_base(point) {
                Ok(value) => value,
                Err(error) => {
                    self.solid_pending = Some(pending);
                    return Err(error);
                }
            };
            let plane = pending.cone_plane.as_deref().expect("torus frame").plane;
            let d = std::array::from_fn::<_, 3, _>(|i| pending.points[1][i] - pending.points[0][i]);
            let x = (0..3).map(|i| d[i] * plane.x[i]).sum::<f64>();
            let y = (0..3).map(|i| d[i] * plane.y[i]).sum::<f64>();
            let chord = x.hypot(y);
            let ringpoint = std::array::from_fn(|i| {
                center[i] + radius * (-y * plane.x[i] + x * plane.y[i]) / chord
            });
            if pending.points.try_reserve_exact(1).is_err() {
                self.solid_pending = Some(pending);
                return Err(Error::Budget("torus points"));
            }
            pending.points.push(ringpoint);
            pending.cone_flags.set_radius_phase(3);
            self.solid_pending = Some(pending);
            return Ok(());
        }
        if pending.points.is_empty() {
            pending.cone_plane = Some(Box::new(ConeFrame {
                plane,
                radius: 0.,
                fit_input_plane: None,
                around_source: None,
                tangent: None,
            }));
        } else {
            let plane = pending.cone_plane.as_deref().expect("torus frame").plane;
            let dot = |a: [f64; 3], b: [f64; 3]| (0..3).map(|i| a[i] * b[i]).sum::<f64>();
            if pending.points.len() == 1 {
                let origin = pending.points[0];
                let d = std::array::from_fn::<_, 3, _>(|i| point[i] - origin[i]);
                let u = dot(d, plane.x);
                let v = dot(d, plane.y);
                if !u.is_finite() || !v.is_finite() || u.hypot(v) <= 1e-9 {
                    self.solid_pending = Some(pending);
                    return Err(Error::Invalid("torus ring radius must be positive"));
                }
                point = std::array::from_fn(|i| origin[i] + u * plane.x[i] + v * plane.y[i]);
                if pending.torus_vertical {
                    let radius = u.hypot(v);
                    let x = std::array::from_fn(|i| (u * plane.x[i] + v * plane.y[i]) / radius);
                    let frame = match super::CPlane::new(origin, x, plane.normal()) {
                        Ok(v) => v,
                        Err(error) => {
                            self.solid_pending = Some(pending);
                            return Err(error);
                        }
                    };
                    if pending.points.try_reserve_exact(1).is_err() {
                        self.solid_pending = Some(pending);
                        return Err(Error::Budget("torus points"));
                    }
                    pending.points.push(point);
                    let stored = pending.cone_plane.as_deref_mut().expect("torus frame");
                    stored.fit_input_plane = Some(plane);
                    stored.plane = frame;
                    self.solid_pending = Some(pending);
                    return Ok(());
                }
            } else if pending.torus_three_point && pending.points.len() == 2 {
                let origin = pending.points[0];
                let d = std::array::from_fn::<_, 3, _>(|i| point[i] - origin[i]);
                let u = dot(d, plane.x);
                let v = dot(d, plane.y);
                point = std::array::from_fn(|i| origin[i] + u * plane.x[i] + v * plane.y[i]);
                if pending.points.try_reserve_exact(1).is_err() {
                    self.solid_pending = Some(pending);
                    return Err(Error::Budget("torus points"));
                }
                pending.points.push(point);
                if let Err(error) = pending.torus_base() {
                    pending.points.pop();
                    self.solid_pending = Some(pending);
                    return Err(error);
                }
                self.solid_pending = Some(pending);
                return Ok(());
            } else {
                let (center, plane, ring, tube) = match pending.torus_tube_geometry(point) {
                    Ok(v) => v,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Err(error);
                    }
                };
                let n = plane.normal();
                let revision = self.document().revision;
                let result = self.solid_torus_points(center, [plane.x, plane.y, n], ring, tube);
                if result.is_err() && self.document().revision == revision {
                    self.solid_pending = Some(pending);
                }
                return result;
            }
        }
        if pending.points.try_reserve_exact(1).is_err() {
            self.solid_pending = Some(pending);
            return Err(Error::Budget("torus points"));
        }
        pending.points.push(point);
        self.solid_pending = Some(pending);
        Ok(())
    }

    pub(super) fn solid_ellipsoid_accept_point(&mut self, mut point: [f64; 3]) -> Result<()> {
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|p| p.ellipsoid_around && p.points.is_empty())
        {
            return self.solid_cone_around_pick(point);
        }
        if let Some(stamp) = self
            .solid_pending
            .as_ref()
            .and_then(|p| p.cone_plane.as_deref())
            .and_then(|f| f.around_source)
        {
            if self.document().stamp(stamp.object).ok().as_ref() != Some(&stamp) {
                return Err(Error::Stale("AroundCurve source"));
            }
        }
        if let Some(pending) = self
            .solid_pending
            .as_ref()
            .filter(|p| p.points.len() < if p.ellipsoid_corner { 2 } else { 3 })
        {
            let mut held = self.scene_cache_usage()?;
            let mut extra = om9_core::budget::Usage::array::<om9_core::geometry::Point3>(1)?;
            extra.points = 1;
            if pending.points.is_empty() {
                extra.bytes = extra
                    .bytes
                    .checked_add(std::mem::size_of::<ConeFrame>() as u64)
                    .ok_or(Error::Overflow("ellipsoid frame bytes"))?;
            }
            held.add(extra, self.coordinator.limits())?;
            self.coordinator.retained_usage_with_held_usage(held)?;
        }
        let plane = self.input_plane();
        let mut pending = self
            .solid_pending
            .take()
            .ok_or(Error::Missing("Ellipsoid session"))?;
        if !pending.values.is_empty() {
            self.solid_pending = Some(pending);
            return Err(Error::Invalid("finish entering radii or restart Ellipsoid"));
        }
        if pending.points.is_empty() {
            pending.cone_plane = Some(Box::new(ConeFrame {
                plane,
                radius: 0.0,
                fit_input_plane: None,
                around_source: None,
                tangent: None,
            }));
        } else {
            let plane = pending
                .cone_plane
                .as_deref()
                .expect("ellipsoid frame")
                .plane;
            if pending.ellipsoid_corner {
                let dot = |a: [f64; 3], b: [f64; 3]| (0..3).map(|i| a[i] * b[i]).sum::<f64>();
                if pending.points.len() == 1 {
                    let delta = std::array::from_fn::<_, 3, _>(|i| point[i] - pending.points[0][i]);
                    let u = dot(delta, plane.x);
                    let v = dot(delta, plane.y);
                    if !u.is_finite() || !v.is_finite() || u.abs() <= 2e-9 || v.abs() <= 2e-9 {
                        self.solid_pending = Some(pending);
                        return Err(Error::Invalid(
                            "ellipse box must have nonzero width and height",
                        ));
                    }
                    point = std::array::from_fn(|i| {
                        pending.points[0][i] + u * plane.x[i] + v * plane.y[i]
                    });
                    if pending.points.try_reserve_exact(1).is_err() {
                        self.solid_pending = Some(pending);
                        return Err(Error::Budget("ellipsoid points"));
                    }
                    pending.points.push(point);
                    self.solid_pending = Some(pending);
                    return Ok(());
                }
                let (center, frame, axes) = match pending.ellipsoid_at_height(point) {
                    Ok(value) => value,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Err(error);
                    }
                };
                let revision = self.document().revision;
                let result = self.solid_ellipsoid_points(center, frame, axes, None);
                if result.is_err() && self.document().revision == revision {
                    self.solid_pending = Some(pending);
                }
                return result;
            }
            if (pending.ellipsoid_diameter && pending.points.len() == 1)
                || (pending.ellipsoid_foci && pending.points.len() <= 2)
            {
                let start = pending.points[0];
                let normal = plane.normal();
                let height = (0..3)
                    .map(|i| (point[i] - start[i]) * normal[i])
                    .sum::<f64>();
                for i in 0..3 {
                    point[i] -= height * normal[i];
                }
            }
            let center = pending.ellipsoid_center();
            let delta = std::array::from_fn::<_, 3, _>(|i| point[i] - center[i]);
            let dot = |a: [f64; 3], b: [f64; 3]| (0..3).map(|i| a[i] * b[i]).sum::<f64>();
            let normal = plane.normal();
            let first = if pending.points.len() >= 2 {
                let d = std::array::from_fn::<_, 3, _>(|i| pending.points[1][i] - center[i]);
                let x = dot(d, plane.x);
                let y = dot(d, plane.y);
                let length = x.hypot(y);
                std::array::from_fn(|i| (x * plane.x[i] + y * plane.y[i]) / length)
            } else {
                plane.x
            };
            let second = [
                normal[1] * first[2] - normal[2] * first[1],
                normal[2] * first[0] - normal[0] * first[2],
                normal[0] * first[1] - normal[1] * first[0],
            ];
            let distance = match pending.points.len() {
                1 => dot(delta, plane.x).hypot(dot(delta, plane.y)),
                2 if pending.ellipsoid_foci => match pending.ellipsoid_foci_radii(point) {
                    Ok([_, b]) => b,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Err(error);
                    }
                },
                2 => dot(delta, second).abs(),
                _ => dot(delta, normal).abs(),
            };
            if !distance.is_finite() || distance <= 1e-9 {
                self.solid_pending = Some(pending);
                return Err(Error::Invalid("ellipsoid axis must have positive length"));
            }
            if pending.points.len() == 3 {
                let (center, frame, axes) = match pending.ellipsoid_at_height(point) {
                    Ok(value) => value,
                    Err(error) => {
                        self.solid_pending = Some(pending);
                        return Err(error);
                    }
                };
                let revision = self.document().revision;
                let foci = (pending.ellipsoid_foci && pending.ellipsoid_mark_foci)
                    .then_some([pending.points[0], pending.points[1]]);
                let result = self.solid_ellipsoid_points(center, frame, axes, foci);
                if result.is_err() && self.document().revision == revision {
                    self.solid_pending = Some(pending);
                }
                return result;
            }
        }
        if pending.points.try_reserve_exact(1).is_err() {
            self.solid_pending = Some(pending);
            return Err(Error::Budget("ellipsoid points"));
        }
        pending.points.push(point);
        self.solid_pending = Some(pending);
        Ok(())
    }

    #[inline(never)]
    pub(super) fn solid_cone_around_center(&mut self, fraction: f64) -> Result<()> {
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            return Err(Error::Invalid("OnCurve fraction must be 0..1"));
        }
        let pending = self
            .solid_pending
            .as_ref()
            .filter(|p| {
                ((matches!(
                    p.feature,
                    "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
                ) && p.cone_base_points == 5)
                    || (p.feature == "OM9-SOLID-018" && p.pyramid_around)
                    || (p.feature == "OM9-SOLID-013" && p.ellipsoid_around)
                    || (p.feature == "OM9-SOLID-015" && p.torus_around))
                    && p.points.is_empty()
            })
            .ok_or(Error::Missing("AroundCurve center stage"))?;
        let _ = pending;
        let ids = self.selected_ordered();
        if ids.len() != 1 {
            return Err(Error::Missing("select one curve"));
        }
        let id = ids[0];
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Stale("AroundCurve source"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("AroundCurve requires a curve"));
        };
        let local = om9_core::curve_ops::point_at(curve, fraction, self.coordinator.limits())?;
        let tangent = om9_core::curve_ops::tangent_at(curve, fraction, self.coordinator.limits())?;
        let center = entity.placement.point(local)?;
        let matrix = entity.placement.0;
        let axis = std::array::from_fn::<_, 3, _>(|i| {
            (0..3).map(|j| matrix[i][j] * tangent[j]).sum::<f64>()
        });
        let magnitude = axis[0].hypot(axis[1]).hypot(axis[2]);
        if !magnitude.is_finite() || magnitude <= 1e-12 {
            return Err(Error::Invalid("AroundCurve tangent"));
        }
        let n = axis.map(|v| v / magnitude);
        let original = self.input_plane();
        let project = |a: [f64; 3]| {
            let d = (0..3).map(|i| a[i] * n[i]).sum::<f64>();
            std::array::from_fn::<_, 3, _>(|i| a[i] - d * n[i])
        };
        let mut x = project(original.x);
        let mut len = x[0].hypot(x[1]).hypot(x[2]);
        if len <= 1e-9 {
            x = project(original.y);
            len = x[0].hypot(x[1]).hypot(x[2]);
        }
        if !len.is_finite() || len <= 1e-9 {
            return Err(Error::Invalid("AroundCurve frame"));
        }
        x = x.map(|v| v / len);
        let y = [
            n[1] * x[2] - n[2] * x[1],
            n[2] * x[0] - n[0] * x[2],
            n[0] * x[1] - n[1] * x[0],
        ];
        let frame = super::CPlane::new(center, x, y)?;
        let stamp = self.document().stamp(id)?;
        let mut held = self.scene_cache_usage()?;
        let mut extra = om9_core::budget::Usage::array::<om9_core::geometry::Point3>(1)?;
        extra.points = 1;
        extra.bytes = extra
            .bytes
            .checked_add(std::mem::size_of::<ConeFrame>() as u64)
            .ok_or(Error::Overflow("cone frame bytes"))?;
        held.add(extra, self.coordinator.limits())?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let pending = self.solid_pending.as_mut().expect("AroundCurve session");
        pending
            .points
            .try_reserve_exact(1)
            .map_err(|_| Error::Budget("AroundCurve center"))?;
        pending.points.push(center);
        pending.cone_plane = Some(Box::new(ConeFrame {
            plane: frame,
            radius: 0.,
            fit_input_plane: None,
            around_source: Some(stamp),
            tangent: None,
        }));
        Ok(())
    }

    pub(super) fn solid_cone_around_pick(
        &mut self,
        point: om9_core::geometry::Point3,
    ) -> Result<()> {
        let ids = self.selected_ordered();
        if ids.len() != 1 {
            return Err(Error::Missing("select one curve"));
        }
        let entity = self
            .document()
            .entities
            .get(&ids[0])
            .ok_or(Error::Stale("AroundCurve source"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("AroundCurve requires a curve"));
        };
        let a = entity.placement.0;
        let axes = [
            [a[0][0], a[1][0], a[2][0]],
            [a[0][1], a[1][1], a[2][1]],
            [a[0][2], a[1][2], a[2][2]],
        ];
        let dot = |u: [f64; 3], v: [f64; 3]| (0..3).map(|i| u[i] * v[i]).sum::<f64>();
        let scale = dot(axes[0], axes[0]);
        if !scale.is_finite()
            || scale <= 1e-24
            || (1..3).any(|i| (dot(axes[i], axes[i]) - scale).abs() > scale * 1e-9)
            || dot(axes[0], axes[1]).abs() > scale * 1e-9
            || dot(axes[0], axes[2]).abs() > scale * 1e-9
            || dot(axes[1], axes[2]).abs() > scale * 1e-9
        {
            return Err(Error::Unsupported(
                "AroundCurve point picking requires a similarity placement",
            ));
        }
        let local = om9_core::top11::inverse_affine(entity.placement)?.point(point)?;
        let (fraction, _, _) = om9_core::curve_surface_ops::closest_point_on_curve(
            curve,
            local,
            self.coordinator.limits(),
        )?;
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|p| p.cone_base_points == 6)
        {
            self.solid_cone_tangent_contact(fraction)
        } else {
            self.solid_cone_around_center(fraction)
        }
    }

    #[inline(never)]
    pub(super) fn solid_cone_tangent_contact(&mut self, fraction: f64) -> Result<()> {
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            return Err(Error::Invalid("OnCurve fraction must be 0..1"));
        }
        if !self.solid_pending.as_ref().is_some_and(|p| {
            matches!(
                p.feature,
                "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
            ) && p.cone_base_points == 6
                && p.points.is_empty()
        }) {
            return Err(Error::Missing("Tangent contact stage"));
        }
        let ids = self.selected_ordered();
        if ids.len() != 1 {
            return Err(Error::Missing("select one tangent curve"));
        }
        let entity = self
            .document()
            .entities
            .get(&ids[0])
            .ok_or(Error::Stale("Tangent source"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("Tangent requires a curve"));
        };
        let point = entity.placement.point(om9_core::curve_ops::point_at(
            curve,
            fraction,
            self.coordinator.limits(),
        )?)?;
        let local = om9_core::curve_ops::tangent_at(curve, fraction, self.coordinator.limits())?;
        let a = entity.placement.0;
        let mut tangent =
            std::array::from_fn::<_, 3, _>(|i| (0..3).map(|j| a[i][j] * local[j]).sum::<f64>());
        let length = tangent[0].hypot(tangent[1]).hypot(tangent[2]);
        if !length.is_finite() || length <= 1e-12 {
            return Err(Error::Invalid("Tangent direction"));
        }
        tangent = tangent.map(|v| v / length);
        let frame = self.input_plane().with_origin(point)?;
        let normal = frame.normal();
        if (0..3).map(|i| tangent[i] * normal[i]).sum::<f64>().abs() > 1e-8 {
            return Err(Error::Unsupported(
                "tangent must lie in the construction plane",
            ));
        }
        let stamp = self.document().stamp(ids[0])?;
        let mut held = self.scene_cache_usage()?;
        let mut extra = om9_core::budget::Usage::array::<om9_core::geometry::Point3>(1)?;
        extra.points = 1;
        extra.bytes = extra
            .bytes
            .checked_add(std::mem::size_of::<ConeFrame>() as u64)
            .ok_or(Error::Overflow("cone frame bytes"))?;
        held.add(extra, self.coordinator.limits())?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let pending = self.solid_pending.as_mut().expect("Tangent session");
        pending
            .points
            .try_reserve_exact(1)
            .map_err(|_| Error::Budget("Tangent contact"))?;
        pending.points.push(point);
        pending.cone_plane = Some(Box::new(ConeFrame {
            plane: frame,
            radius: 0.,
            fit_input_plane: None,
            around_source: Some(stamp),
            tangent: Some(tangent),
        }));
        Ok(())
    }

    pub(super) fn solid_cone_fit_selected_points(&self) -> Result<Vec<om9_core::geometry::Point3>> {
        use om9_core::geometry::{Curve, Point3, Transform};
        fn append(
            out: &mut Vec<Point3>,
            points: &[Point3],
            t: Transform,
            limit: u64,
        ) -> Result<()> {
            let n = out
                .len()
                .checked_add(points.len())
                .ok_or(Error::Overflow("cone fit points"))?;
            if n as u64 > limit {
                return Err(Error::Budget("cone fit points"));
            }
            out.try_reserve(points.len())
                .map_err(|_| Error::Budget("cone fit points"))?;
            for p in points {
                out.push(t.point(*p)?);
            }
            Ok(())
        }
        fn curve(
            out: &mut Vec<Point3>,
            c: &Curve,
            t: Transform,
            limit: u64,
            depth: usize,
            max_depth: usize,
        ) -> Result<()> {
            if depth > max_depth {
                return Err(Error::Budget("cone fit curve depth"));
            }
            match c {
                Curve::Line { start, end, .. } => append(out, &[*start, *end], t, limit),
                Curve::Nurbs(n) => append(out, &n.poles, t, limit),
                Curve::Polycurve { segments, .. } => {
                    for part in segments {
                        curve(out, part, t, limit, depth + 1, max_depth)?;
                    }
                    Ok(())
                }
                _ => Err(Error::Unsupported("curve has no selectable control points")),
            }
        }
        let limits = self.coordinator.limits();
        let mut points = Vec::new();
        for id in self.selected_ordered() {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Stale("fit selection"))?;
            match &entity.geometry {
                Geometry::Point(p) => {
                    append(&mut points, &[*p], entity.placement, limits.max_points)?
                }
                Geometry::Curve(c) => curve(
                    &mut points,
                    c,
                    entity.placement,
                    limits.max_points,
                    0,
                    limits.max_depth,
                )?,
                Geometry::Surface(s) => {
                    append(&mut points, &s.poles, entity.placement, limits.max_points)?
                }
                Geometry::Mesh(m) => append(
                    &mut points,
                    &m.vertices,
                    entity.placement,
                    limits.max_points,
                )?,
                _ => {
                    return Err(Error::Unsupported(
                        "FitPoints needs points, control points or mesh vertices",
                    ));
                }
            }
        }
        let mut held = self.scene_cache_usage()?;
        let mut usage = om9_core::budget::Usage::array::<Point3>(points.capacity())?;
        usage.points = points.len() as u64;
        held.add(usage, limits)?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        Ok(points)
    }

    #[inline(never)]
    pub(super) fn solid_cone_finish_fit(&mut self, pending: &mut SolidPending) -> Result<()> {
        if pending.points.is_empty() {
            pending.points = self.solid_cone_fit_selected_points()?;
        }
        let (center, x, y, radius) = super::circle::fit_circle(&pending.points)?;
        let frame = super::CPlane::new(center, x, y)?;
        if pending.points.len() as u64 >= self.coordinator.limits().max_points {
            return Err(Error::Budget("cone fit points"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(pending.usage()?, self.coordinator.limits())?;
        let mut extra = om9_core::budget::Usage::array::<om9_core::geometry::Point3>(1)?;
        extra.points = 1;
        extra.bytes = extra
            .bytes
            .checked_add(std::mem::size_of::<ConeFrame>() as u64)
            .ok_or(Error::Overflow("cone frame bytes"))?;
        held.add(extra, self.coordinator.limits())?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        pending
            .points
            .try_reserve_exact(1)
            .map_err(|_| Error::Budget("cone fit points"))?;
        let original = pending
            .cone_plane
            .as_deref()
            .map(|f| f.plane)
            .unwrap_or_else(|| self.input_plane());
        pending.cone_plane = Some(Box::new(ConeFrame {
            plane: frame,
            radius: 0.,
            fit_input_plane: Some(original),
            around_source: None,
            tangent: None,
        }));
        pending.points.push(center);
        pending.values.push(radius);
        Ok(())
    }

    pub(super) fn solid_cone_accept_point(
        &mut self,
        point: om9_core::geometry::Point3,
    ) -> Result<()> {
        if self.solid_pending.as_ref().is_some_and(|p| {
            matches!(
                p.feature,
                "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
            ) && matches!(p.cone_base_points, 5 | 6)
                && p.points.is_empty()
        }) {
            return self.solid_cone_around_pick(point);
        }
        let plane = self.input_plane();
        if let Some(pending) = self
            .solid_pending
            .as_ref()
            .filter(|p| p.cone_base_points == 4 && p.values.is_empty())
        {
            if pending.points.len() as u64 >= self.coordinator.limits().max_points {
                return Err(Error::Budget("cone fit points"));
            }
            let mut held = self.scene_cache_usage()?;
            let mut extra = om9_core::budget::Usage::array::<om9_core::geometry::Point3>(1)?;
            extra.points = 1;
            if pending.cone_plane.is_none() {
                extra.bytes = extra
                    .bytes
                    .checked_add(std::mem::size_of::<ConeFrame>() as u64)
                    .ok_or(Error::Overflow("cone frame bytes"))?;
            }
            held.add(extra, self.coordinator.limits())?;
            self.coordinator.retained_usage_with_held_usage(held)?;
        }
        let pending = self
            .solid_pending
            .as_mut()
            .ok_or(Error::Missing("Truncated Cone session"))?;
        if pending.points.is_empty() {
            if pending.cone_base_points == 4 {
                pending
                    .points
                    .try_reserve_exact(1)
                    .map_err(|_| Error::Budget("cone fit points"))?;
            }
            pending.points.push(point);
            pending.cone_plane = Some(Box::new(ConeFrame {
                plane,
                radius: 0.,
                fit_input_plane: None,
                around_source: None,
                tangent: None,
            }));
            return Ok(());
        }
        if pending.cone_base_points == 4 && pending.values.is_empty() {
            pending
                .points
                .try_reserve_exact(1)
                .map_err(|_| Error::Budget("cone fit points"))?;
            pending.points.push(point);
            return Ok(());
        }
        if pending.cone_base_points == 6 && pending.values.is_empty() {
            if !pending.cone_flags.point_mode() {
                return Err(Error::Missing("choose Point before the point constraint"));
            }
            let (center, radius) = pending.cone_tangent_point_base(point)?;
            pending.points.push(center);
            pending.values.push(radius);
            return Ok(());
        }
        if pending.cone_flags.radius_phase() == 1 && pending.values.is_empty() {
            let projected = pending.cone_project_base_point(point)?;
            let radius = (0..3)
                .map(|i| (projected[i] - pending.points[0][i]).powi(2))
                .sum::<f64>()
                .sqrt();
            return self.solid_cone_radius_value(radius, false);
        }
        if pending.cone_flags.radius_phase() == 2 && pending.values.is_empty() {
            let (center, radius) = pending.cone_radius_base(point)?;
            pending.points.push(center);
            pending.values.push(radius);
            return Ok(());
        }
        if pending.cone_base_points == 3 && pending.values.is_empty() {
            if pending.points.len() == 1 {
                let projected = pending.cone_project_base_point(point)?;
                let distance = (0..3)
                    .map(|i| (projected[i] - pending.points[0][i]).powi(2))
                    .sum::<f64>()
                    .sqrt();
                if !distance.is_finite() || distance <= 1e-9 {
                    return Err(Error::Invalid("base points must be distinct"));
                }
                pending.points.push(projected);
                return Ok(());
            }
            let (center, radius) = pending.cone_three_point_base(point)?;
            pending.points.push(center);
            pending.values.push(radius);
            return Ok(());
        }
        if pending.cone_base_points == 2 && pending.values.is_empty() {
            let (center, radius) = pending.cone_diameter_base(point)?;
            pending.points.push(center);
            pending.values.push(radius);
            return Ok(());
        }
        let value = pending.cone_point_measure(point)?;
        self.solid_cone_accept_value(value, false)
    }

    pub(super) fn solid_cone_radius_value(&mut self, value: f64, typed: bool) -> Result<()> {
        let mut pending = self
            .solid_pending
            .take()
            .ok_or(Error::Missing("Truncated Cone session"))?;
        if !((matches!(
            pending.feature,
            "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
        ) && pending.cone_base_points == 3)
            || (pending.feature == "OM9-SOLID-015" && pending.torus_three_point))
            || pending.points.len() != 2
            || !pending.values.is_empty()
        {
            self.solid_pending = Some(pending);
            return Err(Error::Invalid("Radius requires two base points"));
        }
        let radius = if typed {
            match self.document().page_unit.length_to_mm(value) {
                Ok(v) => v,
                Err(e) => {
                    self.solid_pending = Some(pending);
                    return Err(e);
                }
            }
        } else {
            value
        };
        let half = (0..3)
            .map(|i| (pending.points[1][i] - pending.points[0][i]).powi(2))
            .sum::<f64>()
            .sqrt()
            / 2.;
        if !radius.is_finite() || !half.is_finite() || radius <= 1e-9 || radius < half {
            self.solid_pending = Some(pending);
            return Err(Error::Invalid("radius must be at least half the chord"));
        }
        pending
            .cone_plane
            .as_deref_mut()
            .expect("cone frame")
            .radius = radius;
        pending.cone_flags.set_radius_phase(2);
        self.solid_pending = Some(pending);
        Ok(())
    }

    pub(super) fn solid_cone_accept_value(&mut self, value: f64, typed: bool) -> Result<()> {
        let mut pending = self
            .solid_pending
            .take()
            .ok_or(Error::Missing("Truncated Cone session"))?;
        if pending.points.is_empty() {
            self.solid_pending = Some(pending);
            return Err(Error::Missing("pick center of base"));
        }
        if matches!(pending.cone_base_points, 2 | 3 | 4 | 6) && pending.values.is_empty() {
            self.solid_pending = Some(pending);
            return Err(Error::Missing("pick remaining base points"));
        }
        let area_stage = typed
            && pending.cone_area
            && matches!(pending.cone_base_points, 0 | 5)
            && pending.values.is_empty();
        let value = if area_stage {
            (value / std::f64::consts::PI).sqrt() * self.document().page_unit.factor()
        } else if typed {
            match self.document().page_unit.length_to_mm(value) {
                Ok(v) => v,
                Err(e) => {
                    self.solid_pending = Some(pending);
                    return Err(e);
                }
            }
        } else {
            value
        };
        let height_stage = pending.values.len()
            == if pending.feature == "OM9-SOLID-017" {
                2
            } else {
                1
            };
        let thickness_stage = pending.feature == "OM9-SOLID-017"
            && pending.values.len() == 1
            && pending.cone_flags.wall_thickness();
        let value = if typed && pending.cone_diameter && !height_stage && !thickness_stage {
            value / 2.
        } else {
            value
        };
        if !value.is_finite()
            || (height_stage && value.abs() <= 1e-9)
            || (!height_stage && value <= 1e-9)
        {
            self.solid_pending = Some(pending);
            return Err(Error::Invalid("invalid base radius or height"));
        }
        let value = if thickness_stage && typed {
            let inner = pending.values[0] - value;
            if !inner.is_finite() || inner <= 1e-9 {
                self.solid_pending = Some(pending);
                return Err(Error::Invalid(
                    "wall thickness must be smaller than first radius",
                ));
            }
            inner
        } else {
            value
        };
        if pending.feature == "OM9-SOLID-017"
            && pending.values.len() == 1
            && (pending.values[0] - value).abs() <= 1e-9
        {
            self.solid_pending = Some(pending);
            return Err(Error::Invalid("tube radii must differ"));
        }
        if pending.feature == "OM9-SOLID-017" && pending.values.len() == 2 {
            if let Some(stamp) = pending.cone_plane.as_deref().and_then(|f| f.around_source) {
                if self.document().stamp(stamp.object).ok() != Some(stamp) {
                    self.solid_pending = Some(pending);
                    return Err(Error::Stale("Tube source changed"));
                }
            }
            let plane = pending.cone_plane.as_deref().expect("tube frame").plane;
            let mut center = pending.cone_center();
            let mut height = value;
            if pending.cone_flags.both_sides() {
                let normal = plane.normal();
                center = std::array::from_fn(|i| center[i] - value * normal[i]);
                height = 2. * value;
                if !height.is_finite() || !center.iter().all(|v| v.is_finite()) {
                    self.solid_pending = Some(pending);
                    return Err(Error::Invalid("tube BothSides overflow"));
                }
            }
            let result = self.solid_tube_oriented(
                center,
                plane,
                pending.values[0],
                pending.values[1],
                height,
            );
            if result.is_err() {
                self.solid_pending = Some(pending);
            }
            return result;
        }
        if matches!(pending.feature, "OM9-SOLID-016" | "OM9-SOLID-019") && pending.values.len() == 1
        {
            if let Some(stamp) = pending.cone_plane.as_deref().and_then(|f| f.around_source) {
                if self.document().stamp(stamp.object).ok() != Some(stamp) {
                    self.solid_pending = Some(pending);
                    return Err(Error::Stale("base curve changed"));
                }
            }
            let result = self.solid_cone_commit(
                pending.feature,
                pending.cone_center(),
                pending.cone_plane.as_deref().expect("cylinder frame").plane,
                pending.values[0],
                value,
                if pending.feature == "OM9-SOLID-019" {
                    0.
                } else {
                    pending.values[0]
                },
                pending.cone_flags.solid(),
            );
            if result.is_err() {
                self.solid_pending = Some(pending);
            }
            return result;
        }
        if pending.values.len() < 2 {
            pending.values.push(value);
            if thickness_stage {
                pending.cone_flags.set_wall_thickness(false);
            }
            self.solid_pending = Some(pending);
            return Ok(());
        }
        if let Some(stamp) = pending.cone_plane.as_deref().and_then(|f| f.around_source) {
            if self.document().stamp(stamp.object).ok() != Some(stamp) {
                self.solid_pending = Some(pending);
                return Err(Error::Stale("Truncated Cone source changed"));
            }
        }
        let result = self.solid_cone_commit(
            pending.feature,
            pending.cone_center(),
            pending.cone_plane.as_deref().expect("cone frame").plane,
            pending.values[0],
            pending.values[1],
            value,
            pending.cone_flags.solid(),
        );
        if result.is_err() {
            self.solid_pending = Some(pending);
        }
        result
    }

    #[inline(never)]
    pub(super) fn submit_cone_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim();
        if input.eq_ignore_ascii_case("cancel") || input.eq_ignore_ascii_case("esc") {
            self.cancel();
            return Some(Ok(()));
        }
        if pending.feature == "OM9-SOLID-017"
            && (input.eq_ignore_ascii_case("bothsides")
                || input.to_ascii_lowercase().starts_with("bothsides="))
        {
            let value = if input.eq_ignore_ascii_case("bothsides") {
                !pending.cone_flags.both_sides()
            } else if input.eq_ignore_ascii_case("bothsides=yes") {
                true
            } else if input.eq_ignore_ascii_case("bothsides=no") {
                false
            } else {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("BothSides must be Yes or No")));
            };
            pending.cone_flags.set_both_sides(value);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.feature == "OM9-SOLID-017"
            && input.to_ascii_lowercase().starts_with("wallthickness")
        {
            if pending.values.len() != 1 {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "WallThickness requires the first radius",
                )));
            }
            let lower = input.to_ascii_lowercase();
            if lower == "wallthickness" || lower == "wallthickness=" {
                pending.cone_flags.set_wall_thickness(true);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            let value = lower
                .strip_prefix("wallthickness=")
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|v| v.is_finite());
            if let Some(value) = value {
                pending.cone_flags.set_wall_thickness(true);
                self.solid_pending = Some(pending);
                return Some(self.solid_cone_accept_value(value, true));
            }
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "WallThickness requires a positive value",
            )));
        }
        if input.eq_ignore_ascii_case("solid")
            || input.eq_ignore_ascii_case("solid=yes")
            || input.eq_ignore_ascii_case("solid=no")
        {
            if pending.feature == "OM9-SOLID-017" {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Unsupported("Tube is a closed solid")));
            }
            let solid = if input.eq_ignore_ascii_case("solid") {
                !pending.cone_flags.solid()
            } else {
                input.eq_ignore_ascii_case("solid=yes")
            };
            pending.cone_flags.set_solid(solid);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.to_ascii_lowercase().starts_with("solid=") {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("Solid must be Yes or No")));
        }
        if input.eq_ignore_ascii_case("undo") {
            if pending.feature == "OM9-SOLID-017"
                && pending.values.len() == 1
                && pending.cone_flags.wall_thickness()
            {
                pending.cone_flags.set_wall_thickness(false);
            } else if pending.values.is_empty()
                && pending.cone_base_points == 6
                && pending.cone_flags.point_mode()
            {
                pending.cone_flags.set_point_mode(false);
            } else if pending.values.is_empty() && pending.cone_flags.radius_phase() == 2 {
                pending.cone_flags.set_radius_phase(1);
                pending
                    .cone_plane
                    .as_deref_mut()
                    .expect("cone frame")
                    .radius = 0.;
            } else if pending.values.is_empty() && pending.cone_flags.radius_phase() == 1 {
                pending.cone_flags.set_radius_phase(0);
            } else if pending.values.pop().is_none() {
                pending.points.pop();
                if pending.points.is_empty() {
                    pending.cone_plane = None;
                }
            } else if matches!(pending.cone_base_points, 2 | 3 | 4 | 6) && pending.values.is_empty()
            {
                pending.points.pop();
                if pending.cone_base_points == 4
                    && let Some(frame) = pending.cone_plane.as_deref_mut()
                    && let Some(original) = frame.fit_input_plane.take()
                {
                    frame.plane = original;
                }
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.eq_ignore_ascii_case("2point")
            || input.eq_ignore_ascii_case("mode=2point")
            || input.eq_ignore_ascii_case("3point")
            || input.eq_ignore_ascii_case("mode=3point")
            || input.eq_ignore_ascii_case("fitpoints")
            || input.eq_ignore_ascii_case("mode=fitpoints")
            || input.eq_ignore_ascii_case("aroundcurve")
            || input.eq_ignore_ascii_case("mode=aroundcurve")
            || input.eq_ignore_ascii_case("tangent")
            || input.eq_ignore_ascii_case("mode=tangent")
            || input.eq_ignore_ascii_case("center")
            || input.eq_ignore_ascii_case("mode=center")
        {
            if !pending.points.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "choose base mode before selecting points",
                )));
            }
            pending.cone_base_points = if input.to_ascii_lowercase().contains("2point") {
                2
            } else if input.to_ascii_lowercase().contains("3point") {
                3
            } else if input.to_ascii_lowercase().contains("fitpoints") {
                4
            } else if input.to_ascii_lowercase().contains("aroundcurve") {
                5
            } else if input.to_ascii_lowercase().contains("tangent") {
                6
            } else {
                0
            };
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.cone_base_points == 4 && pending.values.is_empty() && input.is_empty() {
            let result = self.solid_cone_finish_fit(&mut pending);
            self.solid_pending = Some(pending);
            return Some(result);
        }
        if pending.cone_base_points == 6
            && pending.points.len() == 1
            && pending.values.is_empty()
            && input.eq_ignore_ascii_case("point")
        {
            pending.cone_flags.set_point_mode(true);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if matches!(pending.cone_base_points, 5 | 6)
            && pending.points.is_empty()
            && input.to_ascii_lowercase().starts_with("oncurve=")
        {
            let tangent = pending.cone_base_points == 6;
            let value = input
                .split_once('=')
                .and_then(|(_, v)| v.parse::<f64>().ok());
            self.solid_pending = Some(pending);
            return Some(match value {
                Some(v) => {
                    if tangent {
                        self.solid_cone_tangent_contact(v)
                    } else {
                        self.solid_cone_around_center(v)
                    }
                }
                None => Err(Error::Invalid("OnCurve fraction")),
            });
        }
        if pending.cone_base_points == 3 && pending.points.len() == 2 && pending.values.is_empty() {
            let lowered = input.to_ascii_lowercase();
            if lowered == "radius" || lowered == "radius=" {
                pending.cone_flags.set_radius_phase(1);
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            if let Some(raw) = lowered.strip_prefix("radius=") {
                let value = raw.parse::<f64>().ok().filter(|v| v.is_finite());
                self.solid_pending = Some(pending);
                return Some(match value {
                    Some(v) => self.solid_cone_radius_value(v, true),
                    None => Err(Error::Invalid("base circle radius")),
                });
            }
            if pending.cone_flags.radius_phase() == 1 && !input.contains(',') {
                let value = input.parse::<f64>().ok().filter(|v| v.is_finite());
                self.solid_pending = Some(pending);
                return Some(match value {
                    Some(v) => self.solid_cone_radius_value(v, true),
                    None => Err(Error::Invalid("base circle radius")),
                });
            }
        }
        if input.eq_ignore_ascii_case("area") || input.to_ascii_lowercase().starts_with("area=") {
            if !matches!(pending.cone_base_points, 0 | 5)
                || pending.points.len() != 1
                || !pending.values.is_empty()
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "Area is available when setting the base size",
                )));
            }
            pending.cone_area = true;
            pending.cone_diameter = false;
            let value = input
                .split_once('=')
                .map(|(_, v)| v.parse::<f64>().ok().filter(|v| v.is_finite()));
            self.solid_pending = Some(pending);
            return Some(match value {
                None => Ok(()),
                Some(Some(v)) => self.solid_cone_accept_value(v, true),
                Some(None) => Err(Error::Invalid("base area")),
            });
        }
        if input.eq_ignore_ascii_case("diameter") || input.eq_ignore_ascii_case("radius") {
            pending.cone_diameter = input.eq_ignore_ascii_case("diameter");
            pending.cone_area = false;
            pending.cone_flags.set_wall_thickness(false);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if input.contains(',') {
            let plane = pending
                .cone_plane
                .as_deref()
                .map(|f| f.plane)
                .unwrap_or_else(|| self.input_plane());
            let base = pending
                .points
                .last()
                .copied()
                .and_then(|p| om9_core::coords::WorldPoint::try_new(p).ok());
            let result = om9_core::coords::parse_input(input, self.document().page_unit)
                .and_then(|v| om9_core::coords::resolve_with_base(v, Some(&plane), base))
                .map(|p| p.xyz());
            self.solid_pending = Some(pending);
            return Some(result.and_then(|p| self.solid_cone_accept_point(p)));
        }
        if pending.cone_base_points == 0
            && pending.points.is_empty()
            && input.split_whitespace().count()
                == if matches!(pending.feature, "OM9-SOLID-016" | "OM9-SOLID-019") {
                    2
                } else {
                    3
                }
        {
            if matches!(pending.feature, "OM9-SOLID-016" | "OM9-SOLID-019")
                && !pending.cone_flags.solid()
            {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing(
                    "pick center before creating an open surface",
                )));
            }
            if pending.feature == "OM9-SOLID-017" && pending.cone_flags.both_sides() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing(
                    "pick center before creating a BothSides tube",
                )));
            }
            let words: Vec<_> = input.split_whitespace().collect();
            let revision = self.document().revision;
            let result = self.dispatch_solid_args(pending.feature, &words);
            if result.is_err() && self.document().revision == revision {
                self.solid_pending = Some(pending);
            }
            return Some(result);
        }
        let value = input.parse::<f64>().ok().filter(|v| v.is_finite());
        self.solid_pending = Some(pending);
        return Some(match value {
            Some(v) => self.solid_cone_accept_value(v, true),
            None => Err(Error::Invalid("enter a point or dimension")),
        });
    }
}
