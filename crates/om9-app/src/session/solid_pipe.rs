// SPDX-License-Identifier: LGPL-2.1-or-later
//! Pipe command stages: curve/edge picking, station shapes, multiple-pipe and chain commits.
//!
//! Moved unchanged from `solid.rs`; the parent `impl Controller` and `solid_pipe.rs` share the
//! same `SolidPending` (fields are `pub(super)` inside the session module).
use super::solid::{PipeChainItem, SolidPending, label};
use super::*;
#[cfg(feature = "occt-brep")]
use om9_kernel::{
    creation::{self, Primitive},
    editing::{self, BooleanOp},
};

impl Controller {
    pub fn solid_pipe_needs_curve(&self) -> bool {
        self.solid_pending.as_ref().is_some_and(|p| {
            if p.feature == "SolidRibDevelopment" {
                return p.slab_source.is_none();
            }
            if p.feature == "OM9-SOLID-021" {
                return p.difference_targets.is_none();
            }
            if ((matches!(
                p.feature,
                "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
            ) && matches!(p.cone_base_points, 5 | 6))
                || (p.feature == "OM9-SOLID-018" && p.pyramid_around)
                || (p.feature == "OM9-SOLID-013" && p.ellipsoid_around)
                || (p.feature == "OM9-SOLID-015" && p.torus_around))
                && p.points.is_empty()
            {
                return !(self.selection.len() == 1
                    && self.selection.iter().next().is_some_and(|id| {
                        matches!(
                            self.document().entities.get(id).map(|e| &e.geometry),
                            Some(Geometry::Curve(_))
                        )
                    }));
            }
            if p.feature != "OM9-SOLID-011" {
                return false;
            }
            if p.pipe_multiple_selecting {
                return true;
            }
            !p.pipe_multiple
                && !(self.selection.len() == 1
                    && self.selection.iter().next().is_some_and(|id| {
                        matches!(
                            self.document().entities.get(id).map(|e| &e.geometry),
                            Some(Geometry::Curve(_))
                        )
                    }))
        })
    }

    pub fn solid_cube_needs_corner(&self) -> bool {
        self.solid_pending
            .as_ref()
            .is_some_and(|p| p.box_mode && p.box_kind == "Cube" && p.points.len() == 1)
    }

    pub(crate) fn solid_pipe_pick_context(&self) -> Option<u64> {
        self.solid_pipe_needs_curve().then_some(self.command_run)
    }

    pub fn solid_pipe_pick_curve(&mut self, id: om9_core::identity::ObjectId) -> Result<()> {
        if !self.solid_pipe_needs_curve() {
            return Err(Error::Missing("Pipe curve selection"));
        }
        if !matches!(
            self.document().entities.get(&id).map(|e| &e.geometry),
            Some(Geometry::Curve(_))
        ) {
            return Err(Error::Invalid("pipe needs a curve"));
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|p| matches!(p.feature, "OM9-SOLID-021" | "SolidRibDevelopment"))
        {
            return self.add_objects(&[id]);
        }
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|p| p.pipe_multiple_selecting)
        {
            if self.solid_pipe_chain_selecting() && !self.selection.contains(&id) {
                let source = self.document().stamp(id)?;
                let pending = self.solid_pending.as_mut().expect("Pipe session");
                if pending.pipe_chain_order.len() as u64 >= self.coordinator.limits().max_outputs {
                    return Err(Error::Budget("Pipe chain selection"));
                }
                pending
                    .pipe_chain_order
                    .try_reserve(1)
                    .map_err(|_| Error::Budget("Pipe chain selection"))?;
                self.add_objects(&[id])?;
                self.solid_pending
                    .as_mut()
                    .expect("Pipe session")
                    .pipe_chain_order
                    .push(PipeChainItem::Curve(source));
                Ok(())
            } else {
                self.add_objects(&[id])
            }
        } else {
            self.select_objects(&[id])
        }
    }

    pub fn solid_pipe_chain_selecting(&self) -> bool {
        self.solid_pending.as_ref().is_some_and(|p| {
            p.feature == "OM9-SOLID-011" && p.pipe_chain && p.pipe_multiple_selecting
        })
    }

    #[cfg(feature = "occt-brep")]
    pub fn solid_pipe_pick_edge(&mut self, reference: super::SelectedSubobject) -> Result<()> {
        if !self.solid_pipe_chain_selecting() {
            return Err(Error::Missing("Pipe edge chain session"));
        }
        if !reference.path.is_empty() {
            return Err(Error::Unsupported(
                "Pipe edge chains do not yet support instances",
            ));
        }
        let om9_scene::scene::Subelement::BrepEdge(key) = reference.subset else {
            return Err(Error::Invalid("select a B-Rep edge"));
        };
        let body = self.solid_brep(reference.root)?;
        let index = body
            .display
            .edges
            .iter()
            .position(|id| *id == key)
            .ok_or(Error::Stale("B-Rep edge"))?;
        let (a, b) = editing::straight_edge_endpoints(body, index, self.coordinator.limits())
            .map_err(|e| Error::Io(format!("Pipe edge: {e}")))?;
        let placement = self
            .document()
            .entities
            .get(&reference.root)
            .ok_or(Error::Stale("Pipe edge"))?
            .placement;
        let endpoints = [placement.point(a)?, placement.point(b)?];
        let pending = self.solid_pending.as_ref().expect("Pipe session");
        if pending.pipe_chain_order.iter().any(|item| matches!(item, PipeChainItem::Edge(old,_) if old.root == reference.root && old.subset == reference.subset)) { return Ok(()); }
        if pending.pipe_chain_order.len() as u64 >= self.coordinator.limits().max_outputs {
            return Err(Error::Budget("Pipe chain edges"));
        }
        self.solid_pending
            .as_mut()
            .expect("Pipe session")
            .pipe_chain_order
            .try_reserve(1)
            .map_err(|_| Error::Budget("Pipe chain edges"))?;
        self.select_subobject_ref(reference.clone())?;
        self.solid_pending
            .as_mut()
            .expect("Pipe session")
            .pipe_chain_order
            .push(PipeChainItem::Edge(reference, endpoints));
        Ok(())
    }

    #[cfg(not(feature = "occt-brep"))]
    pub fn solid_pipe_pick_edge(&mut self, _reference: super::SelectedSubobject) -> Result<()> {
        Err(Error::Unsupported("Pipe edge chains require OCCT"))
    }

    pub(super) fn solid_pipe_accept_radius_point(
        &mut self,
        point: om9_core::geometry::Point3,
    ) -> Result<()> {
        let pending = self
            .solid_pending
            .as_ref()
            .filter(|p| p.feature == "OM9-SOLID-011")
            .ok_or(Error::Missing("Pipe session"))?;
        let count = if pending.pipe_thick { 4 } else { 2 };
        let station_stage = pending.values.len() >= count;
        if station_stage && !pending.pipe_chain && !self.selection.is_empty() {
            let id = *self
                .selected_ordered()
                .first()
                .ok_or(Error::Missing("select one curve"))?;
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Stale("Pipe curve"))?;
            if let Geometry::Curve(curve) = &entity.geometry
                && !matches!(curve, om9_core::geometry::Curve::Line { .. })
            {
                if pending.pipe_thick || (!pending.pipe_global && pending.pipe_multiple) {
                    return Err(Error::Unsupported(
                        "curve stations require non-Thick Pipe; Local requires one curve",
                    ));
                }
                if let Some(fraction) = pending.pipe_station_position {
                    let center = entity.placement.point(om9_core::curve_ops::point_at(
                        curve,
                        fraction,
                        self.coordinator.limits(),
                    )?)?;
                    let tangent = om9_core::curve_ops::tangent_at(
                        curve,
                        fraction,
                        self.coordinator.limits(),
                    )?;
                    let a = entity.placement.0;
                    let axis = std::array::from_fn::<_, 3, _>(|i| {
                        (0..3).map(|j| a[i][j] * tangent[j]).sum::<f64>()
                    });
                    let length = axis[0].hypot(axis[1]).hypot(axis[2]);
                    if !length.is_finite() || length <= 1e-12 {
                        return Err(Error::Invalid("Pipe station tangent"));
                    }
                    let axis = axis.map(|v| v / length);
                    let delta = std::array::from_fn::<_, 3, _>(|i| point[i] - center[i]);
                    let axial = (0..3).map(|i| delta[i] * axis[i]).sum::<f64>();
                    let radial = std::array::from_fn::<_, 3, _>(|i| delta[i] - axial * axis[i]);
                    let radius = radial[0].hypot(radial[1]).hypot(radial[2]);
                    if !radius.is_finite() || radius <= 1e-9 {
                        return Err(Error::Invalid("Pipe point must be away from the rail axis"));
                    }
                    let pending = self.solid_pending.as_mut().expect("Pipe session");
                    if pending.pipe_stations.len() as u64 >= self.coordinator.limits().max_points {
                        return Err(Error::Budget("Pipe station count"));
                    }
                    pending
                        .pipe_stations
                        .try_reserve(1)
                        .map_err(|_| Error::Budget("Pipe stations"))?;
                    pending.pipe_stations.push((fraction, radius));
                    pending.pipe_station_position = None;
                    return Ok(());
                }
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
                        "curve station picking requires a similarity placement",
                    ));
                }
                let local = om9_core::top11::inverse_affine(entity.placement)?.point(point)?;
                let (fraction, _, _) = om9_core::curve_surface_ops::closest_point_on_curve(
                    curve,
                    local,
                    self.coordinator.limits(),
                )?;
                if !fraction.is_finite()
                    || fraction <= 1e-7
                    || fraction >= 1. - 1e-7
                    || pending
                        .pipe_stations
                        .iter()
                        .any(|(old, _)| (old - fraction).abs() <= 1e-7)
                {
                    return Err(Error::Invalid(
                        "intermediate position must be distinct and inside the rail",
                    ));
                }
                self.solid_pending
                    .as_mut()
                    .expect("Pipe session")
                    .pipe_station_position = Some(fraction);
                return Ok(());
            }
        }
        if !pending.pipe_chain && !station_stage && !self.selection.is_empty() {
            let id = *self
                .selected_ordered()
                .first()
                .ok_or(Error::Missing("select one curve"))?;
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Stale("Pipe curve"))?;
            if let Geometry::Curve(curve) = &entity.geometry
                && !matches!(curve, om9_core::geometry::Curve::Line { .. })
            {
                let fraction = if pending.values.len() >= if pending.pipe_thick { 2 } else { 1 } {
                    1.
                } else {
                    0.
                };
                let center = entity.placement.point(om9_core::curve_ops::point_at(
                    curve,
                    fraction,
                    self.coordinator.limits(),
                )?)?;
                let tangent =
                    om9_core::curve_ops::tangent_at(curve, fraction, self.coordinator.limits())?;
                let matrix = entity.placement.0;
                let direction = std::array::from_fn::<_, 3, _>(|i| {
                    (0..3).map(|j| matrix[i][j] * tangent[j]).sum::<f64>()
                });
                let length = direction[0].hypot(direction[1]).hypot(direction[2]);
                if !length.is_finite() || length <= 1e-12 {
                    return Err(Error::Invalid("Pipe endpoint tangent"));
                }
                let direction = direction.map(|v| v / length);
                let offset = std::array::from_fn::<_, 3, _>(|i| point[i] - center[i]);
                let axial = (0..3).map(|i| offset[i] * direction[i]).sum::<f64>();
                let radial = std::array::from_fn::<_, 3, _>(|i| offset[i] - axial * direction[i]);
                let radius = radial[0].hypot(radial[1]).hypot(radial[2]);
                if !radius.is_finite() || radius <= 1e-9 {
                    return Err(Error::Invalid("Pipe point must be away from the rail axis"));
                }
                self.solid_pending
                    .as_mut()
                    .expect("Pipe session")
                    .values
                    .push(radius);
                return Ok(());
            }
        }
        let [start, end] = if pending.pipe_chain && !station_stage {
            let points = self.solid_pipe_chain_points(pending)?;
            if pending.values.len() >= if pending.pipe_thick { 2 } else { 1 } {
                [points[points.len() - 2], points[points.len() - 1]]
            } else {
                [points[0], points[1]]
            }
        } else {
            self.solid_pipe_radius_endpoints(pending)?
        };
        let dir = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
        let length = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if !length.is_finite() || length <= 1.0e-9 {
            return Err(Error::Invalid("Pipe rail length"));
        }
        if pending.values.len() >= count && pending.pipe_station_position.is_none() {
            if pending.pipe_thick
                || (pending.pipe_round && pending.pipe_chain)
                || (!pending.pipe_global && (pending.pipe_multiple || pending.pipe_chain))
            {
                return Err(Error::Unsupported(
                    "intermediate stations require non-Thick Pipe; Round Chain and Local Multiple are unavailable",
                ));
            }
            let offset = [
                point[0] - start[0],
                point[1] - start[1],
                point[2] - start[2],
            ];
            let fraction =
                (offset[0] * dir[0] + offset[1] * dir[1] + offset[2] * dir[2]) / (length * length);
            if !fraction.is_finite()
                || fraction <= 0.0
                || fraction >= 1.0
                || pending
                    .pipe_stations
                    .iter()
                    .any(|(old, _)| (old - fraction).abs() * length <= 1.0e-6)
            {
                return Err(Error::Invalid(
                    "intermediate position must be distinct and inside the rail",
                ));
            }
            self.solid_pending
                .as_mut()
                .expect("Pipe session")
                .pipe_station_position = Some(fraction);
            return Ok(());
        }
        let center = if let Some(t) = pending.pipe_station_position {
            std::array::from_fn(|axis| start[axis] + dir[axis] * t)
        } else if pending.values.len() >= if pending.pipe_thick { 2 } else { 1 } {
            end
        } else {
            start
        };
        let offset = [
            point[0] - center[0],
            point[1] - center[1],
            point[2] - center[2],
        ];
        let axial = (offset[0] * dir[0] + offset[1] * dir[1] + offset[2] * dir[2]) / length;
        let radial = [
            offset[0] - axial * dir[0] / length,
            offset[1] - axial * dir[1] / length,
            offset[2] - axial * dir[2] / length,
        ];
        let radius = (radial[0] * radial[0] + radial[1] * radial[1] + radial[2] * radial[2]).sqrt();
        if !radius.is_finite() || radius <= 1.0e-9 {
            return Err(Error::Invalid("Pipe point must be away from the rail axis"));
        }
        let pending = self.solid_pending.as_mut().expect("Pipe session");
        if let Some(fraction) = pending.pipe_station_position {
            if pending.pipe_stations.len() as u64 >= self.coordinator.limits().max_points {
                return Err(Error::Budget("Pipe station count"));
            }
            pending
                .pipe_stations
                .try_reserve(1)
                .map_err(|_| Error::Budget("Pipe stations"))?;
            pending.pipe_stations.push((fraction, radius));
            pending.pipe_station_position = None;
        } else {
            pending.values.push(radius);
        }
        Ok(())
    }

    pub(super) fn solid_pipe_radius_endpoints(
        &self,
        pending: &SolidPending,
    ) -> Result<[om9_core::geometry::Point3; 2]> {
        if pending.pipe_chain {
            return self.solid_pipe_chain_endpoints(pending);
        }
        let id = *self
            .selected_ordered()
            .first()
            .ok_or(Error::Missing("select one curve"))?;
        let entity = &self.document().entities[&id];
        let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) = &entity.geometry
        else {
            return Err(Error::Unsupported(
                "Pipe radius by point currently requires a straight line",
            ));
        };
        Ok([
            entity.placement.point(*start)?,
            entity.placement.point(*end)?,
        ])
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_pipe_local(&mut self, pending: &SolidPending) -> Result<()> {
        if self.selection.len() != 1 {
            return Err(Error::Missing("select one curve"));
        }
        let id = *self.selection.iter().next().expect("one selected");
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Stale("Pipe curve"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("pipe needs a curve"));
        };
        let count = 64usize;
        if count as u64 > self.coordinator.limits().max_points {
            return Err(Error::Budget("Pipe samples"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::array::<om9_core::geometry::Point3>(count)?,
            self.coordinator.limits(),
        )?;
        held.add(
            om9_core::budget::Usage::array::<f64>(count * if pending.pipe_thick { 3 } else { 2 })?,
            self.coordinator.limits(),
        )?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let mut points =
            om9_core::surface_kernel::sample_curve(curve, count, self.coordinator.limits())?;
        for point in &mut points {
            *point = entity.placement.point(*point)?;
        }
        let distance =
            |a: [f64; 3], b: [f64; 3]| (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt();
        if distance(points[0], points[count - 1]) <= 1e-9 {
            return Err(Error::Unsupported(
                "varying Local Pipe on a closed curve is not supported",
            ));
        }
        let mut lengths = Vec::new();
        lengths
            .try_reserve_exact(count)
            .map_err(|_| Error::Budget("Pipe samples"))?;
        lengths.push(0.);
        for pair in points.windows(2) {
            lengths.push(lengths.last().copied().unwrap() + distance(pair[0], pair[1]));
        }
        let total = *lengths.last().expect("Pipe samples");
        if !total.is_finite() || total <= 1e-9 {
            return Err(Error::Invalid("Pipe path length"));
        }
        let ends = if pending.pipe_thick {
            let v = &pending.values;
            let inner = [v[0].min(v[1]), v[2].min(v[3])];
            let outer = [v[0].max(v[1]), v[2].max(v[3])];
            if outer[0] <= inner[0]
                || outer[1] <= inner[1]
                || (!pending.pipe_capped
                    && (outer[0] - inner[0] <= 1e-7 || outer[1] - inner[1] <= 1e-7))
            {
                return Err(Error::Invalid("Thick Pipe radii must differ at each end"));
            }
            (outer, Some(inner))
        } else {
            ([pending.values[0], pending.values[1]], None)
        };
        let radii = |ends: [f64; 2]| -> Result<Vec<f64>> {
            let mut values = Vec::new();
            values
                .try_reserve_exact(count)
                .map_err(|_| Error::Budget("Pipe samples"))?;
            for &length in &lengths {
                let t = length / total;
                let blend = t * t * (3. - 2. * t);
                values.push(ends[0] + blend * (ends[1] - ends[0]));
            }
            Ok(values)
        };
        let mut outer = creation::pipe_sampled_sections(
            &points,
            &radii(ends.0)?,
            pending.pipe_capped,
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|e| Error::Io(format!("Pipe Local: {e}")))?;
        if pending.pipe_round {
            outer = self.solid_pipe_local_round_caps(outer, &points, ends.0, held)?;
        }
        let body = if let Some(inner_ends) = ends.1 {
            held.add(
                outer.validate(self.coordinator.limits())?,
                self.coordinator.limits(),
            )?;
            self.coordinator.retained_usage_with_held_usage(held)?;
            let mut inner = creation::pipe_sampled_sections(
                &points,
                &radii(inner_ends)?,
                pending.pipe_capped,
                creation::DEFAULT_DEFLECTION_MM,
                self.coordinator.limits(),
            )
            .map_err(|e| Error::Io(format!("Pipe Local inner: {e}")))?;
            if pending.pipe_round {
                inner = self.solid_pipe_local_round_caps(inner, &points, inner_ends, held)?;
            }
            held.add(
                inner.validate(self.coordinator.limits())?,
                self.coordinator.limits(),
            )?;
            self.coordinator.retained_usage_with_held_usage(held)?;
            if pending.pipe_capped {
                editing::boolean(
                    &outer,
                    &inner,
                    BooleanOp::Difference,
                    creation::DEFAULT_DEFLECTION_MM,
                    self.coordinator.limits(),
                )
                .map_err(|e| Error::Io(format!("Pipe Local hollow: {e}")))?
                .body()
                .ok_or(Error::Invalid("Thick Pipe produced an empty body"))?
                .clone()
            } else {
                editing::sew_reversing_second(
                    &outer,
                    &inner,
                    1e-7,
                    creation::DEFAULT_DEFLECTION_MM,
                    self.coordinator.limits(),
                )
                .map_err(|e| Error::Io(format!("Pipe Local open walls: {e}")))?
            }
        } else {
            outer
        };
        self.insert_solid(pending.feature, label(pending.feature), body)
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_pipe_local_round_caps(
        &self,
        mut body: om9_core::brep::Brep,
        points: &[om9_core::geometry::Point3],
        radii: [f64; 2],
        mut held: om9_core::budget::Usage,
    ) -> Result<om9_core::brep::Brep> {
        let start = points[0];
        let end = points[points.len() - 1];
        let start_direction = std::array::from_fn(|i| points[1][i] - start[i]);
        let end_direction = std::array::from_fn(|i| end[i] - points[points.len() - 2][i]);
        for (center, direction, radius, forward) in [
            (start, start_direction, radii[0], false),
            (end, end_direction, radii[1], true),
        ] {
            held.add(
                body.validate(self.coordinator.limits())?,
                self.coordinator.limits(),
            )?;
            let cap = creation::hemisphere(
                center,
                direction,
                radius,
                forward,
                creation::DEFAULT_DEFLECTION_MM,
                self.coordinator.limits(),
            )
            .map_err(|e| Error::Io(format!("Pipe Local cap: {e}")))?;
            held.add(
                cap.validate(self.coordinator.limits())?,
                self.coordinator.limits(),
            )?;
            self.coordinator.retained_usage_with_held_usage(held)?;
            body = editing::boolean(
                &body,
                &cap,
                BooleanOp::Union,
                creation::DEFAULT_DEFLECTION_MM,
                self.coordinator.limits(),
            )
            .map_err(|e| Error::Io(format!("Pipe Local cap union: {e}")))?
            .body()
            .ok_or(Error::Invalid("Pipe Local cap union is empty"))?
            .clone();
        }
        Ok(body)
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn solid_pipe_local(&mut self, _pending: &SolidPending) -> Result<()> {
        Err(Error::Unsupported("Local Pipe requires OCCT"))
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_pipe_curve_station_shape(
        &self,
        entity: &Entity,
        stations: &[(f64, f64)],
        capped: bool,
        smooth: bool,
        round: bool,
        mut held: om9_core::budget::Usage,
    ) -> Result<om9_core::brep::Brep> {
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("pipe needs a curve"));
        };
        let count = 64usize
            .checked_add(stations.len())
            .ok_or(Error::Overflow("Pipe samples"))?;
        if count as u64 > self.coordinator.limits().max_points {
            return Err(Error::Budget("Pipe samples"));
        }
        for usage in [
            om9_core::budget::Usage::array::<f64>(count)?,
            om9_core::budget::Usage::array::<om9_core::geometry::Point3>(count)?,
            om9_core::budget::Usage::array::<f64>(count)?,
            om9_core::budget::Usage::array::<f64>(count)?,
            om9_core::budget::Usage::array::<usize>(stations.len())?,
        ] {
            held.add(usage, self.coordinator.limits())?;
        }
        self.coordinator.retained_usage_with_held_usage(held)?;
        let mut fractions = Vec::new();
        fractions
            .try_reserve_exact(count)
            .map_err(|_| Error::Budget("Pipe samples"))?;
        fractions.extend(
            (0..64)
                .map(|i| i as f64 / 63.)
                .filter(|t| !stations.iter().any(|s| (s.0 - *t).abs() < 1e-10)),
        );
        fractions.extend(stations.iter().map(|s| s.0));
        fractions.sort_by(f64::total_cmp);
        let mut points = Vec::new();
        points
            .try_reserve_exact(fractions.len())
            .map_err(|_| Error::Budget("Pipe samples"))?;
        for &t in &fractions {
            points.push(entity.placement.point(om9_core::curve_ops::point_at(
                curve,
                t,
                self.coordinator.limits(),
            )?)?);
        }
        let distance =
            |a: [f64; 3], b: [f64; 3]| (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt();
        if distance(points[0], points[points.len() - 1]) <= 1e-9 {
            return Err(Error::Unsupported(
                "Pipe stations on a closed curve are not supported",
            ));
        }
        let mut lengths = Vec::new();
        lengths
            .try_reserve_exact(points.len())
            .map_err(|_| Error::Budget("Pipe sample lengths"))?;
        lengths.push(0.);
        for pair in points.windows(2) {
            let next = lengths.last().copied().unwrap() + distance(pair[0], pair[1]);
            lengths.push(next);
        }
        let mut indexes = Vec::new();
        indexes
            .try_reserve_exact(stations.len())
            .map_err(|_| Error::Budget("Pipe station indexes"))?;
        for (t, _) in stations {
            indexes.push(
                fractions
                    .binary_search_by(|v| v.total_cmp(t))
                    .map_err(|_| Error::Invalid("Pipe station sample"))?,
            );
        }
        let mut radii = Vec::new();
        radii
            .try_reserve_exact(points.len())
            .map_err(|_| Error::Budget("Pipe sample radii"))?;
        let mut section = 0usize;
        for i in 0..points.len() {
            while section + 1 < indexes.len() - 1 && i > indexes[section + 1] {
                section += 1;
            }
            let lo = indexes[section];
            let hi = indexes[section + 1];
            let span = lengths[hi] - lengths[lo];
            if !span.is_finite() || span <= 1e-12 {
                return Err(Error::Invalid("Pipe station span"));
            }
            let u = (lengths[i] - lengths[lo]) / span;
            let blend = if smooth { u * u * (3. - 2. * u) } else { u };
            radii.push(
                stations[section].1 + blend * (stations[section + 1].1 - stations[section].1),
            );
        }
        let mut body = creation::pipe_sampled_sections(
            &points,
            &radii,
            capped,
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|e| Error::Io(format!("Pipe stations: {e}")))?;
        if round {
            body = self.solid_pipe_local_round_caps(
                body,
                &points,
                [stations[0].1, stations[stations.len() - 1].1],
                held,
            )?;
        }
        Ok(body)
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_pipe_commit_stations(&mut self, pending: &SolidPending) -> Result<()> {
        let mut stations = Vec::new();
        stations
            .try_reserve_exact(pending.pipe_stations.len() + 2)
            .map_err(|_| Error::Budget("Pipe stations"))?;
        stations.push((0.0, pending.values[0]));
        stations.extend_from_slice(&pending.pipe_stations);
        stations.push((1.0, pending.values[1]));
        stations.sort_by(|a, b| a.0.total_cmp(&b.0));
        if pending.pipe_multiple && !pending.pipe_chain {
            let ids = self.selected_ordered();
            if ids.is_empty() {
                return Err(Error::Missing("select curves"));
            }
            if ids.len() as u64 > self.coordinator.limits().max_outputs {
                return Err(Error::Budget("Pipe output count"));
            }
            let mut held = self.scene_cache_usage()?;
            let mut mutations = Vec::new();
            mutations
                .try_reserve_exact(ids.len())
                .map_err(|_| Error::Budget("Pipe outputs"))?;
            for id in ids {
                let entity = self
                    .document()
                    .entities
                    .get(&id)
                    .ok_or(Error::Stale("Pipe curve"))?;
                let shape =
                    if let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) =
                        &entity.geometry
                        && !pending.pipe_round
                    {
                        let endpoints = [
                            entity.placement.point(*start)?,
                            entity.placement.point(*end)?,
                        ];
                        creation::pipe_stations_straight(
                            endpoints,
                            &stations,
                            pending.pipe_capped,
                            creation::DEFAULT_DEFLECTION_MM,
                            self.coordinator.limits(),
                        )
                        .map_err(|e| Error::Io(format!("Pipe stations: {e}")))?
                    } else {
                        self.solid_pipe_curve_station_shape(
                            entity,
                            &stations,
                            pending.pipe_capped,
                            false,
                            pending.pipe_round,
                            held,
                        )?
                    };
                held.add(
                    shape.validate(self.coordinator.limits())?,
                    self.coordinator.limits(),
                )?;
                self.coordinator.retained_usage_with_held_usage(held)?;
                let result = Entity::new(
                    label(pending.feature),
                    Geometry::Brep(shape),
                    self.document().current_layer,
                );
                mutations.push(Mutation::Insert {
                    document: self.document,
                    entity: result,
                });
            }
            return self.commit_solid_result(
                pending.feature,
                label(pending.feature),
                mutations,
                None,
            );
        }
        if !pending.pipe_chain && !pending.pipe_multiple && self.selection.len() == 1 {
            let id = *self.selection.iter().next().expect("one selected");
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Stale("Pipe curve"))?;
            if let Geometry::Curve(curve) = &entity.geometry
                && (pending.pipe_round
                    || !pending.pipe_global
                    || !matches!(curve, om9_core::geometry::Curve::Line { .. }))
            {
                let body = self.solid_pipe_curve_station_shape(
                    entity,
                    &stations,
                    pending.pipe_capped,
                    !pending.pipe_global,
                    pending.pipe_round,
                    self.scene_cache_usage()?,
                )?;
                return self.insert_solid(pending.feature, label(pending.feature), body);
            }
        }
        let endpoints = self.solid_pipe_radius_endpoints(pending)?;
        let brep = creation::pipe_stations_straight(
            endpoints,
            &stations,
            pending.pipe_capped,
            creation::DEFAULT_DEFLECTION_MM,
            self.coordinator.limits(),
        )
        .map_err(|error| Error::Io(format!("Pipe stations: {error}")))?;
        self.insert_solid(pending.feature, label(pending.feature), brep)
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn solid_pipe_commit_stations(&mut self, _pending: &SolidPending) -> Result<()> {
        Err(Error::Unsupported("Pipe stations require OCCT"))
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_pipe_commit_multiple(&mut self, pending: &SolidPending) -> Result<()> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("select curves"));
        }
        if ids.len() as u64 > self.coordinator.limits().max_outputs {
            return Err(Error::Budget("Pipe output count"));
        }
        let radii =
            if !pending.pipe_thick && !pending.pipe_round && pending.values[0] == pending.values[1]
            {
                &pending.values[..1]
            } else {
                &pending.values[..]
            };
        let mut held = self.scene_cache_usage()?;
        let mut mutations = Vec::new();
        mutations
            .try_reserve_exact(ids.len())
            .map_err(|_| Error::Budget("Pipe outputs"))?;
        for id in ids {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Stale("Pipe curve"))?;
            let shape = self.solid_pipe_shape_entity(
                entity,
                radii,
                pending.pipe_capped,
                pending.pipe_round,
                held,
            )?;
            held.add(
                shape.validate(self.coordinator.limits())?,
                self.coordinator.limits(),
            )?;
            self.coordinator.retained_usage_with_held_usage(held)?;
            let entity = Entity::new(
                label(pending.feature),
                Geometry::Brep(shape),
                self.document().current_layer,
            );
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        self.commit_solid_result(pending.feature, label(pending.feature), mutations, None)
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn solid_pipe_commit_multiple(&mut self, _pending: &SolidPending) -> Result<()> {
        Err(Error::Unsupported("Multiple Pipe requires OCCT"))
    }

    pub(super) fn solid_pipe_existing_chain_selection(&self) -> Result<Vec<PipeChainItem>> {
        let ids = self.selected_ordered();
        let mut items = Vec::new();
        items
            .try_reserve_exact(ids.len())
            .map_err(|_| Error::Budget("Pipe chain selection"))?;
        for id in ids {
            items.push(PipeChainItem::Curve(self.document().stamp(id)?));
        }
        Ok(items)
    }

    pub(super) fn solid_pipe_chain_points(
        &self,
        pending: &SolidPending,
    ) -> Result<Vec<om9_core::geometry::Point3>> {
        if pending.pipe_chain_order.is_empty() {
            return Err(Error::Missing(
                "select connected curve segments or B-Rep edges",
            ));
        }
        let mut segments = Vec::new();
        segments
            .try_reserve_exact(pending.pipe_chain_order.len())
            .map_err(|_| Error::Budget("Pipe chain"))?;
        for item in &pending.pipe_chain_order {
            match item {
                PipeChainItem::Curve(source) => {
                    self.document().resolve(source)?;
                    if !self.selection.contains(&source.object) {
                        return Err(Error::Stale("Pipe chain curve selection"));
                    }
                    let entity = &self.document().entities[&source.object];
                    let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) =
                        &entity.geometry
                    else {
                        return Err(Error::Unsupported(
                            "Pipe ChainEdges currently accepts straight curve segments",
                        ));
                    };
                    segments.push([
                        entity.placement.point(*start)?,
                        entity.placement.point(*end)?,
                    ]);
                }
                PipeChainItem::Edge(edge, endpoints) => {
                    #[cfg(feature = "occt-brep")]
                    {
                        if edge.world_epoch != self.coordinator.epoch()
                            || !self.subselection().contains(edge)
                        {
                            return Err(Error::Stale("Pipe chain edge"));
                        }
                        self.document().resolve(&edge.source)?;
                        segments.push(*endpoints);
                    }
                    #[cfg(not(feature = "occt-brep"))]
                    {
                        let _ = (edge, endpoints);
                        return Err(Error::Unsupported("Pipe edges require OCCT"));
                    }
                }
            }
        }
        let distance =
            |a: [f64; 3], b: [f64; 3]| ((0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>()).sqrt();
        let tol = 1.0e-6;
        let [mut start, mut end] = segments[0];
        if segments.len() > 1
            && distance(end, segments[1][0]).min(distance(end, segments[1][1])) > tol
        {
            std::mem::swap(&mut start, &mut end);
        }
        let length = distance(start, end);
        if !length.is_finite() || length <= tol {
            return Err(Error::Invalid("Pipe chain segment is too short"));
        }
        let mut points = vec![start, end];
        points
            .try_reserve(segments.len().saturating_sub(1))
            .map_err(|_| Error::Budget("Pipe chain points"))?;
        for [a, b] in segments.into_iter().skip(1) {
            let next = if distance(end, a) <= tol {
                b
            } else if distance(end, b) <= tol {
                a
            } else {
                return Err(Error::Invalid(
                    "Pipe chain segments do not meet in selection order",
                ));
            };
            if !distance(end, next).is_finite() || distance(end, next) <= tol {
                return Err(Error::Invalid("Pipe chain segment is too short"));
            }
            end = next;
            points.push(next);
        }
        for triple in points.windows(3) {
            let a = [
                triple[1][0] - triple[0][0],
                triple[1][1] - triple[0][1],
                triple[1][2] - triple[0][2],
            ];
            let b = [
                triple[2][0] - triple[1][0],
                triple[2][1] - triple[1][1],
                triple[2][2] - triple[1][2],
            ];
            let lengths = distance(triple[0], triple[1]) * distance(triple[1], triple[2]);
            let cosine = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / lengths;
            if !cosine.is_finite() || cosine <= -1.0 + 1.0e-9 {
                return Err(Error::Invalid("Pipe chain reverses direction"));
            }
        }
        Ok(points)
    }

    pub(super) fn solid_pipe_chain_endpoints(
        &self,
        pending: &SolidPending,
    ) -> Result<[om9_core::geometry::Point3; 2]> {
        let points = self.solid_pipe_chain_points(pending)?;
        let start = points[0];
        let end = *points.last().expect("chain points");
        let dir = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
        let length = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if !length.is_finite() || length <= 1.0e-6 {
            return Err(Error::Invalid("Pipe chain length"));
        }
        let direction = dir.map(|x| x / length);
        for pair in points.windows(2) {
            let step = [
                pair[1][0] - pair[0][0],
                pair[1][1] - pair[0][1],
                pair[1][2] - pair[0][2],
            ];
            let axial = step[0] * direction[0] + step[1] * direction[1] + step[2] * direction[2];
            let radial = [
                step[0] - axial * direction[0],
                step[1] - axial * direction[1],
                step[2] - axial * direction[2],
            ];
            if axial <= 1.0e-6
                || (radial[0] * radial[0] + radial[1] * radial[1] + radial[2] * radial[2]).sqrt()
                    > 1.0e-6
            {
                return Err(Error::Unsupported(
                    "Pipe point input currently requires a collinear chain",
                ));
            }
        }
        Ok([start, end])
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_pipe_commit_chain(&mut self, pending: &SolidPending) -> Result<()> {
        let points = self.solid_pipe_chain_points(pending)?;
        let [start, end] = match self.solid_pipe_chain_endpoints(pending) {
            Ok(endpoints) => endpoints,
            Err(Error::Unsupported(_)) => {
                let limits = self.coordinator.limits();
                let deflection = creation::DEFAULT_DEFLECTION_MM;
                if pending.pipe_thick {
                    let start = [
                        pending.values[0].min(pending.values[1]),
                        pending.values[0].max(pending.values[1]),
                    ];
                    let end = [
                        pending.values[2].min(pending.values[3]),
                        pending.values[2].max(pending.values[3]),
                    ];
                    if start != end {
                        return Err(Error::Unsupported(
                            "bent Thick Pipe currently requires constant inner and outer radii",
                        ));
                    }
                    if start[1] - start[0] <= 1.0e-7 {
                        return Err(Error::Invalid("Thick Pipe radii are too close"));
                    }
                    let outside = self.solid_pipe_miter_shape(
                        &points,
                        start[1],
                        pending.pipe_capped,
                        pending.pipe_round,
                    )?;
                    let inside = self.solid_pipe_miter_shape(
                        &points,
                        start[0],
                        pending.pipe_capped,
                        pending.pipe_round,
                    )?;
                    let body = if pending.pipe_capped {
                        editing::boolean(
                            &outside,
                            &inside,
                            BooleanOp::Difference,
                            deflection,
                            limits,
                        )
                        .map_err(|error| Error::Io(format!("Pipe chain hollow: {error}")))?
                        .body()
                        .ok_or(Error::Invalid("Thick Pipe produced an empty body"))?
                        .clone()
                    } else {
                        editing::sew_reversing_second(&outside, &inside, 1.0e-7, deflection, limits)
                            .map_err(|error| Error::Io(format!("Pipe chain open walls: {error}")))?
                    };
                    return self.insert_solid(pending.feature, label(pending.feature), body);
                }
                if pending.values[0] != pending.values[1] {
                    return Err(Error::Unsupported(
                        "bent Pipe currently requires constant radius",
                    ));
                }
                let brep = self.solid_pipe_miter_shape(
                    &points,
                    pending.values[0],
                    pending.pipe_capped,
                    pending.pipe_round,
                )?;
                return self.insert_solid(pending.feature, label(pending.feature), brep);
            }
            Err(error) => return Err(error),
        };
        let entity = Entity::new(
            "Pipe chain rail",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0.0, 1.0],
            }),
            None,
        );
        let radii =
            if !pending.pipe_thick && !pending.pipe_round && pending.values[0] == pending.values[1]
            {
                &pending.values[..1]
            } else {
                &pending.values[..]
            };
        let brep = self.solid_pipe_shape_entity(
            &entity,
            radii,
            pending.pipe_capped,
            pending.pipe_round,
            self.scene_cache_usage()?,
        )?;
        self.insert_solid(pending.feature, label(pending.feature), brep)
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn solid_pipe_miter_shape(
        &self,
        points: &[om9_core::geometry::Point3],
        radius: f64,
        capped: bool,
        round: bool,
    ) -> Result<om9_core::brep::Brep> {
        let limits = self.coordinator.limits();
        let deflection = creation::DEFAULT_DEFLECTION_MM;
        if round {
            let length = |a: [f64; 3], b: [f64; 3]| {
                ((0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>()).sqrt()
            };
            if points.len() < 2
                || length(points[0], points[1]) < radius
                || length(points[points.len() - 2], points[points.len() - 1]) < radius
            {
                return Err(Error::Unsupported(
                    "Round caps require endpoint segments at least as long as the radius",
                ));
            }
        }
        let mut body = creation::pipe_miter_polyline(points, radius, capped, deflection, limits)
            .map_err(|error| Error::Io(format!("Pipe chain: {error}")))?;
        if round {
            let sphere =
                creation::primitive(Primitive::Sphere { radius_mm: radius }, deflection, limits)
                    .map_err(|error| Error::Io(format!("Pipe round cap: {error}")))?;
            for point in [points[0], points[points.len() - 1]] {
                let cap = om9_kernel::transform_ops::transform_brep(
                    &sphere,
                    &om9_core::geometry::Transform::translation(point),
                    deflection,
                    limits,
                )
                .map_err(|error| Error::Io(format!("Pipe cap placement: {error}")))?;
                body = editing::boolean(&body, &cap, BooleanOp::Union, deflection, limits)
                    .map_err(|error| Error::Io(format!("Pipe cap union: {error}")))?
                    .body()
                    .ok_or(Error::Invalid("Pipe cap union was empty"))?
                    .clone();
            }
        }
        Ok(body)
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn solid_pipe_commit_chain(&mut self, _pending: &SolidPending) -> Result<()> {
        Err(Error::Unsupported("Pipe ChainEdges requires OCCT"))
    }
}
