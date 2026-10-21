// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-03-14-006 (Superselect By Jewelry Object Type),
//! driven by `implementation/fixtures/M01/OM9-L3-03-14-006/`.
use om9_app::session::Controller;
use om9_app::superselect::{SelectFilter, SkipReason};
use om9_core::budget::Limits;
use om9_core::geometry::{Curve, Geometry, Mesh};
use om9_core::model::{Document, Entity, Extension, GEM_EXTENSION_SLOT, slot_name_valid};
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-03-14-006")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn empty_mesh() -> Geometry {
    Geometry::Mesh(Mesh {
        vertices: Vec::new(),
        faces: Vec::new(),
        normals: Vec::new(),
        colors: Vec::new(),
    })
}

fn insert(document: &mut Document, entity: Entity) -> om9_core::identity::ObjectId {
    let id = entity.id;
    document.entities.insert(id, entity);
    id
}

/// A document with: one semantic gem (Mesh + om9.gem), one mesh that only looks
/// like a gem (same label, no extension), one valid line, one duplicate label.
fn document() -> (
    Document,
    om9_core::identity::ObjectId,
    om9_core::identity::ObjectId,
) {
    let mut document = Document::new("superselect");
    let mut gem = Entity::new("Gem", empty_mesh(), None);
    gem.extensions.insert(
        GEM_EXTENSION_SLOT.to_owned(),
        Extension::new(1, "{\"ref\":\"gem-blue-round\"}").expect("payload"),
    );
    let gem_id = insert(&mut document, gem);
    let lookalike = Entity::new("Gem", empty_mesh(), None);
    let lookalike_id = insert(&mut document, lookalike);
    insert(
        &mut document,
        Entity::new(
            "Line",
            Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [1., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        ),
    );
    insert(&mut document, Entity::new("Gem", empty_mesh(), None));
    (document, gem_id, lookalike_id)
}

fn controller_with(document: Document) -> Controller {
    Controller::new(document, Limits::default()).expect("controller")
}

#[test]
fn g1_to_g5_the_filters_use_typed_evidence_only() {
    let fixture_doc = fixture("positive.json");
    let (document, gem_id, lookalike_id) = document();
    let controller = controller_with(document);

    // G1: exactly the semantic gem matches.
    let gems = controller.superselect(SelectFilter::Gems);
    assert_eq!(
        gems.matched,
        vec![gem_id],
        "{}",
        fixture_doc["cases"][0]["id"]
    );
    assert!(!gems.matched.contains(&lookalike_id));

    // G2: the lookalike is skipped with the reason that says why.
    assert!(
        gems.skipped
            .iter()
            .any(|(id, reason)| *id == lookalike_id && *reason == SkipReason::NoGemExtension)
    );

    // G3: bad geometry is judged by the model, so a valid document matches none.
    let bad = controller.superselect(SelectFilter::BadGeometry);
    assert!(bad.matched.is_empty(), "nothing is invalid here");
    assert!(
        bad.skipped
            .iter()
            .all(|(_, reason)| *reason == SkipReason::GeometryValid)
    );

    // G4: the open filter invents nothing.
    let open = controller.superselect(SelectFilter::OpenGeometry);
    assert!(open.matched.is_empty());
    assert!(
        open.skipped
            .iter()
            .all(|(_, reason)| matches!(reason, SkipReason::NoOpenVerdict(_)))
    );

    // G5: duplicates are the repeated labels, and the first keeps its claim.
    let duplicates = controller.superselect(SelectFilter::Duplicate);
    assert_eq!(
        duplicates.matched.len(),
        2,
        "two of the three Gem-labelled objects repeat"
    );
}

#[test]
fn h1_to_h5_a_name_or_a_colour_is_not_a_gem() {
    let fixture_doc = fixture("negative.json");

    // H1/H2: the same label and no extension is not a gem (the lookalike above).
    let (first, _, lookalike_id) = document();
    let controller = controller_with(first);
    let gems = controller.superselect(SelectFilter::Gems);
    assert!(!gems.matched.contains(&lookalike_id));
    assert_eq!(fixture_doc["cases"][0]["expect"]["matched"], 0);

    // H3: the model refuses a bare word as an extension slot.
    let h3 = &fixture_doc["cases"][2];
    assert_eq!(
        slot_name_valid(h3["slot"].as_str().unwrap()),
        h3["expect"]["valid_slot"].as_bool().unwrap()
    );

    // H4: an empty payload cannot even be stored: the model refuses it when the
    // extension is created, so the filter can never see a payload-free gem.
    assert!(
        Extension::new(1, "  ").is_err(),
        "the model refuses a payload that is not a JSON object"
    );
    let mut valid_gem = Document::new("valid gem payload");
    let mut entity = Entity::new("Gem", empty_mesh(), None);
    entity.extensions.insert(
        GEM_EXTENSION_SLOT.to_owned(),
        Extension::new(1, "{\"ref\":\"gem-blue-round\"}").expect("payload"),
    );
    insert(&mut valid_gem, entity);
    let controller = controller_with(valid_gem);
    assert_eq!(controller.superselect(SelectFilter::Gems).matched.len(), 1);

    // H5: superselect never writes the document.
    let (fresh, _, _) = document();
    let mut controller = controller_with(fresh);
    let journal = controller.coordinator().undo_len();
    let entities = controller.document().entities.len();
    let count = controller.apply_superselect(SelectFilter::Gems);
    assert_eq!(count, 1, "the semantic gem is selected");
    assert_eq!(controller.coordinator().undo_len(), journal);
    assert_eq!(controller.document().entities.len(), entities);
    assert!(
        fixture_doc["cases"][4]["expect"]["journal_unchanged"]
            .as_bool()
            .unwrap()
    );
}
