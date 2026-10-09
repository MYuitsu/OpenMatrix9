use openmatrix9_rust::coordinate_input::{
    Context, CoordinateRequest, CoordinateValue, Expected, Request, Source, Value,
    om9_coordinate_report, om9_coordinate_resolve, om9_coordinate_verify_context, report, resolve,
};
use openmatrix9_rust::coordinates::Frame;

fn context() -> Context {
    Context {
        document_owner: 17,
        document_uid: "document-fixture".into(),
        frame_revision: 23,
        logical_view_slot: 2,
        frame: Frame::new([10., 20., 30.], [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]]).unwrap(),
        unit_context: String::new(),
    }
}
fn request<'a>(context: &'a Context, original: &'a str) -> Request<'a> {
    Request {
        context,
        expected: Expected::Point,
        source: Source::Typed,
        original,
        previous: None,
        world: [0.; 3],
    }
}

#[test]
fn typed_points_use_the_latched_frame_and_last_accepted_world_point() {
    let context = context();
    for (token, previous, expected) in [
        ("1,2,3", None, [13., 21., 32.]),
        ("1,2", None, [10., 21., 32.]),
        ("w1,2,3", None, [1., 2., 3.]),
        ("r1,2,3", Some([13., 21., 32.]), [16., 22., 34.]),
        ("R1,2,3", Some([13., 21., 32.]), [16., 22., 34.]),
        ("@1,2,3", Some([13., 21., 32.]), [16., 22., 34.]),
        ("wr1,2,3", Some([13., 21., 32.]), [14., 23., 35.]),
    ] {
        let mut input = request(&context, token);
        input.previous = previous;
        assert_eq!(
            resolve(&input).unwrap().value(),
            &Value::Point(expected),
            "{token}"
        );
    }
}

#[test]
fn snapshot_units_scale_only_unsuffixed_components() {
    let mut context = context();
    context.unit_context = "1|7|25.4|1|0.01|0.001|0.017453292519943295".into();
    assert_eq!(
        resolve(&request(&context, "1,2cm,3mm")).unwrap().value(),
        &Value::Point([13., 45.4, 50.])
    );
}

#[test]
fn scalar_length_is_allowed_only_in_point_or_length_phase() {
    let mut context = context();
    context.unit_context = "1|7|25.4|1|0.01|0.001|0.017453292519943295".into();
    assert!(resolve(&request(&context, "2")).is_err());
    let mut input = request(&context, "2");
    input.expected = Expected::PointOrLength;
    assert_eq!(resolve(&input).unwrap().value(), &Value::Length(50.8));
    input.original = "2cm";
    assert_eq!(resolve(&input).unwrap().value(), &Value::Length(20.));
}

#[test]
fn world_sources_never_transform_or_scale_the_host_point() {
    let mut context = context();
    context.unit_context = "1|7|25.4|1|0.01|0.001|0.017453292519943295".into();
    for source in [Source::WorldPick, Source::WorldSnap, Source::OriginShortcut] {
        let mut input = request(&context, "");
        input.source = source;
        input.world = [4., 5., 6.];
        assert_eq!(
            resolve(&input).unwrap().value(),
            &Value::Point([4., 5., 6.])
        );
    }
}

#[test]
fn malformed_context_and_unsupported_grammar_never_fall_back() {
    let context = context();
    for token in [
        "", "1", "w1,2", "1,2,3,4", "1,,2", "1, 2", "1;2", "1<30", "N45E", "rw1,2,3", "W1,2,3",
        "1/2,3", "1deg,2", "NaN,0", "inf,0", "1e309,0", "1e9m,0", "r1,2,3",
    ] {
        assert!(
            resolve(&request(&context, token)).is_err(),
            "unexpected acceptance: {token}"
        );
    }
    let mut oversized = context.clone();
    oversized.unit_context = "x".repeat(513);
    assert!(resolve(&request(&oversized, "1,2")).is_err());
    let mut tiny = context.clone();
    tiny.unit_context = "1|1|1e-9|1|0.01|0.001|0.01".into();
    assert!(resolve(&request(&tiny, "5e-324,0")).is_err());
    for raw in [
        "future",
        "2|1|1|1|0.01|0.001|0.01",
        "1|1|0|1|0.01|0.001|0.01",
    ] {
        let mut invalid = context.clone();
        invalid.unit_context = raw.into();
        assert!(resolve(&request(&invalid, "1,2")).is_err());
    }
    for field in 0..3 {
        let mut invalid = context.clone();
        match field {
            0 => invalid.document_owner = 0,
            1 => invalid.frame_revision = 0,
            _ => invalid.logical_view_slot = 4,
        }
        assert!(resolve(&request(&invalid, "1,2")).is_err());
    }
    let mut input = request(&context, "1,2");
    input.previous = Some([f64::NAN, 0., 0.]);
    assert!(resolve(&input).is_err());
}

#[test]
fn same_uid_cannot_reuse_a_document_lifetime_and_unit_drift_rejects() {
    let context = context();
    assert!(context.verify(17, "document-fixture", "").is_ok());
    assert!(context.verify(18, "document-fixture", "").is_err());
    assert!(context.verify(17, "different-document", "").is_err());
    assert!(
        context
            .verify(17, "document-fixture", "1|1|1|1|0.01|0.001|0.01")
            .is_err()
    );
    let mut declared = context.clone();
    declared.unit_context = "1|7|1|1|0.01|0.001|0.01".into();
    assert!(
        declared
            .verify(17, "document-fixture", "1|8|1|1|0.01|0.001|0.01")
            .is_err()
    );
    assert!(declared.verify(17, "document-fixture", "").is_err());
    // No current frame/revision is passed to verification: command-start wins.
    assert_eq!(
        resolve(&request(&context, "1,2")).unwrap().value(),
        &Value::Point([10., 21., 32.])
    );
}

#[test]
fn reports_own_text_snapshot_and_world_source_without_inventing_identity() {
    let (typed, snap, length) = {
        let mut context = context();
        context.document_uid = "doc\"\\\nΩ".into();
        context.unit_context = "1|7|25.4|1|0.01|0.001|0.01".into();
        let original = String::from("\n r1,2cm,3mm\t");
        let mut input = request(&context, &original);
        input.previous = Some([13., 21., 32.]);
        let typed = resolve(&input).unwrap();
        let mut input = request(&context, "");
        input.source = Source::WorldSnap;
        input.world = [4., 5., 6.];
        input.previous = Some([16., 46.4, 52.]);
        let snap = resolve(&input).unwrap();
        let mut input = request(&context, "2");
        input.expected = Expected::PointOrLength;
        (typed, snap, resolve(&input).unwrap())
    };
    assert_eq!(typed.value(), &Value::Point([16., 46.4, 52.]));
    let raw = report(&[typed, snap, length]).unwrap();
    assert!(raw.contains("\"previous_world_mm\":[13,21,32]"));
    assert!(raw.contains("\"identity_status\":\"unavailable\""));
    assert!(!raw.contains("object_id"));
    // The harness independently decodes this JSON with its standard parser.
    println!("COORDINATE_REPORT_JSON={raw}");
}

#[test]
fn reports_reject_mixed_snapshots_and_enforce_the_input_budget() {
    let context = context();
    let first = resolve(&request(&context, "1,2")).unwrap();
    let mut other = context.clone();
    other.frame_revision += 1;
    let second = resolve(&request(&other, "1,2")).unwrap();
    assert!(report(&[first.clone(), second]).is_err());
    assert!(report(&vec![first.clone(); 64]).is_ok());
    assert!(report(&vec![first; 65]).is_err());
    assert_eq!(report(&[]).unwrap(), "{\"version\":1,\"inputs\":[]}");
}

fn abi<'a>(context: &'a Context, original: &'a str) -> CoordinateRequest {
    let origin = context.frame.origin();
    let axes = context.frame.axes();
    CoordinateRequest {
        version: 1,
        expected: 0,
        source: 0,
        has_previous: 0,
        document_owner: context.document_owner,
        frame_revision: context.frame_revision,
        logical_view_slot: context.logical_view_slot,
        reserved: 0,
        frame: [
            origin[0], origin[1], origin[2], axes[0][0], axes[0][1], axes[0][2], axes[1][0],
            axes[1][1], axes[1][2], axes[2][0], axes[2][1], axes[2][2],
        ],
        previous: [0.; 3],
        world: [0.; 3],
        original: original.as_ptr(),
        original_len: original.len(),
        unit_context: context.unit_context.as_ptr(),
        unit_context_len: context.unit_context.len(),
        document_uid: context.document_uid.as_ptr(),
        document_uid_len: context.document_uid.len(),
    }
}
fn sentinel() -> CoordinateValue {
    CoordinateValue {
        kind: 99,
        reserved: 73,
        point: [17., 18., 19.],
        length_mm: 23.,
    }
}

#[test]
fn abi_size_probe_is_read_only_and_short_buffer_never_partially_writes() {
    let context = context();
    let input = abi(&context, "1,2,3");
    let mut value = sentinel();
    let mut error = [0u8; 512];
    let size = unsafe {
        om9_coordinate_resolve(
            &input,
            &mut value,
            std::ptr::null_mut(),
            0,
            error.as_mut_ptr(),
            error.len(),
        )
    };
    assert!(size > 1);
    assert_eq!(value, sentinel());
    let mut output = vec![0xa5; size - 1];
    let actual = unsafe {
        om9_coordinate_resolve(
            &input,
            &mut value,
            output.as_mut_ptr(),
            output.len(),
            error.as_mut_ptr(),
            error.len(),
        )
    };
    assert_eq!(actual, 0);
    assert_eq!(value, sentinel());
    assert!(output.iter().all(|v| *v == 0xa5));
    assert_ne!(error[0], 0);
    let mut output = vec![0xa5; size];
    assert_eq!(
        unsafe {
            om9_coordinate_resolve(
                &input,
                &mut value,
                output.as_mut_ptr(),
                output.len(),
                error.as_mut_ptr(),
                error.len(),
            )
        },
        size
    );
    assert_eq!(
        value,
        CoordinateValue {
            kind: 0,
            reserved: 0,
            point: [13., 21., 32.],
            length_mm: 0.
        }
    );
    assert_eq!(output[size - 1], 0);
    assert_eq!(error[0], 0);
    assert!(
        std::str::from_utf8(&output[..size - 1])
            .unwrap()
            .contains("\"point_mm\":[13,21,32]")
    );
}

#[test]
fn abi_invalid_metadata_utf8_and_bounds_leave_all_results_untouched() {
    let context = context();
    for case in 0..13 {
        let mut input = abi(&context, "1,2,3");
        let invalid_utf8 = [0xff];
        match case {
            0 => input.version = 2,
            1 => input.reserved = 1,
            2 => input.expected = 99,
            3 => input.source = 99,
            4 => input.has_previous = 2,
            5 => input.document_owner = 0,
            6 => input.frame_revision = 0,
            7 => input.logical_view_slot = 4,
            8 => input.frame[3] = f64::NAN,
            9 => {
                input.original = std::ptr::null();
                input.original_len = 1;
            }
            10 => {
                input.original = invalid_utf8.as_ptr();
                input.original_len = 1;
            }
            11 => {
                input.original = std::ptr::dangling();
                input.original_len = 1025;
            }
            _ => {
                input.unit_context = std::ptr::dangling();
                input.unit_context_len = 513;
            }
        }
        let mut value = sentinel();
        let mut output = [0xa5; 4096];
        let mut error = [0u8; 512];
        assert_eq!(
            unsafe {
                om9_coordinate_resolve(
                    &input,
                    &mut value,
                    output.as_mut_ptr(),
                    output.len(),
                    error.as_mut_ptr(),
                    error.len(),
                )
            },
            0,
            "case {case}"
        );
        assert_eq!(value, sentinel());
        assert!(output.iter().all(|v| *v == 0xa5));
        assert_ne!(error[0], 0);
    }
}

#[test]
fn abi_combined_reports_are_atomic_when_a_later_input_is_invalid() {
    let context = context();
    let inputs = [abi(&context, "1,2,3"), abi(&context, "1,,2")];
    let mut output = [0xa5; 4096];
    let mut error = [0u8; 512];
    assert_eq!(
        unsafe {
            om9_coordinate_report(
                inputs.as_ptr(),
                inputs.len(),
                output.as_mut_ptr(),
                output.len(),
                error.as_mut_ptr(),
                error.len(),
            )
        },
        0
    );
    assert!(output.iter().all(|v| *v == 0xa5));
    assert_eq!(
        unsafe {
            om9_coordinate_report(
                std::ptr::null(),
                65,
                output.as_mut_ptr(),
                output.len(),
                error.as_mut_ptr(),
                error.len(),
            )
        },
        0
    );
    assert!(output.iter().all(|v| *v == 0xa5));
    let input = abi(&context, "1,2,3");
    let size = unsafe {
        om9_coordinate_report(
            &input,
            1,
            std::ptr::null_mut(),
            0,
            error.as_mut_ptr(),
            error.len(),
        )
    };
    assert!(size > 1);
    let mut output = vec![0xa5; size];
    assert_eq!(
        unsafe {
            om9_coordinate_report(
                &input,
                1,
                output.as_mut_ptr(),
                size,
                error.as_mut_ptr(),
                error.len(),
            )
        },
        size
    );
    let actual = std::str::from_utf8(&output[..size - 1]).unwrap();
    assert_eq!(
        actual,
        report(&[resolve(&request(&context, "1,2,3")).unwrap()]).unwrap()
    );
}

#[test]
fn abi_context_verification_uses_lifetime_and_units_without_current_frame() {
    let context = context();
    let input = abi(&context, "");
    let mut error = [0u8; 512];
    let uid = context.document_uid.as_bytes();
    let verify = |owner, units: &str, error: &mut [u8]| unsafe {
        om9_coordinate_verify_context(
            &input,
            owner,
            uid.as_ptr(),
            uid.len(),
            units.as_ptr(),
            units.len(),
            error.as_mut_ptr(),
            error.len(),
        )
    };
    assert!(verify(17, "", &mut error));
    assert!(!verify(18, "", &mut error));
    assert!(!verify(17, "1|1|1|1|0.01|0.001|0.01", &mut error));
    assert!(verify(17, "", &mut error));
    assert_eq!(error[0], 0);
}
