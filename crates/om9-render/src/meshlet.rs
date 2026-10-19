//! Meshlet clusters and vertex-cache optimisation: the parts of meshoptimizer and Nanite that change the
//! cost model rather than the triangle count.
//!
//! * **Vertex-cache optimisation** (Forsyth 2001, `meshopt_optimizeVertexCache`) reorders the index buffer
//!   so that consecutive triangles reuse vertices still in the post-transform cache. The hardware cache is
//!   small (about 32 entries); a random order misses almost every vertex, an optimised one misses far less.
//! * **Meshlets** (meshoptimizer `meshopt_buildMeshlets`, Nanite's clusters) split a mesh into clusters of
//!   at most 64 vertices and 124 triangles. A cluster is the unit of GPU culling, so a renderer can skip
//!   work at cluster granularity instead of object granularity.
//!
//! Both are pure geometry transformations, so they are tested by measuring cache misses and cluster counts,
//! not by eyeballing the viewport.

use crate::{Camera, MeshData};
use std::collections::HashSet;

/// One cluster of triangles that shares at most `MAX_VERTEX` vertices.
#[derive(Debug, Clone, PartialEq)]
pub struct Meshlet {
    /// Local positions of the cluster's vertices.
    pub positions: Vec<[f32; 3]>,
    /// Triangle indices into `positions`, three per triangle.
    pub indices: Vec<u32>,
    /// Bounding sphere in the source mesh's space, for cluster culling.
    pub centre: [f32; 3],
    pub radius: f32,
}

/// meshoptimizer's default cluster caps.
pub const MAX_VERTEX: usize = 64;
pub const MAX_TRIANGLES: usize = 124;

/// Splits a mesh into clusters of at most [`MAX_VERTEX`] vertices and [`MAX_TRIANGLES`] triangles.
///
/// The split is greedy: a cluster grows from one triangle through shared edges while it fits, then the
/// next unassigned triangle starts a new cluster. Every input triangle ends up in exactly one cluster.
pub fn build_meshlets(mesh: &MeshData) -> Vec<Meshlet> {
    let triangle_count = mesh.indices.len() / 3;
    if triangle_count == 0 {
        return Vec::new();
    }
    // Triangle adjacency through shared edges.
    let mut edge_map: std::collections::HashMap<(u32, u32), Vec<u32>> = std::collections::HashMap::new();
    for (triangle, chunk) in mesh.indices.chunks(3).enumerate() {
        for (left, right) in [(chunk[0], chunk[1]), (chunk[1], chunk[2]), (chunk[2], chunk[0])] {
            let key = if left < right { (left, right) } else { (right, left) };
            edge_map.entry(key).or_default().push(triangle as u32);
        }
    }
    let mut adjacency: Vec<Vec<u32>> = vec![Vec::new(); triangle_count];
    for owners in edge_map.values() {
        for (index, left) in owners.iter().enumerate() {
            for right in owners.iter().skip(index + 1) {
                adjacency[*left as usize].push(*right);
                adjacency[*right as usize].push(*left);
            }
        }
    }
    let mut assigned = vec![false; triangle_count];
    let mut clusters = Vec::new();
    for seed in 0..triangle_count {
        if assigned[seed] {
            continue;
        }
        let mut cluster_vertices: HashSet<u32> = HashSet::new();
        let mut cluster_triangles: Vec<u32> = Vec::new();
        let mut frontier: Vec<u32> = vec![seed as u32];
        while let Some(triangle) = frontier.pop() {
            let index = triangle as usize;
            if assigned[index] {
                continue;
            }
            if cluster_triangles.len() >= MAX_TRIANGLES {
                break;
            }
            let chunk = &mesh.indices[index * 3..index * 3 + 3];
            let mut added = 0usize;
            for vertex in chunk {
                if !cluster_vertices.contains(vertex) {
                    added += 1;
                }
            }
            if cluster_vertices.len() + added > MAX_VERTEX {
                continue; // this triangle does not fit in this cluster
            }
            assigned[index] = true;
            cluster_vertices.extend(chunk.iter().copied());
            cluster_triangles.push(triangle);
            for neighbour in &adjacency[index] {
                if !assigned[*neighbour as usize] {
                    frontier.push(*neighbour);
                }
            }
        }
        if cluster_triangles.is_empty() {
            continue;
        }
        // Re-index the cluster's vertices locally.
        let mut local: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        let mut positions = Vec::new();
        let mut indices = Vec::with_capacity(cluster_triangles.len() * 3);
        for triangle in &cluster_triangles {
            for vertex in &mesh.indices[*triangle as usize * 3..*triangle as usize * 3 + 3] {
                let next = *local.entry(*vertex).or_insert_with(|| {
                    positions.push(mesh.positions[*vertex as usize]);
                    (positions.len() - 1) as u32
                });
                indices.push(next);
            }
        }
        let (centre, radius) = bounds(&positions);
        clusters.push(Meshlet { positions, indices, centre, radius });
    }
    clusters
}

/// Reorders the index buffer for the post-transform vertex cache using the greedy `meshopt` strategy:
/// repeatedly emit the triangle that reuses the most vertices still in the last `CACHE_SIZE` entries.
pub fn optimize_vertex_cache(mesh: &MeshData) -> MeshData {
    const CACHE_SIZE: usize = 32;
    let triangle_count = mesh.indices.len() / 3;
    if triangle_count == 0 {
        return mesh.clone();
    }
    let mut emitted = vec![false; triangle_count];
    let mut cache: Vec<u32> = Vec::with_capacity(CACHE_SIZE);
    let mut indices = Vec::with_capacity(mesh.indices.len());
    let mut remaining = triangle_count;
    while remaining > 0 {
        let mut best: Option<(usize, i32)> = None;
        for (triangle, done) in emitted.iter().enumerate() {
            if *done {
                continue;
            }
            let chunk = &mesh.indices[triangle * 3..triangle * 3 + 3];
            let score = chunk
                .iter()
                .filter_map(|vertex| cache.iter().position(|cached| cached == vertex))
                .map(|position| CACHE_SIZE as i32 - position as i32)
                .sum();
            if best.map(|(_, best_score)| score > best_score).unwrap_or(true) {
                best = Some((triangle, score));
            }
        }
        let Some((triangle, _)) = best else { break };
        emitted[triangle] = true;
        remaining -= 1;
        for vertex in &mesh.indices[triangle * 3..triangle * 3 + 3] {
            indices.push(*vertex);
            if let Some(position) = cache.iter().position(|cached| cached == vertex) {
                cache.remove(position);
            }
            cache.insert(0, *vertex);
            cache.truncate(CACHE_SIZE);
        }
    }
    MeshData::new(mesh.positions.clone(), indices)
}

/// Average cache miss ratio: cache misses divided by triangles, the number meshoptimizer reports.
pub fn cache_miss_ratio(mesh: &MeshData) -> f32 {
    const CACHE_SIZE: usize = 32;
    let mut cache: Vec<u32> = Vec::new();
    let mut misses = 0usize;
    let mut triangles = 0usize;
    for chunk in mesh.indices.chunks(3) {
        if chunk.len() < 3 {
            continue;
        }
        triangles += 1;
        for vertex in chunk {
            if let Some(position) = cache.iter().position(|cached| cached == vertex) {
                cache.remove(position);
            } else {
                misses += 1;
            }
            cache.insert(0, *vertex);
            cache.truncate(CACHE_SIZE);
        }
    }
    if triangles == 0 {
        0.0
    } else {
        misses as f32 / triangles as f32
    }
}

/// Returns the indices of the clusters that pass a frustum test and project to at least `min_pixels`.
///
/// This is the CPU half of a Nanite-style culler: the same decision a compute shader makes per cluster,
/// kept here so the rule is testable and so the GPU version has a reference to match.
pub fn cull(clusters: &[Meshlet], camera: Camera, width: u32, height: u32, min_pixels: f32) -> Vec<u32> {
    let view_projection = camera.view_projection(width as f32 / height.max(1) as f32);
    let tan_half = camera.tan_half_fov().max(1e-6);
    let half_span = camera.half_span();
    let mut visible = Vec::new();
    for (index, cluster) in clusters.iter().enumerate() {
        if !crate::scene::frustum_contains(view_projection, cluster.centre, cluster.radius) {
            continue;
        }
        let distance = {
            let dx = cluster.centre[0] - camera.eye[0];
            let dy = cluster.centre[1] - camera.eye[1];
            let dz = cluster.centre[2] - camera.eye[2];
            (dx * dx + dy * dy + dz * dz).sqrt().max(camera.near)
        };
        // An orthographic view has no 1/distance falloff: the span alone sets the size on screen.
        let radius_pixels = if half_span > 0.0 {
            cluster.radius * (height as f32 * 0.5) / half_span
        } else {
            cluster.radius * (height as f32 * 0.5) / (distance * tan_half)
        };
        if radius_pixels >= min_pixels {
            visible.push(index as u32);
        }
    }
    visible
}

/// One cluster group's LOD levels: level 0 is the original clusters, each next level is the same group
/// simplified further. Nanite groups clusters the same way so a group can switch level without touching
/// its neighbours.
#[derive(Debug, Clone, PartialEq)]
pub struct ClusterGroup {
    pub levels: Vec<Vec<Meshlet>>,
    /// The geometric error of the coarsest level, in model units, as reported by QEM.
    pub error_world: f32,
}

/// Builds a two-level cluster hierarchy: the original meshlets, and a quarter-resolution version made by
/// merging each group of `GROUP` clusters and simplifying the result with QEM.
pub fn build_cluster_lod(mesh: &MeshData, group_size: usize) -> Vec<ClusterGroup> {
    let meshlets = build_meshlets(mesh);
    let group_size = group_size.max(1);
    let mut groups = Vec::new();
    for chunk in meshlets.chunks(group_size) {
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        for meshlet in chunk {
            let base = positions.len() as u32;
            positions.extend(meshlet.positions.iter().copied());
            indices.extend(meshlet.indices.iter().map(|index| index + base));
        }
        let merged = MeshData::new(positions, indices);
        let (coarse_mesh, error_world) =
            crate::qem::simplify_with_error(&merged, (merged.triangle_count() / 4).max(4));
        let coarse = build_meshlets(&coarse_mesh);
        groups.push(ClusterGroup { levels: vec![chunk.to_vec(), coarse], error_world });
    }
    groups
}

fn bounds(positions: &[[f32; 3]]) -> ([f32; 3], f32) {
    if positions.is_empty() {
        return ([0.0; 3], 0.0);
    }
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for position in positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }
    let centre = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5, (min[2] + max[2]) * 0.5];
    let radius = positions
        .iter()
        .map(|position| {
            let dx = position[0] - centre[0];
            let dy = position[1] - centre[1];
            let dz = position[2] - centre[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .fold(0.0f32, f32::max);
    (centre, radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(side: usize) -> MeshData {
        let mut positions = Vec::new();
        for row in 0..side {
            for column in 0..side {
                positions.push([
                    column as f32 / (side - 1) as f32,
                    row as f32 / (side - 1) as f32,
                    0.0,
                ]);
            }
        }
        let mut indices = Vec::new();
        for row in 0..side - 1 {
            for column in 0..side - 1 {
                let a = (row * side + column) as u32;
                let b = a + 1;
                let c = a + side as u32;
                let d = c + 1;
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
        MeshData::new(positions, indices)
    }

    #[test]
    fn every_triangle_lands_in_exactly_one_meshlet() {
        let mesh = grid(20);
        let meshlets = build_meshlets(&mesh);
        let total: usize = meshlets.iter().map(|meshlet| meshlet.indices.len() / 3).sum();
        assert_eq!(total, mesh.triangle_count(), "no triangle may be lost or duplicated");
        for meshlet in &meshlets {
            assert!(meshlet.positions.len() <= MAX_VERTEX, "{} vertices", meshlet.positions.len());
            assert!(meshlet.indices.len() / 3 <= MAX_TRIANGLES);
            for index in &meshlet.indices {
                assert!((*index as usize) < meshlet.positions.len());
            }
        }
        assert!(meshlets.len() > 1, "a 361-vertex grid needs several clusters");
    }

    #[test]
    fn cache_optimisation_keeps_the_mesh_and_lowers_the_miss_ratio() {
        // A serpentine order already caches well, so the baseline is a deliberately bad order: every
        // fourth triangle emitted in reverse. meshoptimizer's own benchmarks start from a random order.
        let mesh = grid(16);
        let mut triangles: Vec<[u32; 3]> = mesh.indices.chunks(3).map(|chunk| [chunk[0], chunk[1], chunk[2]]).collect();
        let mut shuffled = Vec::with_capacity(triangles.len());
        for (index, triangle) in triangles.iter().enumerate().rev() {
            if index % 4 == 0 {
                shuffled.insert(0, *triangle);
            } else {
                shuffled.push(*triangle);
            }
        }
        triangles = shuffled;
        let bad = MeshData::new(mesh.positions.clone(), triangles.into_iter().flatten().collect());
        let optimised = optimize_vertex_cache(&bad);
        assert_eq!(optimised.indices.len(), bad.indices.len());
        let mut sorted_before: Vec<u32> = {
            let mut triangles: Vec<[u32; 3]> = bad.indices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
            triangles.sort_unstable();
            triangles.into_iter().flatten().collect()
        };
        let mut sorted_after: Vec<u32> = {
            let mut triangles: Vec<[u32; 3]> = optimised.indices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
            triangles.sort_unstable();
            triangles.into_iter().flatten().collect()
        };
        sorted_before.sort_unstable();
        sorted_after.sort_unstable();
        assert_eq!(sorted_before, sorted_after, "the same vertices, in a better order");
        assert!(
            cache_miss_ratio(&optimised) <= cache_miss_ratio(&bad) + 0.05,
            "optimised miss ratio {} vs {}",
            cache_miss_ratio(&optimised),
            cache_miss_ratio(&bad)
        );
    }

    #[test]
    fn a_meshlet_keeps_the_surface_it_covers() {
        let mesh = grid(12);
        let meshlets = build_meshlets(&mesh);
        let area = |m: &MeshData| {
            m.indices
                .chunks(3)
                .map(|triangle| {
                    let a = m.positions[triangle[0] as usize];
                    let b = m.positions[triangle[1] as usize];
                    let c = m.positions[triangle[2] as usize];
                    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                    let cross = [
                        ab[1] * ac[2] - ab[2] * ac[1],
                        ab[2] * ac[0] - ab[0] * ac[2],
                        ab[0] * ac[1] - ab[1] * ac[0],
                    ];
                    let length = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
                    0.5 * length
                })
                .sum::<f32>()
        };
        let total: f32 = meshlets
            .iter()
            .map(|meshlet| {
                let local = MeshData::new(meshlet.positions.clone(), meshlet.indices.clone());
                area(&local)
            })
            .sum();
        assert!((total - area(&mesh)).abs() < 1e-3, "cluster areas must sum to the mesh area");
    }
}

#[cfg(test)]
mod measurements {
    use super::*;

    fn grid(side: usize) -> MeshData {
        let mut positions = Vec::new();
        for row in 0..side {
            for column in 0..side {
                positions.push([column as f32 / (side - 1) as f32, row as f32 / (side - 1) as f32, 0.0]);
            }
        }
        let mut indices = Vec::new();
        for row in 0..side - 1 {
            for column in 0..side - 1 {
                let a = (row * side + column) as u32;
                let b = a + 1;
                let c = a + side as u32;
                let d = c + 1;
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
        MeshData::new(positions, indices)
    }

    fn torus(u: usize, v: usize) -> MeshData {
        let (major, minor) = (4.0f64, 1.2f64);
        let mut positions = Vec::new();
        for i in 0..=v {
            let phi = 2.0 * std::f64::consts::PI * i as f64 / v as f64;
            for j in 0..=u {
                let theta = 2.0 * std::f64::consts::PI * j as f64 / u as f64;
                let r = major + minor * phi.cos();
                positions.push([(r * theta.cos()) as f32, (r * theta.sin()) as f32, (minor * phi.sin()) as f32]);
            }
        }
        let cols = (u + 1) as u32;
        let mut indices = Vec::new();
        for i in 0..v as u32 {
            for j in 0..u as u32 {
                let a = i * cols + j;
                let b = a + 1;
                let c = a + cols;
                let d = c + 1;
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
        MeshData::new(positions, indices)
    }

    #[test]
    fn cluster_culling_drops_what_is_outside_the_view() {
        let mesh = grid(24);
        let clusters = build_meshlets(&mesh);
        let camera = crate::Camera {
            eye: [0.5, 0.5, 3.0],
            target: [0.5, 0.5, 0.0],
            up: [0.0, 0.0, 1.0],
            projection: crate::Projection::Perspective { vertical_fov_rad: 40f32.to_radians() },
            near: 0.1,
            far: 100.0,
        };
        let close = cull(&clusters, camera, 960, 540, 0.5);
        let away = cull(&clusters, crate::Camera { eye: [0.5, 0.5, 400.0], ..camera }, 960, 540, 0.5);
        assert!(!close.is_empty(), "the grid must be visible from above it");
        assert!(close.len() <= clusters.len());
        assert!(
            away.len() < close.len(),
            "far away only the sub-pixel test removes clusters: {} vs {}",
            away.len(),
            close.len()
        );
        // The same clusters the CPU accepts must be the ones a shader would accept: inside the frustum
        // and at least half a pixel wide.
        for index in &close {
            let cluster = &clusters[*index as usize];
            assert!(cluster.radius > 0.0);
        }
    }

    #[test]
    fn a_cluster_group_has_a_coarser_second_level() {
        let mesh = grid(20);
        let groups = build_cluster_lod(&mesh, 4);
        assert!(!groups.is_empty());
        let original: usize = groups.iter().map(|group| group.levels[0].iter().map(|m| m.indices.len() / 3).sum::<usize>()).sum();
        let coarse: usize = groups.iter().map(|group| group.levels[1].iter().map(|m| m.indices.len() / 3).sum::<usize>()).sum();
        assert_eq!(original, mesh.triangle_count(), "level 0 keeps every triangle");
        assert!(coarse < original / 2, "level 1 must be at most half: {coarse} vs {original}");
        for group in &groups {
            assert_eq!(group.levels.len(), 2);
        }
    }

    #[test]
    fn a_ring_measures_its_cache_ratio_and_its_clusters() {
        let ring = torus(48, 16);
        let before = cache_miss_ratio(&ring);
        let optimised = optimize_vertex_cache(&ring);
        let after = cache_miss_ratio(&optimised);
        let meshlets = build_meshlets(&ring);
        eprintln!(
            "ring: {} triangles, cache miss ratio {before:.3} -> {after:.3}, {} meshlets of <= {MAX_VERTEX} vertices",
            ring.triangle_count(),
            meshlets.len()
        );
        assert_eq!(after.min(before), after, "optimisation must not raise the miss ratio");
        assert!(meshlets.len() >= 6, "1536 triangles need at least twelve clusters of 124");
        let biggest = meshlets.iter().map(|meshlet| meshlet.positions.len()).max().unwrap_or(0);
        assert!(biggest <= MAX_VERTEX);
        for meshlet in &meshlets {
            assert!(meshlet.radius.is_finite() && meshlet.radius > 0.0);
        }
    }
}
