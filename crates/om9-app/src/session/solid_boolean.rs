// SPDX-License-Identifier: LGPL-2.1-or-later
//! Boolean, extract-surface and fillet-edge solid stages.
//!
//! Moved unchanged from `solid.rs`; these methods share `SolidPending` with the parent session
//! module and pull their faces/edges from the published B-Rep.
use super::solid::{FilletControl, FilletEdgePick};
use super::*;

impl Controller {
    pub fn solid_boolean_needs_object(&self) -> bool {
        self.solid_pending.as_ref().is_some_and(|p| {
            matches!(
                p.feature,
                "OM9-SOLID-001"
                    | "OM9-SOLID-002"
                    | "OM9-SOLID-003"
                    | "OM9-SOLID-005"
                    | "SolidBooleanSplit"
                    | "SolidCreateSolid"
            ) || (p.feature == "OM9-SOLID-004" && p.boolean_two_preview.is_none())
                || (p.feature == "OM9-SOLID-021" && p.difference_targets.is_some())
                || (p.feature == "SolidRibDevelopment"
                    && p.slab_source.is_some()
                    && p.difference_targets.is_none())
        })
    }

    pub fn solid_boolean_two_preview(&self) -> Option<(&om9_core::brep::Brep, &'static str)> {
        let preview = self.solid_pending.as_ref()?.boolean_two_preview.as_ref()?;
        Some((
            preview.body.as_ref()?,
            [
                "Union",
                "Difference",
                "ReverseDifference",
                "Intersection",
                "InversionIntersection",
            ][preview.mode],
        ))
    }

    pub fn solid_boolean_two_preview_mode(&self) -> Option<&'static str> {
        let preview = self.solid_pending.as_ref()?.boolean_two_preview.as_ref()?;
        Some(
            [
                "Union",
                "Difference",
                "ReverseDifference",
                "Intersection",
                "InversionIntersection",
            ][preview.mode],
        )
    }

    /// Presentation-only mask. It is present for empty previews as well, and
    /// never changes the document's persistent hidden or layer state.
    pub fn solid_boolean_two_hidden_sources(&self) -> Option<[om9_core::identity::ObjectId; 2]> {
        let preview = self.solid_pending.as_ref()?.boolean_two_preview.as_ref()?;
        Some([preview.sources[0].object, preview.sources[1].object])
    }

    pub(crate) fn solid_boolean_pick_context(&self) -> Option<(u64, bool)> {
        self.solid_pending
            .as_ref()
            .filter(|p| {
                matches!(
                    p.feature,
                    "OM9-SOLID-001"
                        | "OM9-SOLID-002"
                        | "OM9-SOLID-003"
                        | "OM9-SOLID-005"
                        | "SolidBooleanSplit"
                        | "SolidCreateSolid"
                ) || (p.feature == "OM9-SOLID-004" && p.boolean_two_preview.is_none())
                    || (p.feature == "OM9-SOLID-021" && p.difference_targets.is_some())
                    || (p.feature == "SolidRibDevelopment"
                        && p.slab_source.is_some()
                        && p.difference_targets.is_none())
            })
            .map(|p| (self.command_run, p.difference_targets.is_some()))
    }

    pub fn solid_difference_pick(
        &mut self,
        id: om9_core::identity::ObjectId,
        remove: bool,
    ) -> Result<()> {
        let pending = self
            .solid_pending
            .as_ref()
            .ok_or(Error::Missing("Boolean Difference"))?;
        if pending.feature != "OM9-SOLID-001" {
            return Err(Error::Invalid("not selecting Boolean Difference objects"));
        }
        self.solid_boolean_pick(id, remove)
    }

    pub fn solid_boolean_pick(
        &mut self,
        id: om9_core::identity::ObjectId,
        remove: bool,
    ) -> Result<()> {
        let pending = self
            .solid_pending
            .as_ref()
            .ok_or(Error::Missing("Boolean selection"))?;
        if !matches!(
            pending.feature,
            "OM9-SOLID-001"
                | "OM9-SOLID-002"
                | "OM9-SOLID-003"
                | "OM9-SOLID-004"
                | "OM9-SOLID-005"
                | "SolidBooleanSplit"
                | "SolidCreateSolid"
                | "OM9-SOLID-021"
                | "SolidRibDevelopment"
        ) {
            return Err(Error::Invalid("not selecting Boolean objects"));
        }
        if pending.feature == "SolidRibDevelopment"
            && (pending.slab_source.is_none() || pending.difference_targets.is_some())
        {
            return Err(Error::Invalid("not selecting Rib boundary"));
        }
        if pending.feature == "OM9-SOLID-004" && pending.boolean_two_preview.is_some() {
            return Err(Error::Invalid(
                "finish or undo the Boolean preview before selecting",
            ));
        }
        if remove {
            return self.remove_objects(&[id]);
        }
        if pending.feature == "OM9-SOLID-004"
            && self.selected_ordered().len() >= 2
            && !self.selection.contains(&id)
        {
            return Err(Error::Invalid(
                "Boolean Two Objects requires exactly two objects",
            ));
        }
        if pending.feature != "SolidBooleanSplit"
            && pending
                .difference_targets
                .as_ref()
                .is_some_and(|targets| targets.iter().any(|stamp| stamp.object == id))
        {
            return Err(Error::Invalid(
                "an object cannot belong to both Boolean sets",
            ));
        }
        if !matches!(
            self.document().entities.get(&id).map(|e| &e.geometry),
            Some(Geometry::Brep(_))
        ) {
            return Err(Error::Invalid("selection is not a B-Rep solid"));
        }
        self.add_objects(&[id])?;
        if self
            .solid_pending
            .as_ref()
            .is_some_and(|p| p.feature == "OM9-SOLID-021")
        {
            return self
                .submit_solid_text("")
                .ok_or(Error::Missing("Boss session"))?;
        }
        Ok(())
    }

    pub(super) fn difference_capture_targets(
        &self,
        ids: &[om9_core::identity::ObjectId],
    ) -> Result<Vec<om9_core::identity::SourceStamp>> {
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::array::<om9_core::identity::SourceStamp>(ids.len())?,
            self.coordinator.limits(),
        )?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        let mut targets = Vec::new();
        targets
            .try_reserve_exact(ids.len())
            .map_err(|_| Error::Budget("boolean targets"))?;
        for id in ids {
            if !matches!(
                self.document().entities.get(id).map(|e| &e.geometry),
                Some(Geometry::Brep(_))
            ) {
                return Err(Error::Invalid("selection is not a B-Rep solid"));
            }
            targets.push(self.document().stamp(*id)?);
        }
        Ok(targets)
    }

    pub fn solid_extract_needs_face(&self) -> bool {
        self.solid_pending
            .as_ref()
            .is_some_and(|p| p.feature == "OM9-SOLID-006")
    }

    pub fn solid_face_needs_pick(&self) -> bool {
        self.solid_pending.as_ref().is_some_and(|p| {
            p.feature == "OM9-SOLID-006" || (p.feature == "OM9-SOLID-029" && !p.shell_select_done)
        })
    }

    pub fn solid_fillet_needs_edge(&self) -> bool {
        self.solid_pending
            .as_ref()
            .is_some_and(|p| p.feature == "OM9-SOLID-007" && p.fillet_preview.is_none())
    }

    pub fn solid_fillet_show_radius(&self) -> bool {
        self.solid_pending
            .as_ref()
            .is_some_and(|p| p.feature == "OM9-SOLID-007" && p.fillet_show_radius)
    }

    pub fn solid_fillet_preview(
        &self,
    ) -> Option<(&om9_core::brep::Brep, om9_core::geometry::Transform)> {
        let preview = self.solid_pending.as_ref()?.fillet_preview.as_ref()?;
        Some((&preview.body, preview.placement))
    }

    pub fn solid_fillet_current_radius(&self) -> Option<f64> {
        let pending = self.solid_pending.as_ref()?;
        if pending.fillet_add_mode {
            return pending.fillet_add_radius;
        }
        let handle = pending.fillet_handle?;
        let endpoint_count = pending.fillet_edges.len().checked_mul(2)?;
        if handle >= endpoint_count {
            return pending
                .fillet_controls
                .get(handle - endpoint_count)
                .map(|c| c.radius);
        }
        pending.fillet_edges.get(handle / 2).map(|edge| {
            if handle % 2 == 0 {
                edge.start_radius
            } else {
                edge.end_radius
            }
        })
    }

    pub fn solid_fillet_handles(&self) -> Vec<(usize, om9_core::geometry::Point3, f64, bool)> {
        let Some(pending) = self
            .solid_pending
            .as_ref()
            .filter(|p| p.fillet_preview.is_some())
        else {
            return Vec::new();
        };
        let Some(preview) = &pending.fillet_preview else {
            return Vec::new();
        };
        let Some(entity) = self.document().entities.get(&preview.source.object) else {
            return Vec::new();
        };
        let Geometry::Brep(brep) = &entity.geometry else {
            return Vec::new();
        };
        let mut handles = Vec::new();
        for (index, edge) in pending.fillet_edges.iter().enumerate() {
            if let Some((a, b)) = edge.endpoints {
                for (side, point, radius) in [(0, a, edge.start_radius), (1, b, edge.end_radius)] {
                    if let Ok(world) = preview.placement.point(point) {
                        let handle = index * 2 + side;
                        handles.push((
                            handle,
                            world,
                            radius,
                            pending.fillet_handle == Some(handle),
                        ));
                    }
                }
                continue;
            }
            let sampled = (|| {
                let om9_scene::scene::Subelement::BrepEdge(key) = edge.reference.subset else {
                    return None;
                };
                let edge_index = brep.display.edges.iter().position(|id| *id == key)?;
                let count = brep
                    .display
                    .segments
                    .iter()
                    .filter(|segment| segment[2] as usize == edge_index)
                    .count();
                let segment = brep
                    .display
                    .segments
                    .iter()
                    .filter(|segment| segment[2] as usize == edge_index)
                    .nth(count / 2)?;
                let a = brep.display.points.get(segment[0] as usize)?;
                let b = brep.display.points.get(segment[1] as usize)?;
                let midpoint = std::array::from_fn(|i| (a[i] + b[i]) * 0.5);
                let world = preview.placement.point(midpoint).ok()?;
                let handle = index * 2;
                Some((
                    handle,
                    world,
                    edge.start_radius,
                    pending.fillet_handle == Some(handle),
                ))
            })();
            if let Some(handle) = sampled {
                handles.push(handle);
            }
        }
        let base = pending.fillet_edges.len().saturating_mul(2);
        for (index, control) in pending.fillet_controls.iter().enumerate() {
            if let Ok(world) = preview.placement.point(control.point) {
                let handle = base + index;
                handles.push((
                    handle,
                    world,
                    control.radius,
                    pending.fillet_handle == Some(handle),
                ));
            }
        }
        handles
    }

    pub fn solid_fillet_adding_handle(&self) -> bool {
        self.solid_pending.as_ref().is_some_and(|p| {
            p.feature == "OM9-SOLID-007" && p.fillet_preview.is_some() && p.fillet_add_mode
        })
    }

    pub fn solid_fillet_copying_to_target(&self) -> bool {
        self.solid_pending
            .as_ref()
            .is_some_and(|p| p.feature == "OM9-SOLID-007" && p.fillet_copy_phase == 2)
    }

    pub fn solid_fillet_copy_to_hit(&mut self, hits: &[(usize, f32)]) -> Result<()> {
        let pending = self
            .solid_pending
            .as_mut()
            .filter(|p| p.fillet_copy_phase == 2 && p.fillet_add_mode)
            .ok_or(Error::Missing("CopyHandle target"))?;
        let radius = pending
            .fillet_add_radius
            .ok_or(Error::Missing("CopyHandle source"))?;
        pending.fillet_copy_phase = 0;
        pending.fillet_add_mode = false;
        let result = self.solid_fillet_select_hit(hits).and_then(|_| {
            self.submit_solid_text(&format!("CurrentRadius={radius}"))
                .ok_or(Error::Missing("fillet session"))?
        });
        if let Some(pending) = self.solid_pending.as_mut() {
            pending.fillet_copy_phase = 2;
            pending.fillet_add_mode = true;
            pending.fillet_add_radius = Some(radius);
            pending.fillet_handle = None;
        }
        result
    }

    pub fn solid_fillet_add_handle(
        &mut self,
        reference: super::SelectedSubobject,
        world: om9_core::geometry::Point3,
    ) -> Result<()> {
        let mut pending = self
            .solid_pending
            .take()
            .ok_or(Error::Missing("fillet session"))?;
        let result = (|| {
            if !pending.fillet_add_mode {
                return Err(Error::Missing("AddHandle mode"));
            }
            let preview = pending
                .fillet_preview
                .as_ref()
                .ok_or(Error::Missing("fillet preview"))?;
            if self.document().stamp(preview.source.object)? != preview.source
                || !self.selection.contains(&preview.source.object)
            {
                return Err(Error::Stale("fillet preview source"));
            }
            let placement = preview.placement;
            let edge = pending
                .fillet_edges
                .iter()
                .position(|item| {
                    item.reference.root == reference.root
                        && item.reference.path == reference.path
                        && item.reference.subset == reference.subset
                        && item.reference.source == reference.source
                })
                .ok_or(Error::Invalid("pick a selected fillet edge"))?;
            if !self.subselection().contains(&reference) {
                return Err(Error::Stale("fillet edge selection"));
            }
            self.select_subobject_ref(reference.clone())?;
            let radius = pending
                .fillet_add_radius
                .ok_or(Error::Missing("set CurrentRadius"))?;
            let (fraction, point) = self.solid_fillet_project_pick(&reference, placement, world)?;
            if fraction <= 1e-9
                || fraction >= 1.0 - 1e-9
                || pending.fillet_controls.iter().any(|control| {
                    control.edge == edge && (control.fraction - fraction).abs() < 1e-9
                })
            {
                return Err(Error::Invalid("select a distinct interior edge location"));
            }
            if pending.fillet_controls.len() as u64 >= self.coordinator.limits().max_outputs {
                return Err(Error::Budget("fillet controls"));
            }
            let mut held = self.scene_cache_usage()?;
            held.add(pending.usage()?, self.coordinator.limits())?;
            held.add(
                om9_core::budget::Usage::array::<FilletControl>(1)?,
                self.coordinator.limits(),
            )?;
            self.coordinator.retained_usage_with_held_usage(held)?;
            pending
                .fillet_controls
                .try_reserve(1)
                .map_err(|_| Error::Budget("fillet controls"))?;
            pending.fillet_controls.push(FilletControl {
                edge,
                fraction,
                point,
                radius,
            });
            let rebuilt = self
                .solid_fillet_build_preview(&pending.fillet_edges, &pending.fillet_controls)
                .and_then(|new_preview| {
                    let mut held = self.scene_cache_usage()?;
                    held.add(pending.usage()?, self.coordinator.limits())?;
                    held.add(new_preview.usage, self.coordinator.limits())?;
                    self.coordinator.retained_usage_with_held_usage(held)?;
                    Ok(new_preview)
                });
            match rebuilt {
                Ok(new_preview) => {
                    pending.fillet_preview = Some(new_preview);
                    Ok(())
                }
                Err(error) => {
                    pending.fillet_controls.pop();
                    Err(error)
                }
            }
        })();
        self.solid_pending = Some(pending);
        result
    }

    pub fn solid_fillet_select_handle(&mut self, index: usize) -> Result<()> {
        let pending = self
            .solid_pending
            .as_ref()
            .filter(|p| p.fillet_preview.is_some())
            .ok_or(Error::Missing("fillet preview"))?;
        let preview = pending.fillet_preview.as_ref().expect("preview");
        if self.document().stamp(preview.source.object)? != preview.source
            || !self.selection.contains(&preview.source.object)
        {
            return Err(Error::Stale("fillet preview source"));
        }
        if !self
            .solid_fillet_handles()
            .iter()
            .any(|item| item.0 == index)
        {
            return Err(Error::Invalid("fillet handle"));
        }
        if pending.fillet_copy_phase == 1 {
            let radius = self
                .solid_fillet_handles()
                .iter()
                .find(|item| item.0 == index)
                .map(|item| item.2)
                .ok_or(Error::Invalid("fillet handle"))?;
            let pending = self.solid_pending.as_mut().expect("fillet session");
            pending.fillet_add_radius = Some(radius);
            pending.fillet_copy_phase = 2;
            pending.fillet_add_mode = true;
            pending.fillet_handle = None;
            return Ok(());
        }
        if pending.fillet_remove_mode {
            return self.solid_fillet_remove_handle(index);
        }
        self.solid_pending
            .as_mut()
            .expect("fillet session")
            .fillet_handle = Some(index);
        Ok(())
    }

    pub(super) fn solid_fillet_remove_handle(&mut self, index: usize) -> Result<()> {
        let mut pending = self
            .solid_pending
            .take()
            .ok_or(Error::Missing("fillet session"))?;
        let result = (|| {
            if !pending.fillet_remove_mode {
                return Err(Error::Missing("RemoveHandle mode"));
            }
            let preview = pending
                .fillet_preview
                .as_ref()
                .ok_or(Error::Missing("fillet preview"))?;
            if self.document().stamp(preview.source.object)? != preview.source
                || !self.selection.contains(&preview.source.object)
            {
                return Err(Error::Stale("fillet preview source"));
            }
            let base = pending
                .fillet_edges
                .len()
                .checked_mul(2)
                .ok_or(Error::Overflow("fillet handle"))?;
            let control = index
                .checked_sub(base)
                .filter(|&i| i < pending.fillet_controls.len())
                .ok_or(Error::Invalid("only added handles can be removed"))?;
            for edge in &pending.fillet_edges {
                self.select_subobject_ref(edge.reference.clone())?;
            }
            let removed = pending.fillet_controls.remove(control);
            let rebuilt = self
                .solid_fillet_build_preview(&pending.fillet_edges, &pending.fillet_controls)
                .and_then(|new_preview| {
                    let mut held = self.scene_cache_usage()?;
                    held.add(pending.usage()?, self.coordinator.limits())?;
                    held.add(new_preview.usage, self.coordinator.limits())?;
                    self.coordinator.retained_usage_with_held_usage(held)?;
                    Ok(new_preview)
                });
            match rebuilt {
                Ok(new_preview) => {
                    pending.fillet_preview = Some(new_preview);
                    pending.fillet_handle = None;
                    Ok(())
                }
                Err(error) => {
                    pending.fillet_controls.insert(control, removed);
                    Err(error)
                }
            }
        })();
        self.solid_pending = Some(pending);
        result
    }

    pub fn solid_fillet_select_hit(&mut self, hits: &[(usize, f32)]) -> Result<()> {
        let &(first, distance) = hits
            .first()
            .ok_or(Error::Missing("fillet handle under the pointer"))?;
        if !distance.is_finite() {
            return Err(Error::Invalid("fillet handle distance"));
        }
        if hits
            .iter()
            .skip(1)
            .any(|hit| hit.1.is_finite() && (hit.1 - distance).abs() < 1.0)
        {
            let pending = self
                .solid_pending
                .as_ref()
                .filter(|p| p.fillet_preview.is_some())
                .ok_or(Error::Missing("fillet preview"))?;
            if first >= pending.fillet_edges.len().saturating_mul(2) {
                return Err(Error::Invalid("ambiguous fillet handles"));
            }
            let preview = pending.fillet_preview.as_ref().expect("fillet preview");
            if self.document().stamp(preview.source.object)? != preview.source
                || !self.selection.contains(&preview.source.object)
            {
                return Err(Error::Stale("fillet preview source"));
            }
            let mut held = self.scene_cache_usage()?;
            held.add(preview.usage, self.coordinator.limits())?;
            held.add(
                om9_core::budget::Usage::array::<usize>(pending.fillet_edges.len())?,
                self.coordinator.limits(),
            )?;
            held.add(
                om9_core::budget::Usage::array::<i32>(pending.fillet_edges.len())?,
                self.coordinator.limits(),
            )?;
            held.add(
                om9_core::budget::Usage::array::<u8>(pending.fillet_edges.len())?,
                self.coordinator.limits(),
            )?;
            self.coordinator.retained_usage_with_held_usage(held)?;
            let masks = self.solid_fillet_shared_endpoint_masks(&pending.fillet_edges, first)?;
            for &(candidate, other_distance) in hits.iter().skip(1) {
                if !other_distance.is_finite() {
                    return Err(Error::Invalid("fillet handle distance"));
                }
                if (other_distance - distance).abs() >= 1.0 {
                    continue;
                }
                if masks
                    .get(candidate / 2)
                    .is_none_or(|mask| mask & (1 << (candidate % 2)) == 0)
                {
                    return Err(Error::Invalid("ambiguous fillet handles"));
                }
            }
        }
        self.solid_fillet_select_handle(first)
    }

    pub fn solid_fillet_pick_edge(&mut self, reference: super::SelectedSubobject) -> Result<()> {
        let pending = self
            .solid_pending
            .as_ref()
            .filter(|p| p.feature == "OM9-SOLID-007" && p.fillet_preview.is_none())
            .ok_or(Error::Missing("Fillet Edge selection"))?;
        if pending.fillet_radius.is_none() {
            return Err(Error::Missing("set NextRadius before choosing edges"));
        }
        if !reference.path.is_empty() {
            return Err(Error::Unsupported(
                "filleting an instance edge is not supported",
            ));
        }
        if !matches!(reference.subset, om9_scene::scene::Subelement::BrepEdge(_)) {
            return Err(Error::Invalid("select a B-Rep edge"));
        }
        if pending
            .fillet_edges
            .iter()
            .any(|edge| edge.reference.root != reference.root)
        {
            return Err(Error::Unsupported("fillet edges must belong to one object"));
        }
        if pending.fillet_edges.iter().any(|edge| {
            edge.reference.root == reference.root && edge.reference.subset == reference.subset
        }) {
            return Ok(());
        }
        let count = pending.fillet_edges.len();
        if count as u64 >= self.coordinator.limits().max_outputs {
            return Err(Error::Budget("fillet edges"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::array::<FilletEdgePick>(count + 1)?,
            self.coordinator.limits(),
        )?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        self.solid_pending
            .as_mut()
            .expect("fillet session")
            .fillet_edges
            .try_reserve(1)
            .map_err(|_| Error::Budget("fillet edges"))?;
        let endpoints = self.solid_fillet_edge_endpoints(&reference)?;
        self.select_subobject_ref(reference.clone())?;
        self.select_objects(&[reference.root])?;
        let pending = self.solid_pending.as_mut().expect("fillet session");
        let radius = pending.fillet_radius.expect("radius validated");
        pending.fillet_edges.push(FilletEdgePick {
            reference,
            start_radius: radius,
            end_radius: radius,
            endpoints,
        });
        Ok(())
    }

    pub fn solid_extract_pick_context(&self) -> Option<u64> {
        self.solid_face_needs_pick().then_some(self.command_run)
    }

    pub fn solid_extract_pick_face(&mut self, reference: super::SelectedSubobject) -> Result<()> {
        if !self.solid_face_needs_pick() {
            return Err(Error::Missing("face selection session"));
        }
        if !reference.path.is_empty() {
            return Err(Error::Unsupported(
                "extracting an instance face is not supported",
            ));
        }
        if !matches!(reference.subset, om9_scene::scene::Subelement::BrepFace(_)) {
            return Err(Error::Invalid("select a B-Rep face"));
        }
        if self.solid_pending.as_ref().is_some_and(|p| {
            p.feature == "OM9-SOLID-029"
                && p.extract_face_picks
                    .iter()
                    .any(|old| old.root != reference.root)
        }) {
            return Err(Error::Unsupported("shell faces must belong to one object"));
        }
        if self
            .solid_pending
            .as_ref()
            .expect("extract session")
            .extract_face_picks
            .iter()
            .any(|old| {
                old.root == reference.root
                    && old.path == reference.path
                    && old.subset == reference.subset
            })
        {
            return Ok(());
        }
        let count = self
            .solid_pending
            .as_ref()
            .expect("extract session")
            .extract_face_picks
            .len();
        if count as u64 >= self.coordinator.limits().max_outputs {
            return Err(Error::Budget("extract face count"));
        }
        let mut held = self.scene_cache_usage()?;
        held.add(
            om9_core::budget::Usage::array::<super::SelectedSubobject>(count + 1)?,
            self.coordinator.limits(),
        )?;
        self.coordinator.retained_usage_with_held_usage(held)?;
        self.solid_pending
            .as_mut()
            .expect("extract session")
            .extract_face_picks
            .try_reserve(1)
            .map_err(|_| Error::Budget("extract face selection"))?;
        self.select_subobject_ref(reference.clone())?;
        self.add_objects(&[reference.root])?;
        self.solid_pending
            .as_mut()
            .expect("extract session")
            .extract_face_picks
            .push(reference);
        Ok(())
    }

    pub fn solid_extract_remove_face(&mut self, reference: super::SelectedSubobject) -> Result<()> {
        let pending = self
            .solid_pending
            .as_ref()
            .filter(|p| {
                p.feature == "OM9-SOLID-006"
                    || (p.feature == "OM9-SOLID-029" && !p.shell_select_done)
            })
            .ok_or(Error::Missing("Extract Surface session"))?;
        let index = pending
            .extract_face_picks
            .iter()
            .position(|old| {
                old.root == reference.root
                    && old.path == reference.path
                    && old.subset == reference.subset
            })
            .ok_or(Error::Missing("face is not selected"))?;
        if !self.subselection().contains(&reference) {
            return Err(Error::Stale("extract face selection"));
        }
        self.toggle_subobject_ref(reference)?;
        self.solid_pending
            .as_mut()
            .expect("extract session")
            .extract_face_picks
            .remove(index);
        Ok(())
    }
}
