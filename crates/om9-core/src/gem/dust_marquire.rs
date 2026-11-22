// SPDX-License-Identifier: LGPL-2.1-or-later
//! Parametric six-sided stone. Rings are in the OM9 frame (length along X).
use super::{GemShape, GemSize, GemSpec, Mesh, Point3, ring_faces};

pub(super) const SAMPLE: GemSize = GemSize {
    length_mm: 11.52,
    width_mm: 6.04643,
    depth_mm: 3.93127,
};
pub(super) const GIRDLE_SHARE: f64 = 0.20914 / 3.93127;
pub(super) const CROWN_SHARE: f64 = 0.93614 / (3.93127 - 0.20914);
pub(super) const LOWER_SCALE: f64 = 0.38304;
pub(super) const LOWER_DEPTH: f64 = 2.20449 / 2.78600;
pub(super) const CROWN_START: usize = 7;
pub(super) const PAVILION_START: usize = 13;

pub(super) fn applies(spec: &GemSpec) -> bool {
    spec.shape == GemShape::DustMarquire && spec.profile.is_none()
}

fn ring(spec: &GemSpec, tip: f64, width: f64, z: f64) -> [Point3; 6] {
    let x = spec.length_mm * 0.5;
    let y = spec.width_mm * 0.5 * width;
    // Parallel oblique edges keep every crown and pavilion quad planar.
    let shoulder = x * (tip - (1.0 - 2.62034 / 5.76) * width);
    let tip = x * tip;
    [
        [tip, 0., z],
        [shoulder, y, z],
        [-shoulder, y, z],
        [-tip, 0., z],
        [-shoulder, -y, z],
        [shoulder, -y, z],
    ]
}

pub(super) fn girdle(spec: &GemSpec) -> [Point3; 6] {
    ring(spec, 1., 1., 0.)
}

pub(super) fn table(spec: &GemSpec) -> [Point3; 6] {
    let width = spec.table_ratio();
    let tip = 1.0 - (1.0 - width) * 0.7568;
    ring(
        spec,
        tip,
        width,
        spec.girdle_mm * 0.5 + spec.crown_height_mm,
    )
}

pub(super) fn culet(spec: &GemSpec) -> Point3 {
    [
        -0.01695 * spec.length_mm,
        0.,
        -spec.girdle_mm * 0.5 - spec.pavilion_depth_mm,
    ]
}

pub(super) fn body(spec: &GemSpec) -> (Vec<Point3>, Vec<Vec<u32>>) {
    let band = spec.girdle_mm * 0.5;
    let mut vertices = table(spec).to_vec();
    vertices.extend(ring(spec, 1., 1., band));
    vertices.extend(ring(spec, 1., 1., -band));
    vertices.extend(ring(
        spec,
        LOWER_SCALE,
        LOWER_SCALE,
        -band - LOWER_DEPTH * spec.pavilion_depth_mm,
    ));
    vertices.push(culet(spec));
    let table = [0, 1, 2, 3, 4, 5];
    let upper = [6, 7, 8, 9, 10, 11];
    let lower = [12, 13, 14, 15, 16, 17];
    let shoulder = [18, 19, 20, 21, 22, 23];
    let mut faces = vec![table.to_vec()];
    ring_faces(&mut faces, &lower, &upper);
    ring_faces(&mut faces, &upper, &table);
    ring_faces(&mut faces, &shoulder, &lower);
    for i in 0..6 {
        faces.push(vec![shoulder[(i + 1) % 6], shoulder[i], 24]);
    }
    (vertices, faces)
}

pub(super) fn table_diagonal(mesh: &Mesh, a: u32, b: u32) -> bool {
    a < 6
        && b < 6
        && (2..=4).contains(&a.abs_diff(b))
        && mesh.vertices.len() == 25
        && mesh.faces.len() == 28
        && mesh.faces[..4]
            .iter()
            .enumerate()
            .all(|(i, f)| f.as_slice() == [0, i as u32 + 1, i as u32 + 2])
}
