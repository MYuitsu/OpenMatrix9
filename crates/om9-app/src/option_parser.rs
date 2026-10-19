// SPDX-License-Identifier: LGPL-2.1-or-later
//! The runtime option parser for interactive commands (OM9-L3-01-02-005).
//!
//! A command declares its option names in its descriptor (`Operation::options`,
//! read without case by `Operation::option`); this module turns what the user
//! typed into typed values and refuses everything else by name. It is pure: no
//! document, no session, no egui.
use om9_core::capability::{Operation, RegistryError};
use om9_core::{Error, Result};

/// One option the user typed.
#[derive(Debug, Clone, PartialEq)]
pub enum OptionValue {
    /// `BothSides=Yes` / `BothSides=No` (also `true` / `false`, `y` / `n`).
    Flag(bool),
    /// `Length=2.5mm`, `Angle=90deg`, `Length=0.25cm` — millimetres, radians.
    Number { value_mm: f64, radians: Option<f64> },
}

impl OptionValue {
    pub fn as_flag(&self) -> Option<bool> {
        match self {
            OptionValue::Flag(value) => Some(*value),
            OptionValue::Number { .. } => None,
        }
    }

    pub fn as_mm(&self) -> Option<f64> {
        match self {
            OptionValue::Number { value_mm, .. } => Some(*value_mm),
            OptionValue::Flag(_) => None,
        }
    }
}

/// One parsed option: the declared spelling plus its value.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedOption {
    pub name: String,
    pub value: OptionValue,
}

/// The unit suffixes the length parser understands. `Unit::parse` is the single unit
/// table of OM9, so the command layer cannot drift from the typed numeric layer
/// (OM9-L3-01-08-001); an empty suffix means the document unit.
fn unit_scale(unit: &str) -> Option<f64> {
    if unit.is_empty() {
        return None;
    }
    om9_core::geometry::Unit::parse(unit)
        .ok()
        .map(|unit| unit.factor())
}

fn number(text: &str, unit: &str, option: &str, default: om9_core::geometry::Unit) -> Result<f64> {
    let value: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("option number"))?;
    if !value.is_finite() {
        // NaN and Infinity are spellings the user can type; they are not numbers
        // a command can act on.
        return Err(Error::Invalid("finite option number"));
    }
    let scale = match unit {
        "deg" => {
            return Ok(value.to_radians());
        }
        "" => default.factor(),
        other => unit_scale(other).ok_or(Error::Unsupported("option unit"))?,
    };
    let _ = option;
    let mm = value * scale;
    if !mm.is_finite() {
        return Err(Error::Invalid("finite option value"));
    }
    Ok(mm)
}

/// Parse one `name=value` assignment against the declared options.
///
/// Returns `None` when the text is not an assignment at all (then it is a point
/// or a command name). Every failure names the option or the reason, and a name
/// the descriptor does not declare is refused with `UnknownOption`.
pub fn parse_option(
    default: om9_core::geometry::Unit,
    operation: &Operation,
    text: &str,
) -> Result<Option<ParsedOption>> {
    let trimmed = text.trim();
    let Some((raw_name, raw_value)) = trimmed.split_once('=') else {
        return Ok(None);
    };
    let asked = raw_name.trim();
    if asked.is_empty() {
        return Err(Error::Invalid("option name"));
    }
    // The descriptor decides how the name is spelled and whether it exists.
    let declared = operation
        .option(asked)
        .map_err(|error: RegistryError| match error {
            RegistryError::UnknownOption { .. } => Error::Unsupported("command option"),
            other => Error::Invalid(match other {
                RegistryError::EmptyName { .. } => "option name",
                _ => "command option",
            }),
        })?;
    let value = raw_value.trim();
    if value.is_empty() {
        return Err(Error::Missing("option value"));
    }
    // A flag option takes Yes/No (and the spellings Rhino accepts); anything else
    // is read as a number with an optional unit.
    let lowered = value.to_ascii_lowercase();
    let flag = match lowered.as_str() {
        "yes" | "true" | "y" | "on" => Some(true),
        "no" | "false" | "n" | "off" => Some(false),
        _ => None,
    };
    // NaN, Infinity and an overflowing literal are things a user can type and a
    // command can never act on; they are refused as numbers, not as gibberish.
    if let Ok(parsed) = value.parse::<f64>()
        && !parsed.is_finite()
    {
        return Err(Error::Invalid("finite option number"));
    }
    let (digits, unit) = split_number_unit(value);
    let value = match (flag, digits) {
        (Some(flag), _) => OptionValue::Flag(flag),
        (None, Some(digits)) => {
            let mm = number(digits, unit, declared, default)?;
            OptionValue::Number {
                value_mm: mm,
                radians: (unit == "deg").then_some(mm),
            }
        }
        (None, None) => return Err(Error::Invalid("option value")),
    };
    Ok(Some(ParsedOption {
        name: declared.to_owned(),
        value,
    }))
}

/// Split `2.5mm` into `("2.5", "mm")`; `None` when there are no digits at all.
fn split_number_unit(value: &str) -> (Option<&str>, &str) {
    let digits_end = value
        .find(|ch: char| {
            !(ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E')
        })
        .unwrap_or(value.len());
    if digits_end == 0 {
        return (None, value);
    }
    (Some(&value[..digits_end]), value[digits_end..].trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_core::capability::{Backend, Preconditions};
    use om9_core::geometry::GeometryKind;
    use om9_core::geometry::Unit;

    fn line() -> Operation {
        Operation {
            id: "line".into(),
            feature: "OM9-FEAT-001".into(),
            aliases: Vec::new(),
            prompt: "Start of line: ".into(),
            input_types: vec![GeometryKind::Point, GeometryKind::Point],
            requires_input: true,
            output: vec![GeometryKind::Curve],
            options: vec!["BothSides".into(), "Length".into(), "Angle".into()],
            preconditions: Preconditions::default(),
            backend: Backend::Rust {
                version: "1".into(),
            },
            prepare: true,
            preview: true,
            commit: true,
            receipts: Vec::new(),
        }
    }

    #[test]
    fn a_point_is_not_an_option() {
        assert_eq!(
            parse_option(Unit::Millimeter, &line(), "3,4,0").unwrap(),
            None
        );
        assert_eq!(
            parse_option(Unit::Millimeter, &line(), "Line").unwrap(),
            None
        );
    }

    #[test]
    fn flags_and_numbers_are_read_with_their_units() {
        let side = parse_option(Unit::Millimeter, &line(), "BothSides=Yes")
            .unwrap()
            .unwrap();
        assert_eq!(side.name, "BothSides");
        assert_eq!(side.value.as_flag(), Some(true));
        let length = parse_option(Unit::Millimeter, &line(), "Length=2.5mm")
            .unwrap()
            .unwrap();
        assert_eq!(length.value.as_mm(), Some(2.5));
        assert_eq!(
            parse_option(Unit::Millimeter, &line(), "Length=0.25cm")
                .unwrap()
                .unwrap()
                .value
                .as_mm(),
            Some(2.5)
        );
        let angle = parse_option(Unit::Millimeter, &line(), "Angle=90deg")
            .unwrap()
            .unwrap();
        assert!((angle.value.as_mm().unwrap() - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    }

    #[test]
    fn the_name_is_read_without_case_or_padding() {
        for spelling in ["bothsides=no", " BothSides = No ", "BOTHSIDES=false"] {
            let parsed = parse_option(Unit::Millimeter, &line(), spelling)
                .unwrap()
                .unwrap();
            assert_eq!(parsed.name, "BothSides", "{spelling}");
            assert_eq!(parsed.value.as_flag(), Some(false), "{spelling}");
        }
    }

    #[test]
    fn everything_else_is_refused_with_a_reason() {
        assert!(matches!(
            parse_option(Unit::Millimeter, &line(), "Angle=NaN"),
            Err(Error::Invalid("finite option number"))
        ));
        assert!(matches!(
            parse_option(Unit::Millimeter, &line(), "Length=1e400"),
            Err(Error::Invalid("finite option number"))
        ));
        assert!(matches!(
            parse_option(Unit::Millimeter, &line(), "Length=abc"),
            Err(Error::Invalid("option value"))
        ));
        assert!(matches!(
            parse_option(Unit::Millimeter, &line(), "Length=2.5kg"),
            Err(Error::Unsupported("option unit"))
        ));
        assert!(matches!(
            parse_option(Unit::Millimeter, &line(), "Angles=90"),
            Err(Error::Unsupported("command option"))
        ));
        assert!(matches!(
            parse_option(Unit::Millimeter, &line(), "Length="),
            Err(Error::Missing("option value"))
        ));
        assert!(matches!(
            parse_option(Unit::Millimeter, &line(), "=Yes"),
            Err(Error::Invalid("option name"))
        ));
    }
}
