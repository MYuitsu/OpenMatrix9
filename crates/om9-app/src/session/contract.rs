// SPDX-License-Identifier: LGPL-2.1-or-later
//! Document contract host: selection, change notification and snapshots
//! (OM9-L3-01-03-010, -011, -012).
//!
//! Observers receive an owned, ordered `ChangeBatch` after the document is
//! final. They never receive the controller, so a callback cannot start a
//! reentrant mutation; a panicking observer is recorded as a diagnostic and the
//! remaining observers still receive the batch.
use om9_archive::snapshot::DocumentSnapshot;
use om9_core::{
    Error, Result,
    budget::Usage,
    changes::{ChangeBatch, ChangeCause},
    selection::SelectionState,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// A document change sink. The batch is borrowed and owned by the caller; the
/// observer can record or forward it, never mutate the document.
pub trait DocumentObserver {
    fn document_changed(&mut self, batch: &ChangeBatch) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObserverDiagnostic {
    pub observer: usize,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotOutcome {
    Restored,
}

/// The document/scene seam a docking layout reads (`OM9-L2-01-03`).
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentSceneState {
    pub document: om9_core::identity::DocumentId,
    pub owner_session: om9_core::identity::SessionId,
    pub revision: u64,
    pub publication_epoch: u64,
    pub scene_epoch: Option<u64>,
    pub scene_objects: usize,
    pub entities: usize,
    pub layers: usize,
    pub views: Vec<om9_scene::workspace::ViewId>,
}

/// The scene host is the controller itself, so a layout can never be built from
/// a scene that the document owner does not know about. The output is identical
/// to the `Workspace` implementation.
impl crate::workbench_regions::ViewportSceneHost for super::Controller {
    fn visible_views(&self) -> Vec<om9_scene::workspace::ViewId> {
        self.workspace.visible_views()
    }
}

/// Published change batches kept for the current session. The ring is bounded,
/// so a long edit session cannot grow without limit.
pub const CHANGE_RING: usize = 64;

impl super::Controller {
    /// The live selection as the document contract describes it.
    pub fn selection_state(&self) -> SelectionState {
        let mut state = SelectionState::new(self.document());
        state.objects = self.selection.clone();
        state.revision = self.selection_revision.max(1);
        state
    }
    pub fn validate_selection(&self, state: &SelectionState) -> Result<()> {
        state.validate(self.document())
    }
    /// Drop selected objects that no longer exist (OM9-L3-01-03-010). Returns
    /// true when the selection changed, which is the caller's redraw signal.
    pub fn prune_selection(&mut self) -> bool {
        let stale: Vec<om9_core::identity::ObjectId> = {
            let document = self.document();
            self.selection
                .iter()
                .copied()
                .filter(|id| !document.entities.contains_key(id))
                .collect()
        };
        if stale.is_empty() {
            return false;
        }
        for id in stale {
            self.selection.remove(&id);
        }
        self.selection_order.retain(|id| self.selection.contains(id));
        self.selection_revision = self.selection_revision.saturating_add(1);
        true
    }
    /// Subscribe a document change sink. Recording starts with the first
    /// subscription: a controller nobody observes does not retain a second copy
    /// of the document (a large document would otherwise double its memory for a
    /// batch no one reads). The baseline is the world's own `Arc`, so recording
    /// never copies the document.
    pub fn add_document_observer(&mut self, observer: Box<dyn DocumentObserver>) -> usize {
        if self.published.is_none() {
            self.published = Some(self.coordinator.world().documents[&self.document].clone());
        }
        self.observers.push(observer);
        self.observers.len() - 1
    }
    /// Keep the typed change ring for readers that poll it instead of subscribing (the MCP
    /// text surface). The sink itself does nothing; the retained baseline is what produces
    /// `last_document_change`. It diffs the document on every publication, which is why the
    /// interactive application never enables it by default.
    pub fn record_changes(&mut self) {
        struct Keep;
        impl DocumentObserver for Keep {
            fn document_changed(&mut self, _batch: &ChangeBatch) -> Result<()> {
                Ok(())
            }
        }
        if !self.is_recording_changes() {
            self.add_document_observer(Box::new(Keep));
        }
    }
    /// True while a baseline is retained, i.e. while changes are being recorded.
    pub fn is_recording_changes(&self) -> bool {
        self.published.is_some()
    }
    pub fn observer_count(&self) -> usize {
        self.observers.len()
    }
    pub fn observer_diagnostics(&self) -> &[ObserverDiagnostic] {
        &self.observer_diagnostics
    }
    pub fn document_changes(&self) -> impl Iterator<Item = &ChangeBatch> {
        self.changes.iter()
    }
    pub fn last_document_change(&self) -> Option<&ChangeBatch> {
        self.changes.back()
    }
    pub fn take_document_changes(&mut self) -> Vec<ChangeBatch> {
        self.changes.drain(..).collect()
    }
    /// Diff the live document against the last published baseline and publish
    /// one ordered batch when anything changed. With no baseline (nobody is
    /// observing) the document is not copied and no batch is produced; the
    /// selection is still pruned, because a stale entry is a correctness issue
    /// whether or not a batch is being recorded. The baseline is captured before
    /// any observer runs, so a callback cannot extend the batch it is reading.
    pub(crate) fn sync_published(&mut self, cause: ChangeCause) {
        self.prune_selection();
        self.prune_subselection();
        let Some(previous) = self.published.take() else {
            return;
        };
        // The world owns the live revision, so this is an `Arc` clone.
        let current = self.coordinator.world().documents[&self.document].clone();
        self.published = Some(current.clone());
        let Ok(Some(batch)) = ChangeBatch::between(&previous, &current, cause) else {
            return;
        };
        if batch.is_empty() {
            return;
        }
        if self.changes.len() >= CHANGE_RING {
            self.changes.pop_front();
        }
        self.changes.push_back(batch.clone());
        // A failed observer (panic or error) is detached after one diagnostic,
        // so a broken sink cannot stop every later publication. Observers that
        // survive still receive every batch.
        let mut broken = Vec::new();
        for (index, observer) in self.observers.iter_mut().enumerate() {
            let outcome = catch_unwind(AssertUnwindSafe(|| observer.document_changed(&batch)))
                .map_err(|_| "observer panicked".to_string())
                .and_then(|result| result.map_err(|error| error.to_string()));
            if let Err(detail) = outcome {
                self.observer_diagnostics.push(ObserverDiagnostic {
                    observer: index,
                    detail,
                });
                broken.push(index);
            }
        }
        for index in broken.into_iter().rev() {
            self.observers.remove(index);
        }
    }
    /// Move the contract state (observers, diagnostics, change ring) onto a
    /// replacement controller and keep the replaced document as the diff
    /// baseline, so a replacement is published as one ordered batch.
    pub(crate) fn adopt_contract_state(&mut self, previous: &mut Self) {
        self.observers = std::mem::take(&mut previous.observers);
        self.observer_diagnostics = std::mem::take(&mut previous.observer_diagnostics);
        self.changes = std::mem::take(&mut previous.changes);
        // Recording continues across a replacement only if it was already on, and
        // the replaced revision is the baseline (an `Arc`, never a copy).
        self.published = previous.published.take();
    }
    /// Charge the contract state that lives beside the scene cache. Empty state
    /// charges nothing, so a fresh controller keeps its previous budget.
    pub(crate) fn contract_usage(&self) -> Result<Usage> {
        let limits = *self.coordinator.limits();
        let mut usage = Usage::array::<ChangeBatch>(self.changes.capacity())?;
        for batch in &self.changes {
            usage.add(
                Usage::array::<om9_core::changes::Change>(batch.changes.capacity())?,
                &limits,
            )?;
        }
        usage.add(
            Usage::array::<Box<dyn DocumentObserver>>(self.observers.capacity())?,
            &limits,
        )?;
        usage.add(
            Usage::array::<ObserverDiagnostic>(self.observer_diagnostics.capacity())?,
            &limits,
        )?;
        for diagnostic in &self.observer_diagnostics {
            usage.add(
                Usage {
                    bytes: diagnostic.detail.capacity() as u64,
                    ..Usage::default()
                },
                &limits,
            )?;
        }
        usage.add(Usage::array::<SelectionState>(1)?, &limits)?;
        Ok(usage)
    }
    pub fn capture_snapshot(&self) -> Result<DocumentSnapshot> {
        DocumentSnapshot::capture(self.document(), self.coordinator.limits())
    }
    /// The document/scene facts the docking regions and the panels need. The
    /// scene epoch is `None` exactly when no scene is presented, which is the
    /// state a failed refresh leaves behind.
    pub fn document_scene_state(&self) -> DocumentSceneState {
        let document = self.document();
        DocumentSceneState {
            document: document.id,
            owner_session: document.owner_session,
            revision: document.revision,
            publication_epoch: self.coordinator.epoch(),
            scene_epoch: self.scene.as_ref().map(|scene| scene.epoch),
            scene_objects: self.scene.as_ref().map_or(0, |scene| scene.objects.len()),
            entities: document.entities.len(),
            layers: document.layers.len(),
            views: self.workspace.visible_views(),
        }
    }
    /// Replace the whole document from a validated snapshot. Atomic: either the
    /// document and the scene are both rebuilt, or nothing changes. Undo is
    /// reset because the snapshot is a new baseline, and the workspace and the
    /// current file path are kept.
    pub fn restore_snapshot(&mut self, snapshot: &DocumentSnapshot) -> Result<SnapshotOutcome> {
        if self.is_running() || self.file.operation.is_some() {
            return Err(Error::Busy);
        }
        let limits = *self.coordinator.limits();
        let held = self.replacement_hold(Usage {
            bytes: 512,
            ..Usage::default()
        })?;
        let document = snapshot.restore(&limits.remaining_after(held)?)?;
        if document.id != self.document {
            return Err(Error::Invalid("snapshot document"));
        }
        let workspace = self.workspace.clone();
        let path = self.file.path.clone();
        let mut candidate = Self::new_with_held(document, limits, held)?;
        candidate.workspace = workspace;
        candidate.file.path = path;
        candidate.adopt_contract_state(self);
        *self = candidate;
        self.sync_published(ChangeCause::Restore);
        Ok(SnapshotOutcome::Restored)
    }
    /// The document a caller would save right now, for provenance queries.
    pub fn document_provenance(
        &self,
        id: om9_core::identity::ObjectId,
    ) -> Option<om9_core::provenance::Provenance> {
        self.document().provenance(id)
    }
    pub fn import_summary(&self) -> om9_core::provenance::ImportSummary {
        self.document().import_summary()
    }
}
