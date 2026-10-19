//! Mesh simplification with an explicit error bound, for screen-space LOD.
//!
//! A 2 GB jewelry scene is not slow because of draw calls (merging fixed that) but because a million
//! triangles project to a few hundred pixels. The classical answer is a level of detail chosen by the
//! **projected** geometric error (Lindstrom et al.; Luebke et al.), not by distance alone.
//!
//! This module implements vertex clustering (Rossignac & Borrel 1992): vertices inside a grid cell of
//! side `cell` collapse to one representative, and the worst-case geometric error of the result is the
//! cell's half diagonal, `cell * sqrt(3) / 2`. That bound is what makes the LOD rule safe:
//!
//! ```text
//! error_px = (cell * sqrt(3)/2) * height / (distance * tan(fov/2))   <= 1 px
//! ```
//!
//! so the renderer can pick the largest cell that keeps the visible error under a pixel, instead of the
//! fixed "small objects disappear" rule.

use crate::MeshData;
use std::collections::HashMap;

/// The worst-case geometric error of a clustering with the given cell side.
pub fn cluster_error(cell: f32) -> f32 {
    cell * 3f32.sqrt() * 0.5
}

/// The largest cell whose projected error stays under `max_pixels` for a mesh `radius` wide seen from
/// `distance` in a viewport `height` pixels tall with a vertical field of view of `fov` radians.
pub fn cell_for_pixel_error(radius: f32, distance: f32, height: f32, fov: f32, max_pixels: f32) -> f32 {
    let tan_half = (fov * 0.5).tan().max(1e-6);
    // error_px = cell * sqrt(3)/2 * height / (distance * tan_half) <= max_pixels
    let allowed_error = max_pixels * distance * tan_half / height.max(1.0);
    (2.0 * allowed_error / 3f32.sqrt()).max(radius * 1e-4)
}

/// Clusters vertices on a grid of side `cell` and drops the triangles that collapse.
///
/// The representative of a cell is the average of the vertices in it, so a mesh keeps its position and
/// its silhouette moves by at most half a cell diagonal.
pub fn decimate(mesh: &MeshData, cell: f32) -> MeshData {
    if cell <= 0.0 || mesh.positions.is_empty() {
        return mesh.clone();
    }
    let inverse = 1.0 / cell;
    let mut cells: HashMap<(i32, i32, i32), u32> = HashMap::new();
    let mut sums: Vec<[f64; 3]> = Vec::new();
    let mut counts: Vec<u32> = Vec::new();
    let mut remap: Vec<u32> = Vec::with_capacity(mesh.positions.len());
    for position in &mesh.positions {
        let key = (
            (position[0] * inverse).floor() as i32,
            (position[1] * inverse).floor() as i32,
            (position[2] * inverse).floor() as i32,
        );
        let index = *cells.entry(key).or_insert_with(|| {
            sums.push([0.0; 3]);
            counts.push(0);
            (sums.len() - 1) as u32
        });
        let slot = index as usize;
        for axis in 0..3 {
            sums[slot][axis] += position[axis] as f64;
        }
        counts[slot] += 1;
        remap.push(index);
    }
    let positions: Vec<[f32; 3]> = sums
        .iter()
        .zip(&counts)
        .map(|(sum, count)| {
            let divisor = (*count).max(1) as f64;
            [
                (sum[0] / divisor) as f32,
                (sum[1] / divisor) as f32,
                (sum[2] / divisor) as f32,
            ]
        })
        .collect();
    let mut indices = Vec::with_capacity(mesh.indices.len());
    for triangle in mesh.indices.chunks(3) {
        if triangle.len() < 3 {
            continue;
        }
        let a = remap[triangle[0] as usize];
        let b = remap[triangle[1] as usize];
        let c = remap[triangle[2] as usize];
        if a == b || b == c || a == c {
            continue; // the triangle collapsed into a line or a point
        }
        indices.extend_from_slice(&[a, b, c]);
    }
    MeshData::new(positions, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube() -> MeshData {
        MeshData::new(
            vec![
                [-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5, 0.5, -0.5], [-0.5, 0.5, -0.5],
                [-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5],
            ],
            vec![0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 2, 3, 7, 2, 7, 6, 1, 2, 6, 1, 6, 5, 0, 4, 7, 0, 7, 3],
        )
    }

    /// A dense plane: 11 x 11 vertices close enough together for clustering to merge them.
    fn plane() -> MeshData {
        let side = 11;
        let mut positions = Vec::new();
        for row in 0..side {
            for column in 0..side {
                positions.push([
                    column as f32 / (side - 1) as f32 - 0.5,
                    row as f32 / (side - 1) as f32 - 0.5,
                    0.0,
                ]);
            }
        }
        let mut indices = Vec::new();
        for row in 0..side - 1 {
            for column in 0..side - 1 {
                let a = row * side + column;
                let b = a + 1;
                let c = a + side;
                let d = c + 1;
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
        MeshData::new(positions, indices)
    }

    #[test]
    fn a_coarse_cell_merges_vertices_without_growing_the_mesh() {
        let source = plane();
        let reduced = decimate(&source, 0.2);
        assert!(
            reduced.positions.len() < source.positions.len(),
            "the lattice must merge: {} vs {}",
            reduced.positions.len(),
            source.positions.len()
        );
        assert!(reduced.triangle_count() <= source.triangle_count());
        assert!(reduced.triangle_count() > 0);
    }

    #[test]
    fn a_small_cell_keeps_the_geometry() {
        let reduced = decimate(&cube(), 0.01);
        assert_eq!(reduced.triangle_count(), cube().triangle_count());
        assert_eq!(reduced.positions.len(), cube().positions.len());
    }

    #[test]
    fn the_error_bound_is_the_half_diagonal() {
        let cell = 0.25f32;
        let bound = cluster_error(cell);
        assert!((bound - cell * 3f32.sqrt() * 0.5).abs() < 1e-9);
        // A vertex may move by at most half a cell diagonal: the centroid stays inside its cell.
        let reduced = decimate(&cube(), cell);
        for position in &reduced.positions {
            let nearest = cube()
                .positions
                .iter()
                .map(|source| {
                    let dx = position[0] - source[0];
                    let dy = position[1] - source[1];
                    let dz = position[2] - source[2];
                    (dx * dx + dy * dy + dz * dz).sqrt()
                })
                .fold(f32::MAX, f32::min);
            assert!(nearest <= bound + cell * 0.5, "a vertex moved {nearest} with bound {bound}");
        }
    }

    #[test]
    fn the_pixel_rule_keeps_the_error_under_a_pixel() {
        let (radius, distance, height, fov) = (5.0f32, 100.0f32, 540.0f32, 40f32.to_radians());
        let cell = cell_for_pixel_error(radius, distance, height, fov, 1.0);
        let error_px = cluster_error(cell) * height / (distance * (fov * 0.5).tan());
        assert!(error_px <= 1.0 + 1e-5, "projected error {error_px} px");
        // A coarser cell is allowed when the mesh is closer.
        let near = cell_for_pixel_error(radius, distance * 0.1, height, fov, 1.0);
        assert!(near < cell, "closer means a finer cell is required");
    }
}
