//! CPU-side geometry registry for the GPU instanced renderer.
//!
//! Geometry is keyed by the document's stable mesh key and uploaded once; occurrences only reference
//! `mesh_id` from `InstanceTable`. LOD meshes and their per-level world errors travel with the entry so
//! the compute pass can choose a level without CPU work per frame.

use crate::MeshData;
use std::collections::HashMap;

pub struct GeometryEntry {
    pub base: MeshData,
    /// LOD 1..N; level 0 is `base`.
    pub lods: Vec<MeshData>,
    /// World-space error per level, level 0 = 0.
    pub errors_world: Vec<f32>,
    pub bounds: ([f32; 3], f32),
}

#[derive(Default)]
pub struct GeometryStore {
    entries: Vec<GeometryEntry>,
    by_key: HashMap<u64, u32>,
}

impl GeometryStore {
    /// Returns the existing id when the same key is inserted again (shared occurrence geometry).
    pub fn insert(&mut self, key: u64, entry: GeometryEntry) -> u32 {
        if let Some(existing) = self.by_key.get(&key) {
            return *existing;
        }
        let id = self.entries.len() as u32;
        self.entries.push(entry);
        self.by_key.insert(key, id);
        id
    }

    pub fn get(&self, id: u32) -> Option<&GeometryEntry> {
        self.entries.get(id as usize)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Bytes the vertex/index buffers will need, including every LOD, for upload budgeting.
    pub fn gpu_bytes(&self) -> u64 {
        let mesh_bytes = |mesh: &MeshData| {
            mesh.positions.len() as u64 * 12 + mesh.indices.len() as u64 * 4
        };
        self.entries
            .iter()
            .map(|entry| {
                mesh_bytes(&entry.base) + entry.lods.iter().map(mesh_bytes).sum::<u64>()
            })
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(triangles: usize) -> GeometryEntry {
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        for i in 0..triangles {
            let base = (i * 3) as u32;
            positions.extend_from_slice(&[[i as f32, 0.0, 0.0], [i as f32, 1.0, 0.0], [i as f32, 1.0, 1.0]]);
            indices.extend_from_slice(&[base, base + 1, base + 2]);
        }
        GeometryEntry {
            base: MeshData::new(positions, indices),
            lods: Vec::new(),
            errors_world: vec![0.0],
            bounds: ([0.0; 3], 1.0),
        }
    }

    #[test]
    fn repeated_keys_share_one_geometry() {
        let mut store = GeometryStore::default();
        let first = store.insert(7, entry(2));
        let second = store.insert(7, entry(2));
        let other = store.insert(9, entry(3));
        assert_eq!(first, second);
        assert_ne!(first, other);
        assert_eq!(store.len(), 2);
        assert_eq!(store.get(first).unwrap().base.triangle_count(), 2);
    }

    #[test]
    fn gpu_bytes_counts_base_and_every_lod() {
        let mut store = GeometryStore::default();
        store.insert(1, entry(2));
        // 6 vertices * 12 bytes + 6 indices * 4 bytes = 96.
        assert_eq!(store.gpu_bytes(), 96);
    }
}
/// GPU copy of one geometry entry: base and every LOD, uploaded once.
pub struct GpuGeometryEntry {
    pub base_vertices: wgpu::Buffer,
    pub base_indices: wgpu::Buffer,
    pub lods: Vec<(wgpu::Buffer, wgpu::Buffer)>,
    pub errors_world: Vec<f32>,
    pub bounds: ([f32; 3], f32),
}

pub struct GpuGeometryStore {
    pub entries: Vec<GpuGeometryEntry>,
}

impl GpuGeometryStore {
    pub fn upload(device: &wgpu::Device, store: &GeometryStore) -> Self {
        use wgpu::util::DeviceExt;
        let make = |mesh: &MeshData| {
            let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("geometry vertices"),
                contents: bytemuck::cast_slice(&mesh.positions),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("geometry indices"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            (vertices, indices)
        };
        let entries = store
            .entries
            .iter()
            .map(|entry| {
                let (base_vertices, base_indices) = make(&entry.base);
                let lods = entry.lods.iter().map(make).collect();
                GpuGeometryEntry {
                    base_vertices,
                    base_indices,
                    lods,
                    errors_world: entry.errors_world.clone(),
                    bounds: entry.bounds,
                }
            })
            .collect();
        Self { entries }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
