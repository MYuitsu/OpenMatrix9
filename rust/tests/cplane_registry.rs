use openmatrix9_rust::coordinates::Frame;
use openmatrix9_rust::cplanes::{self, NamedPlane, Registry};

fn translated(x: f64) -> Frame {
    Frame::new([x, 20., 30.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]]).unwrap()
}

#[test]
fn view_planes_are_isolated_and_ensure_never_replaces_user_state() {
    let mut registry = Registry::default();
    let a = registry.ensure(1, 1, translated(10.)).unwrap();
    registry.ensure(1, 2, Frame::default()).unwrap();
    registry.ensure(2, 1, translated(40.)).unwrap();
    assert_eq!(registry.ensure(1, 1, translated(99.)).unwrap(), a);
    assert_eq!(registry.read(1, 2).unwrap().frame, Frame::default());
    assert_eq!(registry.read(2, 1).unwrap().frame, translated(40.));
}

#[test]
fn previous_next_and_branching_preserve_frames_but_advance_revision() {
    let mut registry = Registry::default();
    let first = registry.ensure(1, 1, translated(10.)).unwrap();
    let second = registry.set(1, 1, translated(20.)).unwrap();
    let third = registry.set(1, 1, translated(30.)).unwrap();
    assert!(first.revision < second.revision && second.revision < third.revision);
    let previous = registry.previous(1, 1).unwrap();
    assert_eq!(previous.frame, translated(20.));
    assert!(previous.revision > third.revision);
    assert_eq!(registry.next(1, 1).unwrap().frame, translated(30.));
    registry.previous(1, 1).unwrap();
    registry.set(1, 1, translated(40.)).unwrap();
    assert!(registry.next(1, 1).is_err());
    assert_eq!(registry.previous(1, 1).unwrap().frame, translated(20.));
    assert_eq!(registry.previous(1, 1).unwrap().frame, translated(10.));
    let before = registry.read(1, 1).unwrap();
    assert!(registry.previous(1, 1).is_err());
    assert_eq!(registry.read(1, 1).unwrap(), before);
}

#[test]
fn duplicate_sets_are_noops_and_closed_view_identity_has_a_new_revision() {
    let mut registry = Registry::default();
    let first = registry.ensure(1, 1, translated(10.)).unwrap();
    assert_eq!(registry.set(1, 1, translated(10.)).unwrap(), first);
    assert!(registry.previous(1, 1).is_err());
    registry.drop_view(1, 1);
    assert!(registry.read(1, 1).is_none());
    let reopened = registry.ensure(1, 1, Frame::default()).unwrap();
    assert!(reopened.revision > first.revision);
    registry.ensure(2, 1, translated(20.)).unwrap();
    registry.drop_document(1);
    assert!(registry.read(1, 1).is_none());
    assert_eq!(registry.read(2, 1).unwrap().frame, translated(20.));
}

#[test]
fn missing_and_zero_identity_cannot_create_or_navigate_a_plane() {
    let mut registry = Registry::default();
    assert!(registry.ensure(0, 1, Frame::default()).is_err());
    assert!(registry.ensure(1, 0, Frame::default()).is_err());
    assert!(registry.set(1, 1, Frame::default()).is_err());
    assert!(registry.previous(1, 1).is_err());
    assert!(registry.next(1, 1).is_err());
    assert!(registry.read(1, 1).is_none());
}

#[test]
fn named_plane_identity_survives_rename_export_import_and_restore() {
    let mut registry = Registry::default();
    registry.ensure(1, 1, Frame::default()).unwrap();
    registry.ensure(1, 2, Frame::default()).unwrap();
    let id = registry.save_named(1, "Work", translated(10.)).unwrap();
    assert!(registry.save_named(1, "Work", translated(20.)).is_err());
    registry.rename_named(1, id, "Ring").unwrap();
    assert_eq!(
        registry.named(1),
        vec![NamedPlane {
            id,
            name: "Ring".into(),
            frame: translated(10.)
        }]
    );
    registry.restore_named(1, 2, id).unwrap();
    assert_eq!(registry.read(1, 2).unwrap().frame, translated(10.));
    assert_eq!(registry.read(1, 1).unwrap().frame, Frame::default());
    let records = registry.named(1);
    registry.drop_document(1);
    assert!(registry.named(1).is_empty());
    registry.import_named(1, &records).unwrap();
    registry.ensure(1, 3, Frame::default()).unwrap();
    registry.restore_named(1, 3, id).unwrap();
    assert_eq!(registry.read(1, 3).unwrap().frame, translated(10.));
    assert!(registry.save_named(1, "Other", Frame::default()).unwrap() > id);
}

#[test]
fn invalid_named_import_or_rename_is_atomic_and_document_scoped() {
    let mut registry = Registry::default();
    let first = registry.save_named(1, "First", translated(10.)).unwrap();
    registry.save_named(1, "Second", translated(20.)).unwrap();
    let before = registry.named(1);
    assert!(registry.rename_named(1, first, "Second").is_err());
    assert_eq!(registry.named(1), before);
    assert!(registry.save_named(1, "  ", Frame::default()).is_err());
    assert!(
        registry
            .save_named(0, "No owner", Frame::default())
            .is_err()
    );
    assert!(
        registry
            .import_named(1, &[before[0].clone(), before[0].clone()])
            .is_err()
    );
    assert_eq!(registry.named(1), before);
    registry.save_named(2, "First", Frame::default()).unwrap();
    assert!(registry.restore_named(2, 1, first).is_err());
    assert_eq!(registry.named(1), before);
}

#[test]
fn named_save_enforces_document_utf8_budget_before_allocating_an_identity() {
    let mut registry = Registry::default();
    let budget = 512 * 1024;
    // Count UTF-8 bytes, including non-ASCII labels, rather than characters.
    let large = "é".repeat((budget - 2) / 2);
    assert_eq!(registry.save_named(1, &large, translated(10.)).unwrap(), 1);
    assert_eq!(registry.save_named(1, "ab", translated(20.)).unwrap(), 2);
    let before = registry.named(1);
    assert!(registry.save_named(1, "c", translated(30.)).is_err());
    assert_eq!(registry.named(1), before);
    // A full table in one document does not consume another document's budget.
    assert_eq!(
        registry
            .save_named(2, &"x".repeat(budget), Frame::default())
            .unwrap(),
        3
    );
}

#[test]
fn named_rename_enforces_replacement_budget_atomically() {
    let mut registry = Registry::default();
    let budget = 512 * 1024;
    let large = registry
        .save_named(1, &"x".repeat(budget - 1), translated(10.))
        .unwrap();
    let small = registry.save_named(1, "y", translated(20.)).unwrap();
    let before = registry.named(1);
    assert!(registry.rename_named(1, small, "yz").is_err());
    assert_eq!(registry.named(1), before);
    // Replacing an existing label frees its own bytes; no-op at the limit works.
    registry
        .rename_named(1, large, &"x".repeat(budget - 1))
        .unwrap();
    registry.rename_named(1, large, "a").unwrap();
    registry
        .rename_named(1, small, &"z".repeat(budget - 1))
        .unwrap();
    assert_eq!(
        registry
            .named(1)
            .iter()
            .map(|plane| plane.name.len())
            .sum::<usize>(),
        budget
    );
}

#[test]
fn named_import_enforces_complete_replacement_budget_atomically() {
    let mut registry = Registry::default();
    let budget = 512 * 1024;
    registry.save_named(1, "Existing", translated(10.)).unwrap();
    registry
        .save_named(2, "Other document", translated(20.))
        .unwrap();
    let other = registry.named(2);
    let records = vec![
        NamedPlane {
            id: 10,
            name: "x".repeat(budget / 2),
            frame: translated(30.),
        },
        NamedPlane {
            id: 11,
            name: "y".repeat(budget / 2),
            frame: translated(40.),
        },
    ];
    registry.import_named(1, &records).unwrap();
    let mut oversized = records.clone();
    oversized[1].name.push('z');
    oversized[1].id = 999;
    assert!(registry.import_named(1, &oversized).is_err());
    assert_eq!(registry.named(1), records);
    assert_eq!(registry.named(2), other);
    assert_eq!(
        registry
            .save_named(2, "After rejection", Frame::default())
            .unwrap(),
        12
    );
}

#[test]
fn three_point_planes_are_right_handed_and_reject_degenerate_construction() {
    let frame =
        Frame::from_three_points([10., 20., 30.], [10., 22., 30.], [10., 21., 33.]).unwrap();
    assert_eq!(frame.origin(), [10., 20., 30.]);
    assert_eq!(frame.axes(), [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]]);
    assert!(Frame::from_three_points([0.; 3], [0.; 3], [1., 0., 0.]).is_err());
    assert!(Frame::from_three_points([0.; 3], [1., 0., 0.], [2., 0., 0.]).is_err());
    assert!(Frame::from_three_points([f64::NAN, 0., 0.], [1., 0., 0.], [0., 1., 0.]).is_err());
}

#[test]
fn native_boundary_copies_values_and_rejects_invalid_planes_without_mutation() {
    let doc = 900_001;
    let view = 7;
    let frame = [10., 20., 30., 0., 1., 0., 0., 0., 1., 1., 0., 0.];
    let mut output = [0.; 12];
    let mut revision = 0;
    unsafe {
        assert!(!cplanes::om9_cplane_ensure(doc, view, std::ptr::null()));
        assert!(cplanes::om9_cplane_ensure(doc, view, frame.as_ptr()));
        assert!(cplanes::om9_cplane_read(
            doc,
            view,
            output.as_mut_ptr(),
            &mut revision
        ));
        assert_eq!(output, frame);
        let first_revision = revision;
        let mut invalid = frame;
        invalid[9] = -1.;
        assert!(!cplanes::om9_cplane_set(doc, view, invalid.as_ptr()));
        invalid[0] = f64::NAN;
        assert!(!cplanes::om9_cplane_set(doc, view, invalid.as_ptr()));
        assert!(cplanes::om9_cplane_read(
            doc,
            view,
            output.as_mut_ptr(),
            &mut revision
        ));
        assert_eq!(output, frame);
        assert_eq!(revision, first_revision);
        let mut second = frame;
        second[0] = 40.;
        assert!(cplanes::om9_cplane_set(doc, view, second.as_ptr()));
        assert!(cplanes::om9_cplane_previous(doc, view));
        assert!(cplanes::om9_cplane_read(
            doc,
            view,
            output.as_mut_ptr(),
            std::ptr::null_mut()
        ));
        assert_eq!(output, frame);
        assert!(cplanes::om9_cplane_next(doc, view));
        assert!(cplanes::om9_cplane_read(
            doc,
            view,
            output.as_mut_ptr(),
            &mut revision
        ));
        assert_eq!(output, second);
        assert!(revision > first_revision);
        cplanes::om9_cplane_drop_document(doc);
        assert!(!cplanes::om9_cplane_read(
            doc,
            view,
            output.as_mut_ptr(),
            &mut revision
        ));
    }
}

#[test]
fn native_named_records_roundtrip_atomically_and_small_buffers_are_not_written() {
    use std::ffi::{CStr, CString};
    let document = 900_002;
    let frame = [10., 20., 30., 0., 1., 0., 0., 0., 1., 1., 0., 0.];
    let name = CString::new("Ring plane").unwrap();
    unsafe {
        assert!(cplanes::om9_cplane_ensure(document, 1, frame.as_ptr()));
        let id = cplanes::om9_cplane_named_save(document, 1, name.as_ptr());
        assert_ne!(id, 0);
        let mut saved_id = 0;
        let mut saved_frame = [0.; 12];
        let mut small = [b'x' as i8; 2];
        let size = cplanes::om9_cplane_named_read(
            document,
            0,
            &mut saved_id,
            saved_frame.as_mut_ptr(),
            small.as_mut_ptr(),
            2,
        );
        assert_eq!(saved_id, id);
        assert_eq!(saved_frame, frame);
        assert_eq!(small, [b'x' as i8; 2]);
        let mut label = vec![0_i8; size];
        assert_eq!(
            cplanes::om9_cplane_named_read(
                document,
                0,
                &mut saved_id,
                saved_frame.as_mut_ptr(),
                label.as_mut_ptr(),
                size
            ),
            size
        );
        assert_eq!(CStr::from_ptr(label.as_ptr()), name.as_c_str());
        let ids = [id, id];
        let names = [name.as_ptr(), name.as_ptr()];
        let mut frames = frame.to_vec();
        frames.extend(frame);
        assert!(!cplanes::om9_cplane_named_import(
            document,
            ids.as_ptr(),
            names.as_ptr(),
            frames.as_ptr(),
            2
        ));
        assert_eq!(cplanes::om9_cplane_named_count(document), 1);
        cplanes::om9_cplane_drop_document(document);
        assert!(cplanes::om9_cplane_named_import(
            document,
            &id,
            names.as_ptr(),
            frame.as_ptr(),
            1
        ));
        assert!(cplanes::om9_cplane_ensure(document, 1, frame.as_ptr()));
        assert!(cplanes::om9_cplane_named_restore(document, 1, id));
        let mut output = [0.; 12];
        assert!(cplanes::om9_cplane_read(
            document,
            1,
            output.as_mut_ptr(),
            std::ptr::null_mut()
        ));
        assert_eq!(output, frame);
        cplanes::om9_cplane_drop_document(document);
    }
}

#[test]
fn native_three_point_rejects_bad_input_without_writing_output() {
    let mut output = [99.; 12];
    unsafe {
        assert!(!cplanes::om9_cplane_three_points(
            [0., 0., 0., 1., 0., 0., 2., 0., 0.].as_ptr(),
            output.as_mut_ptr()
        ));
        assert_eq!(output, [99.; 12]);
        assert!(cplanes::om9_cplane_three_points(
            [10., 20., 30., 10., 22., 30., 10., 21., 33.].as_ptr(),
            output.as_mut_ptr()
        ));
        assert_eq!(output, [10., 20., 30., 0., 1., 0., 0., 0., 1., 1., 0., 0.]);
    }
}

#[test]
fn world_menu_planes_have_explicit_axes_and_affect_only_the_requested_view() {
    let mut registry = Registry::default();
    registry.ensure(1, 1, translated(10.)).unwrap();
    registry.ensure(1, 2, translated(20.)).unwrap();
    for (choice, axes) in [
        (0, [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]),
        (1, [[1., 0., 0.], [0., 0., 1.], [0., -1., 0.]]),
        (2, [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]]),
    ] {
        let result = registry.set_world(1, 1, choice).unwrap();
        assert_eq!(result.frame.origin(), [0.; 3]);
        assert_eq!(result.frame.axes(), axes);
        assert_eq!(registry.read(1, 2).unwrap().frame, translated(20.));
    }
    let before = registry.read(1, 1).unwrap();
    assert!(registry.set_world(1, 1, 3).is_err());
    assert_eq!(registry.read(1, 1).unwrap(), before);
}

#[test]
fn history_menu_availability_is_read_only_and_tracks_navigation_and_new_branches() {
    let mut registry = Registry::default();
    registry.ensure(1, 1, translated(10.)).unwrap();
    assert!(!registry.history_available(1, 1, false));
    assert!(!registry.history_available(1, 1, true));
    registry.set(1, 1, translated(20.)).unwrap();
    let before = registry.read(1, 1).unwrap();
    assert!(registry.history_available(1, 1, false));
    assert!(!registry.history_available(1, 1, true));
    assert_eq!(registry.read(1, 1).unwrap(), before);
    registry.previous(1, 1).unwrap();
    assert!(!registry.history_available(1, 1, false));
    assert!(registry.history_available(1, 1, true));
    registry.set(1, 1, translated(30.)).unwrap();
    assert!(!registry.history_available(1, 1, true));
}
