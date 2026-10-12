use openmatrix9_rust::command_options_ffi::*;
use std::ffi::CString;
use std::ptr;

#[test]
fn arbitrary_choice_aliases_are_owned_and_use_existing_bounded_snapshot_abi() {
    let h = unsafe { om9_options_begin(b"CUSTOM-CHOICE".as_ptr(), 13, 1.) };
    assert_ne!(h, 0);
    assert!(unsafe {
        om9_options_define(
            h,
            b"Direction".as_ptr(),
            9,
            2,
            0.,
            0.,
            false,
            ptr::null(),
            0,
            ptr::null(),
            0,
            3,
        )
    });
    for value in ["Forward", "Reverse"] {
        assert!(unsafe {
            om9_options_choice(
                h,
                b"Direction".as_ptr(),
                9,
                value.as_ptr(),
                value.len(),
                ptr::null(),
                0,
                true,
                ptr::null(),
                0,
            )
        });
    }
    let mut identity = b"Direction=Reverse".to_vec();
    let mut alias = b"R".to_vec();
    assert!(unsafe {
        om9_options_alias(
            h,
            identity.as_ptr(),
            identity.len(),
            alias.as_ptr(),
            alias.len(),
        )
    });
    identity.fill(b'x');
    alias.fill(b'x');
    assert!(unsafe {
        om9_options_initial(h, b"Direction".as_ptr(), 9, 2, 0., b"Forward".as_ptr(), 7)
    });
    assert!(om9_options_seal(h));
    assert!(!unsafe { om9_options_alias(h, b"Direction=Reverse".as_ptr(), 17, b"R".as_ptr(), 1) });
    assert_eq!(event(h, 1, "Direction", 0.).0, 1);
    let mut mnemonic = [0u8; 8];
    assert_eq!(
        unsafe { om9_options_item_text(h, 1, 3, mnemonic.as_mut_ptr(), mnemonic.len()) },
        2
    );
    assert_eq!(&mnemonic[..2], b"R\0");
    let (status, token) = event(h, 2, "r", 0.);
    assert_eq!(status, 2);
    assert!(om9_options_accept(h, token, true));
    let mut value = Om9OptionValue::default();
    assert!(unsafe { om9_options_value(h, b"Direction".as_ptr(), 9, 0, &mut value) });
    assert_eq!(value.choice, 1);
    om9_options_close(h);
}

fn event(h: u64, kind: u32, text: &str, value: f64) -> (u32, u64) {
    let mut token = 0;
    let status =
        unsafe { om9_options_event(h, kind, text.as_ptr(), text.len(), value, &mut token) };
    (status, token)
}
#[test]
fn ffi_candidates_do_not_publish_without_native_acceptance() {
    let h = om9_options_open(13, 1.);
    assert_ne!(h, 0);
    let (status, token) = event(h, 1, "DynamicPreview", 0.);
    assert_eq!(status, 2);
    let mut v = Om9OptionValue::default();
    assert!(unsafe { om9_options_value(h, b"DynamicPreview".as_ptr(), 14, 0, &mut v) });
    assert_eq!(v.number, 1.);
    assert!(om9_options_accept(h, token, true));
    assert!(unsafe { om9_options_value(h, b"DynamicPreview".as_ptr(), 14, 0, &mut v) });
    assert_eq!(v.number, 0.);
    om9_options_close(h);
    assert_eq!(event(h, 1, "DynamicPreview", 0.).0, 4);
}
#[test]
fn ffi_bad_utf8_lengths_and_handles_are_rejected() {
    let h = om9_options_open(13, 1.);
    let mut token = 0;
    assert_eq!(
        unsafe { om9_options_event(h, 2, ptr::null(), 3, 0., &mut token) },
        4
    );
    assert_eq!(
        unsafe { om9_options_event(h, 2, [255u8].as_ptr(), 1, 0., &mut token) },
        4
    );
    assert_eq!(
        unsafe { om9_options_event(h, 999, b"".as_ptr(), 0, 0., &mut token) },
        4
    );
    assert_eq!(
        unsafe { om9_options_event(h, 2, b"x".as_ptr(), 1025, 0., &mut token) },
        4
    );
    assert_eq!(event(0, 1, "Style", 0.).0, 4);
    assert_eq!(om9_options_open(999, 1.), 0);
    om9_options_close(h);
}
#[test]
fn ffi_rejects_superseded_candidate_and_short_snapshot_buffer() {
    let h = om9_options_open(13, 1.);
    let (_, old) = event(h, 1, "DynamicPreview", 0.);
    let (_, new) = event(h, 3, "History", 1.);
    assert!(!om9_options_accept(h, old, true));
    assert!(om9_options_accept(h, new, true));
    let mut tiny = [42u8; 2];
    let required = unsafe { om9_options_snapshot(h, tiny.as_mut_ptr(), tiny.len()) };
    assert!(required > 2);
    assert_eq!(tiny[1], 0);
    assert!(!unsafe { om9_options_value(h, b"History".as_ptr(), 7, 0, ptr::null_mut()) });
    om9_options_close(h);
}
#[test]
fn arbitrary_command_registers_owned_schema_and_uses_same_engine() {
    let h = unsafe { om9_options_begin(b"OM9-CURVE-CUSTOM".as_ptr(), 16, 1.) };
    assert_ne!(h, 0);
    assert!(unsafe {
        om9_options_define(
            h,
            b"Distance".as_ptr(),
            8,
            3,
            0.01,
            100.,
            false,
            b"mm".as_ptr(),
            2,
            ptr::null(),
            0,
            3,
        )
    });
    assert!(unsafe {
        om9_options_define(
            h,
            b"Reverse".as_ptr(),
            7,
            1,
            0.,
            0.,
            false,
            ptr::null(),
            0,
            ptr::null(),
            0,
            3,
        )
    });
    assert!(unsafe { om9_options_initial(h, b"Reverse".as_ptr(), 7, 1, 0., ptr::null(), 0) });
    assert!(om9_options_seal(h));
    assert_eq!(event(h, 1, "Distance", 0.).0, 1);
    let (status, token) = event(h, 2, "2cm", 0.);
    assert_eq!(status, 2);
    assert!(om9_options_accept(h, token, true));
    let mut value = Om9OptionValue::default();
    assert!(unsafe { om9_options_value(h, b"Distance".as_ptr(), 8, 0, &mut value) });
    assert_eq!(value.number, 20.);
    assert!(!unsafe {
        om9_options_define(
            h,
            b"Injected".as_ptr(),
            8,
            1,
            0.,
            0.,
            false,
            ptr::null(),
            0,
            ptr::null(),
            0,
            3,
        )
    });
    om9_options_close(h);
}
#[test]
fn curve_adapter_rejects_restarted_generation_and_native_domain_atomically() {
    unsafe extern "C" {
        fn om9_curve_start(text: *const std::ffi::c_char) -> bool;
    }
    let id = CString::new("InterpCrv").unwrap();
    assert!(unsafe { om9_curve_start(id.as_ptr()) });
    let h = om9_options_open(3, 1.);
    assert!(om9_options_sync_curve(h));
    let (status, token) = event(h, 2, "Degree=2", 0.);
    assert_eq!(status, 2);
    assert_eq!(om9_options_accept_curve(h, token), 0); // existing interpolation accepts odd degree only
    let mut value = Om9OptionValue::default();
    assert!(unsafe { om9_options_value(h, b"Degree".as_ptr(), 6, 0, &mut value) });
    assert_eq!(value.number, 3.);
    let (status, token) = event(h, 2, "Degree=5", 0.);
    assert_eq!(status, 2);
    assert!(unsafe { om9_curve_start(id.as_ptr()) });
    assert!(!om9_options_sync_curve(h));
    assert_eq!(om9_options_accept_curve(h, token), 0);
    om9_options_close(h);
    unsafe extern "C" {
        fn om9_curve_point(x: f64, y: f64, z: f64) -> u32;
    }
    assert!(unsafe { om9_curve_start(id.as_ptr()) });
    let h = om9_options_open(3, 1.);
    assert!(om9_options_sync_curve(h));
    let (_, token) = event(h, 2, "Degree=5", 0.);
    assert_eq!(unsafe { om9_curve_point(0., 0., 0.) }, 1);
    assert_eq!(om9_options_accept_curve(h, token), 0);
    om9_options_close(h);
    unsafe extern "C" {
        fn om9_curve_input(text: *const std::ffi::c_char) -> u32;
    }
    let id = CString::new("Circle").unwrap();
    assert!(unsafe { om9_curve_start(id.as_ptr()) });
    let input = CString::new("Tangent").unwrap();
    assert_eq!(unsafe { om9_curve_input(input.as_ptr()) }, 1);
    let h = om9_options_open(5, 1.);
    assert!(om9_options_sync_curve(h));
    assert_eq!(event(h, 2, "Curve=Line.Edge1@0,4,0", 0.).0, 3);
    om9_options_close(h);
}

#[test]
fn ffi_escape_and_draft_replacement_retire_candidate_tokens() {
    let h = om9_options_open(13, 1.);
    assert_eq!(event(h, 2, "SectionMode=Rebuild", 0.).0, 1);
    let (status, old) = event(h, 2, "24", 0.);
    assert_eq!(status, 2);
    assert_eq!(event(h, 6, "", 0.).0, 1);
    assert!(!om9_options_accept(h, old, true));
    let (_, old) = event(h, 2, "History=Yes", 0.);
    assert_eq!(event(h, 8, "History=No", 0.).0, 1);
    assert!(!om9_options_accept(h, old, true));
    om9_options_close(h);
}

#[test]
fn ffi_registration_rejects_case_only_duplicate_identity() {
    let h = unsafe { om9_options_begin(b"OM9-CUSTOM".as_ptr(), 10, 1.) };
    assert_ne!(h, 0);
    let define = |key: &str| unsafe {
        om9_options_define(
            h,
            key.as_ptr(),
            key.len(),
            1,
            0.,
            0.,
            false,
            ptr::null(),
            0,
            ptr::null(),
            0,
            3,
        )
    };
    assert!(define("Flip"));
    assert!(!define("flip"));
    assert!(om9_options_seal(h));
    om9_options_close(h);
}

#[test]
fn box_ffi_phase_snapshot_and_restart_guard_are_native_owned() {
    use openmatrix9_rust::solid::*;
    let name = CString::new("Box").unwrap();
    assert!(unsafe { om9_solid_start(name.as_ptr()) });
    let h = om9_options_open(14, 1.);
    assert_ne!(h, 0);
    assert!(om9_options_sync_solid(h));
    assert_eq!(om9_options_question_length(h), 20);
    assert_eq!(event(h, 2, "d", 0.).0, 3);
    let action = CString::new("Diagonal").unwrap();
    assert_eq!(unsafe { om9_solid_input(action.as_ptr()) }, 1);
    assert_eq!(om9_solid_point(0., 0., 0., false), 1);
    assert!(om9_options_sync_solid(h));
    assert_eq!(event(h, 2, "d", 0.).0, 4);
    assert_eq!(om9_options_item_count(h), 1);
    assert!(unsafe { om9_solid_start(name.as_ptr()) });
    assert!(!om9_options_sync_solid(h));
    om9_options_close(h);
    om9_solid_cancel();
    let mut short = [0u8; 3];
    assert_eq!(
        unsafe { om9_options_invocation_line(b"Box".as_ptr(), 3, short.as_mut_ptr(), 3) },
        14
    );
    assert_eq!(short[2], 0);
    assert_eq!(
        unsafe { om9_options_invocation_line(b"Box\nDelete".as_ptr(), 10, short.as_mut_ptr(), 3) },
        0
    );
}
