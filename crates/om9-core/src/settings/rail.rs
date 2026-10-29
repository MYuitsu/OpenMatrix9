// SPDX-License-Identifier: LGPL-2.1-or-later
//! Pull Object to Rail (`OM9-L3-04-15-008`): the finger-rail morph placement.
//!
//! The sheet is sparse on this command ("morph setting parts onto the target rail"), so this
//! owner defines the smallest complete contract it can defend:
//!
//! * a rail is a circle (centre, axis, radius) — the finger rail ring the part must follow;
//! * a pull projects the part's origin onto that circle and returns the frame the part adopts;
//! * `Rigid` moves the shortest 3D vector, `Deform` keeps the part's height and only pulls it
//!   radially, which is the difference the rigid/deform option documents;
//! * every result carries the measured displacement, radial gap and axial offset so the UI and
//!   the evidence record do not have to guess.
use crate::{
    Error, Result,
    geometry::{Mesh, Point3},
    settings::{Frame, add, cross, dot, mul, norm, sub, unit},
};

/// The target rail: a circle around `axis` at `radius_mm` from `center`.
#[derive(Debug, Clone, PartialEq)]
pub struct Rail {
    pub center: Point3,
    /// Unit normal of the rail plane.
    pub axis: Point3,
    pub radius_mm: f64,
    /// Optional angular window in degrees (0 = +X direction); the sheet's rail start/end.
    pub range_deg: Option<[f64; 2]>,
}

impl Rail {
    pub fn new(center: Point3, axis: Point3, radius_mm: f64) -> Result<Self> {
        let axis = unit(axis)?;
        if !radius_mm.is_finite() || radius_mm <= 0.0 {
            return Err(Error::Invalid("rail radius"));
        }
        Ok(Self {
            center,
            axis,
            radius_mm,
            range_deg: None,
        })
    }

    pub fn with_range(mut self, from_deg: f64, to_deg: f64) -> Result<Self> {
        if !from_deg.is_finite() || !to_deg.is_finite() || from_deg >= to_deg {
            return Err(Error::Invalid("rail range"));
        }
        self.range_deg = Some([from_deg, to_deg]);
        Ok(self)
    }

    fn basis(&self) -> Result<(Point3, Point3)> {
        let hint = if self.axis[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let x = unit(sub(hint, mul(self.axis, dot(hint, self.axis))))?;
        let y = cross(self.axis, x);
        Ok((x, y))
    }

    /// Point on the rail at an angle measured from the rail's local +X axis.
    pub fn point_at_deg(&self, degrees: f64) -> Result<Point3> {
        let (x, y) = self.basis()?;
        let radians = degrees.to_radians();
        Ok(add(
            self.center,
            add(
                mul(x, self.radius_mm * radians.cos()),
                mul(y, self.radius_mm * radians.sin()),
            ),
        ))
    }
}

/// Where a point projects onto a rail.
#[derive(Debug, Clone, PartialEq)]
pub struct RailProjection {
    /// Nearest point on the rail circle.
    pub point: Point3,
    /// Radial distance of the input from the rail axis.
    pub radial_mm: f64,
    /// Position along the rail axis relative to its centre.
    pub axial_mm: f64,
    /// Angle around the rail axis in degrees, in the rail basis.
    pub angle_deg: f64,
}

impl Rail {
    /// Project a point onto the rail circle.
    pub fn project(&self, point: Point3) -> Result<RailProjection> {
        let relative = sub(point, self.center);
        let axial = dot(relative, self.axis);
        let radial = sub(relative, mul(self.axis, axial));
        let distance = norm(radial);
        if distance <= 1.0e-9 {
            return Err(Error::Invalid("rail projection on axis"));
        }
        let direction = unit(radial)?;
        let (x, y) = self.basis()?;
        let angle_deg = dot(direction, y).atan2(dot(direction, x)).to_degrees();
        if let Some([from, to]) = self.range_deg {
            let normalized = if angle_deg < from { angle_deg + 360.0 } else { angle_deg };
            if normalized < from || normalized > to {
                return Err(Error::Invalid("rail range"));
            }
        }
        Ok(RailProjection {
            point: add(self.center, mul(direction, self.radius_mm)),
            radial_mm: distance,
            axial_mm: axial,
            angle_deg,
        })
    }
}

/// How the part is moved onto the rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailPullMode {
    /// Shortest 3D translation onto the rail point.
    Rigid,
    /// Keep the part's axial position, pull it radially only.
    Deform,
}

/// A completed pull.
#[derive(Debug, Clone, PartialEq)]
pub struct PullResult {
    pub origin: Point3,
    pub frame: Frame,
    pub displacement_mm: f64,
    /// Signed radial gap before the pull; 0 means the part already sat on the rail.
    pub radial_gap_mm: f64,
    /// Axial offset that `Rigid` removes and `Deform` keeps.
    pub axial_offset_mm: f64,
    pub angle_deg: f64,
}

impl PullResult {
    pub fn is_on_rail(&self, rail: &Rail, tolerance_mm: f64) -> bool {
        let relative = sub(self.origin, rail.center);
        let axial = dot(relative, rail.axis);
        let radial = norm(sub(relative, mul(rail.axis, axial)));
        (radial - rail.radius_mm).abs() <= tolerance_mm
    }
}

/// Pull one part origin (with a part axis) onto the rail.
pub fn pull_to_rail(
    origin: Point3,
    part_axis: Point3,
    rail: &Rail,
    mode: RailPullMode,
) -> Result<PullResult> {
    let projection = rail.project(origin)?;
    let relative = sub(origin, rail.center);
    let axial = dot(relative, rail.axis);
    let radial = sub(relative, mul(rail.axis, axial));
    let radial_direction = unit(radial)?;
    let (target, displacement) = match mode {
        RailPullMode::Rigid => {
            let target = projection.point;
            (target, norm(sub(target, origin)))
        }
        RailPullMode::Deform => {
            let target = add(self_center(rail, axial), mul(radial_direction, rail.radius_mm));
            (target, (projection.radial_mm - rail.radius_mm).abs())
        }
    };
    let frame = Frame::from_axis(target, unit(part_axis)?, radial_direction)?;
    Ok(PullResult {
        origin: target,
        frame,
        displacement_mm: displacement,
        radial_gap_mm: projection.radial_mm - rail.radius_mm,
        axial_offset_mm: if mode == RailPullMode::Deform {
            projection.axial_mm
        } else {
            0.0
        },
        angle_deg: projection.angle_deg,
    })
}

fn self_center(rail: &Rail, axial: f64) -> Point3 {
    add(rail.center, mul(rail.axis, axial))
}

/// A report for a morphed mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshPullReport {
    pub mode: RailPullMode,
    pub vertices: usize,
    pub max_displacement_mm: f64,
    pub max_radial_error_mm: f64,
    pub axial_span_mm: f64,
}

/// Morph every vertex of a mesh onto the rail.
///
/// `Rigid` translates the whole body by one vector, so the shape is preserved exactly.
/// `Deform` sets each vertex's radial distance to the rail radius and keeps its axial
/// coordinate, which is the sheet's deform behaviour.
pub fn pull_mesh(mesh: &Mesh, rail: &Rail, mode: RailPullMode) -> Result<(Mesh, MeshPullReport)> {
    if mesh.vertices.is_empty() || mesh.faces.is_empty() {
        return Err(Error::Invalid("rail mesh"));
    }
    let mut out = mesh.clone();
    let mut max_displacement = 0.0_f64;
    let mut max_radial_error = 0.0_f64;
    let mut min_axial = f64::INFINITY;
    let mut max_axial = f64::NEG_INFINITY;
    match mode {
        RailPullMode::Rigid => {
            let centroid = mesh
                .vertices
                .iter()
                .fold([0.0; 3], |acc, point| add(acc, *point));
            let centroid = mul(centroid, 1.0 / mesh.vertices.len() as f64);
            let pull = pull_to_rail(centroid, [0.0, 0.0, 1.0], rail, mode)?;
            let delta = sub(pull.origin, centroid);
            max_displacement = norm(delta);
            for vertex in &mut out.vertices {
                *vertex = add(*vertex, delta);
            }
            max_radial_error = 0.0;
        }
        RailPullMode::Deform => {
            for vertex in &mut out.vertices {
                let relative = sub(*vertex, rail.center);
                let axial = dot(relative, rail.axis);
                let radial = sub(relative, mul(rail.axis, axial));
                let distance = norm(radial);
                if distance <= 1.0e-9 {
                    return Err(Error::Invalid("rail mesh on axis"));
                }
                let direction = unit(radial)?;
                let target = add(self_center(rail, axial), mul(direction, rail.radius_mm));
                max_displacement = max_displacement.max(norm(sub(target, *vertex)));
                max_radial_error = max_radial_error
                    .max((norm(sub(target, self_center(rail, axial))) - rail.radius_mm).abs());
                min_axial = min_axial.min(axial);
                max_axial = max_axial.max(axial);
                *vertex = target;
            }
        }
    }
    let axial_span = if min_axial.is_finite() && max_axial.is_finite() {
        max_axial - min_axial
    } else {
        0.0
    };
    Ok((
        out,
        MeshPullReport {
            mode,
            vertices: mesh.vertices.len(),
            max_displacement_mm: max_displacement,
            max_radial_error_mm: max_radial_error,
            axial_span_mm: axial_span,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::sphere;

    fn rail() -> Rail {
        Rail::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 10.0).unwrap()
    }

    #[test]
    fn a_rail_projects_a_point_radially() {
        let rail = rail();
        let projection = rail.project([14.0, 3.0, 2.5]).unwrap();
        assert!((projection.radial_mm - (14.0_f64 * 14.0 + 9.0).sqrt()).abs() < 1.0e-9);
        assert!((projection.point[2] - 0.0).abs() < 1.0e-9);
        assert!((projection.axial_mm - 2.5).abs() < 1.0e-9);
        assert!((norm(sub(projection.point, [0.0, 0.0, 0.0])) - 10.0).abs() < 1.0e-9);
    }

    #[test]
    fn a_rigid_pull_lands_on_the_rail_and_reports_the_move() {
        let rail = rail();
        let result = pull_to_rail([14.0, 0.0, 2.0], [0.0, 0.0, 1.0], &rail, RailPullMode::Rigid)
            .unwrap();
        assert!(result.is_on_rail(&rail, 1.0e-9));
        assert!((result.displacement_mm - 4.0_f64.hypot(2.0)).abs() < 1.0e-9);
        assert!((result.radial_gap_mm - 4.0).abs() < 1.0e-9);
    }

    #[test]
    fn a_deform_pull_keeps_the_axial_height() {
        let rail = rail();
        let result = pull_to_rail([14.0, 0.0, 2.0], [0.0, 0.0, 1.0], &rail, RailPullMode::Deform)
            .unwrap();
        assert!((result.origin[2] - 2.0).abs() < 1.0e-9);
        assert!((result.displacement_mm - 4.0).abs() < 1.0e-9);
        assert!(result.is_on_rail(&rail, 1.0e-9));
    }

    #[test]
    fn a_point_on_the_rail_axis_is_a_typed_refusal() {
        let rail = rail();
        assert_eq!(
            rail.project([0.0, 0.0, 5.0]).err(),
            Some(Error::Invalid("rail projection on axis"))
        );
    }

    #[test]
    fn a_range_excludes_a_pull_outside_the_window() {
        let rail = Rail::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 10.0)
            .unwrap()
            .with_range(0.0, 45.0)
            .unwrap();
        assert!(rail.project([10.0, 1.0, 0.0]).is_ok());
        assert_eq!(
            rail.project([-10.0, 0.0, 0.0]).err(),
            Some(Error::Invalid("rail range"))
        );
    }

    #[test]
    fn a_rigid_mesh_pull_preserves_the_shape() {
        let mesh = sphere([14.0, 0.0, 2.0], 2.0, 24).unwrap();
        let before = mesh.clone();
        let rail = rail();
        let (pulled, report) = pull_mesh(&mesh, &rail, RailPullMode::Rigid).unwrap();
        assert_eq!(pulled.vertices.len(), before.vertices.len());
        let delta = sub(pulled.vertices[0], before.vertices[0]);
        for (a, b) in pulled.vertices.iter().zip(&before.vertices) {
            assert!(norm(sub(sub(*a, *b), delta)) < 1.0e-9);
        }
        assert!(report.max_displacement_mm > 0.0);
    }

    #[test]
    fn a_deform_mesh_pull_puts_every_vertex_on_the_cylinder() {
        let mesh = sphere([14.0, 0.0, 2.0], 2.0, 24).unwrap();
        let rail = rail();
        let (pulled, report) = pull_mesh(&mesh, &rail, RailPullMode::Deform).unwrap();
        assert_eq!(pulled.vertices.len(), mesh.vertices.len());
        for vertex in &pulled.vertices {
            let relative = sub(*vertex, rail.center);
            let axial = dot(relative, rail.axis);
            let radial = norm(sub(relative, mul(rail.axis, axial)));
            assert!((radial - 10.0).abs() < 1.0e-9);
        }
        assert!(report.max_radial_error_mm < 1.0e-9);
        assert!(report.axial_span_mm > 0.0);
    }
}
