//! RCORE-02: explicit document input units and tolerances; native storage is mm.
//! No implicit document tolerance or migration of existing solver thresholds.
use std::ffi::{CStr, c_char};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitError {
    InvalidContext,
    InvalidNumber,
    WrongDimension,
    Overflow,
    StaleContext,
    Schema,
}
impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidContext => "Invalid unit/tolerance context",
                Self::InvalidNumber => "Expected a finite length",
                Self::WrongDimension => "Unsupported or ambiguous length unit",
                Self::Overflow => "Unit conversion overflow",
                Self::StaleContext => "Unit context revision is stale",
                Self::Schema => "Unsupported unit context schema",
            }
        )
    }
}
impl std::error::Error for UnitError {}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub revision: u64,
    pub model_mm: f64,
    pub page_mm: f64,
    pub absolute_mm: f64,
    pub relative_ratio: f64,
    pub angular_rad: f64,
}
#[derive(Debug, Clone, Copy)]
pub enum Dimension {
    Length,
    Area,
    Volume,
    Curvature,
    GaussianCurvature,
    Unitless,
    AngleRadians,
}
#[derive(Debug, Clone, Copy)]
pub enum UnitChangeMode {
    PreservePhysicalSize,
    PreserveDeclaredNumbers,
}

impl Context {
    pub fn validate(&self) -> Result<(), UnitError> {
        if self.revision == 0
            || [
                self.model_mm,
                self.page_mm,
                self.absolute_mm,
                self.relative_ratio,
                self.angular_rad,
            ]
            .iter()
            .any(|x| !x.is_finite() || *x <= 0.0)
            || self.angular_rad > std::f64::consts::PI
        {
            return Err(UnitError::InvalidContext);
        }
        Ok(())
    }
    pub fn to_native(&self, value: f64, dimension: Dimension) -> Result<f64, UnitError> {
        self.validate()?;
        if !value.is_finite() {
            return Err(UnitError::InvalidNumber);
        }
        let power = match dimension {
            Dimension::Length => 1,
            Dimension::Area => 2,
            Dimension::Volume => 3,
            Dimension::Curvature => -1,
            Dimension::GaussianCurvature => -2,
            Dimension::Unitless | Dimension::AngleRadians => 0,
        };
        let factor = self.model_mm.powi(power);
        let result = value * factor;
        if !factor.is_finite()
            || factor <= 0.0
            || !result.is_finite()
            || (value != 0.0 && result == 0.0)
        {
            Err(UnitError::Overflow)
        } else {
            Ok(result)
        }
    }
    pub fn change_scale(&self, next: &Self, mode: UnitChangeMode) -> Result<f64, UnitError> {
        self.validate()?;
        next.validate()?;
        if next.revision <= self.revision {
            return Err(UnitError::StaleContext);
        }
        let factor = match mode {
            UnitChangeMode::PreservePhysicalSize => 1.0,
            UnitChangeMode::PreserveDeclaredNumbers => next.model_mm / self.model_mm,
        };
        if factor.is_finite() && factor > 0.0 {
            Ok(factor)
        } else {
            Err(UnitError::Overflow)
        }
    }
    pub fn encode(&self) -> Result<String, UnitError> {
        self.validate()?;
        Ok(format!(
            "1|{}|{:e}|{:e}|{:e}|{:e}|{:e}",
            self.revision,
            self.model_mm,
            self.page_mm,
            self.absolute_mm,
            self.relative_ratio,
            self.angular_rad
        ))
    }
    pub fn decode(raw: &str) -> Result<Self, UnitError> {
        if raw.len() > 512 {
            return Err(UnitError::Schema);
        }
        let fields: Vec<_> = raw.split('|').collect();
        if fields.len() != 7 || fields[0] != "1" {
            return Err(UnitError::Schema);
        }
        let number = |i: usize| {
            fields[i]
                .parse::<f64>()
                .map_err(|_| UnitError::InvalidContext)
        };
        let result = Self {
            revision: fields[1].parse().map_err(|_| UnitError::InvalidContext)?,
            model_mm: number(2)?,
            page_mm: number(3)?,
            absolute_mm: number(4)?,
            relative_ratio: number(5)?,
            angular_rad: number(6)?,
        };
        result.validate()?;
        Ok(result)
    }
}

/// Parse a length only. Decimal comma and mixed/fractional quantities are not
/// silently reinterpreted; an explicit unit overrides the command's input unit.
pub fn parse_length(raw: &str, mm_per_unit: f64) -> Result<f64, UnitError> {
    if !mm_per_unit.is_finite() || mm_per_unit <= 0.0 {
        return Err(UnitError::InvalidContext);
    }
    let text = raw.trim();
    if text.is_empty() || text.len() > 128 {
        return Err(UnitError::InvalidNumber);
    }
    let mut number = text;
    let mut scale = mm_per_unit;
    for (suffix, factor) in [
        ("mm", 1.0),
        ("cm", 10.0),
        ("in", 25.4),
        ("ft", 304.8),
        ("m", 1000.0),
    ] {
        if let Some(head) = text.strip_suffix(suffix) {
            number = head.trim();
            scale = factor;
            break;
        }
    }
    if number.is_empty() || number.contains(char::is_whitespace) || number.contains(',') {
        return Err(UnitError::WrongDimension);
    }
    let value = number
        .parse::<f64>()
        .map_err(|_| UnitError::WrongDimension)?;
    if !value.is_finite() {
        return Err(UnitError::InvalidNumber);
    }
    let result = value * scale;
    if !result.is_finite() || (value != 0.0 && result == 0.0) {
        Err(UnitError::Overflow)
    } else {
        Ok(result)
    }
}

/// # Safety
/// `value` points to an aligned, initialized Context for this call; null rejects.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_units_validate(value: *const Context) -> bool {
    !value.is_null() && unsafe { &*value }.validate().is_ok()
}
/// # Safety
/// `raw` is a NUL-terminated UTF-8 string. `out` is aligned writable Context.
/// Both remain valid for this call; no pointer is retained. Failure leaves out untouched.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_units_decode(raw: *const c_char, out: *mut Context) -> bool {
    if raw.is_null() || out.is_null() {
        return false;
    }
    let Ok(raw) = unsafe { CStr::from_ptr(raw) }.to_str() else {
        return false;
    };
    let Ok(value) = Context::decode(raw) else {
        return false;
    };
    unsafe {
        *out = value;
    }
    true
}
/// # Safety
/// `value` is an initialized Context; out is writable for capacity bytes, or null
/// to query size. The return value includes NUL; insufficient output is unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_units_encode(
    value: *const Context,
    out: *mut c_char,
    capacity: usize,
) -> usize {
    if value.is_null() {
        return 0;
    }
    let Ok(text) = unsafe { &*value }.encode() else {
        return 0;
    };
    let required = text.len() + 1;
    if !out.is_null() && capacity >= required {
        unsafe {
            std::ptr::copy_nonoverlapping(text.as_ptr(), out.cast(), text.len());
            *out.add(text.len()) = 0;
        }
    }
    required
}
/// # Safety
/// old/next point to initialized Context values; scale is one writable double.
/// mode 0 preserves physical size, 1 preserves declared numbers. No pointer retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_units_change_scale(
    old: *const Context,
    next: *const Context,
    mode: u32,
    scale: *mut f64,
) -> bool {
    if old.is_null() || next.is_null() || scale.is_null() {
        return false;
    }
    let mode = match mode {
        0 => UnitChangeMode::PreservePhysicalSize,
        1 => UnitChangeMode::PreserveDeclaredNumbers,
        _ => return false,
    };
    let Ok(value) = unsafe { &*old }.change_scale(unsafe { &*next }, mode) else {
        return false;
    };
    unsafe {
        *scale = value;
    }
    true
}
