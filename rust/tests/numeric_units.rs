use openmatrix9_rust::units::{UnitError, parse_angle, parse_length};

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2e-14 * expected.abs().max(1.),
        "actual={actual:?}, expected={expected:?}"
    );
}

#[test]
fn simple_and_mixed_fractions_use_the_existing_length_scale() {
    for (text, scale, expected) in [
        ("5/16", 1., 0.3125),
        ("1-3/4", 1., 1.75),
        ("-1-3/4", 25.4, -44.45),
        ("+5/16", 25.4, 7.9375),
        ("-5/16", 10., -3.125),
        ("0-1/2", 1., 0.5),
        ("3/2", 1., 1.5),
        ("1-3/4in", 1000., 44.45),
        ("5/16 mm", 25.4, 0.3125),
    ] {
        close(parse_length(text, scale).unwrap(), expected);
    }
}

#[test]
fn documented_millimeters_suffix_overrides_model_scale() {
    close(parse_length("1.235millimeters", 25.4).unwrap(), 1.235);
    close(parse_length("-5/16millimeters", 1000.).unwrap(), -0.3125);
}

#[test]
fn feet_and_inches_are_one_signed_physical_quantity() {
    for (text, expected) in [
        ("+16'5\"", 5003.8),
        ("1'2-3/4\"", 374.65),
        ("-1'2-3/4\"", -374.65),
        ("-0'5\"", -127.),
        ("0'11.5\"", 292.1),
        ("2'", 609.6),
        ("1/2'", 152.4),
        ("5/16\"", 7.9375),
    ] {
        close(parse_length(text, 1000.).unwrap(), expected);
    }
}

#[test]
fn fractional_components_are_bounded_exact_unsigned_integers() {
    close(
        parse_length("9007199254740991/9007199254740991", 1.).unwrap(),
        1.,
    );
    for text in [
        "1/0",
        "0/0",
        "1/-2",
        "1/+2",
        "1//2",
        "1/2/3",
        "1.5/2",
        "1/2.5",
        "1e2/3",
        "1-2/2",
        "1-3/2",
        "1--1/2",
        "1-+1/2",
        "1.5-1/2",
        "9007199254740992/1",
        "1/9007199254740992",
        "9007199254740992-1/2",
    ] {
        assert!(parse_length(text, 1.).is_err(), "accepted {text}");
    }
}

#[test]
fn ambiguous_imperial_forms_and_wrong_dimensions_are_rejected() {
    for text in [
        "1'12\"",
        "1'-2\"",
        "1'+2\"",
        "1.5'2\"",
        "1/2'2\"",
        "1'2",
        "1'2\"mm",
        "1''",
        "1\"2'",
        "1' 2\"",
        "1- 1/2",
        "1/ 2",
        "1,2",
        "1/2mm2",
        "1/2deg",
        "1.235millimeters2",
    ] {
        assert!(parse_length(text, 1.).is_err(), "accepted {text}");
    }
}

#[test]
fn fraction_conversion_preserves_overflow_and_underflow_rejection() {
    assert_eq!(parse_length("3/2", f64::MAX), Err(UnitError::Overflow));
    assert_eq!(
        parse_length("1/2", f64::from_bits(1)),
        Err(UnitError::Overflow)
    );
    assert_eq!(parse_length("1/2", 0.), Err(UnitError::InvalidContext));
    for text in ["NaN", "inf", "1e999", "NaN'2\"", "1'NaN\""] {
        assert!(parse_length(text, 1.).is_err(), "accepted {text}");
    }
    assert!(parse_length(&"1".repeat(129), 1.).is_err());
}

#[test]
fn plain_decimal_and_fractional_angles_are_degrees_returned_as_radians() {
    for (text, degrees) in [
        ("90", 90.),
        ("-90", -90.),
        ("+1.8e2", 180.),
        ("5/16", 0.3125),
        ("-1-3/4", -1.75),
        (" 360 ", 360.),
        ("1/2d", 0.5),
    ] {
        close(
            parse_angle(text).unwrap(),
            degrees * std::f64::consts::PI / 180.,
        );
    }
}

#[test]
fn dms_angles_apply_one_sign_to_the_complete_quantity() {
    let documented = (30. + 22. / 60. + 54.43 / 3600.) * std::f64::consts::PI / 180.;
    close(parse_angle("30d22'54.43\"").unwrap(), documented);
    close(parse_angle("-30d22'54.43\"").unwrap(), -documented);
    for (text, degrees) in [
        ("+30d", 30.),
        ("30d22'", 30. + 22. / 60.),
        ("30d54.43\"", 30. + 54.43 / 3600.),
        ("-0d30'", -0.5),
        ("0d0'0.5\"", 0.5 / 3600.),
    ] {
        close(
            parse_angle(text).unwrap(),
            degrees * std::f64::consts::PI / 180.,
        );
    }
}

#[test]
fn angle_grammar_rejects_length_units_and_ambiguous_dms() {
    for text in [
        "",
        "NaN",
        "inf",
        "1e999",
        "1/0",
        "1/2mm",
        "30mm",
        "30deg",
        "1rad",
        "30°",
        "30D22'",
        "30 d",
        "30d 22'",
        "30d22' 1\"",
        "30d60'",
        "30d0'60\"",
        "30d-1'",
        "30d+1'",
        "30d1'-1\"",
        "30d1'+1\"",
        "30.5d1'",
        "30d1.5'",
        "30d1/2'",
        "30d1e1'",
        "30d1'2/3\"",
        "30d1'2e1\"",
        "30d1",
        "30d1'2",
        "30d1'2\"x",
        "30dd",
        "30d1''",
        "30d1\"2'",
        "1'2\"",
        "1,5",
    ] {
        assert!(parse_angle(text).is_err(), "accepted {text}");
    }
}

#[test]
fn angles_have_explicit_magnitude_and_token_bounds_without_wrapping() {
    close(parse_angle("1000000000").unwrap(), 1e9_f64.to_radians());
    close(parse_angle("-1000000000d").unwrap(), -1e9_f64.to_radians());
    for text in ["1000000001", "-1000000001", "1000000000d1'", "1e300"] {
        assert_eq!(
            parse_angle(text),
            Err(UnitError::Overflow),
            "accepted {text}"
        );
    }
    assert!(parse_angle(&"1".repeat(129)).is_err());
    assert_eq!(parse_angle("5e-324"), Err(UnitError::Overflow));
}

#[test]
fn new_angle_and_imperial_forms_do_not_silently_underflow_to_zero() {
    for text in ["1e-999", "-1e-999", "1e-999d"] {
        assert_eq!(parse_angle(text), Err(UnitError::Overflow));
    }
    for text in ["1e-999'", "-1e-999\"", "0'1e-999\""] {
        assert_eq!(parse_length(text, 1.), Err(UnitError::Overflow));
    }
    // Preserve the existing decimal length parser's accepted input behavior.
    assert_eq!(parse_length("1e-999", 1.).unwrap(), 0.);
}

#[test]
fn zero_sign_and_real_subnormal_lengths_remain_representable() {
    for text in ["-0", "-0e-999", "-0/2", "-0-0/2", "-0d0'0\""] {
        assert_eq!(parse_angle(text).unwrap().to_bits(), (-0.0_f64).to_bits());
    }
    for text in ["-0'0\"", "-0/2\""] {
        assert_eq!(
            parse_length(text, 1.).unwrap().to_bits(),
            (-0.0_f64).to_bits()
        );
    }
    assert!(parse_length("5e-324\"", 1.).unwrap() > 0.);
    close(
        parse_angle("999999999d59'59\"").unwrap(),
        17_453_292.519_938_447,
    );
}

#[test]
fn dms_bound_checks_lower_components_before_floating_point_rounding() {
    for text in ["1000000000d0'0.000001\"", "-1000000000d0'0.000001\""] {
        assert_eq!(parse_angle(text), Err(UnitError::Overflow));
    }
}

#[test]
fn angle_bound_checks_fraction_and_decimal_before_rounding() {
    for text in [
        "1000000000-1/9007199254740991",
        "-1000000000-1/9007199254740991",
        "1000000000.00000001",
        "-1000000000.00000001d",
        "1.00000000000000001e9",
        "100000000000000001e-8",
    ] {
        assert_eq!(
            parse_angle(text),
            Err(UnitError::Overflow),
            "accepted {text}"
        );
    }
    for text in [
        "1000000000-0/3",
        "1.00000000000000000e9",
        "999999999.99999999",
        "100000000000000000e-8",
    ] {
        assert!(parse_angle(text).is_ok(), "rejected {text}");
    }
}
