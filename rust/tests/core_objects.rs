// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::core_objects::{Registry, Stamp};

#[test]
fn lifecycle_and_aba() {
    let mut r = Registry::new();
    let d = r.document_open().unwrap();
    let a = r.observe(d, 7).unwrap();
    assert_eq!(r.observe(d, 7), Some(a));
    assert!(r.validate(a));
    r.removed(d, 7);
    let b = r.observe(d, 7).unwrap();
    assert!(b.generation > a.generation);
    assert!(!r.validate(a));
    assert!(r.document_invalidate(d));
    assert!(!r.validate(b));
    let c = r.observe(d, 7).unwrap();
    assert_eq!(c.generation, b.generation);
    assert!(c.epoch > b.epoch);
    r.document_close(d);
    assert_eq!(r.document_count(), 0);
    assert!(!r.validate(c));
    assert_eq!(r.observe(d, 7), None);
    assert!(r.document_open().unwrap() > d);
}

#[test]
fn isolated_revisions_and_invalid_inputs() {
    let mut r = Registry::new();
    assert!(!r.validate(Stamp::default()));
    assert!(!r.changed(0, 7));
    assert!(!r.document_invalidate(0));
    assert_eq!(r.observe(0, 7), None);
    let d = r.document_open().unwrap();
    let e = r.document_open().unwrap();
    assert_eq!(r.observe(d, 0), None);
    assert!(!r.changed(d, 99));
    let a = r.observe(d, 7).unwrap();
    let b = r.observe(e, 7).unwrap();
    assert_ne!(a.generation, b.generation);
    assert!(r.changed(d, 7));
    assert!(!r.validate(a));
    assert!(r.validate(b));
    let c = r.observe(d, 7).unwrap();
    assert_eq!(c.revision, a.revision + 1);
    assert_eq!(c.generation, a.generation);
    let mut forged = c;
    forged.document = e;
    assert!(!r.validate(forged));
}

#[test]
fn exhaustion_fails_closed() {
    let mut r = Registry::with_counter_limit(2);
    let d = r.document_open().unwrap();
    let a = r.observe(d, 1).unwrap();
    assert!(r.changed(d, 1));
    let b = r.observe(d, 1).unwrap();
    assert!(!r.changed(d, 1));
    assert!(!r.validate(b));
    let c = r.observe(d, 1).unwrap();
    assert!(c.generation > a.generation);
    r.removed(d, 1);
    assert_eq!(r.observe(d, 1), None);
    assert!(r.document_invalidate(d));
    assert!(!r.document_invalidate(d));
    assert_eq!(r.observe(d, 2), None);
    let e = r.document_open().unwrap();
    r.document_close(e);
    assert_eq!(r.document_open(), None);
}

#[test]
fn ffi_layout_nulls_and_owned_output() {
    use openmatrix9_rust::core_objects::*;
    assert_eq!(std::mem::size_of::<Stamp>(), 40);
    assert_eq!(std::mem::offset_of!(Stamp, document), 0);
    assert_eq!(std::mem::offset_of!(Stamp, object), 8);
    assert_eq!(std::mem::offset_of!(Stamp, generation), 16);
    assert_eq!(std::mem::offset_of!(Stamp, revision), 24);
    assert_eq!(std::mem::offset_of!(Stamp, epoch), 32);
    let d = om9_object_document_open();
    assert_ne!(d, 0);
    let mut stamp = Stamp::default();
    // SAFETY: all nonnull pointers refer to local initialized aligned storage.
    unsafe {
        assert!(!om9_object_observe(d, 7, std::ptr::null_mut()));
        assert!(!om9_object_validate(std::ptr::null()));
        assert!(om9_object_observe(d, 7, &mut stamp));
        assert!(om9_object_validate(&stamp));
        let original = stamp;
        assert!(!om9_object_observe(0, 7, &mut stamp));
        assert_eq!(stamp, original);
        assert!(om9_object_changed(d, 7));
        assert!(!om9_object_validate(&stamp));
        assert!(om9_object_observe(d, 7, &mut stamp));
        om9_object_document_close(d);
        assert!(!om9_object_validate(&stamp));
    }
}
