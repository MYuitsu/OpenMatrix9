use openmatrix9_rust::curve_conversion_ffi::*;
fn input() -> NativeConversionInput {
    NativeConversionInput {
        form: 2,
        method: 1,
        frame: 1,
        reserved: 0,
        source_domain: [2., 7.],
        target_domain: [2., 7.],
        conversion_tolerance: 0.,
        length_tolerance: 1e-7,
        parameter_tolerance: 1e-10,
    }
}
fn row() -> NativeObservation {
    NativeObservation {
        source_parameter: 3.,
        target_parameter: 3.2,
        inverse_parameter: 3.,
        forward_success: 1,
        inverse_success: 1,
        source_point: [10., 0., 0.],
        intermediate_point: [10., 0., 0.],
        native_point: [10., 0., 0.],
        raw_native_point: [9., 1., 0.],
    }
}
#[test]
fn value_only_abi_layouts_match_native() {
    use std::mem::{align_of, offset_of, size_of};
    assert_eq!(size_of::<NativeConversionInput>(), 72);
    assert_eq!(align_of::<NativeConversionInput>(), 8);
    assert_eq!(offset_of!(NativeConversionInput, source_domain), 16);
    assert_eq!(offset_of!(NativeConversionInput, conversion_tolerance), 48);
    assert_eq!(size_of::<NativeObservation>(), 128);
    assert_eq!(align_of::<NativeObservation>(), 8);
    assert_eq!(offset_of!(NativeObservation, forward_success), 24);
    assert_eq!(offset_of!(NativeObservation, source_point), 32);
    assert_eq!(size_of::<QuerySummary>(), 168);
    assert_eq!(align_of::<QuerySummary>(), 8);
    assert_eq!(offset_of!(QuerySummary, observation), 16);
    assert_eq!(offset_of!(QuerySummary, residuals), 144);
}
#[test]
fn query_action_and_count_policy_survive_abi() {
    assert_eq!(om9_curve_conversion_request(input(), 3.), 2);
    let mut i = input();
    i.form = 1;
    assert_eq!(om9_curve_conversion_request(i, 3.), 1);
    assert_eq!(om9_curve_conversion_preflight(input(), 4096), 0);
    assert_ne!(om9_curve_conversion_preflight(input(), 4097), 0);
}
#[test]
fn valid_nonlinear_observation_returns_owned_values_and_scope() {
    let s = om9_curve_conversion_observation(input(), row());
    assert_eq!(s.status, 0);
    assert_eq!(s.schema_version, 1);
    assert_eq!(s.action, 2);
    assert_eq!(s.raw_parameter_reusable, 0);
    assert_eq!(s.observation.target_parameter, 3.2);
    assert_eq!(s.residuals[0], 0.);
    assert!(s.residuals[2] > 1.);
}
#[test]
fn finite_false_output_is_never_returned_as_success() {
    let mut r = row();
    r.forward_success = 0;
    r.target_parameter = 4.5;
    let s = om9_curve_conversion_observation(input(), r);
    assert_ne!(s.status, 0);
    assert_eq!(s.observation.target_parameter, 0.);
    assert_eq!(s.schema_version, 0);
}
#[test]
fn unknown_abi_flags_and_reserved_values_reject() {
    let mut i = input();
    i.reserved = 1;
    assert_ne!(om9_curve_conversion_preflight(i, 1), 0);
    let mut r = row();
    r.forward_success = 2;
    assert_ne!(om9_curve_conversion_observation(input(), r).status, 0);
    r = row();
    r.inverse_success = u32::MAX;
    assert_ne!(om9_curve_conversion_observation(input(), r).status, 0);
}
#[test]
fn malformed_method_frame_and_native_status_reject() {
    let mut i = input();
    i.method = 99;
    assert_ne!(om9_curve_conversion_preflight(i, 1), 0);
    i = input();
    i.frame = 99;
    assert_ne!(om9_curve_conversion_request(i, 3.), 2);
    i = input();
    i.form = 99;
    assert_ne!(om9_curve_conversion_observation(i, row()).status, 0);
}
#[test]
fn observation_bound_and_inverse_failures_are_not_published() {
    let mut r = row();
    r.native_point[0] += 0.1;
    assert_ne!(om9_curve_conversion_observation(input(), r).status, 0);
    r = row();
    r.inverse_parameter = 3.1;
    assert_ne!(om9_curve_conversion_observation(input(), r).status, 0);
    r = row();
    r.target_parameter = f64::NAN;
    assert_ne!(om9_curve_conversion_observation(input(), r).status, 0);
}
#[test]
fn mapped_parameter_is_gated_before_native_evaluation() {
    assert_eq!(om9_curve_conversion_mapped_parameter(input(), 3.2, 1, 1), 0);
    assert_eq!(om9_curve_conversion_mapped_parameter(input(), 3., 1, 2), 0);
    assert_eq!(
        om9_curve_conversion_mapped_parameter(input(), 4.5, 0, 1),
        108
    );
    assert_eq!(
        om9_curve_conversion_mapped_parameter(input(), f64::NAN, 0, 2),
        109
    );
    assert_eq!(
        om9_curve_conversion_mapped_parameter(input(), f64::NAN, 1, 1),
        107
    );
    assert_eq!(
        om9_curve_conversion_mapped_parameter(input(), 7.1, 1, 1),
        106
    );
    assert_eq!(
        om9_curve_conversion_mapped_parameter(input(), 1.9, 1, 2),
        106
    );
    assert_eq!(
        om9_curve_conversion_mapped_parameter(input(), 3., 2, 1),
        199
    );
    assert_eq!(
        om9_curve_conversion_mapped_parameter(input(), 3., 1, 99),
        199
    );
}
