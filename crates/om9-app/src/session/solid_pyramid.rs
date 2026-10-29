// SPDX-License-Identifier: LGPL-2.1-or-later
//! Pyramid solid stages: base point, height input and typed text.
//!
//! Moved unchanged from `solid.rs`; shares `SolidPending` and the pyramid edge helper.
use super::solid::{ConeFrame, SolidPending, pyramid_edge_geometry};
use super::*;

impl Controller {
    pub(super) fn solid_pyramid_accept_point(&mut self, point: [f64; 3]) -> Result<()> {
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|p| p.pyramid_around && p.points.is_empty())
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
        if self.solid_pending.as_ref().is_some_and(|p| {
            matches!(p.cone_base_points, 1 | 2) && p.points.len() == 1 && p.values.is_empty()
        }) {
            let mut held = self.scene_cache_usage()?;
            let mut extra = om9_core::budget::Usage::array::<om9_core::geometry::Point3>(2)?;
            extra.points = 2;
            held.add(extra, self.coordinator.limits())?;
            self.coordinator.retained_usage_with_held_usage(held)?;
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|p| p.points.is_empty())
        {
            let mut held = self.scene_cache_usage()?;
            let mut extra = om9_core::budget::Usage::array::<om9_core::geometry::Point3>(1)?;
            extra.points = 1;
            extra.bytes = extra
                .bytes
                .checked_add(std::mem::size_of::<ConeFrame>() as u64)
                .ok_or(Error::Overflow("pyramid frame bytes"))?;
            held.add(extra, self.coordinator.limits())?;
            self.coordinator.retained_usage_with_held_usage(held)?;
        }
        let mut pending = self
            .solid_pending
            .take()
            .ok_or(Error::Missing("Pyramid session"))?;
        if !point.iter().all(|v| v.is_finite()) {
            self.solid_pending = Some(pending);
            return Err(Error::Invalid("pyramid point"));
        }
        if pending.points.is_empty() {
            let plane = match self.input_plane().with_origin(point) {
                Ok(v) => v,
                Err(e) => {
                    self.solid_pending = Some(pending);
                    return Err(e);
                }
            };
            if pending.points.try_reserve_exact(1).is_err() {
                self.solid_pending = Some(pending);
                return Err(Error::Budget("pyramid point"));
            }
            pending.points.push(point);
            pending.cone_plane = Some(Box::new(ConeFrame {
                plane,
                radius: 0.,
                fit_input_plane: None,
                around_source: None,
                tangent: None,
            }));
            self.solid_pending = Some(pending);
            return Ok(());
        }
        let plane = pending.cone_plane.as_deref().expect("pyramid frame").plane;
        if matches!(pending.cone_base_points, 1 | 2) && pending.values.is_empty() {
            let result = pyramid_edge_geometry(
                plane,
                pending.points[0],
                point,
                pending.pyramid_sides,
                pending.cone_base_points == 2,
            );
            let (center, second, next, radius) = match result {
                Ok(v) => v,
                Err(e) => {
                    self.solid_pending = Some(pending);
                    return Err(e);
                }
            };
            if pending.points.try_reserve_exact(2).is_err() {
                self.solid_pending = Some(pending);
                return Err(Error::Budget("pyramid edge points"));
            }
            let frame = pending.cone_plane.as_deref_mut().expect("pyramid frame");
            frame.fit_input_plane = Some(frame.plane);
            frame.plane = next;
            pending.points.push(second);
            pending.points.push(center);
            pending.values.push(radius);
            self.solid_pending = Some(pending);
            return Ok(());
        }
        let center = pending.pyramid_center();
        let d = std::array::from_fn::<_, 3, _>(|i| point[i] - center[i]);
        let dot = |axis: [f64; 3]| (0..3).map(|i| d[i] * axis[i]).sum::<f64>();
        if pending.cone_base_points == 3 && pending.values.len() == 1 {
            let radius = dot(plane.x).hypot(dot(plane.y));
            if !radius.is_finite() || radius <= 1e-9 {
                self.solid_pending = Some(pending);
                return Err(Error::Invalid("second star radius"));
            }
            pending.values.push(radius);
            self.solid_pending = Some(pending);
            return Ok(());
        }
        if pending.values.is_empty() {
            let u = dot(plane.x);
            let v = dot(plane.y);
            let radius = u.hypot(v);
            if !radius.is_finite() || radius <= 1e-9 {
                self.solid_pending = Some(pending);
                return Err(Error::Invalid("pyramid base radius"));
            }
            let x = std::array::from_fn(|i| (u * plane.x[i] + v * plane.y[i]) / radius);
            let n = plane.normal();
            let y = [
                n[1] * x[2] - n[2] * x[1],
                n[2] * x[0] - n[0] * x[2],
                n[0] * x[1] - n[1] * x[0],
            ];
            let next = match super::CPlane::new(center, x, y) {
                Ok(v) => v,
                Err(e) => {
                    self.solid_pending = Some(pending);
                    return Err(e);
                }
            };
            pending
                .cone_plane
                .as_deref_mut()
                .expect("pyramid frame")
                .plane = next;
            pending.values.push(radius);
            self.solid_pending = Some(pending);
            return Ok(());
        }
        let height = dot(plane.normal());
        self.solid_pending = Some(pending);
        self.solid_pyramid_height(height)
    }

    pub(super) fn solid_pyramid_height(&mut self, height: f64) -> Result<()> {
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
        let pending = self
            .solid_pending
            .take()
            .ok_or(Error::Missing("Pyramid session"))?;
        if !height.is_finite() || height.abs() <= 1e-9 {
            self.solid_pending = Some(pending);
            return Err(Error::Invalid("pyramid height"));
        }
        let sides = u64::from(pending.pyramid_sides);
        let vertices = sides * if pending.cone_base_points == 3 { 2 } else { 1 };
        if vertices > self.coordinator.limits().max_points
            || vertices * 512 > self.coordinator.limits().max_bytes
            || (pending.cone_base_points == 3 && vertices > 4096)
        {
            self.solid_pending = Some(pending);
            return Err(Error::Budget("pyramid side count"));
        }
        let mut plane = pending.cone_plane.as_deref().expect("pyramid frame").plane;
        let mut radius = pending.values[0];
        if pending.cone_base_points == 0 && pending.cone_flags.pyramid_circumscribed() {
            let angle = std::f64::consts::PI / pending.pyramid_sides as f64;
            let (s, c) = angle.sin_cos();
            let x = std::array::from_fn(|i| c * plane.x[i] - s * plane.y[i]);
            let y = std::array::from_fn(|i| s * plane.x[i] + c * plane.y[i]);
            plane = match super::CPlane::new(pending.points[0], x, y) {
                Ok(v) => v,
                Err(e) => {
                    self.solid_pending = Some(pending);
                    return Err(e);
                }
            };
            radius /= c;
        }
        if !radius.is_finite() {
            self.solid_pending = Some(pending);
            return Err(Error::Invalid("pyramid radius overflow"));
        }
        let result = self.solid_pyramid_commit(
            pending.pyramid_center(),
            plane,
            pending.pyramid_sides,
            radius,
            if pending.cone_base_points == 3 {
                Some(pending.values[1])
            } else {
                None
            },
            height,
            pending.cone_flags.solid(),
        );
        if result.is_err() {
            self.solid_pending = Some(pending);
        }
        result
    }

    pub(super) fn submit_pyramid_text(
        &mut self,
        mut pending: SolidPending,
        text: &str,
    ) -> Option<Result<()>> {
        let input = text.trim();
        let lower = input.to_ascii_lowercase();
        if lower == "cancel" || lower == "esc" {
            self.cancel();
            return Some(Ok(()));
        }
        if lower == "undo" {
            if pending.values.pop().is_some() {
                if matches!(pending.cone_base_points, 1 | 2) {
                    pending.points.truncate(1);
                    if let Some(frame) = pending.cone_plane.as_deref_mut() {
                        if let Some(original) = frame.fit_input_plane.take() {
                            frame.plane = original;
                        }
                    }
                }
            } else {
                pending.points.pop();
                if pending.points.is_empty() {
                    pending.cone_plane = None;
                }
            }
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if lower == "solid" || lower.starts_with("solid=") {
            let value = match lower.as_str() {
                "solid" => !pending.cone_flags.solid(),
                "solid=yes" => true,
                "solid=no" => false,
                _ => {
                    self.solid_pending = Some(pending);
                    return Some(Err(Error::Invalid("Solid expects Yes or No")));
                }
            };
            pending.cone_flags.set_solid(value);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if lower == "aroundcurve" || lower == "mode=aroundcurve" {
            if !pending.points.is_empty() || matches!(pending.cone_base_points, 1 | 2) {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid(
                    "choose AroundCurve before center; Edge is unsupported",
                )));
            }
            pending.pyramid_around = true;
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.pyramid_around && pending.points.is_empty() && lower.starts_with("oncurve=") {
            let value = lower
                .strip_prefix("oncurve=")
                .and_then(|v| v.parse::<f64>().ok());
            self.solid_pending = Some(pending);
            return Some(match value {
                Some(value) => self.solid_cone_around_center(value),
                None => Err(Error::Invalid("OnCurve fraction")),
            });
        }
        let mode = lower.strip_prefix("mode=").unwrap_or(&lower);
        if mode == "edge" || mode == "center" || mode == "star" || mode == "star5" {
            if !pending.points.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("choose base mode before first point")));
            }
            if mode == "edge" && pending.pyramid_around {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Unsupported("Edge with AroundCurve")));
            }
            if mode == "center" {
                pending.pyramid_around = false;
            }
            if mode == "star5" {
                pending.pyramid_sides = 5;
            }
            pending.cone_base_points = if mode == "edge" {
                1
            } else if mode == "star" || mode == "star5" {
                3
            } else {
                0
            };
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.cone_base_points == 3
            && pending.values.len() == 1
            && (lower == "automatic" || lower.is_empty())
        {
            pending.values.push(pending.values[0]);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if lower == "flip"
            && matches!(pending.cone_base_points, 1 | 2)
            && !pending.points.is_empty()
        {
            let flip = pending.cone_base_points != 2;
            if !pending.values.is_empty() {
                let original = pending
                    .cone_plane
                    .as_deref()
                    .and_then(|f| f.fit_input_plane)
                    .expect("edge input plane");
                let result = pyramid_edge_geometry(
                    original,
                    pending.points[0],
                    pending.points[1],
                    pending.pyramid_sides,
                    flip,
                );
                let (center, _, plane, radius) = match result {
                    Ok(v) => v,
                    Err(e) => {
                        self.solid_pending = Some(pending);
                        return Some(Err(e));
                    }
                };
                pending.points[2] = center;
                pending.values[0] = radius;
                pending.cone_plane.as_deref_mut().expect("edge frame").plane = plane;
            }
            pending.cone_base_points = if flip { 2 } else { 1 };
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if lower == "circumscribed" || lower == "inscribed" {
            if !pending.values.is_empty() || pending.cone_base_points != 0 {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("choose base style before radius")));
            }
            pending
                .cone_flags
                .set_pyramid_circumscribed(lower == "circumscribed");
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if let Some(raw) = lower.strip_prefix("numsides=") {
            if !pending.values.is_empty() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("choose sides before base radius")));
            }
            let sides = raw.parse::<u32>().ok().filter(|n| {
                let vertices = u64::from(*n) * if pending.cone_base_points == 3 { 2 } else { 1 };
                *n >= 3
                    && *n <= i32::MAX as u32
                    && vertices <= self.coordinator.limits().max_points
                    && vertices * 512 <= self.coordinator.limits().max_bytes
                    && (pending.cone_base_points != 3 || vertices <= 4096)
            });
            if let Some(sides) = sides {
                pending.pyramid_sides = sides;
                self.solid_pending = Some(pending);
                return Some(Ok(()));
            }
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("pyramid side count or budget")));
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
            return Some(result.and_then(|p| self.solid_pyramid_accept_point(p)));
        }
        if pending.points.is_empty() && input.split_whitespace().count() == 3 {
            if pending.pyramid_around {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("pick center on curve")));
            }
            if !pending.cone_flags.solid() {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing(
                    "pick center before creating an open surface",
                )));
            }
            if pending.cone_base_points != 0 {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("pick pyramid edge")));
            }
            let args: Vec<_> = input.split_whitespace().collect();
            let revision = self.document().revision;
            let result = self.dispatch_solid_args(pending.feature, &args);
            if result.is_err() && self.document().revision == revision {
                self.solid_pending = Some(pending);
            }
            return Some(result);
        }
        let value = input.parse::<f64>().ok().filter(|v| v.is_finite());
        let Some(value) = value else {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Invalid("enter pyramid point or dimension")));
        };
        if pending.points.is_empty() {
            self.solid_pending = Some(pending);
            return Some(Err(Error::Missing("pick pyramid center")));
        }
        let value = match self.document().page_unit.length_to_mm(value) {
            Ok(v) => v,
            Err(e) => {
                self.solid_pending = Some(pending);
                return Some(Err(e));
            }
        };
        if pending.values.is_empty() {
            if matches!(pending.cone_base_points, 1 | 2) {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Missing("pick second edge point")));
            }
            if value <= 1e-9 {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("pyramid base radius")));
            }
            pending.values.push(value);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        if pending.cone_base_points == 3 && pending.values.len() == 1 {
            if value <= 1e-9 {
                self.solid_pending = Some(pending);
                return Some(Err(Error::Invalid("second star radius")));
            }
            pending.values.push(value);
            self.solid_pending = Some(pending);
            return Some(Ok(()));
        }
        self.solid_pending = Some(pending);
        Some(self.solid_pyramid_height(value))
    }
}
