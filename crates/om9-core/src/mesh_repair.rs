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
    // An open rim can be intentional. Reduction preserves the input domain.
    let (vertices, faces) = collapse_to(
        &mesh,
        target_faces,
        preserve_boundary,
        feature_angle_deg,
        10,
        &BTreeSet::new(),
        &Default::default(),
        |_, _, _, _| {},
    )?;
    report.faces_after = faces.len();
    Ok((compact(vertices, faces), report))
}

pub mod reduction;

fn collapse_to(
    mesh: &Mesh,
    target_faces: usize,
    preserve_boundary: bool,
    feature_angle_deg: f64,
    accuracy: u8,
    seams: &BTreeSet<u32>,
    cancel: &crate::transaction::Cancellation,
    mut accepted: impl FnMut(&[Vec<u32>], u32, u32, f64),
) -> Result<(Vec<Point3>, Vec<Vec<u32>>)> {
    let features = feature_edges(mesh, feature_angle_deg);
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
        if cancel.is_canceled() {
            return Err(Error::Canceled);
        }
        let mut counts: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for face in &faces {
            for index in 0..face.len() {
                let a = face[index];
                let b = face[(index + 1) % face.len()];
                let key = if a < b { (a, b) } else { (b, a) };
                *counts.entry(key).or_default() += 1;
            }
        }
        let mut boundary = BTreeMap::<u32, Vec<u32>>::new();
        let mut fixed = BTreeSet::new();
        for (&(a, b), &count) in &counts {
            if count == 1 {
                boundary.entry(a).or_default().push(b);
                boundary.entry(b).or_default().push(a);
            } else if features.contains(&(a, b)) || count > 2 {
                fixed.extend([a, b]);
            }
        }
        if preserve_boundary {
            for (&index, neighbors) in &boundary {
                if !straight_rim_vertex(&vertices, index, neighbors) {
                    fixed.insert(index);
                }
            }
        }
        let mut candidates = Vec::new();
        for (&(a, b), &count) in &counts {
            if count > 2
                || seams.contains(&a)
                || seams.contains(&b)
                || (fixed.contains(&a) && fixed.contains(&b))
            {
                continue;
            }
            // A chord between rim vertices must never pinch the outline.
            let a_on_rim = boundary.contains_key(&a);
            let b_on_rim = boundary.contains_key(&b);
            if a_on_rim && b_on_rim && count != 1 {
                continue;
            }
            let (keep, drop) = if fixed.contains(&b) || (!fixed.contains(&a) && b_on_rim) {
                (b, a)
            } else {
                (a, b)
            };
            if preserve_boundary && count != 1 && boundary.contains_key(&drop) {
                continue;
            }
            let position = if fixed.contains(&keep)
                || (preserve_boundary && boundary.contains_key(&keep))
            {
                vertices[keep as usize]
            } else {
                std::array::from_fn(|i| (vertices[a as usize][i] + vertices[b as usize][i]) / 2.0)
            };
            let cost = quadrics[a as usize]
                .add(quadrics[b as usize])
                .evaluate(position);
            candidates.push((cost, keep, drop, position));
        }
        // Fast mode tests local batches in index order. Accurate mode compares
        // all admissible edges and tests the least costly contraction first.
        let batch = if accuracy == 10 {
            candidates.len().max(1)
        } else {
            1usize << (accuracy - 1)
        };
        for chunk in candidates.chunks_mut(batch) {
            chunk.sort_by(|left, right| left.0.total_cmp(&right.0));
        }
        let before_normals = face_normals(&vertices, &faces);
        let mut collapsed = false;
        for (_, keep, drop, position) in candidates {
            if cancel.is_canceled() {
                return Err(Error::Canceled);
            }
            let old = std::mem::replace(&mut vertices[keep as usize], position);
            if let Some(trial) = collapse_faces(&vertices, &faces, &before_normals, keep, drop) {
                accepted(&faces, keep, drop, if position == old { 0.0 } else { 0.5 });
                quadrics[keep as usize] = quadrics[keep as usize].add(quadrics[drop as usize]);
                faces = trial;
                collapsed = true;
                break;
            }
            vertices[keep as usize] = old;
        }
        if !collapsed {
            break;
        }
    }
    Ok((vertices, faces))
}

fn straight_rim_vertex(vertices: &[Point3], index: u32, neighbors: &[u32]) -> bool {
    let &[a, b] = neighbors else { return false };
    let p = vertices[index as usize];
    let u: Point3 = std::array::from_fn(|i| vertices[a as usize][i] - p[i]);
    let v: Point3 = std::array::from_fn(|i| vertices[b as usize][i] - p[i]);
    let dot: f64 = (0..3).map(|i| u[i] * v[i]).sum();
    let squared = |x: Point3| x.iter().map(|v| v * v).sum::<f64>();
    dot < 0.0 && dot * dot >= (1.0 - 1e-12) * squared(u) * squared(v)
}

fn collapse_faces(
    vertices: &[Point3],
    faces: &[Vec<u32>],
    before: &[Option<Point3>],
    keep: u32,
    drop: u32,
) -> Option<Vec<Vec<u32>>> {
    let mut trial = Vec::with_capacity(faces.len());
    let mut seen = BTreeSet::new();
    let mut edges = BTreeMap::new();
    let mut fan = BTreeMap::<u32, Vec<u32>>::new();
    for (face, old_normal) in faces.iter().zip(before) {
        let mut out: Vec<_> = face
            .iter()
            .map(|&i| if i == drop { keep } else { i })
            .collect();
        out.dedup();
        if out.len() > 1 && out.first() == out.last() {
            out.pop();
        }
        // An incident triangle disappears as its common edge contracts.
        if out.len() < 3 {
            continue;
        }
        let mut key = out.clone();
        key.sort_unstable();
        if key.windows(2).any(|v| v[0] == v[1]) || !seen.insert(key) {
            return None;
        }
        if let Some(pos) = out.iter().position(|v| *v == keep) {
            let a = out[(pos + out.len() - 1) % out.len()];
            let b = out[(pos + 1) % out.len()];
            fan.entry(a).or_default().push(b);
            fan.entry(b).or_default().push(a);
            let (new, _) = normal(vertices, &out)?;
            let old = (*old_normal)?;
            if (0..3).map(|i| new[i] * old[i]).sum::<f64>() <= 0.0 {
                return None;
            }
        }
        for (&a, &b) in out.iter().zip(out.iter().cycle().skip(1)) {
            let edge = edges.entry((a.min(b), a.max(b))).or_insert((0usize, 0i32));
            edge.0 += 1;
            edge.1 += if a < b { 1 } else { -1 };
            if edge.0 > 2 || (edge.0 == 2 && edge.1 != 0) {
                return None;
            }
        }
        trial.push(out);
    }
    // The faces around the surviving vertex must form one disk or half disk.
    let mut pending = vec![*fan.keys().next()?];
    let mut visited = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if visited.insert(id) {
            pending.extend(&fan[&id]);
        }
    }
    (visited.len() == fan.len()).then_some(trial)
}
