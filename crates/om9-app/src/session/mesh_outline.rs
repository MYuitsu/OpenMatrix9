// SPDX-License-Identifier: LGPL-2.1-or-later
use super::Controller;
use om9_addon_abi::session::{Geo, Input, Reply, Want};
use om9_core::{Error, Result, budget::memory::MemoryClass, geometry::Geometry};
use om9_scene::scene::RenderBuffers;

impl Controller {
    pub(super) fn mesh_outline_step(&self, input: &Input) -> Result<Reply> {
        let prompt = || {
            Reply::ask(
                "Select meshes, surfaces, or polysurfaces to outline, Enter",
                serde_json::Value::Null,
            )
            .want(Want::Objects)
        };
        if !matches!(input, Input::Start | Input::Enter) {
            return Ok(prompt());
        }
        let doc = self.document();
        let selected: std::collections::BTreeSet<_> = self
            .selected()
            .iter()
            .copied()
            .filter(|id| {
                matches!(
                    doc.entities[id].geometry,
                    Geometry::Mesh(_)
                        | Geometry::Surface(_)
                        | Geometry::Brep(_)
                        | Geometry::NativeBrep(_)
                )
            })
            .collect();
        if selected.is_empty() {
            if matches!(input, Input::Enter) {
                return Err(Error::Missing("select a mesh, surface or polysurface"));
            }
            return Ok(prompt());
        }
        let scene = self
            .scene()
            .ok_or(Error::Missing("MeshOutline render scene"))?;
        let objects = scene
            .objects
            .iter()
            .filter(|o| selected.contains(&o.root))
            .collect::<Vec<_>>();
        if objects.len() != selected.len() {
            return Err(Error::Missing("MeshOutline selected render mesh"));
        }
        let mut bytes = 0u64;
        for o in &objects {
            let RenderBuffers::Mesh {
                vertices,
                triangles,
                ..
            } = o.buffers.as_ref()
            else {
                return Err(Error::Missing("MeshOutline render triangles"));
            };
            bytes = bytes
                .saturating_add((vertices.len() as u64).saturating_mul(32))
                .saturating_add((triangles.len() as u64).saturating_mul(768));
        }
        let _memory = self.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache,
            bytes,
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)),
            self.scene_cache_usage()?,
        )?;
        let view = &self.workspace.views[&self.workspace.active];
        let paths = om9_scene::mesh_outline::outlines(
            &view.camera,
            view.cplane,
            &objects,
            self.coordinator.limits(),
        )?;
        if paths.is_empty() {
            return Err(Error::Invalid("selection has no outline in this view"));
        }
        let k = doc.page_unit.factor();
        let create = paths
            .into_iter()
            .map(|mut points| {
                for p in &mut points {
                    *p = p.map(|x| x * k);
                }
                Geo::Polyline {
                    points,
                    closed: true,
                }
            })
            .collect();
        Ok(Reply::finish(create))
    }
}
