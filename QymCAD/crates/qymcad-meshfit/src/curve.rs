//! THE CURVE AN EDGE RUNS ALONG, from the two surfaces it parts.
//!
//! Two planes meet in a straight line. Two surfaces turning about one axis - a cylinder, a cone, a sphere, a torus, or a
//! plane square to the axis - meet in circles, and a circle is found in the cut through the axis, where each surface is
//! a curve of height along the axis against distance from it: a rounding meets a corner's sphere in a circle of its own
//! radius, a cap meets a wall in the wall's circle. A plane along a cylinder's axis meets it in straight lines, one where
//! the plane is tangent. Every curve found is checked against the edge's own points; where none fits - two surfaces
//! meeting in a curve that is no line and no circle, or meeting at a graze - the edge keeps its points.

use crate::boundary::settle;
use crate::{Boundaries, Region, Surface, Tolerance, OPEN};
use nalgebra::{Matrix2, Vector2, Vector3};

/// The curve an edge runs along.
#[derive(Clone, Debug, PartialEq)]
pub enum Curve {
    /// `dir` is a unit vector, turned the way the edge's points run.
    Line { point: [f64; 3], dir: [f64; 3] },
    /// In the plane through `center` square to the unit `axis`.
    Circle { center: [f64; 3], axis: [f64; 3], radius: f64 },
    /// In the plane through `center` square to the unit `normal`: half-axis `major` along the unit `major_dir`, `minor`
    /// across it. What a plane cuts a cylinder in at a slant.
    Ellipse { center: [f64; 3], normal: [f64; 3], major_dir: [f64; 3], major: f64, minor: f64 },
    /// No line or circle fits, or the edge runs along a border: the edge's points, within the tolerance of both surfaces.
    Points(Vec<[f64; 3]>),
}

/// Axes this close are one axis: 0.2 deg, twice what a fitted axis is off on the kernel's meshes.
const SAME_AXIS: f64 = 0.0035;

/// The curve of every edge of `b`.
pub fn curves(b: &Boundaries, found: &[Region], tol: &Tolerance) -> Vec<Curve> {
    b.edges
        .iter()
        .map(|e| {
            let surface = |f: u32| if f == OPEN { None } else { found[f as usize].surface.as_ref() };
            let (Some(sa), Some(sb)) = (surface(e.faces[0]), surface(e.faces[1])) else { return Curve::Points(e.points.clone()) };
            let near = 10.0 * tol.distance + 0.02 * size(sa).max(size(sb));
            let circle = common_axis(sa, sb, near).and_then(|(o, u)| circle_about(sa, sb, o, u, &e.points));
            [circle, line(sa, sb, &e.points), ellipse(sa, sb)]
                .into_iter()
                .flatten()
                .find(|c| fits(c, &e.points, e.ends.is_none(), tol, sa, sb))
                // THE POINTS AS THE MESH HAS THEM: each is a corner of triangles of both faces, so it lies within the
                // tolerance of both already, and putting it on them moved it by at most four tolerances at the cost
                // of 5.3 s on the 225 154 triangles of `seam_test_object` - as long as splitting it into regions takes
                .unwrap_or_else(|| Curve::Points(e.points.clone()))
        })
        .collect()
}

fn v(a: [f64; 3]) -> Vector3<f64> {
    Vector3::from(a)
}

fn arr(x: Vector3<f64>) -> [f64; 3] {
    [x.x, x.y, x.z]
}

/// The radius a surface curves by, or zero for a plane and a cone, whose curving varies.
fn size(s: &Surface) -> f64 {
    match *s {
        Surface::Cylinder { radius, .. } | Surface::Sphere { radius, .. } => radius,
        Surface::Torus { major, .. } => major,
        Surface::Plane { .. } | Surface::Cone { .. } | Surface::Free { .. } | Surface::Spline(_) | Surface::Helix { .. } | Surface::RoundHelix { .. } | Surface::Coil { .. } => 0.0,
    }
}

/// The axis `a` and `b` both turn about, as a point on it and a unit direction: a plane's normal where it stands
/// square to the other's axis, a sphere's centre where it lies on it, and a plane and a sphere about the plane's normal.
fn common_axis(a: &Surface, b: &Surface, near: f64) -> Option<(Vector3<f64>, Vector3<f64>)> {
    let own = |s: &Surface| match *s {
        Surface::Cylinder { point, axis, .. } => Some((v(point), v(axis))),
        Surface::Cone { apex, axis, .. } => Some((v(apex), v(axis))),
        Surface::Torus { center, axis, .. } => Some((v(center), v(axis))),
        // a helix and a coil meet what turns about their axis in a curve that winds, not in a circle
        Surface::Plane { .. } | Surface::Sphere { .. } | Surface::Free { .. } | Surface::Spline(_) | Surface::Helix { .. } | Surface::RoundHelix { .. } | Surface::Coil { .. } => None,
    };
    let off = |q: Vector3<f64>, (o, u): (Vector3<f64>, Vector3<f64>)| {
        let d = q - o;
        (d - u * d.dot(&u)).norm()
    };
    let parallel = |x: Vector3<f64>, y: Vector3<f64>| x.dot(&y).abs() > SAME_AXIS.cos();
    let with = |s: &Surface, ax: (Vector3<f64>, Vector3<f64>)| match *s {
        // the plane's normal is taken for the axis: it is fitted on a whole face, the axis on a curved one
        Surface::Plane { normal, .. } => parallel(v(normal), ax.1).then(|| (ax.0, if v(normal).dot(&ax.1) < 0.0 { -v(normal) } else { v(normal) })),
        Surface::Sphere { center, .. } => (off(v(center), ax) < near).then_some(ax),
        _ => own(s).filter(|&(p, u)| parallel(u, ax.1) && off(p, ax) < near).map(|_| ax),
    };
    match (own(a), own(b)) {
        (Some(ax), _) => with(b, ax),
        (None, Some(ax)) => with(a, ax),
        (None, None) => match (a, b) {
            (Surface::Plane { normal, .. }, Surface::Sphere { center, .. }) | (Surface::Sphere { center, .. }, Surface::Plane { normal, .. }) => Some((v(*center), v(*normal))),
            (Surface::Sphere { center: c1, .. }, Surface::Sphere { center: c2, .. }) => (v(*c2) - v(*c1)).try_normalize(1e-9).map(|u| (v(*c1), u)),
            _ => None,
        },
    }
}

/// How far the point at height `h` along the axis `(o, u)` and distance `rho` from it lies off `s`, in the cut through
/// the axis.
fn profile(s: &Surface, o: Vector3<f64>, u: Vector3<f64>, h: f64, rho: f64) -> f64 {
    let height = |q: [f64; 3]| u.dot(&(v(q) - o));
    match *s {
        Surface::Plane { point, .. } => h - height(point),
        Surface::Cylinder { radius, .. } => rho - radius,
        Surface::Cone { apex, axis, half_angle } => rho * half_angle.cos() - (h - height(apex)) * u.dot(&v(axis)).signum() * half_angle.sin(),
        Surface::Sphere { center, radius } => (h - height(center)).hypot(rho) - radius,
        Surface::Torus { center, major, minor, .. } => (rho - major).hypot(h - height(center)) - minor,
        Surface::Free { .. } | Surface::Spline(_) | Surface::Helix { .. } | Surface::RoundHelix { .. } | Surface::Coil { .. } => f64::INFINITY,
    }
}

/// The circle `a` and `b` meet in about the axis `(o, u)`, nearest the edge's points.
fn circle_about(a: &Surface, b: &Surface, o: Vector3<f64>, u: Vector3<f64>, pts: &[[f64; 3]]) -> Option<Curve> {
    let (mut h, mut rho) = (0.0, 0.0);
    for &q in pts {
        let d = v(q) - o;
        let t = u.dot(&d);
        h += t;
        rho += (d - u * t).norm();
    }
    let n = pts.len().max(1) as f64;
    let x = settle(&[h / n, rho / n], |x| vec![profile(a, o, u, x[0], x[1]), profile(b, o, u, x[0], x[1])]);
    (x[1] > 1e-9).then(|| Curve::Circle { center: arr(o + u * x[0]), axis: arr(u), radius: x[1] })
}

/// The straight line two planes meet in, or a plane along a cylinder's axis, nearest the edge's points.
fn line(a: &Surface, b: &Surface, pts: &[[f64; 3]]) -> Option<Curve> {
    let (first, last) = (v(*pts.first()?), v(*pts.last()?));
    let mid = pts.iter().fold(Vector3::zeros(), |m, &q| m + v(q)) / pts.len() as f64;
    let along = |d: Vector3<f64>| if d.dot(&(last - first)) < 0.0 { -d } else { d };
    match (a, b) {
        (Surface::Plane { point: pa, normal: na }, Surface::Plane { point: pb, normal: nb }) => {
            let (na, nb) = (v(*na), v(*nb));
            let dir = na.cross(&nb).try_normalize(1e-9)?;
            let gram = Matrix2::new(na.dot(&na), na.dot(&nb), na.dot(&nb), nb.dot(&nb));
            let w = gram.lu().solve(&Vector2::new(na.dot(&(mid - v(*pa))), nb.dot(&(mid - v(*pb)))))?;
            Some(Curve::Line { point: arr(mid - na * w[0] - nb * w[1]), dir: arr(along(dir)) })
        }
        (Surface::Plane { point, normal }, Surface::Cylinder { point: c, axis, radius }) | (Surface::Cylinder { point: c, axis, radius }, Surface::Plane { point, normal }) => {
            let (n, u) = (v(*normal), v(*axis));
            if n.dot(&u).abs() > SAME_AXIS.sin() {
                return None;
            }
            let d = n.dot(&(v(*c) - v(*point)));
            let foot = v(*c) - n * d;
            let at = if d.abs() >= *radius {
                foot
            } else {
                let w = u.cross(&n).try_normalize(1e-12)? * (radius * radius - d * d).sqrt();
                if (foot + w - mid).norm() < (foot - w - mid).norm() {
                    foot + w
                } else {
                    foot - w
                }
            };
            Some(Curve::Line { point: arr(at), dir: arr(along(u)) })
        }
        _ => None,
    }
}

/// THE ELLIPSE A PLANE CUTS A CYLINDER IN AT A SLANT: its centre where the axis meets the plane, the minor half-axis the
/// radius, square to both the axis and the normal, the major one the radius over the cosine between them. A plane square
/// to the axis gives a circle and one along it lines; both are found elsewhere, and neither is an ellipse here.
fn ellipse(a: &Surface, b: &Surface) -> Option<Curve> {
    let (plane, cylinder) = match (a, b) {
        (Surface::Plane { .. }, Surface::Cylinder { .. }) => (a, b),
        (Surface::Cylinder { .. }, Surface::Plane { .. }) => (b, a),
        _ => return None,
    };
    let (Surface::Plane { point: p, normal }, Surface::Cylinder { point: c, axis, radius }) = (plane, cylinder) else { return None };
    let (n, u) = (v(*normal), v(*axis));
    let cos = n.dot(&u);
    if cos.abs() < 1e-3 || cos.abs() > SAME_AXIS.cos() {
        return None;
    }
    let center = v(*c) + u * ((v(*p) - v(*c)).dot(&n) / cos);
    let small = u.cross(&n).try_normalize(1e-12)?;
    let big = n.cross(&small).normalize();
    Some(Curve::Ellipse { center: arr(center), normal: arr(n), major_dir: arr(big), major: radius / cos.abs(), minor: *radius })
}

/// Whether every point of the edge lies on `c`: within ten tolerances, and wider by the room two tangent surfaces leave
/// - a point within the tolerance of both stands up to sqrt(2 x tolerance x radius) off the line they touch along.
fn fits(c: &Curve, pts: &[[f64; 3]], closed: bool, tol: &Tolerance, a: &Surface, b: &Surface) -> bool {
    let reach = 10.0 * tol.distance + 2.0 * (2.0 * tol.distance * size(a).max(size(b))).sqrt();
    pts.iter().all(|&q| match c {
        Curve::Line { point, dir } => {
            let d = v(q) - v(*point);
            let t = d.dot(&v(*dir));
            (d.norm_squared() - t * t).max(0.0).sqrt() <= reach
        }
        Curve::Circle { center, axis, radius } => {
            let d = v(q) - v(*center);
            let h = d.dot(&v(*axis));
            h.hypot((d - v(*axis) * h).norm() - radius) <= reach
        }
        Curve::Ellipse { center, normal, major_dir, major, minor } => {
            let (d, n, big) = (v(q) - v(*center), v(*normal), v(*major_dir));
            let small = n.cross(&big);
            let (x, y) = (d.dot(&big), d.dot(&small));
            let t = (y / minor).atan2(x / major);
            d.dot(&n).hypot((x - major * t.cos()).hypot(y - minor * t.sin())) <= reach
        }
        Curve::Points(_) => true,
    }) && shown(c, pts, closed)
}

/// Whether the edge's points show the arc: no step between two neighbours turns more than a right angle about its centre.
/// Any circle passes through two points - the straight side of a triangle, 160 deg round a circle of 2.56 where a plane
/// met a sphere on a gear of `cube_gears`, was taken for the arc and made its face 3 400 times its triangle. A mesh drawn
/// from a real arc steps far shorter: 16 deg on `cylinder.3mf`.
fn shown(c: &Curve, pts: &[[f64; 3]], closed: bool) -> bool {
    let (center, axis) = match c {
        Curve::Circle { center, axis, .. } => (center, axis),
        Curve::Ellipse { center, normal, .. } => (center, normal),
        _ => return true,
    };
    if pts.len() < 2 {
        return false;
    }
    let (o, u) = (v(*center), v(*axis));
    let radial = |q: [f64; 3]| {
        let d = v(q) - o;
        d - u * d.dot(&u)
    };
    let wrap = closed.then(|| (pts[pts.len() - 1], pts[0]));
    pts.windows(2).map(|w| (w[0], w[1])).chain(wrap).all(|(p, q)| radial(p).angle(&radial(q)) <= std::f64::consts::FRAC_PI_2)
}
