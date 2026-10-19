// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-01-08 typed numeric contract:
//! - L3-001 canonical millimetre lengths and unit tokens
//! - L3-002 degrees with explicit radian conversion
//! - L3-004 typed parameter values (length/angle/count/ratio)
//! - L3-006 unit-safe serialization
use om9_core::{
    Error,
    budget::Limits,
    geometry::Unit,
    request::{Dimension, OptionDomain, Value},
    units::{AngleDeg, AngleRad, LengthMm, MAX_ABS_DEGREES, UnitError},
};

/* ------------------------------ L3-001 ------------------------------ */

#[test]
fn l3_001_lengths_are_finite_millimetres() {
    assert_eq!(
        LengthMm::try_new(f64::NAN),
        Err(UnitError::NonFinite {
            dimension: Dimension::Length
        })
    );
    assert_eq!(
        LengthMm::try_new(f64::INFINITY),
        Err(UnitError::NonFinite {
            dimension: Dimension::Length
        })
    );
    assert_eq!(LengthMm::try_new(2.5).unwrap().mm(), 2.5);
}

#[test]
fn l3_001_length_text_accepts_a_bare_number_with_the_document_unit() {
    let parsed = LengthMm::parse("2.5", Unit::Millimeter).unwrap();
    assert_eq!(parsed.mm(), 2.5);
    // A token carried by the text overrides the document default.
    assert_eq!(
        LengthMm::parse("2.5cm", Unit::Millimeter).unwrap().mm(),
        25.0
    );
    assert_eq!(
        LengthMm::parse("0.1 in", Unit::Millimeter).unwrap().mm(),
        2.54
    );
    assert_eq!(
        LengthMm::parse("2.5", Unit::Inch).unwrap().mm(),
        63.5,
        "the document unit is the default"
    );
    assert_eq!(
        LengthMm::parse("2.5nope", Unit::Millimeter),
        Err(UnitError::UnsupportedUnit {
            token: "nope".into()
        })
    );
    assert_eq!(
        LengthMm::parse("abc", Unit::Millimeter),
        Err(UnitError::Malformed)
    );
}

#[test]
fn l3_001_unit_tokens_are_case_insensitive_and_round_trip_their_symbol() {
    for (token, unit) in [
        ("mm", Unit::Millimeter),
        ("CM", Unit::Centimeter),
        ("m", Unit::Meter),
        ("IN", Unit::Inch),
        ("ft", Unit::Foot),
        ("um", Unit::Micrometer),
        ("µm", Unit::Micrometer),
    ] {
        let parsed = Unit::parse(token).unwrap();
        assert_eq!(parsed, unit, "{token}");
        assert_eq!(Unit::parse(parsed.symbol()).unwrap(), unit);
    }
    assert!(matches!(
        Unit::parse("furlong"),
        Err(UnitError::UnsupportedUnit { .. })
    ));
    // UnitError converts to the crate error: an unknown unit is a capability refusal.
    assert_eq!(
        Error::from(UnitError::UnsupportedUnit { token: "x".into() }),
        Error::Unsupported("unit")
    );
}

/* ------------------------------ L3-002 ------------------------------ */

#[test]
fn l3_002_degrees_convert_to_radians_within_the_contract_tolerance() {
    let half_turn = AngleDeg::try_new(180.).unwrap();
    let radians = half_turn.to_radians();
    assert!((radians.rad() - std::f64::consts::PI).abs() <= 8.9e-16);
    assert_eq!(radians.to_degrees().unwrap(), half_turn);
    assert_eq!(AngleRad::try_new(1.0).unwrap().rad(), 1.0);
}

#[test]
fn l3_002_angle_text_parses_degrees_and_explicit_radians() {
    assert_eq!(AngleDeg::parse("90").unwrap().deg(), 90.);
    assert_eq!(AngleDeg::parse("90deg").unwrap().deg(), 90.);
    assert_eq!(AngleDeg::parse("90 °").unwrap().deg(), 90.);
    let from_radians = AngleDeg::parse("1.5707963267948966rad").unwrap();
    assert!((from_radians.deg() - 90.).abs() < 1e-9);
    assert!(matches!(
        AngleDeg::parse("90grad"),
        Err(UnitError::UnsupportedUnit { .. })
    ));
}

#[test]
fn l3_002_an_angle_beyond_the_numeric_bound_is_refused() {
    assert_eq!(
        AngleDeg::try_new(MAX_ABS_DEGREES + 1.),
        Err(UnitError::OutOfRange {
            dimension: Dimension::Angle,
            value: MAX_ABS_DEGREES + 1.,
            limit: MAX_ABS_DEGREES,
        })
    );
    assert_eq!(
        AngleDeg::try_new(f64::NAN),
        Err(UnitError::NonFinite {
            dimension: Dimension::Angle
        })
    );
}

/* ------------------------------ L3-004 ------------------------------ */

#[test]
fn l3_004_typed_values_carry_their_dimension_and_read_back_typed() {
    let length = Value::length(LengthMm::try_new(1.25).unwrap());
    assert_eq!(
        length,
        Value::Quantity {
            value: 1.25,
            dimension: Dimension::Length
        }
    );
    assert_eq!(length.as_length().unwrap().mm(), 1.25);
    let angle = Value::angle(AngleDeg::try_new(45.).unwrap());
    assert_eq!(angle.as_angle().unwrap().deg(), 45.);
    assert_eq!(Value::count(3).as_count().unwrap(), 3);
    assert_eq!(
        Value::count(3).as_count(),
        Ok(3),
        "count is an integer quantity"
    );
    assert!(matches!(
        Value::Quantity {
            value: 2.5,
            dimension: Dimension::Count
        }
        .as_count(),
        Err(UnitError::NotAnInteger { .. })
    ));
    assert!(Value::ratio(0.5).is_ok());
    assert!(matches!(
        Value::ratio(f64::NAN),
        Err(Error::Invalid("finite numeric value"))
    ));
    assert_eq!(Value::ratio(0.25).unwrap().as_ratio().unwrap(), 0.25);
}

#[test]
fn l3_004_a_bare_number_is_not_a_length() {
    assert_eq!(
        Value::Number(1.2).as_length(),
        Err(UnitError::MissingUnit {
            expected: Dimension::Length
        })
    );
    assert!(matches!(
        Value::Number(1.2).as_angle(),
        Err(UnitError::MissingUnit {
            expected: Dimension::Angle
        })
    ));
    assert!(matches!(
        Value::count(1).as_length(),
        Err(UnitError::WrongDimension {
            expected: Dimension::Length,
            found: Dimension::Count
        })
    ));
}

/* ------------------------------ L3-005 ------------------------------ */

fn length_domain(step: Option<f64>) -> OptionDomain {
    OptionDomain::Number {
        dimension: Dimension::Length,
        min: Some(1.),
        min_inclusive: true,
        max: Some(10.),
        step,
    }
}

#[test]
fn l3_005_a_number_domain_admits_only_a_matching_typed_quantity() {
    let limits = Limits::default();
    let domain = length_domain(None);
    let ok = Value::length(LengthMm::try_new(5.).unwrap());
    domain.check(&ok, &limits).unwrap();
    // A bare number is refused: the schema requires a dimension.
    assert_eq!(
        domain.check(&Value::Number(5.), &limits),
        Err(Error::Invalid("missing unit"))
    );
    // A quantity of another dimension is refused.
    assert_eq!(
        domain.check(&Value::count(5), &limits),
        Err(Error::Invalid("numeric dimension"))
    );
    for bad in [0.5, 10.5] {
        assert!(
            domain
                .check(&Value::length(LengthMm::try_new(bad).unwrap()), &limits)
                .is_err(),
            "{bad}"
        );
    }
    // A non-finite quantity is refused by the domain even though the typed wrapper
    // would already have rejected it when constructed.
    assert!(
        domain
            .check(
                &Value::Quantity {
                    value: f64::NAN,
                    dimension: Dimension::Length
                },
                &limits
            )
            .is_err()
    );
}

#[test]
fn l3_005_a_step_grid_is_enforced() {
    let limits = Limits::default();
    let domain = length_domain(Some(0.5));
    domain
        .check(&Value::length(LengthMm::try_new(5.5).unwrap()), &limits)
        .unwrap();
    assert_eq!(
        domain.check(&Value::length(LengthMm::try_new(5.25).unwrap()), &limits),
        Err(Error::Invalid("numeric option step"))
    );
}

#[test]
fn l3_005_a_malformed_domain_is_refused_before_any_value() {
    for domain in [
        OptionDomain::Number {
            dimension: Dimension::Length,
            min: Some(10.),
            max: Some(1.),
            min_inclusive: true,
            step: None,
        },
        OptionDomain::Number {
            dimension: Dimension::Length,
            min: Some(f64::NAN),
            max: None,
            min_inclusive: true,
            step: None,
        },
        OptionDomain::Number {
            dimension: Dimension::Length,
            min: None,
            max: None,
            min_inclusive: true,
            step: Some(0.),
        },
        OptionDomain::Choice(vec![]),
        OptionDomain::Choice(vec!["a".into(), "a".into()]),
        OptionDomain::Text { max_bytes: 0 },
    ] {
        assert!(domain.validate().is_err(), "{domain:?}");
    }
    assert!(length_domain(Some(0.5)).validate().is_ok());
}

/* ------------------------------ L3-006 ------------------------------ */

#[test]
fn l3_006_wire_forms_are_unit_tagged_and_never_interchangeable() {
    let length = LengthMm::parse("2.5", Unit::Millimeter).unwrap();
    assert_eq!(serde_json::to_string(&length).unwrap(), r#"{"mm":2.5}"#);
    assert_eq!(
        serde_json::from_str::<LengthMm>(r#"{"mm":2.5}"#).unwrap(),
        length
    );
    assert!(
        serde_json::from_str::<LengthMm>(r#"{"deg":2.5}"#).is_err(),
        "a degrees payload is not a length"
    );
    assert!(
        serde_json::from_str::<LengthMm>(r#"2.5"#).is_err(),
        "a bare number is not a millimetre"
    );
    let angle = AngleDeg::try_new(45.).unwrap();
    assert_eq!(serde_json::to_string(&angle).unwrap(), r#"{"deg":45.0}"#);
    assert!(
        serde_json::from_str::<AngleDeg>(r#"{"rad":1.0}"#).is_err(),
        "radians cannot be read back as degrees"
    );
    let radians = angle.to_radians();
    assert_eq!(
        serde_json::to_string(&radians).unwrap(),
        format!(r#"{{"rad":{}}}"#, radians.rad())
    );
    assert!(serde_json::from_str::<AngleRad>(r#"{"deg":45.0}"#).is_err());
}

#[test]
fn l3_006_a_stored_quantity_keeps_its_dimension_through_json() {
    let value = Value::length(LengthMm::parse("1.234", Unit::Millimeter).unwrap());
    let text = serde_json::to_string(&value).unwrap();
    let reopened: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(reopened.as_length().unwrap().mm(), 1.234);
    // A stored bare number decoded from JSON does not satisfy a length domain.
    let bare: Value = serde_json::from_str(r#"{"Number":1.234}"#).unwrap();
    assert_eq!(
        length_domain(None).check(&bare, &Limits::default()),
        Err(Error::Invalid("missing unit"))
    );
    // Bit-exact length round trip (float_roundtrip is enabled workspace-wide).
    let irrational = LengthMm::try_new(12f64.hypot(6.)).unwrap();
    let text = serde_json::to_string(&irrational).unwrap();
    let back: LengthMm = serde_json::from_str(&text).unwrap();
    assert_eq!(back.mm().to_bits(), irrational.mm().to_bits());
}
