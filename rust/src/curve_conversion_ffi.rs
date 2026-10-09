//! Value-only native query gates. No native allocation, handles or retained pointers.
use crate::curve_conversion::*;
use crate::geometry_contract::SCHEMA_VERSION;
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NativeConversionInput {
    pub form: u32,
    pub method: u32,
    pub frame: u32,
    pub reserved: u32,
    pub source_domain: [f64; 2],
    pub target_domain: [f64; 2],
    pub conversion_tolerance: f64,
    pub length_tolerance: f64,
    pub parameter_tolerance: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct NativeObservation {
    pub source_parameter: f64,
    pub target_parameter: f64,
    pub inverse_parameter: f64,
    pub forward_success: u32,
    pub inverse_success: u32,
    pub source_point: [f64; 3],
    pub intermediate_point: [f64; 3],
    pub native_point: [f64; 3],
    pub raw_native_point: [f64; 3],
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct QuerySummary {
    pub status: u32,
    pub schema_version: u32,
    pub action: u32,
    pub raw_parameter_reusable: u32,
    pub observation: NativeObservation,
    pub residuals: [f64; 3],
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DependencyInput {
    pub conversion: NativeConversionInput,
    pub observation: NativeObservation,
    pub host_source_point: [f64; 3],
    pub use_raw: u32,
    pub reserved: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct DependencySummary {
    pub status: u32,
    pub schema_version: u32,
    pub used_raw: u32,
    pub reserved: u32,
    pub target_parameter: f64,
    pub point: [f64; 3],
    pub host_source_residual: f64,
    pub host_point_residual: f64,
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_parameter_dependency(input: DependencyInput) -> DependencySummary {
    let fail = |status| DependencySummary {
        status,
        ..Default::default()
    };
    if input.use_raw > 1 || input.reserved != 0 {
        return fail(199);
    }
    let conversion = match owned_input(input.conversion) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    let row = match owned_observation(input.observation) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    match dependency_point(conversion, row, input.host_source_point, input.use_raw == 1) {
        Err(e) => fail(status(e)),
        Ok(d) => DependencySummary {
            status: 0,
            schema_version: SCHEMA_VERSION,
            used_raw: u32::from(d.used_raw),
            reserved: 0,
            target_parameter: d.target_parameter,
            point: d.point,
            host_source_residual: d.host_source_residual,
            host_point_residual: d.host_point_residual,
        },
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_conversion_query_limit() -> usize {
    MAX_QUERIES
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_conversion_mapped_parameter(
    input: NativeConversionInput,
    parameter: f64,
    success: u32,
    direction: u32,
) -> u32 {
    if success > 1 {
        return 199;
    }
    let direction = match direction {
        1 => QueryDirection::SourceToTarget,
        2 => QueryDirection::TargetToSource,
        _ => return 199,
    };
    match owned_input(input).and_then(|i| {
        validate_mapped_parameter(i, parameter, success == 1, direction).map_err(status)
    }) {
        Ok(()) => 0,
        Err(error) => error,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_conversion_preflight(
    input: NativeConversionInput,
    count: usize,
) -> u32 {
    match owned_input(input).and_then(|i| preflight(i, count).map_err(status)) {
        Ok(()) => 0,
        Err(error) => error,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_conversion_request(
    input: NativeConversionInput,
    parameter: f64,
) -> u32 {
    match owned_input(input).and_then(|i| validate_request(i, parameter).map_err(status)) {
        Ok(action) => action_code(action),
        Err(error) => error,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_curve_conversion_observation(
    input: NativeConversionInput,
    row: NativeObservation,
) -> QuerySummary {
    let fail = |status| QuerySummary {
        status,
        ..Default::default()
    };
    let input = match owned_input(input) {
        Ok(input) => input,
        Err(e) => return fail(e),
    };
    let owned = match owned_observation(row) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    match validate_observation(input, owned) {
        Err(error) => fail(status(error)),
        Ok(valid) => {
            let v = valid.observation;
            QuerySummary {
                status: 0,
                schema_version: SCHEMA_VERSION,
                action: if input.form == 1 { 1 } else { 2 },
                raw_parameter_reusable: u32::from(valid.raw_parameter_reusable),
                observation: NativeObservation {
                    source_parameter: v.source_parameter,
                    target_parameter: v.target_parameter,
                    inverse_parameter: v.inverse_parameter,
                    forward_success: u32::from(v.forward_success),
                    inverse_success: u32::from(v.inverse_success),
                    source_point: v.source_point,
                    intermediate_point: v.intermediate_point,
                    native_point: v.native_point,
                    raw_native_point: v.raw_native_point,
                },
                residuals: [
                    valid.intermediate_residual,
                    valid.native_residual,
                    valid.raw_parameter_residual,
                ],
            }
        }
    }
}
fn owned_observation(row: NativeObservation) -> Result<ParameterObservation, u32> {
    if row.forward_success > 1 || row.inverse_success > 1 {
        return Err(199);
    }
    Ok(ParameterObservation {
        source_parameter: row.source_parameter,
        target_parameter: row.target_parameter,
        inverse_parameter: row.inverse_parameter,
        forward_success: row.forward_success == 1,
        inverse_success: row.inverse_success == 1,
        source_point: row.source_point,
        intermediate_point: row.intermediate_point,
        native_point: row.native_point,
        raw_native_point: row.raw_native_point,
    })
}
fn action_code(action: QueryAction) -> u32 {
    match action {
        QueryAction::DirectCandidate => 1,
        QueryAction::RequiresNativeMap => 2,
    }
}
fn owned_input(i: NativeConversionInput) -> Result<ConversionInput, u32> {
    if i.reserved != 0 {
        return Err(199);
    }
    let method = match i.method {
        1 => QueryMethod::OpenNurbsToOcct,
        _ => return Err(status(QueryError::UnsupportedMethod)),
    };
    let frame = match i.frame {
        1 => QueryFrame::NativeConversionLocal,
        _ => return Err(status(QueryError::UnsupportedFrame)),
    };
    Ok(ConversionInput {
        form: i.form,
        source_domain: i.source_domain,
        target_domain: i.target_domain,
        conversion_tolerance: i.conversion_tolerance,
        length_tolerance: i.length_tolerance,
        parameter_tolerance: i.parameter_tolerance,
        frame,
        method,
    })
}
fn status(e: QueryError) -> u32 {
    match e {
        QueryError::UnsupportedForm => 100,
        QueryError::InvalidDomain => 101,
        QueryError::InvalidTolerance => 102,
        QueryError::UnsupportedFrame => 103,
        QueryError::UnsupportedMethod => 104,
        QueryError::QueryLimit => 105,
        QueryError::OutsideDomain => 106,
        QueryError::NonFinite => 107,
        QueryError::ForwardMappingFailed => 108,
        QueryError::InverseMappingFailed => 109,
        QueryError::InconsistentInverse => 110,
        QueryError::IntermediateDeviation => 111,
        QueryError::NativeDeviation => 112,
        QueryError::AllocationFailed => 113,
        QueryError::HostSourceDeviation => 114,
        QueryError::RawParameterUnproved => 115,
        QueryError::HostPointDeviation => 116,
    }
}
