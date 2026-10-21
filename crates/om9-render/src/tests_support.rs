//! Small fixed meshes shared by the integration tests.
//!
//! Kept in the library rather than in one test file because several test binaries and the benchmarks use
//! the same shapes; it is not part of the renderer's public contract.

use crate::MeshData;

/// A closed torus with `major` steps around the ring and `minor` around the tube.
pub fn torus(major: usize, minor: usize) -> MeshData {
    assert!(
        major >= 3 && minor >= 3,
        "a torus needs at least three steps each way"
    );
    let major_radius = 2.0f32;
    let minor_radius = 0.6f32;
    let mut positions = Vec::with_capacity(major * minor);
    for around in 0..major {
        let u = around as f32 / major as f32 * std::f32::consts::TAU;
        for along in 0..minor {
            let v = along as f32 / minor as f32 * std::f32::consts::TAU;
            let radius = major_radius + minor_radius * v.cos();
            positions.push([radius * u.cos(), radius * u.sin(), minor_radius * v.sin()]);
        }
    }
    let mut indices = Vec::with_capacity(major * minor * 6);
    for around in 0..major {
        for along in 0..minor {
            let next_around = (around + 1) % major;
            let next_along = (along + 1) % minor;
            let a = (around * minor + along) as u32;
            let b = (next_around * minor + along) as u32;
            let c = (next_around * minor + next_along) as u32;
            let d = (around * minor + next_along) as u32;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }
    MeshData::new(positions, indices)
}

/// Re-exported from [`crate::shapes`]; the benches used it here first.
pub use crate::shapes::sphere;
