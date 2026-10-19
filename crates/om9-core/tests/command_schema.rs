// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-01-02 wave W6: typed command option schemas and completion.
//!
//! Oracles: a schema that cannot hold a value is refused before any user input, the fill step
//! merges defaults under the supplied values and refuses unknown or missing ones, and the
//! completion ranking is deterministic.
use om9_core::{
    Error,
    budget::Limits,
    command_schema::{CommandSchema, OptionSpec, complete, rank, value_candidates},
    request::{Dimension, OptionDomain, Options, Value},
    units::LengthMm,
};

fn limits() -> Limits {
    Limits::default()
}

fn length_domain(min: f64, max: f64) -> OptionDomain {
    OptionDomain::Number {
        dimension: Dimension::Length,
        min: Some(min),
        min_inclusive: true,
        max: Some(max),
        step: None,
    }
}

fn line_schema() -> CommandSchema {
    CommandSchema::new("Line")
        .with_alias("L")
        .with_option(OptionSpec::new(
            "Length",
            length_domain(0.01, 1_000.0),
            Some(Value::length(LengthMm::try_new(10.0).unwrap())),
            false,
        ))
        .with_option(OptionSpec::new(
            "BothSides",
            OptionDomain::Boolean,
            Some(Value::Boolean(false)),
            false,
        ))
        .with_option(OptionSpec::new(
            "Mode",
            OptionDomain::Choice(vec!["Straight".into(), "Chain".into(), "Smooth".into()]),
            None,
            true,
        ))
}

/* ------------------------- schema validation ------------------------- */

#[test]
fn a_schema_is_validated_before_any_value() {
    let schema = line_schema();
    schema.validate(&limits()).unwrap();
    // Duplicate option names are refused (case-insensitively).
    let duplicate = line_schema().with_option(OptionSpec::new(
        "length",
        OptionDomain::Boolean,
        None,
        false,
    ));
    assert_eq!(
        duplicate.validate(&limits()),
        Err(Error::Invalid("command option name"))
    );
    // A default outside its own domain can never be applied.
    let bad_default = CommandSchema::new("Bad").with_option(OptionSpec::new(
        "Length",
        length_domain(1.0, 2.0),
        Some(Value::length(LengthMm::try_new(5.0).unwrap())),
        false,
    ));
    assert_eq!(
        bad_default.validate(&limits()),
        Err(Error::Invalid("numeric option domain"))
    );
    // A required option may not also carry a default.
    let impossible = CommandSchema::new("Bad").with_option(OptionSpec::new(
        "Mode",
        OptionDomain::Boolean,
        Some(Value::Boolean(true)),
        true,
    ));
    assert_eq!(
        impossible.validate(&limits()),
        Err(Error::Invalid("command option default/required"))
    );
    // Duplicate aliases and an empty command name are refused.
    assert_eq!(
        CommandSchema::new("Line")
            .with_alias("L")
            .with_alias("l")
            .validate(&limits()),
        Err(Error::Invalid("command alias"))
    );
    assert_eq!(
        CommandSchema::new("").validate(&limits()),
        Err(Error::Invalid("command name"))
    );
}

/* ------------------------- apply ------------------------- */

#[test]
fn apply_fills_defaults_under_the_supplied_values() {
    let schema = line_schema();
    // Only the required option is supplied: defaults fill the rest.
    let mut supplied = Options::new();
    supplied.insert("Mode".into(), Value::Choice("Chain".into()));
    let options = schema.apply(supplied, &limits()).unwrap();
    assert_eq!(options.len(), 3);
    assert_eq!(
        options["Mode"],
        Value::Choice("Chain".into()),
        "the supplied value wins"
    );
    assert_eq!(options["BothSides"], Value::Boolean(false));
    let length = options["Length"].as_length().unwrap();
    assert!((length.mm() - 10.0).abs() < 1e-12);
}

#[test]
fn apply_normalizes_the_spelling_of_a_supplied_name() {
    let schema = line_schema();
    let mut supplied = Options::new();
    supplied.insert("mode".into(), Value::Choice("Smooth".into()));
    // The declared spelling is restored, so downstream code sees one name.
    let options = schema.apply(supplied, &limits()).unwrap();
    assert!(options.contains_key("Mode"));
    assert!(!options.contains_key("mode"));
    // The value itself is still checked against its domain.
    let mut supplied = Options::new();
    supplied.insert("Mode".into(), Value::Choice("Smoo".into()));
    assert_eq!(
        schema.apply(supplied, &limits()),
        Err(Error::Invalid("option value/type"))
    );
}

#[test]
fn apply_refuses_unknown_and_missing_options() {
    let schema = line_schema();
    let mut unknown = Options::new();
    unknown.insert("Nope".into(), Value::Boolean(true));
    assert_eq!(
        schema.apply(unknown, &limits()),
        Err(Error::Invalid("unknown option"))
    );
    // "Mode" is required and has no default.
    assert_eq!(
        schema.apply(Options::new(), &limits()),
        Err(Error::Missing("required option"))
    );
    // A typed value of the wrong dimension is refused by the domain, not silently coerced.
    let mut wrong = Options::new();
    wrong.insert("Mode".into(), Value::Choice("Chain".into()));
    wrong.insert("Length".into(), Value::Number(5.0));
    assert_eq!(
        schema.apply(wrong, &limits()),
        Err(Error::Invalid("missing unit"))
    );
}

/* ------------------------- completion ------------------------- */

#[test]
fn completion_ranks_exact_then_prefix_then_substring() {
    assert_eq!(rank("Line", "Line"), 0);
    assert_eq!(rank("Line", "Li"), 1);
    assert_eq!(rank("line", "Li"), 2);
    assert_eq!(rank("Polyline", "line"), 3);
    assert_eq!(rank("Circle", "Li"), u32::MAX);
    let candidates = ["Polyline", "Line", "line", "SmallLine"];
    assert_eq!(
        complete(candidates, "line", 10),
        vec!["line", "Line", "Polyline", "SmallLine"],
        "exact, then case-insensitive prefix, then substring, each by name"
    );
    // The limit is honoured and the order stays stable.
    assert_eq!(complete(candidates, "line", 2), vec!["line", "Line"]);
    // An empty input offers everything in name order.
    assert_eq!(
        complete(candidates, "", 10),
        vec!["Line", "Polyline", "SmallLine", "line"]
    );
}

#[test]
fn an_option_completion_lists_the_declared_names() {
    let schema = line_schema();
    assert_eq!(schema.complete("l", 10), vec!["Length"]);
    assert_eq!(
        schema.complete("", 10),
        vec!["BothSides", "Length", "Mode"],
        "declaration order is not the output order; names are sorted"
    );
    assert_eq!(schema.complete("zz", 10), Vec::<String>::new());
}

#[test]
fn a_choice_option_completes_its_declared_values() {
    let schema = line_schema();
    let mode = schema.option("Mode").unwrap();
    assert_eq!(
        value_candidates(mode, "s"),
        vec!["Straight".to_string(), "Smooth".to_string()]
    );
    assert_eq!(value_candidates(mode, "ch"), vec!["Chain".to_string()]);
    assert_eq!(value_candidates(mode, "zz"), Vec::<String>::new());
    // A non-choice option has no value candidates.
    let both = schema.option("BothSides").unwrap();
    assert_eq!(value_candidates(both, ""), Vec::<String>::new());
}
