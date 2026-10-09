use openmatrix9_rust::geometry_contract::Fact;
use openmatrix9_rust::polycurve_contract::*;
fn input() -> Input {
    Input {
        segment_count: 2,
        parent_domain: Some([2., 7.]),
        length_tolerance: 1e-7,
        parameter_tolerance: 1e-12,
        closed: false,
        joint_policy: JointPolicy::PreserveSegmentCurvesNoHeal,
    }
}
fn rows() -> [Segment; 2] {
    [
        Segment {
            ordinal: 0,
            family: Family::Line,
            child_domain: [0., 5.],
            parent_domain: Some([2., 3.]),
            start_xyz: [-5., 0., 0.],
            end_xyz: [0., 0., 0.],
        },
        Segment {
            ordinal: 1,
            family: Family::Nurbs,
            child_domain: [11., 19.],
            parent_domain: Some([3., 7.]),
            start_xyz: [0., 0., 0.],
            end_xyz: [10., 10., 0.],
        },
    ]
}
fn native() -> [NativeSegment; 2] {
    let s = rows();
    [
        NativeSegment {
            ordinal: 0,
            direction: Direction::Forward,
            start_xyz: s[0].start_xyz,
            end_xyz: s[0].end_xyz,
            shared_with_previous: None,
        },
        NativeSegment {
            ordinal: 1,
            direction: Direction::Forward,
            start_xyz: s[1].start_xyz,
            end_xyz: s[1].end_xyz,
            shared_with_previous: Some(true),
        },
    ]
}
#[test]
fn independent_child_domains_do_not_get_conflated_with_parent_intervals() {
    let r = report(input(), &rows(), &native(), 2, false, None).unwrap();
    assert_eq!(r.source[1].child_domain, [11., 19.]);
    assert_eq!(r.source[1].parent_domain, Some([3., 7.]));
    assert_eq!(r.conversion.parameter_map, Fact::Unknown);
    assert_eq!(r.conversion.parameterization, Fact::Unknown);
    assert_eq!(r.conversion.locus, Fact::Unknown);
    assert_eq!(r.conversion.deviation, Fact::Unknown);
}
#[test]
fn raw_wire_can_keep_parent_domain_unknown() {
    let mut i = input();
    i.parent_domain = None;
    let mut s = rows();
    for r in &mut s {
        r.parent_domain = None;
    }
    assert!(report(i, &s, &native(), 2, false, None).is_ok());
}
#[test]
fn count_and_logical_report_budget_precede_row_access() {
    let mut i = input();
    i.segment_count = 0;
    assert_eq!(preflight(i), Err(ContractError::Empty));
    i.segment_count = MAX_SEGMENTS + 1;
    assert_eq!(preflight(i), Err(ContractError::TooMany));
    i.segment_count = usize::MAX;
    assert_eq!(preflight(i), Err(ContractError::TooMany));
    i.segment_count = MAX_SEGMENTS;
    assert!(preflight(i).is_ok());
    assert_eq!(
        report(i, &[], &[], 0, false, None),
        Err(ContractError::IncompleteTraversal)
    );
}
#[test]
fn explicit_finite_positive_bounds_and_no_heal_policy_required() {
    for value in [0., -1., f64::NAN, f64::INFINITY] {
        let mut i = input();
        i.length_tolerance = value;
        assert_eq!(preflight(i), Err(ContractError::InvalidTolerance));
        i = input();
        i.parameter_tolerance = value;
        assert_eq!(preflight(i), Err(ContractError::InvalidTolerance));
    }
    let mut i = input();
    i.joint_policy = JointPolicy::Unsupported;
    assert_eq!(preflight(i), Err(ContractError::UnsupportedPolicy));
}
#[test]
fn malformed_domains_and_partition_intervals_reject() {
    for domain in [[2., 2.], [7., 2.], [f64::NAN, 7.], [-f64::MAX, f64::MAX]] {
        let mut i = input();
        i.parent_domain = Some(domain);
        assert_eq!(preflight(i), Err(ContractError::InvalidDomain));
        let mut s = rows();
        s[0].child_domain = domain;
        assert_eq!(
            report(input(), &s, &native(), 2, false, None),
            Err(ContractError::InvalidDomain)
        );
    }
    let mut s = rows();
    s[1].parent_domain = Some([3.1, 7.]);
    assert_eq!(
        report(input(), &s, &native(), 2, false, None),
        Err(ContractError::ParentDomainMismatch)
    );
    s = rows();
    s[1].parent_domain = Some([3., 6.9]);
    assert_eq!(
        report(input(), &s, &native(), 2, false, None),
        Err(ContractError::ParentDomainMismatch)
    );
}
#[test]
fn nested_reference_missing_children_are_explicitly_unsupported() {
    for family in [Family::Polycurve, Family::Reference, Family::Missing] {
        let mut s = rows();
        s[0].family = family;
        assert_eq!(
            report(input(), &s, &native(), 2, false, None),
            Err(ContractError::UnsupportedFamily)
        );
    }
}
#[test]
fn gap_rejects_without_changing_source_endpoints() {
    let mut s = rows();
    s[1].start_xyz[0] = 1e-3;
    let original = s;
    assert_eq!(
        report(input(), &s, &native(), 2, false, None),
        Err(ContractError::SourceGap)
    );
    assert_eq!(s, original);
}
#[test]
fn nonfinite_and_overflowed_distances_reject() {
    let mut s = rows();
    s[0].start_xyz[0] = f64::NAN;
    assert_eq!(
        report(input(), &s, &native(), 2, false, None),
        Err(ContractError::NonFinite)
    );
    s = rows();
    s[0].end_xyz = [f64::MAX, 0., 0.];
    s[1].start_xyz = [-f64::MAX, 0., 0.];
    assert_eq!(
        report(input(), &s, &native(), 2, false, None),
        Err(ContractError::SourceGap)
    );
    let mut n = native();
    n[0].start_xyz[0] = f64::INFINITY;
    assert_eq!(
        report(input(), &rows(), &n, 2, false, None),
        Err(ContractError::NonFinite)
    );
}
#[test]
fn ordinals_and_occurrences_cannot_be_skipped_or_merged() {
    let mut s = rows();
    s[1].ordinal = 3;
    assert_eq!(
        report(input(), &s, &native(), 2, false, None),
        Err(ContractError::InvalidOrdinal)
    );
    let mut n = native();
    n[1].ordinal = 0;
    assert_eq!(
        report(input(), &rows(), &n, 2, false, None),
        Err(ContractError::InvalidOrdinal)
    );
    assert_eq!(
        report(input(), &rows(), &native(), 1, false, None),
        Err(ContractError::IncompleteTraversal)
    );
    assert_eq!(
        report(input(), &rows()[..1], &native(), 2, false, None),
        Err(ContractError::IncompleteTraversal)
    );
}
#[test]
fn endpoint_order_and_native_shared_joint_are_separate_checks() {
    let mut n = native();
    std::mem::swap(&mut n[1].start_xyz, &mut n[1].end_xyz);
    assert_eq!(
        report(input(), &rows(), &n, 2, false, None),
        Err(ContractError::EndpointMismatch)
    );
    for shared in [None, Some(false)] {
        n = native();
        n[1].shared_with_previous = shared;
        assert_eq!(
            report(input(), &rows(), &n, 2, false, None),
            Err(ContractError::UnsharedJoint)
        );
    }
    n = native();
    n[0].direction = Direction::Unknown;
    assert_eq!(
        report(input(), &rows(), &n, 2, false, None),
        Err(ContractError::UnsupportedDirection)
    );
}
#[test]
fn reversed_traversal_is_valid_with_oriented_points() {
    let mut s = rows();
    s[0].start_xyz = [10., 10., 0.];
    s[0].end_xyz = [0., 0., 0.];
    s[0].family = Family::Nurbs;
    s[1].start_xyz = [0., 0., 0.];
    s[1].end_xyz = [-5., 0., 0.];
    s[1].family = Family::Line;
    let n = [
        NativeSegment {
            ordinal: 0,
            direction: Direction::Reversed,
            start_xyz: s[0].start_xyz,
            end_xyz: s[0].end_xyz,
            shared_with_previous: None,
        },
        NativeSegment {
            ordinal: 1,
            direction: Direction::Reversed,
            start_xyz: s[1].start_xyz,
            end_xyz: s[1].end_xyz,
            shared_with_previous: Some(true),
        },
    ];
    assert!(report(input(), &s, &n, 2, false, None).is_ok());
}
#[test]
fn closed_sequence_needs_geometry_and_native_topology_closure() {
    let mut i = input();
    i.closed = true;
    let mut s = rows();
    s[1].end_xyz = s[0].start_xyz;
    let mut n = native();
    n[1].end_xyz = n[0].start_xyz;
    assert!(
        report(i, &s, &n, 2, true, Some(true))
            .unwrap()
            .closing_joint
            .is_some()
    );
    assert_eq!(
        report(i, &s, &n, 2, false, Some(true)),
        Err(ContractError::ClosedMismatch)
    );
    assert_eq!(
        report(i, &s, &n, 2, true, Some(false)),
        Err(ContractError::UnsharedJoint)
    );
    assert_eq!(
        report(input(), &rows(), &native(), 2, true, None),
        Err(ContractError::ClosedMismatch)
    );
}
#[test]
fn owned_report_survives_and_does_not_certify_global_preservation() {
    let mut s = rows();
    let mut n = native();
    let r = report(input(), &s, &n, 2, false, None).unwrap();
    s[0].start_xyz[0] = 99.;
    n[0].end_xyz[0] = 99.;
    assert_eq!(s[0].start_xyz[0], 99.);
    assert_eq!(n[0].end_xyz[0], 99.);
    assert_eq!(r.source[0].start_xyz, [-5., 0., 0.]);
    assert_eq!(r.native[0].end_xyz, [0., 0., 0.]);
    assert_eq!(r.conversion.topology, Fact::Unknown);
    assert!(r.conversion.preserved_fields.is_empty());
    assert!(r.conversion.dropped_fields.is_empty());
}
