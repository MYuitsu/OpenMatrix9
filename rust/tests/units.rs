use openmatrix9_rust::units::{Context, Dimension, UnitChangeMode, parse_length};

fn context() -> Context {
    Context {
        revision: 1,
        model_mm: 1.0,
        page_mm: 1.0,
        absolute_mm: 0.001,
        relative_ratio: 0.01,
        angular_rad: 0.01,
    }
}

#[test]
fn physical_and_declared_unit_changes_are_distinct() {
    let old = context();
    let mut next = old.clone();
    next.revision = 2;
    next.model_mm = 25.4;
    assert_eq!(
        old.change_scale(&next, UnitChangeMode::PreservePhysicalSize)
            .unwrap(),
        1.0
    );
    let factor = old
        .change_scale(&next, UnitChangeMode::PreserveDeclaredNumbers)
        .unwrap();
    assert!((25.4 * factor - 645.16).abs() < 1e-10);
    assert!(
        old.change_scale(&old, UnitChangeMode::PreservePhysicalSize)
            .is_err()
    );
}

#[test]
fn dimensions_scale_by_power_without_scaling_angles_or_parameters() {
    let c = Context {
        model_mm: 25.4,
        ..context()
    };
    for (dim, expected) in [
        (Dimension::Length, 25.4),
        (Dimension::Area, 645.16),
        (Dimension::Volume, 16387.064),
        (Dimension::Curvature, 1.0 / 25.4),
        (Dimension::GaussianCurvature, 1.0 / 645.16),
        (Dimension::Unitless, 1.0),
        (Dimension::AngleRadians, 1.0),
    ] {
        assert!((c.to_native(1.0, dim).unwrap() - expected).abs() < 1e-10);
    }
    assert!(c.to_native(f64::MAX, Dimension::Volume).is_err());
}

#[test]
fn context_roundtrip_rejects_bad_values_and_future_schemas() {
    let c = context();
    assert_eq!(Context::decode(&c.encode().unwrap()).unwrap(), c);
    for v in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            Context {
                model_mm: v,
                ..c.clone()
            }
            .validate()
            .is_err()
        );
        assert!(
            Context {
                absolute_mm: v,
                ..c.clone()
            }
            .validate()
            .is_err()
        );
    }
    assert!(
        Context {
            angular_rad: 4.0,
            ..c.clone()
        }
        .validate()
        .is_err()
    );
    assert!(Context::decode("2|1|1|1|0.001|0.01|0.01").is_err());
    assert!(Context::decode("1|1|1|1|0.001|0.01|0.01|extra").is_err());
}

#[test]
fn finite_extreme_contexts_cannot_serialize_to_unreadable_records() {
    let c = Context {
        model_mm: 1e308,
        page_mm: 1e-308,
        absolute_mm: 1e-308,
        relative_ratio: 1e308,
        ..context()
    };
    c.validate().unwrap();
    let raw = c.encode().unwrap();
    assert_eq!(Context::decode(&raw).unwrap(), c);
}

#[test]
fn length_parser_resolves_explicit_units_and_rejects_wrong_dimensions() {
    assert_eq!(parse_length("1", 25.4).unwrap(), 25.4);
    assert_eq!(parse_length("1 mm", 25.4).unwrap(), 1.0);
    assert_eq!(parse_length("1in", 1.0).unwrap(), 25.4);
    assert_eq!(parse_length("2cm", 1.0).unwrap(), 20.0);
    assert_eq!(parse_length("1e-3 m", 1.0).unwrap(), 1.0);
    for text in [
        "NaN", "inf", "1e999", "1 2", "1,2", "1 mm2", "2deg", "3kg", "",
    ] {
        assert!(parse_length(text, 1.0).is_err(), "accepted {text}");
    }
    assert!(parse_length("1", 0.0).is_err());
}

#[test]
fn curve_session_latches_explicit_input_units() {
    use openmatrix9_rust::curve::{CurveSession, Effect};
    let mut line = CurveSession::default();
    line.start("Line").unwrap();
    line.set_input_scale(25.4).unwrap();
    line.input("0,0,0").unwrap();
    assert!(line.set_input_scale(1.0).is_err());
    assert_eq!(line.input("1,0,0").unwrap(), Effect::Commit);
    assert_eq!(line.output()[1], [25.4, 0.0, 0.0]);
    line.start("Line").unwrap();
    line.input("0,0,0").unwrap();
    line.input("1,0,0").unwrap();
    assert_eq!(line.output()[1], [1.0, 0.0, 0.0]);
    let mut circle = CurveSession::default();
    circle.start("Circle").unwrap();
    circle.set_input_scale(25.4).unwrap();
    circle.input("0,0,0").unwrap();
    circle.input("1").unwrap();
    assert!((circle.circle().unwrap().radius - 25.4).abs() < 1e-10);
}
