use openmatrix9_rust::phase2_ffi::*;
use openmatrix9_rust::phase2_session::*;
fn inputs(n: usize) -> Vec<RebuildInput> {
    (1..=n)
        .map(|object| RebuildInput {
            witness: Witness {
                document: 1,
                object: object as u64,
                generation: 0,
                signature: "native-shape-placement".into(),
            },
            has_dependents: false,
        })
        .collect()
}
#[test]
fn sixteen_distinct_sources_and_maximum_options_preserve_owned_request() {
    let input = inputs(16);
    let options = RebuildOptions {
        degree: 11,
        point_count: 256,
        delete_input: true,
    };
    let session = RebuildSession::new(input.clone(), options.clone()).unwrap();
    assert_eq!(session.commit_request(&input).unwrap(), options);
    let mut replaced = input;
    replaced[0].witness.signature = "changed-parent-transform".into();
    assert!(session.commit_request(&replaced).is_err());
}
#[test]
fn dependent_preview_and_delete_no_preserve_sources() {
    let mut input = inputs(1);
    input[0].has_dependents = true;
    let mut session = RebuildSession::new(
        input.clone(),
        RebuildOptions {
            degree: 3,
            point_count: 12,
            delete_input: true,
        },
    )
    .unwrap();
    assert!(session.commit_request(&input).is_err());
    session
        .replace_options(RebuildOptions {
            degree: 3,
            point_count: 12,
            delete_input: false,
        })
        .unwrap();
    assert!(!session.commit_request(&input).unwrap().delete_input);
}
#[test]
fn ffi_returns_owned_options_and_rejects_invalid_flag_and_input_count() {
    unsafe {
        let sig = b"native";
        let input = Om9RebuildInput {
            witness: Om9WitnessInput {
                document: 1,
                object: 2,
                generation: 0,
                signature: sig.as_ptr(),
                signature_len: sig.len(),
            },
            has_dependents: 0,
        };
        let options = Om9RebuildOptions {
            degree: 3,
            point_count: 12,
            delete_input: 1,
        };
        let h = om9_phase2_rebuild_create(&input, 1, &options);
        assert_ne!(h, 0);
        let mut out = Om9RebuildOptions {
            degree: 0,
            point_count: 0,
            delete_input: 0,
        };
        assert!(om9_phase2_rebuild_get(h, &mut out));
        assert_eq!((out.degree, out.point_count, out.delete_input), (3, 12, 1));
        let bad = Om9RebuildOptions {
            degree: 3,
            point_count: 12,
            delete_input: 2,
        };
        assert!(!om9_phase2_rebuild_replace(h, &bad));
        assert!(om9_phase2_rebuild_get(h, &mut out));
        assert_eq!(out.delete_input, 1);
        assert!(om9_phase2_rebuild_input_count(16));
        assert!(!om9_phase2_rebuild_input_count(17));
        om9_phase2_rebuild_drop(h);
        assert!(!om9_phase2_rebuild_get(h, &mut out));
    }
}
#[test]
fn join_unique_native_tokens_are_required_before_topology_read() {
    let ids = [1u64, 2];
    let repeated = [1u64, 1];
    unsafe {
        assert!(om9_phase2_join_selection(ids.as_ptr(), 2, 1));
        assert!(!om9_phase2_join_selection(repeated.as_ptr(), 2, 1));
        assert!(!om9_phase2_join_selection(std::ptr::null(), 17, 1));
    }
}
