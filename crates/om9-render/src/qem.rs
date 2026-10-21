//! Quadric error metric decimation (Garland & Heckbert, SIGGRAPH 1997).
//!
//! Vertex clustering cannot push a torus down to a few dozen triangles without losing its shape; a
//! quadric edge collapse can, because it moves the surviving vertex to the point that minimises the
//! squared distance to the original surface rather than to a grid cell centre.
//!
//! Each vertex carries a 4x4 symmetric quadric (stored as the ten distinct coefficients) accumulated from
//! the planes of its incident faces, plus a heavy boundary quadric so an open edge does not collapse into
//! nothing. An edge `(a, b)` costs `v^T (Qa + Qb) v` at the optimal position `v`; the cheapest edge is
//! collapsed first, and the touched edges are re-queued.

use crate::MeshData;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

/// Ten coefficients of a symmetric 4x4 quadric, in the order (a², ab, ac, ad, b², bc, bd, c², cd, d²).
type Quadric = [f64; 10];

const ZERO_QUADRIC: Quadric = [0.0; 10];

fn add_quadric(left: &Quadric, right: &Quadric) -> Quadric {
    let mut sum = [0.0; 10];
    for index in 0..10 {
        sum[index] = left[index] + right[index];
    }
    sum
}

fn plane_quadric(normal: [f64; 3], offset: f64, weight: f64) -> Quadric {
    let [a, b, c] = normal;
    let d = offset;
    [
        weight * a * a,
        weight * a * b,
        weight * a * c,
        weight * a * d,
        weight * b * b,
        weight * b * c,
        weight * b * d,
        weight * c * c,
        weight * c * d,
        weight * d * d,
    ]
}

fn quadric_error(quadric: &Quadric, point: [f64; 3]) -> f64 {
    let [x, y, z] = point;
    let q = quadric;
    q[0] * x * x
        + 2.0 * q[1] * x * y
        + 2.0 * q[2] * x * z
        + 2.0 * q[3] * x
        + q[4] * y * y
        + 2.0 * q[5] * y * z
        + 2.0 * q[6] * y
        + q[7] * z * z
        + 2.0 * q[8] * z
        + q[9]
}

/// The point that minimises the quadric, or the midpoint when the system is singular.
fn optimal_point(quadric: &Quadric, fallback: [f64; 3]) -> [f64; 3] {
    let q = quadric;
    let a = [[q[0], q[1], q[2]], [q[1], q[4], q[5]], [q[2], q[5], q[7]]];
    let b = [-q[3], -q[6], -q[8]];
    let determinant = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    if determinant.abs() < 1e-12 {
        return fallback;
    }
    let solve = |column: usize| {
        let mut m = a;
        for row in 0..3 {
            m[row][column] = b[row];
        }
        let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        det / determinant
    };
    let point = [solve(0), solve(1), solve(2)];
    if point.iter().all(|value| value.is_finite()) {
        point
    } else {
        fallback
    }
}

#[derive(Clone)]
struct Candidate {
    error: f64,
    keep: u32,
    drop: u32,
    position: [f64; 3],
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.error.total_cmp(&other.error) == Ordering::Equal
            && self.keep == other.keep
            && self.drop == other.drop
    }
}
impl Eq for Candidate {}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // `BinaryHeap` is a max-heap, so the cheapest edge must compare as smaller.
        other
            .error
            .total_cmp(&self.error)
            .then_with(|| self.keep.cmp(&other.keep))
            .then_with(|| self.drop.cmp(&other.drop))
    }
}

/// Simplifies `mesh` until it has at most `target_triangles` faces, preserving open boundaries.
pub fn simplify(mesh: &MeshData, target_triangles: usize) -> MeshData {
    simplify_with_error(mesh, target_triangles).0
}

/// Same as [`simplify`], and also reports the largest geometric error introduced, in model units.
///
/// The quadric value of a collapse is a squared distance, so the reported error is its square root: the
/// worst distance a point of the original surface may have moved.
pub fn simplify_with_error(mesh: &MeshData, target_triangles: usize) -> (MeshData, f32) {
    simplify_with_error_locked(mesh, target_triangles, None)
}

/// Same as [`simplify_with_error`], but vertices marked in `locked` never move and never collapse.
/// Jewellery use: open boundaries, cluster-group borders, sharp gem facets and thin prong vertices.
pub fn simplify_with_error_locked(
    mesh: &MeshData,
    target_triangles: usize,
    locked: Option<&[bool]>,
) -> (MeshData, f32) {
    let triangle_count = mesh.indices.len() / 3;
    if target_triangles >= triangle_count {
        return (mesh.clone(), 0.0);
    }
    let mut worst_error = 0.0f64;
    let mut positions: Vec<[f64; 3]> = mesh
        .positions
        .iter()
        .map(|point| [point[0] as f64, point[1] as f64, point[2] as f64])
        .collect();
    let mut faces: Vec<Option<[u32; 3]>> = mesh
        .indices
        .chunks(3)
        .filter(|chunk| chunk.len() == 3)
        .map(|chunk| Some([chunk[0], chunk[1], chunk[2]]))
        .collect();
    let mut quadrics = vec![ZERO_QUADRIC; positions.len()];
    let mut adjacency: Vec<HashSet<u32>> = vec![HashSet::new(); positions.len()];
    for (face_index, face) in faces.iter().enumerate() {
        let Some(face) = face else { continue };
        let [a, b, c] = *face;
        let pa = positions[a as usize];
        let pb = positions[b as usize];
        let pc = positions[c as usize];
        let normal = cross(sub(pb, pa), sub(pc, pa));
        let length = dot(normal, normal).sqrt();
        if length < 1e-12 {
            continue;
        }
        let unit = [normal[0] / length, normal[1] / length, normal[2] / length];
        let offset = -dot(unit, pa);
        // Weight by the triangle area: a large face constrains the surface more than a sliver.
        let area = 0.5 * length;
        let plane = plane_quadric(unit, offset, area);
        for vertex in [a, b, c] {
            quadrics[vertex as usize] = add_quadric(&quadrics[vertex as usize], &plane);
        }
        for (left, right) in [(a, b), (b, c), (c, a)] {
            adjacency[left as usize].insert(face_index as u32);
            adjacency[right as usize].insert(face_index as u32);
        }
    }
    // Boundary edges get a plane perpendicular to their face: collapsing across an open edge would
    // otherwise be nearly free and would eat the silhouette.
    let mut edge_faces: std::collections::HashMap<(u32, u32), Vec<u32>> =
        std::collections::HashMap::new();
    for (face_index, face) in faces.iter().enumerate() {
        let Some(face) = face else { continue };
        for (left, right) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            let key = if left < right {
                (left, right)
            } else {
                (right, left)
            };
            edge_faces.entry(key).or_default().push(face_index as u32);
        }
    }
    for (edge, owners) in &edge_faces {
        if owners.len() != 1 {
            continue;
        }
        let (a, b) = *edge;
        let pa = positions[a as usize];
        let pb = positions[b as usize];
        let Some(face) = faces[owners[0] as usize] else {
            continue;
        };
        let pc = positions[face
            .iter()
            .copied()
            .find(|vertex| *vertex != a && *vertex != b)
            .unwrap_or(face[0]) as usize];
        let normal = cross(sub(pb, pa), sub(pc, pa));
        let length = dot(normal, normal).sqrt();
        if length < 1e-12 {
            continue;
        }
        let face_normal = [normal[0] / length, normal[1] / length, normal[2] / length];
        let edge_direction = sub(pb, pa);
        let boundary_normal = cross(edge_direction, face_normal);
        let length = dot(boundary_normal, boundary_normal).sqrt();
        if length < 1e-12 {
            continue;
        }
        let unit = [
            boundary_normal[0] / length,
            boundary_normal[1] / length,
            boundary_normal[2] / length,
        ];
        let plane = plane_quadric(unit, -dot(unit, pa), 100.0);
        quadrics[a as usize] = add_quadric(&quadrics[a as usize], &plane);
        quadrics[b as usize] = add_quadric(&quadrics[b as usize], &plane);
    }

    // Vertex adjacency, maintained through collapses, for the link condition.
    let mut vertex_adjacency: Vec<HashSet<u32>> = vec![HashSet::new(); positions.len()];
    for face in faces.iter().flatten() {
        for (left, right) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            vertex_adjacency[left as usize].insert(right);
            vertex_adjacency[right as usize].insert(left);
        }
    }
    let mut heap = BinaryHeap::new();
    let mut queued: HashSet<(u32, u32)> = HashSet::new();
    for edge in edge_faces.keys() {
        queue_edge(
            &mut heap,
            &mut queued,
            &quadrics,
            &positions,
            edge.0,
            edge.1,
        );
    }
    let mut alive = triangle_count;
    let mut removed = vec![false; positions.len()];
    while alive > target_triangles {
        let Some(candidate) = heap.pop() else { break };
        if removed[candidate.keep as usize] || removed[candidate.drop as usize] {
            continue;
        }
        let keep = candidate.keep;
        let drop = candidate.drop;
        if locked.is_some_and(|locked| {
            locked.get(keep as usize).copied().unwrap_or(false)
                || locked.get(drop as usize).copied().unwrap_or(false)
        }) {
            continue;
        }
        let drop_faces: Vec<u32> = adjacency[drop as usize].iter().copied().collect();
        if drop_faces.is_empty() {
            removed[drop as usize] = true;
            continue;
        }
        // Link condition: the only shared neighbours may be the vertices opposite the edge in its two
        // incident triangles. Collapsing otherwise creates a non-manifold vertex or a hole.
        let mut shared: Vec<u32> = vertex_adjacency[drop as usize]
            .intersection(&vertex_adjacency[keep as usize])
            .copied()
            .collect();
        shared.sort_unstable();
        let mut opposite: Vec<u32> = Vec::new();
        for face_index in &drop_faces {
            if let Some(face) = faces[*face_index as usize]
                && face.contains(&keep)
                && let Some(other) = face
                    .iter()
                    .copied()
                    .find(|vertex| *vertex != keep && *vertex != drop)
            {
                opposite.push(other);
            }
        }
        opposite.sort_unstable();
        opposite.dedup();
        if shared != opposite {
            continue;
        }
        // Flip prevention: every surviving face must keep facing the same way.
        let mut flipped = false;
        for face_index in &drop_faces {
            let Some(before) = faces[*face_index as usize] else {
                continue;
            };
            let old = triangle_normal(&positions, before);
            let mut after = before;
            for vertex in after.iter_mut() {
                if *vertex == drop {
                    *vertex = keep;
                }
            }
            if after[0] == after[1] || after[1] == after[2] || after[0] == after[2] {
                continue; // collapses away, no normal to compare
            }
            let new = triangle_normal(&positions, after);
            if old.iter().zip(&new).map(|(a, b)| a * b).sum::<f64>() < 0.0 {
                flipped = true;
                break;
            }
        }
        if flipped {
            continue;
        }
        worst_error = worst_error.max(candidate.error.sqrt());
        positions[keep as usize] = candidate.position;
        for face_index in &drop_faces {
            let Some(face) = faces[*face_index as usize].as_mut() else {
                continue;
            };
            for vertex in face.iter_mut() {
                if *vertex == drop {
                    *vertex = keep;
                }
            }
            let [a, b, c] = *face;
            let degenerate = a == b || b == c || a == c;
            if degenerate {
                faces[*face_index as usize] = None;
                alive -= 1;
            } else {
                adjacency[keep as usize].insert(*face_index);
            }
        }
        // Move the adjacency from the dropped vertex to the kept one.
        let neighbours: Vec<u32> = vertex_adjacency[drop as usize].iter().copied().collect();
        for neighbour in &neighbours {
            vertex_adjacency[*neighbour as usize].remove(&drop);
            vertex_adjacency[*neighbour as usize].insert(keep);
            vertex_adjacency[keep as usize].insert(*neighbour);
        }
        vertex_adjacency[keep as usize].remove(&drop);
        vertex_adjacency[drop as usize].clear();
        vertex_adjacency[keep as usize].remove(&keep);
        adjacency[drop as usize].clear();
        removed[drop as usize] = true;
        quadrics[keep as usize] = add_quadric(&quadrics[keep as usize], &quadrics[drop as usize]);
        // Re-queue the edges around the kept vertex; stale entries are skipped when popped.
        let neighbours: Vec<u32> = adjacency[keep as usize]
            .iter()
            .filter_map(|face_index| faces[*face_index as usize])
            .flat_map(|face| face.into_iter())
            .filter(|vertex| *vertex != keep)
            .collect();
        for neighbour in neighbours {
            let key = if keep < neighbour {
                (keep, neighbour)
            } else {
                (neighbour, keep)
            };
            if queued.insert(key) {
                queue_edge(&mut heap, &mut queued, &quadrics, &positions, key.0, key.1);
            }
        }
        queued.remove(&(keep.min(drop), keep.max(drop)));
    }
    let mut out_positions = Vec::new();
    let mut remap: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut out_indices = Vec::new();
    for face in faces.iter().flatten() {
        for vertex in face {
            let next = *remap.entry(*vertex).or_insert_with(|| {
                out_positions.push([
                    positions[*vertex as usize][0] as f32,
                    positions[*vertex as usize][1] as f32,
                    positions[*vertex as usize][2] as f32,
                ]);
                (out_positions.len() - 1) as u32
            });
            out_indices.push(next);
        }
    }
    (
        MeshData::new(out_positions, out_indices),
        worst_error as f32,
    )
}

fn queue_edge(
    heap: &mut BinaryHeap<Candidate>,
    _queued: &mut HashSet<(u32, u32)>,
    quadrics: &[Quadric],
    positions: &[[f64; 3]],
    a: u32,
    b: u32,
) {
    let pa = positions[a as usize];
    let pb = positions[b as usize];
    let midpoint = [
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ];
    let combined = add_quadric(&quadrics[a as usize], &quadrics[b as usize]);
    let mut position = optimal_point(&combined, midpoint);
    // An unconstrained minimiser can land far outside the edge and turn two small faces into one huge
    // triangle, which then costs more fill than the detail it removed. Clamp to the edge's box.
    let length = dot(sub(pb, pa), sub(pb, pa)).sqrt().max(1e-9);
    for axis in 0..3 {
        let low = pa[axis].min(pb[axis]) - length * 0.5;
        let high = pa[axis].max(pb[axis]) + length * 0.5;
        position[axis] = position[axis].clamp(low, high);
    }
    let error = quadric_error(&combined, position).max(0.0);
    heap.push(Candidate {
        error,
        keep: a,
        drop: b,
        position,
    });
}

/// Memoryless simplification (Lindstrom & Turk 1998): run QEM in passes, rebuilding the quadrics from
/// the *current* mesh each pass instead of carrying the original ones. Carrying old quadrics is what makes
/// a 1997 collapse drift when the ratio is aggressive; recomputing them keeps the result closer to the
/// surface at the same triangle count.
pub fn simplify_memoryless(mesh: &MeshData, target_triangles: usize, passes: usize) -> MeshData {
    let mut current = mesh.clone();
    let passes = passes.max(1);
    let start = current.triangle_count();
    for pass in 0..passes {
        let remaining = passes - pass;
        let step_target = if remaining <= 1 {
            target_triangles
        } else {
            // Aim above the final target and let the last pass land on it.
            (target_triangles as f64 * (1.0 + 0.6 * (remaining - 1) as f64)).round() as usize
        };
        if current.triangle_count() <= step_target {
            continue;
        }
        let _ = start;
        current = simplify(&current, step_target);
    }
    current
}

/// Signed volume of a closed mesh, for error comparisons between simplifiers.
pub fn signed_volume(mesh: &MeshData) -> f64 {
    let mut volume = 0.0;
    for triangle in mesh.indices.chunks(3) {
        if triangle.len() < 3 {
            continue;
        }
        let a = mesh.positions[triangle[0] as usize];
        let b = mesh.positions[triangle[1] as usize];
        let c = mesh.positions[triangle[2] as usize];
        let a = [a[0] as f64, a[1] as f64, a[2] as f64];
        let b = [b[0] as f64, b[1] as f64, b[2] as f64];
        let c = [c[0] as f64, c[1] as f64, c[2] as f64];
        volume += dot(a, cross(b, c)) / 6.0;
    }
    volume.abs()
}

/// Total surface area, the other cheap error proxy.
pub fn surface_area(mesh: &MeshData) -> f64 {
    let mut area = 0.0;
    for triangle in mesh.indices.chunks(3) {
        if triangle.len() < 3 {
            continue;
        }
        let a = mesh.positions[triangle[0] as usize];
        let b = mesh.positions[triangle[1] as usize];
        let c = mesh.positions[triangle[2] as usize];
        let a = [a[0] as f64, a[1] as f64, a[2] as f64];
        let b = [b[0] as f64, b[1] as f64, b[2] as f64];
        let c = [c[0] as f64, c[1] as f64, c[2] as f64];
        let normal = cross(sub(b, a), sub(c, a));
        area += 0.5 * dot(normal, normal).sqrt();
    }
    area
}

fn triangle_normal(positions: &[[f64; 3]], face: [u32; 3]) -> [f64; 3] {
    let a = positions[face[0] as usize];
    let b = positions[face[1] as usize];
    let c = positions[face[2] as usize];
    cross(sub(b, a), sub(c, a))
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube() -> MeshData {
        MeshData::new(
            vec![
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
            vec![
                0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 2, 3, 7, 2, 7, 6, 1, 2, 6, 1,
                6, 5, 0, 4, 7, 0, 7, 3,
            ],
        )
    }

    #[test]
    fn a_target_at_or_above_the_input_leaves_the_mesh_alone() {
        let source = cube();
        let result = simplify(&source, 12);
        assert_eq!(result, source);
    }

    #[test]
    fn collapapsing_keeps_every_index_valid() {
        let source = cube();
        let result = simplify(&source, 6);
        assert!(result.triangle_count() <= 6);
        for index in &result.indices {
            assert!((*index as usize) < result.positions.len());
        }
        for triangle in result.indices.chunks(3) {
            assert!(
                triangle[0] != triangle[1]
                    && triangle[1] != triangle[2]
                    && triangle[0] != triangle[2]
            );
        }
    }

    #[test]
    fn a_dense_sphere_keeps_its_extent_when_decimated_hard() {
        // A UV sphere: 16 x 8 quads, enough for the quadric to have something to work with.
        let (rings, segments, radius) = (8usize, 16usize, 1.0f32);
        let mut positions = Vec::new();
        for ring in 0..=rings {
            let phi = std::f32::consts::PI * ring as f32 / rings as f32;
            for segment in 0..segments {
                let theta = 2.0 * std::f32::consts::PI * segment as f32 / segments as f32;
                positions.push([
                    radius * phi.sin() * theta.cos(),
                    radius * phi.sin() * theta.sin(),
                    radius * phi.cos(),
                ]);
            }
        }
        let mut indices = Vec::new();
        for ring in 0..rings {
            for segment in 0..segments {
                let a = (ring * segments + segment) as u32;
                let b = (ring * segments + (segment + 1) % segments) as u32;
                let c = ((ring + 1) * segments + segment) as u32;
                let d = ((ring + 1) * segments + (segment + 1) % segments) as u32;
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
        let source = MeshData::new(positions, indices);
        let result = simplify(&source, 64);
        assert!(
            result.triangle_count() <= 64,
            "got {} triangles",
            result.triangle_count()
        );
        assert!(result.triangle_count() > 0);
        let extent = |mesh: &MeshData| {
            mesh.positions
                .iter()
                .flat_map(|point| point.iter())
                .fold(0.0f32, |value, coordinate| value.max(coordinate.abs()))
        };
        let (before, after) = (extent(&source), extent(&result));
        assert!(
            (before - after).abs() < 0.2,
            "extent changed from {before} to {after}"
        );
    }
}

#[cfg(test)]
mod comparison {
    use super::*;
    use crate::lod;

    fn torus(u: usize, v: usize) -> MeshData {
        let (major, minor) = (4.0f64, 1.2f64);
        let mut positions = Vec::new();
        for i in 0..=v {
            let phi = 2.0 * std::f64::consts::PI * i as f64 / v as f64;
            for j in 0..=u {
                let theta = 2.0 * std::f64::consts::PI * j as f64 / u as f64;
                let r = major + minor * phi.cos();
                positions.push([
                    (r * theta.cos()) as f32,
                    (r * theta.sin()) as f32,
                    (minor * phi.sin()) as f32,
                ]);
            }
        }
        let cols = u + 1;
        let mut indices = Vec::new();
        for i in 0..v {
            for j in 0..u {
                let a = (i * cols + j) as u32;
                let b = a + 1;
                let c = a + cols as u32;
                let d = c + 1;
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
        MeshData::new(positions, indices)
    }

    #[test]
    fn the_three_simplifiers_are_compared_on_the_same_ring() {
        let source = torus(48, 16);
        let (area0, volume0) = (surface_area(&source), signed_volume(&source));
        eprintln!(
            "source: {} triangles, area {area0:.4}, volume {volume0:.4}",
            source.triangle_count()
        );
        for target in [500usize, 200, 80, 30] {
            let started = std::time::Instant::now();
            let qem = simplify(&source, target);
            let qem_time = started.elapsed().as_secs_f64() * 1000.0;

            let started = std::time::Instant::now();
            let memoryless = simplify_memoryless(&source, target, 3);
            let memoryless_time = started.elapsed().as_secs_f64() * 1000.0;

            // The clustering cell that lands near the same triangle count, found by bisection on the
            // radius-normalised cell, so the comparison is at equal cost rather than equal parameter.
            let radius = 5.2f32;
            let mut low = 0.0f32;
            let mut high = radius;
            let mut clustered = source.clone();
            for _ in 0..18 {
                let middle = (low + high) * 0.5;
                let candidate = lod::decimate(&source, middle);
                clustered = candidate.clone();
                if candidate.triangle_count() > target {
                    low = middle;
                } else {
                    high = middle;
                }
            }
            let report = |label: &str, mesh: &MeshData, millis: f64| {
                let area_error = ((surface_area(mesh) - area0) / area0 * 100.0).abs();
                let volume_error = ((signed_volume(mesh) - volume0) / volume0 * 100.0).abs();
                eprintln!(
                    "target={target:3} {label:12} tris={:5} time={millis:7.2} ms area_err={area_error:6.2}% volume_err={volume_error:6.2}%",
                    mesh.triangle_count()
                );
            };
            report("clustering", &clustered, 0.0);
            report("qem", &qem, qem_time);
            report("memoryless", &memoryless, memoryless_time);
        }
    }
}

#[cfg(test)]
mod lock_tests {
    use super::*;

    fn grid(side: u32) -> MeshData {
        let mut positions = Vec::new();
        for y in 0..=side {
            for x in 0..=side {
                positions.push([x as f32, y as f32, 0.0]);
            }
        }
        let mut indices = Vec::new();
        let stride = side + 1;
        for y in 0..side {
            for x in 0..side {
                let a = y * stride + x;
                let b = a + 1;
                let c = a + stride;
                let d = c + 1;
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
        MeshData::new(positions, indices)
    }

    #[test]
    fn locked_vertices_never_move_or_collapse() {
        let side = 10;
        let mesh = grid(side);
        let stride = side + 1;
        let mut locked = vec![false; (stride * stride) as usize];
        for y in 0..=side {
            for x in 0..=side {
                if x == 0 || y == 0 || x == side || y == side {
                    locked[(y * stride + x) as usize] = true;
                }
            }
        }
        let (out, _) = simplify_with_error_locked(&mesh, 20, Some(&locked));
        assert!(out.indices.len() / 3 < mesh.indices.len() / 3);
        for (index, is_locked) in locked.iter().enumerate() {
            if !is_locked {
                continue;
            }
            let point = mesh.positions[index];
            assert!(
                out.positions.iter().any(|other| {
                    (other[0] - point[0]).abs() < 1e-6
                        && (other[1] - point[1]).abs() < 1e-6
                        && (other[2] - point[2]).abs() < 1e-6
                }),
                "locked boundary vertex {point:?} disappeared"
            );
        }
    }
}

/// Vertices that jewellery simplification must keep: open boundaries and edges whose face normals
/// differ by more than `sharp_degrees`. Gem facet rims and prong silhouettes sit on these edges.
pub fn feature_locks(mesh: &MeshData, sharp_degrees: f64) -> Vec<bool> {
    let mut locked = vec![false; mesh.positions.len()];
    let mut edges: std::collections::HashMap<(u32, u32), Vec<u32>> = std::collections::HashMap::new();
    let mut normals: Vec<Option<[f64; 3]>> = Vec::new();
    for (face_index, chunk) in mesh.indices.chunks(3).enumerate() {
        if chunk.len() != 3 {
            continue;
        }
        let (a, b, c) = (chunk[0], chunk[1], chunk[2]);
        let pa = mesh.positions[a as usize].map(|v| v as f64);
        let pb = mesh.positions[b as usize].map(|v| v as f64);
        let pc = mesh.positions[c as usize].map(|v| v as f64);
        let n = cross(sub(pb, pa), sub(pc, pa));
        let len = dot(n, n).sqrt();
        normals.push((len > 1e-12).then(|| [n[0] / len, n[1] / len, n[2] / len]));
        for (l, r) in [(a, b), (b, c), (c, a)] {
            let key = if l < r { (l, r) } else { (r, l) };
            edges.entry(key).or_default().push(face_index as u32);
        }
    }
    let cos_limit = sharp_degrees.to_radians().cos();
    for ((a, b), owners) in &edges {
        let sharp = owners.len() == 1
            || owners.iter().any(|left| {
                owners.iter().any(|right| {
                    let (Some(x), Some(y)) = (normals[*left as usize], normals[*right as usize]) else {
                        return false;
                    };
                    dot(x, y) < cos_limit
                })
            });
        if sharp {
            locked[*a as usize] = true;
            locked[*b as usize] = true;
        }
    }
    locked
}

#[cfg(test)]
mod feature_lock_tests {
    use super::*;

    #[test]
    fn boundaries_and_sharp_edges_are_locked() {
        let mesh = MeshData::new(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![0, 1, 2, 0, 2, 3, 0, 1, 4],
        );
        let locked = feature_locks(&mesh, 30.0);
        assert!(locked[0] && locked[1] && locked[2] && locked[3], "open boundary");
        assert!(locked[4], "sharp fold at the raised triangle");
    }
}

#[cfg(test)]
mod accuracy_tests {
    use super::*;

    #[test]
    fn a_feature_locked_lod_keeps_area_and_volume() {
        let mesh = crate::shapes::sphere(32, 16, 1.0);
        let locked = feature_locks(&mesh, 30.0);
        let (lod, _) = simplify_with_error_locked(&mesh, mesh.triangle_count() / 2, Some(&locked));
        assert!(lod.triangle_count() < mesh.triangle_count());
        let area_error = ((surface_area(&lod) - surface_area(&mesh)) / surface_area(&mesh) * 100.0).abs();
        let volume_error = ((signed_volume(&lod) - signed_volume(&mesh)) / signed_volume(&mesh) * 100.0).abs();
        eprintln!("locked LOD tris {} -> {}, area {area_error:.2}%, volume {volume_error:.2}%", mesh.triangle_count(), lod.triangle_count());
        assert!(area_error < 2.0, "area error {area_error:.2}%");
        assert!(volume_error < 4.0, "volume error {volume_error:.2}%");
    }
}

#[cfg(test)]
mod prong_accuracy_tests {
    use super::*;

    fn cylinder(sides: u32) -> MeshData {
        let rings = 5u32;
        let mut positions = Vec::new();
        for ring in 0..rings {
            let z = ring as f32 * 0.5;
            for i in 0..sides {
                let a = i as f32 / sides as f32 * std::f32::consts::TAU;
                positions.push([0.2 * a.cos(), 0.2 * a.sin(), z]);
            }
        }
        let bottom = positions.len() as u32;
        positions.push([0.0, 0.0, 0.0]);
        positions.push([0.0, 0.0, 2.0]);
        let mut indices = Vec::new();
        for ring in 0..rings - 1 {
            let base = ring * sides;
            let next = base + sides;
            for i in 0..sides {
                let j = (i + 1) % sides;
                indices.extend_from_slice(&[base + i, base + j, next + i, base + j, next + j, next + i]);
            }
        }
        for i in 0..sides {
            let j = (i + 1) % sides;
            indices.extend_from_slice(&[bottom, j, i]);
            let top = (rings - 1) * sides;
            indices.extend_from_slice(&[bottom + 1, top + i, top + j]);
        }
        MeshData::new(positions, indices)
    }

    #[test]
    fn a_thin_prong_keeps_its_silhouette_after_lod() {
        let mesh = cylinder(24);
        let locked = feature_locks(&mesh, 30.0);
        let (lod, _) = simplify_with_error_locked(&mesh, mesh.triangle_count() / 2, Some(&locked));
        assert!(lod.triangle_count() < mesh.triangle_count());
        let bounds = |positions: &[[f32; 3]]| {
            positions.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(mut lo, mut hi), p| {
                for axis in 0..3 {
                    lo[axis] = lo[axis].min(p[axis]);
                    hi[axis] = hi[axis].max(p[axis]);
                }
                (lo, hi)
            })
        };
        let (lo0, hi0) = bounds(&mesh.positions);
        let (lo1, hi1) = bounds(&lod.positions);
        for axis in 0..3 {
            assert!((lo0[axis] - lo1[axis]).abs() < 1e-4, "prong min {axis} moved");
            assert!((hi0[axis] - hi1[axis]).abs() < 1e-4, "prong max {axis} moved");
        }
    }
}
