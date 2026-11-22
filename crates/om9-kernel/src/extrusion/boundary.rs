// SPDX-License-Identifier: LGPL-2.1-or-later
//! Extrusion against a single trimmed face. All picks, section curves and
//! distances are in document coordinates. The original profile retains its placement.
use super::{side_wall, solid_to_face};
use crate::{brep, editing, intersect, measurement, transform_ops};
use om9_core::{
    Error, Result,
    brep::{Brep, CanonicalBrep},
    budget::Limits,
    curve_ops,
    geometry::{Curve, Geometry, Point3, Transform},
};

fn kernel(error: impl std::fmt::Display) -> Error {
    Error::Io(format!("Extrude Lines: {error}"))
}
fn dot(a: Point3, b: Point3) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn delta(a: Point3, b: Point3) -> Point3 {
    std::array::from_fn(|i| a[i] - b[i])
}
fn norm(a: Point3) -> f64 {
    a[0].hypot(a[1]).hypot(a[2])
}
fn shifted(p: Point3, axis: Point3, t: f64) -> Point3 {
    std::array::from_fn(|i| p[i] + t * axis[i])
}

pub struct BoundaryProjection {
    face: Geometry,
    axis: Point3,
    bounds: [f64; 2],
    tolerance: f64,
    deflection: f64,
}

impl BoundaryProjection {
    /// Choose the side of the profile by the nearest face hit along the axis
    /// through the first selected profile's start.
    pub fn new(
        face: CanonicalBrep,
        axis: Point3,
        start: Point3,
        tolerance: f64,
        deflection: f64,
        limits: &Limits,
    ) -> Result<Self> {
        if face.topology().faces.len() != 1 || !tolerance.is_finite() || tolerance <= 0. {
            return Err(Error::Invalid("Select one boundary surface"));
        }
        let length = norm(axis);
        if !length.is_finite() || length == 0. {
            return Err(Error::Invalid("extrusion axis"));
        }
        let axis = axis.map(|v| v / length);
        let bbox = brep::world_bounds(&face, &Transform::identity(), limits).map_err(kernel)?;
        let mut bounds = [0., 0.];
        for i in 0..3 {
            let a = bbox[i] * axis[i];
            let b = bbox[i + 3] * axis[i];
            bounds[0] += a.min(b);
            bounds[1] += a.max(b);
        }
        let pad = (bounds[1] - bounds[0]).max(tolerance * 16.).max(1.);
        bounds[0] -= pad;
        bounds[1] += pad;
        let mut projection = Self {
            face: Geometry::Brep(face),
            axis,
            bounds,
            tolerance,
            deflection,
        };
        let d = projection
            .hits(start, limits)?
            .into_iter()
            .min_by(|a, b| a.abs().total_cmp(&b.abs()))
            .ok_or(Error::Invalid("Extrusion axis misses the trimmed boundary"))?;
        if d.abs() <= tolerance {
            return Err(Error::Invalid("Profile starts on the boundary"));
        }
        if d < 0. {
            projection.axis = axis.map(|v| -v);
            projection.bounds = [-bounds[1], -bounds[0]];
        }
        Ok(projection)
    }

    fn face(&self) -> &CanonicalBrep {
        let Geometry::Brep(face) = &self.face else {
            unreachable!()
        };
        face
    }

    fn hits(&self, p: Point3, limits: &Limits) -> Result<Vec<f64>> {
        let coord = dot(p, self.axis);
        let [a, b] = self.bounds.map(|v| v - coord);
        let probe = Curve::Line {
            start: shifted(p, self.axis, a),
            end: shifted(p, self.axis, b),
            domain: [0., 1.],
        };
        let hits = intersect::curve_brep_fractions_including_endpoints(
            self.face(),
            &probe,
            self.tolerance,
            limits,
        )
        .map_err(kernel)?;
        Ok(hits.into_iter().map(|t| a + (b - a) * t).collect())
    }

    pub fn project_station(&self, p: Point3, limits: &Limits) -> Result<Point3> {
        let distance = self
            .hits(p, limits)?
            .into_iter()
            .filter(|d| *d > self.tolerance)
            .min_by(f64::total_cmp)
            .ok_or(Error::Invalid(
                "Profile misses the boundary in the extrusion direction",
            ))?;
        Ok(shifted(p, self.axis, distance))
    }

    fn local_offset(&self, curve: &Curve, world: Transform, limits: &Limits) -> Result<Point3> {
        // Positive rational weights contain the curve in its control hull. A
        // circle can use its exact support along the transformed direction.
        fn minimum(curve: &Curve, world: Transform, axis: Point3) -> Result<f64> {
            let height = |p| -> Result<f64> { Ok(dot(world.point(p)?, axis)) };
            match curve {
                Curve::Line { start, end, .. } => Ok(height(*start)?.min(height(*end)?)),
                Curve::Circle {
                    center,
                    x,
                    y,
                    radius,
                    ..
                } => {
                    let c = height(*center)?;
                    let extent = |v: Point3| -> Result<f64> {
                        Ok(dot(
                            std::array::from_fn(|i| (0..3).map(|j| world.0[i][j] * v[j]).sum()),
                            axis,
                        ) * radius)
                    };
                    Ok(c - extent(*x)?.hypot(extent(*y)?))
                }
                Curve::Nurbs(n) => n
                    .poles
                    .iter()
                    .try_fold(f64::INFINITY, |m, p| Ok(m.min(height(*p)?))),
                Curve::Polycurve { segments, .. } => segments
                    .iter()
                    .try_fold(f64::INFINITY, |m, c| Ok(m.min(minimum(c, world, axis)?))),
            }
        }
        curve.validate(limits)?;
        let length = self.bounds[1] - minimum(curve, world, self.axis)?;
        if !length.is_finite() || length <= 0. {
            return Err(Error::Invalid("Boundary is behind the profile"));
        }
        let inverse = om9_core::top11::inverse_affine(world)?;
        Ok(std::array::from_fn(|i| {
            (0..3)
                .map(|j| inverse.0[i][j] * self.axis[j] * length)
                .sum()
        }))
    }

    /// Draft in document coordinates until the tool completely separates the base from the far end.
    pub fn tapered(
        &self,
        curve: &Curve,
        requested_axis: Point3,
        angle: f64,
        corners: super::DraftCorners,
        solid: bool,
        limits: &Limits,
    ) -> Result<Brep> {
        let offset = self.local_offset(curve, Transform::identity(), limits)?;
        let span = norm(requested_axis);
        if !span.is_finite() || span == 0. {
            return Err(Error::Invalid("extrusion axis"));
        }
        let requested_axis = requested_axis.map(|v| v / span);
        if dot(requested_axis, self.axis).abs() < 1. - 1e-12 {
            return Err(Error::Invalid("Draft direction differs from boundary pick"));
        }
        let height = dot(offset, requested_axis);
        let room = limits.remaining_after(self.face().storage_usage(limits)?)?;
        let body = super::tapered(
            curve,
            requested_axis,
            height,
            angle,
            corners,
            solid,
            self.deflection,
            &room,
        )
        .map_err(kernel)?;
        super::clip_draft(
            curve,
            &body,
            self.face(),
            self.tolerance,
            self.deflection,
            limits,
        )
        .map_err(kernel)
    }

    pub fn solid(&self, curve: &Curve, world: Transform, limits: &Limits) -> Result<Brep> {
        let offset = self.local_offset(curve, world, limits)?;
        let local;
        let face = if world == Transform::identity() {
            self.face()
        } else {
            local = transform_ops::transform_brep(
                self.face(),
                &om9_core::top11::inverse_affine(world)?,
                self.deflection,
                limits,
            )
            .map_err(kernel)?
            .into_canonical_and_representation()
            .0;
            &local
        };
        solid_to_face(curve, offset, face, self.deflection, limits).map_err(kernel)
    }

    /// The section must separate the base from the far end of the actual wall.
    /// A pair of endpoint hits alone would bridge a hole in the boundary.
    pub fn rail(&self, curve: &Curve, world: Transform, limits: &Limits) -> Result<Curve> {
        if matches!(curve, Curve::Nurbs(n) if n.degree == 1 && n.poles.len() > 2) {
            let points = curve_ops::linear_path::placed_vertices(
                curve,
                world,
                self.tolerance,
                limits,
                &mut || Ok(()),
            )?;
            let mut pieces = Vec::new();
            let mut held = om9_core::budget::Usage::array::<Point3>(points.capacity())?;
            for edge in points.windows(2) {
                let line = Curve::Line {
                    start: edge[0],
                    end: edge[1],
                    domain: [0., 1.],
                };
                let available = limits.remaining_after(held)?;
                let piece = self.rail(&line, Transform::identity(), &available)?;
                held.add(piece.validate(&available)?, limits)?;
                pieces.push(piece);
            }
            return curve_ops::join_curves(&pieces, self.tolerance, limits);
        }
        let ends = curve_ops::curve_endpoints(curve, limits)?;
        let a = self.project_station(world.point(ends[0])?, limits)?;
        let b = self.project_station(world.point(ends[1])?, limits)?;
        let offset = self.local_offset(curve, world, limits)?;
        let wall = side_wall(curve, offset, self.deflection, limits).map_err(kernel)?;
        let available = limits.remaining_after(wall.storage_usage(limits)?)?;
        let wall = Geometry::Brep(wall.into_canonical_and_representation().0);
        let split = measurement::split_surface_by_geometry(
            &wall,
            world,
            &[(&self.face, Transform::identity())],
            self.deflection,
            &available,
        )
        .map_err(kernel)?;
        let t = split.brep.topology();
        // A folded face can cut the wall at several heights. Keep the
        // connected section that reaches the nearest hit at both stations.
        // Two uses of a periodic seam on the same face are not a cut.
        let mut owners = vec![(None, false); t.edges.len()];
        for &[face, edge] in &t.face_edges {
            let (first, shared) = &mut owners[edge as usize];
            *shared |= *first.get_or_insert(face) != face;
        }
        let shared: Vec<_> = owners
            .iter()
            .enumerate()
            .filter_map(|(i, &(_, shared))| shared.then_some(i))
            .collect();
        let remaining = available.remaining_after(split.brep.storage_usage(&available)?)?;
        let mut parts = editing::edges_nurbs(&split.brep, &shared, &remaining).map_err(kernel)?;
        let mut head = a;
        let mut ordered = Vec::with_capacity(parts.len());
        while !parts.is_empty() {
            let mut match_index = None;
            for (i, part) in parts.iter().enumerate() {
                let [p, q] = curve_ops::curve_endpoints(part, &remaining)?;
                if norm(delta(head, p)) <= self.tolerance {
                    match_index = Some((i, false));
                    break;
                }
                if norm(delta(head, q)) <= self.tolerance {
                    match_index = Some((i, true));
                    break;
                }
            }
            let (i, reverse) = match_index.ok_or(Error::Invalid(
                "Boundary leaves gaps in the projected profile",
            ))?;
            let part = parts.swap_remove(i);
            let part = if reverse {
                curve_ops::reverse_curve(&part, &remaining)?
            } else {
                part
            };
            head = curve_ops::curve_endpoints(&part, &remaining)?[1];
            ordered.push(part);
            if norm(delta(head, b)) <= self.tolerance {
                break;
            }
        }
        if norm(delta(head, b)) > self.tolerance {
            return Err(Error::Invalid("Boundary does not reach the profile end"));
        }
        curve_ops::join_curves(&ordered, self.tolerance, &remaining)
    }
}
