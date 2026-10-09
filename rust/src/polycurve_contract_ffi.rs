//! Copied-value segment gates. No native pointers, owners or allocations cross ABI.
use crate::geometry_contract::SCHEMA_VERSION;
use crate::polycurve_contract as core;
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Input {
    pub count: usize,
    pub parent_known: u32,
    pub closed: u32,
    pub policy: u32,
    pub reserved: u32,
    pub length_tolerance: f64,
    pub parameter_tolerance: f64,
    pub parent_domain: [f64; 2],
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Source {
    pub ordinal: usize,
    pub family: u32,
    pub parent_known: u32,
    pub child_domain: [f64; 2],
    pub parent_domain: [f64; 2],
    pub start: [f64; 3],
    pub end: [f64; 3],
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Native {
    pub ordinal: usize,
    pub direction: u32,
    pub shared_previous: u32,
    pub start: [f64; 3],
    pub end: [f64; 3],
}
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Summary {
    pub status: u32,
    pub schema: u32,
    pub metrics: [f64; 4],
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_polycurve_limit() -> usize {
    core::MAX_SEGMENTS
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_polycurve_preflight(input: Input) -> u32 {
    match owned_input(input).and_then(|i| core::preflight(i).map_err(status)) {
        Ok(()) => 0,
        Err(e) => e,
    }
}
/// metrics[0] is the source joint distance; other fields are zero.
#[unsafe(no_mangle)]
pub extern "C" fn om9_polycurve_source(
    input: Input,
    current: Source,
    previous: Source,
    has_previous: u32,
) -> Summary {
    summary((|| {
        let i = owned_input(input)?;
        let current = source(current)?;
        let previous = optional(has_previous, || source(previous))?;
        let joint = core::validate_source_step(i, current, previous).map_err(status)?;
        Ok([joint, 0., 0., 0.])
    })())
}
/// metrics are oriented start/end residuals, source joint and native joint.
#[unsafe(no_mangle)]
pub extern "C" fn om9_polycurve_native(
    input: Input,
    current: Source,
    observed: Native,
    previous: Source,
    previous_native: Native,
    has_previous: u32,
) -> Summary {
    summary((|| {
        let i = owned_input(input)?;
        let s = source(current)?;
        let n = native(observed)?;
        let previous = optional(has_previous, || {
            Ok((source(previous)?, native(previous_native)?))
        })?;
        let m = core::validate_native_step(i, s, n, previous).map_err(status)?;
        Ok([m.endpoint[0], m.endpoint[1], m.source_joint, m.native_joint])
    })())
}
/// Count/closure fold boundary only: callers must check every source/native row.
/// metrics[0..2] are source/native closing distances, or zero for an open wire.
#[unsafe(no_mangle)]
pub extern "C" fn om9_polycurve_finish(
    input: Input,
    first: Source,
    last: Source,
    first_native: Native,
    last_native: Native,
    occurrences: usize,
    native_closed: u32,
    closing_shared: u32,
) -> Summary {
    summary((|| {
        let i = owned_input(input)?;
        if native_closed > 1 {
            return Err(199);
        }
        let closed = core::validate_finish(
            i,
            source(first)?,
            source(last)?,
            native(first_native)?,
            native(last_native)?,
            occurrences,
            native_closed == 1,
            shared(closing_shared)?,
        )
        .map_err(status)?;
        let m = closed.unwrap_or([0.; 2]);
        Ok([m[0], m[1], 0., 0.])
    })())
}
fn summary(result: Result<[f64; 4], u32>) -> Summary {
    match result {
        Ok(metrics) => Summary {
            status: 0,
            schema: SCHEMA_VERSION,
            metrics,
        },
        Err(status) => Summary {
            status,
            ..Default::default()
        },
    }
}
fn optional<T>(flag: u32, read: impl FnOnce() -> Result<T, u32>) -> Result<Option<T>, u32> {
    match flag {
        0 => Ok(None),
        1 => read().map(Some),
        _ => Err(199),
    }
}
fn owned_input(i: Input) -> Result<core::Input, u32> {
    if i.closed > 1 || i.reserved != 0 || i.policy != 1 {
        return Err(199);
    }
    Ok(core::Input {
        segment_count: i.count,
        parent_domain: optional(i.parent_known, || Ok(i.parent_domain))?,
        length_tolerance: i.length_tolerance,
        parameter_tolerance: i.parameter_tolerance,
        closed: i.closed == 1,
        joint_policy: core::JointPolicy::PreserveSegmentCurvesNoHeal,
    })
}
fn source(s: Source) -> Result<core::Segment, u32> {
    let family = match s.family {
        1 => core::Family::Line,
        2 => core::Family::Arc,
        3 => core::Family::Nurbs,
        4 => core::Family::Polyline,
        5 => core::Family::NativeOther,
        6 => core::Family::Polycurve,
        7 => core::Family::Reference,
        8 => core::Family::Missing,
        _ => return Err(199),
    };
    Ok(core::Segment {
        ordinal: s.ordinal,
        family,
        child_domain: s.child_domain,
        parent_domain: optional(s.parent_known, || Ok(s.parent_domain))?,
        start_xyz: s.start,
        end_xyz: s.end,
    })
}
fn shared(tag: u32) -> Result<Option<bool>, u32> {
    match tag {
        0 => Ok(None),
        1 => Ok(Some(false)),
        2 => Ok(Some(true)),
        _ => Err(199),
    }
}
fn native(n: Native) -> Result<core::NativeSegment, u32> {
    let direction = match n.direction {
        0 => core::Direction::Unknown,
        1 => core::Direction::Forward,
        2 => core::Direction::Reversed,
        _ => return Err(199),
    };
    Ok(core::NativeSegment {
        ordinal: n.ordinal,
        direction,
        start_xyz: n.start,
        end_xyz: n.end,
        shared_with_previous: shared(n.shared_previous)?,
    })
}
fn status(e: core::ContractError) -> u32 {
    match e {
        core::ContractError::Empty => 100,
        core::ContractError::TooMany => 101,
        core::ContractError::LogicalBudget => 102,
        core::ContractError::InvalidTolerance => 103,
        core::ContractError::InvalidDomain => 104,
        core::ContractError::UnsupportedPolicy => 105,
        core::ContractError::UnsupportedFamily => 106,
        core::ContractError::InvalidOrdinal => 107,
        core::ContractError::ParentDomainMismatch => 108,
        core::ContractError::NonFinite => 109,
        core::ContractError::SourceGap => 110,
        core::ContractError::UnsupportedDirection => 111,
        core::ContractError::EndpointMismatch => 112,
        core::ContractError::NativeGap => 113,
        core::ContractError::UnsharedJoint => 114,
        core::ContractError::IncompleteTraversal => 115,
        core::ContractError::ClosedMismatch => 116,
        core::ContractError::AllocationFailed => 117,
    }
}
