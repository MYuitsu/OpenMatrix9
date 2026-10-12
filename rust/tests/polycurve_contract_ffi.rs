use openmatrix9_rust::polycurve_contract_ffi::*;
fn input() -> Input {
    Input {
        count: 2,
        parent_known: 1,
        closed: 0,
        policy: 1,
        reserved: 0,
        length_tolerance: 1e-7,
        parameter_tolerance: 1e-12,
        parent_domain: [2., 7.],
    }
}
fn rows() -> [Source; 2] {
    [
        Source {
            ordinal: 0,
            family: 1,
            parent_known: 1,
            child_domain: [0., 5.],
            parent_domain: [2., 3.],
            start: [-5., 0., 0.],
            end: [0., 0., 0.],
        },
        Source {
            ordinal: 1,
            family: 3,
            parent_known: 1,
            child_domain: [11., 19.],
            parent_domain: [3., 7.],
            start: [0., 0., 0.],
            end: [10., 10., 0.],
        },
    ]
}
fn native() -> [Native; 2] {
    let s = rows();
    [
        Native {
            ordinal: 0,
            direction: 1,
            shared_previous: 0,
            start: s[0].start,
            end: s[0].end,
        },
        Native {
            ordinal: 1,
            direction: 1,
            shared_previous: 2,
            start: s[1].start,
            end: s[1].end,
        },
    ]
}
fn fail(r: Summary, status: u32) {
    assert_eq!(r.status, status);
    assert_eq!(r.schema, 0);
    assert_eq!(r.metrics, [0.; 4]);
}
#[test]
fn copied_value_abi_layout_is_explicit_on_native_x64() {
    use std::mem::{align_of, offset_of, size_of};
    assert_eq!((size_of::<Input>(), align_of::<Input>()), (56, 8));
    assert_eq!(offset_of!(Input, length_tolerance), 24);
    assert_eq!((size_of::<Source>(), align_of::<Source>()), (96, 8));
    assert_eq!(offset_of!(Source, start), 48);
    assert_eq!((size_of::<Native>(), align_of::<Native>()), (64, 8));
    assert_eq!(offset_of!(Native, start), 16);
    assert_eq!((size_of::<Summary>(), align_of::<Summary>()), (40, 8));
    assert_eq!(offset_of!(Summary, metrics), 8);
}
#[test]
fn preflight_rejects_budget_before_any_native_rows() {
    assert_eq!(om9_polycurve_limit(), 4096);
    assert_eq!(om9_polycurve_preflight(input()), 0);
    let mut i = input();
    i.count = usize::MAX;
    assert_eq!(om9_polycurve_preflight(i), 101);
    i.count = 0;
    assert_eq!(om9_polycurve_preflight(i), 100);
}
#[test]
fn source_rows_preserve_parent_and_child_domains_separately() {
    let s = rows();
    let r = om9_polycurve_source(input(), s[0], Source::default(), 0);
    assert_eq!((r.status, r.schema), (0, 1));
    assert_eq!(r.metrics, [0.; 4]);
    assert_eq!(om9_polycurve_source(input(), s[1], s[0], 1).status, 0);
    let mut bad = s[1];
    bad.parent_domain[0] = 3.1;
    fail(om9_polycurve_source(input(), bad, s[0], 1), 108);
    bad = s[1];
    bad.start[0] = 1e-3;
    fail(om9_polycurve_source(input(), bad, s[0], 1), 110);
}
#[test]
fn malformed_flags_are_protocol_errors_and_outputs_are_zeroed() {
    let s = rows();
    let n = native();
    for field in 0..4 {
        let mut i = input();
        match field {
            0 => i.parent_known = 2,
            1 => i.closed = 2,
            2 => i.reserved = 1,
            _ => i.policy = 99,
        };
        assert_eq!(om9_polycurve_preflight(i), 199);
    }
    let mut bad = s[0];
    bad.parent_known = 3;
    fail(om9_polycurve_source(input(), bad, s[0], 0), 199);
    bad = s[0];
    bad.family = 99;
    fail(om9_polycurve_source(input(), bad, s[0], 0), 199);
    fail(om9_polycurve_source(input(), s[0], s[0], 2), 199);
    let mut badn = n[0];
    badn.direction = 9;
    fail(
        om9_polycurve_native(input(), s[0], badn, s[0], n[0], 0),
        199,
    );
    badn = n[0];
    badn.shared_previous = 3;
    fail(
        om9_polycurve_native(input(), s[0], badn, s[0], n[0], 0),
        199,
    );
    fail(
        om9_polycurve_finish(input(), s[0], s[1], n[0], n[1], 2, 2, 0),
        199,
    );
    fail(
        om9_polycurve_finish(input(), s[0], s[1], n[0], n[1], 2, 0, 3),
        199,
    );
}
#[test]
fn absent_previous_and_parent_payloads_are_not_consumed() {
    let mut i = input();
    i.parent_known = 0;
    i.parent_domain = [f64::NAN; 2];
    let mut s = rows();
    for row in &mut s {
        row.parent_known = 0;
        row.parent_domain = [f64::NAN; 2];
    }
    let unused = Source {
        family: 99,
        start: [f64::NAN; 3],
        ..Default::default()
    };
    assert_eq!(om9_polycurve_source(i, s[0], unused, 0).status, 0);
    assert_eq!(
        om9_polycurve_native(
            i,
            s[0],
            native()[0],
            unused,
            Native {
                direction: 99,
                ..Default::default()
            },
            0
        )
        .status,
        0
    );
}
#[test]
fn native_endpoint_joint_and_shared_vertex_checks_are_distinct() {
    let s = rows();
    let mut n = native();
    n[0].end[0] = -6e-8;
    n[1].start[0] = 6e-8;
    fail(
        om9_polycurve_native(input(), s[1], n[1], s[0], n[0], 1),
        113,
    );
    n = native();
    n[1].shared_previous = 1;
    fail(
        om9_polycurve_native(input(), s[1], n[1], s[0], n[0], 1),
        114,
    );
    n = native();
    n[1].start[0] = 1e-3;
    fail(
        om9_polycurve_native(input(), s[1], n[1], s[0], n[0], 1),
        112,
    );
    n = native();
    n[1].direction = 2;
    let r = om9_polycurve_native(input(), s[1], n[1], s[0], n[0], 1);
    assert_eq!((r.status, r.schema), (0, 1));
}
#[test]
fn finish_checks_coverage_and_closure_as_a_fold_boundary() {
    let s = rows();
    let n = native();
    let r = om9_polycurve_finish(input(), s[0], s[1], n[0], n[1], 2, 0, 0);
    assert_eq!((r.status, r.schema), (0, 1));
    assert_eq!(r.metrics, [0.; 4]);
    fail(
        om9_polycurve_finish(input(), s[0], s[1], n[0], n[1], 1, 0, 0),
        115,
    );
    fail(
        om9_polycurve_finish(input(), s[0], s[1], n[0], n[1], 2, 1, 0),
        116,
    );
    let mut i = input();
    i.closed = 1;
    let mut s = s;
    let mut n = n;
    s[1].end = s[0].start;
    n[1].end = n[0].start;
    assert_eq!(
        om9_polycurve_finish(i, s[0], s[1], n[0], n[1], 2, 1, 2).status,
        0
    );
    fail(
        om9_polycurve_finish(i, s[0], s[1], n[0], n[1], 2, 1, 0),
        114,
    );
}
#[test]
fn unsupported_leaf_and_nonfinite_observations_reject() {
    let s = rows();
    for family in [6, 7, 8] {
        let mut row = s[0];
        row.family = family;
        fail(om9_polycurve_source(input(), row, s[0], 0), 106);
    }
    let mut row = s[0];
    row.start[0] = f64::INFINITY;
    fail(om9_polycurve_source(input(), row, s[0], 0), 109);
}
