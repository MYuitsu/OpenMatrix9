// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{budget::{Limits, Usage, memory::{CommitMemory, MemoryClass}}, geometry::{Geometry, Transform},
    model::{Document, Entity}, transaction::{TransactionCoordinator, World}, Error};
use om9_scene::scene::Scene;

fn fixture() -> (TransactionCoordinator, Scene) {
    let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Default::default() };
    let mut document = Document::new("managed picking");
    for index in 0..200 {
        let point = Entity::new("point", Geometry::Point([index as f64, 0., 0.]), None);
        document.entities.insert(point.id, point);
    }
    let id = document.id;
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let scene = Scene::capture_in_world(&coordinator, id, Usage::default()).unwrap();
    let held = Usage { bytes: scene.shared_buffer_bytes(), ..Default::default() };
    scene.reserve_memory_allowance(&coordinator, held, None).unwrap();
    (coordinator, scene)
}

#[test]
fn metadata_and_cache_survive_borrow_preview_reuse_eviction_and_readmission() {
    let (coordinator, mut scene) = fixture();
    let quota = scene.memory_allowance_bytes();
    let metadata = scene.metadata_storage_usage().unwrap().bytes;
    let arena = scene.picking_arena_bytes().unwrap();
    assert_eq!(quota, metadata + arena);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), quota);
    assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), arena);
    assert_eq!(scene.cache_held_usage().unwrap().bytes, scene.shared_buffer_bytes());
    let old = scene.object_spheres().unwrap()[0].0;
    scene.spatial_index().unwrap();
    assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
    let actual = scene.cache_usage().unwrap().bytes;
    scene.objects[0].preview = Transform::translation([25., 0., 0.]);
    scene.invalidate_picking_bounds();
    assert_eq!(scene.cache_usage().unwrap().bytes, actual, "invalidation keeps allocated arena storage");
    assert_eq!(scene.object_spheres().unwrap()[0].0, [old[0] + 25., old[1], old[2]]);
    scene.spatial_index().unwrap();
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), quota);
    scene.evict_picking_cache().unwrap();
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), metadata);
    assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
    // A read re-admits from the same account; callers need no manual re-arm.
    scene.spatial_index().unwrap();
    assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
    drop(scene);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
}

#[test]
fn read_handle_keeps_arena_charged_after_scene_eviction_and_drop() {
    let (coordinator, mut scene) = fixture();
    let spheres = scene.object_spheres().unwrap();
    let first = spheres[0];
    let arena = scene.picking_arena_bytes().unwrap();
    let metadata = scene.metadata_storage_usage().unwrap().bytes;
    scene.evict_picking_cache().unwrap();
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), metadata + arena);
    assert_eq!(spheres[0], first);
    let retained_before = coordinator.retained_usage().unwrap().bytes;
    drop(scene);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), arena);
    assert!(retained_before - coordinator.retained_usage().unwrap().bytes > metadata,
        "the last scene also retires its weak registry slot; the pinned arena remains owned separately");
    assert_eq!(spheres[0], first);
    drop(spheres);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
}

#[test]
fn preview_with_an_old_reader_admits_overlap_and_keeps_both_snapshots_correct() {
    let (coordinator, mut scene) = fixture();
    let old = scene.object_spheres().unwrap();
    let position = old[0].0;
    let arena = scene.picking_arena_bytes().unwrap();
    let metadata = scene.metadata_storage_usage().unwrap().bytes;
    scene.objects[0].preview = Transform::translation([25., 0., 0.]);
    scene.invalidate_picking_bounds();
    let new = scene.object_spheres().unwrap();
    assert_eq!(old[0].0, position);
    assert_eq!(new[0].0, [position[0] + 25., position[1], position[2]]);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), metadata + 2 * arena);
    drop(old);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), metadata + arena);
}

#[test]
fn binding_an_existing_raw_cache_requires_exclusive_ownership_and_counts_it_allocated() {
    let (coordinator, source) = fixture();
    let document = source.document.clone();
    drop(source);
    let raw = Scene::capture(document, coordinator.epoch(), coordinator.limits()).unwrap();
    let reader = raw.object_spheres().unwrap();
    let held = Usage { bytes: raw.shared_buffer_bytes(), ..Default::default() };
    assert!(matches!(raw.reserve_memory_allowance(&coordinator, held, None), Err(Error::Busy)));
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    drop(reader);
    raw.reserve_memory_allowance(&coordinator, held, None).unwrap();
    assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), raw.managed_memory_allowance().unwrap());
    drop(raw);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
}

#[test]
fn pressure_reclaims_warm_or_unused_allowance_and_readmits_after_pressure() {
    for warm in [false, true] {
        let (coordinator, scene) = fixture();
        let buffers = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache,
            scene.shared_buffer_bytes(), scene.shared_buffer_bytes(), None, Usage::default()).unwrap();
        let before = coordinator.snapshot();
        if warm { scene.spatial_index().unwrap(); }
        let arena = scene.picking_arena_bytes().unwrap();
        let metadata = scene.metadata_storage_usage().unwrap().bytes;
        let mut pressure = coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, Usage::default()).unwrap();
        let scratch = scene.document.validation_scratch_usage(coordinator.limits()).unwrap().bytes;
        pressure.shrink_to(pressure.bytes() - scratch).unwrap();
        assert!(coordinator.last_external_memory_refusal().is_none());
        let request = coordinator.reserve_external_memory(MemoryClass::Export, 1, None).unwrap();
        assert!(coordinator.last_external_memory_refusal().is_none(), "successful reclamation must not publish transient refusal attempts");
        assert_eq!(scene.memory_allowance_bytes(), metadata, "only optional allowance was reclaimed");
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), metadata + buffers.bytes());
        assert_eq!(coordinator.snapshot(), before);
        drop(request);
        drop(pressure);
        let spheres = scene.object_spheres().unwrap();
        assert_eq!(spheres.len(), 200);
        scene.spatial_index().unwrap();
        assert_eq!(scene.memory_allowance_bytes(), metadata + arena);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
    }
}

#[test]
fn pressure_does_not_evict_a_pinned_reader_or_report_its_bytes_free() {
    let (coordinator, scene) = fixture();
    let _buffers = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache,
        scene.shared_buffer_bytes(), scene.shared_buffer_bytes(), None, Usage::default()).unwrap();
    let reader = scene.object_spheres().unwrap();
    let charged = scene.memory_allowance_bytes();
    let point = reader[0];
    let mut pressure = coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, Usage::default()).unwrap();
    let scratch = scene.document.validation_scratch_usage(coordinator.limits()).unwrap().bytes;
    pressure.shrink_to(pressure.bytes() - scratch).unwrap();
    assert!(matches!(coordinator.reserve_external_memory(MemoryClass::Export, 1, None), Err(Error::Budget(_))));
    assert_eq!(scene.memory_allowance_bytes(), charged);
    assert_eq!(reader[0], point);
    drop(reader);
    let admitted = coordinator.reserve_external_memory(MemoryClass::Export, 1, None).unwrap();
    assert!(scene.memory_allowance_bytes() < charged);
    drop(admitted);
    drop(pressure);
}

#[test]
fn one_byte_shortage_stops_after_one_cache_and_keeps_other_cache_warm() {
    let (coordinator, scene) = fixture();
    let _buffers = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache,
        scene.shared_buffer_bytes(), scene.shared_buffer_bytes(), None, Usage::default()).unwrap();
    let second = scene.try_render_snapshot().unwrap();
    second.reserve_memory_allowance(&coordinator, Usage::default(), None).unwrap();
    scene.spatial_index().unwrap();
    second.spatial_index().unwrap();
    let other_charge = second.memory_allowance_bytes();
    let mut pressure = coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, Usage::default()).unwrap();
    let scratch = scene.document.validation_scratch_usage(coordinator.limits()).unwrap().bytes;
    pressure.shrink_to(pressure.bytes() - scratch).unwrap();
    let _request = coordinator.reserve_external_memory(MemoryClass::Export, 1, None).unwrap();
    assert_eq!(scene.memory_allowance_bytes(), scene.metadata_storage_usage().unwrap().bytes);
    assert_eq!(second.memory_allowance_bytes(), other_charge, "admission must stop before evicting another warm cache");
}

#[test]
fn unused_snapshot_quota_is_reclaimed_before_the_active_picking_arena() {
    let (coordinator, scene) = fixture();
    let _buffers = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache,
        scene.shared_buffer_bytes(), scene.shared_buffer_bytes(), None, Usage::default()).unwrap();
    scene.spatial_index().unwrap();
    let active = scene.memory_allowance_bytes();
    let cold = scene.try_render_snapshot().unwrap();
    cold.reserve_memory_allowance(&coordinator, Usage::default(), None).unwrap();
    let before = coordinator.snapshot();
    let mut pressure = coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, Usage::default()).unwrap();
    let scratch = scene.document.validation_scratch_usage(coordinator.limits()).unwrap().bytes;
    pressure.shrink_to(pressure.bytes() - scratch).unwrap();
    let _request = coordinator.reserve_external_memory(MemoryClass::Export, 1, None).unwrap();
    assert_eq!(scene.memory_allowance_bytes(), active, "unused quota must go before the active picking arena");
    assert_eq!(cold.memory_allowance_bytes(), cold.metadata_storage_usage().unwrap().bytes);
    assert_eq!(coordinator.snapshot(), before);
}

#[test]
fn recently_used_picking_arena_survives_before_an_older_snapshot() {
    let (coordinator, scene) = fixture();
    let _buffers = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache,
        scene.shared_buffer_bytes(), scene.shared_buffer_bytes(), None, Usage::default()).unwrap();
    let older = scene.try_render_snapshot().unwrap();
    older.reserve_memory_allowance(&coordinator, Usage::default(), None).unwrap();
    scene.spatial_index().unwrap();
    older.spatial_index().unwrap();
    // The first registered scene is now the most recently used one.
    scene.object_spheres().unwrap();
    let active = scene.memory_allowance_bytes();
    let mut pressure = coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, Usage::default()).unwrap();
    let scratch = scene.document.validation_scratch_usage(coordinator.limits()).unwrap().bytes;
    pressure.shrink_to(pressure.bytes() - scratch).unwrap();
    let _request = coordinator.reserve_external_memory(MemoryClass::Export, 1, None).unwrap();
    assert_eq!(scene.memory_allowance_bytes(), active, "last access, not registration order, protects the active picker");
    assert_eq!(older.memory_allowance_bytes(), older.metadata_storage_usage().unwrap().bytes);
}

#[test]
fn passive_bounds_never_allocate_probe_or_rearm_an_evicted_arena() {
    let (mut coordinator, mut scene) = fixture();
    let initial = coordinator.external_reserved_bytes(MemoryClass::CpuCache);
    let pending = coordinator.external_pending_bytes(MemoryClass::CpuCache);
    assert!(scene.cached_object_spheres().is_none());
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), initial);
    assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), pending);
    scene.object_spheres().unwrap();
    let expected = scene.cached_object_spheres().unwrap()[0];
    coordinator.set_physical_memory_sampler(|| panic!("passive bounds must not run admission or physical probes"));
    for _ in 0..240 {
        assert_eq!(scene.cached_object_spheres().unwrap()[0], expected);
    }
    scene.evict_picking_cache().unwrap();
    for _ in 0..240 { assert!(scene.cached_object_spheres().is_none()); }
    assert_eq!(scene.memory_allowance_bytes(), scene.metadata_storage_usage().unwrap().bytes);
    assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
}

#[test]
fn passive_bounds_never_return_the_old_preview_and_keep_their_reader_lease() {
    let (coordinator, mut scene) = fixture();
    let arena = scene.picking_arena_bytes().unwrap();
    scene.object_spheres().unwrap();
    let old = scene.cached_object_spheres().unwrap();
    let point = old[0].0;
    scene.objects[0].preview = Transform::translation([25., 0., 0.]);
    scene.invalidate_picking_bounds();
    assert!(scene.cached_object_spheres().is_none());
    let current = scene.object_spheres().unwrap();
    assert_eq!(current[0].0, [point[0] + 25., point[1], point[2]]);
    assert_eq!(old[0].0, point);
    drop(current);
    drop(scene);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), arena);
    assert_eq!(old[0].0, point);
    drop(old);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
}

#[test]
fn four_passive_view_readers_do_not_promote_or_rebuild_caches_under_pressure() {
    let (coordinator, scene) = fixture();
    let _buffers = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache,
        scene.shared_buffer_bytes(), scene.shared_buffer_bytes(), None, Usage::default()).unwrap();
    let views: Vec<_> = (0..4).map(|_| {
        let view = scene.try_render_snapshot().unwrap();
        view.reserve_memory_allowance(&coordinator, Usage::default(), None).unwrap();
        view.object_spheres().unwrap();
        view
    }).collect();
    scene.spatial_index().unwrap();
    let active = scene.memory_allowance_bytes();
    // Display polling must not make view 0 hotter than a real pick.
    for _ in 0..240 {
        for view in &views { assert!(view.cached_object_spheres().is_some()); }
    }
    let mut pressure = coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, Usage::default()).unwrap();
    let scratch = scene.document.validation_scratch_usage(coordinator.limits()).unwrap().bytes;
    pressure.shrink_to(pressure.bytes() - scratch).unwrap();
    let _request = coordinator.reserve_external_memory(MemoryClass::Export, 1, None).unwrap();
    assert!(views[0].cached_object_spheres().is_none());
    assert_eq!(scene.memory_allowance_bytes(), active);
    let charged = coordinator.external_reserved_bytes(MemoryClass::CpuCache);
    for _ in 0..240 {
        assert!(views[0].cached_object_spheres().is_none());
        for view in &views[1..] { assert!(view.cached_object_spheres().is_some()); }
        assert_eq!(scene.object_spheres().unwrap().len(), 200);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), charged);
        assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
    }
}

#[test]
fn concurrent_requests_cannot_both_spend_the_same_reclaimed_arena() {
    use std::{sync::{Arc, Barrier, mpsc}, time::Duration};
    let (coordinator, scene) = fixture();
    let _buffers = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache,
        scene.shared_buffer_bytes(), scene.shared_buffer_bytes(), None, Usage::default()).unwrap();
    scene.spatial_index().unwrap();
    let request = scene.picking_arena_bytes().unwrap() / 2 + 1;
    let mut pressure = coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, Usage::default()).unwrap();
    let scratch = scene.document.validation_scratch_usage(coordinator.limits()).unwrap().bytes;
    pressure.shrink_to(pressure.bytes() - scratch).unwrap();
    let coordinator = Arc::new(coordinator);
    let start = Arc::new(Barrier::new(3));
    let (report, results) = mpsc::channel();
    let mut workers = Vec::new();
    let mut releases = Vec::new();
    for _ in 0..2 {
        let coordinator = coordinator.clone();
        let start = start.clone();
        let report = report.clone();
        let (release, wait) = mpsc::sync_channel(1);
        releases.push(release);
        workers.push(std::thread::spawn(move || {
            start.wait();
            let lease = coordinator.reserve_external_memory(MemoryClass::Export, request, None);
            report.send(lease.is_ok()).unwrap();
            let _ = wait.recv_timeout(Duration::from_secs(10));
            drop(lease);
        }));
    }
    start.wait();
    let outcomes = [results.recv_timeout(Duration::from_secs(10)).unwrap(), results.recv_timeout(Duration::from_secs(10)).unwrap()];
    for release in releases { release.send(()).unwrap(); }
    for worker in workers { worker.join().unwrap(); }
    assert_eq!(outcomes.iter().filter(|result| **result).count(), 1);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 0);
}

#[test]
fn delayed_cache_allocation_rechecks_commit_and_retries_without_changing_document() {
    let (mut coordinator, scene) = fixture();
    let before = coordinator.snapshot();
    let actual = scene.cache_usage().unwrap().bytes;
    coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * 1024 * 1024, used_bytes: 768 * 1024 * 1024 }));
    assert!(matches!(scene.spatial_index(), Err(Error::Budget("available system commit"))));
    assert_eq!(scene.cache_usage().unwrap().bytes, actual);
    assert_eq!(coordinator.snapshot(), before);
    coordinator.set_commit_memory_sampler(|| Some(CommitMemory { limit_bytes: 1024 * 1024 * 1024, used_bytes: 0 }));
    scene.spatial_index().unwrap();
    assert_eq!(coordinator.snapshot(), before);
    assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
}

#[test]
fn cold_read_reclaims_other_unused_pending_quota_before_refusing_commit() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static AVAILABLE: AtomicU64 = AtomicU64::new(0);
    let (mut coordinator, scene) = fixture();
    let second = scene.try_render_snapshot().unwrap();
    second.reserve_memory_allowance(&coordinator, Usage { bytes: scene.shared_buffer_bytes(), ..Default::default() }, None).unwrap();
    let arena = scene.picking_arena_bytes().unwrap();
    AVAILABLE.store(arena, Ordering::Relaxed);
    coordinator.set_commit_memory_sampler(|| Some(CommitMemory {
        limit_bytes: 1024 * 1024 * 1024,
        used_bytes: 768 * 1024 * 1024 - AVAILABLE.load(Ordering::Relaxed),
    }));
    assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 2 * arena);
    scene.spatial_index().unwrap();
    assert_eq!(second.memory_allowance_bytes(), second.metadata_storage_usage().unwrap().bytes);
    assert_eq!(scene.memory_allowance_bytes(), scene.managed_memory_allowance().unwrap());
    assert_eq!(coordinator.external_pending_bytes(MemoryClass::CpuCache), 0);
}
