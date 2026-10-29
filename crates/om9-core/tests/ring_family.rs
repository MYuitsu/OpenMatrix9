// SPDX-License-Identifier: LGPL-2.1-or-later
//! Jewelry ring builder family (`OM9-L2-04-03`): Eternity/Signet/Raised/Award builders,
//! their presets, the associative resize and the final B-Rep feasibility gate.
use std::collections::BTreeMap;

use om9_core::{
    Error,
    associative::RebuildRequest,
    budget::Limits,
    builder::{
        BuilderKind, BuilderSession, field_values, producer_name, register_reference_producers,
    },
    builder_style::{BuilderStyle, StyleFamily, StyleLibrary},
    geometry::{Geometry, Mesh},
    identity::ObjectId,
    model::Document,
    producers::ProducerRegistry,
    request::Value,
    ring_builder::{self, MIN_WALL_MM, RingBuilderKind, RingParams},
    transaction::*,
};

fn limits() -> Limits {
    Limits::default()
}

fn registry() -> ProducerRegistry {
    let mut registry = ProducerRegistry::new();
    register_reference_producers(&mut registry).unwrap();
    registry
}

fn empty_document() -> Document {
    Document::new("ring family")
}

fn preview_mesh(session: &BuilderSession) -> &Mesh {
    let Some(preview) = session.preview() else {
        panic!("a refreshed ring session has a preview");
    };
    let Geometry::Mesh(mesh) = &preview.geometry else {
        panic!("a ring builder preview is a mesh solid");
    };
    mesh
}

/// Commit the session plan in one transaction, exactly like the panel's Apply.
fn apply(session: &BuilderSession, coordinator: &mut TransactionCoordinator) -> ObjectId {
    let document_id = *coordinator.world().documents.keys().next().unwrap();
    let document = &coordinator.world().documents[&document_id];
    let expected_revision = document.revision;
    let plan = session.plan(document).unwrap();
    let prepared = coordinator
        .prepare(TransactionRequest::new(
            session.kind().command(),
            vec![
                Mutation::Insert {
                    document: document_id,
                    entity: plan.entity.clone(),
                },
                Mutation::History {
                    document: document_id,
                    expected_revision,
                    recipe: Box::new(plan.recipe.clone()),
                    published: vec![(plan.entity.id, plan.published.clone())],
                },
            ],
        ))
        .unwrap();
    coordinator.commit(prepared).unwrap();
    plan.entity.id
}

fn coordinator(document: Document) -> TransactionCoordinator {
    TransactionCoordinator::new(World::new(vec![document], &limits()).unwrap(), limits()).unwrap()
}

#[test]
fn an_eternity_band_commits_a_closed_solid_and_keeps_stones_as_frames() {
    let document = empty_document();
    let registry = registry();
    let mut session = BuilderSession::new(BuilderKind::EternityBand, &document, &[]).unwrap();
    assert_eq!(session.title(), "Eternity Band");
    session.refresh(&document, &registry).unwrap();
    let mesh = preview_mesh(&session).clone();
    let report = ring_builder::inspect_solid(&mesh).unwrap();
    assert!(report.closed);
    assert!(report.volume_mm3 > 0.0);
    // The stone ladder is published as frames, not as gem bodies.
    let frames = ring_builder::stone_frames(&RingBuilderKind::Eternity.defaults()).unwrap();
    assert_eq!(frames.len(), 12);
    let plan = session.plan(&document).unwrap();
    assert_eq!(plan.recipe.command, "EternityBand");
    assert_eq!(
        plan.entity.geometry.kind(),
        om9_core::geometry::GeometryKind::Mesh
    );

    let mut coordinator = coordinator(document);
    let output = apply(&session, &mut coordinator);
    let document_id = *coordinator.world().documents.keys().next().unwrap();
    let applied = &coordinator.world().documents[&document_id];
    // One entity: the band. No gem solid was fabricated.
    assert_eq!(applied.entities.len(), 1);
    let Geometry::Mesh(applied_mesh) = &applied.entities[&output].geometry else {
        panic!("the applied entity is the mesh solid");
    };
    assert!(ring_builder::inspect_solid(applied_mesh).unwrap().closed);
    // Undo removes the band and its recipe in one step.
    coordinator.undo().unwrap();
    let undone = &coordinator.world().documents[&document_id];
    assert!(undone.entities.is_empty());
    assert!(undone.history.recipes.is_empty());
}

#[test]
fn the_four_ring_modes_expose_their_own_parameters() {
    let document = empty_document();
    for kind in [
        BuilderKind::EternityBand,
        BuilderKind::SignetRing,
        BuilderKind::RaisedBand,
        BuilderKind::AwardRing,
    ] {
        let session = BuilderSession::new(kind, &document, &[]).unwrap();
        let names: Vec<&str> = session.fields().iter().map(|field| field.name).collect();
        assert!(names.contains(&"diameter_mm"));
        assert!(names.contains(&"width_mm"));
        assert!(names.contains(&"thickness_mm"));
        match kind {
            BuilderKind::EternityBand => {
                assert!(names.contains(&"stone_count"));
                assert!(names.contains(&"stone_diameter_mm"));
            }
            BuilderKind::RaisedBand => {
                assert!(names.contains(&"raised_height_mm"));
                assert!(names.contains(&"raised_width_mm"));
            }
            _ => {
                assert!(names.contains(&"top_width_mm"));
                assert!(names.contains(&"top_length_mm"));
                assert!(names.contains(&"shoulder_mm"));
            }
        }
        assert_eq!(
            producer_name(kind),
            format!("{}Producer", kind.title().replace(' ', ""))
        );
        assert_eq!(BuilderKind::from_tag(kind.tag()), Some(kind));
    }
}

#[test]
fn a_signet_ring_resize_updates_the_same_feature_through_the_recorded_recipe() {
    let document = empty_document();
    let registry = registry();
    let mut session = BuilderSession::new(BuilderKind::SignetRing, &document, &[]).unwrap();
    session.refresh(&document, &registry).unwrap();
    let mut coordinator = coordinator(document);
    let output = apply(&session, &mut coordinator);
    let document_id = *coordinator.world().documents.keys().next().unwrap();
    let (first_volume, recipe_id) = {
        let applied = &coordinator.world().documents[&document_id];
        let Geometry::Mesh(mesh) = &applied.entities[&output].geometry else {
            panic!("mesh");
        };
        (
            ring_builder::inspect_solid(mesh).unwrap().volume_mm3,
            *applied.history.recipes.keys().next().unwrap(),
        )
    };
    // Resize by reopening the recorded feature and editing its diameter: the same object
    // and the same recipe are updated, not a second ring.
    {
        let applied = &coordinator.world().documents[&document_id];
        let mut reopened = BuilderSession::new(BuilderKind::SignetRing, applied, &[]).unwrap();
        reopened.adopt_history(output, recipe_id);
        reopened
            .set_field(
                "diameter_mm",
                Value::length(om9_core::units::LengthMm::try_new(19.0).unwrap()),
            )
            .unwrap();
        reopened.refresh(applied, &registry).unwrap();
        let mut plan = reopened.plan(applied).unwrap();
        let revision = applied.entities[&output].geometry_revision.wrapping_add(1);
        plan.entity.geometry_revision = revision;
        plan.published.geometry_revision = revision;
        let expected_revision = applied.revision;
        let prepared = coordinator
            .prepare(TransactionRequest::new(
                "Signet Ring",
                vec![
                    Mutation::Geometry {
                        source: applied.stamp(output).unwrap(),
                        geometry: plan.entity.geometry.clone(),
                    },
                    Mutation::HistoryUpdate {
                        document: document_id,
                        expected_revision,
                        recipes: vec![Box::new(plan.recipe.clone())],
                        published: vec![(output, plan.published.clone())],
                    },
                ],
            ))
            .unwrap();
        coordinator.commit(prepared).unwrap();
    }
    let resized_volume = {
        let applied = &coordinator.world().documents[&document_id];
        // The owner id did not change: the feature was updated in place.
        let Geometry::Mesh(mesh) = &applied.entities[&output].geometry else {
            panic!("mesh");
        };
        ring_builder::inspect_solid(mesh).unwrap().volume_mm3
    };
    assert!(resized_volume > first_volume);
    // Undo/Redo rolls the associative update back and forward.
    coordinator.undo().unwrap();
    let undone_volume = {
        let applied = &coordinator.world().documents[&document_id];
        let Geometry::Mesh(mesh) = &applied.entities[&output].geometry else {
            panic!("mesh");
        };
        ring_builder::inspect_solid(mesh).unwrap().volume_mm3
    };
    assert!((undone_volume - first_volume).abs() < 1.0);
    coordinator.redo().unwrap();
    let redone_volume = {
        let applied = &coordinator.world().documents[&document_id];
        let Geometry::Mesh(mesh) = &applied.entities[&output].geometry else {
            panic!("mesh");
        };
        ring_builder::inspect_solid(mesh).unwrap().volume_mm3
    };
    assert!((redone_volume - resized_volume).abs() < 1.0);
}

#[test]
fn a_signet_preset_roundtrip_recreates_the_parameters_and_the_solid() {
    let document = empty_document();
    let registry = registry();
    let mut session = BuilderSession::new(BuilderKind::SignetRing, &document, &[]).unwrap();
    session.refresh(&document, &registry).unwrap();
    let baseline_volume = ring_builder::inspect_solid(preview_mesh(&session))
        .unwrap()
        .volume_mm3;
    // Serialize the live signet as a preset, keep it in the user list and on disk.
    let mut style = BuilderStyle::snapshot(&session, "Signet 8x10");
    style.family = vec![StyleFamily::Signet];
    style.builders = vec![BuilderKind::SignetRing.tag().to_string()];
    let mut library = StyleLibrary::builtin();
    library.save_user(style.clone()).unwrap();
    let path = std::env::temp_dir().join("om9_ring_family_preset.json");
    let _ = std::fs::remove_file(&path);
    library.save_to(&path).unwrap();
    let mut loaded = StyleLibrary::builtin();
    assert_eq!(loaded.load_from(&path).unwrap(), 1);
    let loaded_style = loaded.find("Signet 8x10").unwrap().clone();
    assert_eq!(loaded_style.parameters, style.parameters);
    // Loading the preset into a fresh session recreates the same body.
    let mut fresh = BuilderSession::new(BuilderKind::SignetRing, &document, &[]).unwrap();
    fresh.refresh(&document, &registry).unwrap();
    let applied = loaded_style.apply(&mut fresh).unwrap();
    assert!(applied.applied.contains(&"top_width_mm"));
    assert!(applied.applied.contains(&"top_length_mm"));
    fresh.refresh(&document, &registry).unwrap();
    let preset_volume = ring_builder::inspect_solid(preview_mesh(&fresh))
        .unwrap()
        .volume_mm3;
    assert!((preset_volume - baseline_volume).abs() < 1.0);
    // An unknown schema version is refused instead of being guessed.
    let mut corrupt = std::fs::read_to_string(&path).unwrap();
    corrupt = corrupt.replace("om9.builder_style.v1", "om9.builder_style.v9");
    std::fs::write(&path, corrupt).unwrap();
    let mut broken = StyleLibrary::builtin();
    assert_eq!(
        broken.load_from(&path).err().unwrap(),
        Error::Invalid("style schema")
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn each_ring_mode_paints_handles_that_edit_their_own_fields() {
    for kind in [
        BuilderKind::EternityBand,
        BuilderKind::SignetRing,
        BuilderKind::RaisedBand,
        BuilderKind::AwardRing,
    ] {
        let document = empty_document();
        let registry = registry();
        let mut session = BuilderSession::new(kind, &document, &[]).unwrap();
        session.refresh(&document, &registry).unwrap();
        let handles = session.preview().unwrap().handles.clone();
        let names: Vec<&str> = handles.iter().map(|handle| handle.name).collect();
        for required in ["diameter", "width", "wall"] {
            assert!(names.contains(&required), "{kind:?} must paint {required}");
        }
        // The diameter grip sits on the inner surface and a radial drag is two millimetres of
        // diameter per millimetre; the wall grip sits on the opposite outer surface.
        let diameter = handles
            .iter()
            .find(|handle| handle.name == "diameter")
            .unwrap();
        assert_eq!(
            diameter.position,
            [
                kind.ring_family().unwrap().defaults().diameter_mm / 2.0,
                0.0,
                0.0
            ]
        );
        assert_eq!(diameter.scale, 2.0);
        let wall = handles.iter().find(|handle| handle.name == "wall").unwrap();
        assert!(wall.position[0] < 0.0);
        assert_eq!(wall.scale, 1.0);
        let (name, field) = match kind {
            BuilderKind::EternityBand => ("stone", "stone_diameter_mm"),
            BuilderKind::RaisedBand => ("raised", "raised_height_mm"),
            _ => ("shoulder", "shoulder_mm"),
        };
        let handle = handles.iter().find(|handle| handle.name == name).unwrap();
        assert_eq!(handle.field, field);
        // A drag and a panel edit reach the same field: the painted value follows the edit.
        let before = handle.value;
        // A step-aligned drag lands on an exact value, so the painted number equals the field.
        // The raised ridge cannot grow into the wall, so its drag moves inward instead.
        let delta = if name == "raised" { -0.5 } else { 0.5 };
        session.adjust_field(field, delta).unwrap();
        session.refresh(&document, &registry).unwrap();
        let moved = session
            .preview()
            .unwrap()
            .handles
            .iter()
            .find(|handle| handle.field == field)
            .unwrap()
            .value;
        assert!((moved - (before + delta)).abs() < 1e-6);
    }
}

#[test]
fn the_final_validation_gate_rejects_an_open_or_too_thin_ring() {
    let params = RingBuilderKind::Signet.defaults();
    let solid = ring_builder::build(&params).unwrap();
    assert!(ring_builder::inspect_solid(&solid.mesh).unwrap().closed);
    let mut open = solid.mesh.clone();
    open.faces.pop();
    assert_eq!(
        ring_builder::inspect_solid(&open).err().unwrap(),
        Error::Invalid("ring solid open")
    );
    // A wall thinner than the kernel tolerance is refused before the solid is built.
    let mut thin = params.clone();
    thin.thickness_mm = MIN_WALL_MM / 2.0;
    assert_eq!(thin.validate(), Err(Error::Invalid("ring wall")));
    assert!(ring_builder::build(&thin).is_err());
    // A self-intersecting top contour is refused by the same gate.
    let mut folded = params;
    folded.top_length_mm = 60.0;
    assert_eq!(folded.validate(), Err(Error::Invalid("ring signet top")));
}

#[test]
fn the_ring_family_is_registered_as_builder_modes_and_as_style_targets() {
    assert_eq!(BuilderKind::ALL.len(), 9);
    for family in RingBuilderKind::ALL {
        let kind = BuilderKind::from_tag(family.tag()).unwrap();
        assert_eq!(kind.ring_family(), Some(family));
        assert_eq!(kind.command(), family.command());
        assert!(kind.is_ring_builder());
        assert_eq!(
            om9_core::producers::expected_output_kind(family.command()),
            Some(om9_core::geometry::GeometryKind::Mesh)
        );
    }
    // The ring styles match the family, a gem style still does not.
    assert!(StyleFamily::Band.matches(BuilderKind::EternityBand));
    assert!(StyleFamily::Signet.matches(BuilderKind::SignetRing));
    assert!(!StyleFamily::Gem.matches(BuilderKind::AwardRing));
    // A request without overrides builds the family defaults.
    let request = RebuildRequest {
        recipe: om9_core::identity::HistoryId::new(),
        command: RingBuilderKind::Raised.command().to_string(),
        document: om9_core::identity::DocumentId::new(),
        parameters: BTreeMap::new(),
        inputs: Vec::new(),
        outputs: vec![ObjectId::new()],
    };
    let params = RingParams::from_request(RingBuilderKind::Raised, &request).unwrap();
    let solid = ring_builder::build(&params).unwrap();
    assert!(solid.closed && solid.volume_mm3 > 0.0);
    // Field values still read as typed parameters for the panel.
    let session = BuilderSession::new(BuilderKind::AwardRing, &empty_document(), &[]).unwrap();
    let values = field_values(&session);
    assert!(matches!(values["diameter_mm"], Value::Quantity { .. }));
}
