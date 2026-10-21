// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_archive::{Archive, ReplayKey};
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform, Unit},
    history::{EvaluationState, Recipe, ReplayEnvironment, Tolerance},
    model::{Document, Entity},
    request::{Selection, SourceBinding},
    result::{ConversionReport, PreparedOutput, PreparedResult, ResultStatus},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};
use std::collections::BTreeSet;
fn fixture() -> Document {
    let mut d = Document::new("durable recipe");
    let e = Entity::new("A", Geometry::Point([1., 2., 3.]), d.current_layer);
    let id = e.id;
    d.entities.insert(id, e);
    let did = d.id;
    let binding = SourceBinding {
        role: "Point".into(),
        source: d.stamp(id).unwrap(),
        selection: Selection::Whole,
    };
    let output = Entity::new("B", Geometry::Point([2., 4., 6.]), d.current_layer);
    let environment = ReplayEnvironment {
        command_version: 1,
        solver: "fixture-point".into(),
        solver_version: "1".into(),
        schema_version: 1,
        length_unit: Unit::Millimeter,
        frame: Transform::identity(),
        tolerance: Tolerance {
            absolute_mm: 0.001,
            relative: 1e-6,
            angular_rad: 1e-5,
        },
    };
    let recipe = Recipe::new(
        did,
        "fixture-command",
        vec![binding.clone()],
        vec![output.id],
        environment,
    );
    let result = PreparedResult {
        document: did,
        operation: "fixture-command".into(),
        sources: vec![binding.clone()],
        outputs: vec![PreparedOutput {
            entity: output.clone(),
            expected_existing: None,
            geometry_snapshot: output.geometry.clone(),
            geometry_result: output.geometry.clone(),
            source_stamps: vec![binding.source],
            conversion: ConversionReport::identity(output.geometry.kind()),
        }],
        recipe: Some(recipe),
    };
    let mut c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "fixture",
            vec![Mutation::Publish(Box::new(result))],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.world().documents[&did].as_ref().clone()
}

#[test]
fn two_independent_restores_of_the_same_bytes_have_distinct_owner_sessions() {
    let d = fixture();
    let id = *d.entities.keys().next().unwrap();
    let bytes = Archive::encode(&d, &Limits::default()).unwrap();
    let a = Archive::restore(&bytes, &Limits::default(), &BTreeSet::new()).unwrap();
    let b = Archive::restore(&bytes, &Limits::default(), &BTreeSet::new()).unwrap();
    assert_ne!(a.document.stamp(id).unwrap(), b.document.stamp(id).unwrap());
    assert!(b.document.resolve(&a.document.stamp(id).unwrap()).is_err());
    assert_eq!(a.document.entities, b.document.entities);
}

#[test]
fn schema_one_legacy_owner_fields_migrate_without_promoting_stale_source_stamps() {
    let document = fixture();
    let mut value = serde_json::to_value(document).unwrap();
    fn erase(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("owner_session");
                for value in map.values_mut() {
                    erase(value);
                }
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    erase(value);
                }
            }
            _ => {}
        }
    }
    erase(&mut value);
    let legacy: Document = serde_json::from_value(value).unwrap();
    assert!(legacy.owner_session.is_nil());
    legacy.validate(&Limits::default()).unwrap();
    let id = *legacy.entities.keys().next().unwrap();
    let old = legacy.stamp(id).unwrap();
    let bytes = Archive::encode(&legacy, &Limits::default()).unwrap();
    let restored = Archive::restore(&bytes, &Limits::default(), &BTreeSet::new()).unwrap();
    assert!(!restored.document.owner_session.is_nil());
    assert!(restored.document.resolve(&old).is_err());
    assert!(
        restored
            .document
            .history
            .recipes
            .values()
            .all(|r| matches!(r.state, EvaluationState::Unresolved(_)))
    );
}
#[test]
fn missing_replay_module_keeps_geometry_but_marks_recipe_and_result_unresolved_stale() {
    let d = fixture();
    let bytes = Archive::encode(&d, &Limits::default()).unwrap();
    let restored = Archive::restore(&bytes, &Limits::default(), &BTreeSet::new()).unwrap();
    assert_eq!(restored.diagnostics.len(), 1);
    assert!(
        restored
            .document
            .history
            .recipes
            .values()
            .all(|r| matches!(r.state, EvaluationState::Unresolved(_)))
    );
    assert!(
        restored
            .document
            .results
            .values()
            .all(|r| r.status == ResultStatus::Stale)
    );
    assert_eq!(restored.document.entities, d.entities);
    assert!(restored.document.session_generation > d.session_generation);
}
#[test]
fn matching_registered_replay_key_rebinds_only_verified_current_session_stamps() {
    let d = fixture();
    let old = d.stamp(*d.entities.keys().next().unwrap()).unwrap();
    let bytes = Archive::encode(&d, &Limits::default()).unwrap();
    let keys = BTreeSet::from([ReplayKey {
        solver: "fixture-point".into(),
        solver_version: "1".into(),
        schema_version: 1,
        command: "fixture-command".into(),
        command_version: 1,
    }]);
    let restored = Archive::restore(&bytes, &Limits::default(), &keys).unwrap();
    assert!(restored.diagnostics.is_empty());
    assert!(restored.document.resolve(&old).is_err());
    assert!(
        restored
            .document
            .history
            .recipes
            .values()
            .all(|r| r.state == EvaluationState::Clean)
    );
    restored.document.validate(&Limits::default()).unwrap();
}
#[test]
fn solver_version_mismatch_never_acquires_clean_replay_from_similar_name() {
    let d = fixture();
    let bytes = Archive::encode(&d, &Limits::default()).unwrap();
    let keys = BTreeSet::from([ReplayKey {
        solver: "fixture-point".into(),
        solver_version: "2".into(),
        schema_version: 1,
        command: "fixture-command".into(),
        command_version: 1,
    }]);
    let restored = Archive::restore(&bytes, &Limits::default(), &keys).unwrap();
    assert_eq!(restored.diagnostics.len(), 1);
    assert!(
        restored
            .document
            .history
            .recipes
            .values()
            .all(|r| matches!(r.state, EvaluationState::Unresolved(_)))
    );
}

#[test]
fn restoration_keeps_long_diagnostics_owned_at_the_source_parser_admission_boundary() {
    let mut d = fixture();
    let recipe = d.history.recipes.values_mut().next().unwrap();
    recipe.command = "c".repeat(4_000);
    recipe.environment.solver = "s".repeat(4_000);
    recipe.environment.solver_version = "v".repeat(4_000);
    recipe.inputs.clear();
    d.results.clear();
    let bytes = Archive::encode(&d, &Limits::default()).unwrap();
    // Calibrate the complete restoration boundary. Decoding retains the input,
    // the JSON scratch DOM and the typed document; restoration additionally
    // keeps the long recipe-state message and the returned diagnostic copy.
    // Restoration therefore legitimately needs more headroom than decode alone,
    // so find the smallest budget it accepts instead of assuming both boundaries
    // coincide.
    let (mut low, mut high) = (0, Limits::default().max_bytes);
    while low < high {
        let middle = low + (high - low) / 2;
        let limits = Limits {
            max_bytes: middle,
            ..Limits::default()
        };
        match Archive::restore(&bytes, &limits, &BTreeSet::new()) {
            Ok(_) => high = middle,
            Err(om9_core::Error::Budget(_)) => low = middle + 1,
            Err(error) => panic!("valid source restore failed: {error}"),
        }
    }
    let limits = Limits {
        max_bytes: low,
        ..Limits::default()
    };
    // Verify the long recipe-state and returned diagnostic copies both survive;
    // one byte less is refused before any diagnostic is published.
    Archive::decode(&bytes, &limits).expect("the source document itself fits");
    let restored = Archive::restore(&bytes, &limits, &BTreeSet::new()).unwrap();
    assert_eq!(restored.diagnostics.len(), 1);
    assert!(restored.diagnostics[0].len() > 12_000);
    let recipe = restored.document.history.recipes.values().next().unwrap();
    assert_eq!(
        recipe.state,
        EvaluationState::Unresolved(restored.diagnostics[0].clone())
    );
    assert!(matches!(
        Archive::restore(
            &bytes,
            &Limits {
                max_bytes: limits.max_bytes - 1,
                ..limits
            },
            &BTreeSet::new()
        ),
        Err(om9_core::Error::Budget(_))
    ));
}
