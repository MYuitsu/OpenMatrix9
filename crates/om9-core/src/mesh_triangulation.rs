// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded triangulation of a single simple planar mesh face.
use crate::{Error, Result, geometry::Point3, transaction::Cancellation};

fn cross2(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
fn sub(a: Point3, b: Point3) -> Point3 { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn dot(a: Point3, b: Point3) -> f64 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }
fn cross(a: Point3, b: Point3) -> Point3 {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}
fn check_cancel(cancel: Option<&Cancellation>) -> Result<()> {
    if cancel.is_some_and(Cancellation::is_canceled) { return Err(Error::Canceled); }
    Ok(())
}
fn spend(work: &mut usize, limit: usize, cancel: Option<&Cancellation>) -> Result<()> {
    *work = work.checked_add(1).ok_or(Error::Overflow("polygon triangulation work"))?;
    if *work & 255 == 0 { check_cancel(cancel)?; }
    if *work > limit { return Err(Error::Budget("polygon triangulation work")); }
    Ok(())
}

/// Preserves face winding and vertex indices. Rejects nonplanar, self-intersecting,
/// degenerate or numerically ambiguous polygons. `work` is shared across faces.
pub fn triangulate_face(points: &[Point3], face: &[u32], tolerance: f64, work: &mut usize, limit: usize) -> Result<Vec<[u32; 3]>> {
    triangulate_face_inner(points, face, tolerance, work, limit, None)
}

/// Cancellable variant; checks the token during the bounded polygon work.
pub fn triangulate_face_cancellable(points: &[Point3], face: &[u32], tolerance: f64, work: &mut usize, limit: usize, cancel: &Cancellation) -> Result<Vec<[u32; 3]>> {
    triangulate_face_inner(points, face, tolerance, work, limit, Some(cancel))
}

fn triangulate_face_inner(points: &[Point3], face: &[u32], tolerance: f64, work: &mut usize, limit: usize, cancel: Option<&Cancellation>) -> Result<Vec<[u32; 3]>> {
    check_cancel(cancel)?;
    if face.len() < 3 || !tolerance.is_finite() || tolerance <= 0.0 { return Err(Error::Invalid("polygon face")); }
    let mut p = Vec::new();
    p.try_reserve_exact(face.len()).map_err(|_| Error::Budget("polygon vertices"))?;
    for (i, &id) in face.iter().enumerate() {
        if i & 255 == 0 { check_cancel(cancel)?; }
        let vertex = *points.get(id as usize).ok_or(Error::Invalid("polygon face index"))?;
        if vertex.iter().any(|value| !value.is_finite()) { return Err(Error::Invalid("polygon vertex")); }
        p.push(vertex);
    }
    let origin = p[0];
    let mut normal = [0.; 3];
    for i in 1..p.len()-1 {
        if i & 255 == 0 { check_cancel(cancel)?; }
        normal = cross(sub(p[i], origin), sub(p[i+1], origin));
        if dot(normal, normal).sqrt() > tolerance * tolerance { break; }
    }
    let norm = dot(normal, normal).sqrt();
    if !norm.is_finite() || norm <= tolerance * tolerance { return Err(Error::Invalid("degenerate polygon face")); }
    for (i, &point) in p.iter().enumerate() {
        if i & 255 == 0 { check_cancel(cancel)?; }
        if dot(sub(point, origin), normal).abs() > tolerance * norm { return Err(Error::Unsupported("nonplanar polygon face")); }
    }
    let axis = (0..3).max_by(|&a, &b| normal[a].abs().total_cmp(&normal[b].abs())).unwrap();
    let project = |point: Point3| match axis { 0 => [point[1]-origin[1], point[2]-origin[2]], 1 => [point[0]-origin[0], point[2]-origin[2]], _ => [point[0]-origin[0], point[1]-origin[1]] };
    let mut q = Vec::new();
    q.try_reserve_exact(p.len()).map_err(|_| Error::Budget("projected polygon vertices"))?;
    for (i, &point) in p.iter().enumerate() {
        if i & 255 == 0 { check_cancel(cancel)?; }
        q.push(project(point));
    }
    let mut scale: f64 = 0.0;
    let mut area = 0.0;
    for i in 0..q.len() {
        if i & 255 == 0 { check_cancel(cancel)?; }
        let a = q[i]; let b = q[(i+1)%q.len()];
        let length = (b[0]-a[0]).hypot(b[1]-a[1]);
        if !length.is_finite() || length <= tolerance { return Err(Error::Invalid("polygon short edge")); }
        scale = scale.max(length);
        area += a[0]*b[1]-a[1]*b[0];
    }
    let eps = tolerance * scale;
    if !eps.is_finite() || !area.is_finite() || area.abs() <= eps { return Err(Error::Invalid("degenerate polygon face")); }
    let sign = area.signum();
    // Reject crossings and touching non-adjacent edges before ear clipping.
    for i in 0..q.len() { for j in i+1..q.len() {
        if j == i+1 || (i == 0 && j == q.len()-1) { continue; }
        spend(work, limit, cancel)?;
        let (a,b,c,d) = (q[i], q[(i+1)%q.len()], q[j], q[(j+1)%q.len()]);
        let (x,y,z,w) = (cross2(a,b,c), cross2(a,b,d), cross2(c,d,a), cross2(c,d,b));
        if [x,y,z,w].iter().any(|value| !value.is_finite()) { return Err(Error::Invalid("polygon numeric range")); }
        let overlap = |a: [f64;2], b: [f64;2], c: [f64;2]| c[0] >= a[0].min(b[0])-tolerance && c[0] <= a[0].max(b[0])+tolerance && c[1] >= a[1].min(b[1])-tolerance && c[1] <= a[1].max(b[1])+tolerance;
        let opposite = |a: f64, b: f64| (a > 0.0 && b < 0.0) || (a < 0.0 && b > 0.0);
        if (opposite(x,y) && opposite(z,w)) || (x.abs() <= eps && overlap(a,b,c)) || (y.abs() <= eps && overlap(a,b,d)) || (z.abs() <= eps && overlap(c,d,a)) || (w.abs() <= eps && overlap(c,d,b)) {
            return Err(Error::Invalid("self-intersecting polygon face"));
        }
    }}
    let mut remaining = Vec::new();
    remaining.try_reserve_exact(face.len()).map_err(|_| Error::Budget("polygon indices"))?;
    remaining.extend(0..face.len());
    let mut triangles = Vec::new();
    triangles.try_reserve_exact(face.len()-2).map_err(|_| Error::Budget("polygon triangles"))?;
    while remaining.len() > 3 {
        let mut ear = None;
        for i in 0..remaining.len() {
            spend(work, limit, cancel)?;
            let (a,b,c) = (remaining[(i+remaining.len()-1)%remaining.len()], remaining[i], remaining[(i+1)%remaining.len()]);
            if sign * cross2(q[a], q[b], q[c]) <= eps { continue; }
            let mut blocked = false;
            for &other in &remaining {
                if other == a || other == b || other == c { continue; }
                spend(work, limit, cancel)?;
                if sign*cross2(q[a],q[b],q[other]) >= -eps && sign*cross2(q[b],q[c],q[other]) >= -eps && sign*cross2(q[c],q[a],q[other]) >= -eps { blocked = true; break; }
            }
            if !blocked { ear = Some((i, [face[a], face[b], face[c]])); break; }
        }
        let (index, triangle) = ear.ok_or(Error::Invalid("polygon triangulation failed"))?;
        triangles.push(triangle);
        remaining.remove(index);
    }
    if sign * cross2(q[remaining[0]],q[remaining[1]],q[remaining[2]]) <= eps { return Err(Error::Invalid("polygon degenerate triangle")); }
    triangles.push([face[remaining[0]], face[remaining[1]], face[remaining[2]]]);
    check_cancel(cancel)?;
    Ok(triangles)
}
