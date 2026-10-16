// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    model::Document,
    transaction::{TransactionCoordinator, World},
};
use om9_exchange::ExportCapture;

#[test]
fn equal_counters_and_shared_document_do_not_authorize_a_foreign_export_coordinator() {
    let document = Document::new("shared immutable allocation");
    let id = document.id;
    let world = World::new(vec![document], &Limits::default()).unwrap();
    let a = TransactionCoordinator::new(world.clone(), Limits::default()).unwrap();
    let b = TransactionCoordinator::new(world, Limits::default()).unwrap();
    let capture = ExportCapture::all(&a, id).unwrap();
    assert!(capture.revalidate(&b).is_err());
    assert!(capture.revalidate(&a).is_ok());
}

#[test]
fn a_caller_roundtrip_without_public_epoch_change_still_invalidates_export_capture() {
    let document = Document::new("caller lifecycle");
    let id = document.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    let epoch = c.epoch();
    let owner = c.book("caller", false).unwrap();
    c.begin(&owner).unwrap();
    c.abort(&owner).unwrap();
    assert_eq!(c.epoch(), epoch);
    assert!(capture.revalidate(&c).is_err());
}

#[test]
fn plain_export_cannot_bypass_an_active_caller_scope() {
    let document = Document::new("caller private ownership");
    let id = document.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let owner = c.book("caller", true).unwrap();
    c.begin(&owner).unwrap();
    assert!(ExportCapture::all(&c, id).is_err());
    c.abort(&owner).unwrap();
    assert!(ExportCapture::all(&c, id).is_ok());
}
