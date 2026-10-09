use openmatrix9_rust::core_objects::Stamp;
use openmatrix9_rust::geometry_contract_ffi::*;
#[test]
fn descriptor_abi_has_matching_scope_layouts() {
    use std::mem::{align_of, offset_of, size_of};
    assert_eq!(size_of::<DescriptorInput>(), 104);
    assert_eq!(align_of::<DescriptorInput>(), 8);
    assert_eq!(offset_of!(DescriptorInput, facts_scope), 96);
    assert_eq!(offset_of!(DescriptorInput, domain_scope), 100);
    assert_eq!(size_of::<DescriptorSummary>(), 72);
    assert_eq!(align_of::<DescriptorSummary>(), 8);
    assert_eq!(offset_of!(DescriptorSummary, facts_scope), 64);
    assert_eq!(offset_of!(DescriptorSummary, domain_scope), 68);
}
#[test]
fn companion_budget_is_checked_before_native_extraction() {
    assert_eq!(om9_geometry_companion_counts(8192, 0, 2, 0, 0), 0);
    assert_eq!(om9_geometry_companion_counts(8193, 0, 2, 0, 0), 1);
    assert_eq!(om9_geometry_companion_counts(128, 64, 2, 2, 1), 0);
    assert_eq!(om9_geometry_companion_counts(129, 64, 2, 2, 1), 1);
    assert_eq!(om9_geometry_companion_counts(usize::MAX, 2, 2, 2, 1), 2);
    assert_eq!(om9_geometry_companion_counts(2, 0, 2, 0, 2), 2);
    assert_eq!(om9_geometry_companion_counts(2, 0, 16385, 0, 0), 1);
}
#[test]
fn inspected_domain_and_form_are_retained() {
    let mut a = input(2);
    a.legacy_kind = 2;
    a.domain_count = 1;
    a.domains = [0., 1., 0., 0.];
    a.form = 1;
    a.facts_scope = 1;
    a.domain_scope = 2;
    let mut b = a;
    b.domains = [2., 3., 0., 0.];
    b.form = 2;
    assert_ne!(
        format!("{:?}", om9_geometry_descriptor(a)),
        format!("{:?}", om9_geometry_descriptor(b))
    );
}
fn input(rep: u32) -> DescriptorInput {
    DescriptorInput {
        stamp: Stamp {
            document: 1,
            object: 2,
            generation: 3,
            revision: 4,
            epoch: 5,
        },
        legacy_kind: 3,
        representation: rep,
        known: 0,
        values: 0,
        domains: [0.; 4],
        domain_count: 0,
        form: 0,
        facts_scope: 0,
        domain_scope: 0,
    }
}
#[test]
fn existing_exchange_gates() {
    for k in 1..=5 {
        assert!(om9_geometry_operation(2, k, 5, 1) < 100);
        assert!(om9_geometry_operation(3, k, 5, 1) < 100);
    }
    for k in 6..=10 {
        assert!(om9_geometry_operation(3, k, 5, 1) >= 100);
    }
}
#[test]
fn unknown_operations_options_reject() {
    assert!(om9_geometry_operation(3, 3, 4, 1) >= 100);
    assert!(om9_geometry_operation(3, 3, 5, 2) >= 100);
    assert!(om9_geometry_operation(999, 3, 5, 1) >= 100);
}
#[test]
fn face_and_shell_do_not_imply_solid() {
    for rep in [4, 5] {
        let s = om9_geometry_descriptor(input(rep));
        assert_eq!(s.status, 0);
        assert_eq!(s.solid, 1);
    }
}
#[test]
fn solid_comes_from_native_type_observation() {
    let s = om9_geometry_descriptor(input(6));
    assert_eq!(s.status, 0);
    assert_eq!(s.solid, 2);
}
#[test]
fn unknown_representation_is_explicit() {
    let s = om9_geometry_descriptor(input(0));
    assert_eq!(s.status, 0);
    assert_eq!(s.representation, 0);
    assert_eq!(s.solid, 0);
}
#[test]
fn periodic_surface_remains_surface() {
    let mut i = input(4);
    i.known = 0x1f;
    i.values = 0x1f;
    i.domains = [0., std::f64::consts::TAU, 0., 7.];
    i.domain_count = 2;
    i.form = 2;
    i.facts_scope = 1;
    i.domain_scope = 3;
    let s = om9_geometry_descriptor(i);
    assert_eq!(s.status, 0);
    assert_eq!(s.solid, 1);
}
#[test]
fn impossible_metadata_rejects() {
    let mut i = input(2);
    i.legacy_kind = 4;
    assert_ne!(om9_geometry_descriptor(i).status, 0);
    i = input(2);
    i.stamp.generation = 0;
    assert_ne!(om9_geometry_descriptor(i).status, 0);
    i = input(2);
    i.domain_count = 1;
    i.domains = [0., f64::NAN, 0., 0.];
    assert_ne!(om9_geometry_descriptor(i).status, 0);
}
#[test]
fn unknown_flags_reject() {
    let mut i = input(4);
    i.known = 0x8000;
    assert_ne!(om9_geometry_descriptor(i).status, 0);
    i = input(4);
    i.values = 1;
    assert_ne!(om9_geometry_descriptor(i).status, 0);
}

#[test]
fn supporting_flags_do_not_describe_a_restricted_domain() {
    for (rep, kind, domain_scope, domain_count) in [(2, 2, 2, 1), (4, 3, 3, 2)] {
        let mut i = input(rep);
        i.legacy_kind = kind;
        i.known = 3;
        i.values = 3;
        i.form = 1;
        i.facts_scope = 1;
        i.domain_scope = domain_scope;
        i.domain_count = domain_count;
        i.domains = [0., std::f64::consts::PI, 0., 7.];
        let s = om9_geometry_descriptor(i);
        assert_eq!(s.status, 0);
        assert_eq!(s.facts_scope, 1);
        assert_eq!(s.domain_scope, domain_scope);
    }
}

#[test]
fn unscoped_or_incompatible_native_facts_reject() {
    let mut i = input(2);
    i.legacy_kind = 2;
    i.known = 1;
    assert_ne!(om9_geometry_descriptor(i).status, 0);
    i.facts_scope = 1;
    i.domain_count = 1;
    i.domains = [0., 1., 0., 0.];
    assert_ne!(om9_geometry_descriptor(i).status, 0);
    i.domain_scope = 3;
    assert_ne!(om9_geometry_descriptor(i).status, 0);
    i.domain_scope = 2;
    i.facts_scope = 99;
    assert_ne!(om9_geometry_descriptor(i).status, 0);
}
