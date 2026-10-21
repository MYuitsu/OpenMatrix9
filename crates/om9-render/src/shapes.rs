//! Simple procedural meshes the viewport uses as placeholders and for stress scenes.
//!
//! They are renderer-owned geometry, not kernel output: the OCCT path produces its own tessellation and
//! keeps owning that. These exist so a demo scene can be built without a kernel.

use crate::MeshData;

/// A closed UV sphere: `rings` bands from pole to pole and `segments` around, so the triangle count is
/// `2 * rings * segments`. The default is the same order of magnitude as a tessellated OCCT sphere.
pub fn sphere(rings: usize, segments: usize, radius: f32) -> MeshData {
    assert!(
        rings >= 2 && segments >= 3,
        "a sphere needs at least two rings and three segments"
    );
    let mut positions = Vec::with_capacity((rings + 1) * segments);
    for ring in 0..=rings {
        let v = std::f32::consts::PI * ring as f32 / rings as f32;
        for segment in 0..segments {
            let u = std::f32::consts::TAU * segment as f32 / segments as f32;
            positions.push([
                radius * v.sin() * u.cos(),
                radius * v.sin() * u.sin(),
                radius * v.cos(),
            ]);
        }
    }
    let mut indices = Vec::with_capacity(rings * segments * 6);
    for ring in 0..rings {
        for segment in 0..segments {
            let next = (segment + 1) % segments;
            let a = (ring * segments + segment) as u32;
            let b = (ring * segments + next) as u32;
            let c = ((ring + 1) * segments + segment) as u32;
            let d = ((ring + 1) * segments + next) as u32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    MeshData::new(positions, indices)
}
