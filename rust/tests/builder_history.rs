// SPDX-License-Identifier: LGPL-2.1-or-later
use openmatrix9_rust::builder_history::{affine_plan, compatible_shapes, validate_recipe};
#[test]
fn feature_identity_requires_exact_om9_format() {
    use openmatrix9_rust::builder_history::valid_feature_id;
    for id in ["OM9-SETTING-001", "OM9-CUTTER-001", "OM9-HISTORY-001"] {
        assert!(valid_feature_id(id));
    }
    for id in [
        "",
        "builder",
        "OM9-setting-001",
        "OM9-SETTING-000",
        "OM9-SETTING-1",
        "OM9-SETTING-001\n",
        "OM9--001",
    ] {
        assert!(!valid_feature_id(id));
    }
}
#[test]
fn match_budget_checks_aggregate_records_outputs_and_overflow() {
    use openmatrix9_rust::builder_history::valid_replay_budget;
    assert!(valid_replay_budget(4, 16, 2));
    assert!(valid_replay_budget(4096, 16384, 1));
    assert!(!valid_replay_budget(4097, 4097, 1));
    assert!(!valid_replay_budget(1, 16385, 1));
    assert!(!valid_replay_budget(usize::MAX, usize::MAX, 2));
    assert!(!valid_replay_budget(0, 0, 1));
    assert!(!valid_replay_budget(1, 1, 0));
    assert!(!valid_replay_budget(4, 3, 1));
    assert!(openmatrix9_rust::builder_history::om9_builder_replay_budget(4, 16, 2));
    assert!(
        !openmatrix9_rust::builder_history::om9_builder_replay_budget(usize::MAX, usize::MAX, 2)
    );
}
const RECIPE: &str = r#"{"schema":1,"evaluator":"om9.affine-template","version":1,"settings":{"height":2.3,"mode":"bezel","nested":[true,null]},"scale":[1,2,3],"offset":[4,5,6]}"#;
#[test]
fn retains_settings_and_maps_dimensions_without_solving_settings() {
    let r = validate_recipe(RECIPE).unwrap();
    assert_eq!(r.raw, RECIPE);
    let p = affine_plan(&r, [2., 4., 8.], [4., 4., 4.], true).unwrap();
    assert_eq!(p, [2., 2., 1.5, 4., 5., 6.]);
}
#[test]
fn rejects_unsupported_evaluators_versions_and_nonfinite_bounds() {
    for bad in [
        RECIPE.replace("template", "unknown"),
        RECIPE.replace("\"schema\":1", "\"schema\":2"),
        RECIPE.replace("2.3", "1e999"),
        RECIPE.replace("[1,2,3]", "[0,2,3]"),
        RECIPE.replace("\"height\":2.3", "\"height\":2.3,\"height\":3"),
    ] {
        assert!(validate_recipe(&bad).is_err(), "{bad}");
    }
    assert!(
        affine_plan(
            &validate_recipe(RECIPE).unwrap(),
            [0., 2., 3.],
            [2., 3., 4.],
            true
        )
        .is_err()
    );
}
#[test]
fn match_requires_explicit_same_shape_tags() {
    assert!(compatible_shapes("round", &["round", "round"]).is_ok());
    assert!(compatible_shapes("round", &["round", "oval"]).is_err());
    assert!(compatible_shapes("", &[""]).is_err());
    assert!(compatible_shapes(" ", &[" "]).is_err());
    assert!(compatible_shapes("round\n", &["round\n"]).is_err());
    assert!(compatible_shapes("round", &[]).is_err());
}
#[test]
fn rejects_json_limits_escaped_duplicate_keys_and_trailing_content() {
    assert!(validate_recipe(&RECIPE.replace("bezel", "\\u+041")).is_err());
    assert!(
        validate_recipe(&RECIPE.replace("\"height\":2.3", "\"height\":2.3,\"heig\\u0068t\":3"))
            .is_err()
    );
    assert!(validate_recipe(&(RECIPE.to_string() + " null")).is_err());
    assert!(validate_recipe(&RECIPE.replacen('{', "{\u{b}", 1)).is_err());
    assert!(validate_recipe(&RECIPE.replacen('{', "{\u{c}", 1)).is_err());
    assert!(validate_recipe(&RECIPE.replace("2.3", "01")).is_err());
    assert!(
        validate_recipe(&RECIPE.replace("2.3", &format!("{}0{}", "[".repeat(34), "]".repeat(34))))
            .is_err()
    );
    assert!(validate_recipe(&RECIPE.replace("bezel", "\\uD83D\\uDC8E")).is_ok());
}
