// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::{Limits, Usage, memory::MemoryClass},
    geometry::Geometry,
    model::{Definition, Document, Entity},
    transaction::{TransactionCoordinator, World},
};
use om9_scene::scene::Scene;
use std::sync::Arc;

#[test]
fn scene_cache_usage_counts_and_releases_lazy_picking_allocations() {
    let mut document = Document::new("picking cache accounting");
    for index in 0..200 {
        let entity = Entity::new(format!("point {index}"), Geometry::Point([index as f64, 0., 0.]), None);
        document.entities.insert(entity.id, entity);
    }
    let limits = Limits::default();
    let mut scene = Scene::capture(Arc::new(document), 1, &limits).unwrap();
    let initial = scene.cache_usage().unwrap().bytes;
    let scratch_estimate = scene.nearest_pick_scratch_estimate().unwrap().bytes;
    assert_eq!(scene.object_spheres().unwrap().len(), 200);
    let spheres = scene.cache_usage().unwrap().bytes;
    assert!(spheres >= initial + 200 * std::mem::size_of::<([f64; 3], f64)>() as u64);
    scene.spatial_index().unwrap();
    let indexed = scene.cache_usage().unwrap().bytes;
    assert_eq!(indexed, spheres, "index rebuild uses the already admitted arena");
    assert!(scratch_estimate >= indexed - initial, "estimate includes the observed retained index allocations");
    assert_eq!(scene.cache_usage().unwrap().bytes, indexed);
    let snapshot = scene.try_render_snapshot().unwrap();
    assert_eq!(snapshot.cache_usage().unwrap().bytes, initial, "render snapshot starts without picking caches");
    assert!(Arc::ptr_eq(&snapshot.document, &scene.document));
    assert!(snapshot.objects.iter().zip(&scene.objects).all(|(a, b)| Arc::ptr_eq(&a.buffers, &b.buffers)));
    scene.invalidate_picking_bounds();
    assert_eq!(scene.cache_usage().unwrap().bytes, indexed, "preview invalidation retains arena capacity");
    scene.evict_picking_cache().unwrap();
    assert_eq!(scene.cache_usage().unwrap().bytes, initial);
}

#[test]
fn candidate_scene_estimate_covers_observed_scene_cache() {
    let limits = Limits::default();
    let empty = Document::new("empty candidate");
    let empty_planning = Scene::candidate_planning_scratch(&empty).unwrap();
    let empty_validation = empty.validation_scratch_usage(&limits).unwrap();
    assert!(empty_validation.bytes > empty_planning.bytes, "fixture must exercise validation as the larger sequential phase");
    assert_eq!(Scene::candidate_preflight_scratch(&empty, &limits).unwrap().bytes, empty_validation.bytes);
    let mut document = Document::new("candidate scene estimate");
    for index in 0..200 {
        let entity = Entity::new(format!("point {index}"), Geometry::Point([index as f64, 0., 0.]), None);
        document.entities.insert(entity.id, entity);
    }
    let scratch = Scene::candidate_planning_scratch(&document).unwrap();
    let validation = document.validation_scratch_usage(&limits).unwrap();
    let combined = Scene::candidate_preflight_scratch(&document, &limits).unwrap();
    assert_eq!(combined.bytes, scratch.bytes.max(validation.bytes), "sequential scratch must be admitted at its peak without summing both stages");
    let peak = Scene::candidate_scene_peak(&document, &limits).unwrap();
    let scene = Scene::capture(Arc::new(document), 1, &limits).unwrap();
    assert!(scratch.bytes > 0);
    assert!(peak >= scene.cache_usage().unwrap().bytes, "construction estimate should cover retained scene cache");
}

#[test]
fn world_scene_capture_respects_the_shared_managed_ceiling() {
    let mut document = Document::new("managed scene capture");
    for index in 0..500 {
        let entity = Entity::new(format!("point {index}"), Geometry::Point([index as f64, 0., 0.]), None);
        document.entities.insert(entity.id, entity);
    }
    let id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let held = coordinator.retained_usage().unwrap().bytes;
    let total = (held + 128 * 1024).checked_mul(5).unwrap().div_ceil(3);
    coordinator.constrain_managed_ram(total, Usage::default()).unwrap();
    let before = coordinator.snapshot();
    assert!(matches!(Scene::capture_in_world(&coordinator, id, Usage::default()), Err(Error::Budget(_))));
    assert_eq!(coordinator.snapshot(), before);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0, "failed planning must release its reservation");
    // The underlying structural/default limit would otherwise allow this scene.
    assert!(Scene::capture(coordinator.world().documents[&id].clone(), coordinator.epoch(), &limits).is_ok());
}

#[test]
fn world_scene_capture_samples_physical_headroom_at_all_three_stages() {
    const MIB: u64 = 1024 * 1024;
    let mut document = Document::new("scene sampled admission");
    for index in 0..200 {
        let entity = Entity::new("point", Geometry::Point([index as f64, 0., 0.]), None);
        document.entities.insert(entity.id, entity);
    }
    let id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let before = coordinator.snapshot();
    for failure_stage in 0..3 {
        let mut samples = 0;
        let result = Scene::capture_in_world_with_physical_samples(&coordinator, id, Usage::default(), || {
            let index = samples;
            samples += 1;
            Some((1024 * MIB, if index == failure_stage { 256 * MIB } else { 1024 * MIB }))
        });
        assert!(matches!(result, Err(Error::Budget("available physical RAM"))));
        assert_eq!(samples, failure_stage + 1, "must stop at the failing stage");
        assert_eq!(coordinator.snapshot(), before);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }
    let mut samples = 0;
    let scene = Scene::capture_in_world_with_physical_samples(&coordinator, id, Usage::default(), || {
        samples += 1;
        Some((1024 * MIB, 1024 * MIB))
    }).unwrap();
    assert_eq!(samples, 3);
    assert_eq!(scene.objects.len(), 200);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
}

#[test]
fn retained_scene_capture_hands_reservation_to_caller() {
    let mut document = Document::new("scene handoff");
    for index in 0..200 {
        let entity = Entity::new("point", Geometry::Point([index as f64, 0., 0.]), None);
        document.entities.insert(entity.id, entity);
    }
    let id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
    peer.adopt_external_captures(&coordinator).unwrap();
    let document = coordinator.world().documents[&id].clone();
    let (scene, reservation) = Scene::capture_retained_reserved_with_physical_samples(
        &coordinator, document, coordinator.epoch(), Usage::default(), || None,
    ).unwrap();
    assert_eq!(scene.objects.len(), 200);
    assert!(reservation.bytes() >= scene.cache_held_usage().unwrap().bytes);
    assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), reservation.bytes());
    let rest = peer.reserve_remaining_external_memory_with_held(MemoryClass::Export, None, Usage::default()).unwrap();
    assert!(matches!(peer.reserve_external_memory(MemoryClass::Export, 1, None), Err(Error::Budget(_))));
    drop(rest);
    drop(reservation);
    assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0);
}

#[test]
fn world_scene_capture_releases_temporary_reservations_on_success() {
    let mut document = Document::new("scene reservation lifetime");
    let entity = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
    document.entities.insert(entity.id, entity);
    let id = document.id;
    let limits = Limits::default();
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, id, Usage::default()).unwrap();
    assert_eq!(scene.objects.len(), 1);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
}

#[test]
fn empty_shared_instances_charge_each_occurrence_before_descent() {
    let mut document = Document::new("empty shared instance expansion");
    let empty = Definition::new("empty", vec![]);
    let mut definition_id = empty.id;
    document.definitions.insert(empty.id, empty);
    for level in 0..7 {
        let definition = Definition::new(
            format!("level {level}"),
            (0..2)
                .map(|branch| {
                    Entity::new(
                        format!("branch {branch}"),
                        Geometry::Instance(definition_id),
                        None,
                    )
                })
                .collect(),
        );
        definition_id = definition.id;
        document.definitions.insert(definition.id, definition);
    }
    for index in 0..100 {
        let root = Entity::new(
            format!("root {index}"),
            Geometry::Instance(definition_id),
            None,
        );
        document.entities.insert(root.id, root);
    }
    let limits = Limits {
        max_nodes: 2_000,
        max_bytes: 10_000_000,
        ..Limits::default()
    };
    // Catalog validation counts shared definitions once; scene expansion must
    // also bound 25,500 occurrences even though none has drawable geometry.
    assert!(document.validate(&limits).unwrap().nodes < limits.max_nodes);
    assert!(matches!(
        Scene::capture(Arc::new(document.clone()), 1, &limits),
        Err(Error::Budget(_))
    ));
    let coordinator =
        TransactionCoordinator::new(World::new(vec![document.clone()], &limits).unwrap(), limits)
            .unwrap();
    assert!(matches!(
        Scene::capture_in_world(&coordinator, document.id, Usage::default()),
        Err(Error::Budget(_))
    ));
    let generous = Limits {
        max_nodes: 30_000,
        ..limits
    };
    assert!(
        Scene::capture(Arc::new(document), 1, &generous)
            .unwrap()
            .objects
            .is_empty()
    );
}
