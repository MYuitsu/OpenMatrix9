use openmatrix9_rust::curve_conversion::*;
use openmatrix9_rust::curve_conversion_ffi::*;
fn input(form: u32) -> ConversionInput {
    ConversionInput {
        form,
        source_domain: [0., 7.],
        target_domain: [0., 7.],
        conversion_tolerance: 0.,
        length_tolerance: 1e-7,
        parameter_tolerance: 1e-10,
        frame: QueryFrame::NativeConversionLocal,
        method: QueryMethod::OpenNurbsToOcct,
    }
}
fn row() -> ParameterObservation {
    ParameterObservation {
        source_parameter: 0.31,
        target_parameter: 0.29,
        inverse_parameter: 0.31,
        forward_success: true,
        inverse_success: true,
        source_point: [1., 2., 3.],
        intermediate_point: [1., 2., 3.],
        native_point: [1., 2., 3.],
        raw_native_point: [1., 2.1, 3.],
    }
}
#[test]
fn mapped_point_is_bound_to_the_actual_host_source() {
    let d = dependency_point(input(2), row(), [1., 2., 3.], false).unwrap();
    assert_eq!(d.target_parameter, 0.29);
    assert_eq!(d.point, [1., 2., 3.]);
    assert_eq!(d.host_source_residual, 0.);
    assert_eq!(d.host_point_residual, 0.);
    assert!(!d.used_raw);
    assert_eq!(
        dependency_point(input(2), row(), [1., 3., 3.], false),
        Err(QueryError::HostSourceDeviation)
    );
}
#[test]
fn form_two_raw_reuse_rejects_even_at_coincident_sample() {
    assert_eq!(
        dependency_point(input(2), row(), [1., 2., 3.], true),
        Err(QueryError::RawParameterUnproved)
    );
    let mut r = row();
    r.target_parameter = r.source_parameter;
    r.raw_native_point = r.source_point;
    assert_eq!(
        dependency_point(input(2), r, r.source_point, true),
        Err(QueryError::RawParameterUnproved)
    );
}
#[test]
fn form_one_proven_raw_query_is_scoped_to_this_observation() {
    let mut r = row();
    r.target_parameter = r.source_parameter;
    r.raw_native_point = r.source_point;
    let d = dependency_point(input(1), r, r.source_point, true).unwrap();
    assert!(d.used_raw);
    assert_eq!(d.point, r.raw_native_point);
    assert_eq!(d.target_parameter, r.source_parameter);
    r.raw_native_point[1] += 0.01;
    assert_eq!(
        dependency_point(input(1), r, r.source_point, true),
        Err(QueryError::RawParameterUnproved)
    );
}
#[test]
fn host_binding_is_finite_and_uses_declared_length_bound() {
    for p in [[f64::NAN, 2., 3.], [1., f64::INFINITY, 3.]] {
        assert_eq!(
            dependency_point(input(2), row(), p, false),
            Err(QueryError::NonFinite)
        );
    }
    assert!(dependency_point(input(2), row(), [1. + 5e-8, 2., 3.], false).is_ok());
    assert_eq!(
        dependency_point(input(2), row(), [1. + 2e-7, 2., 3.], false),
        Err(QueryError::HostSourceDeviation)
    );
}
#[test]
fn final_selected_point_has_its_own_host_length_bound() {
    let mut r = row();
    r.source_point = [0., 0., 0.];
    r.intermediate_point = [7.5e-8, 0., 0.];
    r.native_point = r.intermediate_point;
    let host = [-7.5e-8, 0., 0.];
    // Each leg alone is within tolerance, but the published point is not.
    assert_eq!(
        dependency_point(input(2), r, host, false),
        Err(QueryError::HostPointDeviation)
    );
    r.target_parameter = r.source_parameter;
    r.raw_native_point = r.native_point;
    assert_eq!(
        dependency_point(input(1), r, host, true),
        Err(QueryError::HostPointDeviation)
    );
}
#[test]
fn existing_native_mapping_failures_remain_fail_closed() {
    let mut r = row();
    r.forward_success = false;
    assert_eq!(
        dependency_point(input(2), r, r.source_point, false),
        Err(QueryError::ForwardMappingFailed)
    );
    let mut r = row();
    r.source_parameter = 8.;
    assert_eq!(
        dependency_point(input(2), r, r.source_point, false),
        Err(QueryError::OutsideDomain)
    );
    let mut r = row();
    r.native_point[0] += 0.01;
    assert_eq!(
        dependency_point(input(2), r, r.source_point, false),
        Err(QueryError::NativeDeviation)
    );
}
fn native() -> DependencyInput {
    let i = input(2);
    let r = row();
    DependencyInput {
        conversion: NativeConversionInput {
            form: i.form,
            method: 1,
            frame: 1,
            reserved: 0,
            source_domain: i.source_domain,
            target_domain: i.target_domain,
            conversion_tolerance: i.conversion_tolerance,
            length_tolerance: i.length_tolerance,
            parameter_tolerance: i.parameter_tolerance,
        },
        observation: NativeObservation {
            source_parameter: r.source_parameter,
            target_parameter: r.target_parameter,
            inverse_parameter: r.inverse_parameter,
            forward_success: 1,
            inverse_success: 1,
            source_point: r.source_point,
            intermediate_point: r.intermediate_point,
            native_point: r.native_point,
            raw_native_point: r.raw_native_point,
        },
        host_source_point: r.source_point,
        use_raw: 0,
        reserved: 0,
    }
}
#[test]
fn dependency_abi_is_copied_values_and_reports_scoped_point() {
    assert_eq!(std::mem::size_of::<DependencyInput>(), 232);
    assert_eq!(std::mem::align_of::<DependencyInput>(), 8);
    assert_eq!(
        std::mem::offset_of!(DependencyInput, host_source_point),
        200
    );
    assert_eq!(std::mem::size_of::<DependencySummary>(), 64);
    assert_eq!(
        std::mem::offset_of!(DependencySummary, target_parameter),
        16
    );
    let d = om9_curve_parameter_dependency(native());
    assert_eq!(d.status, 0);
    assert_eq!(d.schema_version, 1);
    assert_eq!(d.point, [1., 2., 3.]);
    assert_eq!(d.target_parameter, 0.29);
}
#[test]
fn dependency_abi_rejects_bad_flags_and_returns_no_partial_point() {
    let mut i = native();
    i.use_raw = 1;
    let d = om9_curve_parameter_dependency(i);
    assert_eq!(d.status, 115);
    assert_eq!(d.point, [0.; 3]);
    for mode in 0..4 {
        let mut i = native();
        match mode {
            0 => i.use_raw = 2,
            1 => i.reserved = 1,
            2 => i.conversion.reserved = 1,
            _ => i.observation.forward_success = 2,
        };
        let d = om9_curve_parameter_dependency(i);
        assert_eq!(d.status, 199);
        assert_eq!(d.schema_version, 0);
        assert_eq!(d.point, [0.; 3]);
    }
    let mut i = native();
    i.host_source_point = [8., 2., 3.];
    assert_eq!(om9_curve_parameter_dependency(i).status, 114);
}
