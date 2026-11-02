// SPDX-License-Identifier: LGPL-2.1-or-later
//! Pending solid-command state: one `SolidPending` value plus its option helpers and the small
//! preview records the solid stages attach to it.
//!
//! Moved unchanged from `solid.rs`; `solid.rs` re-exports these names for the family modules.
use super::*;

pub struct SolidPending {
    pub feature: &'static str,
    pub(super) slab_source: Option<om9_core::identity::SourceStamp>,
    #[cfg(feature = "occt-brep")]
    pub(super) rib_sources: Vec<(om9_core::identity::SourceStamp, [f64; 3])>,
    pub(super) slab_offset: Option<f64>,
    pub(super) slab_distance: Option<f64>,
    pub(super) slab_normal: [f64; 3],
    pub(super) slab_cplane_normal: [f64; 3],
    pub(super) slab_in_cplane: bool,
    pub(super) slab_both_sides: bool,
    pub values: Vec<f64>,
    pub labels: &'static [&'static str],
    pub(super) text: Option<String>,
    pub points: Vec<om9_core::geometry::Point3>,
    pub(super) cone_plane: Option<Box<ConeFrame>>,
    pub cone_diameter: bool,
    pub(super) cone_area: bool,
    pub cone_base_points: u8,
    pub(super) cone_flags: ConeFlags,
    pub(super) pyramid_sides: u32,
    pub(super) pyramid_around: bool,
    pub box_mode: bool,
    pub box_kind: &'static str,
    pub(super) box_plane: Option<super::CPlane>,
    pub box_width: Option<f64>,
    pub box_length: f64,
    pub(super) ellipsoid_diameter: bool,
    pub(super) ellipsoid_corner: bool,
    pub(super) ellipsoid_around: bool,
    pub(super) ellipsoid_foci: bool,
    pub(super) ellipsoid_mark_foci: bool,
    pub(super) torus_two_point: bool,
    pub(super) torus_three_point: bool,
    pub(super) torus_vertical: bool,
    pub(super) torus_around: bool,
    pub sphere_mode: bool,
    pub sphere_center: Option<om9_core::geometry::Point3>,
    pub sphere_kind: &'static str,
    pub(super) sphere_plane: Option<super::CPlane>,
    pub(super) sphere_radius: f64,
    pub(super) sphere_normal: om9_core::geometry::Point3,
    pub sphere_curve: Vec<om9_core::geometry::Point3>,
    pub sphere_curve2: Vec<om9_core::geometry::Point3>,
    pub sphere_curve3: Vec<om9_core::geometry::Point3>,
    pub(super) sphere_sources: Vec<om9_core::identity::SourceStamp>,
    pub(super) sphere_edge: Option<usize>,
    pub(super) sphere_solution_negative: bool,
    pub(super) difference_targets: Option<Vec<om9_core::identity::SourceStamp>>,
    pub(super) difference_delete_input: bool,
    pub(super) extract_copy: bool,
    pub(super) extract_input_layer: bool,
    pub(super) extract_face_picks: Vec<super::SelectedSubobject>,
    pub(super) shell_thickness: Option<f64>,
    pub(super) shell_select_done: bool,
    pub(super) fillet_edges: Vec<FilletEdgePick>,
    pub(super) fillet_controls: Vec<FilletControl>,
    pub(super) fillet_add_mode: bool,
    pub(super) fillet_remove_mode: bool,
    pub(super) fillet_copy_phase: u8,
    pub(super) fillet_add_radius: Option<f64>,
    pub(super) fillet_radius: Option<f64>,
    pub(super) fillet_show_radius: bool,
    pub(super) fillet_link_handles: bool,
    pub(super) fillet_preview: Option<FilletPreview>,
    pub(super) fillet_handle: Option<usize>,
    pub(super) boolean_two_preview: Option<BooleanTwoPreview>,
    pub(super) pipe_global: bool,
    pub(super) pipe_capped: bool,
    pub(super) pipe_round: bool,
    pub(super) pipe_diameter: bool,
    pub(super) pipe_thick: bool,
    pub(super) pipe_stations: Vec<(f64, f64)>,
    pub(super) pipe_station_position: Option<f64>,
    pub(super) pipe_multiple: bool,
    pub(super) pipe_multiple_selecting: bool,
    pub(super) pipe_chain: bool,
    pub(super) pipe_chain_order: Vec<PipeChainItem>,
}

#[derive(Clone, Copy, Default)]
pub(super) struct ConeFlags(u8);
impl ConeFlags {
    pub(super) fn pyramid_circumscribed(self) -> bool {
        self.0 & 0x40 != 0
    }
    pub(super) fn set_pyramid_circumscribed(&mut self, value: bool) {
        if value {
            self.0 |= 0x40;
        } else {
            self.0 &= !0x40;
        }
    }
    pub(super) fn wall_thickness(self) -> bool {
        self.0 & 0x20 != 0
    }
    pub(super) fn set_wall_thickness(&mut self, value: bool) {
        if value {
            self.0 |= 0x20;
        } else {
            self.0 &= !0x20;
        }
    }
    pub(super) fn both_sides(self) -> bool {
        self.0 & 0x10 != 0
    }
    pub(super) fn set_both_sides(&mut self, value: bool) {
        if value {
            self.0 |= 0x10;
        } else {
            self.0 &= !0x10;
        }
    }
    pub(super) fn radius_phase(self) -> u8 {
        self.0 & 0x03
    }
    pub(super) fn set_radius_phase(&mut self, phase: u8) {
        self.0 = (self.0 & !0x03) | phase;
    }
    pub(super) fn solid(self) -> bool {
        self.0 & 0x04 == 0
    }
    pub(super) fn set_solid(&mut self, solid: bool) {
        if solid {
            self.0 &= !0x04;
        } else {
            self.0 |= 0x04;
        }
    }
    pub(super) fn point_mode(self) -> bool {
        self.0 & 0x08 != 0
    }
    pub(super) fn set_point_mode(&mut self, value: bool) {
        if value {
            self.0 |= 0x08;
        } else {
            self.0 &= !0x08;
        }
    }
}

pub(super) struct ConeFrame {
    pub(super) plane: super::CPlane,
    pub(super) radius: f64,
    pub(super) fit_input_plane: Option<super::CPlane>,
    pub(super) around_source: Option<om9_core::identity::SourceStamp>,
    pub(super) tangent: Option<om9_core::geometry::Point3>,
}

pub(super) fn pyramid_edge_geometry(
    plane: super::CPlane,
    first: [f64; 3],
    point: [f64; 3],
    sides: u32,
    flip: bool,
) -> Result<([f64; 3], [f64; 3], super::CPlane, f64)> {
    let d = std::array::from_fn::<_, 3, _>(|i| point[i] - first[i]);
    let u = (0..3).map(|i| d[i] * plane.x[i]).sum::<f64>();
    let v = (0..3).map(|i| d[i] * plane.y[i]).sum::<f64>();
    let length = u.hypot(v);
    if !length.is_finite() || length <= 1e-9 {
        return Err(Error::Invalid("pyramid edge length"));
    }
    let edge = std::array::from_fn::<_, 3, _>(|i| (u * plane.x[i] + v * plane.y[i]) / length);
    let normal = plane.normal();
    let left = [
        normal[1] * edge[2] - normal[2] * edge[1],
        normal[2] * edge[0] - normal[0] * edge[2],
        normal[0] * edge[1] - normal[1] * edge[0],
    ];
    let distance =
        length / (2. * (std::f64::consts::PI / sides as f64).tan()) * if flip { -1. } else { 1. };
    let second = std::array::from_fn(|i| first[i] + length * edge[i]);
    let center = std::array::from_fn(|i| (first[i] + second[i]) / 2. + distance * left[i]);
    let radius = length / (2. * (std::f64::consts::PI / sides as f64).sin());
    if !radius.is_finite() || !center.iter().all(|v| v.is_finite()) {
        return Err(Error::Invalid("pyramid edge overflow"));
    }
    let x = std::array::from_fn(|i| (first[i] - center[i]) / radius);
    let y = [
        normal[1] * x[2] - normal[2] * x[1],
        normal[2] * x[0] - normal[0] * x[2],
        normal[0] * x[1] - normal[1] * x[0],
    ];
    Ok((center, second, super::CPlane::new(center, x, y)?, radius))
}
impl std::ops::Deref for ConeFrame {
    type Target = super::CPlane;
    fn deref(&self) -> &Self::Target {
        &self.plane
    }
}

pub(super) enum PipeChainItem {
    Curve(om9_core::identity::SourceStamp),
    Edge(super::SelectedSubobject, [om9_core::geometry::Point3; 2]),
}

pub(super) struct FilletPreview {
    pub(super) source: om9_core::identity::SourceStamp,
    pub(super) body: om9_core::brep::Brep,
    pub(super) placement: om9_core::geometry::Transform,
    pub(super) usage: om9_core::budget::Usage,
}

pub(super) struct FilletEdgePick {
    pub(super) reference: super::SelectedSubobject,
    pub(super) start_radius: f64,
    pub(super) end_radius: f64,
    pub(super) endpoints: Option<(om9_core::geometry::Point3, om9_core::geometry::Point3)>,
}

pub(super) struct FilletControl {
    pub(super) edge: usize,
    pub(super) fraction: f64,
    pub(super) point: om9_core::geometry::Point3,
    pub(super) radius: f64,
}

pub(super) struct BooleanTwoPreview {
    pub(super) mode: usize,
    pub(super) sources: [om9_core::identity::SourceStamp; 2],
    pub(super) body: Option<om9_core::brep::Brep>,
    pub(super) usage: om9_core::budget::Usage,
}

impl SolidPending {
    pub(super) fn pyramid_center(&self) -> [f64; 3] {
        if matches!(self.cone_base_points, 1 | 2) && !self.values.is_empty() {
            *self.points.last().expect("pyramid center")
        } else {
            self.points[0]
        }
    }
    pub(super) fn torus_tube_geometry(
        &self,
        point: [f64; 3],
    ) -> Result<([f64; 3], super::CPlane, f64, f64)> {
        if self.points.len() != if self.torus_three_point { 3 } else { 2 } {
            return Err(Error::Missing("torus ring"));
        }
        let plane = self
            .cone_plane
            .as_deref()
            .ok_or(Error::Missing("torus frame"))?
            .plane;
        let (center, ringpoint, ring) = self.torus_base()?;
        let radial = std::array::from_fn::<_, 3, _>(|i| ringpoint[i] - center[i]);
        let x = radial.map(|v| v / ring);
        let n = plane.normal();
        let d = std::array::from_fn::<_, 3, _>(|i| point[i] - ringpoint[i]);
        let dot = |a: [f64; 3], b: [f64; 3]| (0..3).map(|i| a[i] * b[i]).sum::<f64>();
        let tube = dot(d, x).hypot(dot(d, n));
        if !tube.is_finite() || tube <= 1e-9 || tube >= ring {
            return Err(Error::Invalid(
                "torus tube radius must be positive and smaller than ring radius",
            ));
        }
        Ok((center, plane, ring, tube))
    }
    pub(super) fn torus_base(&self) -> Result<([f64; 3], [f64; 3], f64)> {
        let plane = self
            .cone_plane
            .as_deref()
            .ok_or(Error::Missing("torus frame"))?
            .plane;
        let a = *self
            .points
            .first()
            .ok_or(Error::Missing("torus first point"))?;
        let b = *self
            .points
            .get(1)
            .ok_or(Error::Missing("torus second point"))?;
        let center = if self.torus_three_point {
            let c = *self
                .points
                .get(2)
                .ok_or(Error::Missing("torus third point"))?;
            let dot = |d: [f64; 3], v: [f64; 3]| (0..3).map(|i| d[i] * v[i]).sum::<f64>();
            let ab = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
            let ac = std::array::from_fn::<_, 3, _>(|i| c[i] - a[i]);
            let (x1, y1, x2, y2) = (
                dot(ab, plane.x),
                dot(ab, plane.y),
                dot(ac, plane.x),
                dot(ac, plane.y),
            );
            let q1 = x1 * x1 + y1 * y1;
            let q2 = x2 * x2 + y2 * y2;
            let det = 2. * (x1 * y2 - y1 * x2);
            if !det.is_finite()
                || !q1.is_finite()
                || !q2.is_finite()
                || det.abs() <= 1e-12 * q1.max(q2)
            {
                return Err(Error::Invalid("torus base points must be non-collinear"));
            }
            let u = (q1 * y2 - q2 * y1) / det;
            let v = (x1 * q2 - x2 * q1) / det;
            std::array::from_fn(|i| a[i] + u * plane.x[i] + v * plane.y[i])
        } else if self.torus_two_point {
            std::array::from_fn(|i| a[i] * 0.5 + b[i] * 0.5)
        } else {
            a
        };
        let ringpoint = if self.torus_three_point {
            self.points[2]
        } else {
            b
        };
        let d = std::array::from_fn::<_, 3, _>(|i| ringpoint[i] - center[i]);
        let radius = d[0].hypot(d[1]).hypot(d[2]);
        if !radius.is_finite() || radius <= 1e-9 || !center.iter().all(|v| v.is_finite()) {
            return Err(Error::Invalid("torus ring radius must be positive"));
        }
        Ok((center, ringpoint, radius))
    }
    pub(super) fn ellipsoid_center(&self) -> [f64; 3] {
        if (self.ellipsoid_diameter || self.ellipsoid_foci || self.ellipsoid_corner)
            && self.points.len() >= 2
        {
            std::array::from_fn(|i| self.points[0][i] * 0.5 + self.points[1][i] * 0.5)
        } else {
            self.points[0]
        }
    }
    pub(super) fn ellipsoid_foci_radii(&self, point: [f64; 3]) -> Result<[f64; 2]> {
        let distance =
            |a: [f64; 3], b: [f64; 3]| (a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2]);
        let c = distance(self.points[0], self.points[1]) * 0.5;
        let a = distance(point, self.points[0]) * 0.5 + distance(point, self.points[1]) * 0.5;
        let b = ((a - c) * (a + c)).sqrt();
        if !a.is_finite() || !b.is_finite() || c <= 1e-9 || b <= 1e-9 {
            return Err(Error::Invalid(
                "foci and ellipse point must define a nondegenerate ellipse",
            ));
        }
        Ok([a, b])
    }
    pub(super) fn ellipsoid_at_height(
        &self,
        point: [f64; 3],
    ) -> Result<([f64; 3], [[f64; 3]; 3], [f64; 3])> {
        let required = if self.ellipsoid_corner { 2 } else { 3 };
        if self.points.len() != required {
            return Err(Error::Missing("ellipsoid axes"));
        }
        let plane = self
            .cone_plane
            .as_deref()
            .ok_or(Error::Missing("ellipsoid frame"))?
            .plane;
        let center = self.ellipsoid_center();
        let normal = plane.normal();
        let dot = |a: [f64; 3], b: [f64; 3]| (0..3).map(|i| a[i] * b[i]).sum::<f64>();
        let height = dot(std::array::from_fn(|i| point[i] - center[i]), normal).abs();
        if !height.is_finite() || height <= 1e-9 {
            return Err(Error::Invalid("ellipsoid axis must have positive length"));
        }
        let diagonal = std::array::from_fn::<_, 3, _>(|i| self.points[1][i] - self.points[0][i]);
        if self.ellipsoid_corner {
            return Ok((
                center,
                [plane.x, plane.y, normal],
                [
                    dot(diagonal, plane.x).abs() * 0.5,
                    dot(diagonal, plane.y).abs() * 0.5,
                    height,
                ],
            ));
        }
        let d1 = std::array::from_fn::<_, 3, _>(|i| self.points[1][i] - center[i]);
        let u = dot(d1, plane.x);
        let v = dot(d1, plane.y);
        let length = u.hypot(v);
        if !length.is_finite() || length <= 1e-9 {
            return Err(Error::Invalid("ellipsoid first axis"));
        }
        let first = std::array::from_fn(|i| (u * plane.x[i] + v * plane.y[i]) / length);
        let second = [
            normal[1] * first[2] - normal[2] * first[1],
            normal[2] * first[0] - normal[0] * first[2],
            normal[0] * first[1] - normal[1] * first[0],
        ];
        let d2 = std::array::from_fn::<_, 3, _>(|i| self.points[2][i] - center[i]);
        let [a, b] = if self.ellipsoid_foci {
            self.ellipsoid_foci_radii(self.points[2])?
        } else {
            [dot(d1, first).abs(), dot(d2, second).abs()]
        };
        if !a.is_finite() || !b.is_finite() || a <= 1e-9 || b <= 1e-9 {
            return Err(Error::Invalid("ellipsoid base axes"));
        }
        Ok((center, [first, second, normal], [a, b, height]))
    }
    pub(super) fn cone_center(&self) -> om9_core::geometry::Point3 {
        if self.cone_base_points != 0 && !self.values.is_empty() {
            *self.points.last().expect("cone center")
        } else {
            self.points[0]
        }
    }
    pub(super) fn cone_point_measure(&self, point: om9_core::geometry::Point3) -> Result<f64> {
        let frame = self
            .cone_plane
            .as_deref()
            .ok_or(Error::Missing("Truncated Cone frame"))?;
        let delta = std::array::from_fn::<_, 3, _>(|i| point[i] - self.cone_center()[i]);
        let dot = |axis: [f64; 3]| (0..3).map(|i| delta[i] * axis[i]).sum::<f64>();
        Ok(
            if self.values.len()
                == if self.feature == "OM9-SOLID-017" {
                    2
                } else {
                    1
                }
            {
                dot(frame.normal())
            } else {
                dot(frame.x).hypot(dot(frame.y))
            },
        )
    }
    pub(super) fn cone_project_base_point(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Result<om9_core::geometry::Point3> {
        let frame = self
            .cone_plane
            .as_deref()
            .ok_or(Error::Missing("Truncated Cone frame"))?;
        let a = self.points[0];
        let delta = std::array::from_fn::<_, 3, _>(|i| point[i] - a[i]);
        let u = (0..3).map(|i| delta[i] * frame.x[i]).sum::<f64>();
        let v = (0..3).map(|i| delta[i] * frame.y[i]).sum::<f64>();
        let projected = std::array::from_fn(|i| a[i] + u * frame.x[i] + v * frame.y[i]);
        if !projected.iter().all(|v| v.is_finite()) {
            return Err(Error::Invalid("base point overflow"));
        }
        Ok(projected)
    }
    pub(super) fn cone_diameter_base(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Result<(om9_core::geometry::Point3, f64)> {
        let frame = self
            .cone_plane
            .as_deref()
            .ok_or(Error::Missing("Truncated Cone frame"))?;
        let delta = std::array::from_fn::<_, 3, _>(|i| point[i] - self.points[0][i]);
        let u = (0..3).map(|i| delta[i] * frame.x[i]).sum::<f64>();
        let v = (0..3).map(|i| delta[i] * frame.y[i]).sum::<f64>();
        let radius = u.hypot(v) / 2.;
        if !radius.is_finite() || radius <= 1e-9 {
            return Err(Error::Invalid("diameter endpoints must be distinct"));
        }
        let center =
            std::array::from_fn(|i| self.points[0][i] + 0.5 * (u * frame.x[i] + v * frame.y[i]));
        if !center.iter().all(|v| v.is_finite()) {
            return Err(Error::Invalid("diameter endpoint overflow"));
        }
        Ok((center, radius))
    }
    pub(super) fn cone_three_point_base(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Result<(om9_core::geometry::Point3, f64)> {
        let frame = self
            .cone_plane
            .as_deref()
            .ok_or(Error::Missing("Truncated Cone frame"))?;
        let a = self.points[0];
        let b = self.points[1];
        let project =
            |p: [f64; 3], axis: [f64; 3]| (0..3).map(|i| (p[i] - a[i]) * axis[i]).sum::<f64>();
        let x1 = project(b, frame.x);
        let y1 = project(b, frame.y);
        let x2 = project(point, frame.x);
        let y2 = project(point, frame.y);
        let q1 = x1 * x1 + y1 * y1;
        let q2 = x2 * x2 + y2 * y2;
        let det = 2. * (x1 * y2 - y1 * x2);
        if !q1.is_finite()
            || !q2.is_finite()
            || !det.is_finite()
            || q1 <= 1e-18
            || q2 <= 1e-18
            || det.abs() <= 1e-12 * q1.max(q2)
        {
            return Err(Error::Invalid(
                "base points must be distinct and non-collinear",
            ));
        }
        let u = (q1 * y2 - q2 * y1) / det;
        let v = (x1 * q2 - x2 * q1) / det;
        let radius = u.hypot(v);
        let center = std::array::from_fn(|i| a[i] + u * frame.x[i] + v * frame.y[i]);
        if !radius.is_finite() || radius <= 1e-9 || !center.iter().all(|v| v.is_finite()) {
            return Err(Error::Invalid("base circle overflow"));
        }
        Ok((center, radius))
    }
    pub(super) fn cone_tangent_point_base(
        &self,
        point: om9_core::geometry::Point3,
    ) -> Result<(om9_core::geometry::Point3, f64)> {
        let frame = self
            .cone_plane
            .as_deref()
            .ok_or(Error::Missing("Tangent frame"))?;
        let tangent = frame.tangent.ok_or(Error::Missing("Tangent contact"))?;
        let contact = self.points[0];
        let normal = frame.normal();
        let perp = [
            normal[1] * tangent[2] - normal[2] * tangent[1],
            normal[2] * tangent[0] - normal[0] * tangent[2],
            normal[0] * tangent[1] - normal[1] * tangent[0],
        ];
        let projected = self.cone_project_base_point(point)?;
        let delta = std::array::from_fn::<_, 3, _>(|i| projected[i] - contact[i]);
        let side = (0..3).map(|i| delta[i] * perp[i]).sum::<f64>();
        let d2 = (0..3).map(|i| delta[i] * delta[i]).sum::<f64>();
        if !side.is_finite() || !d2.is_finite() || side.abs() <= 1e-9 || d2 <= 1e-18 {
            return Err(Error::Invalid("point does not define a tangent circle"));
        }
        let signed = d2 / (2. * side);
        let radius = signed.abs();
        let center = std::array::from_fn(|i| contact[i] + signed * perp[i]);
        if !radius.is_finite() || radius <= 1e-9 || !center.iter().all(|v| v.is_finite()) {
            return Err(Error::Invalid("tangent circle overflow"));
        }
        Ok((center, radius))
    }
    pub(super) fn cone_radius_base(
        &self,
        side: om9_core::geometry::Point3,
    ) -> Result<(om9_core::geometry::Point3, f64)> {
        let frame = self
            .cone_plane
            .as_deref()
            .ok_or(Error::Missing("Truncated Cone frame"))?;
        let a = self.points[0];
        let b = self.points[1];
        let delta = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
        let x = (0..3).map(|i| delta[i] * frame.x[i]).sum::<f64>();
        let y = (0..3).map(|i| delta[i] * frame.y[i]).sum::<f64>();
        let chord = x.hypot(y);
        let half = chord / 2.;
        let radius = frame.radius;
        if !radius.is_finite() || !chord.is_finite() || chord <= 1e-9 || radius < half {
            return Err(Error::Invalid("radius must be at least half the chord"));
        }
        let mid = std::array::from_fn::<_, 3, _>(|i| a[i] + delta[i] * 0.5);
        let perp = std::array::from_fn::<_, 3, _>(|i| (-y * frame.x[i] + x * frame.y[i]) / chord);
        let direction = (0..3).map(|i| (side[i] - mid[i]) * perp[i]).sum::<f64>();
        if !direction.is_finite() || direction.abs() <= 1e-9 {
            return Err(Error::Invalid("choose a side of the chord"));
        }
        let offset = ((radius - half) * (radius + half)).sqrt();
        let center = std::array::from_fn(|i| mid[i] + offset * direction.signum() * perp[i]);
        if !offset.is_finite() || !center.iter().all(|v| v.is_finite()) {
            return Err(Error::Invalid("base circle overflow"));
        }
        Ok((center, radius))
    }
    pub(super) fn sphere_solution(&self) -> usize {
        if self.sphere_solution_negative { 2 } else { 1 }
    }

    pub(super) fn usage(&self) -> Result<om9_core::budget::Usage> {
        let mut usage = om9_core::budget::Usage::array::<f64>(self.values.capacity())?;
        if matches!(
            self.feature,
            "OM9-SOLID-020"
                | "OM9-SOLID-013"
                | "OM9-SOLID-015"
                | "OM9-SOLID-016"
                | "OM9-SOLID-017"
                | "OM9-SOLID-018"
                | "OM9-SOLID-019"
        ) {
            usage.bytes = usage
                .bytes
                .checked_add(
                    om9_core::budget::Usage::array::<om9_core::geometry::Point3>(
                        self.points.capacity(),
                    )?
                    .bytes,
                )
                .ok_or(Error::Overflow("cone points bytes"))?;
            usage.points = u64::try_from(self.points.len())
                .map_err(|_| Error::Overflow("cone point count"))?;
        }
        if self.cone_plane.is_some() {
            usage.bytes = usage
                .bytes
                .checked_add(std::mem::size_of::<ConeFrame>() as u64)
                .ok_or(Error::Overflow("cone frame bytes"))?;
        }
        if self.sphere_mode {
            for points in [
                &self.points,
                &self.sphere_curve,
                &self.sphere_curve2,
                &self.sphere_curve3,
            ] {
                usage.bytes = usage
                    .bytes
                    .checked_add(
                        om9_core::budget::Usage::array::<om9_core::geometry::Point3>(
                            points.capacity(),
                        )?
                        .bytes,
                    )
                    .ok_or(Error::Overflow("sphere points bytes"))?;
                usage.points = usage
                    .points
                    .checked_add(
                        u64::try_from(points.len())
                            .map_err(|_| Error::Overflow("sphere point count"))?,
                    )
                    .ok_or(Error::Overflow("sphere point count"))?;
            }
            usage.bytes = usage
                .bytes
                .checked_add(
                    om9_core::budget::Usage::array::<om9_core::identity::SourceStamp>(
                        self.sphere_sources.capacity(),
                    )?
                    .bytes,
                )
                .ok_or(Error::Overflow("sphere source bytes"))?;
        }
        usage.bytes = usage
            .bytes
            .checked_add(
                om9_core::budget::Usage::array::<(f64, f64)>(self.pipe_stations.capacity())?.bytes,
            )
            .ok_or(Error::Overflow("pipe station bytes"))?;
        usage.bytes = usage
            .bytes
            .checked_add(
                om9_core::budget::Usage::array::<PipeChainItem>(self.pipe_chain_order.capacity())?
                    .bytes,
            )
            .ok_or(Error::Overflow("pipe chain edge bytes"))?;
        if let Some(text) = &self.text {
            usage.bytes = usage
                .bytes
                .checked_add(text.capacity() as u64)
                .ok_or(Error::Overflow("text prompt bytes"))?;
        }
        if let Some(targets) = &self.difference_targets {
            usage.bytes = usage
                .bytes
                .checked_add(
                    om9_core::budget::Usage::array::<om9_core::identity::SourceStamp>(
                        targets.capacity(),
                    )?
                    .bytes,
                )
                .ok_or(Error::Overflow("boolean target bytes"))?;
        }
        usage.bytes = usage
            .bytes
            .checked_add(
                om9_core::budget::Usage::array::<super::SelectedSubobject>(
                    self.extract_face_picks.capacity(),
                )?
                .bytes,
            )
            .ok_or(Error::Overflow("extract selection bytes"))?;
        usage.bytes = usage
            .bytes
            .checked_add(
                om9_core::budget::Usage::array::<FilletEdgePick>(self.fillet_edges.capacity())?
                    .bytes,
            )
            .ok_or(Error::Overflow("fillet selection bytes"))?;
        usage.bytes = usage
            .bytes
            .checked_add(
                om9_core::budget::Usage::array::<FilletControl>(self.fillet_controls.capacity())?
                    .bytes,
            )
            .ok_or(Error::Overflow("fillet control bytes"))?;
        if let Some(preview) = &self.fillet_preview {
            usage.bytes = usage
                .bytes
                .checked_add(preview.usage.bytes)
                .ok_or(Error::Overflow("fillet preview bytes"))?;
            usage.points = usage
                .points
                .checked_add(preview.usage.points)
                .ok_or(Error::Overflow("fillet preview points"))?;
            usage.nodes = usage
                .nodes
                .checked_add(preview.usage.nodes)
                .ok_or(Error::Overflow("fillet preview nodes"))?;
        }
        if let Some(preview) = &self.boolean_two_preview {
            usage.bytes = usage
                .bytes
                .checked_add(preview.usage.bytes)
                .ok_or(Error::Overflow("Boolean preview bytes"))?;
            usage.points = usage
                .points
                .checked_add(preview.usage.points)
                .ok_or(Error::Overflow("Boolean preview points"))?;
            usage.nodes = usage
                .nodes
                .checked_add(preview.usage.nodes)
                .ok_or(Error::Overflow("Boolean preview nodes"))?;
        }
        Ok(usage)
    }
    pub fn start(feature: &'static str) -> Option<Self> {
        let labels: &'static [&'static str] = match feature {
            "SolidRibDevelopment" => &["Select Rib curve"],
            "SolidSlab" => &["Offset distance", "Height"],
            "SolidBooleanSplit" => &["Select first set (Enter to continue)"],
            "SolidCreateSolid" => &["Select intersecting B-Rep surfaces (Enter to execute)"],
            "OM9-SOLID-021" => &["Select closed planar curves (Enter to continue)"],
            "OM9-SOLID-005" => &["Select surfaces or polysurfaces to cap (Enter to execute)"],
            "OM9-SOLID-001" => &["Select objects to subtract from (Enter to continue)"],
            "OM9-SOLID-002" => &["Select first set (Enter to continue)"],
            "OM9-SOLID-003" => &["Select solids to union (Enter to execute)"],
            "OM9-SOLID-004" => &[
                "Select two B-Rep solids; Operation (U=Union D=Difference R=ReverseDifference I=Intersection X=InversionIntersection)",
            ],
            "OM9-SOLID-008" => &["Text", "Height"],
            "OM9-SOLID-012" => &["Width", "Depth", "Height"],
            "OM9-SOLID-013" => &["X radius", "Y radius", "Z radius"],
            "OM9-SOLID-014" => &["Radius"],
            "OM9-SOLID-015" => &["Ring radius", "Tube radius"],
            "OM9-SOLID-016" => &["Radius", "Height"],
            "OM9-SOLID-017" => &["Outer radius", "Inner radius", "Height"],
            "OM9-SOLID-018" => &["Sides", "Base radius", "Height"],
            "OM9-SOLID-019" => &["Bottom radius", "Height"],
            "OM9-SOLID-020" => &["Bottom radius", "Top radius", "Height"],
            "OM9-SOLID-009" | "OM9-SOLID-010" => &["Height"],
            "OM9-SOLID-011" => &["Radius"],
            "OM9-SOLID-029" => &["Select faces to remove (Enter to continue)"],
            "OM9-SOLID-007" => &["Select edges and set NextRadius (Enter to preview)"],
            "OM9-SOLID-006" => &["Select face or enter face index (Enter to execute)"],
            _ => return None,
        };
        Some(Self {
            feature,
            slab_source: None,
            #[cfg(feature = "occt-brep")]
            rib_sources: Vec::new(),
            slab_offset: None,
            slab_distance: None,
            slab_normal: [0., 0., 1.],
            slab_cplane_normal: [0., 0., 1.],
            slab_in_cplane: false,
            slab_both_sides: false,
            values: Vec::new(),
            labels,
            text: None,
            points: Vec::new(),
            cone_plane: None,
            cone_diameter: false,
            cone_area: false,
            cone_base_points: 0,
            cone_flags: ConeFlags::default(),
            pyramid_sides: 4,
            pyramid_around: false,
            box_mode: false,
            box_kind: "Corners",
            box_plane: None,
            box_width: None,
            box_length: 0.0,
            ellipsoid_diameter: false,
            ellipsoid_corner: false,
            ellipsoid_around: false,
            ellipsoid_foci: false,
            ellipsoid_mark_foci: false,
            torus_two_point: false,
            torus_three_point: false,
            torus_vertical: false,
            torus_around: false,
            sphere_mode: false,
            sphere_center: None,
            sphere_kind: "Center",
            sphere_plane: None,
            sphere_radius: 0.0,
            sphere_normal: [0.0, 0.0, 1.0],
            sphere_curve: Vec::new(),
            sphere_curve2: Vec::new(),
            sphere_curve3: Vec::new(),
            sphere_sources: Vec::new(),
            sphere_edge: None,
            sphere_solution_negative: false,
            difference_targets: None,
            difference_delete_input: false,
            extract_copy: false,
            extract_input_layer: false,
            extract_face_picks: Vec::new(),
            shell_thickness: None,
            shell_select_done: false,
            fillet_edges: Vec::new(),
            fillet_controls: Vec::new(),
            fillet_add_mode: false,
            fillet_remove_mode: false,
            fillet_copy_phase: 0,
            fillet_add_radius: None,
            fillet_radius: None,
            fillet_show_radius: true,
            fillet_link_handles: false,
            fillet_preview: None,
            fillet_handle: None,
            boolean_two_preview: None,
            pipe_global: false,
            pipe_capped: true,
            pipe_round: false,
            pipe_diameter: false,
            pipe_thick: false,
            pipe_stations: Vec::new(),
            pipe_station_position: None,
            pipe_multiple: false,
            pipe_multiple_selecting: false,
            pipe_chain: false,
            pipe_chain_order: Vec::new(),
        })
    }
    pub fn box_pending() -> Self {
        Self {
            feature: "OM9-SOLID-012",
            slab_source: None,
            #[cfg(feature = "occt-brep")]
            rib_sources: Vec::new(),
            slab_offset: None,
            slab_distance: None,
            slab_normal: [0., 0., 1.],
            slab_cplane_normal: [0., 0., 1.],
            slab_in_cplane: false,
            slab_both_sides: false,
            values: Vec::new(),
            labels: &[
                "First corner of base",
                "Other corner of base or length",
                "Height",
            ],
            text: None,
            points: Vec::new(),
            cone_plane: None,
            cone_diameter: false,
            cone_area: false,
            cone_base_points: 0,
            cone_flags: ConeFlags::default(),
            pyramid_sides: 4,
            pyramid_around: false,
            box_mode: true,
            box_kind: "Corners",
            box_plane: None,
            box_width: None,
            box_length: 0.0,
            ellipsoid_diameter: false,
            ellipsoid_corner: false,
            ellipsoid_around: false,
            ellipsoid_foci: false,
            ellipsoid_mark_foci: false,
            torus_two_point: false,
            torus_three_point: false,
            torus_vertical: false,
            torus_around: false,
            sphere_mode: false,
            sphere_center: None,
            sphere_kind: "Center",
            sphere_plane: None,
            sphere_radius: 0.0,
            sphere_normal: [0.0, 0.0, 1.0],
            sphere_curve: Vec::new(),
            sphere_curve2: Vec::new(),
            sphere_curve3: Vec::new(),
            sphere_sources: Vec::new(),
            sphere_edge: None,
            sphere_solution_negative: false,
            difference_targets: None,
            difference_delete_input: false,
            extract_copy: false,
            extract_input_layer: false,
            extract_face_picks: Vec::new(),
            shell_thickness: None,
            shell_select_done: false,
            fillet_edges: Vec::new(),
            fillet_controls: Vec::new(),
            fillet_add_mode: false,
            fillet_remove_mode: false,
            fillet_copy_phase: 0,
            fillet_add_radius: None,
            fillet_radius: None,
            fillet_show_radius: true,
            fillet_link_handles: false,
            fillet_preview: None,
            fillet_handle: None,
            boolean_two_preview: None,
            pipe_global: false,
            pipe_capped: true,
            pipe_round: false,
            pipe_diameter: false,
            pipe_thick: false,
            pipe_stations: Vec::new(),
            pipe_station_position: None,
            pipe_multiple: false,
            pipe_multiple_selecting: false,
            pipe_chain: false,
            pipe_chain_order: Vec::new(),
        }
    }
    pub fn sphere_pending() -> Self {
        Self {
            feature: "OM9-SOLID-014",
            slab_source: None,
            #[cfg(feature = "occt-brep")]
            rib_sources: Vec::new(),
            slab_offset: None,
            slab_distance: None,
            slab_normal: [0., 0., 1.],
            slab_cplane_normal: [0., 0., 1.],
            slab_in_cplane: false,
            slab_both_sides: false,
            values: Vec::new(),
            labels: &["Center", "Radius"],
            text: None,
            points: Vec::new(),
            cone_plane: None,
            cone_diameter: false,
            cone_area: false,
            cone_base_points: 0,
            cone_flags: ConeFlags::default(),
            pyramid_sides: 4,
            pyramid_around: false,
            box_mode: false,
            box_kind: "Corners",
            box_plane: None,
            box_width: None,
            box_length: 0.0,
            ellipsoid_diameter: false,
            ellipsoid_corner: false,
            ellipsoid_around: false,
            ellipsoid_foci: false,
            ellipsoid_mark_foci: false,
            torus_two_point: false,
            torus_three_point: false,
            torus_vertical: false,
            torus_around: false,
            sphere_mode: true,
            sphere_center: None,
            sphere_kind: "Center",
            sphere_plane: None,
            sphere_radius: 0.0,
            sphere_normal: [0.0, 0.0, 1.0],
            sphere_curve: Vec::new(),
            sphere_curve2: Vec::new(),
            sphere_curve3: Vec::new(),
            sphere_sources: Vec::new(),
            sphere_edge: None,
            sphere_solution_negative: false,
            difference_targets: None,
            difference_delete_input: false,
            extract_copy: false,
            extract_input_layer: false,
            extract_face_picks: Vec::new(),
            shell_thickness: None,
            shell_select_done: false,
            fillet_edges: Vec::new(),
            fillet_controls: Vec::new(),
            fillet_add_mode: false,
            fillet_remove_mode: false,
            fillet_copy_phase: 0,
            fillet_add_radius: None,
            fillet_radius: None,
            fillet_show_radius: true,
            fillet_link_handles: false,
            fillet_preview: None,
            fillet_handle: None,
            boolean_two_preview: None,
            pipe_global: false,
            pipe_capped: true,
            pipe_round: false,
            pipe_diameter: false,
            pipe_thick: false,
            pipe_stations: Vec::new(),
            pipe_station_position: None,
            pipe_multiple: false,
            pipe_multiple_selecting: false,
            pipe_chain: false,
            pipe_chain_order: Vec::new(),
        }
    }
    pub fn prompt(&self) -> &'static str {
        if self.feature == "SolidRibDevelopment" {
            return if self.slab_source.is_none() {
                "Select Rib curve"
            } else if self.difference_targets.is_none() {
                "Select Rib boundary"
            } else {
                "Rib signed distance"
            };
        }
        if self.feature == "SolidSlab" {
            return if self.slab_source.is_none() {
                "Select curve for Slab"
            } else if self.slab_offset.is_some() {
                "Slab height"
            } else if self.slab_distance.is_some() {
                "Pick Slab offset side"
            } else {
                "Slab offset distance or through point"
            };
        }
        if self.feature == "OM9-SOLID-018" {
            return if self.points.is_empty() {
                if self.pyramid_around {
                    "Pick center on curve or OnCurve="
                } else if matches!(self.cone_base_points, 1 | 2) {
                    "First point of base edge"
                } else {
                    "Center of pyramid base"
                }
            } else if self.values.is_empty() {
                if matches!(self.cone_base_points, 1 | 2) {
                    "Second point of base edge"
                } else if self.cone_base_points == 0 && self.cone_flags.pyramid_circumscribed() {
                    "Base edge midpoint or radius"
                } else {
                    "Base corner or radius"
                }
            } else if self.cone_base_points == 3 && self.values.len() == 1 {
                "Second star radius or point (Automatic)"
            } else {
                "Apex height or point"
            };
        }
        if self.feature == "OM9-SOLID-017" {
            if self.cone_base_points == 6 && self.values.is_empty() {
                return if self.points.is_empty() {
                    "Pick tangent location or OnCurve="
                } else if self.cone_flags.point_mode() {
                    "Pick point on base circle"
                } else {
                    "Choose Point for a non-tangent constraint"
                };
            }
            if self.cone_base_points == 4 && self.values.is_empty() {
                return "Add fit points or Enter to fit";
            }
            if self.cone_base_points == 5 && self.points.is_empty() {
                return "Pick center on curve or OnCurve=";
            }
            if self.values.is_empty() && self.cone_base_points == 2 {
                return if self.points.is_empty() {
                    "First diameter endpoint"
                } else {
                    "Other diameter endpoint"
                };
            }
            if self.values.is_empty() && self.cone_base_points == 3 {
                if self.cone_flags.radius_phase() == 1 {
                    return "Base circle radius";
                }
                if self.cone_flags.radius_phase() == 2 {
                    return "Choose base circle side";
                }
                return match self.points.len() {
                    0 => "First point on base circle",
                    1 => "Second point on base circle",
                    _ => "Third point on base circle",
                };
            }
            return match (self.points.is_empty(), self.values.len()) {
                (true, _) => "Center of tube base",
                (_, 0) if self.cone_area => "Base area",
                (_, 0) => "First radius or point",
                (_, 1) if self.cone_flags.wall_thickness() => "Wall thickness or point",
                (_, 1) => "Second radius or point",
                _ => "Height or point",
            };
        }
        if matches!(self.feature, "OM9-SOLID-016" | "OM9-SOLID-019") {
            if self.cone_base_points == 6 && self.values.is_empty() {
                return if self.points.is_empty() {
                    "Pick tangent location or OnCurve="
                } else if self.cone_flags.point_mode() {
                    "Pick point on base circle"
                } else {
                    "Choose Point for a non-tangent constraint"
                };
            }
            if self.cone_base_points == 4 && self.values.is_empty() {
                return "Add fit points or Enter to fit";
            }
            if self.cone_base_points == 5 && self.points.is_empty() {
                return "Pick center on curve or OnCurve=";
            }
            if self.cone_base_points == 3 && self.values.is_empty() {
                if self.cone_flags.radius_phase() == 1 {
                    return "Base circle radius";
                }
                if self.cone_flags.radius_phase() == 2 {
                    return "Choose base circle side";
                }
                return match self.points.len() {
                    0 => "First point on base circle",
                    1 => "Second point on base circle",
                    _ => "Third point on base circle",
                };
            }
            if self.cone_base_points == 2 && self.values.is_empty() {
                return if self.points.is_empty() {
                    "First diameter endpoint"
                } else {
                    "Other diameter endpoint"
                };
            }
            return match (self.points.is_empty(), self.values.len()) {
                (true, _) if self.feature == "OM9-SOLID-019" => {
                    "Center of cone (or enter radius and height)"
                }
                (true, _) => "Center of cylinder (or enter radius and height)",
                (_, 0) if self.cone_area => "Base area",
                (_, 0) => "Base radius or point",
                _ => "Height or point",
            };
        }
        if self.feature == "OM9-SOLID-015" && self.values.is_empty() {
            if self.torus_around && self.points.is_empty() {
                return "Pick center on curve or OnCurve=";
            }
            if self.torus_three_point {
                if self.cone_flags.radius_phase() == 1 {
                    return "Base circle radius";
                }
                if self.cone_flags.radius_phase() == 2 {
                    return "Choose base circle side";
                }
                return match self.points.len() {
                    0 => "First point on ring",
                    1 => "Second point on ring",
                    2 => "Third point on ring",
                    _ => "Tube radius or point",
                };
            }
            return match (self.torus_two_point, self.points.len()) {
                (true, 0) => "First diameter endpoint",
                (true, 1) => "Other diameter endpoint",
                (_, 0) => "Center of torus (or enter two radii)",
                (_, 1) => "Ring radius or point",
                _ => "Tube radius or point",
            };
        }
        if self.feature == "OM9-SOLID-013" && self.values.is_empty() {
            if self.ellipsoid_around && self.points.is_empty() {
                return "Pick center on curve or OnCurve=";
            }
            if self.ellipsoid_corner {
                return match self.points.len() {
                    0 => "First corner of ellipse box",
                    1 => "Opposite corner of ellipse box",
                    _ => "End of third axis",
                };
            }
            if self.ellipsoid_foci {
                return match self.points.len() {
                    0 => "First focus",
                    1 => "Second focus",
                    2 => "Point on base ellipse",
                    _ => "End of third axis",
                };
            }
            if self.ellipsoid_diameter {
                return match self.points.len() {
                    0 => "First end of first axis",
                    1 => "Other end of first axis",
                    2 => "End of second axis",
                    _ => "End of third axis",
                };
            }
            return match self.points.len() {
                0 => "Center of ellipsoid (or enter three radii)",
                1 => "End of first axis",
                2 => "End of second axis",
                _ => "End of third axis",
            };
        }
        if self.feature == "OM9-SOLID-020" {
            if self.cone_base_points == 5 && self.points.is_empty() {
                return "Pick center on curve or OnCurve=";
            }
            if self.cone_base_points == 6 && self.values.is_empty() {
                return if self.points.is_empty() {
                    "Pick tangent location or OnCurve="
                } else if self.cone_flags.point_mode() {
                    "Pick point on base circle"
                } else {
                    "Choose Point for a non-tangent constraint"
                };
            }
            if self.cone_base_points == 4 && self.values.is_empty() {
                return "Add fit points or Enter to fit";
            }
            if self.values.is_empty() && self.cone_flags.radius_phase() == 1 {
                return "Base circle radius";
            }
            if self.values.is_empty() && self.cone_flags.radius_phase() == 2 {
                return "Choose base circle side";
            }
            if self.cone_base_points == 3 && self.values.is_empty() {
                return match self.points.len() {
                    0 => "First point on base circle",
                    1 => "Second point on base circle",
                    _ => "Third point on base circle",
                };
            }
            if self.cone_base_points == 2 && self.values.is_empty() {
                return if self.points.is_empty() {
                    "First diameter endpoint"
                } else {
                    "Other diameter endpoint"
                };
            }
            return if self.points.is_empty() {
                "Center of base"
            } else {
                match self.values.len() {
                    0 if self.cone_area => "Base area",
                    0 => "Base radius or point",
                    1 => "Height or point",
                    _ => "Top radius or point",
                }
            };
        }
        if self.feature == "OM9-SOLID-011" && self.pipe_multiple_selecting {
            return if self.pipe_chain {
                "Select connected curve segments (Enter when done)"
            } else {
                "Select curves for Pipe (Enter when done)"
            };
        }
        if self.feature == "OM9-SOLID-011" && self.pipe_station_position.is_some() {
            return if self.pipe_diameter {
                "Intermediate diameter"
            } else {
                "Intermediate radius"
            };
        }
        if self.feature == "OM9-SOLID-011" && self.pipe_thick {
            return match (self.values.len(), self.pipe_diameter) {
                (0, false) => "First start radius",
                (0, true) => "First start diameter",
                (1, false) => "Second start radius",
                (1, true) => "Second start diameter",
                (2, false) => "First end radius",
                (2, true) => "First end diameter",
                (3, false) => "Second end radius",
                (3, true) => "Second end diameter",
                _ => "Press Enter to create thick Pipe (intermediate radii unavailable)",
            };
        }
        if self.feature == "OM9-SOLID-011" && self.pipe_global {
            return match (self.values.len(), self.pipe_diameter) {
                (0, false) => "Start radius (Global)",
                (0, true) => "Start diameter (Global)",
                (1, false) => "End radius (Global)",
                (1, true) => "End diameter (Global)",
                _ => "Pick next radius location or Enter to create Pipe",
            };
        }
        if self.feature == "OM9-SOLID-011" {
            return match (self.values.len(), self.pipe_diameter) {
                (0, false) => "Start radius",
                (0, true) => "Start diameter",
                (1, false) => "End radius",
                (1, true) => "End diameter",
                _ => "Pick next radius location or Enter to create Pipe",
            };
        }
        if self.feature == "OM9-SOLID-029" && self.shell_select_done {
            return "Enter shell thickness";
        }
        if self.feature == "OM9-SOLID-007" && self.fillet_preview.is_some() {
            if self.fillet_copy_phase == 1 {
                return "Select a fillet handle to copy";
            }
            if self.fillet_copy_phase == 2 {
                return "Select a new location or handle to copy to (Enter when done)";
            }
            if self.fillet_add_mode {
                return "Select edge location for AddHandle (Enter when done)";
            }
            if self.fillet_remove_mode {
                return "Select added handles to remove (Enter when done)";
            }
            return if self.fillet_handle.is_some() {
                "Edit selected handle with CurrentRadius (Enter to accept)"
            } else {
                "Select a fillet handle or Enter to accept"
            };
        }
        if self.feature == "OM9-SOLID-021" {
            return if self.difference_targets.is_some() {
                "Select planar boundary surface"
            } else {
                "Select closed planar curves (Enter to continue)"
            };
        }
        if self.feature == "SolidBooleanSplit" {
            return if self.difference_targets.is_some() {
                "Select cutting objects (Enter to execute)"
            } else {
                "Select first set (Enter to continue)"
            };
        }
        if self.feature == "OM9-SOLID-004" && self.boolean_two_preview.is_some() {
            if self
                .boolean_two_preview
                .as_ref()
                .is_some_and(|p| p.body.is_none())
            {
                return "Empty Boolean preview: choose operation or Next";
            }
            return "Boolean preview: choose operation or Next; Enter accepts";
        }
        if self.feature == "OM9-SOLID-002" {
            return if self.difference_targets.is_some() {
                "Select second set (Enter to execute)"
            } else {
                "Select first set (Enter to continue)"
            };
        }
        if self.feature == "OM9-SOLID-001" {
            return if self.difference_targets.is_some() {
                "Select objects to subtract (Enter to execute)"
            } else {
                "Select objects to subtract from (Enter to continue)"
            };
        }
        if self.box_mode {
            return match (self.points.len(), self.box_kind) {
                (0, "Center") => "Center of base",
                (0, "ThreePoint" | "Vertical") => "Start of edge",
                (0, _) => "First corner of base ( Diagonal 3Point Vertical Center ) :",
                (1, "Diagonal") => "Other corner of base or length ( 3Point ) :",
                (1, "Cube") => "Other corner of box",
                (1, "ThreePoint" | "Vertical") => "End of first edge or Length",
                (1, "Center") if self.box_length > 0.0 && self.box_width.is_none() => {
                    "Width. Press Enter to use length"
                }
                (1, "Center") if self.box_length > 0.0 => "Height. Press Enter to use width",
                (1, "Center") => "Other corner of base or length",
                (1, _) => "Other corner of base or length ( 3Point ) :",
                (_, "ThreePoint" | "Vertical") if self.box_width.is_none() => {
                    "Width. Press Enter to use length"
                }
                (_, "Cube") => "Press Enter to create the cube",
                _ => "Height. Press Enter to use width:",
            };
        }
        if self.sphere_mode {
            if self.sphere_kind == "ThreePoint" && self.points.len() == 2 {
                if self.sphere_radius < 0.0 {
                    return "Radius";
                }
                if self.sphere_radius > 0.0 {
                    return "Choose direction away from chord";
                }
            }
            return match (
                self.sphere_kind,
                self.points.len(),
                self.sphere_center.is_some(),
            ) {
                ("TwoPoint", 0, _) => "First diameter endpoint",
                ("TwoPoint", 1, _) => "Other diameter endpoint or diameter",
                ("ThreePoint", 0, _) => "First point",
                ("ThreePoint", 1, _) => "Second point",
                ("ThreePoint", _, _) => "Third point",
                ("FourPoint", 0, _) => "First point",
                ("FourPoint", 1, _) => "Second point",
                ("FourPoint", 2, _) => "Third point",
                ("FourPoint", _, _) => "Fourth point",
                ("FitPoints", _, _) => "Add points. Press Enter to fit sphere",
                ("AroundCurve", _, false) if self.sphere_curve.is_empty() => {
                    "Select curve or B-Rep edge"
                }
                ("AroundCurve", _, false) => "Pick center on curve (OnCurve=0..1)",
                ("AroundCurve", _, true) => "Radius. Pick a point or enter a positive value",
                ("Tangent", _, false) if self.sphere_curve.is_empty() => {
                    "Select tangent curve or Point"
                }
                ("Tangent", _, false) => "Pick tangent point on curve",
                ("Tangent", _, true) => "Radius for tangent sphere",
                ("Tangent2", _, _) => "Radius for two tangent curves",
                ("Tangent3", _, _) => "Radius for three tangent curves",
                ("Tangent2Point", 0, _) => "Pick point constraint",
                ("Tangent2Point", _, _) => "Press Enter for the two curves and point",
                ("TangentCurvePoint", 0, _) => "Pick point constraint",
                ("TangentCurvePoint", 1, _) => "Pick second point or enter radius",
                ("TangentCurvePoint", _, _) => "Press Enter to use the two points",
                ("TangentPoint", 0, _) => "Pick first point constraint",
                ("TangentPoint", 1, _) => "Pick second point constraint",
                ("TangentPoint", 2, _) => "Pick third point or enter radius (Solution=1|2)",
                ("TangentPoint", _, _) => "Radius for two point constraints (Solution=1|2)",
                ("Center" | "Vertical", _, false) => "Center of sphere",
                _ => "Radius. Pick a point or enter a positive value",
            };
        }
        if self.feature == "OM9-SOLID-008" && self.text.is_some() {
            return "Height";
        }
        self.labels
            .get(self.values.len())
            .copied()
            .unwrap_or("Value")
    }
}
