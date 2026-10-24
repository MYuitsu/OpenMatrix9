// SPDX-License-Identifier: LGPL-2.1-or-later
//! Mesh cleanup and reduction utilities (M03); checked geometric repair is
//! provided by mesh_csg under the advanced repair owner OM9-L2-10-02.
use crate::{
    Error, Result,
    geometry::{Mesh, Point3},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RepairReport {
    pub welded_vertices: usize,
    pub degenerate_faces: usize,
    pub duplicate_faces: usize,
    pub isolated_vertices: usize,
    pub flipped_faces: usize,
    pub boundary_edges: usize,
    /// Arrangement fragments classified as internal, not intersection-pair counts.
    pub csg_internal_fragments: usize,
    pub csg_split_events: usize,
    pub faces_before: usize,
    pub faces_after: usize,
}

fn normal(vertices: &[Point3], face: &[u32]) -> Option<(Point3, f64)> {
    if face.len() < 3 {
        return None;
    }
    let a = vertices[face[0] as usize];
    let b = vertices[face[1] as usize];
    let c = vertices[face[2] as usize];
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let length = n[0].hypot(n[1]).hypot(n[2]);
    if length <= 1.0e-15 {
        return None;
    }
    let n = [n[0] / length, n[1] / length, n[2] / length];
    Some((n, -(n[0] * a[0] + n[1] * a[1] + n[2] * a[2])))
}

#[derive(Clone, Copy, Default)]
struct Quadric {
    a2: f64,
    ab: f64,
    ac: f64,
    ad: f64,
    b2: f64,
    bc: f64,
    bd: f64,
    c2: f64,
    cd: f64,
    d2: f64,
}

impl Quadric {
    fn plane(normal: Point3, d: f64) -> Self {
        Self {
            a2: normal[0] * normal[0],
            ab: normal[0] * normal[1],
            ac: normal[0] * normal[2],
            ad: normal[0] * d,
            b2: normal[1] * normal[1],
            bc: normal[1] * normal[2],
            bd: normal[1] * d,
            c2: normal[2] * normal[2],
            cd: normal[2] * d,
            d2: d * d,
        }
    }
    fn add(self, other: Self) -> Self {
        Self {
            a2: self.a2 + other.a2,
            ab: self.ab + other.ab,
            ac: self.ac + other.ac,
            ad: self.ad + other.ad,
            b2: self.b2 + other.b2,
            bc: self.bc + other.bc,
            bd: self.bd + other.bd,
            c2: self.c2 + other.c2,
            cd: self.cd + other.cd,
            d2: self.d2 + other.d2,
        }
    }
    fn evaluate(&self, p: Point3) -> f64 {
        let x = p[0];
        let y = p[1];
        let z = p[2];
        self.a2 * x * x
            + 2.0 * self.ab * x * y
            + 2.0 * self.ac * x * z
            + 2.0 * self.ad * x
            + self.b2 * y * y
            + 2.0 * self.bc * y * z
            + 2.0 * self.bd * y
            + self.c2 * z * z
            + 2.0 * self.cd * z
            + self.d2
    }
}

/// Dihedral feature edges: boundary edges and edges whose adjacent face
/// normals differ by more than `angle_deg`.
pub fn feature_edges(mesh: &Mesh, angle_deg: f64) -> BTreeSet<(u32, u32)> {
    let threshold = angle_deg.to_radians().cos();
    let mut edge_normals: BTreeMap<(u32, u32), Vec<Point3>> = BTreeMap::new();
    for face in &mesh.faces {
        let Some((normal, _)) = normal(&mesh.vertices, face) else {
            continue;
        };
        for index in 0..face.len() {
            let a = face[index];
            let b = face[(index + 1) % face.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_normals.entry(key).or_default().push(normal);
        }
    }
    let mut features = BTreeSet::new();
    for (edge, normals) in edge_normals {
        if normals.len() == 1 {
            features.insert(edge);
            continue;
        }
        for left in 0..normals.len() {
            for right in (left + 1)..normals.len() {
                let dot = normals[left][0] * normals[right][0]
                    + normals[left][1] * normals[right][1]
                    + normals[left][2] * normals[right][2];
                if dot < threshold {
                    features.insert(edge);
                }
            }
        }
    }
    features
}
/// Detect intersections beyond shared boundaries, including coplanar overlaps.
/// This diagnostic does not change the mesh; the CSG repair validates its input.
pub fn self_intersections(mesh: &Mesh, max_pairs: usize) -> Vec<(usize, usize)> {
    if max_pairs == 0 {
        return Vec::new();
    }
    let mut triangles = Vec::new();
    for (face_index, face) in mesh.faces.iter().enumerate() {
        for index in 1..face.len().saturating_sub(1) {
            triangles.push((
                face_index,
                mesh.vertices[face[0] as usize],
                mesh.vertices[face[index] as usize],
                mesh.vertices[face[index + 1] as usize],
            ));
        }
    }
    let mut pairs = Vec::new();
    for left in 0..triangles.len() {
        for right in (left + 1)..triangles.len() {
            if triangles[left].0 == triangles[right].0 {
                continue;
            }
            if aabb_overlap(triangles[left], triangles[right])
                && triangles_intersect(triangles[left], triangles[right])
            {
                pairs.push((triangles[left].0, triangles[right].0));
                if pairs.len() >= max_pairs {
                    return pairs;
                }
            }
        }
    }
    pairs
}

fn aabb_overlap(a: (usize, Point3, Point3, Point3), b: (usize, Point3, Point3, Point3)) -> bool {
    let (_, a0, a1, a2) = a;
    let (_, b0, b1, b2) = b;
    for axis in 0..3 {
        let amin = a0[axis].min(a1[axis]).min(a2[axis]);
        let amax = a0[axis].max(a1[axis]).max(a2[axis]);
        let bmin = b0[axis].min(b1[axis]).min(b2[axis]);
        let bmax = b0[axis].max(b1[axis]).max(b2[axis]);
        if amax < bmin || bmax < amin {
            return false;
        }
    }
    true
}

fn triangles_intersect(
    a: (usize, Point3, Point3, Point3),
    b: (usize, Point3, Point3, Point3),
) -> bool {
    let (_, a0, a1, a2) = a;
    let (_, b0, b1, b2) = b;
    let a = [a0, a1, a2];
    let b = [b0, b1, b2];
    let shared: Vec<_> = a.iter().filter(|p| b.contains(p)).copied().collect();
    crate::mesh_csg::improper_intersection(a, b, &shared, 1.0e-9).unwrap_or(true)
}

/// Checked geometric repair. Open or unsupported surfaces return an error;
/// crossing faces are never deleted as a substitute for rebuilding a solid.
pub use crate::mesh_csg::repair_self_intersections;
/// Fill boundary loops with a centroid fan. Loops longer than `max_edges`
/// are left untouched so a repair pass cannot invent large surfaces.
pub fn fill_holes(mesh: &Mesh, max_edges: usize) -> (Mesh, usize) {
    let mut edges: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for face in &mesh.faces {
        for index in 0..face.len() {
            let a = face[index];
            let b = face[(index + 1) % face.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edges.entry(key).or_default() += 1;
        }
    }
    let boundary: Vec<(u32, u32)> = edges
        .iter()
        .filter(|(_, count)| **count == 1)
        .map(|(&(a, b), _)| (a, b))
        .collect();
    let mut next: BTreeMap<u32, u32> = BTreeMap::new();
    for (a, b) in &boundary {
        next.entry(*a).or_insert(*b);
        next.entry(*b).or_insert(*a);
    }
    let mut visited = BTreeSet::new();
    let mut vertices = mesh.vertices.clone();
    let mut faces = mesh.faces.clone();
    let mut filled = 0usize;
    for (start, _) in &boundary {
        if visited.contains(start) {
            continue;
        }
        let mut loop_vertices = Vec::new();
        let mut current = *start;
        while visited.insert(current) {
            loop_vertices.push(current);
            match next.get(&current) {
                Some(value) if !visited.contains(value) => current = *value,
                _ => break,
            }
        }
        if loop_vertices.len() < 3 || loop_vertices.len() > max_edges {
            continue;
        }
        let mut centroid = [0.0f64; 3];
        for index in &loop_vertices {
            let point = vertices[*index as usize];
            centroid[0] += point[0];
            centroid[1] += point[1];
            centroid[2] += point[2];
        }
        let count = loop_vertices.len() as f64;
        centroid = [
            centroid[0] / count,
            centroid[1] / count,
            centroid[2] / count,
        ];
        let center = vertices.len() as u32;
        vertices.push(centroid);
        for index in 0..loop_vertices.len() {
            let a = loop_vertices[index];
            let b = loop_vertices[(index + 1) % loop_vertices.len()];
            faces.push(vec![a, b, center]);
        }
        filled += 1;
    }
    (
        Mesh {
            vertices,
            faces,
            normals: Vec::new(),
            colors: Vec::new(),
            corner_uvs: Vec::new(),
        },
        filled,
    )
}

fn face_normals(vertices: &[Point3], faces: &[Vec<u32>]) -> Vec<Option<Point3>> {
    faces
        .iter()
        .map(|face| normal(vertices, face).map(|(n, _)| n))
        .collect()
}
fn compact(vertices: Vec<Point3>, mut faces: Vec<Vec<u32>>) -> Mesh {
    let mut used = BTreeSet::new();
    for face in &faces {
        used.extend(face.iter().copied());
    }
    let mut remap = BTreeMap::new();
    let mut output = Vec::with_capacity(used.len());
    for index in used {
        remap.insert(index, output.len() as u32);
        output.push(vertices[index as usize]);
    }
    for face in &mut faces {
        for index in face.iter_mut() {
            *index = remap[index];
        }
    }
    Mesh {
        vertices: output,
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    }
}

fn boundary_edges(faces: &[Vec<u32>]) -> usize {
    let mut edges: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for face in faces {
        for index in 0..face.len() {
            let a = face[index];
            let b = face[(index + 1) % face.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edges.entry(key).or_default() += 1;
        }
    }
    edges.values().filter(|count| **count == 1).count()
}

/// Weld vertices, remove degenerate/duplicate faces and isolated vertices,
/// then unify winding across shared edges.
pub fn repair_mesh(mesh: &Mesh, tolerance: f64) -> Result<(Mesh, RepairReport)> {
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(Error::Invalid("mesh repair tolerance"));
    }
    if mesh.vertices.iter().flatten().any(|v| !v.is_finite()) {
        return Err(Error::Invalid("mesh repair coordinates"));
    }
    for face in &mesh.faces {
        if face.len() < 3 {
            return Err(Error::Invalid("mesh face indices"));
        }
        if face
            .iter()
            .any(|index| *index as usize >= mesh.vertices.len())
        {
            return Err(Error::Invalid("mesh face indices"));
        }
    }
    let mut report = RepairReport {
        faces_before: mesh.faces.len(),
        ..RepairReport::default()
    };
    let cell = tolerance.max(1.0e-9);
    let mut welded = BTreeMap::new();
    let mut vertices = Vec::new();
    let mut remap = vec![0u32; mesh.vertices.len()];
    for (index, point) in mesh.vertices.iter().enumerate() {
        let key = (
            (point[0] / cell).round() as i64,
            (point[1] / cell).round() as i64,
            (point[2] / cell).round() as i64,
        );
        remap[index] = *welded.entry(key).or_insert_with(|| {
            vertices.push(*point);
            (vertices.len() - 1) as u32
        });
    }
    report.welded_vertices = mesh.vertices.len().saturating_sub(vertices.len());
    let mut faces: Vec<Vec<u32>> = mesh
        .faces
        .iter()
        .map(|face| face.iter().map(|index| remap[*index as usize]).collect())
        .collect();
    let before = faces.len();
    faces.retain_mut(|face| {
        let mut unique = Vec::new();
        for index in face.iter() {
            if !unique.contains(index) {
                unique.push(*index);
            }
        }
        *face = unique;
        face.len() >= 3 && normal(&vertices, face).is_some()
    });
    report.degenerate_faces = before.saturating_sub(faces.len());
    let before = faces.len();
    let mut seen = BTreeSet::new();
    faces.retain(|face| {
        let mut sorted = face.clone();
        sorted.sort_unstable();
        seen.insert(sorted)
    });
    report.duplicate_faces = before.saturating_sub(faces.len());

    let mut edge_faces: BTreeMap<(u32, u32), Vec<(usize, bool)>> = BTreeMap::new();
    for (face_index, face) in faces.iter().enumerate() {
        for index in 0..face.len() {
            let a = face[index];
            let b = face[(index + 1) % face.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push((face_index, a < b));
        }
    }
    let mut visited = vec![false; faces.len()];
    for start in 0..faces.len() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut queue = VecDeque::from([start]);
        while let Some(current) = queue.pop_front() {
            for index in 0..faces[current].len() {
                let a = faces[current][index];
                let b = faces[current][(index + 1) % faces[current].len()];
                let key = if a < b { (a, b) } else { (b, a) };
                for &(neighbor, direction) in edge_faces.get(&key).into_iter().flatten() {
                    if neighbor == current || visited[neighbor] {
                        continue;
                    }
                    if (a < b) == direction {
                        faces[neighbor].reverse();
                        report.flipped_faces += 1;
                    }
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
    }

    let mut used = BTreeSet::new();
    for face in &faces {
        used.extend(face.iter().copied());
    }
    report.isolated_vertices = vertices.len().saturating_sub(used.len());
    report.boundary_edges = boundary_edges(&faces);
    report.faces_after = faces.len();
    Ok((compact(vertices, faces), report))
}

/// QEM-guided edge collapse. Boundary edges are preserved when requested and
/// collapses that create degenerate faces are rejected.
/// Backward-compatible advanced reduction at a 30-degree feature angle.
pub fn reduce_mesh_advanced(
    mesh: &Mesh,
    target_faces: usize,
    preserve_boundary: bool,
) -> Result<(Mesh, RepairReport)> {
    reduce_mesh_feature_aware(mesh, target_faces, preserve_boundary, 30.0)
}
/// Feature-aware reduction with an explicit dihedral angle.
pub fn reduce_mesh_feature_aware(
    mesh: &Mesh,
    target_faces: usize,
    preserve_boundary: bool,
    feature_angle_deg: f64,
) -> Result<(Mesh, RepairReport)> {
    if target_faces < 1 {
        return Err(Error::Invalid("mesh reduce target"));
    }
    let (mesh, mut report) = repair_mesh(mesh, 1.0e-9)?;
    let mesh = if self_intersections(&mesh, 1).is_empty() {
        mesh
    } else {
        let (repaired, csg) =
            repair_self_intersections(&mesh, crate::mesh_csg::CsgOptions::default())?;
        report.csg_internal_fragments = csg.internal_fragments;
        report.csg_split_events = csg.split_events;
        repaired
    };
    let (mesh, _) = if report.boundary_edges > 0 {
        fill_holes(&mesh, 16)
    } else {
        (mesh, 0)
    };
    let features = feature_edges(&mesh, feature_angle_deg);
    if mesh.faces.len() <= target_faces {
        return Ok((mesh, report));
    }
    let mut vertices = mesh.vertices.clone();
    let mut faces = mesh.faces.clone();
    let mut quadrics = vec![Quadric::default(); vertices.len()];
    for face in &faces {
        if let Some((normal, d)) = normal(&vertices, face) {
            let quadric = Quadric::plane(normal, d);
            for index in face {
                quadrics[*index as usize] = quadrics[*index as usize].add(quadric);
            }
        }
    }
    while faces.len() > target_faces {
        let mut counts: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for face in &faces {
            for index in 0..face.len() {
                let a = face[index];
                let b = face[(index + 1) % face.len()];
                let key = if a < b { (a, b) } else { (b, a) };
                *counts.entry(key).or_default() += 1;
            }
        }
        let mut candidates: Vec<(f64, u32, u32)> = counts
            .iter()
            .filter(|(edge, count)| {
                let (a, b) = **edge;
                (!preserve_boundary || **count > 1) && !features.contains(&(a, b))
            })
            .map(|(&(a, b), _)| {
                let pa = vertices[a as usize];
                let pb = vertices[b as usize];
                let midpoint = [
                    (pa[0] + pb[0]) / 2.0,
                    (pa[1] + pb[1]) / 2.0,
                    (pa[2] + pb[2]) / 2.0,
                ];
                let cost = quadrics[a as usize]
                    .add(quadrics[b as usize])
                    .evaluate(midpoint);
                (cost, a, b)
            })
            .collect();
        candidates.sort_by(|left, right| left.0.total_cmp(&right.0));
        let mut collapsed = false;
        for (_, keep, drop) in candidates {
            let mut trial = faces.clone();
            for face in &mut trial {
                for index in face.iter_mut() {
                    if *index == drop {
                        *index = keep;
                    }
                }
            }
            let before_normals = face_normals(&vertices, &faces);
            let mut valid = true;
            for (face_index, face) in trial.iter().enumerate() {
                let mut unique = Vec::new();
                for index in face.iter() {
                    if !unique.contains(index) {
                        unique.push(*index);
                    }
                }
                if unique.len() < 3 || normal(&vertices, &unique).is_none() {
                    valid = false;
                    break;
                }
                if let (Some(old), Some((new, _))) = (
                    before_normals.get(face_index).copied().flatten(),
                    normal(&vertices, &unique),
                ) && old[0] * new[0] + old[1] * new[1] + old[2] * new[2] < 0.0
                {
                    valid = false;
                    break;
                }
            }
            if !valid {
                continue;
            }
            let pa = vertices[keep as usize];
            let pb = vertices[drop as usize];
            vertices[keep as usize] = [
                (pa[0] + pb[0]) / 2.0,
                (pa[1] + pb[1]) / 2.0,
                (pa[2] + pb[2]) / 2.0,
            ];
            quadrics[keep as usize] = quadrics[keep as usize].add(quadrics[drop as usize]);
            faces = trial;
            let mut seen = BTreeSet::new();
            faces.retain_mut(|face| {
                let mut unique = Vec::new();
                for index in face.iter() {
                    if !unique.contains(index) {
                        unique.push(*index);
                    }
                }
                *face = unique;
                if face.len() < 3 {
                    return false;
                }
                let mut sorted = face.clone();
                sorted.sort_unstable();
                seen.insert(sorted)
            });
            collapsed = true;
            break;
        }
        if !collapsed {
            break;
        }
    }
    if faces.len() > target_faces {
        faces.truncate(target_faces);
    }
    report.faces_after = faces.len();
    Ok((compact(vertices, faces), report))
}
