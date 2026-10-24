// SPDX-License-Identifier: LGPL-2.1-or-later
//! Portable utilities for OM9-L2-03-19: bounds, NURBS mesh generation and a
//! basic mesh reduction. Exact B-Rep health stays in the OCCT kernel.
use crate::{
    Error, Result,
    budget::Limits,
    geometry::{Geometry, Mesh, NurbsSurface, Point3},
    surface_kernel,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Point3,
    pub max: Point3,
}

impl Bounds {
    pub fn size(&self) -> Point3 {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    pub fn center(&self) -> Point3 {
        [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
            (self.min[2] + self.max[2]) / 2.0,
        ]
    }
}

fn add_points(bounds: &mut Option<Bounds>, points: &[Point3]) {
    for point in points {
        if !point.iter().all(|value| value.is_finite()) {
            continue;
        }
        match bounds {
            Some(current) => {
                for ((min, max), point) in current.min.iter_mut().zip(&mut current.max).zip(point) {
                    *min = min.min(*point);
                    *max = max.max(*point);
                }
            }
            None => {
                *bounds = Some(Bounds {
                    min: *point,
                    max: *point,
                })
            }
        }
    }
}

/// World-space bounds from the entity geometry's own points. Curves use their
/// control points, surfaces their poles, meshes their vertices and B-Reps the
/// tessellated display vertices.
pub fn bounds(geometry: &Geometry) -> Option<Bounds> {
    let mut result = None;
    match geometry {
        Geometry::Point(point) => add_points(&mut result, &[*point]),
        Geometry::Curve(curve) => match curve {
            crate::geometry::Curve::Line { start, end, .. } => {
                add_points(&mut result, &[*start, *end])
            }
            crate::geometry::Curve::Circle {
                center,
                x,
                y,
                radius,
                ..
            } => {
                add_points(
                    &mut result,
                    &[
                        [
                            center[0] + (x[0].abs() + y[0].abs()) * radius,
                            center[1] + (x[1].abs() + y[1].abs()) * radius,
                            center[2] + (x[2].abs() + y[2].abs()) * radius,
                        ],
                        [
                            center[0] - (x[0].abs() + y[0].abs()) * radius,
                            center[1] - (x[1].abs() + y[1].abs()) * radius,
                            center[2] - (x[2].abs() + y[2].abs()) * radius,
                        ],
                    ],
                );
            }
            crate::geometry::Curve::Nurbs(nurbs) => add_points(&mut result, &nurbs.poles),
            crate::geometry::Curve::Polycurve { segments, .. } => {
                for segment in segments {
                    if let Some(inner) = bounds(&Geometry::Curve(segment.clone())) {
                        add_points(&mut result, &[inner.min, inner.max]);
                    }
                }
            }
        },
        Geometry::Surface(surface) => add_points(&mut result, &surface.poles),
        Geometry::Mesh(mesh) => add_points(&mut result, &mesh.vertices),
        Geometry::Cloud(cloud) => add_points(&mut result, &cloud.points),
        Geometry::Brep(brep) => add_points(&mut result, &brep.display.points),
        Geometry::NativeBrep(brep) => add_points(&mut result, &brep.display.vertices),
        Geometry::Hatch(hatch) => add_points(&mut result, &hatch.display.vertices),
        Geometry::Light(light) => add_points(&mut result, &[light.display]),
        Geometry::Instance(_) => {}
    }
    result
}

/// Tessellate a NURBS surface into a display mesh. This is a production mesh
/// grid (not a render-only cache) and preserves the exact surface for later
/// replacement.
pub fn mesh_from_surface(
    surface: &NurbsSurface,
    u_steps: usize,
    v_steps: usize,
    limits: &Limits,
) -> Result<Mesh> {
    if u_steps < 1 || v_steps < 1 || u_steps > 512 || v_steps > 512 {
        return Err(Error::Invalid("mesh surface steps"));
    }
    surface.validate(limits)?;
    let mut vertices = Vec::with_capacity((u_steps + 1) * (v_steps + 1));
    for u in 0..=u_steps {
        for v in 0..=v_steps {
            vertices.push(surface_kernel::evaluate(
                surface,
                u as f64 / u_steps as f64,
                v as f64 / v_steps as f64,
                limits,
            )?);
        }
    }
    let mut faces = Vec::with_capacity(u_steps * v_steps * 2);
    for u in 0..u_steps {
        for v in 0..v_steps {
            let a = (u * (v_steps + 1) + v) as u32;
            let b = ((u + 1) * (v_steps + 1) + v) as u32;
            let c = (u * (v_steps + 1) + v + 1) as u32;
            let d = ((u + 1) * (v_steps + 1) + v + 1) as u32;
            faces.push(vec![a, b, c]);
            faces.push(vec![b, d, c]);
        }
    }
    Ok(Mesh {
        vertices,
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    })
}

/// Basic mesh reduction by vertex clustering. It is an editing aid, not an
/// advanced repair pass: coincident cells collapse and degenerate faces drop.
pub fn reduce_mesh(mesh: &Mesh, target_faces: usize) -> Result<Mesh> {
    if target_faces < 1 {
        return Err(Error::Invalid("mesh reduce target"));
    }
    mesh.validate(&Limits::default())?;
    if mesh.faces.len() <= target_faces {
        return Ok(mesh.clone());
    }
    let mut vertices = mesh.vertices.clone();
    let mut faces: Vec<Vec<u32>> = mesh.faces.clone();
    while faces.len() > target_faces {
        let mut counts: std::collections::BTreeMap<(u32, u32), usize> =
            std::collections::BTreeMap::new();
        for face in &faces {
            for index in 0..face.len() {
                let a = face[index];
                let b = face[(index + 1) % face.len()];
                let key = if a < b { (a, b) } else { (b, a) };
                *counts.entry(key).or_default() += 1;
            }
        }
        let mut best: Option<(f64, u32, u32, bool)> = None;
        for (&(a, b), &count) in &counts {
            let pa = vertices[a as usize];
            let pb = vertices[b as usize];
            let length = (pa[0] - pb[0]).hypot((pa[1] - pb[1]).hypot(pa[2] - pb[2]));
            let interior = count >= 2;
            if best.is_none_or(|(best_length, _, _, best_interior)| {
                (interior && !best_interior) || (interior == best_interior && length < best_length)
            }) {
                best = Some((length, a, b, interior));
            }
        }
        let Some((_, keep, drop, _)) = best else {
            break;
        };
        if keep == drop {
            break;
        }
        let pa = vertices[keep as usize];
        let pb = vertices[drop as usize];
        vertices[keep as usize] = [
            (pa[0] + pb[0]) / 2.0,
            (pa[1] + pb[1]) / 2.0,
            (pa[2] + pb[2]) / 2.0,
        ];
        for face in &mut faces {
            for index in face.iter_mut() {
                if *index == drop {
                    *index = keep;
                }
            }
            let mut unique = Vec::with_capacity(face.len());
            for index in face.iter() {
                if !unique.contains(index) {
                    unique.push(*index);
                }
            }
            *face = unique;
        }
        faces.retain(|face| face.len() >= 3);
        let mut seen = std::collections::BTreeSet::new();
        faces.retain(|face| {
            let mut sorted = face.clone();
            sorted.sort_unstable();
            seen.insert(sorted)
        });
        if faces.len() <= target_faces {
            break;
        }
    }
    if faces.len() > target_faces {
        faces.truncate(target_faces);
    }
    let mut used = std::collections::BTreeSet::new();
    for face in &faces {
        used.extend(face.iter().copied());
    }
    let mut remap = std::collections::BTreeMap::new();
    let mut compact = Vec::with_capacity(used.len());
    for index in used {
        remap.insert(index, compact.len() as u32);
        compact.push(vertices[index as usize]);
    }
    for face in &mut faces {
        for index in face.iter_mut() {
            *index = remap[index];
        }
    }
    Ok(Mesh {
        vertices: compact,
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    })
}
