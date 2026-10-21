//! `MeshStress` — the command that fills the document with mesh objects for GPU stress tests.
//!
//! Typed in the command line as `MeshStress 2000` (or just `MeshStress` for the default), it inserts a
//! lattice of small mesh cubes in **one transaction**, so one Undo removes the whole stress scene. The
//! objects are ordinary document entities: they are drawn by the GPU viewport and can be clicked or
//! band-selected like any other geometry.
use super::Controller;
use om9_core::{
    Error, Result,
    geometry::{Geometry, Mesh, Transform},
    model::Entity,
    transaction::{Mutation, TransactionRequest},
};

/// Objects created by a bare `MeshStress`.
pub const DEFAULT_STRESS_MESHES: usize = 1000;
/// Upper bound on one command, so a typo cannot freeze the application.
pub const MAX_STRESS_MESHES: usize = 20_000;

/// A unit cube: 8 vertices and 12 triangles, the cheapest real object for a draw-call stress test.
fn stress_cube() -> Mesh {
    let vertices = vec![
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
    ];
    let faces = vec![
        vec![0, 2, 1],
        vec![0, 3, 2],
        vec![4, 5, 6],
        vec![4, 6, 7],
        vec![0, 1, 5],
        vec![0, 5, 4],
        vec![2, 3, 7],
        vec![2, 7, 6],
        vec![1, 2, 6],
        vec![1, 6, 5],
        vec![0, 4, 7],
        vec![0, 7, 3],
    ];
    Mesh {
        vertices,
        faces,
        normals: Vec::new(),
        colors: Vec::new(),
    }
}

impl Controller {
    /// Inserts `count` mesh objects in one transaction and returns how many were created.
    pub fn insert_stress_meshes(&mut self, count: usize) -> Result<usize> {
        if count == 0 || count > MAX_STRESS_MESHES {
            return Err(Error::Budget("stress mesh count"));
        }
        let cube = stress_cube();
        let side = (count as f64).cbrt().ceil().max(1.0) as i32;
        let spacing = 1.6;
        let mut mutations = Vec::with_capacity(count);
        for index in 0..count {
            let x = (index as i32 % side) as f64 - f64::from(side) * 0.5;
            let y = ((index as i32 / side) % side) as f64 - f64::from(side) * 0.5;
            let z = (index as i32 / (side * side)) as f64 - f64::from(side) * 0.5;
            let mut entity = Entity::new(
                "Mesh",
                Geometry::Mesh(cube.clone()),
                self.document().current_layer,
            );
            entity.placement = Transform::translation([x * spacing, y * spacing, z * spacing]);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        if self.scene.is_none() {
            return Err(Error::Missing("stress scene"));
        }
        let held = self.scene_cache_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new("Mesh Stress", mutations), held)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(count)
    }
}
