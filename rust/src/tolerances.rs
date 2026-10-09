// SPDX-License-Identifier: LGPL-2.1-or-later
//! RCORE-02 operation-specific effective tolerances. Values are canonical mm.
//! Display, topology, picking and tessellation tolerances never enter this policy.
use crate::units::Context;
use std::ffi::{CStr, c_char};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    JoinCurves = 1,
    JoinSurfaces = 2,
}
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Legacy = 0,
    Document = 1,
    Override = 2,
}
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnsupportedOperation = 1,
    InvalidContext = 2,
    InvalidTolerance = 3,
    InvalidBound = 4,
    BelowResolution = 5,
    InvalidPointer = 6,
    Internal = 7,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resolved {
    pub context_revision: u64,
    pub operation: Operation,
    pub rule_version: u32,
    pub source: Source,
    pub used_fields: u32,
    pub requested_mm: f64,
    pub effective_mm: f64,
    pub numerical_floor_mm: f64,
}

pub fn resolve(
    operation: Operation,
    context: Option<&Context>,
    override_mm: Option<f64>,
    coordinate_bound: f64,
) -> Result<Resolved, Error> {
    if let Some(context) = context {
        context.validate().map_err(|_| Error::InvalidContext)?;
    }
    if !coordinate_bound.is_finite() || coordinate_bound < 0.0 {
        return Err(Error::InvalidBound);
    }
    let (requested_mm, source) = if let Some(value) = override_mm {
        (value, Source::Override)
    } else if let Some(context) = context {
        (context.absolute_mm, Source::Document)
    } else {
        (
            match operation {
                Operation::JoinCurves => 1e-7,
                Operation::JoinSurfaces => 1e-6,
            },
            Source::Legacy,
        )
    };
    // Rhino 5's documented surface-Join threshold is 2 x document absolute.
    // An explicit OM9 override is the actual cutoff; legacy defaults stay intact.
    let effective_mm = requested_mm
        * if operation == Operation::JoinSurfaces && source == Source::Document {
            2.0
        } else {
            1.0
        };
    if !requested_mm.is_finite()
        || !(1e-9..=1e6).contains(&requested_mm)
        || !effective_mm.is_finite()
        || !(1e-9..=1e6).contains(&effective_mm)
    {
        return Err(Error::InvalidTolerance);
    }
    // Conservative arithmetic guard, not a certified OCCT geometry error bound.
    let numerical_floor_mm = 32.0 * f64::EPSILON * coordinate_bound.max(1.0);
    if effective_mm < numerical_floor_mm {
        return Err(Error::BelowResolution);
    }
    Ok(Resolved {
        context_revision: context.map_or(0, |c| c.revision),
        operation,
        rule_version: 1,
        source,
        used_fields: 1,
        requested_mm,
        effective_mm,
        numerical_floor_mm,
    })
}

impl Resolved {
    /// Only resolver-produced values reach this serializer; it retains no context.
    pub fn report(&self) -> String {
        let operation = match self.operation {
            Operation::JoinCurves => "join_curves",
            Operation::JoinSurfaces => "join_surfaces",
        };
        let source = match self.source {
            Source::Legacy => "legacy_operation_default",
            Source::Document => "document_absolute",
            Source::Override => "explicit_override_mm",
        };
        format!(
            concat!(
                "{{\"schema_version\":1,\"operation\":\"{}\",\"rule_version\":{},",
                "\"context_revision\":{},\"source\":\"{}\",\"requested_mm\":{:e},",
                "\"effective_mm\":{:e},\"numerical_floor_mm\":{:e},",
                "\"relative_used\":false,\"angular_used\":false}}"
            ),
            operation,
            self.rule_version,
            self.context_revision,
            source,
            self.requested_mm,
            self.effective_mm,
            self.numerical_floor_mm
        )
    }
}

/// # Safety
/// `raw` is null (legacy) or readable NUL-terminated UTF-8 for this call. `out`
/// points to aligned writable storage for a Resolved. Pointers are not retained;
/// native host access and pointer validity are caller obligations. Error leaves
/// output untouched. Panic containment applies to unwind builds, not panic=abort.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_tolerance_resolve(
    raw: *const c_char,
    operation: u32,
    override_mm: f64,
    has_override: bool,
    coordinate_bound: f64,
    out: *mut Resolved,
) -> u32 {
    if out.is_null() {
        return Error::InvalidPointer as u32;
    }
    let result = std::panic::catch_unwind(|| {
        let operation = match operation {
            1 => Operation::JoinCurves,
            2 => Operation::JoinSurfaces,
            _ => return Err(Error::UnsupportedOperation),
        };
        let context = if raw.is_null() {
            None
        } else {
            let text = unsafe { CStr::from_ptr(raw) }
                .to_str()
                .map_err(|_| Error::InvalidContext)?;
            if text.is_empty() {
                None
            } else {
                Some(Context::decode(text).map_err(|_| Error::InvalidContext)?)
            }
        };
        resolve(
            operation,
            context.as_ref(),
            has_override.then_some(override_mm),
            coordinate_bound,
        )
    });
    match result {
        Ok(Ok(value)) => {
            unsafe {
                out.write(value);
            }
            0
        }
        Ok(Err(error)) => error as u32,
        Err(_) => Error::Internal as u32,
    }
}

/// # Safety
/// `value` points to an unmodified successful resolver output (including valid
/// enum values). `out` is writable for capacity bytes or null for a size query.
/// No pointers are retained; a short buffer is unchanged. Return includes NUL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_tolerance_report(
    value: *const Resolved,
    out: *mut c_char,
    capacity: usize,
) -> usize {
    if value.is_null() {
        return 0;
    }
    std::panic::catch_unwind(|| {
        let text = unsafe { &*value }.report();
        let required = text.len() + 1;
        if !out.is_null() && capacity >= required {
            unsafe {
                std::ptr::copy_nonoverlapping(text.as_ptr(), out.cast(), text.len());
                *out.add(text.len()) = 0;
            }
        }
        required
    })
    .unwrap_or(0)
}
