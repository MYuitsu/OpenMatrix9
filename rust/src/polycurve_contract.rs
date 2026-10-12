// SPDX-License-Identifier: LGPL-2.1-or-later
//! Ordered copied observations; no fitting, endpoint repair or native owners.
use crate::geometry_contract::{ConversionReport, RepresentationKind};
pub const MAX_SEGMENTS: usize = 4096;
pub const MAX_LOGICAL_BYTES: usize = 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointPolicy {
    PreserveSegmentCurvesNoHeal,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Line,
    Arc,
    Nurbs,
    Polyline,
    NativeOther,
    Polycurve,
    Reference,
    Missing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Reversed,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Input {
    pub segment_count: usize,
    pub parent_domain: Option<[f64; 2]>,
    pub length_tolerance: f64,
    pub parameter_tolerance: f64,
    pub closed: bool,
    pub joint_policy: JointPolicy,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub ordinal: usize,
    pub family: Family,
    pub child_domain: [f64; 2],
    pub parent_domain: Option<[f64; 2]>,
    pub start_xyz: [f64; 3],
    pub end_xyz: [f64; 3],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeSegment {
    pub ordinal: usize,
    pub direction: Direction,
    pub start_xyz: [f64; 3],
    pub end_xyz: [f64; 3],
    pub shared_with_previous: Option<bool>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Measurements {
    pub endpoint: [f64; 2],
    pub source_joint: f64,
    pub native_joint: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PolycurveReport {
    pub input: Input,
    pub source: Vec<Segment>,
    pub native: Vec<NativeSegment>,
    pub measurements: Vec<Measurements>,
    pub closing_joint: Option<[f64; 2]>,
    pub conversion: ConversionReport,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractError {
    Empty,
    TooMany,
    LogicalBudget,
    InvalidTolerance,
    InvalidDomain,
    UnsupportedPolicy,
    UnsupportedFamily,
    InvalidOrdinal,
    ParentDomainMismatch,
    NonFinite,
    SourceGap,
    UnsupportedDirection,
    EndpointMismatch,
    NativeGap,
    UnsharedJoint,
    IncompleteTraversal,
    ClosedMismatch,
    AllocationFailed,
}
fn domain(d: [f64; 2]) -> Result<(), ContractError> {
    if !d.into_iter().all(f64::is_finite) || d[0] >= d[1] || !(d[1] - d[0]).is_finite() {
        return Err(ContractError::InvalidDomain);
    }
    Ok(())
}
fn near(a: f64, b: f64, tolerance: f64) -> bool {
    let delta = (a - b).abs();
    delta.is_finite() && delta <= tolerance
}
fn points(a: [f64; 3], b: [f64; 3]) -> Result<(), ContractError> {
    if !a.into_iter().chain(b).all(f64::is_finite) {
        return Err(ContractError::NonFinite);
    }
    Ok(())
}
fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2])
}
fn bounded_distance(
    a: [f64; 3],
    b: [f64; 3],
    tolerance: f64,
    error: ContractError,
) -> Result<f64, ContractError> {
    let value = distance(a, b);
    if !value.is_finite() || value > tolerance {
        return Err(error);
    }
    Ok(value)
}
pub fn preflight(input: Input) -> Result<(), ContractError> {
    if input.segment_count == 0 {
        return Err(ContractError::Empty);
    }
    if input.segment_count > MAX_SEGMENTS {
        return Err(ContractError::TooMany);
    }
    let bytes = input
        .segment_count
        .checked_mul(
            std::mem::size_of::<Segment>()
                + std::mem::size_of::<NativeSegment>()
                + std::mem::size_of::<Measurements>(),
        )
        .and_then(|n| n.checked_add(std::mem::size_of::<PolycurveReport>()))
        .ok_or(ContractError::LogicalBudget)?;
    if bytes > MAX_LOGICAL_BYTES {
        return Err(ContractError::LogicalBudget);
    }
    if !input.length_tolerance.is_finite()
        || input.length_tolerance <= 0.
        || !input.parameter_tolerance.is_finite()
        || input.parameter_tolerance <= 0.
    {
        return Err(ContractError::InvalidTolerance);
    }
    if let Some(d) = input.parent_domain {
        domain(d)?;
    }
    if input.joint_policy != JointPolicy::PreserveSegmentCurvesNoHeal {
        return Err(ContractError::UnsupportedPolicy);
    }
    Ok(())
}
fn source_row(input: Input, row: Segment) -> Result<(), ContractError> {
    if row.ordinal >= input.segment_count {
        return Err(ContractError::InvalidOrdinal);
    }
    if matches!(
        row.family,
        Family::Polycurve | Family::Reference | Family::Missing
    ) {
        return Err(ContractError::UnsupportedFamily);
    }
    domain(row.child_domain)?;
    points(row.start_xyz, row.end_xyz)?;
    match (input.parent_domain, row.parent_domain) {
        (Some(parent), Some(d)) => {
            domain(d)?;
            if d[0] < parent[0] || d[1] > parent[1] {
                return Err(ContractError::ParentDomainMismatch);
            }
        }
        (None, None) => {}
        _ => return Err(ContractError::ParentDomainMismatch),
    }
    Ok(())
}
pub fn validate_source_step(
    input: Input,
    current: Segment,
    previous: Option<Segment>,
) -> Result<f64, ContractError> {
    preflight(input)?;
    source_row(input, current)?;
    if let Some(previous) = previous {
        source_row(input, previous)?;
        if previous.ordinal.checked_add(1) != Some(current.ordinal) {
            return Err(ContractError::InvalidOrdinal);
        }
        if let (Some(a), Some(b)) = (previous.parent_domain, current.parent_domain)
            && !near(a[1], b[0], input.parameter_tolerance)
        {
            return Err(ContractError::ParentDomainMismatch);
        }
        bounded_distance(
            previous.end_xyz,
            current.start_xyz,
            input.length_tolerance,
            ContractError::SourceGap,
        )
    } else {
        if current.ordinal != 0 {
            return Err(ContractError::InvalidOrdinal);
        }
        if let (Some(parent), Some(d)) = (input.parent_domain, current.parent_domain)
            && !near(parent[0], d[0], input.parameter_tolerance)
        {
            return Err(ContractError::ParentDomainMismatch);
        }
        Ok(0.)
    }
}
fn native_row(input: Input, row: NativeSegment) -> Result<(), ContractError> {
    if row.ordinal >= input.segment_count {
        return Err(ContractError::InvalidOrdinal);
    }
    if row.direction == Direction::Unknown {
        return Err(ContractError::UnsupportedDirection);
    }
    points(row.start_xyz, row.end_xyz)
}
pub fn validate_native_step(
    input: Input,
    source: Segment,
    native: NativeSegment,
    previous: Option<(Segment, NativeSegment)>,
) -> Result<Measurements, ContractError> {
    let source_joint = validate_source_step(input, source, previous.map(|p| p.0))?;
    native_row(input, native)?;
    if native.ordinal != source.ordinal {
        return Err(ContractError::InvalidOrdinal);
    }
    let endpoint = [
        bounded_distance(
            source.start_xyz,
            native.start_xyz,
            input.length_tolerance,
            ContractError::EndpointMismatch,
        )?,
        bounded_distance(
            source.end_xyz,
            native.end_xyz,
            input.length_tolerance,
            ContractError::EndpointMismatch,
        )?,
    ];
    let native_joint = if let Some((previous_source, previous_native)) = previous {
        native_row(input, previous_native)?;
        if previous_native.ordinal != previous_source.ordinal {
            return Err(ContractError::InvalidOrdinal);
        }
        if native.shared_with_previous != Some(true) {
            return Err(ContractError::UnsharedJoint);
        }
        bounded_distance(
            previous_native.end_xyz,
            native.start_xyz,
            input.length_tolerance,
            ContractError::NativeGap,
        )?
    } else {
        if native.shared_with_previous.is_some() {
            return Err(ContractError::UnsharedJoint);
        }
        0.
    };
    Ok(Measurements {
        endpoint,
        source_joint,
        native_joint,
    })
}
/// Final count/closure gate assumes every intermediate row was separately checked.
/// It is a fold boundary, not a whole-sequence or geometric certificate by itself.
#[allow(
    clippy::too_many_arguments,
    reason = "Fold boundary keeps source/native first/last rows and closure facts explicit"
)]
pub fn validate_finish(
    input: Input,
    first: Segment,
    last: Segment,
    first_native: NativeSegment,
    last_native: NativeSegment,
    occurrences: usize,
    native_closed: bool,
    closing_shared: Option<bool>,
) -> Result<Option<[f64; 2]>, ContractError> {
    preflight(input)?;
    source_row(input, first)?;
    source_row(input, last)?;
    native_row(input, first_native)?;
    native_row(input, last_native)?;
    if occurrences != input.segment_count {
        return Err(ContractError::IncompleteTraversal);
    }
    if first.ordinal != 0
        || last.ordinal != input.segment_count - 1
        || first_native.ordinal != first.ordinal
        || last_native.ordinal != last.ordinal
    {
        return Err(ContractError::InvalidOrdinal);
    }
    if let (Some(parent), Some(d)) = (input.parent_domain, last.parent_domain)
        && !near(parent[1], d[1], input.parameter_tolerance)
    {
        return Err(ContractError::ParentDomainMismatch);
    }
    if input.closed != native_closed {
        return Err(ContractError::ClosedMismatch);
    }
    if !input.closed {
        return Ok(None);
    }
    if closing_shared != Some(true) {
        return Err(ContractError::UnsharedJoint);
    }
    let source = bounded_distance(
        last.end_xyz,
        first.start_xyz,
        input.length_tolerance,
        ContractError::SourceGap,
    )?;
    let native = bounded_distance(
        last_native.end_xyz,
        first_native.start_xyz,
        input.length_tolerance,
        ContractError::NativeGap,
    )?;
    Ok(Some([source, native]))
}
pub fn report(
    input: Input,
    source: &[Segment],
    native: &[NativeSegment],
    occurrences: usize,
    native_closed: bool,
    closing_shared: Option<bool>,
) -> Result<PolycurveReport, ContractError> {
    preflight(input)?;
    if source.len() != input.segment_count
        || native.len() != input.segment_count
        || occurrences != input.segment_count
    {
        return Err(ContractError::IncompleteTraversal);
    }
    // Source admission precedes output acceptance, without modifying source rows.
    for (i, &row) in source.iter().enumerate() {
        validate_source_step(input, row, i.checked_sub(1).map(|p| source[p]))?;
    }
    let mut measurements = Vec::new();
    measurements
        .try_reserve_exact(input.segment_count)
        .map_err(|_| ContractError::AllocationFailed)?;
    for (i, (&s, &n)) in source.iter().zip(native).enumerate() {
        measurements.push(validate_native_step(
            input,
            s,
            n,
            i.checked_sub(1).map(|p| (source[p], native[p])),
        )?);
    }
    let end = input.segment_count - 1;
    let closing_joint = validate_finish(
        input,
        source[0],
        source[end],
        native[0],
        native[end],
        occurrences,
        native_closed,
        closing_shared,
    )?;
    let mut owned_source = Vec::new();
    let mut owned_native = Vec::new();
    owned_source
        .try_reserve_exact(input.segment_count)
        .map_err(|_| ContractError::AllocationFailed)?;
    owned_native
        .try_reserve_exact(input.segment_count)
        .map_err(|_| ContractError::AllocationFailed)?;
    owned_source.extend_from_slice(source);
    owned_native.extend_from_slice(native);
    Ok(PolycurveReport {
        input,
        source: owned_source,
        native: owned_native,
        measurements,
        closing_joint,
        conversion: ConversionReport::uninspected(
            RepresentationKind::Polycurve,
            RepresentationKind::Polycurve,
        ),
    })
}
