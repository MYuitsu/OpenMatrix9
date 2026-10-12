//! RCORE-02: explicit document input units and tolerances; native storage is mm.
//! No implicit document tolerance or migration of existing solver thresholds.
use std::ffi::{CStr, c_char};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitError {
    InvalidContext,
    InvalidNumber,
    WrongDimension,
    InvalidAngle,
    WrongAngle,
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
                Self::InvalidAngle => "Expected a finite angle",
                Self::WrongAngle => "Unsupported or ambiguous angle",
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

const MAX_NUMERIC_BYTES: usize = 128;
// OM9 validation choices, not limits documented by Rhino: fractional integer
// components are exactly representable in f64; mixed fractions must be proper.
const MAX_FRACTION_COMPONENT: u64 = (1 << 53) - 1;
const MAX_ANGLE_DEGREES: f64 = 1e9;

fn numeric_text(raw: &str) -> Result<&str, UnitError> {
    let text = raw.trim();
    if text.is_empty() || text.len() > MAX_NUMERIC_BYTES {
        Err(UnitError::InvalidNumber)
    } else {
        Ok(text)
    }
}

fn sign_and_magnitude(text: &str) -> (f64, &str) {
    if let Some(rest) = text.strip_prefix('-') {
        (-1., rest)
    } else {
        (1., text.strip_prefix('+').unwrap_or(text))
    }
}

fn integer_component(text: &str) -> Result<f64, UnitError> {
    if text.is_empty() || !text.bytes().all(|value| value.is_ascii_digit()) {
        return Err(UnitError::WrongDimension);
    }
    let value = text.parse::<u64>().map_err(|_| UnitError::WrongDimension)?;
    if value > MAX_FRACTION_COMPONENT {
        return Err(UnitError::WrongDimension);
    }
    Ok(value as f64)
}

fn scalar(text: &str) -> Result<f64, UnitError> {
    if text.is_empty() || text.contains(char::is_whitespace) || text.contains(',') {
        return Err(UnitError::WrongDimension);
    }
    let value = if let Some((head, denominator)) = text.split_once('/') {
        let (sign, head) = sign_and_magnitude(head);
        let denominator = integer_component(denominator)?;
        if denominator == 0. {
            return Err(UnitError::WrongDimension);
        }
        let (whole, numerator) = if let Some((whole, numerator)) = head.split_once('-') {
            let whole = integer_component(whole)?;
            let numerator = integer_component(numerator)?;
            if numerator >= denominator {
                return Err(UnitError::WrongDimension);
            }
            (whole, numerator)
        } else {
            (0., integer_component(head)?)
        };
        sign * (whole + numerator / denominator)
    } else {
        text.parse::<f64>().map_err(|_| UnitError::WrongDimension)?
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err(UnitError::InvalidNumber)
    }
}

fn unsigned_scalar(text: &str) -> Result<f64, UnitError> {
    if text.starts_with(['+', '-']) {
        return Err(UnitError::WrongDimension);
    }
    strict_scalar(text)
}

fn strict_scalar(text: &str) -> Result<f64, UnitError> {
    let value = scalar(text)?;
    // New grammars must not turn a nonzero decimal into zero during parsing.
    // Fractions cannot underflow here with the bounded integer components.
    // Keep the legacy decimal length path unchanged for compatibility.
    if value == 0.
        && !text.contains('/')
        && text
            .split(['e', 'E'])
            .next()
            .unwrap_or("")
            .bytes()
            .any(|value| matches!(value, b'1'..=b'9'))
    {
        return Err(UnitError::Overflow);
    }
    Ok(value)
}

fn angle_scalar(text: &str) -> Result<f64, UnitError> {
    let value = strict_scalar(text)?;
    if value.abs() != MAX_ANGLE_DEGREES {
        return Ok(value);
    }
    // At the inclusive bound, f64 can hide a positive fractional remainder.
    // Compare the already validated, bounded token before rounding it away.
    let (_, magnitude) = sign_and_magnitude(text);
    let exceeds_bound = if let Some((head, denominator)) = magnitude.split_once('/') {
        let denominator = denominator
            .parse::<u128>()
            .map_err(|_| UnitError::Overflow)?;
        let (whole, numerator) = head.split_once('-').unwrap_or(("0", head));
        let whole = whole.parse::<u128>().map_err(|_| UnitError::Overflow)?;
        let numerator = numerator.parse::<u128>().map_err(|_| UnitError::Overflow)?;
        // scalar() bounded every component to 2^53-1, so these products fit.
        whole * denominator + numerator > 1_000_000_000_u128 * denominator
    } else {
        let (mantissa, exponent) = magnitude.split_once(['e', 'E']).unwrap_or((magnitude, "0"));
        let exponent = exponent.parse::<i32>().map_err(|_| UnitError::Overflow)?;
        let fractional_digits = mantissa.split_once('.').map_or(0, |(_, tail)| tail.len());
        let digits: String = mantissa
            .chars()
            .filter(|value| value.is_ascii_digit())
            .collect();
        let digits = digits.trim_start_matches('0');
        let order = digits.len() as i32 - fractional_digits as i32 + exponent - 1;
        order > 9
            || (order == 9
                && (digits.as_bytes()[0] != b'1' || digits[1..].bytes().any(|value| value != b'0')))
    };
    if exceeds_bound {
        Err(UnitError::Overflow)
    } else {
        Ok(value)
    }
}

fn scaled(value: f64, scale: f64) -> Result<f64, UnitError> {
    let result = value * scale;
    if !result.is_finite() || (value != 0. && result == 0.) {
        Err(UnitError::Overflow)
    } else {
        Ok(result)
    }
}

fn imperial(text: &str) -> Result<f64, UnitError> {
    if text.contains(char::is_whitespace) {
        return Err(UnitError::WrongDimension);
    }
    let (sign, magnitude) = sign_and_magnitude(text);
    let millimeters = if let Some((feet, rest)) = magnitude.split_once('\'') {
        if rest.is_empty() {
            scaled(unsigned_scalar(feet)?, 304.8)?
        } else {
            // OM9 compound-form policy: whole feet, a single overall sign,
            // and an inch remainder in [0,12). No implicit normalization.
            let feet = integer_component(feet)?;
            let inches = unsigned_scalar(rest.strip_suffix('"').ok_or(UnitError::WrongDimension)?)?;
            if !(0. ..12.).contains(&inches) {
                return Err(UnitError::WrongDimension);
            }
            scaled(feet, 304.8)? + scaled(inches, 25.4)?
        }
    } else {
        let inches = magnitude
            .strip_suffix('"')
            .ok_or(UnitError::WrongDimension)?;
        scaled(unsigned_scalar(inches)?, 25.4)?
    };
    scaled(millimeters, sign)
}

/// Parse decimal/scientific, simple fraction or mixed fraction length input.
/// Rhino 5 documents `5/16`, `1-3/4`, `1.235millimeters`, `+16'5"` and
/// `1'2-3/4"` (user_interface/unit_systems.htm). Explicit units override input
/// scale. Existing whitespace before mm/cm/m/in/ft remains accepted; compound
/// quantities contain no whitespace. Numeric bounds above are OM9 policy.
pub fn parse_length(raw: &str, mm_per_unit: f64) -> Result<f64, UnitError> {
    if !mm_per_unit.is_finite() || mm_per_unit <= 0.0 {
        return Err(UnitError::InvalidContext);
    }
    let text = numeric_text(raw)?;
    if text.contains(['\'', '"']) {
        return imperial(text);
    }
    let mut number = text;
    let mut scale = mm_per_unit;
    for (suffix, factor) in [
        ("millimeters", 1.0),
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
    scaled(scalar(number)?, scale)
}

/// Parse degrees or DMS and retain degrees for checks before trig conversion.
/// Rhino 5 documents `30d22'54.43"` and disallows internal numeric whitespace.
/// OM9 choices: lowercase `d`, one overall sign, whole degrees/minutes when
/// lower components follow, decimal seconds, minutes/seconds in [0,60), and
/// absolute total degrees <=1e9. No wrapping, radians suffix or unit inference.
pub(crate) fn angle_degrees(raw: &str) -> Result<f64, UnitError> {
    let parse_degrees = || {
        let text = numeric_text(raw)?;
        if text.contains(char::is_whitespace) {
            return Err(UnitError::WrongDimension);
        }
        if let Some((head, rest)) = text.split_once('d') {
            if rest.is_empty() {
                return angle_scalar(head);
            }
            let (sign, degrees) = sign_and_magnitude(head);
            let degrees = integer_component(degrees)?;
            let (minutes, seconds) = if let Some((minutes, rest)) = rest.split_once('\'') {
                (integer_component(minutes)?, rest)
            } else {
                (0., rest)
            };
            let seconds = if seconds.is_empty() {
                0.
            } else {
                let seconds = seconds.strip_suffix('"').ok_or(UnitError::WrongDimension)?;
                if seconds.is_empty()
                    || !seconds
                        .bytes()
                        .all(|value| value.is_ascii_digit() || value == b'.')
                {
                    return Err(UnitError::WrongDimension);
                }
                strict_scalar(seconds)?
            };
            if !(0. ..60.).contains(&minutes) || !(0. ..60.).contains(&seconds) {
                return Err(UnitError::WrongDimension);
            }
            // A tiny lower component can round away at the degree limit.
            if degrees > MAX_ANGLE_DEGREES
                || (degrees == MAX_ANGLE_DEGREES && (minutes != 0. || seconds != 0.))
            {
                return Err(UnitError::Overflow);
            }
            Ok(sign * (degrees + minutes / 60. + seconds / 3600.))
        } else {
            angle_scalar(text)
        }
    };
    let degrees = parse_degrees().map_err(|error| match error {
        UnitError::InvalidNumber => UnitError::InvalidAngle,
        UnitError::WrongDimension => UnitError::WrongAngle,
        other => other,
    })?;
    if degrees.abs() > MAX_ANGLE_DEGREES {
        return Err(UnitError::Overflow);
    }
    Ok(degrees)
}

/// Parse a bounded degree/DMS token and return radians without length scaling.
pub fn parse_angle(raw: &str) -> Result<f64, UnitError> {
    scaled(angle_degrees(raw)?, std::f64::consts::PI / 180.)
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
