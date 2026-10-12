use openmatrix9_rust::curve_conversion::*;
use openmatrix9_rust::geometry_contract::Fact;
fn input(form: u32) -> ConversionInput {
    ConversionInput {
        form,
        source_domain: [2., 7.],
        target_domain: [2., 7.],
        conversion_tolerance: 0.,
        length_tolerance: 1e-7,
        parameter_tolerance: 1e-10,
        frame: QueryFrame::NativeConversionLocal,
        method: QueryMethod::OpenNurbsToOcct,
    }
}
fn row() -> ParameterObservation {
    ParameterObservation {
        source_parameter: 3.,
        target_parameter: 3.2,
        inverse_parameter: 3.,
        forward_success: true,
        inverse_success: true,
        source_point: [10., 0., 0.],
        intermediate_point: [10., 0., 0.],
        native_point: [10., 0., 0.],
        raw_native_point: [9., 1., 0.],
    }
}
#[test]
fn native_statuses_are_distinct() {
    assert_eq!(
        validate_request(input(1), 3.),
        Ok(QueryAction::DirectCandidate)
    );
    assert_eq!(
        validate_request(input(2), 3.),
        Ok(QueryAction::RequiresNativeMap)
    );
    for form in [0, 3, u32::MAX] {
        assert_eq!(
            validate_request(input(form), 3.),
            Err(QueryError::UnsupportedForm)
        );
    }
}
#[test]
fn matching_domains_do_not_allow_nonlinear_raw_reuse() {
    let v = validate_observation(input(2), row()).unwrap();
    assert!(!v.raw_parameter_reusable);
    assert!(v.raw_parameter_residual > 1.);
    assert_eq!(v.observation.target_parameter, 3.2);
}
#[test]
fn known_native_identity_is_per_query_only() {
    let mut r = row();
    r.target_parameter = r.source_parameter;
    r.raw_native_point = r.source_point;
    assert!(
        validate_observation(input(1), r)
            .unwrap()
            .raw_parameter_reusable
    );
    assert!(
        !report(input(1), &[r])
            .unwrap()
            .conversion
            .source_parameter_is_reusable()
    );
}
#[test]
fn near_target_boundary_does_not_allow_raw_extrapolation() {
    let mut i = input(1);
    i.parameter_tolerance = 0.01;
    i.target_domain = [3.001, 7.];
    let mut r = row();
    r.target_parameter = 3.001;
    r.raw_native_point = r.source_point;
    assert!(!validate_observation(i, r).unwrap().raw_parameter_reusable);
}
#[test]
fn false_forward_ignores_finite_output_sentinel() {
    let mut r = row();
    r.forward_success = false;
    r.target_parameter = 4.5;
    assert_eq!(
        validate_observation(input(2), r),
        Err(QueryError::ForwardMappingFailed)
    );
}
#[test]
fn false_inverse_ignores_nonfinite_output_sentinel() {
    let mut r = row();
    r.inverse_success = false;
    r.inverse_parameter = f64::NAN;
    assert_eq!(
        validate_observation(input(2), r),
        Err(QueryError::InverseMappingFailed)
    );
}
#[test]
fn request_does_not_wrap_or_extrapolate_closed_parameters() {
    for p in [1.9, 7.1] {
        assert_eq!(
            validate_request(input(2), p),
            Err(QueryError::OutsideDomain)
        );
    }
    for p in [2., 7.] {
        assert!(validate_request(input(2), p).is_ok());
    }
}
#[test]
fn nonfinite_and_invalid_domains_reject() {
    for d in [[2., 2.], [7., 2.], [f64::NAN, 7.], [-f64::MAX, f64::MAX]] {
        let mut i = input(2);
        i.source_domain = d;
        assert_eq!(validate_request(i, 3.), Err(QueryError::InvalidDomain));
        i = input(2);
        i.target_domain = d;
        assert_eq!(validate_request(i, 3.), Err(QueryError::InvalidDomain));
    }
    assert_eq!(
        validate_request(input(2), f64::INFINITY),
        Err(QueryError::NonFinite)
    );
}
#[test]
fn tolerances_must_be_explicit_finite_and_positive() {
    for value in [0., -1., f64::NAN, f64::INFINITY] {
        let mut i = input(2);
        i.length_tolerance = value;
        assert_eq!(validate_request(i, 3.), Err(QueryError::InvalidTolerance));
        i = input(2);
        i.parameter_tolerance = value;
        assert_eq!(validate_request(i, 3.), Err(QueryError::InvalidTolerance));
    }
    let mut i = input(2);
    i.conversion_tolerance = -1.;
    assert_eq!(validate_request(i, 3.), Err(QueryError::InvalidTolerance));
}
#[test]
fn unsupported_frame_or_method_cannot_claim_correspondence() {
    for f in [QueryFrame::Unknown, QueryFrame::World] {
        let mut i = input(2);
        i.frame = f;
        assert_eq!(validate_request(i, 3.), Err(QueryError::UnsupportedFrame));
    }
    let mut i = input(2);
    i.method = QueryMethod::Unknown;
    assert_eq!(validate_request(i, 3.), Err(QueryError::UnsupportedMethod));
}
#[test]
fn successful_mapping_values_and_points_are_checked() {
    let mut r = row();
    r.target_parameter = 7.1;
    assert_eq!(
        validate_observation(input(2), r),
        Err(QueryError::OutsideDomain)
    );
    r = row();
    r.inverse_parameter = 3.1;
    assert_eq!(
        validate_observation(input(2), r),
        Err(QueryError::InconsistentInverse)
    );
    r = row();
    r.native_point[0] = f64::NAN;
    assert_eq!(
        validate_observation(input(2), r),
        Err(QueryError::NonFinite)
    );
    r = row();
    r.intermediate_point[0] += 0.01;
    assert_eq!(
        validate_observation(input(2), r),
        Err(QueryError::IntermediateDeviation)
    );
    r = row();
    r.native_point[0] += 0.01;
    assert_eq!(
        validate_observation(input(2), r),
        Err(QueryError::NativeDeviation)
    );
}
#[test]
fn no_aggregate_report_is_published_after_a_bad_row() {
    let mut bad = row();
    bad.forward_success = false;
    assert_eq!(
        report(input(2), &[row(), bad]),
        Err(QueryError::ForwardMappingFailed)
    );
}
#[test]
fn copied_query_report_never_fabricates_global_map_or_deviation() {
    let mut rows = vec![row()];
    let r = report(input(2), &rows).unwrap();
    rows[0].native_point[0] += 999.;
    assert_eq!(r.queries[0].observation.native_point, [10., 0., 0.]);
    assert_eq!(r.conversion.parameter_map, Fact::Unknown);
    assert_eq!(r.conversion.parameterization, Fact::Unknown);
    assert_eq!(r.conversion.deviation, Fact::Unknown);
}
#[test]
fn query_budget_is_checked_before_reading_rows() {
    let rows = vec![row(); MAX_QUERIES + 1];
    assert_eq!(report(input(2), &rows), Err(QueryError::QueryLimit));
    assert!(report(input(2), &vec![row(); MAX_QUERIES]).is_ok());
    assert!(report(input(2), &[]).unwrap().queries.is_empty());
}
