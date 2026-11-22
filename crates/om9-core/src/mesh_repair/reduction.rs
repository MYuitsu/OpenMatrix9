// SPDX-License-Identifier: LGPL-2.1-or-later
//! Reduce a document mesh in triangle units, carrying its channels through each collapse.
use super::{collapse_to, normal};
use crate::{
    Error, Result, budget::Limits, geometry::Mesh, rhino_semantics::MeshUv,
    transaction::Cancellation,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn triangle_count(mesh: &Mesh) -> usize {
    mesh.faces.iter().map(|f| f.len().saturating_sub(2)).sum()
}

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub triangles: usize,
    pub feature_angle: f64,
    pub accuracy: u8,
}

struct CornerChannel<'a> {
    original: &'a [[f64; 2]],
    vertex: Vec<[f64; 2]>,
}

impl<'a> CornerChannel<'a> {
    fn new(source: &'a [[f64; 2]], mesh: &Mesh, seams: &mut BTreeSet<u32>) -> Result<Self> {
        let mut vertex = Vec::new();
        if !source.is_empty() {
            if source.len() != mesh.faces.iter().map(Vec::len).sum::<usize>() {
                return Err(Error::Invalid("mesh corner UV count"));
            }
            vertex.resize(mesh.vertices.len(), [0.; 2]);
            let mut seen = vec![false; vertex.len()];
            for (&index, &uv) in mesh.faces.iter().flatten().zip(source) {
                let i = index as usize;
                if seen[i] && vertex[i] != uv {
                    seams.insert(index);
                }
                seen[i] = true;
                vertex[i] = uv;
            }
        }
        Ok(Self {
            original: source,
            vertex,
        })
    }
    fn push(&self, out: &mut Vec<[f64; 2]>, id: u32, corner: usize, seams: &BTreeSet<u32>) {
        if !self.original.is_empty() {
            out.push(if seams.contains(&id) {
                self.original[corner]
            } else {
                self.vertex[id as usize]
            });
        }
    }
}

fn blend<const N: usize>(values: &mut [[f64; N]], keep: usize, drop: usize, t: f64) {
    if !values.is_empty() && t != 0.0 {
        values[keep] = std::array::from_fn(|i| values[keep][i] * (1.0 - t) + values[drop][i] * t);
    }
}

pub fn reduce(
    mesh: &Mesh,
    uv: Option<&MeshUv>,
    options: Options,
    limits: &Limits,
    cancel: &Cancellation,
) -> Result<(Mesh, Option<MeshUv>)> {
    mesh.validate(limits)?;
    if !(1..=10).contains(&options.accuracy)
        || !(0.0..=180.0).contains(&options.feature_angle)
        || options.triangles == 0
    {
        return Err(Error::Invalid("mesh reduction settings"));
    }
    if let Some(uv) = uv {
        uv.validate(limits)?;
        if !uv.uvs.is_empty() && uv.uvs.len() != mesh.vertices.len() {
            return Err(Error::Invalid("mesh vertex UV count"));
        }
    }
    let mut seams = BTreeSet::new();
    let mut native_uv = CornerChannel::new(&mesh.corner_uvs, mesh, &mut seams)?;
    let mut record_uv =
        CornerChannel::new(uv.map_or(&[], |v| v.corners.as_slice()), mesh, &mut seams)?;
    if cancel.is_canceled() {
        return Err(Error::Canceled);
    }
    if triangle_count(mesh) <= options.triangles {
        return Ok((mesh.clone(), uv.cloned()));
    }
    let triangles = crate::settings::triangulate_mesh(mesh)?;
    triangles.validate(limits)?;
    let mut origins = Vec::with_capacity(triangles.faces.len());
    let mut offset = 0;
    for (f, face) in mesh.faces.iter().enumerate() {
        for i in 1..face.len() - 1 {
            origins.push((f, [offset, offset + i, offset + i + 1]));
        }
        offset += face.len();
    }
    let mut legacy = uv.map_or_else(Vec::new, |v| v.uvs.clone());
    let mut normals = mesh.normals.clone();
    let mut colors: Vec<[f64; 4]> = mesh.colors.iter().map(|v| v.map(f64::from)).collect();
    let (vertices, faces) = collapse_to(
        &triangles,
        options.triangles,
        true,
        options.feature_angle,
        options.accuracy,
        &seams,
        cancel,
        |faces, a, b, t| {
            let mut i = 0;
            origins.retain(|_| {
                let face = &faces[i];
                i += 1;
                !(face.contains(&a) && face.contains(&b))
            });
            let (a, b) = (a as usize, b as usize);
            blend(&mut legacy, a, b, t);
            blend(&mut native_uv.vertex, a, b, t);
            blend(&mut record_uv.vertex, a, b, t);
            blend(&mut colors, a, b, t);
            blend(&mut normals, a, b, t);
            if !normals.is_empty() && t != 0.0 {
                let len = normals[a].iter().map(|x| x * x).sum::<f64>().sqrt();
                if len > 0.0 {
                    normals[a] = normals[a].map(|v| v / len);
                }
            }
        },
    )?;
    if faces.len() == triangles.faces.len() {
        return Ok((mesh.clone(), uv.cloned()));
    }
    let used: BTreeSet<_> = faces.iter().flatten().copied().collect();
    let remap: BTreeMap<_, _> = used
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i as u32))
        .collect();
    let mut result = Mesh {
        vertices: used.iter().map(|&i| vertices[i as usize]).collect(),
        normals: used
            .iter()
            .filter_map(|&i| normals.get(i as usize).copied())
            .collect(),
        colors: used
            .iter()
            .filter_map(|&i| colors.get(i as usize).map(|v| v.map(|c| c.round() as u8)))
            .collect(),
        faces: Vec::new(),
        corner_uvs: Vec::new(),
    };
    let mut corners = Vec::new();
    let mut i = 0;
    while i < faces.len() {
        let (source, original) = origins[i];
        let mut polygon = faces[i].clone();
        let mut indices = original.to_vec();
        // Rejoin the two surviving halves of a source quad when its diagonal
        // still lies inside a convex, coplanar quad.
        if mesh.faces[source].len() == 4
            && origins.get(i + 1).is_some_and(|v| v.0 == source)
            && polygon[0] == faces[i + 1][0]
            && polygon[2] == faces[i + 1][1]
        {
            let ids = [polygon[0], polygon[1], polygon[2], faces[i + 1][2]];
            let reference = normal(&vertices, &ids);
            if reference.is_some_and(|(n, _)| {
                (1..4).all(|j| {
                    let fan = [ids[j], ids[(j + 1) % 4], ids[(j + 2) % 4]];
                    normal(&vertices, &fan)
                        .is_some_and(|(m, _)| (0..3).map(|k| n[k] * m[k]).sum::<f64>() > 1.0 - 1e-8)
                })
            }) {
                i += 1;
                polygon.push(ids[3]);
                indices.push(origins[i].1[2]);
            }
        }
        for (&id, corner) in polygon.iter().zip(indices) {
            native_uv.push(&mut result.corner_uvs, id, corner, &seams);
            record_uv.push(&mut corners, id, corner, &seams);
        }
        result
            .faces
            .push(polygon.iter().map(|i| remap[i]).collect());
        i += 1;
    }
    result.validate(limits)?;
    let mapped = uv.map(|source| MeshUv {
        uvs: used
            .iter()
            .filter_map(|&i| legacy.get(i as usize).copied())
            .collect(),
        corners,
        material: source.material.clone(),
        pbr: source.pbr.clone(),
    });
    Ok((result, mapped))
}
