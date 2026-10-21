// SPDX-License-Identifier: LGPL-2.1-or-later
//! Source-checked subobject selection, including complete nested occurrence paths.
use super::Controller;
use om9_core::{
    Error, Result,
    budget::Usage,
    identity::{ObjectId, SourceStamp},
};
use om9_scene::scene::{PickHit, Scene, SceneObject, Subelement};

/// A member path is part of identity: shared topology in two occurrences is two selectable parts.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedSubobject {
    pub root: ObjectId,
    pub path: Vec<ObjectId>,
    pub subset: Subelement,
    pub source: SourceStamp,
    pub world_epoch: u64,
}
impl SelectedSubobject {
    pub fn from_hit(hit: &PickHit) -> Self {
        Self {
            root: hit.root,
            path: hit.path.clone(),
            subset: hit.subset.clone(),
            source: hit.source,
            world_epoch: hit.world_epoch,
        }
    }
    fn same_key(&self, other: &Self) -> bool {
        self.root == other.root && self.path == other.path && self.subset == other.subset
    }
    fn validate<'a>(&self, scene: &'a Scene, epoch: u64) -> Result<&'a SceneObject> {
        if self.world_epoch != epoch || scene.epoch != epoch {
            return Err(Error::Stale("subobject publication"));
        }
        scene.document.resolve(&self.source)?;
        let object = scene
            .objects
            .iter()
            .find(|object| object.root == self.root && object.path == self.path)
            .ok_or(Error::Missing("subobject occurrence path"))?;
        if object.source != self.source {
            return Err(Error::Stale("subobject source"));
        }
        if !object.visible || !object.selectable || !object.has_pick_geometry() {
            return Err(Error::Permission("subobject owner"));
        }
        validate_subset(object, &scene.entity_for(object)?.geometry, &self.subset)?;
        // Resolve the definition path, never a nearby or first matching member.
        if scene.entity_for(object)?.id != object.leaf {
            return Err(Error::Stale("subobject leaf"));
        }
        Ok(object)
    }
}

impl Controller {
    fn root_subobject(&self, root: ObjectId, subset: Subelement) -> Result<SelectedSubobject> {
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("subobject scene"))?;
        let object = scene
            .objects
            .iter()
            .find(|o| o.root == root && o.path.is_empty())
            .ok_or(Error::Missing(
                "subobject root geometry; occurrence path required",
            ))?;
        Ok(SelectedSubobject {
            root,
            path: Vec::new(),
            subset,
            source: object.source,
            world_epoch: scene.epoch,
        })
    }
    /// Root-only compatibility door; instances must provide the path from their actual hit.
    pub fn select_subobject(&mut self, root: ObjectId, subset: Subelement) -> Result<()> {
        let reference = self.root_subobject(root, subset)?;
        self.select_subobject_ref(reference)
    }
    pub fn toggle_subobject(&mut self, root: ObjectId, subset: Subelement) -> Result<()> {
        let reference = self.root_subobject(root, subset)?;
        self.toggle_subobject_ref(reference)
    }
    pub fn select_subobject_ref(&mut self, reference: SelectedSubobject) -> Result<()> {
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("subobject scene"))?;
        reference.validate(scene, self.coordinator.epoch())?;
        if self.subselection.iter().any(|old| old.same_key(&reference)) {
            return Ok(());
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            Usage::array::<SelectedSubobject>(self.subselection.len() + 1)?,
            self.coordinator.limits(),
        )?;
        held.add(
            Usage::array::<ObjectId>(reference.path.capacity())?,
            self.coordinator.limits(),
        )?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        self.subselection
            .try_reserve(1)
            .map_err(|_| Error::Budget("subobject selection"))?;
        self.subselection.push(reference);
        self.sub_revision = self.sub_revision.saturating_add(1);
        Ok(())
    }
    pub fn toggle_subobject_ref(&mut self, reference: SelectedSubobject) -> Result<()> {
        reference.validate(
            self.scene
                .as_ref()
                .ok_or(Error::Missing("subobject scene"))?,
            self.coordinator.epoch(),
        )?;
        if let Some(index) = self
            .subselection
            .iter()
            .position(|old| old.same_key(&reference))
        {
            self.subselection.remove(index);
            self.sub_revision = self.sub_revision.saturating_add(1);
            Ok(())
        } else {
            self.select_subobject_ref(reference)
        }
    }
    pub fn clear_subselection(&mut self) {
        if !self.subselection.is_empty() {
            self.subselection.clear();
            self.sub_revision = self.sub_revision.saturating_add(1);
        }
    }
    pub fn subselection(&self) -> &[SelectedSubobject] {
        &self.subselection
    }
    pub fn sub_revision(&self) -> u64 {
        self.sub_revision
    }
    pub fn prune_subselection(&mut self) {
        let Some(scene) = self.scene.as_ref() else {
            self.clear_subselection();
            return;
        };
        let before = self.subselection.len();
        self.subselection
            .retain(|key| key.validate(scene, self.coordinator.epoch()).is_ok());
        if before != self.subselection.len() {
            self.sub_revision = self.sub_revision.saturating_add(1);
        }
    }
}

fn validate_subset(
    object: &om9_scene::scene::SceneObject,
    geometry: &om9_core::geometry::Geometry,
    subset: &Subelement,
) -> Result<()> {
    use om9_scene::scene::RenderBuffers;
    if let om9_core::geometry::Geometry::Brep(shape) = geometry {
        let valid = match subset {
            Subelement::BrepFace(id) => shape.display.faces.contains(id),
            Subelement::BrepEdge(id) => shape.display.edges.contains(id),
            Subelement::BrepVertex(id) => shape.display.vertex_ids.contains(id),
            _ => false,
        };
        return if valid {
            Ok(())
        } else {
            Err(Error::Stale("BRep topology identity"))
        };
    }
    let valid = match (subset, object.buffers.as_ref()) {
        (Subelement::MeshVertex(index), RenderBuffers::Mesh { vertices, .. }) => {
            (*index as usize) < vertices.len()
        }
        (Subelement::MeshFace(face), RenderBuffers::Mesh { triangles, .. }) => {
            triangles.iter().any(|(_, f)| f == face)
        }
        (Subelement::MeshEdge { a, b }, RenderBuffers::Mesh { edges, .. }) => {
            a < b
                && edges
                    .iter()
                    .any(|[x, y]| (*x == *a && *y == *b) || (*x == *b && *y == *a))
        }
        (Subelement::CurveParameter(t), RenderBuffers::Curve { parameters, .. }) => {
            t.is_finite()
                && parameters
                    .first()
                    .zip(parameters.last())
                    .is_some_and(|(a, b)| t >= a && t <= b)
        }
        (Subelement::CloudPoint(index), RenderBuffers::Points { native_indices, .. }) => {
            native_indices.contains(index)
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(Error::Stale("subobject topology key"))
    }
}

impl Controller {
    /// Deletes objects in one transaction. The selection and the sub-selection drop the removed ids, so no
    /// stale key is left behind.
    pub fn delete_objects(&mut self, ids: &[ObjectId]) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        if self.scene.is_none() {
            return Err(Error::Missing("delete scene"));
        }
        let mut mutations = Vec::with_capacity(ids.len());
        for id in ids {
            if !self.document().entities.contains_key(id) {
                return Err(Error::Missing("delete object"));
            }
            mutations.push(om9_core::transaction::Mutation::Delete {
                source: self.document().stamp(*id)?,
            });
        }
        let held = self.scene_cache_usage()?;
        let prepared = self.coordinator.prepare_with_held_usage(
            om9_core::transaction::TransactionRequest::new("Delete", mutations),
            held,
        )?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            for id in ids {
                self.selection.remove(id);
            }
            self.prune_subselection();
            self.refresh_scene()?;
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(ids.len())
    }
}
