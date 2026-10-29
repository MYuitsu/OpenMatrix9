// SPDX-License-Identifier: LGPL-2.1-or-later
//! Application contract tests for gem instance edits and reports (`OM9-L2-04-13`).
//!
//! The core oracle lives in `crates/om9-core/tests/gem_edits.rs`; this file proves the session
//! wiring of the nine Matrix commands: one-transaction Control edits, Match Attributes, the
//! Save/Load style store, Gem Update geometry regeneration, the read-only Map and Reporter, and
//! the attribute history that proves ids and the placement link survive an edit.

use om9_app::session::{
    Controller, InvocationOrigin,
    gem_edits::{CONTROL, HISTORY, LOAD_STYLE, MAP, MATCH, REPORTER, SAVE_STYLE, UPDATE},
    gem_placement::ON_CURVE,
};
use om9_core::{
    budget::Limits,
    gem::{GemRecord, GemShape, GemSpec, build_mesh, gem_record},
    gem_placement::PlacementLink,
    geometry::{Curve, Geometry},
    identity::ObjectId,
    model::{
        Document, Entity, GEM_EXTENSION_SLOT, GEM_PLACEMENT_EXTENSION_SLOT,
    },
};

fn controller_with(entities: Vec<Entity>) -> Controller {
    let mut document = Document::new("gem-edits");
    for entity in entities {
        document.entities.insert(entity.id, entity);
    }
    Controller::new(document, Limits::default()).expect("controller")
}

fn round(diameter: f64) -> GemSpec {
    GemSpec::round(diameter, diameter * 0.65).expect("round spec")
}

fn gem_entity(spec: GemSpec, label: &str) -> Entity {
    gem_entity_with_material(spec, label, "diamond")
}

fn gem_entity_with_material(spec: GemSpec, label: &str, material: &str) -> Entity {
    let mesh = build_mesh(&spec).expect("gem mesh");
    let mut record = GemRecord::new(spec);
    record.material = material.to_owned();
    let mut entity = Entity::new(label, Geometry::Mesh(mesh.mesh), None);
    entity.extensions.insert(
        GEM_EXTENSION_SLOT.to_owned(),
        record.as_extension().expect("gem extension"),
    );
    entity
}

fn gems(sizes: &[f64]) -> Vec<Entity> {
    sizes
        .iter()
        .enumerate()
        .map(|(index, size)| gem_entity(round(*size), &format!("Gem {}", index + 1)))
        .collect()
}

fn ids(controller: &Controller) -> Vec<ObjectId> {
    controller.document().entities.keys().copied().collect()
}

fn record(controller: &Controller, id: ObjectId) -> GemRecord {
    gem_record(&controller.document().entities[&id]).expect("a gem record")
}

fn mesh_span_x(controller: &Controller, id: ObjectId) -> f64 {
    match &controller.document().entities[&id].geometry {
        Geometry::Mesh(mesh) => {
            let min = mesh
                .vertices
                .iter()
                .map(|point| point[0])
                .fold(f64::INFINITY, f64::min);
            let max = mesh
                .vertices
                .iter()
                .map(|point| point[0])
                .fold(f64::NEG_INFINITY, f64::max);
            max - min
        }
        other => panic!("expected a mesh, got {other:?}"),
    }
}

fn style_file(name: &str) -> std::path::PathBuf {
    // The cargo target tree is writable in every sandbox this suite runs under; the OS temp
    // directory is not, and a style round-trip test must not depend on it.
    std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("om9-gem-edit-{}-{name}", std::process::id()))
        .join("gem-styles.json")
}

#[test]
fn control_edits_every_selected_stone_in_one_transaction_and_undo_restores_it() {
    let mut controller = controller_with(gems(&[6.0, 4.0]));
    let ids = ids(&controller);
    let sizes: Vec<f64> = ids
        .iter()
        .map(|id| record(&controller, *id).spec.length_mm)
        .collect();
    let identities: Vec<_> = ids
        .iter()
        .map(|id| record(&controller, *id).gem_id)
        .collect();
    controller.select_objects(&ids).expect("select");
    let revision = controller.document().revision;
    controller
        .dispatch(CONTROL, InvocationOrigin::Cmd)
        .expect("gem control");
    assert!(controller.prompt().contains("Gem Control"));
    controller.submit("Size=8").expect("size");
    assert_eq!(controller.gem_edits_plans().len(), 2);
    controller.submit("").expect("apply");
    assert_eq!(controller.document().revision, revision + 1, "one transaction");
    for (id, identity) in ids.iter().zip(&identities) {
        let edited = record(&controller, *id);
        assert_eq!(edited.spec.length_mm, 8.0);
        assert_eq!(edited.gem_id, *identity, "the edit never renumbers a stone");
    }
    controller.undo().expect("undo");
    for (id, size) in ids.iter().zip(&sizes) {
        assert_eq!(record(&controller, *id).spec.length_mm, *size);
    }
    controller.redo().expect("redo");
    for id in &ids {
        assert_eq!(record(&controller, *id).spec.length_mm, 8.0);
    }
}

#[test]
fn control_refuses_a_non_gem_selection_and_an_unknown_option() {
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let line_id = line.id;
    let mut controller = controller_with(vec![line]);
    controller.select_objects(&[line_id]).expect("select");
    assert_eq!(
        controller
            .dispatch(CONTROL, InvocationOrigin::Cmd)
            .expect_err("not a gem"),
        om9_core::Error::Invalid("gem edit needs gems")
    );

    let mut controller = controller_with(gems(&[6.0]));
    controller.select_objects(&ids(&controller)).expect("select");
    controller
        .dispatch(CONTROL, InvocationOrigin::Cmd)
        .expect("control");
    assert_eq!(
        controller.submit("Mask=all").expect_err("mask is not a control option"),
        om9_core::Error::Invalid("gem edit option")
    );
    assert!(controller.is_running(), "a refusal keeps the tool open");
    controller.submit("Cancel").expect("cancel");
    assert!(!controller.is_running());
}

#[test]
fn match_attributes_copies_geometry_from_the_first_selected_stone() {
    let mut controller = controller_with(gems(&[6.0, 4.0, 5.0]));
    let order = ids(&controller);
    let source = *order
        .iter()
        .find(|id| record(&controller, **id).spec.length_mm == 6.0)
        .expect("the 6 mm source");
    let mut selection = vec![source];
    selection.extend(order.iter().copied().filter(|id| *id != source));
    let original_sizes: Vec<f64> = selection
        .iter()
        .map(|id| record(&controller, *id).spec.length_mm)
        .collect();
    controller.select_objects(&selection).expect("select");
    controller
        .dispatch(MATCH, InvocationOrigin::Cmd)
        .expect("match");
    controller.submit("Mask=geometry").expect("mask");
    controller.submit("").expect("apply");
    assert_eq!(record(&controller, source).spec.length_mm, 6.0);
    for id in &selection[1..] {
        assert_eq!(record(&controller, *id).spec.length_mm, 6.0);
    }
    controller.undo().expect("undo");
    for (id, size) in selection.iter().zip(&original_sizes) {
        assert_eq!(record(&controller, *id).spec.length_mm, *size);
    }
}

#[test]
fn save_and_load_style_roundtrip_through_the_style_file() {
    let path = style_file("roundtrip");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
    let mut controller = controller_with(gems(&[6.0, 4.0]));
    let order = ids(&controller);
    let six = *order
        .iter()
        .find(|id| record(&controller, **id).spec.length_mm == 6.0)
        .expect("the 6 mm source");
    let four = *order
        .iter()
        .find(|id| record(&controller, **id).spec.length_mm == 4.0)
        .expect("the 4 mm target");
    controller.select_objects(&[six]).expect("select source");
    controller
        .dispatch(SAVE_STYLE, InvocationOrigin::Cmd)
        .expect("save style");
    controller.submit("Name=Round Six").expect("name");
    controller.submit(&format!("File={}", path.display())).expect("file");
    controller.submit("").expect("apply");
    assert!(path.is_file(), "the style file is written");
    let text = std::fs::read_to_string(&path).expect("style text");
    assert!(text.contains("om9.gem_style_file.v1"), "versioned store");
    assert!(text.contains("Round Six"));

    controller.select_objects(&[four]).expect("select target");
    controller
        .dispatch(LOAD_STYLE, InvocationOrigin::Cmd)
        .expect("load style");
    controller.submit("Name=Round Six").expect("name");
    controller.submit(&format!("File={}", path.display())).expect("file");
    controller.submit("").expect("apply");
    assert_eq!(record(&controller, four).spec.length_mm, 6.0);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn save_style_refuses_a_name_the_shared_store_would_reject() {
    let mut controller = controller_with(gems(&[6.0]));
    controller.select_objects(&ids(&controller)).expect("select");
    controller
        .dispatch(SAVE_STYLE, InvocationOrigin::Cmd)
        .expect("save style");
    assert_eq!(
        controller.submit("Name=bad/name").expect_err("bad name"),
        om9_core::Error::Invalid("style name")
    );
    assert!(controller.is_running());
    controller.submit("Cancel").expect("cancel");
}

#[test]
fn update_rebuilds_the_geometry_from_the_stored_record() {
    let spec = round(6.0);
    let stored = GemRecord::new(spec.clone());
    // A stale body: the mesh is the 4 mm stone while the record says 6 mm.
    let stale = build_mesh(&round(4.0)).expect("stale mesh");
    let mut entity = Entity::new("stale gem", Geometry::Mesh(stale.mesh), None);
    entity.extensions.insert(
        GEM_EXTENSION_SLOT.to_owned(),
        stored.as_extension().expect("record"),
    );
    let id = entity.id;
    let mut controller = controller_with(vec![entity]);
    assert!((mesh_span_x(&controller, id) - 4.0).abs() < 0.2);
    controller.select_objects(&[id]).expect("select");
    controller
        .dispatch(UPDATE, InvocationOrigin::Cmd)
        .expect("update");
    controller.submit("").expect("apply");
    assert!(
        (mesh_span_x(&controller, id) - 6.0).abs() < 0.2,
        "Gem Update rebuilds the 6 mm body"
    );
    assert_eq!(record(&controller, id).spec.length_mm, 6.0);
    assert_eq!(record(&controller, id).gem_id, stored.gem_id);
}

#[test]
fn the_map_numbers_the_selected_stones_without_touching_the_document() {
    let mut controller = controller_with(gems(&[6.0, 4.0, 5.0]));
    let order = ids(&controller);
    controller.select_objects(&order).expect("select");
    let revision = controller.document().revision;
    controller
        .dispatch(MAP, InvocationOrigin::Cmd)
        .expect("map");
    let rows = controller.gem_map_rows();
    assert_eq!(rows.len(), 3, "the overlay receives one marker per stone");
    assert_eq!(
        rows.iter().map(|row| row.number).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    let text = controller.gem_edits_report_text().expect("report");
    assert!(text.contains("#1 ·"), "first row numbered: {text}");
    assert!(text.contains("#3 ·"), "last row numbered: {text}");
    assert_eq!(controller.document().revision, revision, "a report is read-only");
    controller.submit("").expect("close");
    assert!(!controller.is_running());
}

#[test]
fn the_reporter_aggregates_count_and_calculated_mass() {
    let mut controller = controller_with(gems(&[6.0, 6.0, 4.0]));
    controller.select_objects(&ids(&controller)).expect("select");
    let revision = controller.document().revision;
    controller
        .dispatch(REPORTER, InvocationOrigin::Cmd)
        .expect("reporter");
    let text = controller.gem_edits_report_text().expect("report");
    assert!(text.contains("2 stone(s)"), "6 mm pair aggregated: {text}");
    assert!(text.contains("TOTAL · 3 stone(s)"));
    assert!(text.contains("ct"));
    assert!(text.contains("worst Δ"), "the trade cross-check is printed: {text}");
    assert!(
        text.contains("NOT A CERTIFICATE"),
        "the report never calls itself a certificate: {text}"
    );
    let report = controller.gem_report_data().expect("typed report");
    assert_eq!(report.total_count, 3);
    assert!(report.total_carat_formula.is_some());
    assert_eq!(report.outside_tolerance, 0);
    assert_eq!(report.assumed_density, 0);
    assert_eq!(controller.document().revision, revision);
    assert_eq!(
        controller.submit("Density=0").expect_err("density"),
        om9_core::Error::Invalid("gem density")
    );
    assert!(controller.is_running(), "a refusal keeps the report open");
    controller.submit("Cancel").expect("cancel");
}

#[test]
fn the_reporter_uses_the_material_density_instead_of_one_diamond_basis() {
    let mut controller = controller_with(vec![
        gem_entity_with_material(round(6.0), "Diamond", "diamond"),
        gem_entity_with_material(round(6.0), "Ruby", "ruby"),
    ]);
    controller.select_objects(&ids(&controller)).expect("select");
    controller
        .dispatch(REPORTER, InvocationOrigin::Cmd)
        .expect("reporter");
    let report = controller.gem_report_data().expect("typed report");
    assert_eq!(report.rows.len(), 2, "materials never merge into one row");
    let ruby = report
        .rows
        .iter()
        .find(|row| row.material == "ruby")
        .expect("ruby row");
    let diamond = report
        .rows
        .iter()
        .find(|row| row.material == "diamond")
        .expect("diamond row");
    assert!((ruby.density_g_cm3 - 4.00).abs() < 1.0e-12);
    assert!(ruby.carat > diamond.carat * 1.1, "corundum outweighs diamond");
    assert_eq!(report.assumed_density, 0);
    let text = controller.gem_edits_report_text().expect("report");
    assert!(text.contains("ruby"));
    assert!(text.contains("4.00 g/cm³"));
    controller.submit("Cancel").expect("cancel");
}

#[test]
fn an_edit_keeps_the_placement_link_and_the_attribute_history_shows_it() {
    let line = Entity::new(
        "rail",
        Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0],
            end: [30.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let line_id = line.id;
    let mut controller = controller_with(vec![line]);
    controller.select_objects(&[line_id]).expect("select line");
    controller
        .dispatch(ON_CURVE, InvocationOrigin::Cmd)
        .expect("gem on curve");
    controller.submit("Start=2").expect("start");
    controller.submit("End=28").expect("end");
    controller.submit("Spacing=6").expect("spacing");
    controller.submit("").expect("apply");
    let gem = ids(&controller)
        .into_iter()
        .find(|id| gem_record(&controller.document().entities[id]).is_some())
        .expect("a placed gem");
    let before = PlacementLink::from_extension(
        controller.document().entities[&gem]
            .extensions
            .get(GEM_PLACEMENT_EXTENSION_SLOT)
            .expect("a placement link"),
    )
    .expect("readable link");
    let gem_id = record(&controller, gem).gem_id;

    controller.select_objects(&[gem]).expect("select gem");
    controller
        .dispatch(CONTROL, InvocationOrigin::Cmd)
        .expect("control");
    controller.submit("Size=4").expect("size");
    controller.submit("").expect("apply");

    let after = PlacementLink::from_extension(
        controller.document().entities[&gem]
            .extensions
            .get(GEM_PLACEMENT_EXTENSION_SLOT)
            .expect("the link survives the edit"),
    )
    .expect("readable link");
    assert_eq!(after.source.tag(), before.source.tag());
    assert_eq!(after.source.object(), before.source.object());
    assert_eq!(record(&controller, gem).gem_id, gem_id);
    assert_eq!(record(&controller, gem).spec.length_mm, 4.0);

    controller
        .dispatch(HISTORY, InvocationOrigin::Cmd)
        .expect("history");
    let text = controller.gem_edits_report_text().expect("history report");
    assert!(text.contains("link curve"), "history names the live link: {text}");
    controller.submit("").expect("close");
}

#[test]
fn the_palette_icon_and_the_typed_command_reach_the_same_service() {
    let mut controller = controller_with(gems(&[6.0]));
    controller.select_objects(&ids(&controller)).expect("select");
    controller
        .dispatch("gvGemControl", InvocationOrigin::Cmd)
        .expect("typed gem control");
    assert!(controller.gem_edits_session().is_some());
    controller.submit("Shape=Oval").expect("shape");
    controller.submit("").expect("apply");
    assert_eq!(record(&controller, ids(&controller)[0]).spec.shape, GemShape::Oval);
}
