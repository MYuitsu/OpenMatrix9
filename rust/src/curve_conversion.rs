// SPDX-License-Identifier: LGPL-2.1-or-later
//! Copied observations for one exact native conversion lineage, never a solver.
use crate::geometry_contract::{ConversionReport, RepresentationKind};

pub const MAX_QUERIES: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryFrame {
    Unknown,
    NativeConversionLocal,
    World,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryMethod {
    Unknown,
    OpenNurbsToOcct,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConversionInput {
    pub form: u32,
    pub source_domain: [f64; 2],
    pub target_domain: [f64; 2],
    pub conversion_tolerance: f64,
    pub length_tolerance: f64,
    pub parameter_tolerance: f64,
    pub frame: QueryFrame,
    pub method: QueryMethod,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryAction {
    DirectCandidate,
    RequiresNativeMap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryDirection {
    SourceToTarget,
    TargetToSource,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterObservation {
    pub source_parameter: f64,
    pub target_parameter: f64,
    pub inverse_parameter: f64,
    pub forward_success: bool,
    pub inverse_success: bool,
    pub source_point: [f64; 3],
    pub intermediate_point: [f64; 3],
    pub native_point: [f64; 3],
    pub raw_native_point: [f64; 3],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValidatedQuery {
    pub observation: ParameterObservation,
    pub intermediate_residual: f64,
    pub native_residual: f64,
    pub raw_parameter_residual: f64,
    /// This permission is restricted to this inspected query only.
    pub raw_parameter_reusable: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CurveConversionQueryReport {
    pub conversion: ConversionReport,
    pub input: ConversionInput,
    pub queries: Vec<ValidatedQuery>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryError {
    UnsupportedForm,
    InvalidDomain,
    InvalidTolerance,
    UnsupportedFrame,
    UnsupportedMethod,
    QueryLimit,
    OutsideDomain,
    NonFinite,
    ForwardMappingFailed,
    InverseMappingFailed,
    InconsistentInverse,
    IntermediateDeviation,
    NativeDeviation,
    AllocationFailed,
}
fn validate_input(input: ConversionInput) -> Result<QueryAction, QueryError> {
    let action = match input.form {
        1 => QueryAction::DirectCandidate,
        2 => QueryAction::RequiresNativeMap,
        _ => return Err(QueryError::UnsupportedForm),
    };
    for domain in [input.source_domain, input.target_domain] {
        if !domain.into_iter().all(f64::is_finite)
            || domain[0] >= domain[1]
            || !(domain[1] - domain[0]).is_finite()
        {
            return Err(QueryError::InvalidDomain);
        }
    }
    // Zero native conversion tolerance is retained as API provenance. It is
    // not interpreted as exactness or as the requested point-query bound.
    if !input.conversion_tolerance.is_finite()
        || input.conversion_tolerance < 0.
        || !input.length_tolerance.is_finite()
        || input.length_tolerance <= 0.
        || !input.parameter_tolerance.is_finite()
        || input.parameter_tolerance <= 0.
    {
        return Err(QueryError::InvalidTolerance);
    }
    if input.frame != QueryFrame::NativeConversionLocal {
        return Err(QueryError::UnsupportedFrame);
    }
    if input.method != QueryMethod::OpenNurbsToOcct {
        return Err(QueryError::UnsupportedMethod);
    }
    Ok(action)
}
fn in_domain(parameter: f64, domain: [f64; 2]) -> Result<(), QueryError> {
    if !parameter.is_finite() {
        return Err(QueryError::NonFinite);
    }
    if parameter < domain[0] || parameter > domain[1] {
        return Err(QueryError::OutsideDomain);
    }
    Ok(())
}
pub fn validate_request(input: ConversionInput, parameter: f64) -> Result<QueryAction, QueryError> {
    let action = validate_input(input)?;
    in_domain(parameter, input.source_domain)?;
    Ok(action)
}
/// Reject a failed or malformed native output before another native call uses it.
pub fn validate_mapped_parameter(
    input: ConversionInput,
    parameter: f64,
    success: bool,
    direction: QueryDirection,
) -> Result<(), QueryError> {
    validate_input(input)?;
    let (domain, failure) = match direction {
        QueryDirection::SourceToTarget => (input.target_domain, QueryError::ForwardMappingFailed),
        QueryDirection::TargetToSource => (input.source_domain, QueryError::InverseMappingFailed),
    };
    if !success {
        return Err(failure);
    }
    in_domain(parameter, domain)
}
/// Count guard before copying native query rows or allocating owned reports.
pub fn preflight(input: ConversionInput, count: usize) -> Result<(), QueryError> {
    validate_input(input)?;
    if count > MAX_QUERIES {
        return Err(QueryError::QueryLimit);
    }
    Ok(())
}
fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2])
}
pub fn validate_observation(
    input: ConversionInput,
    row: ParameterObservation,
) -> Result<ValidatedQuery, QueryError> {
    validate_request(input, row.source_parameter)?;
    // Native false has no numerical result, even if its output argument is
    // finite or appears geometrically plausible. Never validate a sentinel.
    if !row.forward_success {
        return Err(QueryError::ForwardMappingFailed);
    }
    if !row.inverse_success {
        return Err(QueryError::InverseMappingFailed);
    }
    in_domain(row.target_parameter, input.target_domain)?;
    in_domain(row.inverse_parameter, input.source_domain)?;
    if (row.inverse_parameter - row.source_parameter).abs() > input.parameter_tolerance {
        return Err(QueryError::InconsistentInverse);
    }
    if !row
        .source_point
        .into_iter()
        .chain(row.intermediate_point)
        .chain(row.native_point)
        .chain(row.raw_native_point)
        .all(f64::is_finite)
    {
        return Err(QueryError::NonFinite);
    }
    let intermediate_residual = distance(row.source_point, row.intermediate_point);
    let native_residual = distance(row.source_point, row.native_point);
    let raw_parameter_residual = distance(row.source_point, row.raw_native_point);
    if !intermediate_residual.is_finite() || intermediate_residual > input.length_tolerance {
        return Err(QueryError::IntermediateDeviation);
    }
    if !native_residual.is_finite() || native_residual > input.length_tolerance {
        return Err(QueryError::NativeDeviation);
    }
    if !raw_parameter_residual.is_finite() {
        return Err(QueryError::NonFinite);
    }
    Ok(ValidatedQuery {
        observation: row,
        intermediate_residual,
        native_residual,
        raw_parameter_residual,
        raw_parameter_reusable: input.form == 1
            && in_domain(row.source_parameter, input.target_domain).is_ok()
            && raw_parameter_residual <= input.length_tolerance
            && (row.target_parameter - row.source_parameter).abs() <= input.parameter_tolerance,
    })
}
pub fn report(
    input: ConversionInput,
    rows: &[ParameterObservation],
) -> Result<CurveConversionQueryReport, QueryError> {
    preflight(input, rows.len())?;
    let mut queries = Vec::new();
    queries
        .try_reserve_exact(rows.len())
        .map_err(|_| QueryError::AllocationFailed)?;
    for &row in rows {
        queries.push(validate_observation(input, row)?);
    }
    Ok(CurveConversionQueryReport {
        // Completed query observations are separate from global preservation,
        // affine maps, topology, attributes and certified whole-curve deviation.
        conversion: ConversionReport::uninspected(
            RepresentationKind::Curve,
            RepresentationKind::Curve,
        ),
        input,
        queries,
    })
}
