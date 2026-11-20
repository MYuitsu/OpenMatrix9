// SPDX-License-Identifier: LGPL-2.1-or-later
use std::collections::BTreeSet;

use om9_core::{
    associative::RecomputeCache,
    budget::Limits,
    geometry::{Curve, Geometry, Transform, Unit},
    history::{Recipe, ReplayEnvironment, Tolerance},
    model::{Document, Entity},
    producers::{join_history_updates_from_candidate, recompute_registered, reference_registry},
    request::{Selection, SourceBinding, Value},
    transaction::{Cancellation, Mutation, TransactionCoordinator, TransactionRequest, World},
    units::LengthMm,
};

fn line(a: f64, b: f64) -> Geometry {
    Geometry::Curve(Curve::Line {
        start: [a, 0.0, 0.0],
        end: [b, 0.0, 0.0],
        domain: [0.0, 1.0],
    })
}

#[test]
fn staged_parent_edit_and_two_dependents_commit_and_undo_together() {
    let limits = Limits::default();
    let mut doc = Document::with_template_tolerance("dependent transaction");
    let mut ids = Vec::new();
    for (a, b) in [(0.0, 1.0), (1.0, 2.0), (2.0, 3.0), (0.0, 2.0), (0.0, 3.0)] {
        let entity = Entity::new("line", line(a, b), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut recipes = Vec::new();
    for (sources, output) in [([ids[0], ids[1]], ids[3]), ([ids[3], ids[2]], ids[4])] {
        let inputs = sources
            .into_iter()
            .map(|id| SourceBinding {
                role: "curve".into(),
                source: doc.stamp(id).unwrap(),
                selection: Selection::Whole,
            })
            .collect();
        let mut recipe = Recipe::new(
            doc.id,
            "JoinHistory",
            inputs,
            vec![output],
            ReplayEnvironment {
                command_version: 1,
                solver: "test".into(),
                solver_version: "1".into(),
                schema_version: 1,
                length_unit: Unit::Millimeter,
                frame: Transform::identity(),
                tolerance: Tolerance::om9_template(),
            },
        );
        recipe.parameters.insert(
            "tolerance_mm".into(),
            Value::length(LengthMm::try_new(0.001).unwrap()),
        );
        recipes.push(recipe.id);
        doc.history.recipes.insert(recipe.id, recipe);
    }
    let registry = reference_registry().unwrap();
    for id in &recipes {
        recompute_registered(&mut doc, &mut RecomputeCache::default(), *id, &registry).unwrap();
    }
    let document = doc.id;
    let mut coordinator =
        TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let before = (*coordinator.world().documents[&document]).clone();
    let source_edit = Mutation::Geometry {
        source: before.stamp(ids[0]).unwrap(),
        geometry: line(-1.0, 1.0),
    };
    let preview = coordinator
        .prepare(TransactionRequest::new(
            "parent edit",
            vec![source_edit.clone()],
        ))
        .unwrap();
    let candidate = (*preview.candidate_world().documents[&document]).clone();
    drop(preview);
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(join_history_updates_from_candidate(
        &before, &mut candidate.clone(), &BTreeSet::from([ids[0]]),
        &registry, &limits, &cancel,
    ).is_err());
    assert!(join_history_updates_from_candidate(
        &before, &mut before.clone(), &BTreeSet::from([ids[0]]),
        &registry, &limits, &Cancellation::default(),
    ).is_err());
    let disconnected = Mutation::Geometry {
        source: before.stamp(ids[1]).unwrap(),
        geometry: line(1.0, 2.1),
    };
    let failed_preview = coordinator.prepare(TransactionRequest::new(
        "disconnect downstream", vec![disconnected],
    )).unwrap();
    let mut failed_candidate = (*failed_preview.candidate_world().documents[&document]).clone();
    drop(failed_preview);
    assert!(join_history_updates_from_candidate(
        &before, &mut failed_candidate, &BTreeSet::from([ids[1]]),
        &registry, &limits, &Cancellation::default(),
    ).is_err());
    assert_eq!(&*coordinator.world().documents[&document], &before);
    assert_eq!(coordinator.undo_len(), 0);
    struct ForbiddenProducer;
    impl om9_core::producers::FeatureProducer for ForbiddenProducer {
        fn produce(&self, _: &om9_core::associative::RebuildRequest)
            -> om9_core::Result<Vec<om9_core::associative::RebuiltOutput>> {
            panic!("unsupported producer must not run")
        }
        fn produce_with_limits(&self, _: &om9_core::associative::RebuildRequest, _: &Limits, _: &Cancellation)
            -> om9_core::Result<Vec<om9_core::associative::RebuiltOutput>> {
            panic!("unsupported producer must not run")
        }
    }
    let mut forbidden = candidate.clone();
    forbidden.history.recipes.get_mut(&recipes[0]).unwrap().command = "Forbidden".into();
    let mut forbidden_registry = om9_core::producers::ProducerRegistry::new();
    forbidden_registry.register("Forbidden", ForbiddenProducer).unwrap();
    assert!(matches!(join_history_updates_from_candidate(
        &before, &mut forbidden, &BTreeSet::from([ids[0]]), &forbidden_registry, &limits, &Cancellation::default()),
        Err(om9_core::Error::Unsupported("dependent history producer handoff"))));
    forbidden.history.recipes.get_mut(&recipes[0]).unwrap().policy.update = false;
    assert!(join_history_updates_from_candidate(
        &before, &mut forbidden, &BTreeSet::from([ids[0]]), &forbidden_registry, &limits, &Cancellation::default()).unwrap().is_empty());
    let mut duplicate_input = candidate.clone();
    let recipe = duplicate_input.history.recipes.get_mut(&recipes[0]).unwrap();
    recipe.inputs.push(recipe.inputs[0].clone());
    assert!(matches!(join_history_updates_from_candidate(
        &before, &mut duplicate_input, &BTreeSet::from([ids[0]]), &registry, &limits, &Cancellation::default()),
        Err(om9_core::Error::Invalid("duplicate join history input"))));
    let duplicate = TransactionRequest::new("duplicate", vec![source_edit.clone(), source_edit.clone()]);
    assert!(coordinator.prepare_join_history_edit(duplicate, Default::default()).is_err());
    let rejected = coordinator.prepare_join_history_edit(
        TransactionRequest::new("budget", vec![source_edit.clone()]), Default::default()).unwrap();
    let mut looser = limits;
    looser.max_points += 1;
    assert!(matches!(rejected.draft_minimum_bytes(&looser), Err(om9_core::Error::Invalid("history draft limits"))));
    let tiny = coordinator.reserve_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::CpuCache, 1, None, Default::default()).unwrap();
    assert!(matches!(rejected.assemble(&limits, &Cancellation::default(), tiny),
        Err(om9_core::Error::Budget("history draft memory"))));
    let rejected = coordinator.prepare_join_history_edit(
        TransactionRequest::new("foreign", vec![source_edit.clone()]), Default::default()).unwrap();
    let other = TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap();
    let foreign = other.reserve_external_memory_with_held(om9_core::budget::memory::MemoryClass::CpuCache,
        rejected.draft_minimum_bytes(&limits).unwrap(), None, Default::default()).unwrap();
    assert!(matches!(rejected.assemble(&limits, &Cancellation::default(), foreign),
        Err(om9_core::Error::Invalid("history draft memory account"))));
    let rejected = coordinator.prepare_join_history_edit(
        TransactionRequest::new("canceled", vec![source_edit.clone()]), Default::default()).unwrap();
    let reservation = coordinator.reserve_external_memory_with_held(om9_core::budget::memory::MemoryClass::CpuCache,
        rejected.draft_minimum_bytes(&limits).unwrap(), None, Default::default()).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(matches!(rejected.assemble(&limits, &cancel, reservation), Err(om9_core::Error::Canceled)));
    let rejected = coordinator.prepare_join_history_edit(
        TransactionRequest::new("refused snapshot", vec![source_edit.clone()]), Default::default()).unwrap();
    let reservation = coordinator.reserve_external_memory_with_held(om9_core::budget::memory::MemoryClass::CpuCache,
        rejected.draft_minimum_bytes(&limits).unwrap() * 2, None, Default::default()).unwrap();
    let mut admissions = 0;
    assert!(matches!(rejected.assemble_with_admission(&limits, &Cancellation::default(), reservation, |_, _| {
        admissions += 1;
        if admissions == 3 { Err(om9_core::Error::Budget("test snapshot refusal")) } else { Ok(()) }
    }), Err(om9_core::Error::Budget("test snapshot refusal"))));
    assert_eq!(admissions, 3);
    assert_eq!(&*coordinator.world().documents[&document], &before);
    assert_eq!(coordinator.undo_len(), 0);
    let rejected = coordinator.prepare_join_history_edit(
        TransactionRequest::new("refused updates", vec![source_edit.clone()]), Default::default()).unwrap();
    let reservation = coordinator.reserve_external_memory_with_held(om9_core::budget::memory::MemoryClass::CpuCache,
        rejected.draft_minimum_bytes(&limits).unwrap() * 2, None, Default::default()).unwrap();
    let mut admissions = 0;
    assert!(matches!(rejected.assemble_with_admission(&limits, &Cancellation::default(), reservation, |_, _| {
        admissions += 1;
        if admissions == 4 { Err(om9_core::Error::Budget("test update refusal")) } else { Ok(()) }
    }), Err(om9_core::Error::Budget("test update refusal"))));
    assert_eq!(admissions, 4);
    assert_eq!(&*coordinator.world().documents[&document], &before);
    assert_eq!(coordinator.undo_len(), 0);
    let segments = (0..64).map(|index| {
        let a = -1.0 + index as f64 / 32.0;
        let Geometry::Curve(curve) = line(a, a + 1.0 / 32.0) else { unreachable!() };
        curve
    }).collect();
    let longer = Mutation::Geometry { source: before.stamp(ids[0]).unwrap(),
        geometry: Geometry::Curve(Curve::Polycurve { segments, reversed: false }) };
    let growing = coordinator.prepare_join_history_edit(
        TransactionRequest::new("growing chain", vec![longer]), Default::default()).unwrap();
    let reservation = coordinator.reserve_external_memory_with_held(om9_core::budget::memory::MemoryClass::CpuCache,
        1, None, Default::default()).unwrap();
    let peer = coordinator.memory_admission_peer().unwrap();
    let mut allowances = Vec::new();
    let (growing_request, growing_input, growing_draft) = growing.assemble_with_admission(
        &limits, &Cancellation::default(), reservation, |guard, required| {
            allowances.push(required);
            let additional = required.saturating_sub(guard.bytes());
            if additional == 0 { Ok(()) } else {
                peer.grow_external_memory_with_held(guard, additional, None, Default::default())
            }
        }).unwrap();
    assert!(allowances.len() >= 3 && allowances[2] > allowances[1], "{allowances:?}");
    drop((growing_request, growing_input, growing_draft));
    let source_request = TransactionRequest::new("parent and history", vec![source_edit.clone()]);
    let draft = coordinator.prepare_join_history_edit(source_request, Default::default()).unwrap();
    let reservation = coordinator.reserve_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::CpuCache, 1, None, Default::default()).unwrap();
    let peer = coordinator.memory_admission_peer().unwrap();
    let (request, _input_reservation, _draft_reservation) = std::thread::spawn(move ||
        draft.assemble_with_admission(&limits, &Cancellation::default(), reservation, |guard, required| {
            let additional = required.saturating_sub(guard.bytes());
            if additional == 0 { Ok(()) } else {
                peer.grow_external_memory_with_held(guard, additional, None, Default::default())
            }
        })).join().unwrap().unwrap();
    assert_eq!(request.mutations.len(), 4);
    let prepared = coordinator.prepare(request).unwrap();
    coordinator.commit(prepared).unwrap();
    let after = (*coordinator.world().documents[&document]).clone();
    let Geometry::Curve(joined) = &after.entities[&ids[4]].geometry else {
        panic!("curve")
    };
    let ends = om9_core::curve_ops::curve_endpoints(joined, &limits).unwrap();
    assert!(ends.contains(&[-1.0, 0.0, 0.0]) && ends.contains(&[3.0, 0.0, 0.0]));
    assert_eq!(coordinator.undo_len(), 1);
    coordinator.undo().unwrap();
    assert_eq!(&*coordinator.world().documents[&document], &before);
    coordinator.redo().unwrap();
    assert_eq!(&*coordinator.world().documents[&document], &after);

    let mut changed_coordinator = TransactionCoordinator::new(
        World::new(vec![before.clone()], &limits).unwrap(), limits).unwrap();
    let draft = changed_coordinator.prepare_join_history_edit(
        TransactionRequest::new("pending", vec![source_edit.clone()]), Default::default()).unwrap();
    let reservation = changed_coordinator.reserve_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::CpuCache, draft.draft_minimum_bytes(&limits).unwrap(),
        None, Default::default()).unwrap();
    let (request, _input, _draft) = draft.assemble(&limits, &Cancellation::default(), reservation).unwrap();
    let intervening = changed_coordinator.prepare(TransactionRequest::new("intervening", vec![Mutation::Geometry {
        source: before.stamp(ids[0]).unwrap(), geometry: line(-2.0, 1.0),
    }])).unwrap();
    changed_coordinator.commit(intervening).unwrap();
    let current = changed_coordinator.world().documents[&document].clone();
    assert!(changed_coordinator.prepare(request).is_err());
    assert_eq!(&*changed_coordinator.world().documents[&document], &*current);
    assert_eq!(changed_coordinator.undo_len(), 1);

    // An add-on may create an unrelated object while it edits a History parent.
    // The insert and all dependent outputs must publish (or fail) together.
    let mut mixed = TransactionCoordinator::new(World::new(vec![before.clone()], &limits).unwrap(), limits).unwrap();
    let inserted = Entity::new("created", Geometry::Point([8., 2., 0.]), None);
    let inserted_id = inserted.id;
    let insert = Mutation::Insert { document, entity: inserted };
    let mut extensions = before.entities[&ids[0]].extensions.clone();
    extensions.insert("om9.flow".into(), om9_core::model::Extension::new(1, "{\"label\":\"updated\"}").unwrap());
    let metadata = Mutation::Extensions { source: before.stamp(ids[0]).unwrap(), extensions };
    assert!(matches!(mixed.prepare_join_history_edit(TransactionRequest::new("insert only", vec![insert.clone()]), Default::default()),
        Err(om9_core::Error::Invalid("history source edit request"))));
    let foreign_insert = Mutation::Insert { document: om9_core::identity::DocumentId::new(), entity: Entity::new("foreign", Geometry::Point([0.; 3]), None) };
    assert!(matches!(mixed.prepare_join_history_edit(TransactionRequest::new("foreign", vec![source_edit.clone(), foreign_insert]), Default::default()),
        Err(om9_core::Error::Invalid("history source edit documents"))));
    let draft = mixed.prepare_join_history_edit(TransactionRequest::new("mixed", vec![source_edit.clone(), metadata, insert.clone()]), Default::default()).unwrap();
    let reservation = mixed.reserve_external_memory_with_held(om9_core::budget::memory::MemoryClass::CpuCache,
        draft.draft_minimum_bytes(&limits).unwrap() * 2, None, Default::default()).unwrap();
    let (request, _input, _draft) = draft.assemble(&limits, &Cancellation::default(), reservation).unwrap();
    mixed.commit(mixed.prepare(request).unwrap()).unwrap();
    let after_mixed = (*mixed.world().documents[&document]).clone();
    assert!(after_mixed.entities.contains_key(&inserted_id));
    assert!(after_mixed.entities[&ids[0]].extensions.contains_key("om9.flow"));
    let Geometry::Curve(curve) = &after_mixed.entities[&ids[4]].geometry else { panic!("curve") };
    assert!(om9_core::curve_ops::curve_endpoints(curve, &limits).unwrap().contains(&[-1., 0., 0.]));
    mixed.undo().unwrap();
    assert_eq!(&*mixed.world().documents[&document], &before);
    mixed.redo().unwrap();
    assert_eq!(&*mixed.world().documents[&document], &after_mixed);
    let failed = TransactionCoordinator::new(World::new(vec![before.clone()], &limits).unwrap(), limits).unwrap();
    let draft = failed.prepare_join_history_edit(TransactionRequest::new("failed mixed", vec![
        Mutation::Geometry { source: before.stamp(ids[1]).unwrap(), geometry: line(1., 2.1) }, insert,
    ]), Default::default()).unwrap();
    let reservation = failed.reserve_external_memory_with_held(om9_core::budget::memory::MemoryClass::CpuCache,
        draft.draft_minimum_bytes(&limits).unwrap() * 2, None, Default::default()).unwrap();
    assert!(draft.assemble(&limits, &Cancellation::default(), reservation).is_err());
    assert_eq!(&*failed.world().documents[&document], &before);
    assert_eq!(failed.undo_len(), 0);
}

#[test]
fn many_source_edits_are_prepared_and_duplicate_sources_are_refused() {
    let limits = Limits::default();
    let mut doc = Document::with_template_tolerance("many sources");
    let mut ids = Vec::new();
    for index in 0..256 {
        let entity = Entity::new("line", line(index as f64, index as f64 + 1.0), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let edits: Vec<_> = ids.iter().map(|id| Mutation::Geometry {
        source: doc.stamp(*id).unwrap(), geometry: line(-1.0, 1.0),
    }).collect();
    let document = doc.id;
    let coordinator = TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let prepared = coordinator.prepare_join_history_edit(
        TransactionRequest::new("many sources", edits.clone()), Default::default()).unwrap();
    drop(prepared);
    let mut duplicates = edits;
    duplicates.push(duplicates[0].clone());
    assert!(matches!(coordinator.prepare_join_history_edit(
        TransactionRequest::new("duplicate source", duplicates), Default::default()),
        Err(om9_core::Error::Invalid("duplicate history source edit"))));
    assert_eq!(coordinator.world().documents[&document].entities.len(), 256);
    assert_eq!(coordinator.undo_len(), 0);
}

#[test]
fn captured_preparation_peer_commits_only_against_its_original_epoch() {
    let limits = Limits::default();
    let mut doc = Document::with_template_tolerance("worker transaction");
    let entity = Entity::new("line", line(0.0, 1.0), None);
    let id = entity.id;
    let document = doc.id;
    doc.entities.insert(id, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    let original = (*coordinator.world().documents[&document]).clone();
    let request = TransactionRequest::new("worker edit", vec![Mutation::Geometry {
        source: original.stamp(id).unwrap(), geometry: line(-1.0, 1.0),
    }]);
    let peer = coordinator.capture_transaction_preparation_peer(Default::default()).unwrap();
    let (peer, prepared) = std::thread::spawn(move || {
        let prepared = peer.coordinator().prepare(request).unwrap();
        peer.coordinator().register_prepared_document(&prepared, document).unwrap();
        (peer, prepared)
    }).join().unwrap();
    coordinator.commit(prepared).unwrap();
    drop(peer);
    assert_eq!(coordinator.undo_len(), 1);
    coordinator.undo().unwrap();
    assert_eq!(&*coordinator.world().documents[&document], &original);
    coordinator.redo().unwrap();

    let current = (*coordinator.world().documents[&document]).clone();
    let request = TransactionRequest::new("stale worker edit", vec![Mutation::Geometry {
        source: current.stamp(id).unwrap(), geometry: line(-2.0, 1.0),
    }]);
    let peer = coordinator.capture_transaction_preparation_peer(Default::default()).unwrap();
    let prepared = peer.coordinator().prepare(request).unwrap();
    let intervening = coordinator.prepare(TransactionRequest::new("intervening", vec![Mutation::Geometry {
        source: current.stamp(id).unwrap(), geometry: line(-3.0, 1.0),
    }])).unwrap();
    coordinator.commit(intervening).unwrap();
    let changed = coordinator.world().documents[&document].clone();
    assert!(matches!(coordinator.commit(prepared), Err(om9_core::Error::Stale(_))));
    assert_eq!(&*coordinator.world().documents[&document], &*changed);
    assert_eq!(coordinator.undo_len(), 2);
}
