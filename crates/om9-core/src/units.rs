// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed lengths and angles (OM9-L3-01-08-001/002/006).
//!
//! Canonical internal length is millimetres and canonical angle storage is degrees;
//! radians exist only as an explicit conversion result. A bare `f64` never crosses a
//! module boundary as a length or an angle, and the wire form is tagged by unit, so a
//! value stored in radians cannot be read back as degrees and a bare number cannot be
//! read back as millimetres.
use crate::{Error, geometry::Unit, request::Dimension};
use serde::{Deserialize, Serialize};

/// Admission bound for an absolute angle in degrees (OM9 numeric policy, not Matrix data).
pub const MAX_ABS_DEGREES: f64 = 1.0e6;

#[derive(Debug, Clone, PartialEq)]
pub enum UnitError {
    NonFinite {
        dimension: Dimension,
    },
    OutOfRange {
        dimension: Dimension,
        value: f64,
        limit: f64,
    },
    UnsupportedUnit {
        token: String,
    },
    MissingUnit {
        expected: Dimension,
    },
    WrongDimension {
        expected: Dimension,
        found: Dimension,
    },
    NotAnInteger {
        value: f64,
    },
    Malformed,
}
impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonFinite { dimension } => write!(f, "expected a finite {dimension:?} value"),
            Self::OutOfRange {
                dimension,
                value,
                limit,
            } => {
                write!(f, "{dimension:?} {value} outside +-{limit}")
            }
            Self::UnsupportedUnit { token } => write!(f, "unsupported unit {token:?}"),
            Self::MissingUnit { expected } => {
                write!(f, "a {expected:?} value needs an explicit unit")
            }
            Self::WrongDimension { expected, found } => {
                write!(f, "expected {expected:?}, found {found:?}")
            }
            Self::NotAnInteger { value } => write!(f, "{value} is not an integer count"),
            Self::Malformed => f.write_str("numeric text does not parse"),
        }
    }
}
impl std::error::Error for UnitError {}
impl From<UnitError> for Error {
    fn from(value: UnitError) -> Self {
        match value {
            // An unknown unit token is a capability refusal, not malformed data.
            UnitError::UnsupportedUnit { .. } => Error::Unsupported("unit"),
            UnitError::NonFinite { .. } => Error::Invalid("finite numeric value"),
            UnitError::OutOfRange { .. } => Error::Invalid("numeric value range"),
            UnitError::MissingUnit { .. } => Error::Invalid("missing unit"),
            UnitError::WrongDimension { .. } => Error::Invalid("numeric dimension"),
            UnitError::NotAnInteger { .. } => Error::Invalid("integer value"),
            UnitError::Malformed => Error::Invalid("numeric text"),
        }
    }
}

/// `a * b / d` with the rounding error of the product compensated by a fused multiply-add.
///
/// A plain `a * b / d` rounds twice (once for the product, once for the quotient), which costs
/// up to two ULP on a round trip. Computing the exact product residual with `mul_add` and the
/// exact quotient remainder with another `mul_add` leaves a single final addition, so a
/// conversion round trip stays within one ULP (measured in `tests/l2_01_08_precision.rs`).
fn mul_div(a: f64, b: f64, d: f64) -> f64 {
    let product = a * b;
    if !product.is_finite() || !b.is_finite() || !d.is_finite() || d == 0.0 {
        // Overflow or a degenerate ratio: fall back to the plain form and let the finite
        // check at the call site decide.
        return a * b / d;
    }
    let residual = a.mul_add(b, -product);
    let quotient = product / d;
    let remainder = quotient.mul_add(-d, product);
    quotient + (remainder + residual) / d
}

fn non_finite(dimension: Dimension) -> UnitError {
    UnitError::NonFinite { dimension }
}

/// Split an optional trailing unit token from numeric text ("2.5", "2.5 mm", "90deg").
fn split_unit(text: &str) -> (&str, Option<&str>) {
    let trimmed = text.trim();
    let cut = trimmed
        .char_indices()
        .rev()
        .find(|(_, c)| !(c.is_ascii_alphabetic() || *c == 'µ' || *c == '°'))
        .map_or(0, |(index, c)| index + c.len_utf8());
    let (number, unit) = trimmed.split_at(cut);
    if unit.is_empty() {
        (number.trim_end(), None)
    } else {
        (number.trim_end(), Some(unit))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "LengthRepr")]
pub struct LengthMm {
    mm: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LengthRepr {
    mm: f64,
}
impl TryFrom<LengthRepr> for LengthMm {
    type Error = UnitError;
    fn try_from(value: LengthRepr) -> Result<Self, UnitError> {
        Self::try_new(value.mm)
    }
}
impl LengthMm {
    pub fn try_new(mm: f64) -> Result<Self, UnitError> {
        if !mm.is_finite() {
            return Err(non_finite(Dimension::Length));
        }
        Ok(Self { mm })
    }
    /// Convert from `unit` to canonical millimetres through the exact integer ratio: one
    /// multiplication by an exactly representable numerator and one final division.
    pub fn from_unit(value: f64, unit: Unit) -> Result<Self, UnitError> {
        if !value.is_finite() {
            return Err(non_finite(Dimension::Length));
        }
        let (numerator, denominator) = unit.ratio();
        Self::try_new(mul_div(value, numerator, denominator))
    }
    /// Express canonical millimetres in `unit` for display. The division is the only rounding,
    /// and `LengthMm::from_unit(self.in_unit(u), u)` round-trips within one ULP.
    pub fn in_unit(self, unit: Unit) -> f64 {
        let (numerator, denominator) = unit.ratio();
        mul_div(self.mm, denominator, numerator)
    }
    /// `"2.5"` uses `default`; `"2.5mm"`, `"2.5 mm"` and `"0.1 in"` carry their own unit.
    pub fn parse(text: &str, default: Unit) -> Result<Self, UnitError> {
        let (number, unit) = split_unit(text);
        let value: f64 = number.parse().map_err(|_| UnitError::Malformed)?;
        let unit = match unit {
            Some(token) => Unit::parse(token)?,
            None => default,
        };
        Self::from_unit(value, unit)
    }
    pub fn mm(self) -> f64 {
        self.mm
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "DegreesRepr")]
pub struct AngleDeg {
    deg: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DegreesRepr {
    deg: f64,
}
impl TryFrom<DegreesRepr> for AngleDeg {
    type Error = UnitError;
    fn try_from(value: DegreesRepr) -> Result<Self, UnitError> {
        Self::try_new(value.deg)
    }
}
impl AngleDeg {
    pub fn try_new(deg: f64) -> Result<Self, UnitError> {
        if !deg.is_finite() {
            return Err(non_finite(Dimension::Angle));
        }
        if deg.abs() > MAX_ABS_DEGREES {
            return Err(UnitError::OutOfRange {
                dimension: Dimension::Angle,
                value: deg,
                limit: MAX_ABS_DEGREES,
            });
        }
        Ok(Self { deg })
    }
    /// `"90"`, `"90deg"`, `"90°"` are degrees; `"1.5707963rad"` converts explicitly.
    pub fn parse(text: &str) -> Result<Self, UnitError> {
        let (number, unit) = split_unit(text);
        let value: f64 = number.parse().map_err(|_| UnitError::Malformed)?;
        match unit {
            None => Self::try_new(value),
            Some(token) if token.eq_ignore_ascii_case("deg") || token == "°" => {
                Self::try_new(value)
            }
            Some(token) if token.eq_ignore_ascii_case("rad") => {
                let rad = AngleRad::try_new(value)?;
                AngleDeg::try_new(rad.rad().to_degrees())
            }
            Some(token) => Err(UnitError::UnsupportedUnit {
                token: token.chars().take(16).collect(),
            }),
        }
    }
    pub fn deg(self) -> f64 {
        self.deg
    }
    /// Radians from degrees: multiply by full-precision PI and divide by 180, so 180 degrees is
    /// exactly `std::f64::consts::PI` and 90 degrees is exactly `PI / 2` (measured in
    /// `tests/l2_01_08_precision.rs`). `f64::to_radians` multiplies by a rounded `PI/180`
    /// constant instead.
    pub fn to_radians(self) -> AngleRad {
        AngleRad {
            rad: mul_div(self.deg, std::f64::consts::PI, 180.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "RadiansRepr")]
pub struct AngleRad {
    rad: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RadiansRepr {
    rad: f64,
}
impl TryFrom<RadiansRepr> for AngleRad {
    type Error = UnitError;
    fn try_from(value: RadiansRepr) -> Result<Self, UnitError> {
        Self::try_new(value.rad)
    }
}
impl AngleRad {
    pub fn try_new(rad: f64) -> Result<Self, UnitError> {
        if !rad.is_finite() {
            return Err(non_finite(Dimension::Angle));
        }
        Ok(Self { rad })
    }
    pub fn rad(self) -> f64 {
        self.rad
    }
    /// Degrees from radians with the same single-rounding rule as `AngleDeg::to_radians`.
    pub fn to_degrees(self) -> Result<AngleDeg, UnitError> {
        AngleDeg::try_new(mul_div(self.rad, 180.0, std::f64::consts::PI))
    }
}
