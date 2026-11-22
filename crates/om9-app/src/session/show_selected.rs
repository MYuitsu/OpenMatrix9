// SPDX-License-Identifier: LGPL-2.1-or-later
use super::{Controller, selection_memory::ReservedRootSelection};
use om9_core::{Error, Result, budget::memory::MemoryClass};
use om9_scene::scene::Scene;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PickOwners {
    Hidden,
    PointsOn,
}

impl PickOwners {
    pub(super) fn prompt(self) -> &'static str {
        match self {
            Self::Hidden => "Select objects to show, Enter when done",
            Self::PointsOn => "Select objects to turn points off, Enter when done",
        }
    }
}

pub(super) struct IsolatedSelection {
    pub(super) kind: PickOwners,
    original: Scene,
    selection: ReservedRootSelection,
    preview_id: uuid::Uuid,
}

impl Controller {
    pub(crate) fn isolating_objects(&self) -> bool {
        self.isolated_selection.is_some()
    }

    pub(super) fn start_isolated_selection(&mut self, kind: PickOwners) -> Result<()> {
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("isolated picking scene"))?;
        if scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("isolated picking scene"));
        }
        if kind == PickOwners::Hidden
            && !self
                .document()
                .entities
                .values()
                .any(|entity| entity.attributes.hidden)
        {
            return Err(Error::Missing("no hidden objects to show"));
        }
        let mut memory = self.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache,
            scene
                .render_snapshot_metadata_estimate()?
                .bytes
                .checked_add(scene.picking_arena_bytes()?)
                .ok_or(Error::Overflow("isolated picking memory"))?,
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)),
            self.scene_cache_usage()?,
        )?;
        let preview = match kind {
            PickOwners::Hidden => scene.hidden_selection_snapshot()?,
            PickOwners::PointsOn => {
                scene.grip_owner_selection_snapshot(|id| self.grip_owner_mode(id).is_some())?
            }
        };
        let actual = preview.managed_memory_allowance()?;
        if actual > memory.bytes() {
            let additional = actual - memory.bytes();
            self.coordinator.grow_external_memory_with_held(
                &mut memory,
                additional,
                om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)),
                self.scene_cache_usage()?,
            )?;
        }
        preview.transfer_memory_allowance(
            &mut memory,
            &self.coordinator,
            self.scene_cache_usage()?,
        )?;
        if !preview
            .objects
            .iter()
            .any(|o| o.selectable && o.has_pick_geometry())
        {
            return Err(Error::Missing(match kind {
                PickOwners::Hidden => "no selectable hidden objects to show",
                PickOwners::PointsOn => "no objects have points turned on",
            }));
        }
        let selection = self.prepare_root_selection(scene, &self.selected_ordered(), false)?;
        let preview_id = preview.publication_id();
        let original = self.scene.replace(preview).expect("scene validated above");
        self.clear_selection();
        self.isolated_selection = Some(IsolatedSelection {
            kind,
            original,
            selection,
            preview_id,
        });
        Ok(())
    }

    pub(super) fn isolated_selection_current(&self) -> bool {
        self.isolated_selection
            .as_ref()
            .zip(self.scene.as_ref())
            .is_some_and(|(pending, scene)| {
                scene.publication_id() == pending.preview_id
                    && scene.epoch == self.coordinator.epoch()
            })
    }

    pub(super) fn restore_isolated_selection(&mut self) {
        let current = self.isolated_selection_current();
        if let Some(pending) = self.isolated_selection.take() {
            if current {
                self.scene = Some(pending.original);
                self.publish_root_selection(pending.selection);
            } else {
                self.prune_selection();
            }
        }
    }

    pub(super) fn finish_isolated_selection(&mut self) -> Result<()> {
        if !self.isolated_selection_current() {
            self.restore_isolated_selection();
            return Err(Error::Stale("isolated picking preview"));
        }
        let kind = self
            .isolated_selection
            .as_ref()
            .expect("validated selection")
            .kind;
        let ids = self.selected_ordered();
        self.restore_isolated_selection();
        match kind {
            PickOwners::Hidden if !ids.is_empty() => {
                let sources = ids
                    .iter()
                    .map(|id| self.document().stamp(*id))
                    .collect::<Result<Vec<_>>>()?;
                self.set_objects_hidden(sources, false)?;
            }
            PickOwners::PointsOn => self.turn_off_owner_grips(&ids),
            PickOwners::Hidden => {}
        }
        Ok(())
    }
}
