// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded, disposable mesh CSG. This module never receives canonical CAD sources.
use super::*;
use om9_core::{
    geometry::Mesh,
    mesh_csg::{BooleanOp, CsgOptions, mesh_boolean},
};
use om9_scene::scene::{RenderBuffers, SceneBuffers};
use std::collections::BTreeMap;

const ALLOWANCE: u64 = 64 * 1024 * 1024;
const MAX_INPUT_TRIANGLES: usize = 25_000;
const MAX_OUTPUT_TRIANGLES: usize = 250_000;

pub(super) struct Output {
    pub buffers: SceneBuffers,
    pub id: uuid::Uuid,
    pub compute_ms: f64,
    pub triangles: usize,
}
pub(super) struct PreviewJob {
    receiver: mpsc::Receiver<Result<Output>>,
    pub output: Option<Output>,
    pub error: Option<String>,
}
impl PreviewJob {
    pub fn start(
        controller: &Controller,
        stamps: &[SourceStamp],
        cancel: Cancellation,
    ) -> Result<Self> {
        let scene = controller.scene().ok_or(Error::Missing("preview scene"))?;
        if stamps.len() > 1024 {
            return Err(Error::Budget("Boolean preview inputs"));
        }
        let physical =
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut owner = controller.coordinator().reserve_external_memory_with_held(
            MemoryClass::CpuCache,
            ALLOWANCE,
            physical,
            controller.scene_cache_usage()?,
        )?;
        owner.set_allocated_bytes(ALLOWANCE)?;
        // Share admitted immutable render buffers; no UI-thread mesh copies.
        let input_owner = controller.share_scene_buffer_owner(scene)?;
        let mut inputs = Vec::new();
        inputs
            .try_reserve_exact(stamps.len())
            .map_err(|_| Error::Budget("Boolean preview inputs"))?;
        for stamp in stamps {
            let object = scene
                .objects
                .iter()
                .find(|o| o.root == stamp.object && o.leaf == stamp.object)
                .ok_or(Error::Unsupported("Boolean preview occurrence"))?;
            inputs.push((object.buffers.clone(), object.transform));
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("om9-boolean-mesh-preview".into())
            .spawn(move || {
                let started = std::time::Instant::now();
                let result = compute(&inputs, &cancel).map(|data| {
                    let triangles = match &data {
                        RenderBuffers::Mesh { triangles, .. } => triangles.len(),
                        _ => 0,
                    };
                    Output {
                        buffers: SceneBuffers::transient(data, Arc::new(owner)),
                        id: uuid::Uuid::new_v4(),
                        compute_ms: started.elapsed().as_secs_f64() * 1000.,
                        triangles,
                    }
                });
                drop(inputs);
                drop(input_owner);
                if !cancel.is_canceled() {
                    let _ = sender.send(result);
                }
            })
            .map_err(|e| Error::Io(format!("mesh preview worker: {e}")))?;
        Ok(Self {
            receiver,
            output: None,
            error: None,
        })
    }
    pub fn failed(error: Error) -> Self {
        let (_, receiver) = mpsc::sync_channel(1);
        Self {
            receiver,
            output: None,
            error: Some(error.to_string()),
        }
    }
    pub fn poll(&mut self) {
        if self.output.is_none()
            && let Ok(result) = self.receiver.try_recv()
        {
            match result {
                Ok(output) => self.output = Some(output),
                Err(error) => {
                    eprintln!("Boolean temporary preview unavailable: {error}");
                    self.error = Some(error.to_string());
                }
            }
        }
    }
}

fn empty() -> Mesh {
    Mesh {
        vertices: vec![],
        faces: vec![],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    }
}

fn coarse(
    buffers: &SceneBuffers,
    previous: Option<&(om9_render::MeshData, Mesh)>,
) -> Result<(om9_render::MeshData, Mesh)> {
    let RenderBuffers::Mesh {
        vertices,
        triangles,
        ..
    } = buffers.as_ref()
    else {
        return Err(Error::Unsupported("Boolean preview mesh"));
    };
    if triangles.len() > MAX_INPUT_TRIANGLES || vertices.len() > 100_000 {
        return Err(Error::Budget("Boolean preview input mesh"));
    }
    // Welding and simplification affect only this visibly approximate preview.
    let mut weld = BTreeMap::new();
    let mut data = om9_render::MeshData::new(Vec::new(), Vec::new());
    for &(face, _) in triangles {
        let mut ids = [0; 3];
        for i in 0..3 {
            let p = vertices[face[i] as usize];
            let key = p.map(|v| (v * 1e7).round() as i64);
            ids[i] = *weld.entry(key).or_insert_with(|| {
                let id = data.positions.len() as u32;
                data.positions.push(p.map(|v| v as f32));
                id
            });
        }
        if ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2] {
            data.indices.extend(ids);
        }
    }
    if let Some((old, cached)) = previous
        && *old == data
    {
        return Ok((data, cached.clone()));
    }
    let low = om9_render::qem::simplify(&data, 128);
    let mut mesh = empty();
    mesh.vertices = low.positions.iter().map(|p| p.map(f64::from)).collect();
    mesh.faces = low.indices.chunks_exact(3).map(|f| f.to_vec()).collect();
    Ok((data, mesh))
}
fn bounds(mesh: &Mesh) -> [[f64; 3]; 2] {
    let mut b = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
    for p in &mesh.vertices {
        for i in 0..3 {
            b[0][i] = b[0][i].min(p[i]);
            b[1][i] = b[1][i].max(p[i]);
        }
    }
    b
}
fn overlaps(a: [[f64; 3]; 2], b: [[f64; 3]; 2]) -> bool {
    (0..3).all(|i| a[0][i] <= b[1][i] + 1e-5 && b[0][i] <= a[1][i] + 1e-5)
}
fn append(to: &mut Mesh, other: Mesh) {
    let offset = to.vertices.len() as u32;
    to.vertices.extend(other.vertices);
    to.faces.extend(
        other
            .faces
            .into_iter()
            .map(|f| f.into_iter().map(|i| i + offset).collect()),
    );
}
fn compute(inputs: &[(SceneBuffers, Transform)], cancel: &Cancellation) -> Result<RenderBuffers> {
    let mut groups: Vec<Mesh> = Vec::new();
    let mut previous: Option<(om9_render::MeshData, Mesh)> = None;
    for (buffers, transform) in inputs {
        if cancel.is_canceled() {
            return Err(Error::Canceled);
        }
        // Bound scratch per operand; QEM output is independent of Light/High.
        let (data, low) = coarse(buffers, previous.as_ref())?;
        let mut mesh = low.clone();
        previous = Some((data, low));
        for p in &mut mesh.vertices {
            *p = transform.point(*p)?;
        }
        let mut index = 0;
        while index < groups.len() {
            if cancel.is_canceled() {
                return Err(Error::Canceled);
            }
            if overlaps(bounds(&mesh), bounds(&groups[index])) {
                let other = groups.swap_remove(index);
                // ponytail: bounded coarse CSG; refuse pathological components,
                // preserving exact work and originals rather than guessing a result.
                let options = CsgOptions {
                    tolerance: 1e-5,
                    max_triangles: 8000,
                    max_fragments: 16000,
                    max_work: 5_000_000,
                };
                mesh = match mesh_boolean(&mesh, &other, BooleanOp::Union, options) {
                    Ok(result) => result,
                    // Coarse QEM facets can leave tiny intersection slivers.
                    // Retry only this approximate display with a smaller
                    // classifier tolerance; all closedness checks stay active.
                    Err(Error::Invalid(_)) => mesh_boolean(
                        &mesh,
                        &other,
                        BooleanOp::Union,
                        CsgOptions {
                            tolerance: 1e-7,
                            ..options
                        },
                    )?,
                    Err(error) => return Err(error),
                }
                .0;
                index = 0;
            } else {
                index += 1;
            }
        }
        groups.push(mesh);
        if groups.iter().map(|g| g.faces.len()).sum::<usize>() > MAX_OUTPUT_TRIANGLES {
            return Err(Error::Budget("Boolean preview result"));
        }
    }
    let mut all = empty();
    for group in groups {
        append(&mut all, group);
    }
    let triangles: Vec<_> = all
        .faces
        .iter()
        .enumerate()
        .map(|(i, f)| ([f[0], f[1], f[2]], i as u32))
        .collect();
    let edges = all
        .faces
        .iter()
        .flat_map(|f| [[f[0], f[1]], [f[1], f[2]], [f[2], f[0]]])
        .collect();
    Ok(RenderBuffers::Mesh {
        vertices: all.vertices,
        triangles,
        edges,
    })
}
