//! Declarative schema of supported consumers. Defaults are existing OM9 choices;
//! native adapters seed input-dependent values before exposing the session.
use crate::command_options::*;
use std::collections::BTreeMap;
pub fn schema(family: u32) -> Option<(Vec<OptionSpec>, BTreeMap<String, OptionValue>)> {
    if matches!(family, 14 | 15) {
        let model = crate::solid::Session::new(if family == 14 {
            crate::solid::Kind::Box
        } else {
            crate::solid::Kind::Sphere
        })
        .command_options(1, 1.)
        .ok()?;
        return Some((model.schema.clone(), model.values().clone()));
    }
    let mut s = Vec::new();
    let mut values = BTreeMap::new();
    let mut add = |spec: OptionSpec, value: Option<OptionValue>| {
        if let Some(value) = value {
            values.insert(spec.key.clone(), value);
        }
        s.push(spec);
    };
    let choices = |names: &[&str]| names.iter().map(|n| Choice::new(n)).collect();
    match family {
        1..=3 => {
            if family == 2 {
                add(
                    OptionSpec::toggle("BothSides").alias("B"),
                    Some(OptionValue::Toggle(false)),
                );
            } else {
                add(
                    OptionSpec::toggle("PersistentClose").alias("P"),
                    Some(OptionValue::Toggle(false)),
                );
            }
            if family == 1 {
                add(
                    OptionSpec::choice(
                        "Mode",
                        vec![
                            Choice::new("Line"),
                            Choice::new("Arc").disabled("Arc mode is not implemented"),
                        ],
                    )
                    .alias("M"),
                    Some(OptionValue::Choice("Line".into())),
                );
            }
            if family != 3 {
                add(
                    OptionSpec::number("Length", 1e-7, 1e9, "mm", false).alias("L"),
                    None,
                );
            }
            if family == 3 {
                add(
                    OptionSpec::number("Degree", 1., 11., "", true),
                    Some(OptionValue::Number(3.)),
                );
                add(
                    OptionSpec::choice("Knots", choices(&["Uniform", "Chord", "SqrtChrd"]))
                        .alias("K"),
                    Some(OptionValue::Choice("Uniform".into())),
                );
                add(OptionSpec::action("Sharp"), None);
            }
            if family != 2 {
                add(OptionSpec::action("Close").alias("C"), None);
            }
            add(OptionSpec::action("Undo").alias("U"), None);
        }
        4 => {
            add(
                OptionSpec::choice(
                    "Mode",
                    choices(&["Corners", "3Point", "Vertical", "Center"]),
                ),
                Some(OptionValue::Choice("Corners".into())),
            );
            add(OptionSpec::number("Length", 1e-7, 1e9, "mm", false), None);
            add(OptionSpec::number("Width", -1e9, 1e9, "mm", false), None);
            add(OptionSpec::action("Undo").alias("U"), None);
        }
        5 => {
            add(
                OptionSpec::choice(
                    "Mode",
                    choices(&[
                        "Center",
                        "2Point",
                        "3Point",
                        "Vertical",
                        "Tangent",
                        "AroundCurve",
                        "FitPoints",
                    ]),
                ),
                Some(OptionValue::Choice("Center".into())),
            );
            for key in [
                "Radius",
                "Diameter",
                "Circumference",
                "Area",
                "Orientation",
                "Point",
                "Curve",
                "OnCurve",
                "Undo",
            ] {
                add(
                    if key == "Undo" {
                        OptionSpec::action(key)
                    } else {
                        OptionSpec::native_input(key)
                    },
                    None,
                );
            }
            add(
                OptionSpec::toggle("Deformable").alias("Approx"),
                Some(OptionValue::Toggle(false)),
            );
            add(
                OptionSpec::toggle("History"),
                Some(OptionValue::Toggle(false)),
            );
            add(
                OptionSpec::number("PointCount", 4., 256., "", true),
                Some(OptionValue::Number(8.)),
            );
            add(
                OptionSpec::number("Degree", 1., 11., "", true),
                Some(OptionValue::Number(3.)),
            );
        }
        6 => {
            add(
                OptionSpec::choice(
                    "Mode",
                    choices(&[
                        "Center",
                        "Corner",
                        "Diameter",
                        "Vertical",
                        "FromFoci",
                        "AroundCurve",
                    ]),
                ),
                Some(OptionValue::Choice("Center".into())),
            );
            add(OptionSpec::action("Undo").alias("U"), None);
            add(OptionSpec::native_input("Curve"), None);
            add(OptionSpec::native_input("OnCurve"), None);
            add(
                OptionSpec::toggle("MarkFoci"),
                Some(OptionValue::Toggle(false)),
            );
            add(
                OptionSpec::toggle("Deformable").alias("Approx"),
                Some(OptionValue::Toggle(false)),
            );
            add(
                OptionSpec::number("PointCount", 4., 256., "", true),
                Some(OptionValue::Number(8.)),
            );
            add(
                OptionSpec::number("Degree", 1., 11., "", true),
                Some(OptionValue::Number(3.)),
            );
        }
        7 => {
            add(
                OptionSpec::number("PointCount", 2., 256., "", true),
                Some(OptionValue::Number(16.)),
            );
            add(
                OptionSpec::number("Degree", 1., 11., "", true),
                Some(OptionValue::Number(3.)),
            );
            add(
                OptionSpec::toggle("DeleteInput"),
                Some(OptionValue::Toggle(false)),
            );
            add(OptionSpec::action("Preview"), None);
        }
        11..=13 => {
            if family == 13 {
                add(
                    OptionSpec::choice(
                        "Style",
                        choices(&[
                            "Normal",
                            "Straight Sections",
                            "Loose",
                            "Tight",
                            "Uniform",
                            "Developable",
                        ]),
                    ),
                    Some(OptionValue::Choice("Normal".into())),
                );
            }
            if family == 11 {
                add(
                    OptionSpec::toggle("Frenet"),
                    Some(OptionValue::Toggle(false)),
                );
            }
            if family == 12 {
                add(
                    OptionSpec::toggle("MaintainHeight"),
                    Some(OptionValue::Toggle(false)),
                );
                for key in ["ContinuityA", "ContinuityB"] {
                    add(
                        OptionSpec::choice(key, choices(&["G0", "G1", "G2"])),
                        Some(OptionValue::Choice("G0".into())),
                    );
                }
            }
            if family == 13 {
                for key in ["MatchStart", "MatchEnd"] {
                    add(OptionSpec::toggle(key), Some(OptionValue::Toggle(false)));
                }
            }
            add(
                OptionSpec::toggle("Closed"),
                Some(OptionValue::Toggle(false)),
            );
            add(
                OptionSpec::choice(
                    "SectionMode",
                    vec![
                        Choice::new("Do Not Simplify"),
                        Choice::new("Rebuild").child("PointCount"),
                        Choice::new("Refit").child("Tolerance"),
                    ],
                ),
                Some(OptionValue::Choice("Do Not Simplify".into())),
            );
            add(
                OptionSpec::number("PointCount", 2., 256., "", true).parent("SectionMode"),
                Some(OptionValue::Number(16.)),
            );
            add(
                OptionSpec::number("Tolerance", 1e-7, 1e6, "mm", false)
                    .parent("SectionMode")
                    .alias("RefitTolerance"),
                Some(OptionValue::Number(0.01)),
            );
            add(
                OptionSpec::toggle("History"),
                Some(OptionValue::Toggle(false)),
            );
            add(
                OptionSpec::toggle("DynamicPreview"),
                Some(OptionValue::Toggle(true)),
            );
            add(OptionSpec::action("Preview"), None);
            add(OptionSpec::action("Automatic"), None);
            add(OptionSpec::action("Natural"), None);
        }
        _ => return None,
    }
    // These are declared command shortcuts, not inferred from rendered text.
    // Choice shortcuts live inside their parent phase and cannot capture a
    // coordinate/dimension while the parent is closed.
    let shortcuts: &[(&str, &str)] = match family {
        1 => &[
            ("PersistentClose", "P"),
            ("Mode", "M"),
            ("Length", "L"),
            ("Close", "C"),
            ("Undo", "U"),
        ],
        2 => &[("BothSides", "B"), ("Length", "L"), ("Undo", "U")],
        3 => &[
            ("PersistentClose", "P"),
            ("Degree", "D"),
            ("Knots", "K"),
            ("Sharp", "S"),
            ("Close", "C"),
            ("Undo", "U"),
        ],
        4 => &[
            ("Mode", "M"),
            ("Length", "L"),
            ("Width", "W"),
            ("Undo", "U"),
        ],
        5 => &[
            ("Mode", "M"),
            ("Radius", "R"),
            ("Diameter", "D"),
            ("Circumference", "F"),
            ("Area", "A"),
            ("Orientation", "O"),
            ("Point", "P"),
            ("Curve", "C"),
            ("OnCurve", "N"),
            ("Undo", "U"),
            ("Deformable", "B"),
            ("History", "H"),
            ("PointCount", "T"),
            ("Degree", "E"),
        ],
        6 => &[
            ("Mode", "M"),
            ("Undo", "U"),
            ("Curve", "C"),
            ("OnCurve", "N"),
            ("MarkFoci", "F"),
            ("Deformable", "B"),
            ("PointCount", "P"),
            ("Degree", "D"),
        ],
        7 => &[
            ("PointCount", "P"),
            ("Degree", "D"),
            ("DeleteInput", "I"),
            ("Preview", "V"),
        ],
        11 => &[
            ("Frenet", "F"),
            ("Closed", "C"),
            ("SectionMode", "S"),
            ("PointCount", "P"),
            ("Tolerance", "T"),
            ("History", "H"),
            ("DynamicPreview", "D"),
            ("Preview", "V"),
            ("Automatic", "U"),
            ("Natural", "N"),
        ],
        12 => &[
            ("MaintainHeight", "M"),
            ("ContinuityA", "A"),
            ("ContinuityB", "B"),
            ("Closed", "C"),
            ("SectionMode", "S"),
            ("PointCount", "P"),
            ("Tolerance", "T"),
            ("History", "H"),
            ("DynamicPreview", "D"),
            ("Preview", "V"),
            ("Automatic", "U"),
            ("Natural", "N"),
        ],
        13 => &[
            ("Style", "S"),
            ("MatchStart", "A"),
            ("MatchEnd", "E"),
            ("Closed", "C"),
            ("SectionMode", "O"),
            ("PointCount", "P"),
            ("Tolerance", "T"),
            ("History", "H"),
            ("DynamicPreview", "D"),
            ("Preview", "V"),
            ("Automatic", "U"),
            ("Natural", "N"),
        ],
        _ => &[],
    };
    let choice_shortcuts: &[(&str, &str)] = &[
        ("Line", "L"),
        ("Arc", "A"),
        ("Uniform", "U"),
        ("Chord", "C"),
        ("SqrtChrd", "S"),
        ("Corners", "C"),
        ("Center", "E"),
        ("3Point", "3"),
        ("Vertical", "V"),
        ("2Point", "2"),
        ("Tangent", "T"),
        ("AroundCurve", "A"),
        ("FitPoints", "F"),
        ("Corner", "C"),
        ("Diameter", "D"),
        ("FromFoci", "F"),
        ("Normal", "N"),
        ("Straight Sections", "S"),
        ("Loose", "L"),
        ("Tight", "T"),
        ("Developable", "D"),
        ("Do Not Simplify", "D"),
        ("Rebuild", "B"),
        ("Refit", "F"),
        ("G0", "0"),
        ("G1", "1"),
        ("G2", "2"),
    ];
    for spec in &mut s {
        if let Some((_, alias)) = shortcuts.iter().find(|(key, _)| *key == spec.key) {
            if !spec.aliases.iter().any(|a| a.eq_ignore_ascii_case(alias)) {
                spec.aliases.push((*alias).into());
            }
        }
        for choice in &mut spec.choices {
            if let Some((_, alias)) = choice_shortcuts
                .iter()
                .find(|(value, _)| *value == choice.value)
            {
                choice
                    .aliases
                    .push(if choice.value == "Center" && !matches!(family, 4 | 6) {
                        "C".into()
                    } else {
                        (*alias).into()
                    });
            }
        }
    }
    Some((s, values))
}
pub fn feature_id(family: u32) -> &'static str {
    match family {
        1 => "OM9-CURVE-001",
        2 => "OM9-CURVE-002",
        3 => "OM9-CURVE-003",
        4 => "OM9-CURVE-004",
        5 => "OM9-CURVE-005",
        6 => "OM9-CURVE-006",
        7 => "OM9-CURVE-009",
        11 => "OM9-SURFACE-001",
        12 => "OM9-SURFACE-003",
        13 => "OM9-SURFACE-009",
        14 => "OM9-SOLID-012",
        15 => "OM9-SOLID-014",
        _ => "",
    }
}
/// Existing solver state is authoritative, including legacy mode shortcuts and
/// accepted-input Undo. The common engine does not choose geometry defaults.
pub fn curve_state(
    session: &crate::curve::CurveSession,
) -> Option<(&'static str, BTreeMap<String, OptionValue>)> {
    use crate::native_curve_capability::{Options, Stage};
    let request = session.capability_request(Stage::Capture)?;
    let id = request.options.feature_id();
    let mut values = BTreeMap::new();
    let mut add = |key: &str, value: OptionValue| {
        values.insert(key.into(), value);
    };
    match request.options {
        Options::Line { both_sides, length } => {
            add("BothSides", OptionValue::Toggle(both_sides));
            if let Some(n) = length {
                add("Length", OptionValue::Number(n));
            }
        }
        Options::Polyline { close, length } => {
            add("PersistentClose", OptionValue::Toggle(close));
            add("Mode", OptionValue::Choice("Line".into()));
            if let Some(n) = length {
                add("Length", OptionValue::Number(n));
            }
        }
        Options::InterpCrv {
            degree,
            knots,
            close,
        } => {
            add("PersistentClose", OptionValue::Toggle(close));
            add("Degree", OptionValue::Number(degree as f64));
            add(
                "Knots",
                OptionValue::Choice(
                    match knots {
                        crate::spline::Knots::Uniform => "Uniform",
                        crate::spline::Knots::Chord => "Chord",
                        crate::spline::Knots::SqrtChord => "SqrtChrd",
                    }
                    .into(),
                ),
            );
        }
        Options::Rectangle(r) => {
            add(
                "Mode",
                OptionValue::Choice(
                    match r.mode {
                        crate::rectangle::Mode::Corners => "Corners",
                        crate::rectangle::Mode::Center => "Center",
                        crate::rectangle::Mode::ThreePoint => "3Point",
                        crate::rectangle::Mode::Vertical => "Vertical",
                    }
                    .into(),
                ),
            );
            if let Some(n) = r.length {
                add("Length", OptionValue::Number(n));
            }
            if let Some(n) = r.width {
                add("Width", OptionValue::Number(n));
            }
        }
        Options::Circle(c) => {
            use crate::circle::Mode::*;
            add(
                "Mode",
                OptionValue::Choice(
                    match c.mode {
                        Center => "Center",
                        TwoPoint => "2Point",
                        ThreePoint => "3Point",
                        Vertical => "Vertical",
                        Tangent => "Tangent",
                        AroundCurve => "AroundCurve",
                        FitPoints => "FitPoints",
                    }
                    .into(),
                ),
            );
            add("Deformable", OptionValue::Toggle(c.deformable));
            add("Degree", OptionValue::Number(c.degree as f64));
            add("PointCount", OptionValue::Number(c.point_count as f64));
            add("History", OptionValue::Toggle(c.history));
        }
        Options::Ellipse(e) => {
            use crate::ellipse::Mode::*;
            add(
                "Mode",
                OptionValue::Choice(
                    match e.mode {
                        Center => "Center",
                        Corner => "Corner",
                        Diameter => "Diameter",
                        Vertical => "Vertical",
                        FromFoci => "FromFoci",
                        AroundCurve => "AroundCurve",
                    }
                    .into(),
                ),
            );
            add("Deformable", OptionValue::Toggle(e.deformable));
            add("Degree", OptionValue::Number(e.degree as f64));
            add("PointCount", OptionValue::Number(e.point_count as f64));
            add("MarkFoci", OptionValue::Toggle(e.mark_foci));
        }
    }
    Some((id, values))
}
pub fn rules(family: u32) -> Vec<Constraint> {
    match family {
        5..=7 => vec![Constraint::GreaterThan {
            higher: "PointCount".into(),
            lower: "Degree".into(),
            message: "PointCount must be greater than Degree".into(),
        }],
        13 => vec![Constraint::ForbiddenPair {
            first: "Style".into(),
            first_value: OptionValue::Choice("Developable".into()),
            second: "Closed".into(),
            second_value: OptionValue::Toggle(true),
            message: "Developable does not support Closed loft".into(),
        }],
        _ => Vec::new(),
    }
}
