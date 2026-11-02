// SPDX-License-Identifier: LGPL-2.1-or-later
//! Core contract tests for gem instance edits and reports (`OM9-L2-04-13`).

use om9_core::{
    Error,
    gem::{
        GemCut, GemId, GemRecord, GemShape, GemSpec,
    },
    gem_edits::{
        DIAMOND_DENSITY_G_CM3, GEM_STYLE_FILE_SCHEMA, GemEdit, GemStyle, GemStyleStore,
        MatchMask, REPORT_TOLERANCE_PERCENT, gem_map, gem_report, gem_weight, match_attributes,
        match_edit, material_density, report_line, weight_formula,
    },
    identity::ObjectId,
};

fn round(diameter: f64) -> GemSpec {
    GemSpec::round(diameter, diameter * 0.65).expect("round spec")
}

fn gem(spec: GemSpec) -> GemRecord {
    GemRecord::new(spec)
}

#[test]
fn an_empty_edit_is_a_valid_no_op_that_keeps_the_identity() {
    let base = gem(round(6.0));
    let edit = GemEdit::default();
    assert!(edit.is_empty());
    assert!(!edit.changes_geometry());
    let edited = edit.record(&base).expect("no-op record");
    assert_eq!(edited.gem_id, base.gem_id);
    assert_eq!(edited.spec, base.spec);
}

#[test]
fn a_size_edit_keeps_the_crown_to_pavilion_proportion() {
    let base = gem(round(6.0));
    let edit = GemEdit {
        length_mm: Some(8.0),
        width_mm: Some(8.0),
        depth_mm: Some(base.spec.depth_mm() * 1.2),
        ..GemEdit::default()
    };
    assert!(edit.changes_geometry());
    let edited = edit.record(&base).expect("size edit");
    assert_eq!(edited.spec.length_mm, 8.0);
    assert_eq!(edited.spec.width_mm, 8.0);
    assert!(edited.spec.depth_mm() > base.spec.depth_mm());
    assert_eq!(edited.gem_id, base.gem_id);
    assert_eq!(edited.spec.shape, base.spec.shape);
}

#[test]
fn a_round_stone_collapses_two_diameters_and_refuses_a_cut_shape_mismatch() {
    let base = gem(round(6.0));
    let round_mismatch = GemEdit {
        length_mm: Some(8.0),
        width_mm: Some(6.0),
        ..GemEdit::default()
    };
    let collapsed = round_mismatch.spec(&base.spec).expect("round rule");
    assert_eq!(
        (collapsed.length_mm, collapsed.width_mm),
        (6.0, 6.0),
        "a round stone keeps one diameter, it never becomes an oval"
    );
    let shape_mismatch = GemEdit {
        cut: Some(GemCut::Cabochon),
        shape: Some(GemShape::Heart),
        ..GemEdit::default()
    };
    assert_eq!(
        shape_mismatch.spec(&base.spec).expect_err("cut shape"),
        Error::Invalid("gem shape for cut")
    );
}

#[test]
fn a_cut_edit_keeps_a_compatible_shape_and_rebuilds_the_record() {
    let base = gem(round(6.0));
    let edit = GemEdit {
        cut: Some(GemCut::Simple),
        ..GemEdit::default()
    };
    let edited = edit.record(&base).expect("cut edit");
    assert_eq!(edited.spec.cut, GemCut::Simple);
    assert_eq!(edited.spec.shape, GemShape::Round);
    assert_eq!(edited.gem_id, base.gem_id);
}

#[test]
fn match_attributes_copies_only_the_masked_groups() {
    let source = gem(round(6.0));
    let mut target = gem(round(4.0));
    target.material = "ruby".to_owned();
    target.color = [190, 30, 45, 255];
    let target_id = target.gem_id;
    let edit = match_edit(&source, MatchMask::GEOMETRY);
    assert!(edit.material.is_none());
    let matched = match_attributes(&source, &[target.clone()], MatchMask::GEOMETRY)
        .expect("geometry match");
    assert_eq!(matched[0].spec.length_mm, 6.0);
    assert_eq!(matched[0].spec.width_mm, 6.0);
    assert_eq!(matched[0].material, "ruby", "finish is not in the mask");
    assert_eq!(matched[0].gem_id, target_id);
    let finish = match_attributes(&source, &[target], MatchMask::FINISH).expect("finish match");
    assert_eq!(finish[0].material, source.material);
    assert_eq!(finish[0].spec.length_mm, 4.0, "geometry is not in the mask");
    assert_eq!(finish[0].gem_id, target_id);
}

#[test]
fn match_attributes_refuses_an_empty_mask() {
    let source = gem(round(6.0));
    assert_eq!(
        match_attributes(&source, &[gem(round(4.0))], MatchMask::NONE).expect_err("mask"),
        Error::Invalid("gem match mask")
    );
    assert_eq!(
        MatchMask::parse("nonsense").expect_err("unknown mask"),
        Error::Invalid("gem match mask")
    );
    assert_eq!(MatchMask::parse("geometry").expect("parse"), MatchMask::GEOMETRY);
}

#[test]
fn a_style_roundtrips_the_attributes_but_not_the_gem_identity() {
    let base = gem(round(6.0));
    let style = GemStyle::from_record("Round 6", &base);
    style.validate().expect("valid style");
    let json = style.to_json().expect("style json");
    let back = GemStyle::from_json(&json).expect("style back");
    assert_eq!(back, style);
    let mut target = gem(round(4.0));
    let target_id = target.gem_id;
    target.material = "ruby".to_owned();
    let applied = style.apply(&target).expect("apply style");
    assert_eq!(applied.spec.length_mm, 6.0);
    assert_eq!(applied.material, base.material);
    assert_eq!(applied.gem_id, target_id, "a style never renumbers the stone");
}

#[test]
fn a_style_store_replaces_by_name_and_saves_atomically() {
    let mut store = GemStyleStore::new();
    let first = GemStyle::from_record("Round 6", &gem(round(6.0)));
    let second = GemStyle::from_record("Round 8", &gem(round(8.0)));
    assert!(!store.insert(first.clone()).expect("insert"));
    assert!(store.insert(first).expect("replace"));
    assert_eq!(store.len(), 1);
    store.insert(second).expect("second");
    assert_eq!(store.len(), 2);
    assert_eq!(store.get("round 6").expect("lookup").name, "Round 6");

    // Keep the round-trip inside the cargo target tree: some sandboxes deny writes to the OS
    // temp directory even though reads work, and a test must not fail for that reason.
    let directory = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("om9-gem-style-{}", std::process::id()));
    let path = directory.join("gem-styles.json");
    store.save_to(&path).expect("save store");
    let loaded = GemStyleStore::load_or_empty(&path).expect("load store");
    assert_eq!(loaded, store);
    assert_eq!(loaded.schema, GEM_STYLE_FILE_SCHEMA);
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_missing_style_file_reads_as_the_empty_store_but_a_bad_name_is_refused() {
    let directory = std::env::temp_dir().join(format!("om9-gem-missing-{}", std::process::id()));
    let path = directory.join("absent.json");
    let store = GemStyleStore::load_or_empty(&path).expect("empty store");
    assert!(store.is_empty());
    let mut style = GemStyle::from_record("bad/name", &gem(round(6.0)));
    assert_eq!(
        style.validate().expect_err("bad name"),
        Error::Invalid("style name")
    );
    style.name = "Good".to_owned();
    style.schema = "om9.other".to_owned();
    assert_eq!(
        style.validate().expect_err("bad schema"),
        Error::Invalid("gem style schema")
    );
}

#[test]
fn the_reporter_groups_by_shape_cut_and_size_and_totals_the_mass() {
    let records = vec![
        gem(round(6.0)),
        gem(round(6.0)),
        GemRecord::new(GemSpec::oval(8.0, 6.0, 4.0).expect("oval")),
    ];
    let report = gem_report(&records, None).expect("report");
    assert_eq!(report.total_count, 3);
    assert_eq!(report.rows.len(), 2);
    let round_row = report
        .rows
        .iter()
        .find(|row| row.shape == GemShape::Round)
        .expect("round row");
    assert_eq!(round_row.count, 2);
    assert_eq!(round_row.material, "diamond");
    assert!((round_row.density_g_cm3 - DIAMOND_DENSITY_G_CM3).abs() < 1.0e-12);
    assert!(round_row.carat > 0.0);
    assert!(report.total_grams > 0.0);
    assert!((report.total_carat * 0.2 - report.total_grams).abs() < 1.0e-9);
    assert!(report_line(round_row).contains("2 stone(s)"));
    assert!(report_line(round_row).contains("g/cm³"));
}

#[test]
fn the_reporter_uses_the_material_density_and_cross_checks_the_trade_formula() {
    let mut diamond = gem(round(6.0));
    let ruby = {
        let mut record = gem(round(6.0));
        record.material = "ruby".to_owned();
        record
    };
    let ruby_record = ruby.clone();
    let report = gem_report(&[diamond.clone(), ruby_record], None).expect("report");
    assert_eq!(report.rows.len(), 2, "different materials never merge");
    let ruby_row = report
        .rows
        .iter()
        .find(|row| row.material == "ruby")
        .expect("ruby row");
    assert!((ruby_row.density_g_cm3 - 4.00).abs() < 1.0e-12);
    let diamond_row = report
        .rows
        .iter()
        .find(|row| row.material == "diamond")
        .expect("diamond row");
    assert!(
        ruby_row.carat > diamond_row.carat * 1.1,
        "corundum is heavier than diamond for the same girdle"
    );
    assert_eq!(report.assumed_density, 0, "both materials are in the table");
    assert_eq!(report.outside_tolerance, 0, "the mesh and the formula agree");
    let formula = weight_formula(&ruby.spec).expect("round trade formula");
    assert!(formula.carat > 0.0);
    assert_eq!(formula.name, "round brilliant");
    let delta = ruby_row.delta_percent.expect("cross-check");
    assert!(delta.abs() < REPORT_TOLERANCE_PERCENT, "delta {delta}");
    diamond.material = "unobtainium".to_owned();
    let unknown = gem_report(&[diamond], None).expect("report");
    assert_eq!(unknown.assumed_density, 1, "an unknown material is flagged");
    assert!((unknown.rows[0].density_g_cm3 - DIAMOND_DENSITY_G_CM3).abs() < 1.0e-12);
    let overridden = gem_report(&[gem(round(6.0))], Some(4.00)).expect("report");
    assert!(overridden.density_override.is_some_and(|density| density == 4.00));
    assert!(weight_formula(&GemSpec::round(6.0, 4.0).expect("round")).is_some());
    let mut cabochon = GemSpec::round(6.0, 4.0).expect("round");
    cabochon.cut = GemCut::Cabochon;
    // A cabochon dome has no faceted trade formula; the reporter omits the cross-check, it
    // never applies the wrong formula.
    assert!(weight_formula(&cabochon).is_none());
}

#[test]
fn the_reporter_refuses_an_impossible_density() {
    let records = vec![gem(round(6.0))];
    for density in [0.0, -3.52, f64::NAN, 100.0] {
        assert_eq!(
            gem_report(&records, Some(density)).expect_err("density"),
            Error::Invalid("gem density")
        );
    }
    let weight = gem_weight(&records[0].spec, DIAMOND_DENSITY_G_CM3).expect("weight");
    assert!(weight.volume_mm3 > 0.0 && weight.carat > 0.0);
    assert!((weight.carat * 0.2 - weight.grams).abs() < 1.0e-12);
}

#[test]
fn a_density_override_scales_the_mass_but_not_the_formula_delta() {
    let records = vec![gem(round(6.0))];
    let material = gem_report(&records, None).expect("material basis");
    let overridden = gem_report(&records, Some(4.00)).expect("override basis");
    assert!(
        overridden.total_carat > material.total_carat * 1.1,
        "a denser basis must weigh more"
    );
    let material_delta = material.rows[0].delta_percent.expect("cross-check");
    let override_delta = overridden.rows[0].delta_percent.expect("cross-check");
    assert!(
        (material_delta - override_delta).abs() < 1.0e-9,
        "the trade cross-check is relative, so the density basis cancels"
    );
    assert_eq!(material_density("RUBY"), Some(4.00));
    assert_eq!(material_density("unobtainium"), None);
}

#[test]
fn the_map_numbers_the_scene_in_order_and_refuses_a_bad_point() {
    let spec = round(6.0);
    let entries = vec![
        (GemId(ObjectId::new()), [0.0, 0.0, 5.0], spec.clone()),
        (GemId(ObjectId::new()), [10.0, 0.0, 5.0], spec.clone()),
    ];
    let rows = gem_map(&entries).expect("map");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].number, 1);
    assert_eq!(rows[1].number, 2);
    assert_eq!(rows[1].point, [10.0, 0.0, 5.0]);
    let bad = vec![(GemId(ObjectId::new()), [f64::NAN, 0.0, 0.0], spec)];
    assert_eq!(
        gem_map(&bad).expect_err("bad point"),
        Error::Invalid("gem map point")
    );
}
