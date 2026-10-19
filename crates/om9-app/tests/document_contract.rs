// SPDX-License-Identifier: LGPL-2.1-or-later
//! Document contract at the controller boundary:
//! OM9-L3-01-03-010 (selection), -011 (change notification), -012 (snapshot).
use om9_app::session::{
    Controller,
    contract::{DocumentObserver, SnapshotOutcome},
};
use om9_core::{
    budget::Limits,
    changes::{Change, ChangeCause},
    identity::ObjectId,
    model::Document,
    selection::SelectionState,
};
use std::sync::{Arc, Mutex};

fn controller() -> Controller {
    Controller::new(Document::new("contract"), Limits::default()).unwrap()
}

/// Create one point object and return its id.
fn one_point(c: &mut Controller) -> ObjectId {
    c.submit("Point").unwrap();
    c.submit("1,2,3").unwrap();
    *c.document()
        .entities
        .keys()
        .next()
        .expect("the Point command published one object")
}

/// One recorded batch: cause, ordered changes, from-revision, to-revision.
type RecordedBatch = (ChangeCause, Vec<Change>, u64, u64);

#[derive(Clone, Default)]
struct Recorder(Arc<Mutex<Vec<RecordedBatch>>>);

impl DocumentObserver for Recorder {
    fn document_changed(&mut self, batch: &om9_core::changes::ChangeBatch) -> om9_core::Result<()> {
        self.0.lock().unwrap().push((
            batch.cause,
            batch.changes.clone(),
            batch.from_revision,
            batch.to_revision,
        ));
        Ok(())
    }
}

struct Panicking;

impl DocumentObserver for Panicking {
    fn document_changed(
        &mut self,
        _batch: &om9_core::changes::ChangeBatch,
    ) -> om9_core::Result<()> {
        panic!("observer failure must not abort the publication");
    }
}

struct Failing;

impl DocumentObserver for Failing {
    fn document_changed(
        &mut self,
        _batch: &om9_core::changes::ChangeBatch,
    ) -> om9_core::Result<()> {
        Err(om9_core::Error::Io("observer refused".into()))
    }
}

/* ------------------------------ L3-010 ------------------------------ */

#[test]
fn l3_010_selection_is_pruned_after_undo_and_redo_restores_the_object() {
    let mut c = controller();
    let id = one_point(&mut c);
    c.select_root(Some(id), false).unwrap();
    assert!(c.selected().contains(&id));
    let state = c.selection_state();
    c.validate_selection(&state).unwrap();
    c.undo().unwrap();
    assert!(
        c.selected().is_empty(),
        "an object that no longer exists must not stay selected"
    );
    assert!(
        !c.prune_selection(),
        "the selection is already clean after the Undo"
    );
    c.redo().unwrap();
    assert!(c.document().entities.contains_key(&id));
    assert!(
        c.selected().is_empty(),
        "Redo restores the object, never a stale selection entry"
    );
}

#[test]
fn l3_010_a_selection_from_another_document_generation_is_stale() {
    let mut c = controller();
    let id = one_point(&mut c);
    c.select_root(Some(id), false).unwrap();
    let captured = c.selection_state();
    let other = controller();
    assert_eq!(
        other.validate_selection(&captured),
        Err(om9_core::Error::Stale("selection document")),
        "a selection captured before New/Open is never accepted by the new document"
    );
    let mut empty = SelectionState::new(other.document());
    empty.revision = 0;
    assert_eq!(
        empty.validate(other.document()),
        Err(om9_core::Error::Invalid("selection revision"))
    );
}

#[test]
fn l3_010_a_stale_list_index_is_detected_instead_of_mapped_to_a_neighbour() {
    let mut c = controller();
    let first = one_point(&mut c);
    c.submit("Point").unwrap();
    c.submit("9,9,9").unwrap();
    let mut ids: Vec<ObjectId> = c.document().entities.keys().copied().collect();
    ids.sort_unstable();
    for id in &ids {
        c.select_root(Some(*id), true).unwrap();
    }
    assert_eq!(c.selected().len(), 2);
    let state = c.selection_state();
    let published = state.ordered();
    assert_eq!(published.len(), 2);
    assert!(state.resolve_index(&published, 0).is_ok());
    let mut reversed = published.clone();
    reversed.reverse();
    assert_eq!(
        state.resolve_index(&reversed, 0),
        Err(om9_core::Error::Stale("selection index"))
    );
    c.undo().unwrap();
    // After the Undo the published list is one entry shorter, so the old index
    // must be detected as stale rather than resolving to the survivor.
    let state = c.selection_state();
    assert_eq!(
        state.resolve_index(&published, 1),
        Err(om9_core::Error::Stale("selection index"))
    );
    assert!(c.selected().contains(&first) || !c.document().entities.contains_key(&first));
}

/* ------------------------------ L3-011 ------------------------------ */

#[test]
fn l3_011_one_edit_publishes_one_ordered_batch_with_a_new_revision() {
    let mut c = controller();
    let recorder = Recorder::default();
    c.add_document_observer(Box::new(recorder.clone()));
    let id = one_point(&mut c);
    let batches = recorder.0.lock().unwrap().clone();
    assert_eq!(batches.len(), 1, "one accepted edit is one batch");
    let (cause, changes, from, to) = &batches[0];
    assert_eq!(*cause, ChangeCause::Edit);
    assert_eq!(changes, &vec![Change::EntityAdded(id)]);
    assert!(to > from, "the batch reports the revision increase");
    assert_eq!(c.last_document_change().unwrap().to_revision, *to);
    assert_eq!(c.document_changes().count(), 1);
    c.take_document_changes();
    assert_eq!(c.document_changes().count(), 0);
}

#[test]
fn l3_011_undo_and_redo_are_published_with_their_own_cause() {
    let mut c = controller();
    let recorder = Recorder::default();
    c.add_document_observer(Box::new(recorder.clone()));
    let id = one_point(&mut c);
    c.undo().unwrap();
    c.redo().unwrap();
    let batches = recorder.0.lock().unwrap().clone();
    assert_eq!(batches.len(), 3);
    assert_eq!(batches[0].0, ChangeCause::Edit);
    assert_eq!(batches[1].0, ChangeCause::Undo);
    assert_eq!(batches[1].1, vec![Change::EntityRemoved(id)]);
    assert_eq!(batches[2].0, ChangeCause::Redo);
    assert_eq!(batches[2].1, vec![Change::EntityAdded(id)]);
    for pair in batches.windows(2) {
        // Undo restores an earlier revision, so the invariant is a contiguous
        // chain (each batch starts where the previous one ended), not an ever
        // increasing revision.
        assert_eq!(
            pair[1].2, pair[0].3,
            "each published batch continues the previous one: {batches:?}"
        );
    }
}

#[test]
fn l3_011_a_panicking_observer_is_recorded_and_the_others_still_run() {
    let mut c = controller();
    let recorder = Recorder::default();
    c.add_document_observer(Box::new(Panicking));
    c.add_document_observer(Box::new(Failing));
    c.add_document_observer(Box::new(recorder.clone()));
    let id = one_point(&mut c);
    let diagnostics = c.observer_diagnostics();
    assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
    assert_eq!(diagnostics[0].observer, 0);
    assert_eq!(diagnostics[1].observer, 1);
    assert!(diagnostics[1].detail.contains("observer refused"));
    assert_eq!(
        recorder.0.lock().unwrap().len(),
        1,
        "the third observer still receives the batch"
    );
    assert!(c.document().entities.contains_key(&id));
    assert_eq!(
        c.observer_count(),
        1,
        "a panicking or failing observer is detached after one diagnostic"
    );
}

#[test]
fn l3_011_recording_starts_with_the_first_observer_and_holds_nothing_before() {
    let mut c = controller();
    assert!(
        !c.is_recording_changes(),
        "an unobserved controller must not retain a second copy of the document"
    );
    one_point(&mut c);
    assert!(c.last_document_change().is_none());
    assert_eq!(c.document_changes().count(), 0);
    let recorder = Recorder::default();
    c.add_document_observer(Box::new(recorder.clone()));
    assert!(c.is_recording_changes());
    c.submit("Point").unwrap();
    c.submit("7,7,7").unwrap();
    assert_eq!(recorder.0.lock().unwrap().len(), 1);
    assert_eq!(c.document_changes().count(), 1);
}

#[test]
fn l3_011_an_observer_cannot_change_the_document_it_is_told_about() {
    let mut c = controller();
    let recorder = Recorder::default();
    c.add_document_observer(Box::new(recorder.clone()));
    let id = one_point(&mut c);
    let revision = c.document().revision;
    let batches = recorder.0.lock().unwrap().clone();
    assert_eq!(
        batches[0].3, revision,
        "the batch names the published revision"
    );
    for _ in 0..3 {
        // Re-notifying the same batch cannot mutate anything: the observer only
        // ever holds a shared batch, never the controller.
        let batch = c.last_document_change().unwrap().clone();
        let mut recorder = recorder.clone();
        recorder.document_changed(&batch).unwrap();
    }
    assert_eq!(c.document().revision, revision);
    assert!(c.document().entities.contains_key(&id));
    assert_eq!(c.coordinator().undo_len(), 1);
}

/* ------------------------------ L3-012 ------------------------------ */

#[test]
fn l3_012_snapshot_restore_rebuilds_the_document_and_resets_undo() {
    let mut c = controller();
    let id = one_point(&mut c);
    let snapshot = c.capture_snapshot().unwrap();
    c.select_root(Some(id), false).unwrap();
    c.submit("Point").unwrap();
    c.submit("4,5,6").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    assert_eq!(c.coordinator().undo_len(), 2);
    assert_eq!(
        c.restore_snapshot(&snapshot).unwrap(),
        SnapshotOutcome::Restored
    );
    assert_eq!(c.document().entities.len(), 1);
    assert!(c.document().entities.contains_key(&id));
    assert_eq!(
        c.coordinator().undo_len(),
        0,
        "the snapshot is the new baseline, so Undo cannot walk behind it"
    );
    assert!(c.selected().is_empty());
    assert!(
        c.scene().is_some(),
        "the scene is rebuilt from the snapshot"
    );
}

#[test]
fn l3_012_another_documents_snapshot_is_refused_without_changing_the_controller() {
    let mut c = controller();
    one_point(&mut c);
    let other = controller();
    let foreign = other.capture_snapshot().unwrap();
    let before_document = c.document().clone();
    let before_undo = c.coordinator().undo_len();
    assert_eq!(
        c.restore_snapshot(&foreign),
        Err(om9_core::Error::Invalid("snapshot document"))
    );
    assert_eq!(c.document(), &before_document);
    assert_eq!(c.coordinator().undo_len(), before_undo);
}

#[test]
fn l3_012_a_restore_inside_a_running_command_is_busy() {
    let mut c = controller();
    one_point(&mut c);
    let snapshot = c.capture_snapshot().unwrap();
    c.submit("Point").unwrap();
    assert!(c.is_running());
    assert_eq!(c.restore_snapshot(&snapshot), Err(om9_core::Error::Busy));
    c.cancel();
    assert_eq!(
        c.restore_snapshot(&snapshot).unwrap(),
        SnapshotOutcome::Restored
    );
}
