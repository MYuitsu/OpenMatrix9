// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error, Result,
    budget::Limits,
    geometry::{Curve, Geometry, Point3, Unit, finite},
    history::{EvaluationState, Recipe, ReplayEnvironment, Tolerance},
    identity::DocumentId,
    model::{Document, Entity},
    request::{Selection, SourceBinding},
    transaction::{Mutation, TransactionCoordinator, TransactionRequest, World},
};
use om9_scene::{
    camera::CPlane,
    scene::{RenderBuffers, Scene},
    workspace::Workspace,
};
use std::{collections::VecDeque, sync::Arc};

#[cfg(all(test, feature = "occt-brep"))]
thread_local! {
    static TEST_REFUSE_TOPOLOGY_CACHE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static TEST_REFUSE_SPHERE_HISTORY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(all(test, feature = "occt-brep"))]
mod topology_owner_tests {
    use super::*;
    use om9_core::budget::memory::MemoryClass;

    fn controller_with_topology_proof() -> (Controller, om9_core::identity::ObjectId) {
        let limits = Limits::default();
        let mut controller = Controller::new(Document::with_template_tolerance("topology owner"), limits).unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = controller.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache, 4 * 1024 * 1024, physical, held).unwrap();
        let snapshot = om9_kernel::creation::primitive_with_topology(
            om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits,
            |live, pending| {
                if live.checked_add(pending).ok_or(Error::Overflow("test capture"))? > reservation.bytes() {
                    return Err(Error::Budget("test capture"));
                }
                reservation.set_allocated_bytes(live)?;
                reservation.recheck_pending()
            }).unwrap();
        let (body, proof) = snapshot.into_parts();
        let proof_bytes = proof.storage_bytes().unwrap();
        let total = proof_bytes + body.storage_usage(&limits).unwrap().bytes;
        reservation.set_allocated_bytes(total).unwrap();
        reservation.shrink_to(total).unwrap();
        let proof_reservation = reservation.split_off(proof_bytes).unwrap();
        let entity = Entity::new("box", Geometry::Brep(body), None);
        let id = entity.id;
        controller.commit_entity_reserved_brep("OM9-SOLID-012", "box", entity, &mut reservation).unwrap();
        assert_eq!(reservation.bytes(), 0);
        drop(reservation);
        let stamp = controller.document().stamp(id).unwrap();
        controller.adopt_topology_proof(stamp, proof, proof_reservation).unwrap();
        (controller, id)
    }

    #[test]
    fn history_worker_redo_restores_occt_scene_display_and_pick_binding() {
        let (mut controller, id) = controller_with_topology_proof();
        let before = controller.document().clone();
        controller.request_history_worker(false).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_history_worker() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(!controller.document().entities.contains_key(&id));
        controller.request_history_worker(true).unwrap();
        loop {
            if let Some(result) = controller.poll_history_worker() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(controller.document(), &before);
        let stamp = controller.document().stamp(id).unwrap();
        let scene = controller.scene().unwrap();
        let display = scene.root_brep_pick_display(&stamp).unwrap();
        let Geometry::Brep(brep) = &controller.document().entities[&id].geometry else { panic!("B-rep") };
        assert_eq!(display.topology.faces, brep.topology().faces);
        assert_eq!(display.topology.edges, brep.topology().edges);
        for axis in 0..3 {
            let lo = display.points.iter().map(|point| point[axis]).fold(f64::INFINITY, f64::min);
            let hi = display.points.iter().map(|point| point[axis]).fold(f64::NEG_INFINITY, f64::max);
            assert!(lo.abs() < 1e-8 && (hi - [4., 5., 6.][axis]).abs() < 1e-8);
        }
        assert!(scene.root_brep_display_pin(&stamp).unwrap().is_some());
        let camera = om9_scene::camera::Camera::orthographic([2., 2.5, 20.], [2., 2.5, 0.], [0., 1., 0.], 20.).unwrap();
        let viewport = om9_scene::camera::Viewport { origin: [0., 0.], size: [400., 400.] };
        let screen = camera.project([2., 2.5, 6.], viewport).unwrap().unwrap().screen;
        let pick = scene.pick_kind(screen, &camera, viewport, 2., om9_scene::scene::PickIntent::Select,
            om9_scene::scene::PickKind::Face).unwrap().unwrap();
        assert_eq!(pick.root, id);
        assert!(matches!(pick.subset, om9_scene::scene::Subelement::BrepFace(face) if brep.topology().faces.contains(&face)));
    }

    #[test]
    fn attested_dupedge_uses_registered_source_and_preserves_history() {
        use om9_core::evaluation::{evaluate_curve, Parameter};
        use om9_scene::scene::Subelement;
        for remesh in [false, true] {
        let (mut c, id) = controller_with_topology_proof();
        if remesh {
            c.request_topology_remesh_for(id, 0.02, 0.3,
                16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = c.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            assert_eq!(c.scene().unwrap().root_brep_pick_display(&c.document().stamp(id).unwrap()).unwrap().deflection_mm, 0.02);
        }
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!("B-rep") };
        let segment = brep.display().segments.iter().find(|s| {
            let a = brep.display().points[s[0] as usize];
            let b = brep.display().points[s[1] as usize];
            (a[1].abs() < 1e-8 && b[1].abs() < 1e-8)
                && (a[2] - 6.).abs() < 1e-8 && (b[2] - 6.).abs() < 1e-8
                && (a[0] - b[0]).abs() > 3.9
        }).unwrap();
        let edge = brep.topology().edges[segment[2] as usize];
        c.select_objects(&[id]).unwrap();
        c.select_subobject(id, Subelement::BrepEdge(edge)).unwrap();
        let before = c.document().clone();
        let undo = c.coordinator().undo_len();
        c.submit("DupEdge").unwrap();
        let result = c.document().entities.values().find(|entity| entity.id != id).unwrap();
        let Geometry::Curve(curve) = &result.geometry else { panic!("curve") };
        let endpoints: Vec<_> = [0., 1.].into_iter().map(|t| evaluate_curve(curve, Parameter::Fraction(t), 0,
            result.placement, c.coordinator().limits()).unwrap()[0]).collect();
        for expected in [[0., 0., 6.], [4., 0., 6.]] {
            assert!(endpoints.iter().any(|point| (0..3).all(|i| (point[i] - expected[i]).abs() < 1e-7)));
        }
        let after = c.document().clone();
        assert_eq!(c.coordinator().undo_len(), undo + 1);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
        }
    }

    #[test]
    fn ordinary_scene_pin_survives_document_and_releases_after_last_reader() {
        let limits = Limits::default();
        let body = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
        let live = body.display_liveness();
        let display_bytes = body.display_storage_bytes().unwrap();
        let entity = Entity::new("box", Geometry::Brep(body), None);
        let id = entity.id;
        let mut document = Document::with_template_tolerance("ordinary scene pin");
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        let stamp = controller.document().stamp(id).unwrap();
        assert!(controller.scene().unwrap().root_brep_display_pin(&stamp).unwrap().is_some());
        assert!(!controller.scene().unwrap().has_root_remesh_profile(&stamp, [0, 0]));
        controller.submit("Point").unwrap(); controller.submit("20,20,20").unwrap();
        controller.undo().unwrap(); controller.redo().unwrap();
        let stamp = controller.document().stamp(id).unwrap();
        let pin = controller.scene().unwrap().root_brep_display_pin(&stamp).unwrap().unwrap();
        let document = Arc::downgrade(&controller.scene().unwrap().document);
        let peer = controller.coordinator.memory_admission_peer().unwrap();
        drop(controller);
        assert!(document.upgrade().is_none(), "display pin must not retain document");
        assert!(live.is_alive());
        let held = peer.retained_usage().unwrap().bytes;
        drop(pin);
        assert!(!live.is_alive());
        assert_eq!(held - peer.retained_usage().unwrap().bytes, display_bytes);
    }

    #[test]
    #[ignore = "manual scene admission timing"]
    fn timing_ordinary_scene_unique_brep_pins() {
        let limits = Limits::default();
        let seed = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
        for count in [1usize, 16, 64] {
            let mut document = Document::with_template_tolerance("scene pin timing");
            for i in 0..count {
                let shape = om9_core::brep::Brep::from_native(seed.source().to_owned(), seed.display().clone(), &limits).unwrap();
                let mut entity = Entity::new("box", Geometry::Brep(shape), None);
                entity.placement = om9_core::geometry::Transform::translation([i as f64 * 5., 0., 0.]);
                document.entities.insert(entity.id, entity);
            }
            let mut samples = Vec::new();
            let mut retained = 0;
            for _ in 0..5 {
                let candidate = document.clone();
                let start = std::time::Instant::now();
                let controller = Controller::new(candidate, limits).unwrap();
                samples.push(start.elapsed().as_secs_f64() * 1000.);
                retained = controller.coordinator.retained_usage().unwrap().bytes;
            }
            eprintln!("unique_breps={count} scene_create_ms={samples:?} max_ms={} retained_managed_bytes={retained}",
                samples.iter().copied().fold(0f64, f64::max));
        }
    }

    #[test]
    fn candidate_scene_pin_admission_failure_keeps_world_and_redo() {
        use std::cell::Cell;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits::default();
        let mut controller = Controller::new(Document::with_template_tolerance("candidate pin refusal"), limits).unwrap();
        controller.submit("Point").unwrap(); controller.submit("10,10,10").unwrap(); controller.undo().unwrap();
        let before = controller.coordinator.snapshot();
        let publication = controller.scene().unwrap().publication_id();
        let body = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
        let entity = Entity::new("box", Geometry::Brep(body), None);
        let held = controller.scene_cache_usage().unwrap();
        let prepared = controller.coordinator.prepare_with_held_usage(
            TransactionRequest::new("box", vec![Mutation::Insert { document: controller.document, entity }]), held).unwrap();
        let after_preflight = Cell::new(false);
        let result = controller.commit_with_candidate_scene_guard(prepared, held, None,
            || Some((1024 * MIB, if after_preflight.get() { 0 } else { 1024 * MIB })),
            |_| { after_preflight.set(true); Ok(()) });
        assert!(matches!(result, Err(Error::Budget("available physical RAM"))));
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.coordinator.redo_len(), 1);
        assert_eq!(controller.scene().unwrap().publication_id(), publication);
    }

    #[test]
    fn controller_adopts_reserved_proof_and_drops_it_after_edit() {
        let (mut controller, id) = controller_with_topology_proof();
        assert!(controller.topology_proof_for(None).is_some());
        controller.undo().unwrap();
        assert!(controller.topology_proof_for(None).is_none());
        assert!(!controller.document().entities.contains_key(&id));
    }

    #[test]
    fn topology_proof_rebinds_after_unrelated_edit_but_not_after_deletion() {
        let (mut controller, id) = controller_with_topology_proof();
        let proof = controller.topology_proof_for(None).unwrap();
        controller.submit("Point").unwrap(); controller.submit("20,20,20").unwrap();
        assert!(proof.verify_unchanged_owner(&controller.coordinator).is_err());
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let held = controller.scene_cache_usage().unwrap();
        let mut scratch = controller.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache,
            1024 * 1024, physical, held).unwrap();
        let permit = proof.acquire_worker().unwrap();
        assert!(matches!(proof.rebind_unchanged(&controller.coordinator, |_, _| Ok(())), Err(Error::Busy)));
        drop(permit);
        proof.rebind_unchanged(&controller.coordinator, |live, pending| {
            if live.checked_add(pending).ok_or(Error::Overflow("test proof scratch"))? > scratch.bytes() {
                return Err(Error::Budget("test proof scratch"));
            }
            scratch.set_allocated_bytes(live)?;
            scratch.recheck_pending()
        }).unwrap();
        assert!(proof.verify_unchanged_owner(&controller.coordinator).is_ok());
        let stamp = controller.document().stamp(id).unwrap();
        let request = TransactionRequest::new("delete", vec![Mutation::Delete { source: stamp }]);
        let prepared = controller.coordinator.prepare(request).unwrap();
        controller.coordinator.commit(prepared).unwrap();
        assert!(proof.rebind_unchanged(&controller.coordinator, |_, _| Ok(())).is_err());
    }

    #[test]
    fn topology_remesh_rebinds_sphere_after_placement_and_undo() {
        let mut controller = Controller::new(Document::with_template_tolerance("sphere proof move"), Limits::default()).unwrap();
        controller.commit_primitive_optional_topology("OM9-SOLID-014", "sphere", "sphere",
            om9_kernel::creation::Primitive::Sphere { radius_mm: 5. }, om9_core::geometry::Transform::identity()).unwrap();
        let id = controller.topology_proof_for(None).unwrap().stamp().object;
        let proof = controller.topology_proof_for(Some(id)).unwrap();
        controller.select_objects(&[id]).unwrap();
        let mut old_job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let old_receipt = wait_remesh(&mut old_job).unwrap();
        controller.numeric_move([7., -2., 3.]).unwrap();
        assert!(old_receipt.validate_current(&controller).is_err());
        assert!(controller.topology_proof_for(Some(id)).is_some(), "placement-only edit must retain proof for revalidation");
        let before_refusal = controller.document().clone();
        let publication = controller.scene().unwrap().publication_id();
        assert!(matches!(proof.rebind_unchanged(&controller.coordinator, |_, _| Err(Error::Budget("test rebind"))),
            Err(Error::Budget("test rebind"))));
        assert_eq!(controller.document(), &before_refusal);
        assert_eq!(controller.scene().unwrap().publication_id(), publication);
        assert!(old_receipt.validate_current(&controller).is_err());
        let mut moved_job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let moved = wait_remesh(&mut moved_job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        moved.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        assert!(old_receipt.validate_current(&controller).is_err(), "rebinding must not revive the old receipt");
        assert_eq!(controller.document().entities[&id].placement, om9_core::geometry::Transform::translation([7., -2., 3.]));
        controller.undo().unwrap();
        assert!(controller.topology_proof_for(Some(id)).is_some(), "Undo must retain proof for revalidation");
        let redo_count = controller.coordinator.redo_len();
        assert!(matches!(proof.rebind_unchanged(&controller.coordinator, |_, _| Err(Error::Budget("test rebind"))),
            Err(Error::Budget("test rebind"))));
        assert_eq!(controller.coordinator.redo_len(), redo_count);
        let mut undone_job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let undone = wait_remesh(&mut undone_job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        undone.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        assert_eq!(controller.document().entities[&id].placement, om9_core::geometry::Transform::identity());
        controller.redo().unwrap();
        let mut redone_job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let redone = wait_remesh(&mut redone_job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        redone.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        assert_eq!(controller.document().entities[&id].placement, om9_core::geometry::Transform::translation([7., -2., 3.]));
        let replacement = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 6. },
            om9_kernel::creation::DEFAULT_DEFLECTION_MM, controller.coordinator.limits()).unwrap();
        let stamp = controller.document().stamp(id).unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let prepared = controller.coordinator.prepare_with_held_usage(TransactionRequest::new("replace sphere", vec![
            Mutation::Geometry { source: stamp, geometry: Geometry::Brep(replacement) },
        ]), held).unwrap();
        controller.commit_with_candidate_scene(prepared, held).unwrap();
        assert!(controller.topology_proof_for(Some(id)).is_none(), "changed source must not use the old proof");
        assert!(controller.topology_source_proof_for(Some(id)).is_none(), "changed canonical source cannot use the old attestation");
        assert!(proof.rebind_unchanged(&controller.coordinator, |_, _| Ok(())).is_err(), "held old proof must reject changed source");
        assert!(proof.rebind_source_unchanged(&controller.coordinator, |_, _| Ok(())).is_err(), "source-only rebind must reject changed source");
        controller.undo().unwrap();
        assert!(controller.topology_proof_for(Some(id)).is_some(), "Undo should recover the matching cached proof");
        let mut restored_job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let restored = wait_remesh(&mut restored_job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        restored.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        controller.redo().unwrap();
        assert!(controller.topology_proof_for(Some(id)).is_none(), "Redo must not reuse the old source proof");
    }

    #[test]
    fn topology_cache_tracks_two_geometry_versions_of_one_sphere() {
        let mut controller = Controller::new(Document::with_template_tolerance("sphere versions"), Limits::default()).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let original = controller.topology_proof_for(None).unwrap();
        let id = original.stamp().object;
        let limits = *controller.coordinator.limits();
        let snapshot = om9_kernel::creation::primitive_with_topology(
            om9_kernel::creation::Primitive::Sphere { radius_mm: 6. }, 0.1, &limits, |_, _| Ok(())).unwrap();
        let (replacement, replacement_proof) = snapshot.into_parts();
        let held = controller.scene_cache_usage().unwrap();
        let stamp = controller.document().stamp(id).unwrap();
        let prepared = controller.coordinator.prepare_with_held_usage(TransactionRequest::new("replace sphere", vec![
            Mutation::Geometry { source: stamp, geometry: Geometry::Brep(replacement) },
        ]), held).unwrap();
        controller.commit_with_candidate_scene(prepared, held).unwrap();
        assert!(controller.topology_proof_for(Some(id)).is_none());
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let bytes = replacement_proof.storage_bytes().unwrap();
        let mut reservation = controller.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, held).unwrap();
        reservation.set_allocated_bytes(bytes).unwrap();
        controller.adopt_topology_proof(controller.document().stamp(id).unwrap(), replacement_proof, reservation).unwrap();
        let newer = controller.topology_proof_for(Some(id)).unwrap();
        assert!(!Arc::ptr_eq(&original, &newer));
        controller.undo().unwrap();
        assert!(Arc::ptr_eq(&controller.topology_proof_for(Some(id)).unwrap(), &original));
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        wait_remesh(&mut job).unwrap();
        controller.redo().unwrap();
        assert!(Arc::ptr_eq(&controller.topology_proof_for(Some(id)).unwrap(), &newer));
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        wait_remesh(&mut job).unwrap();
        let old_owner = Arc::downgrade(&original);
        drop(original);
        controller.coordinator.reclaim_optional_memory(u64::MAX).unwrap();
        assert!(old_owner.upgrade().is_some(), "the Sphere insertion history still retains the original proof");
        assert!(Arc::ptr_eq(&controller.topology_proof_for(Some(id)).unwrap(), &newer));
        controller.undo().unwrap();
        assert!(controller.document().entities.contains_key(&id));
        assert!(controller.topology_proof_for(Some(id)).is_some(), "Undo can reuse the history-retained proof");
    }

    fn start_remesh(controller: &mut Controller, bytes: u64) -> TopologyRemeshJob {
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let reservation = controller.coordinator.reserve_external_memory_with_held(
            MemoryClass::Kernel, bytes, physical, held).unwrap();
        controller.start_topology_remesh(0.02, 0.3, reservation).unwrap()
    }

    fn wait_remesh(job: &mut TopologyRemeshJob) -> Result<TopologyRemeshReceipt> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = job.try_result() { return result; }
            assert!(std::time::Instant::now() < deadline, "remesh worker timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    #[test]
    fn topology_remesh_accepts_matching_candidate() {
        let (mut controller, id) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let receipt = wait_remesh(&mut job).unwrap();
        let candidate = receipt.candidate(&controller).unwrap();
        let Geometry::Brep(original) = &controller.document().entities[&id].geometry else { panic!("expected brep") };
        assert_eq!(candidate.source(), original.source());
        assert_eq!(candidate.topology().faces, original.topology().faces);
        assert_eq!(candidate.topology().edges, original.topology().edges);
        assert_eq!(candidate.topology().vertex_ids, original.topology().vertex_ids);
        assert_eq!(controller.coordinator.undo_len(), 1);
    }

    #[test]
    fn topology_remesh_releases_unused_kernel_reservation_after_worker() {
        let (mut controller, _) = controller_with_topology_proof();
        let requested = 16 * 1024 * 1024;
        let mut job = start_remesh(&mut controller, requested);
        let receipt = wait_remesh(&mut job).unwrap();
        let (reserved, retained) = receipt.retained_kernel_bytes().unwrap();
        assert!(retained > 0 && retained < requested);
        assert_eq!(reserved, retained, "the live receipt should retain its actual candidate bytes");
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), reserved);
        drop(receipt);
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn published_remesh_releases_candidate_source_but_keeps_display_charge() {
        let (mut controller, _) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let receipt = wait_remesh(&mut job).unwrap();
        let (reserved, _) = receipt.retained_kernel_bytes().unwrap();
        let candidate = receipt.candidate(&controller).unwrap().clone();
        let display_bytes = candidate.display_storage_bytes().unwrap();
        let display = candidate.display_liveness();
        let source = candidate.source_liveness();
        drop(candidate);
        let display_candidate = receipt.prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        let scene = display_candidate.prepare_scene(&controller, 8 * 1024 * 1024).unwrap();
        assert!(source.is_alive());
        scene.publish(&mut controller).unwrap();
        assert!(!source.is_alive(), "published scene should not retain the remesh source copy");
        assert!(display.is_alive(), "scene still owns the remeshed display");
        assert!(display_bytes < reserved);
        assert!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel) >= display_bytes);
        controller.undo().unwrap();
        assert!(!display.is_alive());
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn detached_render_buffer_does_not_retain_remesh_display_charge() {
        let (mut controller, _) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let receipt = wait_remesh(&mut job).unwrap();
        let display = receipt.candidate(&controller).unwrap().display_liveness();
        receipt.prepare_display(&controller, 8 * 1024 * 1024).unwrap()
            .prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        let buffers = controller.scene().unwrap().objects[0].buffers.clone();
        assert!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel) > 0);
        controller.undo().unwrap();
        assert!(!display.is_alive());
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), 0,
            "a buffer-only clone must not retain the display reservation");
        assert!(matches!(buffers.as_ref(), RenderBuffers::Mesh { triangles, .. } if !triangles.is_empty()));
        let with_buffer = controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache);
        assert!(with_buffer > 0);
        drop(buffers);
        assert!(controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache) < with_buffer,
            "the buffer reservation must remain until its last clone is dropped");
    }

    #[test]
    fn detached_remesh_object_keeps_display_and_its_charge_together() {
        let (mut controller, _) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let receipt = wait_remesh(&mut job).unwrap();
        let display = receipt.candidate(&controller).unwrap().display_liveness();
        receipt.prepare_display(&controller, 8 * 1024 * 1024).unwrap()
            .prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        let object = controller.scene().unwrap().objects[0].clone();
        let second = object.clone();
        controller.undo().unwrap();
        assert!(display.is_alive());
        let retained = controller.coordinator.external_reserved_bytes(MemoryClass::Kernel);
        assert!(retained > 0, "the detached display must retain its reservation");
        drop(object);
        assert!(display.is_alive());
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), retained);
        drop(second);
        assert!(!display.is_alive());
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn dropping_prepared_remesh_scene_keeps_current_scene_and_releases_candidate() {
        let (mut controller, _) = controller_with_topology_proof();
        let publication = controller.scene().unwrap().publication_id();
        let before = controller.coordinator.snapshot();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let receipt = wait_remesh(&mut job).unwrap();
        let candidate = receipt.candidate(&controller).unwrap().clone();
        let display = candidate.display_liveness();
        let source = candidate.source_liveness();
        drop(candidate);
        let scene = receipt.prepare_display(&controller, 8 * 1024 * 1024).unwrap()
            .prepare_scene(&controller, 8 * 1024 * 1024).unwrap();
        assert!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel) > 0);
        drop(scene);
        assert!(!source.is_alive());
        assert!(!display.is_alive());
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
        assert_eq!(controller.scene().unwrap().publication_id(), publication);
        assert_eq!(controller.coordinator.snapshot(), before);
    }

    #[test]
    fn canonical_record_inspects_source_without_using_display() {
        let (controller, id) = controller_with_topology_proof();
        let stamp = controller.document().stamp(id).unwrap();
        let source = controller.coordinator.capture_brep_source(&stamp).unwrap();
        let encoded = serde_json::to_vec(&source.canonical_record()).unwrap();
        let limits = *controller.coordinator.limits();
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let reservation = controller.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache,
            16 * 1024 * 1024, physical, held).unwrap();
        let record = om9_archive::canonical_brep::decode_reserved(&encoded, &limits,
            &controller.coordinator.external_memory_account(), reservation, || false).unwrap();
        let counts = om9_kernel::brep::inspect_canonical_record(record.record(), &limits, |_, _| Ok(())).unwrap();
        assert_eq!(counts, [record.record().topology().faces.len() as u64,
            record.record().topology().edges.len() as u64, record.record().topology().vertex_ids.len() as u64]);
        // Count agreement cannot attest correspondence between UUIDs and native indices.
        let mut wrong: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        wrong["topology"]["faces"].as_array_mut().unwrap().push(serde_json::json!(uuid::Uuid::new_v4()));
        let topology: om9_core::brep::BrepTopology = serde_json::from_value(wrong["topology"].clone()).unwrap();
        wrong["binding_sha256"] = serde_json::json!(om9_core::brep::source_topology_fingerprint(record.record().fingerprint(), &topology));
        let wrong = serde_json::to_vec(&wrong).unwrap();
        let reservation = controller.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache,
            16 * 1024 * 1024, physical, held).unwrap();
        let wrong = om9_archive::canonical_brep::decode_reserved(&wrong, &limits,
            &controller.coordinator.external_memory_account(), reservation, || false).unwrap();
        assert!(matches!(om9_kernel::brep::inspect_canonical_record(wrong.record(), &limits, |_, _| Ok(())),
            Err(om9_kernel::KernelError::InvalidInput(message)) if message.contains("count mismatch")));
    }

    #[test]
    fn topology_remesh_rejects_result_after_undo() {
        let (mut controller, _) = controller_with_topology_proof();
        let before = Arc::strong_count(&controller.scene().unwrap().document);
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        assert_eq!(Arc::strong_count(&controller.scene().unwrap().document), before,
            "worker must not add an owner of the document");
        controller.undo().unwrap();
        let receipt = wait_remesh(&mut job).unwrap();
        assert!(receipt.candidate(&controller).is_err());
        assert!(controller.document().entities.is_empty());
    }

    #[test]
    fn topology_remesh_rejects_same_document_scene_replacement() {
        let (mut controller, id) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let epoch = controller.scene().unwrap().epoch;
        controller.replace_scene_same_epoch_for_test(0.).unwrap();
        assert_eq!(controller.scene().unwrap().epoch, epoch);
        let receipt = wait_remesh(&mut job).unwrap();
        assert!(matches!(receipt.validate_current(&controller), Err(Error::Stale("topology remesh scene"))));
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
    }

    #[test]
    fn topology_remesh_cancel_keeps_document_and_history() {
        let (mut controller, id) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        job.cancel();
        match wait_remesh(&mut job) {
            Err(Error::Canceled) => {},
            Ok(receipt) => assert!(matches!(receipt.validate_current(&controller), Err(Error::Canceled))),
            Err(other) => panic!("unexpected error: {other:?}"),
        }
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
    }

    #[test]
    fn topology_remesh_refuses_insufficient_reservation() {
        let (mut controller, id) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 1);
        assert!(wait_remesh(&mut job).is_err());
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
    }

    #[test]
    fn topology_remesh_prepares_accounted_display_without_changing_document() {
        let (mut controller, id) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let receipt = wait_remesh(&mut job).unwrap();
        let before = controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache);
        let prepared = receipt.prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        let shape = prepared.candidate(&controller).unwrap();
        let RenderBuffers::Mesh { vertices, triangles, edges } = prepared.buffers() else { panic!("expected mesh") };
        assert_eq!(&vertices[..shape.display().points.len()], &shape.display().points);
        assert!(vertices.len() > shape.display().points.len());
        assert_eq!(triangles.len(), shape.display().triangles.len());
        assert!(edges.len() > shape.display().segments.len());
        assert!(triangles.iter().zip(&shape.display().triangles).all(|(a, b)| *a == ([b[0], b[1], b[2]], b[3])));
        assert!(edges.iter().zip(&shape.display().segments).all(|(a, b)| *a == [b[0], b[1]]));
        assert!(controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache) > before);
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
        assert!(prepared.validate_current(&controller).is_ok());
        controller.replace_scene_same_epoch_for_test(0.).unwrap();
        assert!(matches!(prepared.validate_current(&controller), Err(Error::Stale("topology remesh scene"))));
        drop(prepared);
        assert!(controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache) < before);
    }

    #[test]
    fn topology_remesh_preparation_refuses_small_budget_without_edit() {
        let (mut controller, id) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let receipt = wait_remesh(&mut job).unwrap();
        assert!(matches!(receipt.prepare_display(&controller, 1), Err(Error::Budget(_))));
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
    }

    #[test]
    fn topology_remesh_private_scene_picks_occt_face_and_preserves_document() {
        use om9_scene::{camera::{Camera, Viewport}, scene::{PickIntent, PickKind, Subelement}};
        let (mut controller, id) = controller_with_topology_proof();
        let old_scene = controller.scene().unwrap().publication_id();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let receipt = wait_remesh(&mut job).unwrap();
        let prepared = receipt.prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        let shape = prepared.candidate(&controller).unwrap();
        let triangle = shape.display().triangles.iter().find(|tri| tri[..3].iter().all(|i| (shape.display().points[*i as usize][2] - 6.).abs() < 1e-9)).unwrap();
        let expected_face = shape.topology().faces[triangle[3] as usize];
        let prepared = prepared.prepare_scene(&controller, 8 * 1024 * 1024).unwrap();
        assert!(prepared.validate_current(&controller).is_ok());
        assert_ne!(prepared.scene().publication_id(), old_scene);
        assert_eq!(controller.scene().unwrap().publication_id(), old_scene);
        let camera = Camera::orthographic([2., 2.5, 20.], [2., 2.5, 0.], [0., 1., 0.], 10.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [800., 800.] };
        let cursor = camera.project([2., 2.5, 6.], viewport).unwrap().unwrap().screen;
        let hit = prepared.scene().pick_kind(cursor, &camera, viewport, 1., PickIntent::Select, PickKind::Face).unwrap().unwrap();
        assert_eq!(hit.subset, Subelement::BrepFace(expected_face));
        let candidates = om9_scene::osnap::candidates(prepared.scene(), &camera, viewport, cursor, Default::default(), None).unwrap();
        assert!(candidates.iter().any(|candidate| candidate.surface_source.as_ref().is_some_and(|source| source.face == Some(expected_face))));
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
        controller.replace_scene_same_epoch_for_test(0.).unwrap();
        assert!(matches!(prepared.validate_current(&controller), Err(Error::Stale("topology remesh scene"))));
    }

    #[test]
    fn topology_remesh_private_scene_refuses_small_metadata_budget() {
        let (mut controller, id) = controller_with_topology_proof();
        let old_scene = controller.scene().unwrap().publication_id();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let receipt = wait_remesh(&mut job).unwrap();
        let display = receipt.prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        assert!(matches!(display.prepare_scene(&controller, 1), Err(Error::Budget(_))));
        assert_eq!(controller.scene().unwrap().publication_id(), old_scene);
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
        let mut retry = start_remesh(&mut controller, 16 * 1024 * 1024);
        assert!(wait_remesh(&mut retry).is_ok(), "budget failure must retain reusable proof");
    }

    #[test]
    fn topology_remesh_scene_snapshot_retains_candidate_reservation() {
        let (mut controller, _) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let display = wait_remesh(&mut job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        let prepared = display.prepare_scene(&controller, 8 * 1024 * 1024).unwrap();
        let snapshot = prepared.scene().try_render_snapshot().unwrap();
        let kernel = controller.coordinator.external_reserved_bytes(MemoryClass::Kernel);
        assert!(kernel > 0);
        drop(prepared);
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), kernel);
        drop(snapshot);
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn topology_remesh_recomputes_gumball_pivot_for_new_scene() {
        use om9_scene::scene::RenderBuffers;
        let mut controller = Controller::new(Document::with_template_tolerance("sphere pivot"), Limits::default()).unwrap();
        controller.commit_primitive_optional_topology("OM9-SOLID-014", "sphere", "sphere",
            om9_kernel::creation::Primitive::Sphere { radius_mm: 5. }, om9_core::geometry::Transform::identity()).unwrap();
        let id = controller.topology_proof_for(None).unwrap().stamp().object;
        controller.select_objects(&[id]).unwrap();
        let before = controller.gumball_pivot().unwrap().origin;
        let old_epoch = controller.scene().unwrap().epoch;
        let old_publication = controller.scene().unwrap().publication_id();
        let reservation = controller.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 16 * 1024 * 1024,
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)), controller.scene_cache_usage().unwrap()).unwrap();
        let mut job = controller.start_topology_remesh(0.5, 0.9, reservation).unwrap();
        let display = wait_remesh(&mut job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        display.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        let scene = controller.scene().unwrap();
        assert_eq!(scene.epoch, old_epoch);
        assert_ne!(scene.publication_id(), old_publication);
        let object = scene.objects.iter().find(|object| object.root == id).unwrap();
        let RenderBuffers::Mesh { vertices, .. } = object.buffers.as_ref() else { panic!("sphere mesh") };
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for point in vertices {
            let point = object.display_transform().unwrap().point(*point).unwrap();
            for i in 0..3 { min[i] = min[i].min(point[i]); max[i] = max[i].max(point[i]); }
        }
        let expected: [f64; 3] = std::array::from_fn(|i| min[i] * 0.5 + max[i] * 0.5);
        assert!(before.iter().zip(expected).any(|(a, b)| (a - b).abs() > 1e-6), "fixture must change the mesh centre");
        assert_eq!(controller.gumball_pivot().unwrap().origin, expected);
    }

    #[test]
    fn topology_remesh_releases_replaced_scene_buffer_lease_after_last_snapshot() {
        let (mut controller, _) = controller_with_topology_proof();
        let scene = controller.scene().unwrap();
        let original = controller.share_scene_buffer_owner(scene).unwrap();
        let weak = Arc::downgrade(&original);
        let snapshot = scene.try_render_snapshot().unwrap();
        drop(original);
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let display = wait_remesh(&mut job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        display.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        assert_eq!(controller.scene().unwrap().shared_buffer_bytes(), 0, "one root was fully replaced");
        assert!(weak.upgrade().is_some(), "old snapshot must keep its buffer charge");
        drop(snapshot);
        assert!(weak.upgrade().is_none(), "replaced buffers and their charge must be released");
    }

    #[test]
    fn topology_remesh_keeps_original_lease_while_another_root_uses_it() {
        let (mut controller, first, second) = controller_with_two_topology_proofs();
        let scene = controller.scene().unwrap();
        let original = controller.share_scene_buffer_owner(scene).unwrap();
        let weak = Arc::downgrade(&original);
        let snapshot = scene.try_render_snapshot().unwrap();
        drop(original);
        controller.request_topology_remesh_for(first, 0.02, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let scene = controller.scene().unwrap();
        assert!(scene.shared_buffer_bytes() > 0, "second root still uses original buffers");
        assert!(scene.objects.iter().any(|object| object.root == second));
        assert!(weak.upgrade().is_some(), "remaining buffer must keep its charge");
        controller.request_topology_remesh_for(second, 0.03, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "second remesh timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(controller.scene().unwrap().shared_buffer_bytes(), 0, "both original roots were replaced");
        assert!(weak.upgrade().is_some(), "the original snapshot must retain its charge");
        drop(snapshot);
        assert!(weak.upgrade().is_none(), "replacement owners must not pin an unused original lease");
        drop(controller);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn topology_remesh_publishes_cpu_scene_and_undo_remains_valid() {
        use om9_scene::{camera::{Camera, Viewport}, scene::{PickIntent, PickKind, Subelement}};
        let (mut controller, id) = controller_with_topology_proof();
        let old_scene = controller.scene().unwrap().publication_id();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let display = wait_remesh(&mut job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        let shape = display.candidate(&controller).unwrap();
        let triangle = shape.display().triangles.iter().find(|tri| tri[..3].iter().all(|i| (shape.display().points[*i as usize][2] - 6.).abs() < 1e-9)).unwrap();
        let expected_face = shape.topology().faces[triangle[3] as usize];
        let prepared = display.prepare_scene(&controller, 8 * 1024 * 1024).unwrap();
        prepared.publish(&mut controller).unwrap();
        assert_ne!(controller.scene().unwrap().publication_id(), old_scene);
        controller.coordinator.reclaim_optional_memory(u64::MAX).unwrap();
        assert!(controller.topology_proof_for(Some(id)).is_some(), "published scene pins its proof");
        assert!(controller.scene_cache_usage().is_ok());
        let camera = Camera::orthographic([2., 2.5, 20.], [2., 2.5, 0.], [0., 1., 0.], 10.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [800., 800.] };
        let cursor = camera.project([2., 2.5, 6.], viewport).unwrap().unwrap().screen;
        let hit = controller.scene().unwrap().pick_kind(cursor, &camera, viewport, 1., PickIntent::Select, PickKind::Face).unwrap().unwrap();
        assert_eq!(hit.subset, Subelement::BrepFace(expected_face));
        let current = controller.scene().unwrap();
        let updated = current.update_owned(Arc::clone(&current.document), current.epoch, None, controller.coordinator.limits()).unwrap();
        let updated_hit = updated.pick_kind(cursor, &camera, viewport, 1., PickIntent::Select, PickKind::Face).unwrap().unwrap();
        assert_eq!(updated_hit.subset, Subelement::BrepFace(expected_face));
        drop(updated);
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
        controller.undo().unwrap();
        assert!(!controller.document().entities.contains_key(&id));
        controller.redo().unwrap();
        assert!(controller.document().entities.contains_key(&id));
    }

    #[test]
    fn topology_remesh_replaces_same_root_twice_and_retains_old_snapshot() {
        use om9_scene::{camera::{Camera, Viewport}, scene::{PickIntent, PickKind, Subelement}};
        let (mut controller, id) = controller_with_topology_proof();
        let camera = Camera::orthographic([2., 2.5, 20.], [2., 2.5, 0.], [0., 1., 0.], 10.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [800., 800.] };
        let cursor = camera.project([2., 2.5, 6.], viewport).unwrap().unwrap().screen;
        let mut first = start_remesh(&mut controller, 16 * 1024 * 1024);
        let display = wait_remesh(&mut first).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        display.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        let old = controller.scene().unwrap().try_render_snapshot().unwrap();
        let old_id = old.publication_id();
        let old_hit = old.pick_kind(cursor, &camera, viewport, 1., PickIntent::Select, PickKind::Face).unwrap().unwrap();
        assert!(matches!(old_hit.subset, Subelement::BrepFace(_)));
        let one_kernel = controller.coordinator.external_reserved_bytes(MemoryClass::Kernel);
        let mut second = start_remesh(&mut controller, 16 * 1024 * 1024);
        let display = wait_remesh(&mut second).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        display.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        assert_ne!(controller.scene().unwrap().publication_id(), old_id);
        let new_hit = controller.scene().unwrap().pick_kind(cursor, &camera, viewport, 1., PickIntent::Select, PickKind::Face).unwrap().unwrap();
        assert_eq!(new_hit.subset, old_hit.subset);
        assert_eq!(old.pick_kind(cursor, &camera, viewport, 1., PickIntent::Select, PickKind::Face).unwrap().unwrap().subset, old_hit.subset);
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), 2 * one_kernel);
        drop(old);
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), one_kernel);
        assert!(controller.document().entities.contains_key(&id));
        controller.undo().unwrap();
        assert!(!controller.document().entities.contains_key(&id));
    }

    fn controller_with_two_topology_proofs() -> (Controller, om9_core::identity::ObjectId, om9_core::identity::ObjectId) {
        use om9_kernel::creation::{primitive_with_topology, Primitive};
        let limits = Limits::default();
        let a = primitive_with_topology(Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits, |_, _| Ok(())).unwrap();
        let b = primitive_with_topology(Primitive::Sphere { radius_mm: 3. }, 0.1, &limits, |_, _| Ok(())).unwrap();
        let (a, proof_a) = a.into_parts(); let (b, proof_b) = b.into_parts();
        let mut document = Document::with_template_tolerance("two attested roots");
        let first = Entity::new("box", Geometry::Brep(a), None);
        let mut second = Entity::new("sphere", Geometry::Brep(b), None);
        second.placement = om9_core::geometry::Transform::translation([10., 0., 0.]);
        let (first_id, second_id) = (first.id, second.id);
        document.entities.insert(first_id, first); document.entities.insert(second_id, second);
        let mut controller = Controller::new(document, limits).unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        for (id, proof) in [(first_id, proof_a), (second_id, proof_b)] {
            let bytes = proof.storage_bytes().unwrap();
            let held = controller.scene_cache_usage().unwrap();
            let mut reservation = controller.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, held).unwrap();
            reservation.set_allocated_bytes(bytes).unwrap();
            controller.adopt_topology_proof(controller.document().stamp(id).unwrap(), proof, reservation).unwrap();
        }
        (controller, first_id, second_id)
    }

    #[test]
    fn boolean_output_profile_comes_from_scene_and_survives_remesh() {
        for background in [false, true] {
            let (mut controller, box_id, sphere_id) = controller_with_two_topology_proofs();
            let before = controller.document().clone();
            controller.request_topology_remesh_for(box_id, 0.02, 0.3,
                16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let stamp = controller.document().stamp(box_id).unwrap();
            assert_eq!(controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().deflection_mm, 0.02);
            assert_eq!(controller.scene().unwrap().root_brep_output_deflection(&stamp).unwrap(), 0.1);
            assert_eq!(controller.document(), &before);
            if background { controller.enable_background_boolean_union(); }
            controller.select_objects(&[box_id, sphere_id]).unwrap();
            controller.submit("BooleanUnion").unwrap(); controller.submit("DeleteInput=Yes").unwrap(); controller.submit("").unwrap();
            if background {
                assert!(controller.boolean_union_worker_pending());
                loop {
                    if let Some(result) = controller.poll_boolean_union_worker() { result.unwrap(); break; }
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }
            let result = controller.document().entities.values().next().unwrap();
            let Geometry::Brep(body) = &result.geometry else { panic!("Boolean result") };
            assert_eq!(body.display().deflection_mm, 0.1);
            let volume = om9_kernel::creation::measure(body, controller.coordinator().limits()).unwrap().volume_mm3;
            assert!((volume - (120. + 36. * std::f64::consts::PI)).abs() < 1e-5, "{volume}");
            controller.undo().unwrap(); assert_eq!(controller.document(), &before);
        }
    }

    #[test]
    fn topology_proof_reuses_exact_source_lease_and_rejects_other_root() {
        let (controller, box_id, sphere_id) = controller_with_two_topology_proofs();
        let proof = controller.topology_proof_for(Some(box_id)).unwrap();
        let Geometry::Brep(shape) = &controller.document().entities[&box_id].geometry else { panic!("BRep") };
        let display = shape.display_liveness();
        let source = shape.source_liveness();
        let first = proof.capture_source(controller.coordinator()).unwrap().into_source();
        let second = proof.capture_source(controller.coordinator()).unwrap();
        assert!(second.shares_source_lease(&first), "repeated workers reuse the attested source owner");
        let distinct = controller.coordinator().capture_brep_source(&controller.document().stamp(box_id).unwrap()).unwrap();
        assert!(!distinct.same_lease(&first));
        assert!(distinct.same_canonical_allocations(&first));
        assert!(proof.bind_captured_source(controller.coordinator(), distinct).is_ok());
        let other = controller.coordinator().capture_brep_source(&controller.document().stamp(sphere_id).unwrap()).unwrap();
        assert!(matches!(proof.bind_captured_source(controller.coordinator(), other), Err(om9_core::Error::Stale(_))));
        assert_eq!(controller.document().entities.len(), 2);
        drop(first); drop(second); drop(controller);
        assert!(!display.is_alive(), "proof must not retain document display");
        assert!(source.is_alive(), "proof retains its canonical source lease");
        drop(proof);
        assert!(!source.is_alive(), "the final proof releases its canonical source");
    }

    #[test]
    fn topology_proof_rebinds_shared_source_without_display_or_scratch() {
        let (mut controller, box_id, _) = controller_with_two_topology_proofs();
        let proof = controller.topology_proof_for(Some(box_id)).unwrap();
        controller.select_objects(&[box_id]).unwrap();
        controller.numeric_move([3., 0., 0.]).unwrap();
        let before = controller.coordinator.snapshot();
        let permit = proof.acquire_worker().unwrap();
        assert!(matches!(proof.rebind_shared_source(controller.coordinator()), Err(Error::Busy)));
        drop(permit);
        assert!(proof.rebind_shared_source(controller.coordinator()).unwrap());
        assert!(proof.verify_unchanged_source(controller.coordinator()).is_ok());
        assert!(proof.verify_unchanged_owner(controller.coordinator()).is_err());
        assert!(controller.coordinator.snapshot() == before, "source proof rebind must not change document or history");
        controller.request_topology_remesh_for(box_id, 0.05, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(controller.document().entities[&box_id].placement, om9_core::geometry::Transform::translation([3., 0., 0.]));
    }

    #[test]
    fn remesh_rebinds_attested_source_without_trusting_replaced_display() {
        let (mut controller, box_id, _) = controller_with_two_topology_proofs();
        let proof = controller.topology_proof_for(Some(box_id)).unwrap();
        let Geometry::Brep(original) = &controller.document().entities[&box_id].geometry else { panic!("BRep") };
        let ids = original.topology().clone();
        let mut changed_display = original.display().clone();
        changed_display.deflection_mm *= 2.0;
        let replacement = om9_core::brep::Brep::from_native(original.source().to_owned(), changed_display,
            controller.coordinator.limits()).unwrap();
        let stamp = controller.document().stamp(box_id).unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let prepared = controller.coordinator.prepare_with_held_usage(TransactionRequest::new("replace display fixture", vec![
            Mutation::Geometry { source: stamp, geometry: Geometry::Brep(replacement) },
        ]), held).unwrap();
        controller.commit_with_candidate_scene(prepared, held).unwrap();
        assert!(controller.topology_proof_for(Some(box_id)).is_none(), "old display proof is not valid for replacement");
        assert!(controller.topology_source_proof_for(Some(box_id)).is_some());
        assert!(!proof.rebind_shared_source(controller.coordinator()).unwrap(), "independent source owner needs full validation");
        assert!(proof.rebind_unchanged(controller.coordinator(), |_, _| Ok(())).is_err());
        let before = controller.coordinator.snapshot();
        let world_before = controller.document().clone();
        let counts_before = (controller.coordinator.undo_len(), controller.coordinator.redo_len(), controller.coordinator.epoch());
        let publication = controller.scene().unwrap().publication_id();
        assert!(matches!(proof.rebind_source_unchanged(controller.coordinator(), |_, _| Err(Error::Budget("source scratch"))),
            Err(Error::Budget("source scratch"))));
        assert!(controller.coordinator.snapshot() == before, "refusal must preserve document and history");
        assert_eq!(controller.scene().unwrap().publication_id(), publication);
        controller.request_topology_remesh_for(box_id, 0.05, 0.3, 1, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { assert!(matches!(result, Err(Error::Budget(_)))); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(controller.coordinator.snapshot() == before, "worker budget refusal must preserve document and history");
        assert_eq!(controller.scene().unwrap().publication_id(), publication);
        assert!(proof.verify_unchanged_source(controller.coordinator()).is_err());
        controller.request_topology_remesh_for(box_id, 0.05, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let topology_remesh::PendingTopologyRemesh::Worker { mut job, scene_reservation, .. } =
            controller.topology_remesh_job.take().expect("worker job") else { panic!("worker job") };
        let canceled = wait_remesh(&mut job).unwrap();
        job.cancel();
        assert!(matches!(canceled.validate_current(&controller), Err(Error::Canceled)));
        assert!(proof.verify_unchanged_source(controller.coordinator()).is_err());
        drop(canceled); drop(scene_reservation);
        controller.request_topology_remesh_for(box_id, 0.05, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let topology_remesh::PendingTopologyRemesh::Worker { mut job, scene_reservation, .. } =
            controller.topology_remesh_job.take().expect("worker job") else { panic!("worker job") };
        let stale = wait_remesh(&mut job).unwrap();
        controller.replace_scene_same_epoch_for_test(0.).unwrap();
        assert!(matches!(stale.validate_current(&controller), Err(Error::Stale(_))));
        assert!(proof.verify_unchanged_source(controller.coordinator()).is_err());
        drop(stale); drop(scene_reservation);
        controller.request_topology_remesh_for(box_id, 0.05, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let topology_remesh::PendingTopologyRemesh::Worker { mut job, display_budget, scene_reservation } =
            controller.topology_remesh_job.take().expect("worker job") else { panic!("worker job") };
        let receipt = wait_remesh(&mut job).unwrap();
        assert_ne!(receipt.rebind_thread().expect("independent source rebind"), std::thread::current().id());
        assert!(proof.verify_unchanged_source(controller.coordinator()).is_err(), "worker must not publish the binding");
        receipt.validate_current(&controller).unwrap();
        drop(scene_reservation);
        receipt.prepare_display(&controller, display_budget).unwrap()
            .prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        assert!(controller.document() == &world_before, "remesh must not change document");
        assert_eq!((controller.coordinator.undo_len(), controller.coordinator.redo_len(), controller.coordinator.epoch()), counts_before);
        assert!(proof.verify_unchanged_source(controller.coordinator()).is_ok());
        assert!(proof.verify_unchanged_owner(controller.coordinator()).is_err(), "source binding does not attest replacement display");
        let stamp = controller.document().stamp(box_id).unwrap();
        let display = controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
        assert_eq!(&*display.topology, &ids);
        assert_eq!(display.deflection_mm, 0.05);
        let Geometry::Brep(shape) = &controller.document().entities[&box_id].geometry else { panic!("BRep") };
        let mut wrong_topology = shape.display().clone();
        wrong_topology.topology.faces[0] = uuid::Uuid::new_v4();
        let replacement = om9_core::brep::Brep::from_native(shape.source().to_owned(), wrong_topology,
            controller.coordinator.limits()).unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let prepared = controller.coordinator.prepare_with_held_usage(TransactionRequest::new("replace topology fixture", vec![
            Mutation::Geometry { source: stamp, geometry: Geometry::Brep(replacement) },
        ]), held).unwrap();
        controller.commit_with_candidate_scene(prepared, held).unwrap();
        assert!(controller.topology_source_proof_for(Some(box_id)).is_none(), "source text alone does not attest new topology IDs");
        assert!(proof.rebind_source_unchanged(controller.coordinator(), |_, _| Ok(())).is_err());
    }

    #[test]
    fn extract_sphere_face_after_coarser_scene_remesh() {
        let (mut controller, _, sphere) = controller_with_two_topology_proofs();
        controller.request_topology_remesh_for(sphere, 1.0, 1.5,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let stamp = controller.document().stamp(sphere).unwrap();
        let scene_deflection = controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().deflection_mm;
        let Geometry::Brep(brep) = &controller.document().entities[&sphere].geometry else { panic!("sphere BRep") };
        assert!(scene_deflection > brep.display().deflection_mm);
        controller.select_objects(&[sphere]).unwrap();
        let before = controller.document().clone();
        controller.solid_extract("OM9-SOLID-006", 0, true, false).unwrap();
        assert_eq!(controller.document().entities.len(), 3);
        let output = controller.document().entities.values().find(|entity| !before.entities.contains_key(&entity.id)).unwrap();
        let Geometry::Brep(face) = &output.geometry else { panic!("extracted BRep") };
        assert!((face.display().deflection_mm - 0.1).abs() < 1e-12, "output keeps document profile");
        let measure = om9_kernel::creation::measure(face, controller.coordinator.limits()).unwrap();
        assert!((measure.area_mm2 - 36. * std::f64::consts::PI).abs() < 1e-6);
        controller.undo().unwrap(); assert_eq!(controller.document(), &before);
        controller.redo().unwrap(); assert_eq!(controller.document().entities.len(), 3);
    }

    #[test]
    #[ignore = "manual timing measurement for bounded synthetic sphere"]
    fn timing_extract_sphere_face_display_verification() {
      for deflection in [0.1, 0.01, 0.001] {
        let limits = om9_core::budget::Limits::default();
        let sphere = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, deflection, &limits).unwrap();
        let triangles = sphere.display().triangles.len();
        let mut doc = om9_core::model::Document::with_template_tolerance("face verification timing");
        let entity = om9_core::model::Entity::new("sphere", Geometry::Brep(sphere), None);
        let id = entity.id; doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, limits).unwrap();
        for _ in 0..3 {
            c.select_objects(&[id]).unwrap();
            let start = std::time::Instant::now();
            c.solid_extract("OM9-SOLID-006", 0, true, false).unwrap();
            eprintln!("sphere deflection={deflection} triangles={triangles} extraction_ms={:.3}", start.elapsed().as_secs_f64() * 1000.);
            c.undo().unwrap();
        }
      }
    }

    #[test]
    fn face_display_verification_can_be_canceled() {
        let limits = om9_core::budget::Limits::default();
        let sphere = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.001, &limits).unwrap();
        let display = sphere.display();
        let tri = display.triangles[0];
        let witness = std::array::from_fn(|i| display.points[tri[i] as usize]);
        let already = om9_core::transaction::Cancellation::default();
        already.cancel();
        assert!(matches!(om9_kernel::editing::verify_face_display_cancellable(&sphere, display, 0, witness, 1e-6, &limits, &already), Err(om9_core::Error::Canceled)));
        let token = om9_core::transaction::Cancellation::default();
        std::thread::scope(|scope| {
            let (start, ready) = std::sync::mpsc::sync_channel(0);
            let worker_token = token.clone();
            let sphere_ref = &sphere;
            let worker = scope.spawn(move || {
                start.send(()).unwrap();
                om9_kernel::editing::verify_face_display_cancellable(sphere_ref, display, 0, witness, 1e-6, &limits, &worker_token)
            });
            ready.recv().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(25));
            token.cancel();
            assert!(matches!(worker.join().unwrap(), Err(om9_core::Error::Canceled)));
        });
    }

    fn coarsely_remeshed_cylinder() -> (Controller, om9_core::identity::ObjectId, usize) {
        use om9_kernel::creation::{primitive_with_topology, Primitive};
        let limits = Limits::default();
        let captured = primitive_with_topology(Primitive::Cylinder { radius_mm: 2., height_mm: 5. },
            0.1, &limits, |_, _| Ok(())).unwrap();
        let (brep, proof) = captured.into_parts();
        let side = (0..brep.topology().faces.len()).find(|&i| {
            let face = om9_kernel::editing::extract_face(&brep, i, 0.1, &limits).unwrap();
            let area = om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2;
            (area - 20. * std::f64::consts::PI).abs() < 1e-6
        }).unwrap();
        let mut doc = om9_core::model::Document::with_template_tolerance("coarse cylinder");
        let entity = om9_core::model::Entity::new("cylinder", Geometry::Brep(brep), None);
        let id = entity.id; doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, limits).unwrap();
        c.submit("Point").unwrap(); c.submit("30,30,30").unwrap(); c.undo().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let bytes = proof.storage_bytes().unwrap();
        let mut reservation = c.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, c.scene_cache_usage().unwrap()).unwrap();
        reservation.set_allocated_bytes(bytes).unwrap();
        c.adopt_topology_proof(c.document().stamp(id).unwrap(), proof, reservation).unwrap();
        c.request_topology_remesh_for(id, 1.0, 1.5, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = c.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(c.scene().unwrap().root_brep_pick_display(&c.document().stamp(id).unwrap()).unwrap().deflection_mm, 1.0);
        c.select_objects(&[id]).unwrap();
        (c, id, side)
    }

    #[test]
    fn fillet_uses_published_witness_profile_after_remesh() {
        use om9_scene::scene::Subelement;
        for profile in [0.04, 1.0] { for picked in [false, true] {
        let (mut c, id, _) = coarsely_remeshed_cylinder();
        if profile != 1.0 {
            c.request_topology_remesh_for(id, profile, 0.3, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = c.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
        let stamp = c.document().stamp(id).unwrap();
        let display = c.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
        let segment = display.segments.iter().filter(|s| {
            let a = display.points[s[0] as usize]; let b = display.points[s[1] as usize];
            (a[2] - b[2]).abs() < 1e-8 && (a[2] - 5.).abs() < 1e-8
        }).max_by(|a,b| {
            let length = |s: &&[u32;3]| { let p=display.points[s[0] as usize];let q=display.points[s[1] as usize];
                (0..3).map(|i|(p[i]-q[i]).powi(2)).sum::<f64>() };
            length(a).total_cmp(&length(b))
        }).copied().expect("circular edge segment");
        let edge = segment[2] as usize;
        let witness = [display.points[segment[0] as usize], display.points[segment[1] as usize]];
        let edge_segments: Vec<_> = display.segments.iter().filter(|item| item[2] as usize == edge).collect();
        let handle_segment = edge_segments[edge_segments.len() / 2];
        let a = display.points[handle_segment[0] as usize]; let b = display.points[handle_segment[1] as usize];
        let expected_handle: [f64; 3] = std::array::from_fn(|i| (a[i] + b[i]) * 0.5);
        let key = display.topology.edges[edge];
        let Geometry::Brep(source) = &c.document().entities[&id].geometry else { panic!("BRep") };
        let limits = *c.coordinator.limits();
        assert_eq!(source.display().deflection_mm, 0.1);
        if profile == 1.0 { assert!(om9_kernel::editing::verified_edge_endpoints(source, witness, 0.1, 1e-6, &limits).is_err(), "fixture must distinguish old document profile"); }
        assert!(om9_kernel::editing::verified_edge_endpoints(source, witness, profile, 1e-6, &limits).unwrap().is_none());
        let expected = om9_kernel::editing::fillet_with_radii(source, &[(edge, 0.2)], 0.1, &limits).unwrap();
        let expected_volume = om9_kernel::creation::measure(&expected, &limits).unwrap().volume_mm3;
        let before = c.document().clone();
        let reference = SelectedSubobject { root:id, path:vec![], subset:Subelement::BrepEdge(key), source:stamp, world_epoch:c.coordinator.epoch() };
        assert!(c.solid_fillet_edge_endpoints(&reference).unwrap().is_none());
        if picked {
            c.submit("FilletEdge").unwrap(); c.submit("NextRadius=0.2").unwrap();
            c.solid_fillet_pick_edge(reference).unwrap(); c.submit("").unwrap();
            let handles = c.solid_fillet_handles();
            assert_eq!(handles.len(), 1);
            assert!((0..3).all(|i| (handles[0].1[i] - expected_handle[i]).abs() < 1e-8), "handle must use the published scene display");
            c.submit("").unwrap();
        } else { c.submit(&format!("FilletEdge {edge} 0.2")).unwrap(); }
        let Geometry::Brep(result) = &c.document().entities[&id].geometry else { panic!("fillet result") };
        assert!((om9_kernel::creation::measure(result, &limits).unwrap().volume_mm3 - expected_volume).abs() < 1e-7);
        let after = c.document().clone();
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
        }}
    }

    #[test]
    fn captured_face_pin_retains_scene_override_and_rejects_other_document() {
        let (mut c, id, _) = coarsely_remeshed_cylinder();
        let stamp = c.document().stamp(id).unwrap();
        let scene = c.scene().unwrap();
        let foreign = Controller::new(om9_core::model::Document::new("foreign"), om9_core::budget::Limits::default()).unwrap();
        assert!(foreign.scene().unwrap().root_brep_display_pin(&stamp).is_err());
        let publication = scene.publication_id();
        let saved = scene.root_brep_display_pin(&stamp).unwrap().unwrap();
        assert_eq!(saved.display().deflection_mm, 1.0);
        c.submit("Point").unwrap(); c.submit("40,40,40").unwrap();
        assert_ne!(publication, c.scene().unwrap().publication_id());
        assert_eq!(saved.display().deflection_mm, 1.0, "old pinned override must remain alive");
    }

    #[test]
    fn explicit_isocurve_build_worker_commits_one_transaction() {
        let limits=om9_core::budget::Limits::default();
        let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.1,&limits).unwrap();
        let face=shape.topology().faces[0];
        let entity=om9_core::model::Entity::new("sphere",Geometry::Brep(shape),None);let id=entity.id;
        let mut doc=om9_core::model::Document::with_template_tolerance("isocurve worker commit");doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,limits).unwrap();
        c.submit("Point").unwrap();c.submit("50,50,50").unwrap();c.undo().unwrap();
        c.select_objects(&[id]).unwrap();
        c.select_subobject(id,om9_scene::scene::Subelement::BrepFace(face)).unwrap();
        c.submit("ExtractIsocurve").unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {if let Some(result)=c.poll_face_verification(){result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
        let before=c.document().clone();c.request_isocurve_build_worker().unwrap();
        assert!(matches!(c.submit("V=1"),Err(Error::Busy)));
        assert_eq!(c.document(),&before);
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        let refusal=loop {if let Some(result)=c.poll_isocurve_build_worker(){break result;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
        assert!(matches!(refusal,Err(Error::Budget(_))));
        assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);assert!(c.isocurve_recovery_key().is_some());
        drop(pressure);c.request_isocurve_build_worker().unwrap();
        loop {if let Some(result)=c.poll_isocurve_build_worker(){result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
        let curves:Vec<_>=c.document().entities.values().filter(|entity|entity.label=="ExtractIsocurve").collect();
        assert_eq!(curves.len(),1);
        let Geometry::Curve(curve)=&curves[0].geometry else {panic!("curve")};
        let point=om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(0.25),0,curves[0].placement,&limits).unwrap()[0];
        assert!((point[0].hypot(point[1])-3.).abs()<1e-6 && point[2].abs()<1e-6);
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }

    #[test]
    fn source_isocurve_build_job_checks_stale_and_retains_output_quota() {
        use std::sync::atomic::AtomicBool;
        let limits=om9_core::budget::Limits::default();
        let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.1,&limits).unwrap();
        let face=shape.topology().faces[0];
        let entity=om9_core::model::Entity::new("sphere",Geometry::Brep(shape),None);let id=entity.id;
        let mut document=om9_core::model::Document::with_template_tolerance("isocurve worker");document.entities.insert(id,entity);
        let mut c=Controller::new(document,limits).unwrap();
        c.submit("Point").unwrap();c.submit("50,50,50").unwrap();c.undo().unwrap();
        c.select_objects(&[id]).unwrap();c.select_subobject(id,om9_scene::scene::Subelement::BrepFace(face)).unwrap();
        c.submit("ExtractIsocurve").unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {if let Some(result)=c.poll_face_verification(){result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
        let before=c.coordinator().snapshot();let active=Arc::new(AtomicBool::new(false));
        let mut job=super::isocurve_worker::IsocurveJob::start(&c,Arc::clone(&active)).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        let output=loop {if let Some(result)=job.poll(&c){break result.unwrap();}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
        assert_ne!(output.thread,std::thread::current().id());assert_eq!(output.curves.len(),1);
        for fraction in [0.,0.25,0.5,0.75,1.] {
            let point=om9_core::evaluation::evaluate_curve(&output.curves[0],om9_core::evaluation::Parameter::Fraction(fraction),
                0,om9_core::geometry::Transform::identity(),&limits).unwrap()[0];
            assert!((point[0].hypot(point[1])-3.).abs()<1e-6 && point[2].abs()<1e-6,"{point:?}");
        }
        assert!(output.reservation.bytes()>0);assert_eq!(c.coordinator().snapshot(),before);
        drop(output);drop(job);
        let mut canceled=super::isocurve_worker::IsocurveJob::start(&c,Arc::clone(&active)).unwrap();
        canceled.cancel();assert!(matches!(canceled.poll(&c),Some(Err(Error::Canceled))));drop(canceled);
        while active.load(std::sync::atomic::Ordering::Acquire) {
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(c.coordinator().snapshot(),before);
        let mut pressured=super::isocurve_worker::IsocurveJob::start(&c,Arc::clone(&active)).unwrap();
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        let failure=loop {if let Some(result)=pressured.poll(&c){break result;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
        assert!(matches!(failure,Err(Error::Budget(_))));drop(pressure);drop(pressured);
        assert_eq!(c.coordinator().snapshot(),before);
        let mut changed=super::isocurve_worker::IsocurveJob::start(&c,Arc::clone(&active)).unwrap();
        c.submit("V=0.25").unwrap();
        assert!(matches!(changed.poll(&c),Some(Err(Error::Stale(_)))));drop(changed);
        while active.load(std::sync::atomic::Ordering::Acquire) {
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let mut stale=super::isocurve_worker::IsocurveJob::start(&c,Arc::clone(&active)).unwrap();
        c.select_objects(&[]).unwrap();
        assert!(matches!(stale.poll(&c),Some(Err(Error::Stale(_)))));
        assert_eq!(c.coordinator().snapshot(),before);
    }

    #[test]
    fn selected_isocurve_bounds_are_prepared_on_face_worker() {
        let (mut c, id) = controller_with_topology_proof();
        c.select_objects(&[id]).unwrap();
        let stamp = c.document().stamp(id).unwrap();
        let face = c.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.faces[0];
        c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(face)).unwrap();
        let selection = c.subselection()[0].clone();
        let reserve = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 16*1024*1024, None).unwrap();
        let mut job = super::face_verification::FaceVerificationJob::start_inner(&c, std::slice::from_ref(&selection),
            reserve, true, Some(super::isocurve::ISOCURVE), None, None).unwrap();
        let deadline = std::time::Instant::now()+std::time::Duration::from_secs(30);
        let receipt = loop { if let Some(result) = job.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now()<deadline); std::thread::sleep(std::time::Duration::from_millis(2)); };
        assert!(receipt.isocurve_bounds_thread_for_test().is_some_and(|thread| thread != std::thread::current().id()));
        let bounds=receipt.isocurve_bounds_for(&c,&selection).unwrap();
        assert!(bounds.iter().all(|axis| axis[0].is_finite() && axis[1].is_finite() && axis[0]<axis[1]));
        c.clear_subselection();
        assert!(receipt.isocurve_bounds_for(&c,&selection).is_err());
        drop(receipt); drop(job);
        c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(face)).unwrap();
        let selection=c.subselection()[0].clone();let before=c.coordinator().snapshot();
        let reserve=c.coordinator().reserve_external_memory(MemoryClass::Kernel,1024*1024,None).unwrap();
        let mut job=super::face_verification::FaceVerificationJob::start_inner(&c,std::slice::from_ref(&selection),
            reserve,true,Some(super::isocurve::ISOCURVE),None,None).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        let reply=loop {if let Some(reply)=job.take_growth_for_test(){break reply;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
        reply.send(Err(Error::Budget("isocurve bounds fixture"))).unwrap();
        let result=loop {if let Some(result)=job.try_result(){break result;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
        assert!(matches!(result,Err(Error::Budget("isocurve bounds fixture"))));
        assert_eq!(c.coordinator().snapshot(),before);
    }

    #[test]
    fn face_verification_source_and_display_outlive_document_without_pinning_it() {
        for remeshed in [false, true] {
            let (mut c, id) = if remeshed { let (c, id, _) = coarsely_remeshed_cylinder(); (c, id) }
                else { controller_with_topology_proof() };
            c.select_objects(&[id]).unwrap();
            let stamp = c.document().stamp(id).unwrap();
            let face = c.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.faces[0];
            c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(face)).unwrap();
            let selection = c.subselection()[0].clone();
            let reserve = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 16 * 1024 * 1024, None).unwrap();
            let mut job = super::face_verification::FaceVerificationJob::start(&c, selection, reserve).unwrap();
            let old = Arc::downgrade(&c.scene().unwrap().document);
            let peer = c.coordinator.memory_admission_peer().unwrap();
            drop(c);
            assert!(old.upgrade().is_none(), "face verification must not retain the old document");
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            let receipt = loop {
                if let Some(result) = job.try_result() { break result.unwrap(); }
                assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(2));
            };
            assert!(peer.external_reserved_bytes(MemoryClass::Kernel) > 0);
            assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0, "completed witness must release its source display owner");
            drop(receipt); drop(job);
            assert_eq!(peer.external_reserved_bytes(MemoryClass::Kernel), 0);
            assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0);
        }
    }

    #[test]
    fn source_face_capture_refusal_preserves_history_and_can_retry() {
        let (mut c, id) = controller_with_topology_proof();
        c.submit("Point").unwrap(); c.submit("30,30,30").unwrap(); c.undo().unwrap();
        c.select_objects(&[id]).unwrap();
        let stamp = c.document().stamp(id).unwrap();
        let face = c.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.faces[0];
        c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(face)).unwrap();
        let selection = c.subselection()[0].clone();
        let reserve = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 16 * 1024 * 1024, None).unwrap();
        let pressure = c.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Export, None, c.scene_cache_usage().unwrap()).unwrap();
        let before = c.document().clone(); let undo = c.coordinator().undo_len(); let redo = c.coordinator().redo_len();
        assert!(matches!(super::face_verification::FaceVerificationJob::start(&c, selection.clone(), reserve), Err(Error::Budget(_))));
        assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), undo); assert_eq!(c.coordinator().redo_len(), redo);
        drop(pressure);
        let reserve = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 16 * 1024 * 1024, None).unwrap();
        let mut job = super::face_verification::FaceVerificationJob::start(&c, selection, reserve).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let receipt = loop { if let Some(result) = job.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(2)); };
        assert!(receipt.accept(&c).is_ok());
        assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), undo); assert_eq!(c.coordinator().redo_len(), redo);
    }

    #[test]
    fn shell_worker_does_not_retain_document() {
        for count in [1, 2] {
        let (mut c, id) = controller_with_topology_proof();
        c.select_objects(&[id]).unwrap();
        let stamp = c.document().stamp(id).unwrap();
        let faces = c.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.faces[..count].to_vec();
        for face in faces { c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(face)).unwrap(); }
        let selections = c.subselection().to_vec();
        let limits = *c.coordinator.limits();
        let reserve = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 16 * 1024 * 1024, None).unwrap();
        let mut job = super::face_verification::FaceVerificationJob::start_inner(&c, &selections, reserve, true,
            Some("OM9-SOLID-029"), Some(0.2), None).unwrap();
        let old = Arc::downgrade(&c.scene().unwrap().document);
        let peer = c.coordinator.memory_admission_peer().unwrap();
        drop(c);
        assert!(old.upgrade().is_none(), "Shell worker must not retain the document");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let receipt = loop { if let Some(result) = job.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(2)); };
        let volume = om9_kernel::creation::measure(receipt.shell_output_for_test().unwrap(), &limits).unwrap().volume_mm3;
        assert!(volume > 0. && volume < 120., "unexpected Shell volume {volume}");
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0, "completed Shell must release source display");
        assert!(peer.external_reserved_bytes(MemoryClass::Kernel) > 0, "Shell output remains charged until receipt drop");
        drop(receipt); drop(job);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::Kernel), 0);
        }
    }

    #[test]
    fn grouped_extract_worker_does_not_retain_document() {
        for copy in [false, true] {
            let (mut c, box_id, sphere_id) = controller_with_two_topology_proofs();
            c.select_objects(&[box_id, sphere_id]).unwrap();
            for (id, count) in [(box_id, 2), (sphere_id, 1)] {
                let stamp = c.document().stamp(id).unwrap();
                let faces = c.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.faces[..count].to_vec();
                for face in faces { c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(face)).unwrap(); }
            }
            let selections = c.subselection().to_vec();
            let limits = *c.coordinator.limits();
            let options = super::face_verification::ExtractOptions { copy, input_layer: false, current_layer: c.document().current_layer };
            let reserve = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 32 * 1024 * 1024, None).unwrap();
            let mut job = super::face_verification::FaceVerificationJob::start_inner(&c, &selections, reserve, true,
                Some("OM9-SOLID-006"), None, Some(options)).unwrap();
            let old = Arc::downgrade(&c.scene().unwrap().document);
            let peer = c.coordinator.memory_admission_peer().unwrap();
            drop(c);
            assert!(old.upgrade().is_none(), "Extract Surface worker must not retain the document");
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            let receipt = loop { if let Some(result) = job.try_result() { break result.unwrap(); }
                assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(2)); };
            let (faces, keep, deleted, replaced) = receipt.extract_summary_for_test().unwrap();
            assert_eq!(faces.len(), 3);
            for face in faces { let measure = om9_kernel::creation::measure(face, &limits).unwrap(); assert_eq!(measure.faces, 1); assert!(measure.area_mm2 > 0.); }
            assert_eq!((keep, deleted, replaced), if copy { (2, 0, 0) } else { (0, 1, 1) });
            assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0);
            assert!(peer.external_reserved_bytes(MemoryClass::Kernel) > 0);
            drop(receipt); drop(job);
            assert_eq!(peer.external_reserved_bytes(MemoryClass::Kernel), 0);
        }
    }

    #[test]
    fn extract_worker_uses_remeshed_witness_and_source_output_profile() {
        let (mut c, _, sphere) = controller_with_two_topology_proofs();
        c.request_topology_remesh_for(sphere, 1.0, 1.5,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = c.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let stamp = c.document().stamp(sphere).unwrap();
        let display = c.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
        assert_eq!(display.deflection_mm, 1.0);
        assert_eq!(c.scene().unwrap().root_brep_output_deflection(&stamp).unwrap(), 0.1);
        let snapshot = c.scene().unwrap().try_render_snapshot().unwrap();
        assert_eq!(snapshot.root_brep_output_deflection(&stamp).unwrap(), 0.1);
        drop(snapshot);
        let face = display.topology.faces[0];
        c.select_objects(&[sphere]).unwrap();
        c.select_subobject(sphere, om9_scene::scene::Subelement::BrepFace(face)).unwrap();
        let selections = c.subselection().to_vec();
        let options = super::face_verification::ExtractOptions { copy: true, input_layer: false, current_layer: c.document().current_layer };
        let reserve = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 32 * 1024 * 1024, None).unwrap();
        let mut job = super::face_verification::FaceVerificationJob::start_inner(&c, &selections, reserve, true,
            Some("OM9-SOLID-006"), None, Some(options)).unwrap();
        let receipt = loop {
            if let Some(result) = job.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        };
        receipt.accept(&c).unwrap();
        let (faces, keep, deleted, replaced) = receipt.extract_summary_for_test().unwrap();
        assert_eq!((faces.len(), keep, deleted, replaced), (1, 1, 0, 0));
        assert!((faces[0].display().deflection_mm - 0.1).abs() < 1e-12);
        let measure = om9_kernel::creation::measure(faces[0], c.coordinator.limits()).unwrap();
        assert!((measure.area_mm2 - 36. * std::f64::consts::PI).abs() < 1e-6);
        c.clear_subselection();
        assert!(matches!(receipt.accept(&c), Err(om9_core::Error::Stale(_))));
    }

    #[test]
    fn dupedge_source_view_uses_current_scene_profile_after_remesh() {
        let (mut c, _, sphere) = controller_with_two_topology_proofs();
        c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
        c.request_topology_remesh_for(sphere, 0.005, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = c.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let stamp = c.document().stamp(sphere).unwrap();
        let Geometry::Brep(source) = &c.document().entities[&sphere].geometry else { panic!("B-rep") };
        assert_eq!(source.display().deflection_mm, 0.1);
        assert_eq!(c.scene().unwrap().root_brep_pick_display(&stamp).unwrap().deflection_mm, 0.005);
        c.submit("DupEdge").unwrap();
        let before = c.coordinator().snapshot();
        let publication = c.scene().unwrap().publication_id();
        let mut job = c.start_dupedge_source_view(sphere, 32 * 1024 * 1024, 16 * 1024 * 1024).unwrap();
        let receipt = loop {
            if let Some(result) = job.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        };
        receipt.prepare_scene(&c, 8 * 1024 * 1024).unwrap().publish(&mut c).unwrap();
        assert_eq!(c.scene().unwrap().root_brep_pick_display(&stamp).unwrap().deflection_mm, 0.005);
        assert_eq!(c.coordinator().snapshot(), before);
        assert_eq!(c.coordinator().redo_len(), 1);
        c.cancel();
        assert_eq!(c.scene().unwrap().publication_id(), publication);
        assert_eq!(c.coordinator().snapshot(), before);
        assert_eq!(c.coordinator().redo_len(), 1);
    }

    #[test]
    fn face_verification_worker_rejects_stale_receipt_and_preserves_history() {
        let (mut c, id, side) = coarsely_remeshed_cylinder();
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!() };
        let key = brep.topology().faces[side];
        c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(key)).unwrap();
        let selection = c.subselection()[0].clone();
        let reserve = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        let mut job = super::face_verification::FaceVerificationJob::start(&c, selection, reserve).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let receipt = loop {
            if let Some(result) = job.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        };
        drop(job);
        receipt.accept(&c).unwrap();
        assert_eq!(c.coordinator().redo_len(), 1);
        let before = c.document().clone();
        c.clear_subselection();
        assert!(receipt.accept(&c).is_err(), "selection changed");
        c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(key)).unwrap();
        let reserve = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        let mut second = super::face_verification::FaceVerificationJob::start(&c, c.subselection()[0].clone(), reserve).unwrap();
        let second_receipt = loop {
            if let Some(result) = second.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        };
        drop(second);
        second_receipt.accept(&c).unwrap();
        c.replace_scene_same_epoch_for_test(0.).unwrap();
        assert!(second_receipt.accept(&c).is_err(), "scene publication changed");
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().redo_len(), 1);
    }

    #[test]
    fn face_verification_worker_can_be_canceled_without_document_change() {
        let (mut c, id, side) = coarsely_remeshed_cylinder();
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!() };
        let key = brep.topology().faces[side];
        c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(key)).unwrap();
        let before = c.document().clone();
        let reserve = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        let mut job = super::face_verification::FaceVerificationJob::start(&c, c.subselection()[0].clone(), reserve).unwrap();
        job.cancel();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let result = loop {
            if let Some(result) = job.try_result() { break result; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        };
        match result { Err(Error::Canceled) => (), Ok(receipt) => assert!(matches!(receipt.accept(&c), Err(Error::Canceled))), _ => panic!("expected cancellation") }
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().redo_len(), 1);
    }

    fn prepared_large_face_extract() -> (Controller, om9_core::identity::ObjectId) {
        let limits = om9_core::budget::Limits::default();
        let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
        assert!(shape.display().triangles.len() > 512);
        let key = shape.topology().faces[0];
        let mut document = om9_core::model::Document::with_template_tolerance("async extract");
        let entity = om9_core::model::Entity::new("sphere", Geometry::Brep(shape), None);
        let id = entity.id; document.entities.insert(id, entity);
        let mut c = Controller::new(document, limits).unwrap();
        c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
        c.select_objects(&[id]).unwrap(); c.submit("ExtractSrf").unwrap(); c.submit("Copy=Yes").unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key),
            source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
        c.solid_extract_pick_face(reference).unwrap();
        (c, id)
    }

    fn pending_large_face_extract() -> (Controller, om9_core::identity::ObjectId) {
        let (mut c, id) = prepared_large_face_extract();
        c.submit("").unwrap();
        assert!(c.face_verification_pending());
        (c, id)
    }

    fn pending_extract_with_small_reservation() -> Controller {
        let (mut c, _) = prepared_large_face_extract();
        let picks = c.solid_pending.as_ref().unwrap().extract_face_picks.clone();
        let options = super::face_verification::ExtractOptions { copy: true, input_layer: false, current_layer: c.document().current_layer };
        let reservation = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 4096, None).unwrap();
        let job = super::face_verification::FaceVerificationJob::start_inner(&c, &picks, reservation, true,
            Some("OM9-SOLID-006"), None, Some(options)).unwrap();
        c.face_verification_job = Some(job); c.face_verification_input = Some(String::new());
        c
    }

    #[test]
    fn extract_worker_requests_growth_from_current_controller_and_handles_refusal_or_cancel() {
        use std::sync::atomic::Ordering;
        for mode in ["grant", "refuse", "cancel", "cancel_with_credit", "stale"] {
            let mut c = pending_extract_with_small_reservation();
            let before = c.document().clone();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            if mode.starts_with("cancel") {
                let reply = loop {
                    if let Some(reply) = c.face_verification_job.as_ref().unwrap().take_growth_for_test() { break reply; }
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(2));
                };
                assert_eq!(c.coordinator().external_reserved_bytes(MemoryClass::Kernel), 4096);
                if mode == "cancel_with_credit" {
                    let extra = c.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel, 1024 * 1024,
                        None, c.scene_cache_usage().unwrap()).unwrap();
                    reply.send(Ok(extra)).unwrap_or_else(|_| panic!("worker dropped the pending credit"));
                }
                c.cancel();
                while c.face_verification_active.load(Ordering::Acquire) {
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                assert_eq!(c.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
                drop(reply);
                assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
                continue;
            }
            if mode == "stale" { c.replace_scene_same_epoch_for_test(0.).unwrap(); }
            let pressure = if mode == "refuse" {
                Some(c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Export, None, c.scene_cache_usage().unwrap()).unwrap())
            } else { None };
            let mut observed_growth = false;
            let result = loop {
                if let Some(result) = c.poll_face_verification() { break result; }
                observed_growth |= c.coordinator().external_reserved_bytes(MemoryClass::Kernel) > 4096;
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            };
            if mode == "grant" {
                assert!(observed_growth); result.unwrap(); c.undo().unwrap();
            } else if mode == "stale" {
                assert!(observed_growth); assert!(matches!(result, Err(Error::Stale(_))), "{result:?}");
                assert_eq!(c.coordinator().redo_len(), 1);
            } else {
                assert!(matches!(result, Err(Error::Budget(_))), "{result:?}");
                assert!(c.solid_pending.is_some()); drop(pressure);
                assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
                c.submit("").unwrap();
                loop {
                    if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                c.undo().unwrap();
            }
            assert_eq!(c.document(), &before);
        }
    }

    #[test]
    fn shell_worker_can_grow_its_reservation_during_construction() {
        let limits = om9_core::budget::Limits::default();
        let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.0001, &limits).unwrap();
        let cap = (0..body.topology().faces.len()).find(|&i| {
            let face = om9_kernel::editing::extract_face(&body, i, 0.1, &limits).unwrap();
            (om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2 - 4. * std::f64::consts::PI).abs() < 1e-6
        }).unwrap();
        let key = body.topology().faces[cap];
        let mut document = om9_core::model::Document::with_template_tolerance("shell worker growth");
        let entity = om9_core::model::Entity::new("cylinder", Geometry::Brep(body), None);
        let id = entity.id; document.entities.insert(id, entity);
        let mut c = Controller::new(document, limits).unwrap();
        c.select_objects(&[id]).unwrap(); let before = c.document().clone();
        c.submit("Shell").unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key),
            source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
        c.solid_extract_pick_face(reference).unwrap(); c.submit("").unwrap(); c.submit("Thickness=0.5").unwrap();
        let picks = c.solid_pending.as_ref().unwrap().extract_face_picks.clone();
        let reservation = c.coordinator().reserve_external_memory(MemoryClass::Kernel, 4096, None).unwrap();
        let job = super::face_verification::FaceVerificationJob::start_inner(&c, &picks, reservation, true,
            Some("OM9-SOLID-029"), Some(0.5), None).unwrap();
        c.face_verification_job = Some(job); c.face_verification_input = Some(String::new());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut observed_growth = false;
        loop {
            if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
            observed_growth |= c.coordinator().external_reserved_bytes(MemoryClass::Kernel) > 4096;
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(observed_growth);
        let Geometry::Brep(shell) = &c.document().entities[&id].geometry else { panic!() };
        assert!((om9_kernel::creation::measure(shell, &limits).unwrap().volume_mm3 - 9.875 * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }

    #[test]
    fn face_worker_admission_refusal_preserves_prompt_and_allows_retry() {
        let (mut c, _) = prepared_large_face_extract();
        let before = c.document().clone();
        let prompt = c.prompt();
        let held = c.scene_cache_usage().unwrap();
        let pressure = c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Export, None, held).unwrap();
        assert!(matches!(c.submit(""), Err(Error::Budget(_))));
        assert!(!c.face_verification_pending());
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().redo_len(), 1);
        assert_eq!(c.prompt(), prompt);
        drop(pressure);
        c.submit("").unwrap();
        assert!(c.face_verification_pending());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(c.document().entities.len(), 2);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }

    #[test]
    fn face_worker_survives_refused_file_and_history_operations() {
        use crate::file_lifecycle::FileKind;
        let (mut c, _) = pending_large_face_extract();
        let before = c.document().clone();
        assert!(matches!(c.undo(), Err(Error::Busy)));
        assert!(matches!(c.redo(), Err(Error::Busy)));
        for kind in [FileKind::New, FileKind::Open, FileKind::SaveAs] {
            assert!(c.begin_file_operation(kind).is_err());
            assert!(c.file_operation().is_none());
        }
        assert!(c.face_verification_pending());
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().redo_len(), 1);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(c.document().entities.len(), 2);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }

    #[test]
    fn canceled_face_worker_cannot_publish_into_new_document() {
        use crate::file_lifecycle::{FileKind, FileOutcome, UnsavedDecision};
        use std::sync::atomic::Ordering;
        let (mut c, _) = pending_large_face_extract();
        let old_document = c.document().id;
        let active = Arc::clone(&c.face_verification_active);
        c.cancel();
        assert!(!c.face_verification_pending());
        let outcome = c.begin_file_operation(FileKind::New).unwrap();
        if matches!(outcome, FileOutcome::Pending) {
            let token = c.file_operation().unwrap().token();
            assert!(matches!(c.decide_unsaved(token, UnsavedDecision::Discard).unwrap(), FileOutcome::Replaced));
        } else {
            assert!(matches!(outcome, FileOutcome::Replaced));
        }
        assert_ne!(c.document().id, old_document);
        let replacement = c.document().clone();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while active.load(Ordering::Acquire) {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(c.poll_face_verification().is_none());
        assert_eq!(c.document(), &replacement);
        assert_eq!(c.coordinator().undo_len(), 0);
        assert_eq!(c.coordinator().redo_len(), 0);
    }

    #[test]
    fn canceled_face_worker_cannot_publish_into_open_document() {
        use crate::file_lifecycle::{FileKind, FileOutcome, FilePhase, UnsavedDecision};
        use std::sync::atomic::Ordering;
        let path = std::env::temp_dir().join(format!("om9-face-worker-open-{}.om9", uuid::Uuid::new_v4()));
        let opened = om9_core::model::Document::with_template_tolerance("worker replacement");
        let limits = om9_core::budget::Limits::default();
        om9_archive::Archive::save(&path, &opened, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let (mut c, _) = pending_large_face_extract();
        let active = Arc::clone(&c.face_verification_active);
        c.cancel();
        c.begin_file_operation(FileKind::Open).unwrap();
        let mut token = c.file_operation().unwrap().token();
        if c.file_operation().unwrap().phase() == FilePhase::Unsaved {
            c.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
            token = c.file_operation().unwrap().token();
        }
        assert!(matches!(c.accept_file_path(token, &path).unwrap(), FileOutcome::Replaced));
        assert_eq!(c.document().id, opened.id);
        let replacement = c.document().clone();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while active.load(Ordering::Acquire) {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(c.poll_face_verification().is_none());
        assert_eq!(c.document(), &replacement);
        assert_eq!(c.coordinator().undo_len(), 0);
        assert_eq!(c.coordinator().redo_len(), 0);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn large_face_extract_uses_worker_and_commits_one_undo() {
        let (mut c, id) = pending_large_face_extract();
        let before = c.document().clone();
        assert_eq!(c.coordinator().redo_len(), 1);
        assert!(matches!(c.submit("Copy=No"), Err(Error::Busy)));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let output = c.document().entities.values().find(|e| e.id != id).unwrap();
        let Geometry::Brep(face) = &output.geometry else { panic!() };
        assert!((om9_kernel::creation::measure(face, c.coordinator.limits()).unwrap().area_mm2 - 36. * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document().entities.len(), 2);
    }

    #[test]
    fn large_numeric_extract_uses_worker_for_copy_and_move() {
        for copy in [true, false] {
            let limits = om9_core::budget::Limits::default();
            let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
            let mut document = om9_core::model::Document::with_template_tolerance("async numeric extract");
            let entity = om9_core::model::Entity::new("sphere", Geometry::Brep(body), None);
            let id = entity.id; document.entities.insert(id, entity);
            let mut c = Controller::new(document, limits).unwrap();
            c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
            c.select_objects(&[id]).unwrap();
            let before = c.document().clone();
            c.submit("ExtractSrf").unwrap();
            c.submit(if copy { "Copy=Yes" } else { "Copy=No" }).unwrap();
            c.submit("0").unwrap(); assert!(c.face_verification_pending());
            assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
            assert!(matches!(c.submit("Copy=Yes"), Err(Error::Busy)));
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let output = c.document().entities.values().find(|e| e.id != id).unwrap();
            let Geometry::Brep(face) = &output.geometry else { panic!() };
            assert!((om9_kernel::creation::measure(face, c.coordinator.limits()).unwrap().area_mm2 - 36. * std::f64::consts::PI).abs() < 1e-6);
            assert_eq!(c.document().entities.contains_key(&id), copy);
            c.undo().unwrap(); assert_eq!(c.document(), &before);
            c.redo().unwrap(); assert_eq!(c.document().entities.contains_key(&id), copy);
            c.undo().unwrap(); c.select_objects(&[id]).unwrap();
            c.submit("ExtractSrf 0 Copy=Yes").unwrap(); assert!(c.face_verification_pending());
            c.cancel(); assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
        }
    }

    #[test]
    fn large_inline_extract_uses_worker_and_preserves_validation() {
        for copy in [true, false] {
            let limits = om9_core::budget::Limits::default();
            let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
            let mut document = om9_core::model::Document::with_template_tolerance("async inline extract");
            let entity = om9_core::model::Entity::new("sphere", Geometry::Brep(body), None);
            let id = entity.id; document.entities.insert(id, entity);
            let mut c = Controller::new(document, limits).unwrap();
            c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
            c.select_objects(&[id]).unwrap();
            let before = c.document().clone();
            assert!(c.submit("ExtractSrf 0 Copy=Yes Copy=No").is_err());
            assert!(!c.face_verification_pending()); assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
            c.submit(if copy { "ExtractSrf 0 Copy=Yes OutputLayer=Current" } else { "ExtractSrf 0 Copy=No OutputLayer=Current" }).unwrap();
            assert!(c.face_verification_pending());
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let output = c.document().entities.values().find(|e| e.id != id).unwrap();
            let Geometry::Brep(face) = &output.geometry else { panic!() };
            assert!((om9_kernel::creation::measure(face, c.coordinator.limits()).unwrap().area_mm2 - 36. * std::f64::consts::PI).abs() < 1e-6);
            assert_eq!(c.document().entities.contains_key(&id), copy);
            c.undo().unwrap(); assert_eq!(c.document(), &before);
            c.redo().unwrap(); assert_eq!(c.document().entities.contains_key(&id), copy);
        }
    }

    #[test]
    fn large_numeric_extract_cancel_and_stale_keep_document_and_redo() {
        for stale in [false, true] {
            let limits = om9_core::budget::Limits::default();
            let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
            let mut document = om9_core::model::Document::with_template_tolerance("async numeric refusal");
            let entity = om9_core::model::Entity::new("sphere", Geometry::Brep(body), None);
            let id = entity.id; document.entities.insert(id, entity);
            let mut c = Controller::new(document, limits).unwrap();
            c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
            c.select_objects(&[id]).unwrap();
            let before = c.document().clone();
            c.submit("ExtractSrf").unwrap(); c.submit("Copy=Yes").unwrap(); c.submit("0").unwrap();
            assert!(c.face_verification_pending());
            if stale {
                c.replace_scene_same_epoch_for_test(0.).unwrap();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                loop {
                    if let Some(result) = c.poll_face_verification() { assert!(result.is_err()); break; }
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            } else { c.cancel(); assert!(!c.face_verification_pending()); }
            assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
        }
    }

    #[test]
    fn large_face_extract_worker_cancel_and_stale_preserve_document_and_redo() {
        for stale in [false, true] {
            let (mut c, _) = pending_large_face_extract();
            let before = c.document().clone();
            if stale {
                c.replace_scene_same_epoch_for_test(0.).unwrap();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                loop {
                    if let Some(result) = c.poll_face_verification() { assert!(result.is_err()); break; }
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            } else { c.cancel(); assert!(!c.face_verification_pending()); }
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().redo_len(), 1);
        }
    }

    #[test]
    fn extract_worker_rejects_changed_options_and_commit_pressure() {
        use std::sync::atomic::Ordering;
        for pressure_mode in [false, true] {
            let (mut c, _) = pending_large_face_extract();
            let before = c.document().clone();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            while c.face_verification_active.load(Ordering::Acquire) {
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let pressure = if pressure_mode {
                Some(c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Export, None, c.scene_cache_usage().unwrap()).unwrap())
            } else {
                c.solid_pending.as_mut().unwrap().extract_copy = false;
                None
            };
            let result = loop {
                if let Some(result) = c.poll_face_verification() { break result; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            };
            if pressure_mode { assert!(matches!(result, Err(Error::Budget(_))), "{result:?}"); }
            else { assert!(matches!(result, Err(Error::Stale("extract options"))), "{result:?}"); }
            assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
            assert!(c.solid_pending.is_some());
            drop(pressure);
            c.solid_pending.as_mut().unwrap().extract_copy = true;
            c.submit("").unwrap();
            loop {
                if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            c.undo().unwrap(); assert_eq!(c.document(), &before);
        }
    }

    #[test]
    fn large_face_extract_worker_refuses_mixed_display_and_keeps_redo() {
        let limits = om9_core::budget::Limits::default();
        let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.0001, &limits).unwrap();
        let side = (0..body.topology().faces.len()).find(|&i| {
            let face = om9_kernel::editing::extract_face(&body, i, 0.1, &limits).unwrap();
            (om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2 - 20. * std::f64::consts::PI).abs() < 1e-6
        }).unwrap();
        let mut display = body.display().clone();
        assert!(display.triangles.len() > 512);
        let cap = display.triangles.iter().position(|tri| tri[3] as usize != side).unwrap();
        let mut misplaced = display.triangles.remove(cap); misplaced[3] = side as u32; display.triangles.push(misplaced);
        let key = display.faces[side];
        let body = om9_core::brep::Brep::from_native(body.source().to_owned(), display, &limits).unwrap();
        let mut document = om9_core::model::Document::with_template_tolerance("async mixed face");
        let entity = om9_core::model::Entity::new("cylinder", Geometry::Brep(body), None);
        let id = entity.id; document.entities.insert(id, entity);
        let mut c = Controller::new(document, limits).unwrap();
        c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
        c.select_objects(&[id]).unwrap(); c.submit("ExtractSrf").unwrap(); c.submit("Copy=Yes").unwrap();
        let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key),
            source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
        c.solid_extract_pick_face(reference).unwrap();
        let before = c.document().clone();
        c.submit("").unwrap(); assert!(c.face_verification_pending());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = c.poll_face_verification() { assert!(result.is_err()); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
        assert!(c.solid_pending.is_some(), "failed verification must retain the prompt");
    }

    #[test]
    fn large_multi_face_extract_worker_is_atomic() {
        for (copy, invalid_last) in [(true, false), (false, false), (true, true)] {
            let limits = om9_core::budget::Limits::default();
            let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.0001, &limits).unwrap();
            let side = (0..body.topology().faces.len()).find(|&i| {
                let face = om9_kernel::editing::extract_face(&body, i, 0.1, &limits).unwrap();
                (om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2 - 20. * std::f64::consts::PI).abs() < 1e-6
            }).unwrap();
            let cap = (0..body.topology().faces.len()).find(|&i| i != side).unwrap();
            let mut display = body.display().clone();
            assert!(display.triangles.len() > 512);
            if invalid_last {
                let ordinal = display.triangles.iter().position(|tri| tri[3] as usize == side).unwrap();
                let mut misplaced = display.triangles.remove(ordinal); misplaced[3] = cap as u32; display.triangles.push(misplaced);
            }
            let keys = [display.faces[side], display.faces[cap]];
            let body = om9_core::brep::Brep::from_native(body.source().to_owned(), display, &limits).unwrap();
            let mut document = om9_core::model::Document::with_template_tolerance("async multi extract");
            let entity = om9_core::model::Entity::new("cylinder", Geometry::Brep(body), None);
            let id = entity.id; document.entities.insert(id, entity);
            let mut c = Controller::new(document, limits).unwrap();
            c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
            c.select_objects(&[id]).unwrap(); let before = c.document().clone();
            c.submit("ExtractSrf").unwrap(); c.submit(if copy { "Copy=Yes" } else { "Copy=No" }).unwrap();
            for key in keys {
                let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key),
                    source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
                c.solid_extract_pick_face(reference).unwrap();
            }
            c.submit("").unwrap(); assert!(c.face_verification_pending());
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            let result = loop {
                if let Some(result) = c.poll_face_verification() { break result; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            };
            if invalid_last {
                assert!(result.is_err()); assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
                assert!(c.solid_pending.is_some());
            } else {
                result.unwrap();
                let mut areas: Vec<_> = c.document().entities.values().filter(|entity| entity.id != id).map(|entity| {
                    let Geometry::Brep(face) = &entity.geometry else { panic!() };
                    om9_kernel::creation::measure(face, c.coordinator.limits()).unwrap().area_mm2
                }).collect();
                areas.sort_by(f64::total_cmp);
                assert_eq!(areas.len(), 2);
                assert!((areas[0] - 4. * std::f64::consts::PI).abs() < 1e-6);
                assert!((areas[1] - 20. * std::f64::consts::PI).abs() < 1e-6);
                c.undo().unwrap(); assert_eq!(c.document(), &before);
                c.redo().unwrap(); assert_eq!(c.document().entities.len(), 3);
            }
        }
    }

    #[test]
    fn large_extract_worker_multiple_sources_preserves_placement_and_atomic_history() {
        for copy in [true, false] {
            let limits = om9_core::budget::Limits::default();
            let mut document = om9_core::model::Document::with_template_tolerance("async multi source");
            let mut roots = Vec::new();
            for (radius, x) in [(3., 0.), (4., 15.)] {
                let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: radius }, 0.01, &limits).unwrap();
                assert!(body.display().triangles.len() > 512);
                let key = body.topology().faces[0];
                let mut entity = om9_core::model::Entity::new("sphere", Geometry::Brep(body), None);
                entity.placement = om9_core::geometry::Transform::translation([x, 0., 0.]);
                roots.push((entity.id, key, entity.placement, radius)); document.entities.insert(entity.id, entity);
            }
            let mut c = Controller::new(document, limits).unwrap();
            c.submit("Point").unwrap(); c.submit("40,40,40").unwrap(); c.undo().unwrap();
            c.select_objects(&roots.iter().map(|root| root.0).collect::<Vec<_>>()).unwrap();
            let before = c.document().clone();
            c.submit("ExtractSrf").unwrap(); c.submit(if copy { "Copy=Yes" } else { "Copy=No" }).unwrap();
            for (id, key, _, _) in &roots {
                let reference = super::SelectedSubobject { root: *id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(*key),
                    source: c.document().stamp(*id).unwrap(), world_epoch: c.coordinator().epoch() };
                c.solid_extract_pick_face(reference).unwrap();
            }
            c.submit("").unwrap(); assert!(c.face_verification_pending());
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let outputs: Vec<_> = c.document().entities.values().filter(|entity| !before.entities.contains_key(&entity.id)).collect();
            assert_eq!(outputs.len(), 2);
            for (id, _, placement, radius) in &roots {
                assert_eq!(c.document().entities.contains_key(id), copy);
                let output = outputs.iter().find(|entity| entity.placement == *placement).unwrap();
                let Geometry::Brep(face) = &output.geometry else { panic!() };
                assert!((om9_kernel::creation::measure(face, &limits).unwrap().area_mm2 - 4. * std::f64::consts::PI * radius * radius).abs() < 1e-6);
            }
            c.undo().unwrap(); assert_eq!(c.document(), &before);
            c.redo().unwrap(); assert_eq!(c.document().entities.len(), if copy { 4 } else { 2 });
        }
    }

    #[test]
    #[ignore = "large multi-source native workload; run explicitly for admission evidence"]
    fn extract_worker_actual_outputs_exceed_sixteen_mib() {
        extract_worker_actual_outputs(false);
    }

    #[test]
    #[ignore = "large independent native workload; run explicitly for responsiveness evidence"]
    fn extract_worker_independent_outputs_exceed_sixteen_mib() {
        extract_worker_actual_outputs(true);
    }

    fn extract_worker_actual_outputs(independent: bool) {
        const MIB: u64 = 1024 * 1024;
        let limits = om9_core::budget::Limits::default();
        let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.001, &limits).unwrap();
        let triangle = super::face_verification::choose_witness(body.display(), 0, None).unwrap();
        let probe = om9_kernel::editing::extract_verified_face_with_witness_deflection(&body, 0, triangle, 1e-6,
            body.display().deflection_mm, body.display().deflection_mm, &limits).unwrap();
        let face_bytes = probe.storage_usage(&limits).unwrap().bytes;
        let count = (16 * MIB / face_bytes + 3) as usize;
        assert!((2..=96).contains(&count), "fixture output size {face_bytes}, sources {count}");
        drop(probe);
        let mut document = om9_core::model::Document::with_template_tolerance("large extract outputs");
        let mut roots = Vec::new();
        let mut face_keys = Vec::new();
        for index in 0..count {
            let source = if independent {
                om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.001, &limits).unwrap()
            } else { body.clone() };
            face_keys.push(source.topology().faces[0]);
            let mut entity = om9_core::model::Entity::new("sphere", Geometry::Brep(source), None);
            entity.placement = om9_core::geometry::Transform::translation([index as f64 * 10., 0., 0.]);
            roots.push(entity.id); document.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(document, limits).unwrap();
        c.select_objects(&roots).unwrap(); let before = c.document().clone();
        c.submit("ExtractSrf").unwrap(); c.submit("Copy=Yes").unwrap();
        for (id, key) in roots.iter().zip(face_keys) {
            let reference = super::SelectedSubobject { root: *id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key),
                source: c.document().stamp(*id).unwrap(), world_epoch: c.coordinator().epoch() };
            c.solid_extract_pick_face(reference).unwrap();
        }
        let start = std::time::Instant::now();
        c.submit("").unwrap(); assert!(c.face_verification_pending());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
        let mut peak_reserved = 0;
        let mut max_poll = std::time::Duration::ZERO;
        loop {
            let poll_start = std::time::Instant::now();
            let result = c.poll_face_verification();
            max_poll = max_poll.max(poll_start.elapsed());
            if let Some(result) = result { result.unwrap(); break; }
            peak_reserved = peak_reserved.max(c.coordinator().external_reserved_bytes(MemoryClass::Kernel));
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let output_bytes: u64 = c.document().entities.values().filter(|entity| !before.entities.contains_key(&entity.id)).map(|entity| {
            let Geometry::Brep(face) = &entity.geometry else { panic!() };
            face.storage_usage(&limits).unwrap().bytes
        }).sum();
        assert_eq!(c.document().entities.len(), count * 2);
        assert!(output_bytes > 16 * MIB, "{output_bytes}");
        assert!(peak_reserved > 16 * MIB, "{peak_reserved}");
        let sample = c.document().entities.values().find(|entity| !before.entities.contains_key(&entity.id)).unwrap();
        let Geometry::Brep(face) = &sample.geometry else { panic!() };
        assert!((om9_kernel::creation::measure(face, &limits).unwrap().area_mm2 - 36. * std::f64::consts::PI).abs() < 1e-6);
        eprintln!("independent={independent} sources={count} face_bytes={face_bytes} outputs={output_bytes} peak_kernel_reservation={peak_reserved} max_poll_ms={} elapsed_ms={}", max_poll.as_millis(), start.elapsed().as_millis());
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document().entities.len(), count * 2);
    }

    #[test]
    fn large_extract_worker_respects_output_layer_choice() {
        for input_layer in [false, true] {
            let limits = om9_core::budget::Limits::default();
            let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
            let mut document = om9_core::model::Document::with_template_tolerance("async extract layers");
            let source_layer = om9_core::model::Layer::new("Source", [255, 0, 0, 255]);
            let current_layer = om9_core::model::Layer::new("Current", [0, 255, 0, 255]);
            let source_id = source_layer.id; let current_id = current_layer.id;
            document.layers.insert(source_id, source_layer); document.layers.insert(current_id, current_layer);
            document.layer_order.extend([source_id, current_id]); document.current_layer = Some(current_id);
            let entity = om9_core::model::Entity::new("sphere", Geometry::Brep(body), Some(source_id));
            let id = entity.id; document.entities.insert(id, entity);
            let mut c = Controller::new(document, limits).unwrap();
            c.select_objects(&[id]).unwrap(); let before = c.document().clone();
            c.submit(if input_layer { "ExtractSrf 0 Copy=Yes OutputLayer=Input" } else { "ExtractSrf 0 Copy=Yes OutputLayer=Current" }).unwrap();
            assert!(c.face_verification_pending());
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let output = c.document().entities.values().find(|entity| entity.id != id).unwrap();
            assert_eq!(output.attributes.layer, Some(if input_layer { source_id } else { current_id }));
            c.undo().unwrap(); assert_eq!(c.document(), &before);
        }
    }

    #[test]
    fn extract_move_worker_obeys_same_object_limit_as_synchronous_path() {
        let limits = om9_core::budget::Limits { max_objects: 1, ..om9_core::budget::Limits::default() };
        let mut outcomes = Vec::new();
        for worker in [false, true] {
            let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
            assert!(body.display().triangles.len() > 512);
            let mut document = om9_core::model::Document::with_template_tolerance("one object extract");
            let entity = om9_core::model::Entity::new("sphere", Geometry::Brep(body), None);
            let id = entity.id; document.entities.insert(id, entity);
            let mut c = Controller::new(document, limits).unwrap();
            c.select_objects(&[id]).unwrap();
            let before = c.document().clone();
            let result = if worker {
                c.submit("ExtractSrf 0 Copy=No").unwrap();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                loop {
                    if let Some(result) = c.poll_face_verification() { break result; }
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            } else { c.solid_extract("OM9-SOLID-006", 0, false, false) };
            if result.is_err() { assert_eq!(c.document(), &before); }
            else { assert_eq!(c.document().entities.len(), 1); assert!(!c.document().entities.contains_key(&id)); }
            outcomes.push(result.is_ok());
        }
        assert_eq!(outcomes[0], outcomes[1]);
    }

    #[test]
    fn large_shell_verifies_on_worker_and_preserves_history() {
        for (route, corrupt) in [(0, false), (0, true), (1, false), (2, false), (3, false), (3, true), (4, false), (5, false)] {
            let limits = om9_core::budget::Limits::default();
            let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.0001, &limits).unwrap();
            let side = (0..body.topology().faces.len()).find(|&i| {
                let face = om9_kernel::editing::extract_face(&body, i, 0.1, &limits).unwrap();
                (om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2 - 20. * std::f64::consts::PI).abs() < 1e-6
            }).unwrap();
            let cap = (0..body.topology().faces.len()).find(|&i| i != side).unwrap();
            let other_cap = (0..body.topology().faces.len()).find(|&i| i != side && i != cap).unwrap();
            let mut display = body.display().clone();
            assert!(display.triangles.len() > 512);
            if corrupt {
                let ordinal = display.triangles.iter().position(|tri| tri[3] as usize == side).unwrap();
                let mut misplaced = display.triangles.remove(ordinal);
                misplaced[3] = if route == 3 { other_cap } else { cap } as u32;
                display.triangles.push(misplaced);
            }
            let key = display.faces[cap];
            let other_key = display.faces[other_cap];
            let body = om9_core::brep::Brep::from_native(body.source().to_owned(), display, &limits).unwrap();
            let mut document = om9_core::model::Document::with_template_tolerance("async shell");
            if route >= 4 { document.page_unit = om9_core::geometry::Unit::Centimeter; }
            let entity = om9_core::model::Entity::new("cylinder", Geometry::Brep(body), None);
            let id = entity.id; document.entities.insert(id, entity);
            let mut c = Controller::new(document, limits).unwrap();
            c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
            c.select_objects(&[id]).unwrap(); let before = c.document().clone();
            if route == 0 || route == 3 || route == 4 {
                c.submit("Shell").unwrap();
                let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key),
                    source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
                c.solid_extract_pick_face(reference).unwrap();
                if route == 3 {
                    let other = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(other_key),
                        source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
                    c.solid_extract_pick_face(other).unwrap();
                }
                c.submit("").unwrap(); c.submit(if route == 4 { "0.05" } else { "0.5" }).unwrap();
            } else if route == 1 {
                c.submit("Shell").unwrap(); c.submit(&format!("{cap} 0.5")).unwrap();
            } else {
                c.submit(&format!("Shell {cap} {}", if route == 5 { "0.05" } else { "0.5" })).unwrap();
            }
            assert!(c.face_verification_pending());
            assert!(matches!(c.submit("Thickness=1"), Err(Error::Busy)));
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            let result = loop {
                if let Some(result) = c.poll_face_verification() { break result; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(2));
            };
            if corrupt {
                assert!(result.is_err()); assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
                assert!(c.solid_pending.is_some());
            } else {
                result.unwrap();
                let Geometry::Brep(shell) = &c.document().entities[&id].geometry else { panic!() };
                let volume = om9_kernel::creation::measure(shell, &limits).unwrap().volume_mm3;
                let expected = if route == 3 { 8.75 } else { 9.875 } * std::f64::consts::PI;
                assert!((volume - expected).abs() < 1e-6, "{volume}");
                c.undo().unwrap(); assert_eq!(c.document(), &before);
                c.redo().unwrap();
                let Geometry::Brep(shell) = &c.document().entities[&id].geometry else { panic!() };
                assert!((om9_kernel::creation::measure(shell, &limits).unwrap().volume_mm3 - volume).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn shell_snapshot_admission_can_refuse_and_retry() {
        let limits = om9_core::budget::Limits::default();
        let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.1, &limits).unwrap();
        let before = body.clone();
        let cap = (0..body.topology().faces.len()).find(|&i| {
            let face = om9_kernel::editing::extract_face(&body, i, 0.1, &limits).unwrap();
            (om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2 - 4. * std::f64::consts::PI).abs() < 1e-6
        }).unwrap();
        let triangle = super::face_verification::choose_witness(body.display(), cap, None).unwrap();
        let mut refused = 0;
        let result = om9_kernel::editing::shell_verified_faces_with_deflection_admitted(&body, &[cap], &[triangle], 1e-6, 0.1, 0.5, 0.1, &limits,
            |_, pending| { assert!(pending > 0); refused += 1; Err(Error::Budget("test shell output")) });
        assert!(result.is_err()); assert!(refused > 0); assert_eq!(body, before);
        let mut admitted = 0;
        let shell = om9_kernel::editing::shell_verified_faces_with_deflection_admitted(&body, &[cap], &[triangle], 1e-6, 0.1, 0.5, 0.1, &limits,
            |_, _| { admitted += 1; Ok(()) }).unwrap();
        assert!(admitted > 0);
        assert!((om9_kernel::creation::measure(&shell, &limits).unwrap().volume_mm3 - 9.875 * std::f64::consts::PI).abs() < 1e-6);
        assert_eq!(body, before);
    }

    #[test]
    fn extract_and_remainder_snapshots_honor_admission() {
        let limits = om9_core::budget::Limits::default();
        let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.1, &limits).unwrap();
        let before = body.clone();
        let caps: Vec<_> = (0..body.topology().faces.len()).filter(|&i| {
            let face = om9_kernel::editing::extract_face(&body, i, 0.1, &limits).unwrap();
            (om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2 - 4. * std::f64::consts::PI).abs() < 1e-6
        }).collect();
        assert_eq!(caps.len(), 2);
        let triangles: Vec<_> = caps.iter().map(|&i| super::face_verification::choose_witness(body.display(), i, None).unwrap()).collect();
        for mode in 0..3 {
            for refuse in [true, false] {
                let mut calls = 0;
                let mut admit = |_: u64, _: u64| { calls += 1; if refuse { Err(Error::Budget("test output")) } else { Ok(()) } };
                let result = match mode {
                    0 => om9_kernel::editing::extract_verified_face_with_witness_deflection_admitted(&body, caps[0], triangles[0], 1e-6, 0.1, 0.1, &limits, &mut admit),
                    1 => om9_kernel::editing::remove_verified_face_with_deflection_admitted(&body, caps[0], triangles[0], 1e-6, 0.1, 0.1, &limits, &mut admit),
                    _ => om9_kernel::editing::remove_verified_faces_with_deflection_admitted(&body, &caps, &triangles, 1e-6, 0.1, 0.1, &limits, &mut admit),
                };
                assert!(calls > 0); assert_eq!(result.is_err(), refuse);
                if let Ok(output) = result {
                    let measures = om9_kernel::creation::measure(&output, &limits).unwrap();
                    let expected_faces = if mode == 1 { 2 } else { 1 };
                    assert_eq!(measures.faces, expected_faces);
                }
                assert_eq!(body, before);
            }
        }
    }

    #[test]
    fn extract_worker_counts_earlier_outputs_before_admitting_next() {
        let (c, id) = prepared_large_face_extract();
        let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!() };
        let triangle = super::face_verification::choose_witness(body.display(), 0, None).unwrap();
        let limits = *c.coordinator().limits();
        let mut peak = 0u64;
        let probe = om9_kernel::editing::extract_verified_face_with_witness_deflection_admitted(body, 0, triangle, 1e-6,
            body.display().deflection_mm, body.display().deflection_mm, &limits, |live, pending| {
                peak = peak.max(live.checked_add(pending).unwrap()); Ok(())
            }).unwrap();
        let bytes = probe.storage_usage(&limits).unwrap().bytes;
        assert!(bytes > 1 && peak > 0);
        drop(probe);
        let budget = peak.checked_add(bytes).unwrap() - 1;
        let mut reservation = c.coordinator().reserve_external_memory(MemoryClass::Kernel, budget, None).unwrap();
        let cancel = om9_core::transaction::Cancellation::default();
        let mut retained = 0;
        let invoke = |admit: &mut dyn FnMut(u64, u64) -> Result<()>| {
            om9_kernel::editing::extract_verified_face_with_witness_deflection_admitted(body, 0, triangle, 1e-6,
                body.display().deflection_mm, body.display().deflection_mm, &limits, admit)
        };
        let mut fixed_budget = |reservation: &mut om9_core::transaction::ExternalMemoryReservation, needed| {
            if needed > reservation.bytes() { Err(Error::Budget("extract worker output")) } else { Ok(()) }
        };
        let first = super::face_verification::admitted_extract_snapshot(&cancel, &mut reservation, 0, &mut retained, &limits, &mut fixed_budget, invoke).unwrap();
        assert_eq!(retained, bytes);
        let second = super::face_verification::admitted_extract_snapshot(&cancel, &mut reservation, 0, &mut retained, &limits, &mut fixed_budget, invoke);
        assert!(matches!(second, Err(Error::Budget("extract worker output"))), "{second:?}");
        assert_eq!(retained, bytes); assert_eq!(c.coordinator().redo_len(), 1);
        assert!((om9_kernel::creation::measure(&first, &limits).unwrap().area_mm2 - 36. * std::f64::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn large_shell_worker_cancel_stale_and_admission_preserve_redo() {
        for mode in ["cancel", "stale", "pressure", "pressure_thickness"] {
            let limits = om9_core::budget::Limits::default();
            let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.0001, &limits).unwrap();
            let cap = (0..body.topology().faces.len()).find(|&i| {
                let face = om9_kernel::editing::extract_face(&body, i, 0.1, &limits).unwrap();
                (om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2 - 4. * std::f64::consts::PI).abs() < 1e-6
            }).unwrap();
            let key = body.topology().faces[cap];
            let mut document = om9_core::model::Document::with_template_tolerance("shell worker safety");
            let entity = om9_core::model::Entity::new("cylinder", Geometry::Brep(body), None);
            let id = entity.id; document.entities.insert(id, entity);
            let mut c = Controller::new(document, limits).unwrap();
            c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
            c.select_objects(&[id]).unwrap(); let before = c.document().clone();
            c.submit("Shell").unwrap();
            if mode != "pressure_thickness" { c.submit("Thickness=0.5").unwrap(); }
            let reference = super::SelectedSubobject { root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepFace(key),
                source: c.document().stamp(id).unwrap(), world_epoch: c.coordinator().epoch() };
            c.solid_extract_pick_face(reference).unwrap();
            if mode == "pressure_thickness" { c.submit("").unwrap(); }
            let finish_input = if mode == "pressure_thickness" { "0.5" } else { "" };
            if mode.starts_with("pressure") {
                let held = c.scene_cache_usage().unwrap();
                let pressure = c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Export, None, held).unwrap();
                let prompt = c.prompt();
                assert!(matches!(c.submit(finish_input), Err(Error::Budget(_))));
                assert_eq!(c.prompt(), prompt); assert!(!c.face_verification_pending());
                assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
                drop(pressure);
            }
            c.submit(finish_input).unwrap(); assert!(c.face_verification_pending());
            if mode == "cancel" { c.cancel(); assert!(!c.face_verification_pending()); }
            else {
                if mode == "stale" { c.replace_scene_same_epoch_for_test(0.).unwrap(); }
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                let result = loop {
                    if let Some(result) = c.poll_face_verification() { break result; }
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(2));
                };
                if mode == "stale" { assert!(result.is_err()); assert!(c.solid_pending.is_some()); }
                else { result.unwrap(); c.undo().unwrap(); }
            }
            assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
        }
    }

    #[test]
    fn shell_worker_output_commit_refusal_preserves_document_and_retries() {
        use std::sync::atomic::Ordering;
        let limits = om9_core::budget::Limits::default();
        let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.0001, &limits).unwrap();
        let cap = (0..body.topology().faces.len()).find(|&i| {
            let face = om9_kernel::editing::extract_face(&body, i, 0.1, &limits).unwrap();
            (om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2 - 4. * std::f64::consts::PI).abs() < 1e-6
        }).unwrap();
        let mut document = om9_core::model::Document::with_template_tolerance("shell commit pressure");
        let entity = om9_core::model::Entity::new("cylinder", Geometry::Brep(body), None);
        let id = entity.id; document.entities.insert(id, entity);
        let mut c = Controller::new(document, limits).unwrap();
        c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
        c.select_objects(&[id]).unwrap(); let before = c.document().clone();
        c.submit(&format!("Shell {cap} 0.5")).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while c.face_verification_active.load(Ordering::Acquire) {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let held = c.scene_cache_usage().unwrap();
        let pressure = c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Export, None, held).unwrap();
        let result = loop {
            if let Some(result) = c.poll_face_verification() { break result; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        };
        assert!(matches!(result, Err(Error::Budget(_))), "expected commit admission refusal: {result:?}"); assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().redo_len(), 1); assert!(c.solid_pending.is_some());
        drop(pressure);
        c.submit(&format!("{cap} 0.5")).unwrap();
        loop {
            if let Some(result) = c.poll_face_verification() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }

    #[test]
    fn extract_curved_face_from_coarse_scene_by_selection() {
            let (mut c, id, side) = coarsely_remeshed_cylinder();
            let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!() };
            let key = brep.topology().faces[side];
            let before = c.document().clone();
            c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(key)).unwrap();
            let picks = c.subselection().to_vec();
            c.solid_extract_picks(&picks, true, false).unwrap();
            let output = c.document().entities.values().find(|entity| !before.entities.contains_key(&entity.id)).unwrap();
            let Geometry::Brep(face) = &output.geometry else { panic!() };
            let expected_area = 20. * std::f64::consts::PI;
            assert!((om9_kernel::creation::measure(face, c.coordinator.limits()).unwrap().area_mm2 - expected_area).abs() < 1e-6);
            assert_eq!(face.display().deflection_mm, 0.1);
            c.undo().unwrap(); assert_eq!(c.document(), &before);
            c.redo().unwrap(); assert_eq!(c.document().entities.len(), 2);
    }

    #[test]
    fn coarse_cylinder_cap_can_be_extracted_without_copy() {
      for selected in [false, true] {
        let (mut c, id, side) = coarsely_remeshed_cylinder();
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!() };
        let cap = (0..brep.topology().faces.len()).find(|&i| i != side).unwrap();
        let before = c.document().clone();
        if selected {
            c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(brep.topology().faces[cap])).unwrap();
            let picks = c.subselection().to_vec();
            c.solid_extract_picks(&picks, false, false).unwrap();
        } else { c.solid_extract("OM9-SOLID-006", cap, false, false).unwrap(); }
        let output = c.document().entities.values().find(|entity| !before.entities.contains_key(&entity.id)).unwrap();
        let Geometry::Brep(face) = &output.geometry else { panic!() };
        assert!((om9_kernel::creation::measure(face, c.coordinator.limits()).unwrap().area_mm2 - 4. * std::f64::consts::PI).abs() < 1e-6);
        assert_eq!(face.display().deflection_mm, 0.1);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document().entities.len(), 2);
      }
    }

    #[test]
    fn coarse_cylinder_extracts_two_selected_faces_in_one_undo() {
      for copy in [true, false] {
        let (mut c, id, side) = coarsely_remeshed_cylinder();
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!() };
        let cap = (0..brep.topology().faces.len()).find(|&i| i != side).unwrap();
        let keys = [brep.topology().faces[side], brep.topology().faces[cap]];
        let before = c.document().clone();
        for key in keys { c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(key)).unwrap(); }
        let picks = c.subselection().to_vec();
        c.solid_extract_picks(&picks, copy, false).unwrap();
        let mut areas: Vec<_> = c.document().entities.values().filter(|e| !before.entities.contains_key(&e.id)).map(|e| {
            let Geometry::Brep(face) = &e.geometry else { panic!() };
            assert_eq!(face.display().deflection_mm, 0.1);
            om9_kernel::creation::measure(face, c.coordinator.limits()).unwrap().area_mm2
        }).collect();
        areas.sort_by(f64::total_cmp);
        assert_eq!(areas.len(), 2);
        assert!((areas[0] - 4. * std::f64::consts::PI).abs() < 1e-6);
        assert!((areas[1] - 20. * std::f64::consts::PI).abs() < 1e-6);
        if !copy {
            let Geometry::Brep(remainder) = &c.document().entities[&id].geometry else { panic!() };
            let measured = om9_kernel::creation::measure(remainder, c.coordinator.limits()).unwrap();
            assert_eq!(measured.faces, 1);
            assert!((measured.area_mm2 - 4. * std::f64::consts::PI).abs() < 1e-6);
        }
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document().entities.len(), 3);
      }
    }

    #[test]
    fn coarse_cylinder_shell_cap_numeric_and_selection() {
        for selected in [false, true] {
            let (mut c, id, side) = coarsely_remeshed_cylinder();
            let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!() };
            let cap = (0..brep.topology().faces.len()).find(|&i| i != side).unwrap();
            let key = brep.topology().faces[cap];
            let before = c.document().clone();
            if selected {
                c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(key)).unwrap();
                let picks = c.subselection().to_vec();
                c.solid_shell_picks(&picks, 0.5).unwrap();
            } else { c.solid_shell("OM9-SOLID-029", cap, 0.5).unwrap(); }
            let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!() };
            let volume = om9_kernel::creation::measure(body, c.coordinator.limits()).unwrap().volume_mm3;
            assert!((volume - 9.875 * std::f64::consts::PI).abs() < 1e-6, "{volume}");
            assert_eq!(body.display().deflection_mm, 0.1);
            c.undo().unwrap(); assert_eq!(c.document(), &before);
            c.redo().unwrap();
        }
    }

    #[test]
    fn coarse_cylinder_shell_curved_face_refuses_like_source() {
        for selected in [false, true] {
            let (mut c, id, side) = coarsely_remeshed_cylinder();
            let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!() };
            let key = brep.topology().faces[side];
            assert!(om9_kernel::editing::shell(brep, side, 0.5, 0.1, c.coordinator.limits()).is_err(),
                "the original-profile native operation must show the same failure");
            let before = c.document().clone();
            let result = if selected {
                c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(key)).unwrap();
                let picks = c.subselection().to_vec();
                c.solid_shell_picks(&picks, 0.5)
            } else { c.solid_shell("OM9-SOLID-029", side, 0.5) };
            let error = result.unwrap_err();
            assert!(error.to_string().contains("shell construction failed"), "{error}");
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().redo_len(), 1);
        }
    }

    #[test]
    fn shell_box_from_coarse_scene_numeric_and_selection() {
        for selected in [false, true] {
            let (mut c, id, _) = controller_with_two_topology_proofs();
            c.request_topology_remesh_for(id, 1.0, 1.5, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = c.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            c.select_objects(&[id]).unwrap();
            let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!() };
            let key = brep.topology().faces[0];
            let before = c.document().clone();
            if selected {
                c.select_subobject(id, om9_scene::scene::Subelement::BrepFace(key)).unwrap();
                let picks = c.subselection().to_vec();
                c.solid_shell_picks(&picks, 0.5).unwrap();
            } else { c.solid_shell("OM9-SOLID-029", 0, 0.5).unwrap(); }
            let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!() };
            assert_eq!(body.display().deflection_mm, 0.1);
            assert!(om9_kernel::creation::measure(body, c.coordinator.limits()).unwrap().volume_mm3 > 0.);
            c.undo().unwrap(); assert_eq!(c.document(), &before);
            c.redo().unwrap();
        }
    }

    #[test]
    fn topology_remesh_worker_publishes_source_wires_without_pickable_edges() {
        use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind,RenderBuffers}};
        let (mut controller,id)=controller_with_topology_proof();
        let before=controller.document().clone();
        let undo=controller.coordinator.undo_len(); let redo=controller.coordinator.redo_len();
        controller.request_topology_remesh_for(id,0.04,0.3,16*1024*1024,8*1024*1024,8*1024*1024).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {
            if let Some(result)=controller.poll_topology_remesh() {result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline); std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let scene=controller.scene().unwrap();
        let object=scene.objects.iter().find(|object|object.root==id).unwrap();
        let stamp=controller.document().stamp(id).unwrap();
        let display=scene.root_brep_pick_display(&stamp).unwrap();
        let RenderBuffers::Mesh {vertices,edges,..}=object.buffers.as_ref() else {panic!("BRep buffer")};
        assert!(vertices.len()>display.points.len() && edges.len()>display.segments.len());
        let wires=vertices[display.points.len()..].to_vec();
        for p in &wires {
            assert!(p[0]>=-1e-7 && p[0]<=4.+1e-7 && p[1]>=-1e-7 && p[1]<=5.+1e-7 && p[2]>=-1e-7 && p[2]<=6.+1e-7);
            assert!(p[0].abs()<1e-7 || (p[0]-4.).abs()<1e-7 || p[1].abs()<1e-7 || (p[1]-5.).abs()<1e-7 || p[2].abs()<1e-7 || (p[2]-6.).abs()<1e-7);
        }
        let camera=Camera::orthographic([2.,2.5,20.],[2.,2.5,0.],[0.,1.,0.],10.).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[800.,800.]};
        let cursor=camera.project([2.,2.5,6.],viewport).unwrap().unwrap().screen;
        assert!(scene.pick_kind(cursor,&camera,viewport,1.,PickIntent::Select,PickKind::Edge).unwrap().is_none());
        assert!(scene.pick_edge_candidates(cursor,&camera,viewport,1.,PickIntent::Select).unwrap().is_empty());
        assert_eq!(controller.document(),&before);assert_eq!(controller.coordinator.undo_len(),undo);assert_eq!(controller.coordinator.redo_len(),redo);
        controller.request_topology_remesh_for(id,0.1,0.3,16*1024*1024,8*1024*1024,8*1024*1024).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {
            if let Some(result)=controller.poll_topology_remesh() {result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let scene=controller.scene().unwrap();
        let display=scene.root_brep_pick_display(&stamp).unwrap();
        let object=scene.objects.iter().find(|object|object.root==id).unwrap();
        let RenderBuffers::Mesh {vertices,..}=object.buffers.as_ref() else {panic!("BRep buffer")};
        assert_eq!(&vertices[display.points.len()..],wires.as_slice(),"wire source must not depend on display deflection");
        controller.undo().unwrap();assert!(!controller.document().entities.contains_key(&id));
        controller.redo().unwrap();assert_eq!(controller.document(),&before);
    }

    #[test]
    fn topology_remesh_wire_budget_failure_keeps_scene_and_history() {
        let (mut controller,id)=controller_with_topology_proof();
        controller.submit("Point").unwrap();controller.submit("30,30,30").unwrap();controller.undo().unwrap();
        let before=controller.document().clone();let publication=controller.scene().unwrap().publication_id();
        let undo=controller.coordinator.undo_len();let redo=controller.coordinator.redo_len();
        controller.request_topology_remesh_for(id,0.04,0.3,16*1024*1024,1,8*1024*1024).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {
            if let Some(result)=controller.poll_topology_remesh() {assert!(result.is_err());break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(controller.scene().unwrap().publication_id(),publication);
        assert_eq!(controller.document(),&before);assert_eq!(controller.coordinator.undo_len(),undo);assert_eq!(controller.coordinator.redo_len(),redo);
    }

    #[test]
    fn extract_wireframe_matches_published_brep_wires_after_remesh() {
        use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform};
        for profile in [0.04,0.1] { for menu in [false,true] {
            let (mut controller,id)=controller_with_topology_proof();
            controller.submit("Point").unwrap();controller.submit("30,30,30").unwrap();controller.undo().unwrap();
            let before=controller.document().clone();
            let Geometry::Brep(brep)=&before.entities[&id].geometry else {panic!("BRep")};
            let limits=*controller.coordinator.limits();
            let expected=om9_kernel::editing::default_face_isocurves_from_source_admitted(brep.source(),&limits,|_,_|Ok(())).unwrap();
            controller.request_topology_remesh_for(id,profile,0.3,16*1024*1024,8*1024*1024,8*1024*1024).unwrap();
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
            loop {
                if let Some(result)=controller.poll_topology_remesh() {result.unwrap();break;}
                assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));
            }
            controller.select_objects(&[id]).unwrap();
            if menu {controller.dispatch("OM9-CURVE-054",InvocationOrigin::Menu).unwrap();}
            else {controller.submit("ExtractWireframe").unwrap();}
            let outputs:Vec<_>=controller.document().entities.values().filter(|e|e.label=="ExtractWire").collect();
            assert_eq!(outputs.len(),24);assert_eq!(expected.len(),12);
            for curve in &expected {
                let ends=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,Transform::identity(),&limits).unwrap()[0]);
                let matches=outputs.iter().filter(|e| {
                    let Geometry::Curve(output)=&e.geometry else{return false};
                    let actual=[0.,1.].map(|t|evaluate_curve(output,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
                    let near=|a:[f64;3],b:[f64;3]| (0..3).all(|i|(a[i]-b[i]).abs()<1e-7);
                    (near(ends[0],actual[0]) && near(ends[1],actual[1])) || (near(ends[0],actual[1]) && near(ends[1],actual[0]))
                }).count();
                assert_eq!(matches,1,"each source isocurve must appear exactly once");
            }
            let after=controller.document().clone();controller.undo().unwrap();assert_eq!(controller.document(),&before);
            controller.redo().unwrap();assert_eq!(controller.document(),&after);
        }}
    }

    #[test]
    fn extract_wireframe_after_remesh_pressure_keeps_redo_and_scene() {
        let (mut controller,id)=controller_with_topology_proof();
        controller.submit("Point").unwrap();controller.submit("30,30,30").unwrap();controller.undo().unwrap();
        controller.request_topology_remesh_for(id,0.04,0.3,16*1024*1024,8*1024*1024,8*1024*1024).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {
            if let Some(result)=controller.poll_topology_remesh() {result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));
        }
        controller.select_objects(&[id]).unwrap();
        let before=controller.document().clone();let publication=controller.scene().unwrap().publication_id();
        let undo=controller.coordinator.undo_len();let redo=controller.coordinator.redo_len();
        let pressure=controller.coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,controller.scene_cache_usage().unwrap()).unwrap();
        assert!(controller.submit("ExtractWireframe").is_err());
        assert_eq!(controller.document(),&before);assert_eq!(controller.scene().unwrap().publication_id(),publication);
        assert_eq!(controller.coordinator.undo_len(),undo);assert_eq!(controller.coordinator.redo_len(),redo);
        drop(pressure);
        controller.submit("ExtractWireframe").unwrap();
        assert_eq!(controller.document().entities.values().filter(|entity|entity.label=="ExtractWire").count(),24);
    }

    #[test]
    fn extract_wireframe_multiple_roots_respects_each_published_representation() {
        use om9_core::evaluation::{evaluate_curve,Parameter};
        for menu in [false,true] {
            let (mut controller,box_id,sphere_id)=controller_with_two_topology_proofs();
            controller.submit("Point").unwrap();controller.submit("30,30,30").unwrap();controller.undo().unwrap();
            let before=controller.document().clone();let limits=*controller.coordinator.limits();
            let Geometry::Brep(sphere)=&before.entities[&sphere_id].geometry else {panic!("sphere")};
            let sphere_edges=om9_kernel::editing::all_source_edges_nurbs_admitted(sphere,&limits,|_,_|Ok(())).unwrap().len();
            controller.request_topology_remesh_for(box_id,0.04,0.3,16*1024*1024,8*1024*1024,8*1024*1024).unwrap();
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
            loop {
                if let Some(result)=controller.poll_topology_remesh() {result.unwrap();break;}
                assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));
            }
            controller.select_objects(&[box_id,sphere_id]).unwrap();
            if menu {controller.dispatch("OM9-CURVE-054",InvocationOrigin::Menu).unwrap();}
            else {controller.submit("ExtractWireframe").unwrap();}
            let outputs:Vec<_>=controller.document().entities.values().filter(|entity|entity.label=="ExtractWire").collect();
            assert_eq!(outputs.len(),24+sphere_edges);
            let mut box_curves=0;let mut sphere_curves=0;
            for entity in outputs {
                let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
                let points=[0.,0.5,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]);
                if points.iter().all(|p|p[0]>=-1e-6 && p[0]<=4.+1e-6) {
                    box_curves+=1;
                    for p in points {assert!(p[1]>=-1e-6 && p[1]<=5.+1e-6 && p[2]>=-1e-6 && p[2]<=6.+1e-6);}
                } else {
                    sphere_curves+=1;
                    for p in points {assert!(((p[0]-10.).powi(2)+p[1].powi(2)+p[2].powi(2)-9.).abs()<1e-5,"{p:?}");}
                }
            }
            assert_eq!((box_curves,sphere_curves),(24,sphere_edges));
            let after=controller.document().clone();controller.undo().unwrap();assert_eq!(controller.document(),&before);
            controller.redo().unwrap();assert_eq!(controller.document(),&after);
        }
    }

    #[test]
    fn source_wire_worker_commits_only_when_polled_and_keeps_history() {
        let (mut controller,id)=controller_with_topology_proof();
        controller.submit("Point").unwrap();controller.submit("30,30,30").unwrap();controller.undo().unwrap();
        controller.select_objects(&[id]).unwrap();let before=controller.document().clone();
        let undo=controller.coordinator.undo_len();let redo=controller.coordinator.redo_len();
        controller.request_brep_wireframe_worker().unwrap();
        assert!(controller.brep_wireframe_worker_pending());assert!(controller.is_running());
        assert_eq!(controller.document(),&before);assert_eq!(controller.coordinator.redo_len(),redo);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {
            if let Some(result)=controller.poll_brep_wireframe_worker() {result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(!controller.is_running());
        assert_eq!(controller.document().entities.values().filter(|e|e.label=="ExtractWire").count(),12);
        let after=controller.document().clone();assert_eq!(controller.coordinator.undo_len(),undo+1);
        controller.undo().unwrap();assert_eq!(controller.document(),&before);
        controller.redo().unwrap();assert_eq!(controller.document(),&after);
    }

    #[test]
    fn source_wire_worker_refuses_changed_selection_and_cancel_keeps_document() {
        let (mut controller,id)=controller_with_topology_proof();
        controller.submit("Point").unwrap();controller.submit("30,30,30").unwrap();controller.undo().unwrap();
        controller.select_objects(&[id]).unwrap();let before=controller.document().clone();
        let undo=controller.coordinator.undo_len();let redo=controller.coordinator.redo_len();
        let baseline=controller.coordinator.external_reserved_bytes(MemoryClass::Kernel);
        controller.request_brep_wireframe_worker().unwrap();
        controller.select_objects(&[]).unwrap();controller.select_objects(&[id]).unwrap();
        assert!(matches!(controller.poll_brep_wireframe_worker(),Some(Err(Error::Stale(_)))));
        assert_eq!(controller.document(),&before);assert_eq!(controller.coordinator.undo_len(),undo);assert_eq!(controller.coordinator.redo_len(),redo);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        while controller.coordinator.external_reserved_bytes(MemoryClass::Kernel)>baseline
            || controller.wireframe_worker_active.load(std::sync::atomic::Ordering::Acquire) {
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));
        }
        controller.request_brep_wireframe_worker().unwrap();controller.cancel();
        assert!(!controller.brep_wireframe_worker_pending());assert_eq!(controller.document(),&before);
        assert_eq!(controller.coordinator.undo_len(),undo);assert_eq!(controller.coordinator.redo_len(),redo);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        while controller.coordinator.external_reserved_bytes(MemoryClass::Kernel)>baseline
            || controller.wireframe_worker_active.load(std::sync::atomic::Ordering::Acquire) {
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    #[test]
    fn source_wire_worker_uses_remesh_and_refuses_memory_without_publication() {
        let (mut controller,id)=controller_with_topology_proof();
        controller.submit("Point").unwrap();controller.submit("30,30,30").unwrap();controller.undo().unwrap();
        controller.request_topology_remesh_for(id,0.04,0.3,16*1024*1024,8*1024*1024,8*1024*1024).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {if let Some(result)=controller.poll_topology_remesh(){result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));}
        controller.select_objects(&[id]).unwrap();
        let before=controller.document().clone();let publication=controller.scene().unwrap().publication_id();
        let undo=controller.coordinator.undo_len();let redo=controller.coordinator.redo_len();
        let mut pressure=controller.coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,controller.scene_cache_usage().unwrap()).unwrap();
        pressure.shrink_to(pressure.bytes()-8192).unwrap();
        controller.request_brep_wireframe_worker().unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {if let Some(result)=controller.poll_brep_wireframe_worker(){assert!(matches!(result,Err(Error::Budget(_))));break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));}
        assert_eq!(controller.document(),&before);assert_eq!(controller.scene().unwrap().publication_id(),publication);
        assert_eq!(controller.coordinator.undo_len(),undo);assert_eq!(controller.coordinator.redo_len(),redo);
        drop(pressure);controller.request_brep_wireframe_worker().unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {if let Some(result)=controller.poll_brep_wireframe_worker(){result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));}
        assert_eq!(controller.document().entities.values().filter(|e|e.label=="ExtractWire").count(),24);
        controller.undo().unwrap();assert_eq!(controller.document(),&before);
    }

    #[test]
    fn source_wire_worker_rejects_replaced_scene_at_same_epoch() {
        let (mut controller,id)=controller_with_topology_proof();controller.select_objects(&[id]).unwrap();
        let before=controller.document().clone();let epoch=controller.coordinator.epoch();
        let undo=controller.coordinator.undo_len();let redo=controller.coordinator.redo_len();
        controller.request_brep_wireframe_worker().unwrap();
        controller.replace_scene_same_epoch_for_test(0.).unwrap();assert_eq!(controller.coordinator.epoch(),epoch);
        assert!(matches!(controller.poll_brep_wireframe_worker(),Some(Err(Error::Stale(_)))));
        assert_eq!(controller.document(),&before);assert_eq!(controller.coordinator.undo_len(),undo);assert_eq!(controller.coordinator.redo_len(),redo);
    }

    #[test]
    fn topology_remesh_targets_two_attested_roots_without_replacing_the_other() {
        let (mut controller, first_id, second_id) = controller_with_two_topology_proofs();
        let before_first = controller.scene().unwrap().objects.iter().find(|o| o.root == first_id).unwrap().buffers.clone();
        let before_second = controller.scene().unwrap().objects.iter().find(|o| o.root == second_id).unwrap().buffers.clone();
        let run = |controller: &mut Controller, id| {
            controller.request_topology_remesh_for(id, 0.04, 0.3, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline, "remesh timed out");
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        };
        run(&mut controller, first_id);
        let after_first = controller.scene().unwrap().objects.iter().find(|o| o.root == first_id).unwrap().buffers.clone();
        assert!(!after_first.ptr_eq(&before_first));
        assert!(controller.scene().unwrap().objects.iter().find(|o| o.root == second_id).unwrap().buffers.ptr_eq(&before_second));
        run(&mut controller, second_id);
        assert!(controller.scene().unwrap().objects.iter().find(|o| o.root == first_id).unwrap().buffers.ptr_eq(&after_first));
        assert!(!controller.scene().unwrap().objects.iter().find(|o| o.root == second_id).unwrap().buffers.ptr_eq(&before_second));
        let camera = om9_scene::camera::Camera::orthographic([5., 2., 30.], [5., 2., 0.], [0., 1., 0.], 20.).unwrap();
        let viewport = om9_scene::camera::Viewport { origin: [0., 0.], size: [1000., 1000.] };
        for (id, point) in [(first_id, [2., 2.5, 6.]), (second_id, [10., 0., 3.])] {
            let cursor = camera.project(point, viewport).unwrap().unwrap().screen;
            let hit = controller.scene().unwrap().pick_kind(cursor, &camera, viewport, 1.,
                om9_scene::scene::PickIntent::Select, om9_scene::scene::PickKind::Face).unwrap().unwrap();
            assert_eq!(hit.source.object, id);
            let Geometry::Brep(shape) = &controller.document().entities[&id].geometry else { panic!("BRep expected") };
            let om9_scene::scene::Subelement::BrepFace(face) = hit.subset else { panic!("face expected") };
            assert!(shape.topology().faces.contains(&face));
        }
        controller.submit("Point").unwrap(); controller.submit("1,2,3").unwrap();
        assert!(controller.topology_proof_for(Some(first_id)).is_some());
        assert!(controller.topology_proof_for(Some(second_id)).is_some());
        run(&mut controller, first_id);
    }

    #[test]
    fn topology_remesh_pressure_reclaims_only_idle_other_proofs() {
        let (mut controller, first_id, second_id) = controller_with_two_topology_proofs();
        let target_pin = controller.topology_proof_for(Some(first_id)).unwrap();
        let pinned = controller.topology_proof_for(Some(second_id)).unwrap();
        controller.coordinator.reclaim_optional_memory(u64::MAX).unwrap();
        assert!(controller.topology_proof_for(Some(second_id)).is_some(), "pinned proof must survive");
        drop(pinned);
        let before = controller.coordinator.snapshot();
        let scene = controller.scene().unwrap().publication_id();
        let held = controller.scene_cache_usage().unwrap();
        let mut pressure = controller.coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Export, None, held).unwrap();
        let cpu_before = controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache);
        const SLACK: u64 = 64 * 1024;
        let validation_scratch = controller.document().validation_scratch_usage(controller.coordinator.limits()).unwrap().bytes;
        pressure.shrink_to(pressure.bytes() - SLACK - validation_scratch).unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let attempted = controller.coordinator.reserve_external_memory_with_held(MemoryClass::Upload,
            SLACK + 1, physical, controller.scene_cache_usage().unwrap()).unwrap();
        assert!(controller.topology_proof_for(Some(first_id)).is_some());
        assert!(controller.topology_proof_for(Some(second_id)).is_none());
        assert!(controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache) < cpu_before,
            "retiring an idle proof must actually release its reservation");
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.scene().unwrap().publication_id(), scene);
        drop(attempted);
        drop(pressure);
        drop(target_pin);
        let resumed = controller.reserve_remesh_budget(MemoryClass::Kernel, 1, first_id).unwrap();
        drop(resumed);
    }

    #[test]
    fn topology_cache_insert_reclaims_an_idle_entry_under_pressure() {
        let (mut controller, first_id, second_id) = controller_with_two_topology_proofs();
        let first = controller.topology_proof_for(Some(first_id)).unwrap();
        let second = controller.topology_proof_for(Some(second_id)).unwrap();
        drop(controller.topology_cache.take());
        controller.coordinator.reclaim_optional_memory(u64::MAX).unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let cache = topology_cache::TopologyProofCache::new(&controller.coordinator, held, physical).unwrap();
        let first_owner = Arc::downgrade(&first);
        cache.insert(&controller.coordinator, first, held, physical).unwrap();
        assert_eq!(first_owner.strong_count(), 1, "the first proof must be idle");
        let before = controller.coordinator.snapshot();
        let scene = controller.scene().unwrap().publication_id();
        let mut pressure = controller.coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Export, physical, held).unwrap();
        let scratch = controller.document().validation_scratch_usage(controller.coordinator.limits()).unwrap().bytes;
        pressure.shrink_to(pressure.bytes() - scratch).unwrap();
        let inserted = cache.insert(&controller.coordinator, second, held, physical);
        assert!(inserted.is_ok(), "{inserted:?}, first owners={}", first_owner.strong_count());
        assert!(cache.find(Some(first_id), controller.document()).is_none());
        assert!(cache.find(Some(second_id), controller.document()).is_some());
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.scene().unwrap().publication_id(), scene);
        drop(pressure);
    }

    #[test]
    fn topology_cache_skips_absent_proof_and_can_reclaim_it_before_undo() {
        let (mut controller, first_id, second_id) = controller_with_two_topology_proofs();
        let first = Arc::downgrade(&controller.topology_proof_for(Some(first_id)).unwrap());
        let second = controller.topology_proof_for(Some(second_id)).unwrap();
        controller.delete_objects(&[first_id]).unwrap();
        assert!(controller.topology_proof_for(Some(first_id)).is_none());
        assert_eq!(controller.topology_proof_for(None).unwrap().stamp().object, second_id);
        assert!(first.upgrade().is_some(), "absent proof should remain available to Undo");
        controller.coordinator.reclaim_optional_memory(u64::MAX).unwrap();
        assert!(first.upgrade().is_none(), "an absent idle proof can be reclaimed");
        controller.undo().unwrap();
        assert!(controller.document().entities.contains_key(&first_id));
        assert!(controller.topology_proof_for(Some(first_id)).is_none());
        assert!(controller.topology_proof_for(Some(second_id)).is_some());
        drop(second);
    }

    #[test]
    fn topology_cache_can_reclaim_proof_after_redo_branch_is_discarded() {
        let mut controller = Controller::new(Document::with_template_tolerance("proof branch"), Limits::default()).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let proof = controller.topology_proof_for(None).unwrap();
        let id = proof.stamp().object;
        let weak = Arc::downgrade(&proof);
        drop(proof);
        controller.undo().unwrap();
        assert!(controller.topology_proof_for(Some(id)).is_none());
        assert!(weak.upgrade().is_some(), "Redo still owns the Sphere source");
        controller.submit("Point").unwrap();
        controller.submit("20,20,20").unwrap();
        assert_eq!(controller.coordinator.redo_len(), 0);
        assert!(weak.upgrade().is_some(), "an idle proof may remain in the optional cache");
        controller.coordinator.reclaim_optional_memory(u64::MAX).unwrap();
        assert!(weak.upgrade().is_none(), "the idle proof must be reclaimable");
    }

    #[test]
    fn registered_source_keeps_proof_alive_after_history_branch_is_discarded() {
        let mut controller = Controller::new(Document::with_template_tolerance("proof worker"), Limits::default()).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let proof = controller.topology_proof_for(None).unwrap();
        let object = proof.stamp().object;
        let Geometry::Brep(original) = &controller.document().entities[&object].geometry else { panic!("sphere BRep") };
        let original_display = original.display_liveness();
        let original_topology = original.topology().clone();
        let weak = Arc::downgrade(&proof);
        let source = proof.capture_source(&controller.coordinator).unwrap();
        drop(proof);
        controller.undo().unwrap();
        controller.submit("Point").unwrap();
        controller.submit("20,20,20").unwrap();
        assert_eq!(controller.coordinator.redo_len(), 0);
        assert!(controller.topology_proof_for(None).is_none());
        controller.coordinator.reclaim_optional_memory(u64::MAX).unwrap();
        assert!(weak.upgrade().is_some(), "the registered source owns its proof");
        assert!(!original_display.is_alive(), "discarded history must not pin the old display");
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut tiny = controller.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel, 1, physical,
            controller.scene_cache_usage().unwrap()).unwrap();
        let refused = om9_kernel::brep::capture_source_topology(source.source(), 0.02, 0.3,
            controller.coordinator.limits(), |live, pending| {
                if live.checked_add(pending).ok_or(Error::Overflow("source-only remesh"))? > tiny.bytes() {
                    return Err(Error::Budget("source-only remesh"));
                }
                tiny.set_allocated_bytes(live)?;
                tiny.recheck_pending()
            });
        assert!(refused.is_err(), "a source-only remesh must refuse an insufficient reservation");
        drop(tiny);
        let mut reservation = controller.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel, 16 * 1024 * 1024, physical,
            controller.scene_cache_usage().unwrap()).unwrap();
        let mut account = |live: u64, pending: u64| -> om9_core::Result<()> {
            if live.checked_add(pending).ok_or(Error::Overflow("source-only remesh"))? > reservation.bytes() {
                return Err(Error::Budget("source-only remesh"));
            }
            reservation.set_allocated_bytes(live)?;
            reservation.recheck_pending()
        };
        let candidate = om9_kernel::brep::capture_source_topology(source.source(), 0.02, 0.3,
            controller.coordinator.limits(), &mut account).unwrap();
        let rebound = candidate.into_rebound_for_source(&source, controller.coordinator.limits(), &mut account).unwrap();
        assert_eq!(rebound.brep().topology(), &original_topology,
            "source-only remesh must preserve the attested IDs and adjacency");
        drop(account);
        drop(rebound);
        drop(reservation);
        drop(source);
        controller.coordinator.reclaim_optional_memory(u64::MAX).unwrap();
        assert!(weak.upgrade().is_none(), "the last reader releases the retired proof");
    }

    #[test]
    fn sphere_delete_undo_finds_history_proof_after_optional_cache_drop() {
        let mut controller = Controller::new(Document::new("B0 delete proof"), Limits::default()).unwrap();
        controller.submit("Sphere 5").unwrap();
        let object = *controller.document().entities.keys().next().unwrap();
        let proof = controller.topology_proof_for(Some(object)).expect("product Sphere proof");
        let weak = Arc::downgrade(&proof);
        drop(proof);
        drop(controller.topology_cache.take());
        assert_eq!(controller.delete_objects(&[object]).unwrap(),1);
        assert!(weak.upgrade().is_some(), "history must retain proof after delete");
        controller.undo().unwrap();
        let rebound = controller.topology_proof_for(Some(object)).expect("restored Sphere history proof");
        let mut account = |_,_| Ok(());
        rebound.rebind_unchanged(&controller.coordinator, &mut account).unwrap();
        assert_eq!(rebound.stamp().object,object);
    }

    #[test]
    fn sphere_history_retains_proof_after_cache_drop_and_reuses_it_after_redo() {
        let mut controller = Controller::new(Document::with_template_tolerance("sphere history proof"), Limits::default()).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let proof = controller.topology_proof_for(None).expect("Sphere proof");
        let object = proof.stamp().object;
        let weak = Arc::downgrade(&proof);
        drop(proof);
        drop(controller.topology_cache.take());
        assert!(weak.upgrade().is_some(), "history must retain the proof after the optional cache is gone");
        controller.undo().unwrap();
        assert!(controller.topology_proof_for(Some(object)).is_none());
        controller.redo().unwrap();
        assert!(controller.topology_proof_for(Some(object)).is_some());
        controller.request_topology_remesh_for(object, 0.02, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::yield_now();
        }
        controller.undo().unwrap();
        controller.submit("Point").unwrap();
        controller.submit("20,20,20").unwrap();
        assert_eq!(controller.coordinator.redo_len(), 0);
        assert!(weak.upgrade().is_none(), "discarded history must release its proof");
    }

    #[test]
    fn sphere_history_source_is_reused_by_the_remesh_worker_path() {
        let mut controller = Controller::new(Document::with_template_tolerance("sphere source owner"), Limits::default()).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let proof = controller.topology_proof_for(None).unwrap();
        let source = controller.coordinator.current_evidence_source(&proof.stamp()).expect("history source lease");
        let worker = topology_remesh::capture_proof_source(&controller, proof).unwrap();
        assert!(worker.shares_source_lease(&source), "worker must clone the existing tracked source lease");
        assert_eq!(worker.source(), source.source());
    }

    #[test]
    fn sphere_history_proof_survives_real_optional_cache_admission_refusal() {
        let mut controller = Controller::new(Document::with_template_tolerance("sphere cache refusal"), Limits::default()).unwrap();
        controller.insert_attested_sphere_for_test().unwrap();
        let proof = controller.topology_proof_for(None).unwrap();
        let stamp = proof.stamp();
        drop(controller.topology_cache.take());
        controller.retain_inserted_topology_proof(stamp, proof.clone()).unwrap();
        let before = controller.coordinator.snapshot();
        let held = controller.scene_cache_usage().unwrap();
        let pressure = controller.coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, held).unwrap();
        assert!(controller.cache_topology_proof(proof.clone()).is_err(), "cache must refuse under real account pressure");
        assert_eq!(controller.coordinator.snapshot(), before);
        assert!(controller.topology_cache.is_none());
        drop(pressure);
        drop(before);
        drop(proof);
        assert!(controller.topology_proof_for(Some(stamp.object)).is_some(), "history retains proof independently of cache");
        controller.request_topology_remesh_for(stamp.object, 0.02, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::yield_now();
        }
    }

    #[test]
    fn production_sphere_keeps_proof_when_optional_cache_refuses() {
        let mut controller = Controller::new(Document::with_template_tolerance("sphere cache rejection"), Limits::default()).unwrap();
        let result = TEST_REFUSE_TOPOLOGY_CACHE.with(|refuse| {
            refuse.set(true);
            let result = controller.solid_sphere_points([0., 0., 0.], 5.);
            refuse.set(false);
            result
        });
        result.unwrap();
        assert!(controller.topology_cache.is_none());
        let proof = controller.topology_proof_for(None).expect("production Sphere retained proof without cache");
        let object = proof.stamp().object;
        drop(proof);
        controller.undo().unwrap();
        controller.redo().unwrap();
        assert!(controller.topology_proof_for(Some(object)).is_some());
        controller.request_topology_remesh_for(object, 0.02, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::yield_now();
        }
    }

    #[test]
    fn remesh_retries_sphere_proof_retention_after_initial_refusal_and_intervening_edit() {
        let mut controller = Controller::new(Document::with_template_tolerance("sphere proof retry"), Limits::default()).unwrap();
        let result = TEST_REFUSE_SPHERE_HISTORY.with(|refuse| {
            refuse.set(true);
            let result = controller.solid_sphere_points([0., 0., 0.], 5.);
            refuse.set(false);
            result
        });
        result.unwrap();
        let object = controller.topology_proof_for(None).unwrap().stamp().object;
        controller.submit("Point").unwrap();
        controller.submit("20,20,20").unwrap();
        let stamp = controller.document().stamp(object).unwrap();
        assert!(controller.coordinator.history_evidence_for::<om9_kernel::brep::SharedTopologyProof>(&stamp).is_none());
        let document_before = controller.document().clone();
        let undo_before = controller.coordinator.undo_len();
        controller.request_topology_remesh_for(object, 0.02, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::yield_now();
        }
        assert_eq!(controller.document(), &document_before);
        assert_eq!(controller.coordinator.undo_len(), undo_before);
        assert!(controller.coordinator.history_evidence_for::<om9_kernel::brep::SharedTopologyProof>(&stamp).is_some());
        drop(controller.topology_cache.take());
        assert!(controller.topology_proof_for(Some(object)).is_some());
        controller.undo().unwrap();
        controller.redo().unwrap();
        assert!(controller.topology_proof_for(Some(object)).is_some());
    }

    #[test]
    fn sphere_native_proof_survives_trim_when_later_history_restores_same_source() {
        use om9_core::resource::{Resource, ResourceKind};
        use om9_core::transaction::UndoPolicy;
        let limits = Limits::default();
        let resource = Resource::from_bytes("pressure", ResourceKind::OpaqueBinary, &vec![7; 2 * 1024 * 1024], &limits).unwrap();
        let resource_id = resource.id;
        let mut document = Document::with_template_tolerance("sphere trim proof");
        document.resources.insert(resource_id, resource);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let proof = controller.topology_proof_for(None).unwrap();
        let object = proof.stamp().object;
        let weak = Arc::downgrade(&proof);
        drop(proof);
        drop(controller.topology_cache.take());
        let document = controller.document().id;
        let revision = controller.document().revision;
        let held = controller.scene_cache_usage().unwrap();
        let request = TransactionRequest::new("delete resource", vec![Mutation::ResourceDelete {
            document, expected_revision: revision, resource: resource_id,
        }]);
        let prepared = controller.coordinator.prepare_with_held_usage(request, held).unwrap();
        controller.commit_with_candidate_scene(prepared, held).unwrap();
        controller.coordinator.set_undo_policy(UndoPolicy { minimum_steps: 1, max_memory_mb: 1 }).unwrap();
        assert_eq!(controller.coordinator.undo_len(), 1, "Sphere insertion must have been trimmed");
        assert!(weak.upgrade().is_some());
        controller.undo().unwrap();
        assert!(controller.topology_proof_for(Some(object)).is_some());
        controller.request_topology_remesh_for(object, 0.02, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::yield_now();
        }
    }

    #[test]
    fn sphere_native_proof_survives_history_clear_while_current_then_releases_on_delete() {
        let mut controller = Controller::new(Document::with_template_tolerance("current sphere proof"), Limits::default()).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let proof = controller.topology_proof_for(None).unwrap();
        let object = proof.stamp().object;
        let weak = Arc::downgrade(&proof);
        drop(proof);
        drop(controller.topology_cache.take());
        let document = controller.document().id;
        let point = Entity::new("point", Geometry::Point([20., 20., 20.]), None);
        let mut request = TransactionRequest::new("unrecorded point", vec![Mutation::Insert { document, entity: point }]);
        request.recording = false;
        let held = controller.scene_cache_usage().unwrap();
        let prepared = controller.coordinator.prepare_with_held_usage(request, held).unwrap();
        controller.commit_with_candidate_scene(prepared, held).unwrap();
        assert_eq!((controller.coordinator.undo_len(), controller.coordinator.redo_len()), (0, 0));
        assert!(controller.topology_proof_for(Some(object)).is_some());
        controller.request_topology_remesh_for(object, 0.02, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::yield_now();
        }
        let source = controller.document().stamp(object).unwrap();
        let mut request = TransactionRequest::new("unrecorded delete", vec![Mutation::Delete { source }]);
        request.recording = false;
        let held = controller.scene_cache_usage().unwrap();
        let prepared = controller.coordinator.prepare_with_held_usage(request, held).unwrap();
        controller.commit_with_candidate_scene(prepared, held).unwrap();
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn sphere_native_proof_survives_close_of_other_shared_history_document() {
        let limits = Limits::default();
        let mut controller = Controller::new(Document::with_template_tolerance("sphere remains"), limits).unwrap();
        let sphere_document = controller.document().id;
        let other = Document::with_template_tolerance("other document");
        let other_id = other.id;
        controller.coordinator.open_document(other).unwrap();
        controller.refresh_scene().unwrap();
        let snapshot = om9_kernel::creation::primitive_with_topology(
            om9_kernel::creation::Primitive::Sphere { radius_mm: 5. }, 0.1, &limits, |_, _| Ok(())).unwrap();
        let (brep, proof) = snapshot.into_parts();
        let entity = Entity::new("sphere", Geometry::Brep(brep), None);
        let object = entity.id;
        let point = Entity::new("point", Geometry::Point([20., 20., 20.]), None);
        let held = controller.scene_cache_usage().unwrap();
        let request = TransactionRequest::new("two documents", vec![
            Mutation::Insert { document: sphere_document, entity },
            Mutation::Insert { document: other_id, entity: point },
        ]);
        let prepared = controller.coordinator.prepare_with_held_usage(request, held).unwrap();
        controller.commit_with_candidate_scene(prepared, held).unwrap();
        let bytes = proof.storage_bytes().unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut reservation = controller.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical, held).unwrap();
        reservation.set_allocated_bytes(bytes).unwrap();
        let stamp = controller.document().stamp(object).unwrap();
        let shared = controller.prepare_shared_topology_proof(stamp, proof, reservation).unwrap();
        controller.retain_inserted_topology_proof(stamp, shared.clone()).unwrap();
        drop(shared);
        controller.coordinator.close_document(other_id).unwrap();
        assert_eq!(controller.coordinator.undo_len(), 0);
        controller.refresh_scene().unwrap();
        assert!(controller.topology_proof_for(Some(object)).is_some());
        controller.request_topology_remesh_for(object, 0.02, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::yield_now();
        }
    }

    #[test]
    fn two_sphere_native_proofs_survive_trim_into_the_same_history_entry() {
        use om9_core::resource::{Resource, ResourceKind};
        use om9_core::transaction::UndoPolicy;
        let limits = Limits::default();
        let resource = Resource::from_bytes("pressure", ResourceKind::OpaqueBinary, &vec![7; 2 * 1024 * 1024], &limits).unwrap();
        let resource_id = resource.id;
        let mut document = Document::with_template_tolerance("two sphere trim proofs");
        document.resources.insert(resource_id, resource);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let first = controller.topology_proof_for(None).unwrap().stamp().object;
        controller.solid_sphere_points([20., 0., 0.], 5.).unwrap();
        let second = controller.document().entities.keys().copied().find(|id| *id != first).unwrap();
        assert!(controller.topology_proof_for(Some(second)).is_some());
        drop(controller.topology_cache.take());
        assert_eq!(controller.topology_proof_for(None).unwrap().stamp().object, first.min(second));
        let document = controller.document().id;
        let revision = controller.document().revision;
        let held = controller.scene_cache_usage().unwrap();
        let request = TransactionRequest::new("delete resource", vec![Mutation::ResourceDelete {
            document, expected_revision: revision, resource: resource_id,
        }]);
        let prepared = controller.coordinator.prepare_with_held_usage(request, held).unwrap();
        controller.commit_with_candidate_scene(prepared, held).unwrap();
        controller.coordinator.set_undo_policy(UndoPolicy { minimum_steps: 1, max_memory_mb: 1 }).unwrap();
        assert_eq!(controller.coordinator.undo_len(), 1);
        controller.undo().unwrap();
        assert!(controller.topology_proof_for(Some(first)).is_some(), "first Sphere proof must survive");
        assert!(controller.topology_proof_for(Some(second)).is_some(), "second Sphere proof must survive");
        for object in [first, second] {
            controller.request_topology_remesh_for(object, 0.02, 0.3,
                16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            loop {
                if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline, "remesh timed out");
                std::thread::yield_now();
            }
        }
        controller.submit("Point").unwrap();
        controller.submit("40,40,40").unwrap();
        assert_eq!(controller.coordinator.redo_len(), 0);
        assert!(controller.topology_proof_for(Some(first)).is_some(), "branching must preserve the first proof");
        assert!(controller.topology_proof_for(Some(second)).is_some(), "branching must preserve the second proof");
    }

    #[test]
    fn topology_cache_retains_last_matching_document_after_original_is_dropped() {
        let (controller, id) = controller_with_topology_proof();
        let cache = controller.topology_cache.as_ref().unwrap().clone();
        let matching = Arc::new(controller.document().clone());
        cache.retain(&matching);
        let mut changed = (*matching).clone();
        let replacement = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Sphere { radius_mm: 7. },
            om9_kernel::creation::DEFAULT_DEFLECTION_MM, controller.coordinator.limits()).unwrap();
        changed.entities.get_mut(&id).unwrap().geometry = Geometry::Brep(replacement);
        drop(controller);
        cache.retain(&changed);
        assert!(cache.find(Some(id), &matching).is_some(),
            "the last matching retained document should preserve its optional proof");
    }

    #[test]
    fn topology_cache_insert_respects_a_pinned_proof_and_can_retry() {
        let (mut controller, first_id, second_id) = controller_with_two_topology_proofs();
        let first = controller.topology_proof_for(Some(first_id)).unwrap();
        let pinned = first.clone();
        let second = controller.topology_proof_for(Some(second_id)).unwrap();
        drop(controller.topology_cache.take());
        controller.coordinator.reclaim_optional_memory(u64::MAX).unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let cache = topology_cache::TopologyProofCache::new(&controller.coordinator, held, physical).unwrap();
        cache.insert(&controller.coordinator, first, held, physical).unwrap();
        let before = controller.coordinator.snapshot();
        let mut pressure = controller.coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Export, physical, held).unwrap();
        let scratch = controller.document().validation_scratch_usage(controller.coordinator.limits()).unwrap().bytes;
        pressure.shrink_to(pressure.bytes() - scratch).unwrap();
        assert!(matches!(cache.insert(&controller.coordinator, second.clone(), held, physical), Err(Error::Budget(_))));
        assert!(cache.find(Some(first_id), controller.document()).is_some());
        assert!(cache.find(Some(second_id), controller.document()).is_none());
        assert_eq!(controller.coordinator.snapshot(), before);
        drop(pressure);
        cache.insert(&controller.coordinator, second, held, physical).unwrap();
        assert!(cache.find(Some(first_id), controller.document()).is_some());
        assert!(cache.find(Some(second_id), controller.document()).is_some());
        drop(pinned);
    }

    #[test]
    fn topology_cache_concurrent_inserts_do_not_duplicate_a_proof() {
        let (mut controller, first_id, second_id) = controller_with_two_topology_proofs();
        let first = controller.topology_proof_for(Some(first_id)).unwrap();
        let second = controller.topology_proof_for(Some(second_id)).unwrap();
        drop(controller.topology_cache.take());
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let cache = topology_cache::TopologyProofCache::new(&controller.coordinator, held, physical).unwrap();
        cache.insert(&controller.coordinator, first, held, physical).unwrap();
        assert_eq!(Arc::strong_count(&second), 1);
        let barrier = std::sync::Barrier::new(3);
        std::thread::scope(|scope| {
            for _ in 0..2 {
                let proof = second.clone();
                let cache = &cache;
                let coordinator = &controller.coordinator;
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    cache.insert(coordinator, proof, held, physical).unwrap();
                });
            }
            barrier.wait();
        });
        assert_eq!(Arc::strong_count(&second), 2, "one caller and one cached entry");
        assert!(cache.find(Some(first_id), controller.document()).is_some());
        assert!(cache.find(Some(second_id), controller.document()).is_some());
    }

    #[test]
    fn topology_remesh_stale_publication_does_not_replace_current_scene() {
        let (mut controller, id) = controller_with_topology_proof();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let display = wait_remesh(&mut job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        let prepared = display.prepare_scene(&controller, 8 * 1024 * 1024).unwrap();
        controller.replace_scene_same_epoch_for_test(0.).unwrap();
        let current = controller.scene().unwrap().publication_id();
        assert!(matches!(prepared.publish(&mut controller), Err(Error::Stale("topology remesh scene"))));
        assert_eq!(controller.scene().unwrap().publication_id(), current);
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
    }

    #[test]
    fn topology_remesh_builds_display_on_worker_and_publishes() {
        let (mut controller, id) = controller_with_topology_proof();
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let kernel = controller.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 16 * 1024 * 1024, physical, held).unwrap();
        let display = controller.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 8 * 1024 * 1024, physical, held).unwrap();
        let ui_thread = std::thread::current().id();
        let old_scene = controller.scene().unwrap().publication_id();
        let mut job = controller.start_topology_remesh_with_display(0.02, 0.3, kernel, display).unwrap();
        let receipt = wait_remesh(&mut job).unwrap();
        assert_ne!(receipt.prepared_on_thread().unwrap(), ui_thread);
        let prepared = receipt.prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        let RenderBuffers::Mesh { triangles, .. } = prepared.buffers() else { panic!("mesh expected") };
        assert!(!triangles.is_empty());
        prepared.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        assert_ne!(controller.scene().unwrap().publication_id(), old_scene);
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
    }

    #[test]
    fn topology_remesh_worker_display_budget_refusal_keeps_document() {
        let (mut controller, id) = controller_with_topology_proof();
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let kernel = controller.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, 16 * 1024 * 1024, physical, held).unwrap();
        let display = controller.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, 1, physical, held).unwrap();
        let old_scene = controller.scene().unwrap().publication_id();
        let mut job = controller.start_topology_remesh_with_display(0.02, 0.3, kernel, display).unwrap();
        assert!(matches!(wait_remesh(&mut job), Err(Error::Budget("remesh display buffer"))));
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), 0);
        assert_eq!(controller.scene().unwrap().publication_id(), old_scene);
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
        let mut retry = start_remesh(&mut controller, 16 * 1024 * 1024);
        assert!(wait_remesh(&mut retry).is_ok(), "worker failure must retain reusable proof");
    }

    #[test]
    fn topology_remesh_proof_allows_only_one_worker_and_releases_permit() {
        let (controller, _) = controller_with_topology_proof();
        let proof = controller.topology_proof_for(None).unwrap();
        let first = proof.acquire_worker().unwrap();
        assert!(matches!(proof.acquire_worker(), Err(Error::Busy)));
        drop(first);
        assert!(proof.acquire_worker().is_ok());
    }

    #[test]
    fn topology_remesh_scheduler_publishes_once_and_rejects_duplicate() {
        let (mut controller, id) = controller_with_topology_proof();
        let original = controller.scene().unwrap().publication_id();
        controller.request_topology_remesh(0.02, 0.3, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        assert!(controller.topology_remesh_pending());
        assert!(matches!(controller.request_topology_remesh(0.02, 0.3, 1, 1, 1), Err(Error::Busy)));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh scheduler timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(!controller.topology_remesh_pending());
        assert_ne!(controller.scene().unwrap().publication_id(), original);
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
        controller.undo().unwrap();
        assert!(!controller.document().entities.contains_key(&id));
    }

    #[test]
    fn topology_remesh_scheduler_cancel_and_undo_do_not_publish() {
        let (mut controller, id) = controller_with_topology_proof();
        let original = controller.scene().unwrap().publication_id();
        controller.request_topology_remesh(0.02, 0.3, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        controller.cancel_topology_remesh();
        assert!(!controller.topology_remesh_pending());
        assert!(controller.poll_topology_remesh().is_none());
        assert_eq!(controller.scene().unwrap().publication_id(), original);
        assert!(controller.document().entities.contains_key(&id));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Ok(permit) = controller.topology_proof_for(None).unwrap().acquire_worker() { drop(permit); break; }
            assert!(std::time::Instant::now() < deadline, "cancelled worker did not stop");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let mut retry = start_remesh(&mut controller, 16 * 1024 * 1024);
        assert!(wait_remesh(&mut retry).is_ok(), "cancellation must retain reusable proof");
        controller.undo().unwrap();
        assert!(!controller.document().entities.contains_key(&id));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while controller.coordinator.external_reserved_bytes(MemoryClass::Kernel) != 0 {
            assert!(std::time::Instant::now() < deadline, "cancelled remesh did not release reservation");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    #[test]
    fn topology_remesh_scheduler_undo_cancels_pending_job() {
        let (mut controller, id) = controller_with_topology_proof();
        controller.request_topology_remesh(0.02, 0.3, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        controller.undo().unwrap();
        assert!(!controller.topology_remesh_pending());
        assert!(controller.poll_topology_remesh().is_none());
        assert!(!controller.document().entities.contains_key(&id));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while controller.coordinator.external_reserved_bytes(MemoryClass::Kernel) != 0 {
            assert!(std::time::Instant::now() < deadline, "undo did not release worker reservation");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    #[test]
    fn topology_remesh_background_export_uses_captured_source() {
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
        let (mut controller, id) = controller_with_topology_proof();
        controller.select_objects(&[id]).unwrap();
        let destination = Cleanup(std::env::temp_dir().join(format!("om9-remesh-job-{}.stl", uuid::Uuid::new_v4())));
        let profile = StlExportProfile { linear_tolerance_mm: 0.01, angular_tolerance_rad: 0.3 };
        controller.export_stl_selected_with_profile(&destination.0, profile).unwrap();
        let expected = std::fs::read(&destination.0).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let held = controller.scene_cache_usage().unwrap();
        let reservation = controller.coordinator.reserve_external_memory_with_held(MemoryClass::Export, 64 * 1024 * 1024, physical, held).unwrap();
        let mut export = StlExportJob::start(capture, destination.0.clone(), profile, reservation).unwrap();
        let old_scene = controller.scene().unwrap().publication_id();
        controller.request_topology_remesh(0.02, 0.3, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_ne!(controller.scene().unwrap().publication_id(), old_scene);
        loop {
            if let Some(result) = export.try_result() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "export timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(std::fs::read(&destination.0).unwrap(), expected);
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.undo_len(), 1);
    }

    #[test]
    fn topology_remesh_keeps_stl_identical_across_display_change_and_undo_redo() {
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }
        let mut controller = Controller::new(Document::with_template_tolerance("remesh export"), Limits::default()).unwrap();
        controller.commit_primitive_optional_topology("OM9-SOLID-014", "sphere", "sphere",
            om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, om9_core::geometry::Transform::identity()).unwrap();
        let id = controller.topology_proof_for(None).unwrap().stamp().object;
        controller.select_objects(&[id]).unwrap();
        let original = controller.document().clone();
        let destination = Cleanup(std::env::temp_dir().join(format!("om9-remesh-export-{}.stl", uuid::Uuid::new_v4())));
        let profile = StlExportProfile { linear_tolerance_mm: 0.05, angular_tolerance_rad: 0.3 };
        let first = controller.export_stl_selected_with_profile(&destination.0, profile).unwrap();
        let original_stl = std::fs::read(&destination.0).unwrap();
        let captured = controller.capture_stl_selected().unwrap();
        let original_mesh = match &controller.document().entities[&id].geometry {
            Geometry::Brep(shape) => shape.display().triangles.len(), _ => panic!("expected BRep"),
        };
        let held = controller.scene_cache_usage().unwrap();
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let reservation = controller.coordinator.reserve_external_memory_with_held(
            MemoryClass::Kernel, 16 * 1024 * 1024, physical, held).unwrap();
        let mut job = controller.start_topology_remesh(0.5, 0.9, reservation).unwrap();
        let display = wait_remesh(&mut job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        assert_ne!(display.candidate(&controller).unwrap().display().triangles.len(), original_mesh);
        display.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        assert_eq!(*controller.document(), original);
        let second = controller.export_stl_selected_with_profile(&destination.0, profile).unwrap();
        assert_eq!(second.triangles, first.triangles);
        assert_eq!(std::fs::read(&destination.0).unwrap(), original_stl);
        let captured_receipt = captured.export_with_profile(&destination.0, profile).unwrap();
        assert_eq!(captured_receipt.triangles, first.triangles);
        assert_eq!(std::fs::read(&destination.0).unwrap(), original_stl);
        controller.undo().unwrap();
        assert!(!controller.document().entities.contains_key(&id));
        controller.redo().unwrap();
        controller.select_objects(&[id]).unwrap();
        let third = controller.export_stl_selected_with_profile(&destination.0, profile).unwrap();
        assert_eq!(third.triangles, first.triangles);
        assert_eq!(std::fs::read(&destination.0).unwrap(), original_stl);
    }

    #[test]
    #[cfg(windows)]
    #[ignore = "requires configured and fingerprint-admitted native 3DM exchange"]
    fn topology_remesh_keeps_3dm_source_and_topology() {
        struct Cleanup(Vec<std::path::PathBuf>);
        impl Drop for Cleanup { fn drop(&mut self) { for path in &self.0 { let _ = std::fs::remove_file(path); } } }
        let (mut controller, id) = controller_with_topology_proof();
        let original = controller.document().clone();
        let Geometry::Brep(source) = &original.entities[&id].geometry else { panic!("expected BRep") };
        let expected = om9_kernel::creation::measure(source, controller.coordinator.limits()).unwrap();
        let nonce = uuid::Uuid::new_v4();
        let paths = Cleanup(vec![std::env::temp_dir().join(format!("om9-remesh-before-{nonce}.3dm")),
            std::env::temp_dir().join(format!("om9-remesh-after-{nonce}.3dm"))]);
        controller.export_3dm(&paths.0[0]).unwrap();
        let mut job = start_remesh(&mut controller, 16 * 1024 * 1024);
        let display = wait_remesh(&mut job).unwrap().prepare_display(&controller, 8 * 1024 * 1024).unwrap();
        display.prepare_scene(&controller, 8 * 1024 * 1024).unwrap().publish(&mut controller).unwrap();
        controller.export_3dm(&paths.0[1]).unwrap();
        assert_eq!(*controller.document(), original);
        for path in &paths.0 {
            let mut imported = Controller::new(Document::with_template_tolerance("roundtrip"), Limits::default()).unwrap();
            imported.import_3dm(path).unwrap();
            assert_eq!(imported.document().entities.len(), 1);
            let entity = imported.document().entities.values().next().unwrap();
            let Geometry::Brep(brep) = &entity.geometry else { panic!("3DM did not return editable BRep") };
            let got = om9_kernel::creation::measure(brep, imported.coordinator.limits()).unwrap();
            assert_eq!((got.faces, got.edges, got.vertices, got.solids, got.shells, got.closed),
                (expected.faces, expected.edges, expected.vertices, expected.solids, expected.shells, expected.closed));
            assert!((got.volume_mm3 - expected.volume_mm3).abs() < 1e-6);
            assert!((got.area_mm2 - expected.area_mm2).abs() < 1e-6);
            for (a, b) in got.bounds_mm.iter().zip(expected.bounds_mm) { assert!((*a - b).abs() < 1e-6); }
        }
    }

    #[test]
    fn experimental_primitive_path_keeps_optional_proof_without_changing_undo() {
        let mut controller = Controller::new(Document::with_template_tolerance("primitive proofs"), Limits::default()).unwrap();
        controller.commit_primitive_optional_topology("OM9-SOLID-014", "sphere", "sphere",
            om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, om9_core::geometry::Transform::translation([1., 2., 3.])).unwrap();
        let sphere = controller.topology_proof_for(None).expect("sphere proof").stamp().object;
        assert!(controller.document().entities.contains_key(&sphere));
        assert_eq!(controller.coordinator.undo_len(), 1);
        controller.commit_primitive_optional_topology("OM9-SOLID-012", "box", "box",
            om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, om9_core::geometry::Transform::identity()).unwrap();
        let box_id = *controller.document().entities.keys().find(|id| **id != sphere).expect("box");
        assert!(controller.topology_proof_for(Some(box_id)).is_some());
        assert_ne!(sphere, box_id);
        assert_eq!(controller.coordinator.undo_len(), 2);
        controller.undo().unwrap();
        assert!(controller.topology_proof_for(Some(box_id)).is_none());
        assert!(controller.topology_proof_for(Some(sphere)).is_some());
        assert!(!controller.document().entities.contains_key(&box_id));
        assert!(controller.document().entities.contains_key(&sphere));
        controller.redo().unwrap();
        assert!(controller.document().entities.contains_key(&box_id));
    }
    #[test]
    fn optional_proof_refusal_still_creates_the_sphere_with_one_undo() {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) { super::solid::TEST_REFUSE_TOPOLOGY.with(|flag| flag.set(false)); }
        }
        let mut controller = Controller::new(Document::with_template_tolerance("proof fallback"), Limits::default()).unwrap();
        super::solid::TEST_REFUSE_TOPOLOGY.with(|flag| flag.set(true));
        let reset = Reset;
        controller.solid_sphere_points([1., 2., 3.], 3.).unwrap();
        drop(reset);
        assert!(controller.topology_proof_for(None).is_none());
        assert_eq!(controller.document().entities.len(), 1);
        assert_eq!(controller.coordinator.undo_len(), 1);
        controller.undo().unwrap();
        assert!(controller.document().entities.is_empty());
        controller.redo().unwrap();
        assert_eq!(controller.document().entities.len(), 1);
    }
    #[test]
    fn production_sphere_proof_can_drive_remesh() {
        let mut controller = Controller::new(Document::with_template_tolerance("default primitive"), Limits::default()).unwrap();
        controller.solid_sphere_points([1., 2., 3.], 3.).unwrap();
        let id = controller.topology_proof_for(None).expect("production Sphere proof").stamp().object;
        let before = controller.document().clone();
        let scene = controller.scene().unwrap().publication_id();
        controller.request_topology_remesh_for(id, 0.2, 0.5, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_ne!(controller.scene().unwrap().publication_id(), scene);
        assert_eq!(*controller.document(), before);
        controller.solid_box_dimensions([0., 0., 0.], 4., 5., 6., CPlane::world_xy()).unwrap();
        let box_id = *controller.document().entities.keys().find(|object| **object != id).unwrap();
        assert!(controller.topology_proof_for(Some(box_id)).is_some(), "Box retains its independently captured topology proof");
        assert_eq!(controller.coordinator.undo_len(), 2);
        controller.undo().unwrap();
        controller.undo().unwrap();
        assert!(controller.topology_proof_for(Some(id)).is_none());
        assert!(controller.topology_proof_for(None).is_none());
        controller.redo().unwrap();
        assert!(controller.document().entities.contains_key(&id));
        assert!(controller.topology_proof_for(Some(id)).is_some());
        controller.request_topology_remesh_for(id, 0.1, 0.4, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "redo remesh timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    #[test]
    fn previous_remesh_representation_reuses_pins_and_reclaims_unpinned_bytes() {
        use om9_core::transaction::OptionalMemoryCache;
        let (mut controller, id) = controller_with_topology_proof();
        let wait = |controller: &mut Controller| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(5));
            }
        };
        let request = |controller: &mut Controller, profile| controller.request_topology_remesh_for(id, profile, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let before = controller.document().clone();
        let undo = controller.coordinator.undo_len(); let redo = controller.coordinator.redo_len();
        request(&mut controller, 0.04); wait(&mut controller);
        let stamp = controller.document().stamp(id).unwrap();
        let pin = controller.scene().unwrap().root_brep_representation(&stamp).unwrap();
        let buffers = controller.scene().unwrap().objects.iter().find(|o| o.root == id).unwrap().buffers.clone();
        request(&mut controller, 0.1); wait(&mut controller);
        let cache = controller.remesh_representation_cache.as_ref().expect("previous profile cache").clone();
        assert_eq!(cache.reclaim_optional_memory(1).unwrap(), 0, "a live scene reader must pin the cached representation");
        request(&mut controller, 0.04);
        assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Stored { .. })));
        assert!(matches!(controller.poll_topology_remesh(), Some(Ok(()))));
        let object = controller.scene().unwrap().objects.iter().find(|o| o.root == id).unwrap();
        assert!(object.buffers.ptr_eq(&buffers), "stored buffers must be reused");
        assert_eq!(controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().deflection_mm, 0.04);
        assert_eq!(controller.document(), &before); assert_eq!(controller.coordinator.undo_len(), undo); assert_eq!(controller.coordinator.redo_len(), redo);
        drop(pin); drop(buffers);
        let reserved = |controller: &Controller| controller.coordinator.external_reserved_bytes(MemoryClass::Kernel)
            + controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache);
        let old = reserved(&controller);
        let released = cache.reclaim_optional_memory(1).unwrap();
        assert!(released > 0, "the retired second profile should be reclaimable");
        assert_eq!(old - reserved(&controller), released, "reported bytes must equal released reservations");
        assert!(controller.scene().unwrap().has_root_remesh_profile(&stamp, [0.04f64.to_bits(),0.3f64.to_bits()]));
    }

    #[test]
    fn cached_remesh_rebinds_after_placement_edit() {
        let (mut controller,id)=controller_with_topology_proof();
        let wait=|controller:&mut Controller| {
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
            loop {if let Some(result)=controller.poll_topology_remesh(){result.unwrap();break;}
                assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));}
        };
        let request=|controller:&mut Controller,profile|controller.request_topology_remesh_for(id,profile,0.3,
            16*1024*1024,8*1024*1024,8*1024*1024).unwrap();
        request(&mut controller,0.04);wait(&mut controller);
        let old_stamp=controller.document().stamp(id).unwrap();
        let mut wrong_session=controller.scene().unwrap().root_brep_representation(&old_stamp).unwrap();
        let mut invalid_stamp=old_stamp;invalid_stamp.session_generation+=1;
        assert!(matches!(wrong_session.retarget_same_object(invalid_stamp),Err(Error::Stale(_))));
        drop(wrong_session);
        let buffers=controller.scene().unwrap().objects.iter().find(|object|object.root==id).unwrap().buffers.clone();
        request(&mut controller,0.1);wait(&mut controller);
        controller.select_objects(&[id]).unwrap();controller.numeric_move([3.,0.,0.]).unwrap();
        let after_edit=controller.document().clone();let undo=controller.coordinator.undo_len();let redo=controller.coordinator.redo_len();
        let new_stamp=controller.document().stamp(id).unwrap();
        assert_ne!(old_stamp,new_stamp,"fixture must exercise a changed source stamp");
        controller.request_topology_remesh_for(id,0.04,0.3,1,1,8*1024*1024).unwrap();
        assert!(matches!(controller.topology_remesh_job,Some(topology_remesh::PendingTopologyRemesh::Stored { .. })),
            "the same attested source must use its cached representation after a document-only edit");
        assert!(matches!(controller.poll_topology_remesh(),Some(Ok(()))));
        let object=controller.scene().unwrap().objects.iter().find(|object|object.root==id).unwrap();
        assert!(object.buffers.ptr_eq(&buffers));
        assert_eq!(object.display_transform().unwrap().point([0.,0.,0.]).unwrap(),[3.,0.,0.]);
        assert_eq!(controller.scene().unwrap().root_brep_pick_display(&new_stamp).unwrap().deflection_mm,0.04);
        assert_eq!(controller.document(),&after_edit);assert_eq!(controller.coordinator.undo_len(),undo);assert_eq!(controller.coordinator.redo_len(),redo);
        assert!(controller.scene().unwrap().root_brep_pick_display(&old_stamp).is_err());
        for redo_step in [false,true] {
            if redo_step {controller.redo().unwrap();} else {controller.undo().unwrap();}
            let expected=controller.document().clone();let undo=controller.coordinator.undo_len();let redo=controller.coordinator.redo_len();
            let stamp=controller.document().stamp(id).unwrap();
            controller.request_topology_remesh_for(id,0.04,0.3,1,1,8*1024*1024).unwrap();
            assert!(matches!(controller.topology_remesh_job,Some(topology_remesh::PendingTopologyRemesh::Stored { .. })));
            assert!(matches!(controller.poll_topology_remesh(),Some(Ok(()))));
            let object=controller.scene().unwrap().objects.iter().find(|object|object.root==id).unwrap();
            assert!(object.buffers.ptr_eq(&buffers));
            assert_eq!(object.display_transform().unwrap().point([0.,0.,0.]).unwrap(),if redo_step {[3.,0.,0.]} else {[0.,0.,0.]});
            assert_eq!(controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().deflection_mm,0.04);
            assert_eq!(controller.document(),&expected);assert_eq!(controller.coordinator.undo_len(),undo);assert_eq!(controller.coordinator.redo_len(),redo);
        }
        let Geometry::Brep(original)=&controller.document().entities[&id].geometry else {panic!("BRep")};
        let independent=om9_core::brep::Brep::from_native(original.source().to_owned(),original.display().clone(),controller.coordinator.limits()).unwrap();
        let stamp=controller.document().stamp(id).unwrap();let held=controller.scene_cache_usage().unwrap();
        let prepared=controller.coordinator.prepare_with_held_usage(TransactionRequest::new("independent source",vec![
            Mutation::Geometry {source:stamp,geometry:Geometry::Brep(independent)}]),held).unwrap();
        controller.commit_with_candidate_scene(prepared,held).unwrap();
        let expected=controller.document().clone();let publication=controller.scene().unwrap().publication_id();
        let undo=controller.coordinator.undo_len();let redo=controller.coordinator.redo_len();
        controller.request_topology_remesh_for(id,0.04,0.3,1,8*1024*1024,8*1024*1024).unwrap();
        assert!(matches!(controller.topology_remesh_job,Some(topology_remesh::PendingTopologyRemesh::Worker { .. })),
            "an independently allocated source must pass worker validation before cache reuse");
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        let result=loop {if let Some(result)=controller.poll_topology_remesh(){break result;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));};
        assert!(matches!(result,Err(Error::Budget(_))),"{result:?}");
        assert_eq!(controller.document(),&expected);assert_eq!(controller.scene().unwrap().publication_id(),publication);
        assert_eq!(controller.coordinator.undo_len(),undo);assert_eq!(controller.coordinator.redo_len(),redo);
    }

    #[test]
    fn remesh_cache_reuses_profiles_for_distinct_roots_and_preserves_pins() {
        use om9_core::transaction::OptionalMemoryCache;
        let (mut controller, box_id, sphere_id) = controller_with_two_topology_proofs();
        let before = controller.document().clone();
        let (undo, redo) = (controller.coordinator.undo_len(), controller.coordinator.redo_len());
        let run = |controller: &mut Controller, id, profile| {
            controller.request_topology_remesh_for(id, profile, 0.3,
                16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        };
        run(&mut controller, box_id, 0.04);
        let box_stamp = controller.document().stamp(box_id).unwrap();
        let pinned = controller.scene().unwrap().root_brep_representation(&box_stamp).unwrap();
        run(&mut controller, box_id, 0.1);
        run(&mut controller, sphere_id, 0.04);
        let sphere_stamp = controller.document().stamp(sphere_id).unwrap();
        let sphere_buffers = controller.scene().unwrap().objects.iter().find(|o| o.root == sphere_id).unwrap().buffers.clone();
        run(&mut controller, sphere_id, 0.1);
        let cache = controller.remesh_representation_cache.as_ref().unwrap().clone();
        controller.request_topology_remesh_for(box_id, 0.04, 0.3, 1, 1, 8 * 1024 * 1024).unwrap();
        assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Stored { .. })));
        assert!(matches!(controller.poll_topology_remesh(), Some(Ok(()))));
        controller.request_topology_remesh_for(sphere_id, 0.04, 0.3, 1, 1, 8 * 1024 * 1024).unwrap();
        assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Stored { .. })));
        assert!(matches!(controller.poll_topology_remesh(), Some(Ok(()))));
        assert!(controller.scene().unwrap().objects.iter().find(|o| o.root == sphere_id).unwrap().buffers.ptr_eq(&sphere_buffers));
        assert_eq!(controller.scene().unwrap().root_brep_pick_display(&sphere_stamp).unwrap().deflection_mm, 0.04);
        // Retire the pinned box profile again: it is now held by the cache and
        // the independent reader, rather than by the current scene.
        run(&mut controller, box_id, 0.1);
        assert_eq!(controller.document(), &before);
        assert_eq!((controller.coordinator.undo_len(), controller.coordinator.redo_len()), (undo, redo));
        let reserved = |c: &Controller| c.coordinator.external_reserved_bytes(MemoryClass::Kernel)
            + c.coordinator.external_reserved_bytes(MemoryClass::CpuCache);
        let old = reserved(&controller);
        let released = cache.reclaim_optional_memory(u64::MAX).unwrap();
        assert!(released > 0);
        assert_eq!(old - reserved(&controller), released);
        assert_eq!(pinned.profile()[0], 0.04f64.to_bits());
        assert!(pinned.display_bytes().unwrap() > 0, "reader remains usable during reclamation");
        controller.request_topology_remesh_for(box_id, 0.04, 0.3, 1, 1, 8 * 1024 * 1024).unwrap();
        assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Stored { .. })),
            "a pinned cached representation must survive reclamation");
        assert!(matches!(controller.poll_topology_remesh(), Some(Ok(()))));
        drop(pinned);
        drop(sphere_buffers);
    }

    #[test]
    fn remesh_cache_evicts_oldest_unpinned_profile_after_hit() {
        let (mut controller, id) = controller_with_topology_proof();
        let run = |controller: &mut Controller, profile| {
            controller.request_topology_remesh_for(id, profile, 0.3,
                16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        };
        for profile in [0.01, 0.02, 0.03, 0.04, 0.05] { run(&mut controller, profile); }
        controller.request_topology_remesh_for(id, 0.01, 0.3, 1, 1, 8 * 1024 * 1024).unwrap();
        assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Stored { .. })));
        assert!(matches!(controller.poll_topology_remesh(), Some(Ok(()))));
        run(&mut controller, 0.06);
        controller.request_topology_remesh_for(id, 0.01, 0.3, 1, 1, 8 * 1024 * 1024).unwrap();
        assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Stored { .. })));
        controller.cancel_topology_remesh();
        controller.request_topology_remesh_for(id, 0.02, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Worker { .. })));
        controller.cancel_topology_remesh();
    }

    #[test]
    fn remesh_cache_does_not_evict_when_all_slots_are_pinned() {
        let (mut controller, id) = controller_with_topology_proof();
        let stamp = controller.document().stamp(id).unwrap();
        let mut pins = Vec::new();
        for profile in [0.01, 0.02, 0.03, 0.04, 0.05, 0.06] {
            controller.request_topology_remesh_for(id, profile, 0.3,
                16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            if profile <= 0.04 { pins.push(controller.scene().unwrap().root_brep_representation(&stamp).unwrap()); }
        }
        controller.request_topology_remesh_for(id, 0.01, 0.3, 1, 1, 8 * 1024 * 1024).unwrap();
        assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Stored { .. })));
        assert!(matches!(controller.poll_topology_remesh(), Some(Ok(()))));
        assert_eq!(pins.len(), 4);
    }

    #[test]
    fn stored_remesh_refusal_cancel_and_stale_keep_current_state() {
        for mode in ["budget", "cancel", "stale"] {
            let (mut controller, id) = controller_with_topology_proof();
            for profile in [0.04, 0.1] {
                controller.request_topology_remesh_for(id, profile, 0.3, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                loop { if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                    assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(5)); }
            }
            let before = controller.document().clone();
            let undo = controller.coordinator.undo_len(); let redo = controller.coordinator.redo_len();
            controller.request_topology_remesh_for(id, 0.04, 0.3, 1, 1, if mode == "budget" { 1 } else { 8 * 1024 * 1024 }).unwrap();
            assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Stored { .. })));
            if mode == "cancel" { controller.cancel_topology_remesh(); }
            if mode == "stale" { controller.replace_scene_same_epoch_for_test(0.).unwrap(); }
            let publication = controller.scene().unwrap().publication_id();
            if mode == "cancel" || mode == "stale" { assert!(controller.poll_topology_remesh().is_none()); }
            else { assert!(controller.poll_topology_remesh().unwrap().is_err()); }
            assert_eq!(controller.scene().unwrap().publication_id(), publication);
            assert_eq!(controller.document(), &before); assert_eq!(controller.coordinator.undo_len(), undo); assert_eq!(controller.coordinator.redo_len(), redo);
        }
    }

    #[test]
    fn stored_remesh_does_not_retain_replaced_document() {
        let (mut controller, id) = controller_with_topology_proof();
        for profile in [0.04, 0.1] {
            controller.request_topology_remesh_for(id, profile, 0.3, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop { if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(5)); }
        }
        let cache = controller.remesh_representation_cache.as_ref().unwrap().clone();
        let old = Arc::downgrade(&controller.scene().unwrap().document);
        controller.restore_master_document(Document::with_template_tolerance("replacement")).unwrap();
        assert!(old.upgrade().is_none(), "representation cache must not retain the old document");
        assert!(om9_core::transaction::OptionalMemoryCache::reclaim_optional_memory(&*cache, 1).unwrap() > 0);
    }

    #[test]
    fn remesh_cache_refuses_display_without_matching_owner_proof() {
        let (mut controller, id) = controller_with_topology_proof();
        let Geometry::Brep(shape) = &controller.document().entities[&id].geometry else { panic!("BRep") };
        let shape = shape.clone();
        let stamp = controller.document().stamp(id).unwrap();
        let (buffers, _) = Scene::prepare_brep_buffers(&shape, controller.coordinator.limits(), |_, _| Ok(())).unwrap();
        let forged = controller.scene().unwrap().try_replace_root_brep_display(&stamp, &shape, buffers,
            [0.04f64.to_bits(), 0.3f64.to_bits()], Arc::new(()), Arc::new(())).unwrap();
        controller.publish_owned_scene(forged, None);
        controller.request_topology_remesh_for(id, 0.04, 0.3, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Worker { .. })), "matching profile alone must not bypass proof");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop { if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(5)); }
        assert!(controller.remesh_representation_cache.is_none(), "a non-remesh display must never inherit the current proof");
    }

    #[test]
    fn remesh_cache_topology_copy_does_not_pin_or_alias_representation() {
        let (mut controller, id) = controller_with_topology_proof();
        let request = |controller: &mut Controller, profile| controller.request_topology_remesh_for(id, profile, 0.3,
            16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let wait = |controller: &mut Controller| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop { if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(5)); }
        };
        request(&mut controller, 0.04); wait(&mut controller);
        let stamp = controller.document().stamp(id).unwrap();
        let detached = controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().topology.clone();
        request(&mut controller, 0.1); wait(&mut controller);
        request(&mut controller, 0.04);
        assert!(matches!(controller.poll_topology_remesh(), Some(Ok(()))));
        assert_eq!(detached.faces.len(), 6);
        request(&mut controller, 0.1);
        assert!(matches!(controller.topology_remesh_job, Some(topology_remesh::PendingTopologyRemesh::Stored { .. })));
        assert!(matches!(controller.poll_topology_remesh(), Some(Ok(()))));
        assert_eq!(detached.faces.len(), 6, "the independent copy stays usable after the cache entry is replaced");
        let copied_display = controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap().clone();
        let old_document = Arc::downgrade(&controller.scene().unwrap().document);
        let peer = controller.coordinator.memory_admission_peer().unwrap();
        drop(controller);
        assert!(old_document.upgrade().is_none());
        assert_eq!(peer.external_reserved_bytes(MemoryClass::Kernel), 0, "copied topology must not pin the remesh reservation");
        assert_eq!(copied_display.topology.faces.len(), 6);
        drop(detached);
    }

    #[test]
    fn current_sphere_remesh_profile_is_reused_only_for_an_exact_match() {
        let mut controller = Controller::new(Document::with_template_tolerance("sphere cache hit"), Limits::default()).unwrap();
        controller.solid_sphere_points([1., 2., 3.], 5.).unwrap();
        let id = controller.topology_proof_for(None).unwrap().stamp().object;
        let wait = |controller: &mut Controller| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
                assert!(std::time::Instant::now() < deadline, "remesh timed out");
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        };
        controller.request_topology_remesh_for(id, 0.2, 0.5, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        wait(&mut controller);
        let publication = controller.scene().unwrap().publication_id();
        let kernel = controller.coordinator.external_reserved_bytes(MemoryClass::Kernel);
        controller.request_topology_remesh_for(id, 0.2, 0.5, 1, 1, 1).unwrap();
        assert!(controller.topology_remesh_pending(), "a cache hit must still be reported through poll");
        assert_eq!(controller.scene().unwrap().publication_id(), publication);
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), kernel);
        assert!(matches!(controller.request_topology_remesh_for(id, 0.2, 0.5, 1, 1, 1), Err(Error::Busy)));
        assert!(matches!(controller.poll_topology_remesh(), Some(Ok(()))));
        assert!(!controller.topology_remesh_pending());
        assert!(matches!(controller.request_topology_remesh_for(id, f64::NAN, 0.5, 1, 1, 1), Err(Error::Invalid(_))));
        controller.request_topology_remesh_for(id, 0.2, 0.4, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        assert!(controller.topology_remesh_pending(), "a different angle must rebuild");
        wait(&mut controller);
        assert_ne!(controller.scene().unwrap().publication_id(), publication);
        controller.undo().unwrap();
        assert!(controller.request_topology_remesh_for(id, 0.2, 0.4, 1, 1, 1).is_err());
    }

    #[test]
    fn current_remesh_profile_does_not_bypass_stale_document_validation() {
        let mut controller = Controller::new(Document::with_template_tolerance("sphere stale cache"), Limits::default()).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let id = controller.topology_proof_for(None).unwrap().stamp().object;
        controller.request_topology_remesh_for(id, 0.2, 0.5, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        controller.select_objects(&[id]).unwrap();
        controller.numeric_move([3., 0., 0.]).unwrap();
        controller.request_topology_remesh_for(id, 0.2, 0.5, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        assert!(controller.topology_remesh_pending(), "a changed document must revalidate instead of taking the fast hit");
        controller.undo().unwrap();
        assert!(!controller.topology_remesh_pending(), "Undo cancels the stale worker");
        assert_eq!(controller.document().entities[&id].placement, om9_core::geometry::Transform::identity());
    }

    #[test]
    fn pending_remesh_cache_hit_is_canceled_by_undo() {
        let mut controller = Controller::new(Document::with_template_tolerance("cached undo"), Limits::default()).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let id = controller.topology_proof_for(None).unwrap().stamp().object;
        controller.request_topology_remesh_for(id, 0.2, 0.5, 16 * 1024 * 1024, 8 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_topology_remesh() { result.unwrap(); break; }
            assert!(std::time::Instant::now() < deadline, "remesh timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let kernel = controller.coordinator.external_reserved_bytes(MemoryClass::Kernel);
        controller.request_topology_remesh_for(id, 0.2, 0.5, 1, 1, 1).unwrap();
        assert!(controller.topology_remesh_pending());
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Kernel), kernel);
        controller.undo().unwrap();
        assert!(!controller.topology_remesh_pending());
        assert!(controller.poll_topology_remesh().is_none());
        assert!(!controller.document().entities.contains_key(&id));
    }
    #[test]
    #[ignore = "isolated release CPU timing probe; not a desktop benchmark"]
    fn controller_primitive_topology_release_cost() {
        use om9_kernel::creation::{self, Primitive};
        let limits = Limits::default();
        for (name, primitive) in [("box", Primitive::Box { x_mm: 10., y_mm: 10., z_mm: 10. }),
            ("sphere", Primitive::Sphere { radius_mm: 5. })] {
            let mut plain = Vec::new(); let mut attested = Vec::new();
            for _ in 0..10 {
                let mut baseline = Controller::new(Document::with_template_tolerance("baseline"), limits).unwrap();
                let start = std::time::Instant::now();
                let brep = creation::primitive(primitive, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
                baseline.commit_entity("probe", "probe", Entity::new("probe", Geometry::Brep(brep), None)).unwrap();
                plain.push(start.elapsed().as_micros());
                let mut candidate = Controller::new(Document::with_template_tolerance("candidate"), limits).unwrap();
                let start = std::time::Instant::now();
                candidate.commit_primitive_optional_topology("probe", "probe", name, primitive,
                    om9_core::geometry::Transform::identity()).unwrap();
                attested.push(start.elapsed().as_micros());
                assert!(candidate.topology_proof_for(None).is_some());
            }
            plain.sort_unstable(); attested.sort_unstable();
            println!("{name} samples=10 plain_median_us={} plain_max_us={} proof_median_us={} proof_max_us={}",
                (plain[4] + plain[5]) / 2, plain[9], (attested[4] + attested[5]) / 2, attested[9]);
        }
    }
}

// Fields drop in declaration order: scene buffers must be released before
// their shared-account reservation, including on validation/commit errors.
struct ReservedCandidateScene {
    scene: Scene,
    buffer_owner: Option<SceneBufferOwner>,
    reservation: om9_core::transaction::ExternalMemoryReservation,
}

/// Worker buffers retain their charge until after the data is dropped. No
/// canonical clone crosses the exclusive source-reservation handoff.
struct PreparedEditWire {
    object: om9_core::identity::ObjectId,
    buffers: om9_scene::scene::PreparedBrepBuffers,
    reservation: om9_core::transaction::ExternalMemoryReservation,
}

struct HistoryWorker {
    token: uuid::Uuid,
    receiver: std::sync::mpsc::Receiver<Result<(om9_core::transaction::PreparedHistory, ReservedCandidateScene)>>,
    cancel: Arc<std::sync::atomic::AtomicBool>,
    before: Arc<Document>,
    scene_publication: uuid::Uuid,
    selection_revision: u64,
    sub_revision: u64,
    redo: bool,
    peer_metadata_reservation: Option<om9_core::transaction::ExternalMemoryReservation>,
    _current_scene_reservation: om9_core::transaction::ExternalMemoryReservation,
}
impl Drop for HistoryWorker {
    fn drop(&mut self) { self.cancel.store(true, std::sync::atomic::Ordering::Release); }
}

#[cfg(test)]
mod history_worker_tests {
    use super::*;
    fn fixture() -> Controller {
        let mut controller = Controller::new(Document::with_template_tolerance("history worker"), Limits::default()).unwrap();
        controller.submit("Point").unwrap();
        controller.submit("1,2,3").unwrap();
        controller
    }
    fn wait(controller: &mut Controller) -> Result<()> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(result) = controller.poll_history_worker() { return result; }
            assert!(std::time::Instant::now() < deadline, "history worker timed out");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
    #[test]
    fn history_worker_publishes_undo_and_redo_after_private_preparation() {
        let mut controller = fixture();
        let before = controller.coordinator.snapshot();
        let publication = controller.scene().unwrap().publication_id();
        controller.request_history_worker(false).unwrap();
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.scene().unwrap().publication_id(), publication);
        assert!(matches!(controller.coordinator.prepare(TransactionRequest::new("blocked", vec![])), Err(Error::Busy)));
        assert!(matches!(controller.request_history_worker(false), Err(Error::Busy)));
        wait(&mut controller).unwrap();
        assert!(controller.document().entities.is_empty());
        assert_eq!(controller.coordinator.redo_len(), 1);
        controller.request_history_worker(true).unwrap();
        wait(&mut controller).unwrap();
        assert_eq!(controller.document().entities.len(), 1);
        assert_eq!(controller.coordinator.undo_len(), 1);
    }
    #[test]
    fn history_worker_cancel_and_scene_replacement_keep_world_and_stacks() {
        for replace_scene in [false, true] {
            let mut controller = fixture();
            let before = controller.coordinator.snapshot();
            controller.request_history_worker(false).unwrap();
            if replace_scene { controller.replace_scene_same_epoch_for_test(0.).unwrap(); }
            else { controller.cancel(); }
            let result = wait(&mut controller);
            assert!(matches!(result, Err(Error::Canceled) | Err(Error::Stale("history scene context"))));
            assert_eq!(controller.coordinator.snapshot(), before);
            assert!(!controller.history_worker_pending());
            controller.undo().unwrap();
            assert!(controller.document().entities.is_empty());
        }
    }
    #[test]
    fn history_worker_memory_refusal_preserves_world_and_releases_barrier() {
        const RAM: u64 = 8 * 1024 * 1024 * 1024;
        fn sample() -> Option<(u64, u64)> {
            Some((RAM, if std::thread::current().name() == Some("om9-history-scene") { 0 } else { RAM }))
        }
        let mut controller = fixture();
        controller.coordinator.set_physical_memory_sampler(sample);
        let before = controller.coordinator.snapshot();
        let baseline = controller.coordinator.external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache);
        controller.request_history_worker(false).unwrap();
        assert!(matches!(wait(&mut controller), Err(Error::Budget(_))));
        assert_eq!(controller.coordinator.snapshot(), before);
        assert!(controller.coordinator.external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache) <= baseline,
            "worker guards must be released; optional cache reclamation may reduce the baseline");
        controller.undo().unwrap();
        assert!(controller.document().entities.is_empty());
    }
    #[test]
    fn history_worker_refuses_changed_selection_without_overwriting_it() {
        let mut controller = fixture();
        let id = *controller.document().entities.keys().next().unwrap();
        controller.select_objects(&[id]).unwrap();
        let before = controller.coordinator.snapshot();
        controller.request_history_worker(false).unwrap();
        controller.clear_selection();
        assert!(matches!(wait(&mut controller), Err(Error::Stale("history scene context"))));
        assert_eq!(controller.coordinator.snapshot(), before);
        assert!(controller.selected().is_empty());
        controller.undo().unwrap();
        assert!(controller.document().entities.is_empty());
    }
}

struct SceneBufferOwner {
    epoch: u64,
    document: std::sync::Weak<Document>,
    bytes: u64,
    lease: Arc<SceneBufferLease>,
}

#[cfg(feature = "occt-brep")]
struct DupEdgeSceneBackup {
    scene: Scene,
    owner: Option<SceneBufferOwner>,
    preview_publication: uuid::Uuid,
    reservation: om9_core::transaction::ExternalMemoryReservation,
}


pub(crate) struct SceneBufferLease {
    previous: Option<Arc<SceneBufferLease>>,
    _reservation: om9_core::transaction::ExternalMemoryReservation,
}
impl SceneBufferLease {
    fn new(previous: Option<Arc<Self>>, reservation: om9_core::transaction::ExternalMemoryReservation) -> Result<Arc<Self>> {
        let mut lease = Arc::new(Self { previous, _reservation: reservation });
        // The render buffers and this Arc now exist. Keep their managed charge,
        // but stop treating them as an allocation still to be made.
        let reservation = &mut Arc::get_mut(&mut lease).expect("new scene buffer lease")._reservation;
        reservation.mark_allocated(reservation.pending_bytes())?;
        Ok(lease)
    }
}

fn scene_buffer_lease_overhead() -> u64 {
    std::mem::size_of::<SceneBufferLease>() as u64 + 4 * std::mem::size_of::<usize>() as u64
}

impl Drop for SceneBufferLease {
    fn drop(&mut self) {
        // A long edit session can build a long chain. Release uniquely owned
        // predecessors iteratively rather than overflowing the call stack.
        let mut next = self.previous.take();
        while let Some(previous) = next {
            match Arc::try_unwrap(previous) {
                Ok(mut previous) => { next = previous.previous.take(); drop(previous); }
                Err(_) => break,
            }
        }
    }
}
#[cfg(test)]
mod incremental_scene_memory_tests {
    use super::*;
    use om9_core::{budget::{Usage, memory::MemoryClass}, geometry::Mesh, identity::ObjectId,
        transaction::PreparedTransaction};

    #[cfg(feature = "occt-brep")]
    #[test]
    fn detached_ordinary_sphere_object_pins_display_without_document() {
        let limits = Limits::default();
        let mut controller = Controller::new(Document::with_template_tolerance("detached sphere"), limits).unwrap();
        controller.solid_sphere_points([0., 0., 0.], 5.).unwrap();
        let entity = controller.document().entities.values().next().unwrap();
        let Geometry::Brep(brep) = &entity.geometry else { panic!("expected BRep") };
        let display = brep.display_liveness();
        let document = Arc::downgrade(&controller.scene().unwrap().document);
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&controller.coordinator).unwrap();
        let object = controller.scene().unwrap().objects[0].clone();
        drop(controller);
        assert!(document.upgrade().is_none());
        assert!(display.is_alive(), "a detached occurrence retains its independently tracked display");
        assert!(matches!(object.buffers.as_ref(), RenderBuffers::Mesh { triangles, .. } if !triangles.is_empty()));
        assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) > 0);
        drop(object);
        assert!(!display.is_alive());
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_snapshot_admission_refusal_preserves_document_and_redo() {
        const MIB: u64 = 1024 * 1024;
        for numeric in [false, true] {
            let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
            let mut controller = Controller::new(Document::with_template_tolerance("sphere pressure"), limits).unwrap();
            for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
            controller.undo().unwrap();
            let before = controller.coordinator.snapshot();
            let scene = controller.scene.as_ref().unwrap().publication_id();
            let held = controller.scene_cache_usage().unwrap();
            let mut pressure = controller.coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Export, None, held).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            let create = |controller: &mut Controller| {
                if numeric { controller.submit("Sphere 5").map(|_| ()) }
                else { controller.solid_sphere_points([0., 0., 0.], 5.) }
            };
            assert!(matches!(create(&mut controller), Err(Error::Budget(_))), "numeric={numeric}");
            assert_eq!(controller.coordinator.snapshot(), before);
            assert_eq!(controller.coordinator.redo_len(), 1);
            assert_eq!(controller.scene.as_ref().unwrap().publication_id(), scene);
            drop(pressure);
            create(&mut controller).unwrap();
            assert_eq!(controller.coordinator.redo_len(), 0);
            assert_eq!(controller.document().entities.len(), 2);
            controller.undo().unwrap();
            assert_eq!(controller.document().entities.len(), 1);
            controller.redo().unwrap();
            assert_eq!(controller.document().entities.len(), 2);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_point_input_retries_after_snapshot_admission_refusal() {
        const MIB: u64 = 1024 * 1024;
        let cases: &[(&str, &[[f64; 3]], Option<[f64; 3]>, Option<&str>)] = &[
            ("Sphere", &[[0., 0., 0.]], Some([10., 0., 0.]), None),
            ("Sphere", &[[0., 0., 0.]], None, Some("5")),
            ("Sphere 2Point", &[[0., 0., 0.]], Some([10., 0., 0.]), None),
            ("Sphere 2Point", &[[0., 0., 0.]], None, Some("10")),
            ("Sphere 3Point", &[[0., 0., 0.], [10., 0., 0.]], Some([0., 10., 0.]), None),
            ("Sphere 4Point", &[[0., 0., 0.], [10., 0., 0.], [0., 10., 0.]], Some([0., 0., 10.]), None),
            ("Sphere FitPoints", &[[5., 0., 0.], [-5., 0., 0.], [0., 5., 0.], [0., 0., 5.]], None, Some("")),
        ];
        for (command, points, final_point, final_text) in cases {
            let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
            let mut controller = Controller::new(Document::with_template_tolerance("sphere retry"), limits).unwrap();
            for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
            controller.undo().unwrap();
            controller.submit(command).unwrap();
            for point in *points { controller.accept_world_point(*point).unwrap(); }
            let prompt = controller.prompt();
            let before = controller.coordinator.snapshot();
            let held = controller.scene_cache_usage().unwrap();
            let mut pressure = controller.coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Export, None, held).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            let finish = |controller: &mut Controller| match final_point {
                Some(point) => controller.accept_world_point(*point),
                None => controller.submit(final_text.expect("text completion")).map(|_| ()),
            };
            assert!(matches!(finish(&mut controller), Err(Error::Budget(_))), "{command}");
            assert_eq!(controller.coordinator.snapshot(), before);
            assert_eq!(controller.coordinator.redo_len(), 1);
            assert_eq!(controller.prompt(), prompt, "{command} must keep its pending input");
            drop(pressure);
            finish(&mut controller).unwrap();
            assert_eq!(controller.document().entities.len(), 2);
            assert_eq!(controller.coordinator.redo_len(), 0);
        }
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn sphere_curve_input_retries_after_snapshot_admission_refusal() {
        const MIB: u64 = 1024 * 1024;
        for (mode, numeric) in [("AroundCurve", false), ("AroundCurve", true), ("Tangent", false), ("Tangent", true)] {
            let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
            let mut document = Document::with_template_tolerance("sphere curve retry");
            let curve = om9_core::curve_build::polyline(&[[0., 0., 0.], [20., 0., 0.]]).unwrap();
            let source = Entity::new("path", Geometry::Curve(curve), None);
            let source_id = source.id;
            document.entities.insert(source_id, source);
            let mut controller = Controller::new(document, limits).unwrap();
            controller.submit("Point").unwrap(); controller.submit("4,5,6").unwrap();
            controller.undo().unwrap();
            controller.select_objects(&[source_id]).unwrap();
            controller.submit("Sphere").unwrap(); controller.submit(mode).unwrap();
            controller.accept_world_point([10., 0., 0.]).unwrap();
            let prompt = controller.prompt();
            let before = controller.coordinator.snapshot();
            let held = controller.scene_cache_usage().unwrap();
            let mut pressure = controller.coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Export, None, held).unwrap();
            pressure.shrink_to(pressure.bytes() - 2048).unwrap();
            let finish = |controller: &mut Controller| {
                if numeric { controller.submit("5").map(|_| ()) }
                else { controller.accept_world_point([10., 5., 0.]) }
            };
            assert!(matches!(finish(&mut controller), Err(Error::Budget(_))), "{mode} numeric={numeric}");
            assert_eq!(controller.coordinator.snapshot(), before);
            assert_eq!(controller.coordinator.redo_len(), 1);
            assert_eq!(controller.prompt(), prompt, "{mode} numeric={numeric}");
            assert!(controller.document().entities.contains_key(&source_id));
            drop(pressure);
            finish(&mut controller).unwrap();
            assert_eq!(controller.document().entities.len(), 2);
            assert_eq!(controller.coordinator.redo_len(), 0);
            let sphere = controller.document().entities.values().find(|entity| entity.id != source_id).unwrap();
            let Geometry::Brep(brep) = &sphere.geometry else { panic!("expected sphere B-rep") };
            let measured = om9_kernel::creation::measure(brep, &limits).unwrap();
            assert!((measured.volume_mm3 - 4. * std::f64::consts::PI / 3. * 125.).abs() < 1e-6);
            controller.undo().unwrap();
            assert_eq!(controller.document().entities.len(), 1);
            assert!(controller.document().entities.contains_key(&source_id));
            controller.redo().unwrap();
            assert_eq!(controller.document().entities.len(), 2);
        }
    }

    #[test]
    fn detached_ordinary_scene_buffer_keeps_registered_owner() {
        let limits = Limits::default();
        let mut document = Document::new("detached buffer");
        let entity = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
        document.entities.insert(entity.id, entity);
        let controller = Controller::new(document, limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&controller.coordinator).unwrap();
        let buffer = controller.scene().unwrap().objects[0].buffers.clone();
        let held = controller.scene().unwrap().shared_buffer_bytes();
        assert!(held > 0);
        drop(controller);
        assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) >= held,
            "ordinary buffer must retain its reservation after controller drop");
        drop(buffer);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn detached_incremental_buffer_keeps_charge_across_undo() {
        let limits = Limits::default();
        let mut controller = Controller::new(Document::with_template_tolerance("incremental buffer"), limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&controller.coordinator).unwrap();
        controller.submit("Point").unwrap(); controller.submit("1,2,3").unwrap();
        let buffer = controller.scene().unwrap().objects[0].buffers.clone();
        controller.undo().unwrap();
        assert!(controller.scene().unwrap().objects.is_empty());
        let held = peer.external_reserved_bytes(MemoryClass::CpuCache);
        assert!(held > 0);
        drop(buffer);
        assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) < held);
    }

    fn prepare(controller: &Controller) -> (PreparedTransaction, Usage, Usage, ObjectId) {
        let entity = Entity::new("mesh", Geometry::Mesh(Mesh {
            vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]; 10_000], normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        let id = entity.id;
        let increment = controller.scene.as_ref().unwrap().update_held_usage(Some(&entity)).unwrap();
        let mut held = controller.scene_cache_usage().unwrap();
        held.add(increment, controller.coordinator.limits()).unwrap();
        let request = TransactionRequest::new("mesh", vec![Mutation::Insert { document: controller.document, entity }]);
        let prepared = controller.coordinator.prepare_with_held_usage(request, held).unwrap();
        (prepared, held, increment, id)
    }

    #[test]
    fn incremental_scene_reserves_before_build_and_cancellation_preserves_redo() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        let mut controller = Controller::new(Document::with_template_tolerance("incremental"), limits).unwrap();
        for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
        controller.undo().unwrap();
        let before = controller.coordinator.snapshot();
        let scene_epoch = controller.scene.as_ref().unwrap().epoch;
        let mut baseline = controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache);
        let initial_buffers = controller.scene.as_ref().unwrap().shared_buffer_bytes();
        assert_eq!(controller.coordinator.redo_len(), 1);
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&controller.coordinator).unwrap();
        let (prepared, held, increment, id) = prepare(&controller);
        let refused = controller.commit_incremental_scene_with_guard(prepared, held, increment, Some(id), false,
            || Some((1024 * MIB, 256 * MIB)), || panic!("low RAM must refuse before scene construction"));
        assert!(matches!(refused, Err(Error::Budget("available physical RAM"))));
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.coordinator.redo_len(), 1);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline - controller.scene.as_ref().unwrap().picking_arena_bytes().unwrap(),
            "physical refusal releases unused arena quota, preserving source/history owners");
        baseline = peer.external_reserved_bytes(MemoryClass::CpuCache);
        let initial_memory = controller.scene.as_ref().unwrap().memory_allowance_bytes();
        let (prepared, held, increment, id) = prepare(&controller);
        let canceled = controller.commit_incremental_scene_with_guard(prepared, held, increment, Some(id), false,
            || Some((1024 * MIB, 1024 * MIB)), || {
                assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) >= increment.bytes);
                let rest = peer.reserve_remaining_external_memory_with_held(MemoryClass::Export, None, Usage::default())?;
                assert!(matches!(peer.reserve_external_memory(MemoryClass::Export, 1, None), Err(Error::Budget(_))));
                drop(rest);
                Err(Error::Canceled)
            });
        assert!(matches!(canceled, Err(Error::Canceled)));
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.coordinator.redo_len(), 1);
        assert_eq!(controller.scene.as_ref().unwrap().epoch, scene_epoch);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline);
        let (prepared, held, increment, id) = prepare(&controller);
        let mut competing = None;
        let refused = controller.commit_incremental_scene_with_guard(prepared, held, increment, Some(id), false,
            || Some((1024 * MIB, 1024 * MIB)), || {
                competing = Some(peer.reserve_remaining_external_memory_with_held(MemoryClass::Export, None, Usage::default())?);
                Ok(())
            });
        assert!(matches!(refused, Err(Error::Budget(_))));
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.coordinator.redo_len(), 1);
        assert_eq!(controller.scene.as_ref().unwrap().epoch, scene_epoch);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline);
        drop(competing);
        let (prepared, held, increment, id) = prepare(&controller);
        let receipt = controller.commit_incremental_scene_with_guard(prepared, held, increment, Some(id), false,
            || Some((1024 * MIB, 1024 * MIB)), || {
                assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) >= increment.bytes);
                Ok(())
            }).unwrap();
        assert!(receipt.published);
        assert!(controller.document().entities.contains_key(&id));
        assert!(controller.scene.as_ref().unwrap().objects.iter().any(|object| object.root == id));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline
            + controller.scene.as_ref().unwrap().shared_buffer_bytes() - initial_buffers + scene_buffer_lease_overhead()
            - initial_memory + controller.scene.as_ref().unwrap().memory_allowance_bytes());
    }

    #[test]
    fn system_commit_refusal_preserves_scene_and_redo_then_allows_retry() {
        use om9_core::budget::memory::CommitMemory;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        let mut controller = Controller::new(Document::with_template_tolerance("commit refusal"), limits).unwrap();
        for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
        controller.undo().unwrap();
        let before = controller.coordinator.snapshot();
        let epoch = controller.scene.as_ref().unwrap().epoch;
        let baseline = controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache);
        let pending_before = controller.coordinator.external_pending_bytes(MemoryClass::CpuCache);
        let (prepared, held, increment, id) = prepare(&controller);
        controller.coordinator.set_commit_memory_sampler(|| Some(CommitMemory {
            limit_bytes: 1024 * MIB, used_bytes: 768 * MIB,
        }));
        let refused = controller.commit_incremental_scene_with_guard(prepared, held, increment, Some(id), false,
            || Some((1024 * MIB, 1024 * MIB)), || panic!("commit pressure must refuse before scene build"));
        assert!(matches!(refused, Err(Error::Budget("available system commit"))));
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.coordinator.redo_len(), 1);
        assert_eq!(controller.scene.as_ref().unwrap().epoch, epoch);
        let optional = controller.scene.as_ref().unwrap().picking_arena_bytes().unwrap();
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache), baseline - optional);
        assert_eq!(controller.coordinator.external_pending_bytes(MemoryClass::CpuCache), pending_before - optional);
        controller.coordinator.set_commit_memory_sampler(|| Some(CommitMemory {
            limit_bytes: 1024 * MIB, used_bytes: 0,
        }));
        let (prepared, held, increment, id) = prepare(&controller);
        let receipt = controller.commit_incremental_scene_with_guard(prepared, held, increment, Some(id), false,
            || Some((1024 * MIB, 1024 * MIB)), || Ok(())).unwrap();
        assert!(receipt.published);
        assert!(controller.document().entities.contains_key(&id));
        assert_eq!(controller.coordinator.redo_len(), 0);
        assert_eq!(controller.coordinator.external_pending_bytes(MemoryClass::CpuCache), controller.scene.as_ref().unwrap().picking_arena_bytes().unwrap());
    }

    #[test]
    fn layer_and_property_edits_refuse_pressure_without_losing_scene_or_redo() {
        use super::properties::PropertyEdit;
        const MIB: u64 = 1024 * 1024;
        for property in [false, true] {
            let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
            let mut controller = Controller::new(Document::with_template_tolerance("metadata pressure"), limits).unwrap();
            for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
            controller.undo().unwrap();
            let id = *controller.document().entities.keys().next().unwrap();
            let before = controller.coordinator.snapshot();
            let epoch = controller.scene.as_ref().unwrap().epoch;
            let baseline = controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache);
            let held = controller.scene_cache_usage().unwrap();
            let pressure = controller.coordinator.reserve_remaining_external_memory_with_held(MemoryClass::Export, None, held).unwrap();
            let stamp = controller.document().stamp(id).unwrap();
            let result = if property {
                controller.edit_object_property(vec![stamp], PropertyEdit::Label("changed".into()))
            } else {
                controller.set_objects_hidden(vec![stamp], true)
            };
            assert!(matches!(result, Err(Error::Budget(_))), "unexpected refusal: {result:?}");
            assert_eq!(controller.coordinator.snapshot(), before);
            assert_eq!(controller.coordinator.redo_len(), 1);
            assert_eq!(controller.scene.as_ref().unwrap().epoch, epoch);
            drop(pressure);
            let stamp = controller.document().stamp(id).unwrap();
            if property {
                controller.edit_object_property(vec![stamp], PropertyEdit::Label("changed".into())).unwrap();
                assert_eq!(controller.document().entities[&id].label, "changed");
            } else {
                controller.set_objects_hidden(vec![stamp], true).unwrap();
                assert!(!controller.scene.as_ref().unwrap().objects.iter().find(|object| object.root == id).unwrap().visible);
            }
            assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::CpuCache), baseline);
            controller.undo().unwrap();
        }
    }

    #[test]
    fn clipboard_paste_under_shared_pressure_preserves_redo_and_can_retry() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        let mut controller = Controller::new(Document::with_template_tolerance("paste pressure"), limits).unwrap();
        for point in ["1,2,3", "4,5,6"] {
            controller.submit("Point").unwrap();
            controller.submit(point).unwrap();
        }
        controller.undo().unwrap();
        let id = *controller.document().entities.keys().next().unwrap();
        controller.select_objects(&[id]).unwrap();
        let clipboard = controller.copy_selected_om9().unwrap();
        let before = controller.coordinator.snapshot();
        let scene_epoch = controller.scene.as_ref().unwrap().epoch;
        let selected = controller.selected().clone();
        let held = controller.scene_cache_usage().unwrap();
        let pressure = controller.coordinator.reserve_remaining_external_memory_with_held(
            MemoryClass::Export, None, held,
        ).unwrap();
        assert!(matches!(controller.paste_om9(&clipboard), Err(Error::Budget(_))));
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.coordinator.redo_len(), 1);
        assert_eq!(controller.scene.as_ref().unwrap().epoch, scene_epoch);
        assert_eq!(controller.selected(), &selected);
        drop(pressure);
        let pasted = controller.paste_om9(&clipboard).unwrap();
        assert_eq!(pasted.len(), 1);
        assert!(controller.scene.as_ref().unwrap().objects.iter().any(|object| object.root == pasted[0]));
        controller.undo().unwrap();
        assert_eq!(controller.coordinator.world(), &before.world);
        controller.redo().unwrap();
        assert!(controller.document().entities.contains_key(&pasted[0]));
    }

    #[test]
    fn long_scene_buffer_lease_chain_releases_without_recursive_drop() {
        let limits = Limits::default();
        let coordinator = TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap();
        let mut chain = None;
        let mut retained_prefix = None;
        for index in 0..10_000 {
            let reservation = coordinator.reserve_external_memory(MemoryClass::CpuCache, 1, None).unwrap();
            let next = Arc::new(SceneBufferLease { previous: chain, _reservation: reservation });
            if index == 4_999 { retained_prefix = Some(next.clone()); }
            chain = Some(next);
        }
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 10_000);
        drop(chain);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 5_000);
        drop(retained_prefix);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }

    #[test]
    fn incremental_scene_grows_shared_reservation_for_actual_capacity_before_commit() {
        use std::cell::Cell;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        let mut controller = Controller::new(Document::with_template_tolerance("scene actual capacity"), limits).unwrap();
        for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
        controller.undo().unwrap();
        let before = controller.coordinator.snapshot();
        let epoch = controller.scene.as_ref().unwrap().epoch;
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&controller.coordinator).unwrap();
        let make_small = |held: &mut Usage, increment: &mut Usage| {
            held.bytes = held.bytes.checked_sub(increment.bytes).unwrap() + 1;
            increment.bytes = 1;
        };
        let mut baseline = peer.external_reserved_bytes(MemoryClass::CpuCache);
        let initial_buffers = controller.scene.as_ref().unwrap().shared_buffer_bytes();
        let (prepared, mut held, mut increment, id) = prepare(&controller);
        make_small(&mut held, &mut increment);
        let samples = Cell::new(0);
        let refused = controller.commit_incremental_scene_with_guard(prepared, held, increment, Some(id), false,
            || { let n = samples.get(); samples.set(n + 1); Some((1024 * MIB, if n == 0 { 1024 * MIB } else { 256 * MIB })) },
            || { assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline + 1); Ok(()) });
        assert!(matches!(refused, Err(Error::Budget("available physical RAM"))));
        assert_eq!(samples.get(), 2, "actual capacity must require a fresh admission");
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.coordinator.redo_len(), 1);
        assert_eq!(controller.scene.as_ref().unwrap().epoch, epoch);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline - controller.scene.as_ref().unwrap().picking_arena_bytes().unwrap());
        baseline = peer.external_reserved_bytes(MemoryClass::CpuCache);
        let initial_memory = controller.scene.as_ref().unwrap().memory_allowance_bytes();

        let (prepared, mut held, mut increment, id) = prepare(&controller);
        make_small(&mut held, &mut increment);
        let samples = Cell::new(0);
        let receipt = controller.commit_incremental_scene_with_guard(prepared, held, increment, Some(id), false,
            || { samples.set(samples.get() + 1); Some((1024 * MIB, 1024 * MIB)) },
            || { assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline + 1); Ok(()) }).unwrap();
        assert!(receipt.published);
        assert_eq!(samples.get(), 2);
        assert!(controller.scene.as_ref().unwrap().objects.iter().any(|object| object.root == id));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline
            + controller.scene.as_ref().unwrap().shared_buffer_bytes() - initial_buffers + scene_buffer_lease_overhead()
            - initial_memory + controller.scene.as_ref().unwrap().memory_allowance_bytes());
    }

    #[test]
    fn full_candidate_scene_reconciles_actual_capacity_before_commit() {
        use std::cell::Cell;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        let mut controller = Controller::new(Document::with_template_tolerance("full scene capacity"), limits).unwrap();
        for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
        controller.undo().unwrap();
        let before = controller.coordinator.snapshot();
        let epoch = controller.scene.as_ref().unwrap().epoch;
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&controller.coordinator).unwrap();

        let (prepared, mut held, increment, _) = prepare(&controller);
        let baseline = peer.external_reserved_bytes(MemoryClass::CpuCache);
        held.bytes -= increment.bytes;
        let samples = Cell::new(0);
        let refused = controller.commit_with_candidate_scene_guard(prepared, held, None,
            || { let n = samples.get(); samples.set(n + 1); Some((1024 * MIB, if n < 2 { 1024 * MIB } else { 256 * MIB })) },
            |reservation| { reservation.shrink_to(1)?; assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline + 1); Ok(()) });
        assert!(matches!(refused, Err(Error::Budget("available physical RAM"))));
        assert_eq!(samples.get(), 3);
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.coordinator.redo_len(), 1);
        assert_eq!(controller.scene.as_ref().unwrap().epoch, epoch);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline - controller.scene.as_ref().unwrap().picking_arena_bytes().unwrap());

        let (prepared, mut held, increment, id) = prepare(&controller);
        held.bytes -= increment.bytes;
        let samples = Cell::new(0);
        let receipt = controller.commit_with_candidate_scene_guard(prepared, held, None,
            || { samples.set(samples.get() + 1); Some((1024 * MIB, 1024 * MIB)) },
            |reservation| reservation.shrink_to(1)).unwrap();
        assert!(receipt.published);
        assert_eq!(samples.get(), 3);
        assert!(controller.scene.as_ref().unwrap().objects.iter().any(|object| object.root == id));
        let scene = controller.scene.as_ref().unwrap();
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), scene.shared_buffer_bytes() + scene_buffer_lease_overhead() + scene.memory_allowance_bytes());
        assert_eq!(controller.registered_scene_buffer_bytes(scene), scene.shared_buffer_bytes());
    }
}
mod angle_isocurve;
mod select_kind;
pub mod annotation;
mod apply_crv;
mod arc;
mod arc_blend;
mod blend;
pub mod builder;
pub mod ring_rail;
mod ring_rail_library;
#[cfg(feature = "occt-brep")]
mod ring_preview_worker;
mod circle;
mod command_options;
mod command_line;
mod dispatch_geometry;
mod point_input;
mod text_input;
mod text_input_surface;
#[cfg(test)]
mod command_options_tests;
#[cfg(all(test, feature = "occt-brep"))]
mod solid_input_regressions;
mod continue_curve;
mod continue_interp;
pub mod contract;
mod control_curve;
mod corner_ops;
mod cross_section;
mod crv2view;
mod crv_ends;
mod crv_seam;
mod curve_boolean;
mod curve_soft_edit;
mod divide;
mod dup_border;
mod ellipse;
mod extend_curve;
mod extrude_surface;
mod fillet_corners;
mod fit;
#[cfg(feature = "occt-brep")]
pub mod face_verification;
pub mod gem;
pub mod gem_cluster;
pub mod channel_cutters;
pub mod gem_cutters;
pub mod gem_edits;
pub mod gem_pave;
pub mod gem_placement;
pub mod gumball_numeric;
pub mod head_commands;
mod heightfield;
mod helix;
pub mod history;
mod interp;
mod interp_on_surface;
mod intersect;
mod intersect_two_sets;
mod isocurve;
pub mod measure;
mod mesh_curve;
pub mod metal;
pub mod metal_catalog;
#[cfg(feature = "occt-brep")]
mod metal_worker;
mod offset;
mod offset_normal;
mod offset_on_surface;
mod polygon;
mod polyline;
mod polyline_on_surface;
mod project;
mod pull;
mod rebuild;
mod rect;
mod registry;
mod refit_srf;
mod revolve;
pub mod ring_tools;
mod section;
pub mod settings_commands;
mod silhouette;
mod sketch;
mod sketch_on_surface;
mod spiral;
pub mod stress;
mod surface_snap;
pub use surface_snap::{SurfaceSnapJob, SurfaceSnapReceipt, SurfaceSnapScheduler};
pub(crate) use surface_snap::TypedSurfaceJob;
#[cfg(feature = "occt-brep")]
mod edge_snap;
#[cfg(feature = "occt-brep")]
pub(crate) use edge_snap::{EdgeBoundsCache, EdgeSnapSourceCache, EdgeSnapViewInput, EdgeSnapViewJob, EdgeSnapViewScheduler};
#[cfg(feature = "occt-brep")]
mod intersection_snap;
#[cfg(feature = "occt-brep")]
pub(crate) use intersection_snap::{IntersectionJob, IntersectionOccurrence, IntersectionScheduler};
#[cfg(feature = "occt-brep")]
mod topology_remesh;
#[cfg(feature = "occt-brep")]
mod topology_cache;
#[cfg(feature = "occt-brep")]
pub use topology_remesh::{TopologyRemeshJob, TopologyRemeshReceipt, DupEdgeSourceViewJob, DupEdgeSourceViewReceipt, PreparedDupEdgeSourceScene};
mod subobjects;
mod through_points;
mod to_lines;
pub mod transform;
mod tween_curves;
mod uv_curves;
mod wireframe;
#[cfg(feature = "occt-brep")]
mod wireframe_worker;
#[cfg(feature = "occt-brep")]
mod isocurve_worker;
pub use subobjects::SelectedSubobject;
mod addon_builder;
mod addon_dialog;
pub use addon_dialog::{AddonDialog, DialogKind, DialogRow, humanize};
mod addon_session;
mod boolean_builder_worker;
pub use addon_session::AddonSession;
mod addon_command;
pub use addon_command::IconTarget;
pub use addon_builder::{AddonBuilder, BuilderPreview, display_value};
pub(crate) mod display_state;
pub(crate) mod f6_state;
mod files;
#[cfg(feature = "occt-brep")]
pub(crate) use files::shared_brep_archive_cache;
#[cfg(feature = "occt-brep")]
pub(crate) use files::PreparedNamedBrepOpen;
mod clipboard;
pub(crate) use clipboard::jobbag_selection_closure;
#[cfg(feature = "occt-brep")]
pub(crate) use clipboard::PreparedBrepJobbag;
mod stl;
pub(crate) use stl::InteractiveStlExport;
pub use stl::{StlCancellation, StlExportCapture, StlExportJob, StlExportProfile};
pub use files::format_import_summary;
pub mod gumball;
pub use gumball::GumballAlignment;
mod jobbag;
mod join_history;
mod join_history_worker;
mod layer_dialogs;
mod layers;
pub(crate) mod pictureframe_state;
mod plugins;
pub(crate) mod project_state;
mod properties;
mod solid;
mod selection_memory;
mod solid_pending;
#[cfg(feature = "occt-brep")]
mod boolean_union_worker;
mod solid_geometry;
mod solid_pipe;
mod solid_boolean;
mod solid_cone;
mod solid_pyramid;
mod solid_text_stages;
mod surface_commands;
mod surface_edit;
mod top11;
pub use top11::GripPick;
pub use properties::{FIELDS, FieldState, PropertyEdit, PropertyField};
pub const LINE: &str = "OM9-CURVE-002";
pub const ZOOM_EXTENTS: &str = "ViewZoomZoomExtents";
pub const ZOOM_SELECTED: &str = "ViewZoomZoomSelected";
pub const VIEW_HOME: &str = "ViewHome";
pub const VIEW_CENTER: &str = "ViewCenter";
pub const POINT: &str = "OM9-CURVE-034";
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationOrigin {
    Menu,
    Cmd,
    F6,
    Builder,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchOutcome {
    Command,
    File(crate::file_lifecycle::FileOutcome),
    Top11(top11::Top11Result),
    /// File ▸ Export Selected asks the UI to open the `.stl` save dialog; the
    /// Controller owns the export itself.
    StlExport,
}
/// Which step of an interactive command is waiting for the user
/// (OM9-L3-01-02-004). It is derived from the one session state, so the prompt,
/// the audit and the refusals cannot disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdPhase {
    /// No command is running.
    Idle,
    /// Waiting for the first point.
    AwaitPoint,
    /// Waiting for the end point of a two-point command.
    AwaitEnd,
}

/// The session facts one command run was started against (OM9-L3-01-02-003):
/// which document, which view, what was selected, and the revisions that make
/// the snapshot checkable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandContext {
    pub document: DocumentId,
    pub view: om9_scene::workspace::ViewId,
    pub selection: Vec<om9_core::identity::ObjectId>,
    pub revision: u64,
    pub epoch: u64,
}

/// One read-only view of the active command input: owner, run generation,
/// phase and the option snapshots derived from the same owner state.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandInputSnapshot {
    pub owner: Option<&'static str>,
    pub run: u64,
    pub input_revision: u64,
    pub phase: CmdPhase,
    pub options: Vec<crate::command_options::CommandOption>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SphereRecoveryKey {
    run: u64,
    pub(crate) kind: &'static str,
    points_revision: u64,
    points: [[u64; 3]; 3],
    pub(crate) count: usize,
    center: Option<[u64; 3]>,
    radius: u64,
    normal: [u64; 3],
    plane: Option<CPlane>,
    source: Option<om9_core::identity::SourceStamp>,
    solution_negative: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BoxRecoveryKey {
    run: u64,
    kind: &'static str,
    points: [[u64; 3]; 2],
    count: usize,
    length: u64,
    width: Option<u64>,
    plane: CPlane,
}
#[derive(Debug, Clone)]
struct CommandSession {
    feature: &'static str,
    context: CommandContext,
    plane: CPlane,
    unit: Unit,
    start: Option<Point3>,
    both_sides: bool,
    /// `Length=…` fixes the end point's distance from the start; the click then
    /// only chooses the direction (OM9-L3-01-02-005).
    length_mm: Option<f64>,
    /// Points already collected by the three-corner plane command.
    points: Vec<Point3>,
}

/// A selected-surface command waiting for one numeric distance.
#[derive(Debug, Clone)]
struct SurfacePending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
}

/// A profile/rail command waiting for an axis origin and direction.
#[derive(Debug, Clone)]
struct CurvePairPending {
    feature: &'static str,
    profile: Curve,
    rail: Curve,
}

/// A two-surface command waiting for source/target edge names.
#[derive(Debug, Clone)]
struct SurfacePairPending {
    feature: &'static str,
    source: om9_core::geometry::NurbsSurface,
    target: om9_core::geometry::NurbsSurface,
    source_edge: Option<om9_core::surface_ops::SurfaceEdge>,
    target_edge: Option<om9_core::surface_ops::SurfaceEdge>,
    /// Match continuity; Position (G0) is the documented default.
    continuity: om9_core::surface_ops::SurfaceContinuity,
}

/// A seam command waiting for an axis and tolerance.
#[derive(Debug, Clone)]
struct SeamPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
    axis: Option<om9_core::surface_ops::SurfaceIso>,
    tolerance_mm: Option<f64>,
}

/// A soft-edit command waiting for its base point, move vector and documented options.
#[derive(Debug, Clone)]
struct SoftEditPending {
    feature: &'static str,
    id: om9_core::identity::ObjectId,
    surface: om9_core::geometry::NurbsSurface,
    /// `U`,`V` base point in the surface parameter domain.
    base: [Option<f64>; 2],
    /// `Dx`,`Dy`,`Dz` move vector; Matrix's "pick a point to move to" minus the base.
    delta: [Option<f64>; 3],
    u_distance: f64,
    v_distance: f64,
    copy: bool,
    fix_edges: bool,
    constraint: self::surface_edit::SoftEditConstraint,
}

impl SoftEditPending {
    fn base_complete(&self) -> bool {
        self.base.iter().all(Option::is_some)
    }

    fn delta_complete(&self) -> bool {
        self.delta.iter().all(Option::is_some)
    }

    fn base(&self) -> [f64; 2] {
        [self.base[0].expect("base u"), self.base[1].expect("base v")]
    }

    fn delta(&self) -> [f64; 3] {
        [
            self.delta[0].expect("delta x"),
            self.delta[1].expect("delta y"),
            self.delta[2].expect("delta z"),
        ]
    }
}

/// A tween command waiting for a fraction between two surfaces.
#[derive(Debug, Clone)]
struct TweenPending {
    feature: &'static str,
    first: om9_core::geometry::NurbsSurface,
    second: om9_core::geometry::NurbsSurface,
}

/// An extend command waiting for an edge and distance.
#[derive(Debug, Clone)]
struct ExtendPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
    edge: Option<om9_core::surface_ops::SurfaceEdge>,
    distance_mm: Option<f64>,
}

/// A split command waiting for an axis and parameter.
#[derive(Debug, Clone)]
struct SplitPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
    axis: Option<om9_core::surface_ops::SurfaceIso>,
    parameter: Option<f64>,
}

/// A tangent command waiting for an edge and direction.
#[derive(Debug, Clone)]
struct TangentPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
    edge: Option<om9_core::surface_ops::SurfaceEdge>,
    direction: Option<om9_core::geometry::Point3>,
}

/// An extrude-all command waiting for a height.
#[derive(Debug, Clone)]
struct ExtrudeAllPending {
    feature: &'static str,
    curves: Vec<Curve>,
}

/// A variable-offset command waiting for two distances and an axis.
#[derive(Debug, Clone)]
struct VarOffsetPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
    start: Option<f64>,
    end: Option<f64>,
    axis: Option<om9_core::surface_ops::SurfaceIso>,
}

/// A shrink command waiting for a UV sub-domain.
#[derive(Debug, Clone)]
struct ShrinkPending {
    feature: &'static str,
    surface: om9_core::geometry::NurbsSurface,
    u0: Option<f64>,
    u1: Option<f64>,
    v0: Option<f64>,
    v1: Option<f64>,
}

/// A variable-blend command waiting for two tangent scales.
#[derive(Debug, Clone)]
struct VarBlendPending {
    feature: &'static str,
    first: Curve,
    second: Curve,
    start_scale: Option<f64>,
    end_scale: Option<f64>,
}

/// A chamfer command waiting for a setback.
#[derive(Debug, Clone)]
struct ChamferPending {
    feature: &'static str,
    first: Curve,
    second: Curve,
}

/// A fillet command waiting for one or two radii.
#[derive(Debug, Clone)]
struct FilletPending {
    feature: &'static str,
    first: Curve,
    second: Curve,
    start_radius: Option<f64>,
    end_radius: Option<f64>,
}

/// A variable chamfer command waiting for two setbacks.
#[derive(Debug, Clone)]
struct VarChamferPending {
    feature: &'static str,
    first: Curve,
    second: Curve,
    start_setback: Option<f64>,
    end_setback: Option<f64>,
}

/// A Sweep History command waiting for Closed/Flip/Simple options.
#[derive(Debug, Clone)]
struct HistoryOptionsPending {
    feature: &'static str,
    curves: Vec<Curve>,
    ids: Vec<om9_core::identity::ObjectId>,
    rail_count: usize,
    closed: bool,
    flip: bool,
    /// `Closed` is only offered when every rail is closed and at least two profiles
    /// are selected (Matrix p183/p186); otherwise it starts `No` and is not offered.
    closed_available: bool,
}

/// Sweep 2 Add Slash: two alignment points on the two rails.
#[derive(Debug, Clone)]
struct SlashPending {
    feature: &'static str,
    curves: Vec<Curve>,
    ids: Vec<om9_core::identity::ObjectId>,
    rail_count: usize,
    closed: bool,
    flip: bool,
    points: Vec<Point3>,
}
struct PreparedInitialScene {
    coordinator: TransactionCoordinator,
    scene: Scene,
    file: crate::file_lifecycle::FileState,
    id: DocumentId,
}

pub struct Controller {
    viewport_detail: crate::display::ViewportDetail,
    #[cfg(feature = "occt-brep")]
    viewport_detail_change: Option<topology_remesh::DetailChange>,
    coordinator: TransactionCoordinator,
    file_open_worker: Option<files::FileOpenWorker>,
    #[cfg(feature = "occt-brep")]
    file_save_worker: Option<files::FileSaveWorker>,
    #[cfg(feature = "occt-brep")]
    brep_archive_cache: Option<Arc<files::BrepArchiveCache>>,
    /// Invalidated when this document session is replaced; captured exports
    /// retain the old token until they finish or acknowledge cancellation.
    stl_session: stl::StlSessionOwner,
    /// Monotonic command-start generation used to reject stale option IDs.
    command_run: u64,
    input_revision: u64,
    sphere_recovery_run: u64,
    box_recovery_run: u64,
    sphere_points_revision: u64,
    isocurve_recovery_run: u64,
    invoked_feature: Option<String>,
    plugins: Vec<std::rc::Rc<om9_plugin_host::LoadedPlugin>>,
    #[cfg(feature = "occt-brep")]
    sdk_snap_workers: std::cell::RefCell<plugins::PluginSnapWorkers>,
    /// The last accepted world point, for `@x,y` relative entry.
    last_point: Option<om9_core::coords::WorldPoint>,
    /// Cursor constraints applied to viewport picks, not to typed coordinates.
    ortho: bool,
    planar: bool,
    project: bool,
    grid_snap: bool,
    grid_snap_step_mm: f64,
    /// Gumball placement: an explicit origin (relocate) and the alignment of its axes.
    gumball_origin: Option<om9_core::geometry::Point3>,
    gumball_alignment: crate::session::gumball::GumballAlignment,
    gumball_bounds_cache: std::cell::RefCell<Option<crate::session::gumball::BoundsCache>>,
    /// Snappy drag uses the active object-snap candidate; Smooth (default) uses
    /// the raw pointer ray. Stored as a session setting, not a widget cache.
    gumball_snappy: bool,
    /// Drag strength: 1.0 moves the pointer distance, 0.5 half. Snapped and
    /// numeric input remain exact.
    gumball_drag_strength: f64,
    /// Pending Top 11 point command (OM9-L2-03-08).
    top11_pending: Option<self::top11::Top11Pending>,
    dupedge_pending: Option<CommandContext>,
    #[cfg(feature = "occt-brep")]
    dupedge_session_id: uuid::Uuid,
    #[cfg(feature = "occt-brep")]
    dupedge_source_view_active: Arc<std::sync::atomic::AtomicBool>,
    #[cfg(feature = "occt-brep")]
    dupedge_source_view_job: Option<DupEdgeSourceViewJob>,
    #[cfg(feature = "occt-brep")]
    face_verification_job: Option<face_verification::FaceVerificationJob>,
    #[cfg(feature = "occt-brep")]
    face_verification_input: Option<String>,
    #[cfg(feature = "occt-brep")]
    face_verification_active: Arc<std::sync::atomic::AtomicBool>,
    #[cfg(feature = "occt-brep")]
    face_verification_receipt: Option<face_verification::FaceVerificationReceipt>,
    #[cfg(feature = "occt-brep")]
    wireframe_worker: Option<wireframe_worker::WireframeJob>,
    #[cfg(feature = "occt-brep")]
    isocurve_build_worker: Option<isocurve_worker::IsocurveJob>,
    #[cfg(feature = "occt-brep")]
    boolean_union_worker: Option<boolean_union_worker::UnionJob>,
    #[cfg(feature = "occt-brep")]
    background_boolean_union_enabled: bool,
    #[cfg(feature = "occt-brep")]
    boolean_union_active: Arc<std::sync::atomic::AtomicBool>,
    #[cfg(feature = "occt-brep")]
    background_isocurve_build_enabled: bool,
    #[cfg(feature = "occt-brep")]
    isocurve_build_active: Arc<std::sync::atomic::AtomicBool>,
    #[cfg(feature = "occt-brep")]
    background_wireframe_enabled: bool,
    #[cfg(feature = "occt-brep")]
    wireframe_worker_active: Arc<std::sync::atomic::AtomicBool>,
    #[cfg(feature = "occt-brep")]
    dupedge_scene_backup: Option<DupEdgeSceneBackup>,
    dupedge_source_picks: Vec<self::wireframe::DupEdgeSourcePick>,
    dupedge_picks_reservation: Option<om9_core::transaction::ExternalMemoryReservation>,
    #[cfg(feature = "occt-brep")]
    native_isocurve_pick: Option<self::isocurve::NativeIsocurvePick>,
    solid_pending: Option<solid::SolidPending>,
    circle_pending: Option<self::circle::CirclePending>,
    circle_rebuild: Option<(bool, usize, usize)>,
    rect_pending: Option<self::rect::RectPending>,
    polyline_pending: Option<self::polyline::PolylinePending>,
    interp_pending: Option<self::interp::InterpPending>,
    ellipse_pending: Option<self::ellipse::EllipsePending>,
    arc_pending: Option<self::arc::ArcPending>,
    rebuild_pending: Option<self::rebuild::RebuildPending>,
    builder: Option<om9_core::builder::BuilderSession>,
    ring_rail_replace: Option<ring_rail::Confirmation>,
    #[cfg(feature = "occt-brep")]
    ring_preview_request: Option<ring_preview_worker::RingPreviewRequest>,
    #[cfg(feature = "occt-brep")]
    ring_preview_candidate: Option<ring_preview_worker::RingPreviewCandidate>,
    #[cfg(feature = "occt-brep")]
    ring_preview_active: Arc<std::sync::atomic::AtomicBool>,
    #[cfg(feature = "occt-brep")]
    background_ring_preview_enabled: bool,
    #[cfg(feature = "occt-brep")]
    ring_preview_refresh_queued: bool,
    #[cfg(feature = "occt-brep")]
    ring_preview_apply_queued: bool,
    /// Open gem tool (loader, builders, positioner); its preview owns nothing in the document.
    gem_session: Option<self::gem::GemSession>,
    /// Open F6 Gem Properties editor.
    gem_editor: Option<self::gem::GemPropertiesEditor>,
    /// Open gem-on-curve/surface placement tool (`OM9-L2-04-10`). Its preview owns nothing in
    /// the document; Apply commits one transaction.
    gem_placement: Option<self::gem_placement::GemPlacementSession>,
    /// Open halo/cluster/baguette builder (`OM9-L2-04-11`). Its preview owns nothing in the
    /// document; Apply commits one transaction.
    gem_cluster: Option<self::gem_cluster::GemClusterSession>,
    /// Open gem edit/report tool (`OM9-L2-04-13`). A mutating tool previews every stone and
    /// commits ONE transaction on Apply; the report tools never touch the document.
    gem_edits: Option<self::gem_edits::GemEditsSession>,
    /// Open pavé/azure/prong/sphere/springs builder (`OM9-L2-04-12`). Its preview owns
    /// nothing in the document; Apply commits one transaction.
    gem_pave: Option<self::gem_pave::GemPaveSession>,
    /// Open Gem Cutter / Gem Cutter Library builder (`OM9-L2-04-16`). Its preview owns nothing
    /// in the document; Enter commits one transaction (the Interactive Boolean cut plus the
    /// cutter bodies under the Preserve/Remove answer).
    gem_cutters: Option<self::gem_cutters::GemCutterSession>,
    /// Open Channel / Micro Prong / Bright Cut builder (`OM9-L2-04-17`). One staged session
    /// for the four row commands; Enter commits the Boolean cuts and the cutter bodies.
    row_cutters: Option<self::channel_cutters::RowCutterSession>,
    /// Inactive builder tabs, each keeping its own parameters while another tab is active.
    builder_saved: std::collections::BTreeMap<
        om9_core::builder::BuilderKind,
        om9_core::builder::BuilderSession,
    >,
    /// Builder styles: the built-in library plus the user's own (`OM9-L2-03-12`).
    style_library: om9_core::builder_style::StyleLibrary,
    /// Where user styles persist. `OM9_STYLE_LIBRARY` names the file; without it the user
    /// list lives for the session only.
    style_path: Option<std::path::PathBuf>,
    ring_rail_library: om9_core::ring_rail_shape::RailShapeLibrary,
    ring_rail_library_path: Option<std::path::PathBuf>,
    ring_rail_shape: Option<om9_core::ring_rail_shape::RailShape>,
    /// The last style-load problem, shown once in the panel instead of being swallowed.
    style_error: Option<String>,
    /// The style the open session last applied, for the browser's selected state.
    applied_style: Option<String>,
    /// Pavé styles (`OM9-L3-04-12-001`): built-ins plus the user's own snapshots.
    pave_style_library: om9_core::gem_pave::PaveStyleLibrary,
    /// Where user pavé styles persist (`OM9_PAVE_STYLE_LIBRARY`); without it they live for
    /// this session only.
    pave_style_path: Option<std::path::PathBuf>,
    /// Last pavé style problem, shown once in the builder window.
    pave_style_error: Option<String>,
    history_cache: om9_core::associative::RecomputeCache,
    /// Matrix History recording policy for recipes created from now on (`OM9-L3-03-13-001`).
    history_record: bool,
    /// Associative recompute policy (`OM9-L3-03-13-002`).
    history_update: bool,
    /// True while a rebuild owns the document; close/open refuses to race it (`-006`).
    history_rebuilding: bool,
    /// One-shot message for the UI status line after a history operation.
    history_notice: Option<String>,
    /// The command line asked for the history report window.
    history_window_requested: bool,
    command_history_requested: bool,
    fit_pending: Option<self::fit::FitPending>,
    refit_srf_pending: Option<self::refit_srf::RefitSrfPending>,
    offset_curve_pending: Option<self::offset::OffsetPending>,
    project_pending: Option<self::project::ProjectPending>,
    blend_pending: Option<self::blend::BlendPending>,
    isocurve_pending: Option<self::isocurve::IsoPending>,
    pull_pending: Option<self::pull::PullPending>,
    intersect_pending: Option<self::intersect::IntersectPending>,
    crv2view_pending: Option<self::crv2view::Crv2ViewPending>,
    crv_seam_pending: Option<self::crv_seam::CrvSeamPending>,
    control_curve_pending: Option<self::control_curve::ControlCurvePending>,
    offset_on_surface_pending: Option<self::offset_on_surface::OffsetOnSurfacePending>,
    offset_normal_pending: Option<self::offset_normal::OffsetNormalPending>,
    corner_pending: Option<self::corner_ops::CornerPending>,
    cross_section_pending: Option<self::cross_section::CrossSectionPending>,
    fillet_corners_pending: Option<self::fillet_corners::FilletCornersPending>,
    continue_curve_pending: Option<self::continue_curve::ContinueCurvePending>,
    through_points_pending: Option<self::through_points::ThroughPointsPending>,
    polygon_pending: Option<self::polygon::PolygonPending>,
    helix_pending: Option<self::helix::HelixPending>,
    spiral_pending: Option<self::spiral::SpiralPending>,
    divide_pending: Option<self::divide::DividePending>,
    to_lines_pending: Option<self::to_lines::ToLinesPending>,
    extend_curve_pending: Option<self::extend_curve::ExtendPending>,
    tween_curves_pending: Option<self::tween_curves::TweenCurvesPending>,
    continue_interp_pending: Option<self::continue_interp::ContinueInterpPending>,
    arc_blend_pending: Option<self::arc_blend::ArcBlendPending>,
    /// Pending numeric distance for Offset/Extrude Normal.
    surface_pending: Option<SurfacePending>,
    curve_pair_pending: Option<CurvePairPending>,
    surface_pair_pending: Option<SurfacePairPending>,
    seam_pending: Option<SeamPending>,
    soft_edit_pending: Option<SoftEditPending>,
    curve_soft_edit_pending: Option<self::curve_soft_edit::CurveSoftEditPending>,
    angle_isocurve_pending: Option<self::angle_isocurve::AngleIsocurvePending>,
    extrude_surface_pending: Option<self::extrude_surface::ExtrudePending>,
    heightfield_pending: Option<self::heightfield::HeightfieldPending>,
    tween_pending: Option<TweenPending>,
    extend_pending: Option<ExtendPending>,
    split_pending: Option<SplitPending>,
    tangent_pending: Option<TangentPending>,
    extrude_all_pending: Option<ExtrudeAllPending>,
    variable_offset_pending: Option<VarOffsetPending>,
    shrink_pending: Option<ShrinkPending>,
    variable_blend_pending: Option<VarBlendPending>,
    chamfer_pending: Option<ChamferPending>,
    fillet_pending: Option<FilletPending>,
    variable_chamfer_pending: Option<VarChamferPending>,
    history_options_pending: Option<HistoryOptionsPending>,
    slash_pending: Option<SlashPending>,
    /// Active read-out of the Measure menu (OM9-L2-03-20). It is read-only: the
    /// document and the Undo journal never change.
    measure_pending: Option<self::measure::MeasurePending>,
    annotation_pending: Option<self::annotation::Pending>,
    annotation_style: om9_core::annotation::Style,
    transform_pending: Option<self::transform::TransformPending>,
    /// The last published read-out, tagged with the selection it measured.
    last_measurement: Option<self::measure::MeasurementLabel>,
    /// Metal material registry and the last valid Metal Weights report (`OM9-L2-04-19`).
    metal_registry: om9_core::metal_weight::MetalMaterialRegistry,
    last_metal_report: Option<self::metal::MetalWeightReport>,
    last_metal_catalog_report: Option<self::metal::MetalCatalogReport>,
    metal_panel_open: bool,
    metal_panel_material: Option<String>,
    metal_panel_object_materials: bool,
    metal_request_generation: u64,
    join_history_worker: Option<join_history_worker::JoinHistoryJob>,
    boolean_builder_worker: Option<boolean_builder_worker::BooleanBuilderJob>,
    boolean_builder_active: Arc<std::sync::atomic::AtomicBool>,
    background_boolean_builder: bool,
    join_history_worker_active: Arc<std::sync::atomic::AtomicBool>,
    background_join_history: bool,
    #[cfg(feature = "occt-brep")]
    metal_worker: Option<metal_worker::MetalJob>,
    #[cfg(feature = "occt-brep")]
    metal_worker_active: std::sync::Arc<std::sync::atomic::AtomicBool>,
    #[cfg(feature = "occt-brep")]
    metal_report_owner: Option<om9_core::transaction::ExternalMemoryReservation>,
    #[cfg(feature = "occt-brep")]
    metal_catalog_owner: Option<om9_core::transaction::ExternalMemoryReservation>,
    #[cfg(feature = "occt-brep")]
    background_metal: bool,
    metal_panel_reference: bool,
    metal_panel_user: bool,
    metal_material_bindings: std::collections::BTreeMap<(om9_core::identity::DocumentId, om9_core::identity::MaterialId), String>,
    /// Staged head/bezel command (`OM9-L2-04-14`).
    head_pending: Option<self::head_commands::HeadPending>,
    head_sequence: u64,
    /// Staged setting-parts command (`OM9-L2-04-15`) and the features it already committed.
    settings_pending: Option<self::settings_commands::SettingsPending>,
    settings_volumes: Vec<(String, f64)>,
    /// Settings features committed as cutters; they remove metal and stay out of the mass.
    pub(crate) settings_cutters: Vec<String>,
    /// Volume each committed cutter removes, so the mass read-out can report the net metal.
    pub(crate) settings_cutter_volumes: Vec<(String, f64)>,
    /// User (Bezier) profiles saved in this session; the profile library is editable here.
    pub(crate) head_profiles: Vec<om9_core::settings::head::Profile>,
    /// The last committed channel border, reused by Metal Piece's optional border input.
    pub(crate) settings_border: Option<om9_core::settings::channel_border::ChannelBorder>,
    settings_sequence: u64,
    last_settings_mass: Option<self::settings_commands::SettingsMassReport>,
    /// Active edit/control point display for the selected curves.
    top11_grip: Option<om9_core::top11::GripMode>,
    /// Grips picked by click or window while grips are on: (object, grip index).
    grip_selection: Vec<(om9_core::identity::ObjectId, usize)>,
    /// The gumball transform being previewed on the selected grips.
    grip_preview: Option<om9_core::geometry::Transform>,
    /// Faces, edges and vertices picked with Ctrl+Shift+click, in pick order.
    subselection: Vec<SelectedSubobject>,
    // Drop nested paths/slots before releasing their shared-account owner.
    subselection_reservation: Option<om9_core::transaction::ExternalMemoryReservation>,
    sub_revision: u64,
    /// Bumped while a gumball drag previews, so every viewport redraws the moving object.
    preview_revision: u64,
    document: DocumentId,
    /// Optional in-process proofs for attested roots. Document-changing scene
    /// publications invalidate them; verified display-only replacement retains them.
    #[cfg(feature = "occt-brep")]
    topology_cache: Option<Arc<topology_cache::TopologyProofCache>>,
    #[cfg(feature = "occt-brep")]
    remesh_representation_cache: Option<Arc<topology_remesh::RemeshRepresentationCache>>,
    #[cfg(feature = "occt-brep")]
    topology_remesh_job: Option<topology_remesh::PendingTopologyRemesh>,
    scene: Option<Scene>,
    /// Scene positions of each object, for add-on steps (see `addon_session::SceneRoots`).
    scene_roots: std::sync::Mutex<addon_session::SceneRoots>,
    // Declared after scene so its last lease is released after scene buffers.
    scene_buffer_owner: std::sync::Mutex<Option<SceneBufferOwner>>,
    // Keep the old-scene reservation until after the scene and its buffers drop.
    history_worker: Option<HistoryWorker>,
    session: Option<CommandSession>,
    recent: VecDeque<String>,
    repeat: Option<String>,
    view_menu_generation: u64,
    selection: std::collections::BTreeSet<om9_core::identity::ObjectId>,
    /// Selection in pick order, filtered by the live set. Commands that combine
    /// multiple curves/surfaces use this order; stale entries are ignored.
    selection_order: Vec<om9_core::identity::ObjectId>,
    // Declared after the collections: drop their storage before its lease.
    selection_reservation: Option<om9_core::transaction::ExternalMemoryReservation>,
    /// Bumped whenever the selection set changes, so a published selection
    /// state has a revision a caller can compare.
    selection_revision: u64,
    layer_dialog: Option<crate::layer_dialog::LayerDialog>,
    layer_dialog_serial: u64,
    file: crate::file_lifecycle::FileState,
    /// The ordered run events of this session (OM9-L3-01-02-010).
    pub events: crate::command_events::EventLog,
    /// Document change observers (OM9-L3-01-03-011). They receive an owned batch
    /// after the document is final and cannot reach the controller.
    observers: Vec<Box<dyn contract::DocumentObserver>>,
    observer_diagnostics: Vec<contract::ObserverDiagnostic>,
    changes: VecDeque<om9_core::changes::ChangeBatch>,
    /// The batch recorded for the last plugin command; it is the feed a following
    /// plugin command reads, independent of the observer baseline.
    plugin_change: Option<om9_core::changes::ChangeBatch>,
    /// The revision the change batches are diffed against. It is the world's own
    /// `Arc`, so recording costs an `Arc` clone, never a document copy.
    published: Option<std::sync::Arc<Document>>,
    pub workspace: Workspace,
    /// Installed add-ons (Preferences › Add-ons). `None` in tests and audits.
    addons: Option<crate::addons::SharedHost>,
    /// lowercase command name / feature id / catalog code → catalog code an add-on provides.
    addon_names: std::collections::BTreeMap<String, String>,
    /// Messages add-on commands produced, drained into the CMD history by the UI.
    addon_messages: Vec<String>,
    /// catalog code → typed Matrix command name.
    addon_typed: std::collections::BTreeMap<String, String>,
    /// The open builder of an add-on command, if any.
    addon_builder: Option<addon_builder::AddonBuilder>,
    /// The running interactive add-on command (Polyline, Move, Gem Cutter …), if any.
    addon_session: Option<addon_session::AddonSession>,
    /// The last add-on command that ran to its end: right-click or an empty Enter with no
    /// command running starts it again (Rhino's repeat).
    addon_last: Option<String>,
    /// Interface language of the add-on commands (Tiếng Việt / English), from the enabled
    /// add-ons' `lang` tables and their `language` setting. English without add-ons.
    tr: om9_addons::i18n::Tr,
}
impl Controller {
    pub fn viewport_detail(&self) -> crate::display::ViewportDetail { self.viewport_detail }
    /// Used for a fresh session before any geometry is created.
    pub(crate) fn initialize_viewport_detail(&mut self, detail: crate::display::ViewportDetail) {
        self.viewport_detail = detail;
    }
    pub fn capture_view_menu(
        &self,
        id: om9_scene::workspace::ViewId,
    ) -> Result<crate::viewport_menu::ViewMenuCapture> {
        let (context, _) = self.coordinator.capture_context(None)?;
        let view = self
            .workspace
            .views
            .get(&id)
            .ok_or(Error::Missing("viewport menu target"))?;
        if view.id != id {
            return Err(Error::Invalid("viewport menu key/identity"));
        }
        Ok(crate::viewport_menu::ViewMenuCapture::new(
            context,
            self.document(),
            view,
            &self.workspace,
            self.view_menu_generation,
        ))
    }
    pub fn validate_view_menu(
        &self,
        capture: &crate::viewport_menu::ViewMenuCapture,
    ) -> Result<()> {
        let (context, _) = self.coordinator.capture_context(None)?;
        capture.validate(
            context,
            self.document(),
            &self.workspace,
            self.view_menu_generation,
        )
    }
    pub fn apply_view_menu(
        &mut self,
        capture: &crate::viewport_menu::ViewMenuCapture,
        action: crate::viewport_menu::MenuAction,
    ) -> Result<()> {
        self.validate_view_menu(capture)?;
        action.availability()?;
        let generation = self
            .view_menu_generation
            .checked_add(1)
            .ok_or(Error::Overflow("viewport menu generation"))?;
        match action {
            crate::viewport_menu::MenuAction::Maximize => {
                self.workspace.toggle_maximize(capture.target())
            }
            crate::viewport_menu::MenuAction::Preset(
                crate::viewport_menu::DisplayPreset::Wireframe,
            )
            | crate::viewport_menu::MenuAction::Preset(
                crate::viewport_menu::DisplayPreset::Shaded,
            ) => {
                let mode = match action {
                    crate::viewport_menu::MenuAction::Preset(
                        crate::viewport_menu::DisplayPreset::Shaded,
                    ) => om9_scene::workspace::DisplayMode::Shaded,
                    _ => om9_scene::workspace::DisplayMode::Wireframe,
                };
                let view = self
                    .workspace
                    .views
                    .get_mut(&capture.target())
                    .ok_or(Error::Missing("viewport menu target"))?;
                if view.mode != mode {
                    let revision = view
                        .revision
                        .checked_add(1)
                        .ok_or(Error::Overflow("view revision"))?;
                    view.mode = mode;
                    view.revision = revision;
                }
                Ok(())
            }
            crate::viewport_menu::MenuAction::SetCPlane(preset) => {
                let view = self
                    .workspace
                    .views
                    .get_mut(&capture.target())
                    .ok_or(Error::Missing("viewport menu target"))?;
                let revision = view
                    .cplane
                    .revision
                    .checked_add(1)
                    .ok_or(Error::Overflow("CPlane revision"))?;
                view.cplane = CPlane {
                    revision,
                    ..CPlane::preset(preset)
                };
                Ok(())
            }
            _ => unreachable!("availability admitted only implemented actions"),
        }?;
        self.view_menu_generation = generation;
        Ok(())
    }
    pub fn new(document: Document, limits: Limits) -> Result<Self> {
        let controller = Self::new_with_held(document, limits, om9_core::budget::Usage::default())?;
        let scene = controller.scene.as_ref().ok_or(Error::Missing("initial scene"))?;
        if scene.shared_buffer_bytes() != 0 {
            controller.share_scene_buffer_owner(scene)?;
        }
        scene.reserve_memory_allowance(&controller.coordinator, controller.auxiliary_usage()?,
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)))?;
        Ok(controller)
    }
    #[cfg(test)]
    pub(crate) fn test_with_legacy_displays(document: Document, limits: Limits,
        displays: om9_core::legacy_v4::LegacyDisplays) -> Result<Self> {
        let controller = Self::new_with_held_and_setup(document, limits, Default::default(), |coordinator, held| {
            let mut reservation = coordinator.reserve_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::CpuCache, 4 * 1024 * 1024, None, held)?;
            coordinator.adopt_legacy_displays(displays, &mut reservation, held)?;
            Ok(held)
        })?;
        let scene = controller.scene.as_ref().ok_or(Error::Missing("initial scene"))?;
        if scene.shared_buffer_bytes() != 0 { controller.share_scene_buffer_owner(scene)?; }
        scene.reserve_memory_allowance(&controller.coordinator, controller.auxiliary_usage()?,
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)))?;
        Ok(controller)
    }
    fn new_with_held(
        document: Document,
        limits: Limits,
        held: om9_core::budget::Usage,
    ) -> Result<Self> {
        Self::new_with_held_and_setup(document, limits, held, |_, held| Ok(held))
    }
    /// Prepare account-owned representations after the private world exists
    /// and before its scene is built. Setup may replace the scene held usage
    /// when the candidate joins an existing shared account.
    fn new_with_held_and_setup(
        document: Document,
        limits: Limits,
        held: om9_core::budget::Usage,
        setup: impl FnOnce(&mut TransactionCoordinator, om9_core::budget::Usage) -> Result<om9_core::budget::Usage>,
    ) -> Result<Self> {
        Self::from_prepared_initial(Self::prepare_initial_scene(document, limits, held, setup)?)
    }
    fn prepare_initial_scene(
        document: Document,
        limits: Limits,
        held: om9_core::budget::Usage,
        setup: impl FnOnce(&mut TransactionCoordinator, om9_core::budget::Usage) -> Result<om9_core::budget::Usage>,
    ) -> Result<PreparedInitialScene> {
        let mut held = held;
        held.add(Workspace::startup_usage()?, &limits)?;
        let id = document.id;
        let remaining = limits.remaining_after(held)?;
        let mut peak = held;
        peak.add(document.validate(&remaining)?, &limits)?;
        peak.add(
            om9_core::budget::Usage::btree::<DocumentId, Arc<Document>>(1)?,
            &limits,
        )?;
        peak.add(om9_core::budget::Usage::array::<Document>(1)?, &limits)?;
        let file = crate::file_lifecycle::FileState::initialized(&document)?;
        let mut coordinator = TransactionCoordinator::new_with_held(
            World::new(vec![document], &remaining)?,
            limits,
            held,
        )?;
        coordinator.set_physical_memory_sampler(|| {
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes))
        });
        #[cfg(windows)]
        coordinator.set_commit_memory_sampler(om9_native_owner::commit_memory);
        if let Some(total) = om9_native_owner::physical_memory_bytes() {
            coordinator.constrain_managed_ram(total, held)?;
            coordinator.set_undo_ram_capacity(total)?;
        }
        let held = setup(&mut coordinator, held)?;
        let scene = Scene::capture_in_world_with_physical_samples(&coordinator, id, held, || {
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes))
        })?;
        Ok(PreparedInitialScene { coordinator, scene, file, id })
    }
    fn from_prepared_initial(prepared: PreparedInitialScene) -> Result<Self> {
        let PreparedInitialScene { coordinator, scene, file, id } = prepared;
        // A legacy imported display is not evidence that the lightweight profile
        // has been built. Fresh sessions still default to Light.
        let viewport_detail = if scene.document.entities.values().any(|e| matches!(e.geometry, Geometry::Brep(_))) {
            crate::display::ViewportDetail::High
        } else { crate::display::ViewportDetail::Light };
        Ok(Self {
            scene_roots: Default::default(),
            coordinator,
            file_open_worker: None,
            #[cfg(feature = "occt-brep")]
            file_save_worker: None,
            #[cfg(feature = "occt-brep")]
            brep_archive_cache: None,
            history_worker: None,
            command_run: 0,
            input_revision: 0,
            sphere_recovery_run: 0,
            box_recovery_run: 0,
            sphere_points_revision: 0,
            isocurve_recovery_run: 0,
            invoked_feature: None,
            plugins: Vec::new(),
            #[cfg(feature = "occt-brep")]
            sdk_snap_workers: std::cell::RefCell::new(plugins::PluginSnapWorkers::default()),
            last_point: None,
            ortho: false,
            planar: false,
            grid_snap: false,
            grid_snap_step_mm: 0.5,
            project: false,
            gumball_origin: None,
            gumball_alignment: crate::session::gumball::GumballAlignment::World,
            gumball_bounds_cache: std::cell::RefCell::new(None),
            gumball_snappy: false,
            gumball_drag_strength: 1.0,
            top11_pending: None,
            dupedge_pending: None,
            #[cfg(feature = "occt-brep")]
            dupedge_session_id: uuid::Uuid::nil(),
            #[cfg(feature = "occt-brep")]
            dupedge_source_view_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            #[cfg(feature = "occt-brep")]
            dupedge_source_view_job: None,
            #[cfg(feature = "occt-brep")]
            face_verification_job: None,
            #[cfg(feature = "occt-brep")]
            face_verification_input: None,
            #[cfg(feature = "occt-brep")]
            face_verification_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            #[cfg(feature = "occt-brep")]
            face_verification_receipt: None,
            #[cfg(feature = "occt-brep")]
            wireframe_worker: None,
            #[cfg(feature = "occt-brep")]
            isocurve_build_worker: None,
            #[cfg(feature = "occt-brep")]
            boolean_union_worker: None,
            #[cfg(feature = "occt-brep")]
            background_boolean_union_enabled: false,
            viewport_detail,
            #[cfg(feature = "occt-brep")]
            viewport_detail_change: None,
            #[cfg(feature = "occt-brep")]
            boolean_union_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            #[cfg(feature = "occt-brep")]
            background_isocurve_build_enabled: false,
            #[cfg(feature = "occt-brep")]
            isocurve_build_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            #[cfg(feature = "occt-brep")]
            background_wireframe_enabled: false,
            #[cfg(feature = "occt-brep")]
            wireframe_worker_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            #[cfg(feature = "occt-brep")]
            dupedge_scene_backup: None,
            dupedge_source_picks: Vec::new(),
            dupedge_picks_reservation: None,
            #[cfg(feature = "occt-brep")]
            native_isocurve_pick: None,
            solid_pending: None,
            circle_pending: None,
            circle_rebuild: None,
            rect_pending: None,
            polyline_pending: None,
            interp_pending: None,
            ellipse_pending: None,
            arc_pending: None,
            rebuild_pending: None,
            builder: None,
            ring_rail_replace: None,
            #[cfg(feature = "occt-brep")]
            ring_preview_request: None,
            #[cfg(feature = "occt-brep")]
            ring_preview_candidate: None,
            #[cfg(feature = "occt-brep")]
            ring_preview_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            #[cfg(feature = "occt-brep")]
            background_ring_preview_enabled: false,
            #[cfg(feature = "occt-brep")]
            ring_preview_refresh_queued: false,
            #[cfg(feature = "occt-brep")]
            ring_preview_apply_queued: false,
            gem_session: None,
            gem_editor: None,
            gem_placement: None,
            gem_cluster: None,
            gem_edits: None,
            gem_pave: None,
            gem_cutters: None,
            row_cutters: None,
            builder_saved: std::collections::BTreeMap::new(),
            style_library: {
                let mut library = om9_core::builder_style::StyleLibrary::builtin();
                if let Some(path) = std::env::var_os("OM9_STYLE_LIBRARY") {
                    // A broken user file never hides the built-in library; the error is kept
                    // for the panel to show once.
                    let _ = library.load_from(&std::path::PathBuf::from(path));
                }
                library
            },
            style_path: std::env::var_os("OM9_STYLE_LIBRARY").map(std::path::PathBuf::from),
            ring_rail_library: Default::default(),
            ring_rail_library_path: None,
            ring_rail_shape: None,
            style_error: None,
            applied_style: None,
            pave_style_library: {
                let mut library = om9_core::gem_pave::PaveStyleLibrary::builtin();
                if let Some(path) = std::env::var_os("OM9_PAVE_STYLE_LIBRARY")
                    && let Ok(loaded) = om9_core::gem_pave::PaveStyleLibrary::load_user(
                        &std::path::PathBuf::from(path),
                    )
                {
                    library = loaded;
                }
                library
            },
            pave_style_path: std::env::var_os("OM9_PAVE_STYLE_LIBRARY")
                .map(std::path::PathBuf::from),
            pave_style_error: None,
            history_cache: om9_core::associative::RecomputeCache::default(),
            history_record: true,
            history_update: true,
            history_rebuilding: false,
            history_notice: None,
            history_window_requested: false,
            command_history_requested: false,
            fit_pending: None,
            refit_srf_pending: None,
            offset_curve_pending: None,
            project_pending: None,
            blend_pending: None,
            isocurve_pending: None,
            pull_pending: None,
            intersect_pending: None,
            crv2view_pending: None,
            crv_seam_pending: None,
            control_curve_pending: None,
            offset_on_surface_pending: None,
            offset_normal_pending: None,
            corner_pending: None,
            cross_section_pending: None,
            fillet_corners_pending: None,
            continue_curve_pending: None,
            through_points_pending: None,
            polygon_pending: None,
            helix_pending: None,
            spiral_pending: None,
            divide_pending: None,
            to_lines_pending: None,
            extend_curve_pending: None,
            tween_curves_pending: None,
            continue_interp_pending: None,
            arc_blend_pending: None,
            surface_pending: None,
            curve_pair_pending: None,
            surface_pair_pending: None,
            seam_pending: None,
            soft_edit_pending: None,
            curve_soft_edit_pending: None,
            angle_isocurve_pending: None,
            extrude_surface_pending: None,
            heightfield_pending: None,
            tween_pending: None,
            extend_pending: None,
            split_pending: None,
            tangent_pending: None,
            extrude_all_pending: None,
            variable_offset_pending: None,
            shrink_pending: None,
            variable_blend_pending: None,
            chamfer_pending: None,
            fillet_pending: None,
            variable_chamfer_pending: None,
            history_options_pending: None,
            slash_pending: None,
            measure_pending: None,
            annotation_pending: None,
            annotation_style: om9_core::annotation::Style::default(),
            transform_pending: None,
            last_measurement: None,
            metal_registry: om9_core::metal_weight::MetalMaterialRegistry::builtin(),
            last_metal_report: None,
            last_metal_catalog_report: None,
            metal_panel_open: false,
            metal_panel_material: None,
            metal_panel_object_materials: false,
            metal_request_generation: 0,
            join_history_worker: None,
            boolean_builder_worker: None,
            boolean_builder_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            background_boolean_builder: false,
            join_history_worker_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            background_join_history: false,
            #[cfg(feature = "occt-brep")]
            metal_worker: None,
            #[cfg(feature = "occt-brep")]
            metal_worker_active: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            #[cfg(feature = "occt-brep")]
            metal_report_owner: None,
            #[cfg(feature = "occt-brep")]
            metal_catalog_owner: None,
            #[cfg(feature = "occt-brep")]
            background_metal: false,
            metal_panel_reference: true,
            metal_panel_user: true,
            metal_material_bindings: std::collections::BTreeMap::new(),
            head_pending: None,
            head_sequence: 0,
            settings_pending: None,
            settings_volumes: Vec::new(),
            settings_cutters: Vec::new(),
            settings_cutter_volumes: Vec::new(),
            head_profiles: Vec::new(),
            settings_border: None,
            settings_sequence: 0,
            last_settings_mass: None,
            top11_grip: None,
            grip_selection: Vec::new(),
            grip_preview: None,
            subselection: Vec::new(),
            subselection_reservation: None,
            sub_revision: 1,
            preview_revision: 1,
            stl_session: stl::StlSessionOwner::new(),
            scene_buffer_owner: std::sync::Mutex::new(None),
            document: id,
            #[cfg(feature = "occt-brep")]
            topology_cache: None,
            #[cfg(feature = "occt-brep")]
            remesh_representation_cache: None,
            #[cfg(feature = "occt-brep")]
            topology_remesh_job: None,
            scene: Some(scene),
            session: None,
            recent: VecDeque::new(),
            repeat: None,
            view_menu_generation: 0,
            selection: std::collections::BTreeSet::new(),
            selection_order: Vec::new(),
            selection_reservation: None,
            selection_revision: 1,
            layer_dialog: None,
            layer_dialog_serial: 0,
            file,
            events: crate::command_events::EventLog::new(),
            observers: Vec::new(),
            observer_diagnostics: Vec::new(),
            changes: VecDeque::new(),
            plugin_change: None,
            published: None,
            workspace: Workspace::new(),
            addons: None,
            addon_names: std::collections::BTreeMap::new(),
            addon_messages: Vec::new(),
            addon_typed: Default::default(),
            addon_builder: None,
            addon_session: None,
            addon_last: None,
            tr: om9_addons::i18n::Tr::english(),
        })
    }
    pub fn coordinator(&self) -> &TransactionCoordinator {
        &self.coordinator
    }
    pub(crate) fn box_recovery_key(&self) -> Option<BoxRecoveryKey> {
        let pending = self.solid_pending.as_ref()?;
        if !pending.box_mode { return None; }
        match pending.box_kind {
            "Corners" | "Diagonal" if pending.points.len() == 2 => {},
            "Cube" if pending.points.len() == 1 => {},
            "Center" if pending.points.len() == 1 && pending.box_length > 0. && pending.box_width.is_some() => {},
            "ThreePoint" | "Vertical" if pending.points.len() == 2 && pending.box_width.is_some() => {},
            _ => return None,
        }
        let mut points = [[0; 3]; 2];
        for (dest, point) in points.iter_mut().zip(&pending.points) { *dest = point.map(f64::to_bits); }
        Some(BoxRecoveryKey { run: self.box_recovery_run, kind: pending.box_kind, points,
            count: pending.points.len(), length: pending.box_length.to_bits(), width: pending.box_width.map(f64::to_bits),
            plane: pending.box_plane.unwrap_or_else(|| self.input_plane()) })
    }
    pub(crate) fn sphere_recovery_key(&self) -> Option<SphereRecoveryKey> {
        self.sphere_recovery_key_for_selection(false)
    }

    pub(crate) fn sphere_fit_selection_recovery_key(&self) -> Option<SphereRecoveryKey> {
        self.sphere_recovery_key_for_selection(true).filter(|key| key.kind == "FitPoints")
    }

    pub(crate) fn sphere_point_recovery_key(&self) -> Option<SphereRecoveryKey> {
        self.sphere_recovery_key_for_selection(true)
    }

    fn sphere_recovery_key_for_selection(&self, selecting: bool) -> Option<SphereRecoveryKey> {
        let pending = self.solid_pending.as_ref()?;
        if !pending.sphere_mode { return None; }
        match pending.sphere_kind {
            "FitPoints" if selecting => {},
            "Center" if pending.sphere_center.is_some() => {},
            "Vertical" if pending.sphere_center.is_some() => {},
            "AroundCurve" | "Tangent" if pending.sphere_center.is_some() && pending.sphere_sources.len() == 1 => {},
            "TwoPoint" if pending.points.len() == 1 => {},
            "ThreePoint" if pending.points.len() == 2 => {},
            "FourPoint" if pending.points.len() == 3 => {},
            "FitPoints" if pending.points.len() >= 4 => {},
            _ => return None,
        }
        let mut points = [[0; 3]; 3];
        for (dest, point) in points.iter_mut().zip(&pending.points) { *dest = point.map(f64::to_bits); }
        let plane = Some(pending.sphere_plane.unwrap_or_else(|| self.input_plane()));
        Some(SphereRecoveryKey { run: self.sphere_recovery_run, kind: pending.sphere_kind, points_revision: self.sphere_points_revision, points,
            count: pending.points.len(), center: pending.sphere_center.map(|p| p.map(f64::to_bits)),
            radius: pending.sphere_radius.to_bits(), normal: pending.sphere_normal.map(f64::to_bits), plane,
            source: pending.sphere_sources.first().copied(), solution_negative: pending.sphere_solution_negative })
    }
    pub fn discard_oldest_undo(&mut self) -> Result<bool> {
        self.coordinator.discard_oldest_undo()
    }
    pub fn set_undo_policy(&mut self, policy: om9_core::transaction::UndoPolicy) -> Result<()> {
        if self.is_running() { return Err(Error::Busy); }
        self.coordinator.set_undo_policy(policy)
    }
    pub(crate) fn disable_undo_for_benchmark(&mut self) -> Result<()> {
        self.coordinator.disable_undo_for_benchmark()
    }
    pub fn document(&self) -> &Document {
        &self.coordinator.world().documents[&self.document]
    }
    pub fn scene(&self) -> Option<&Scene> {
        self.scene.as_ref()
    }
    /// Adopt a proof whose buffers were admitted during construction. The
    /// proof is optional cache state; it never changes the document or Undo.
    #[cfg(feature = "occt-brep")]
    pub fn adopt_topology_proof(&mut self, stamp: om9_core::identity::SourceStamp,
        proof: om9_kernel::brep::TopologyAttestation,
        reservation: om9_core::transaction::ExternalMemoryReservation) -> Result<()> {
        let shared = self.prepare_shared_topology_proof(stamp, proof, reservation)?;
        self.cache_topology_proof(shared)
    }
    #[cfg(feature = "occt-brep")]
    fn prepare_shared_topology_proof(&self, stamp: om9_core::identity::SourceStamp,
        proof: om9_kernel::brep::TopologyAttestation,
        reservation: om9_core::transaction::ExternalMemoryReservation)
        -> Result<Arc<om9_kernel::brep::SharedTopologyProof>> {
        use om9_core::{budget::{Usage, memory::MemoryClass}, geometry::Geometry};
        let entity = self.document().resolve(&stamp)?;
        let Geometry::Brep(brep) = &entity.geometry else { return Err(Error::Invalid("topology proof geometry")); };
        let count = brep.topology().faces.len().checked_add(brep.topology().edges.len())
            .and_then(|n| n.checked_add(brep.topology().vertex_ids.len())).ok_or(Error::Overflow("topology proof scratch"))?;
        let bytes = Usage::array::<uuid::Uuid>(count)?.bytes.max(1);
        let held = self.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut scratch = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, bytes, physical(), held)?;
        let owned = om9_kernel::brep::RegisteredTopologyProof::adopt(&self.coordinator, stamp, proof, reservation, |live, pending| {
            let needed = live.checked_add(pending).ok_or(Error::Overflow("topology proof scratch"))?.max(1);
            if needed > scratch.bytes() {
                let additional = needed - scratch.bytes();
                self.coordinator.grow_external_memory_with_held(&mut scratch, additional, physical(), held)?;
            }
            scratch.set_allocated_bytes(live)?;
            scratch.recheck_pending()
        })?;
        let owner_reservation = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache,
            om9_kernel::brep::SharedTopologyProof::allocation_bytes(), physical(), held)?;
        om9_kernel::brep::SharedTopologyProof::new(owned, &self.coordinator, owner_reservation)
    }
    #[cfg(feature = "occt-brep")]
    fn cache_topology_proof(&mut self, shared: Arc<om9_kernel::brep::SharedTopologyProof>) -> Result<()> {
        #[cfg(test)]
        if TEST_REFUSE_TOPOLOGY_CACHE.with(|refuse| refuse.get()) { return Err(Error::Budget("test topology cache refusal")); }
        let held = self.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        if self.topology_cache.is_none() {
            let document = self.coordinator.world().documents.get(&self.document).ok_or(Error::Missing("topology cache document"))?;
            self.topology_cache = Some(topology_cache::TopologyProofCache::new(&self.coordinator, document, held, physical())?);
        }
        self.topology_cache.as_ref().expect("topology cache").insert(&self.coordinator, shared, held, physical())
    }
    #[cfg(feature = "occt-brep")]
    fn topology_proof_for(&self, object: Option<om9_core::identity::ObjectId>)
        -> Option<Arc<om9_kernel::brep::SharedTopologyProof>> {
        self.topology_proof_for_kind(object, false)
    }
    #[cfg(feature = "occt-brep")]
    fn topology_source_proof_for(&self, object: Option<om9_core::identity::ObjectId>)
        -> Option<Arc<om9_kernel::brep::SharedTopologyProof>> {
        self.topology_proof_for_kind(object, true)
    }
    #[cfg(feature = "occt-brep")]
    fn topology_proof_for_kind(&self, object: Option<om9_core::identity::ObjectId>, source_only: bool)
        -> Option<Arc<om9_kernel::brep::SharedTopologyProof>> {
        if let Some(cache) = &self.topology_cache {
            let candidate = if source_only { cache.find_source(object, self.document()) } else { cache.find(object, self.document()) };
            if let Some(proof) = candidate {
                let owner: Arc<dyn om9_core::transaction::OptionalMemoryCache> = cache.clone();
                let _ = self.coordinator.touch_optional_memory_cache(&owner);
                return Some(proof);
            }
        }
        let find = |id| {
            let stamp = self.document().stamp(id).ok()?;
            let proof = self.coordinator.history_evidence_for::<om9_kernel::brep::SharedTopologyProof>(&stamp)?;
            let Geometry::Brep(brep) = &self.document().entities.get(&id)?.geometry else { return None; };
            (if source_only { proof.may_candidate_source(brep) } else { proof.may_candidate_source(brep) }).then_some(proof)
        };
        match object {
            Some(id) => find(id),
            None => self.coordinator.first_current_evidence::<om9_kernel::brep::SharedTopologyProof>(
                self.document().id, |_, proof, brep| if source_only { proof.may_candidate_source(brep) } else { proof.may_candidate_source(brep) }),
        }
    }
    /// Keep the already-accounted proof with the insertion entry when possible.
    /// Failure leaves the committed document and its display untouched.
    #[cfg(feature = "occt-brep")]
    fn retain_inserted_topology_proof(&mut self, stamp: om9_core::identity::SourceStamp,
        proof: Arc<om9_kernel::brep::SharedTopologyProof>) -> Result<()> {
        #[cfg(test)]
        if TEST_REFUSE_SPHERE_HISTORY.with(|refuse| refuse.get()) { return Err(Error::Budget("test sphere history refusal")); }
        proof.verify_unchanged_owner(&self.coordinator)?;
        let bytes = proof.reserved_bytes()?;
        let held = self.scene_cache_usage()?;
        let owner: Arc<dyn std::any::Any + Send + Sync> = proof.clone();
        self.coordinator.retain_insert_evidence(&stamp, owner.clone(), bytes, held)?;
        self.retain_topology_source(&stamp, &proof, &owner)
    }
    #[cfg(feature = "occt-brep")]
    fn retain_current_topology_proof(&mut self, proof: Arc<om9_kernel::brep::SharedTopologyProof>) -> Result<()> {
        proof.verify_unchanged_source(&self.coordinator)?;
        let stamp = proof.stamp();
        let bytes = proof.reserved_bytes()?;
        let held = self.scene_cache_usage()?;
        let owner: Arc<dyn std::any::Any + Send + Sync> = proof.clone();
        self.coordinator.retain_current_evidence(&stamp, owner.clone(), bytes, held)?;
        self.retain_topology_source(&stamp, &proof, &owner)
    }
    #[cfg(feature = "occt-brep")]
    fn retain_topology_source(&mut self, stamp: &om9_core::identity::SourceStamp,
        proof: &Arc<om9_kernel::brep::SharedTopologyProof>, owner: &Arc<dyn std::any::Any + Send + Sync>) -> Result<()> {
        let source = match self.coordinator.current_evidence_source(stamp) {
            Some(source) => proof.bind_captured_source(&self.coordinator, source)?.into_source(),
            None => proof.capture_source(&self.coordinator)?.into_source(),
        };
        self.coordinator.retain_source_with_evidence(stamp, owner, source)
    }
    #[cfg(feature = "occt-brep")]
    fn prune_topology_proofs(&mut self, document: &Arc<Document>) {
        if let Some(cache) = &self.topology_cache { cache.retain(document); }
    }
    fn publish_owned_scene(&mut self, scene: Scene, buffer_owner: Option<SceneBufferOwner>) {
        // SDK snap work is bound to the previous publication. Drop its
        // receivers now so cancellation can drain without another SDK call.
        #[cfg(feature = "occt-brep")]
        { *self.sdk_snap_workers.get_mut() = Default::default(); }
        #[cfg(feature = "occt-brep")]
        drop(self.dupedge_source_view_job.take());
        #[cfg(feature = "occt-brep")]
        drop(self.dupedge_scene_backup.take());
        #[cfg(feature = "occt-brep")]
        self.prune_topology_proofs(&scene.document);
        #[cfg(feature = "occt-brep")]
        drop(self.topology_remesh_job.take());
        if let Some(owner) = &buffer_owner {
            scene.bind_shared_buffer_owner(owner.lease.clone());
        }
        let old_scene = self.scene.replace(scene);
        drop(old_scene);
        let old_owner = std::mem::replace(&mut *self.scene_buffer_owner.lock().unwrap_or_else(|e| e.into_inner()), buffer_owner);
        drop(old_owner);
    }
    #[cfg(all(test, feature = "occt-brep"))]
    pub(crate) fn publish_paired_brep_for_test(&mut self, id: om9_core::identity::ObjectId, shape: &om9_core::brep::Brep) -> Result<()> {
        let stamp = self.document().stamp(id)?;
        let (buffers, _) = Scene::prepare_brep_buffers(shape, self.coordinator.limits(), |_, _| Ok(()))?;
        let scene = self.scene().ok_or(Error::Missing("test scene"))?.try_replace_root_brep_display(
            &stamp, shape, buffers, [0, 0], Arc::new(()), Arc::new(()),
        )?;
        self.publish_owned_scene(scene, None);
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn replace_scene_same_epoch_for_test(&mut self, offset_y: f64) -> Result<()> {
        let current = self.scene.as_ref().ok_or(Error::Missing("test scene"))?;
        let mut replacement = Scene::capture(Arc::clone(&current.document), current.epoch, self.coordinator.limits())?;
        for object in &mut replacement.objects {
            if let RenderBuffers::Curve { points, .. } = object.buffers.make_mut_unregistered()? {
                for point in points { point[1] += offset_y; }
            }
        }
        self.publish_owned_scene(replacement, None);
        Ok(())
    }
    #[cfg(all(test, feature = "occt-brep"))]
    pub(crate) fn insert_attested_sphere_for_test(&mut self) -> Result<()> {
        self.commit_primitive_optional_topology("OM9-SOLID-014", "sphere", "sphere",
            om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, om9_core::geometry::Transform::identity())
    }
    fn clear_scene(&mut self) {
        self.cancel_boolean_builder_worker();
        #[cfg(feature = "occt-brep")]
        drop(self.wireframe_worker.take());
        #[cfg(feature = "occt-brep")]
        drop(self.isocurve_build_worker.take());
        #[cfg(feature = "occt-brep")]
        drop(self.boolean_union_worker.take());
        #[cfg(feature = "occt-brep")]
        { drop(self.face_verification_job.take()); drop(self.face_verification_input.take()); drop(self.face_verification_receipt.take()); }
        #[cfg(feature = "occt-brep")]
        drop(self.dupedge_source_view_job.take());
        #[cfg(feature = "occt-brep")]
        drop(self.dupedge_scene_backup.take());
        #[cfg(feature = "occt-brep")]
        drop(self.topology_cache.take());
        #[cfg(feature = "occt-brep")]
        drop(self.remesh_representation_cache.take());
        #[cfg(feature = "occt-brep")]
        drop(self.topology_remesh_job.take());
        drop(self.scene.take());
        let old_owner = self.scene_buffer_owner.lock().unwrap_or_else(|e| e.into_inner()).take();
        drop(old_owner);
    }
    fn registered_scene_buffer_bytes(&self, scene: &Scene) -> u64 {
        let owner = self.scene_buffer_owner.lock().unwrap_or_else(|e| e.into_inner());
        owner.as_ref().filter(|owner| owner.epoch == scene.epoch
            && owner.document.ptr_eq(&Arc::downgrade(&scene.document)))
            .map_or(0, |owner| owner.bytes)
    }
    pub(crate) fn share_scene_buffer_owner(&self, scene: &Scene)
        -> Result<Arc<SceneBufferLease>> {
        self.share_scene_buffer_owner_with_sample(scene,
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)))
    }
    pub(crate) fn share_scene_buffer_owner_with_sample(&self, scene: &Scene, physical: Option<(u64, u64)>)
        -> Result<Arc<SceneBufferLease>> {
        if !self.scene.as_ref().is_some_and(|current| std::ptr::eq(current, scene)) {
            return Err(Error::Stale("viewport scene"));
        }
        let mut owner = self.scene_buffer_owner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = owner.as_ref() {
            if existing.epoch != scene.epoch || !existing.document.ptr_eq(&Arc::downgrade(&scene.document)) {
                return Err(Error::Invalid("scene buffer owner mismatch"));
            }
            if existing.bytes == scene.shared_buffer_bytes() {
                scene.bind_shared_buffer_owner(existing.lease.clone());
                return Ok(existing.lease.clone());
            }
        }
        let bytes = scene.shared_buffer_bytes();
        let already = owner.as_ref().map_or(0, |owner| owner.bytes);
        let new_bytes = bytes.checked_sub(already).ok_or(Error::Invalid("scene buffer accounting"))?;
        let overhead = scene_buffer_lease_overhead();
        let charge = new_bytes.checked_add(overhead).ok_or(Error::Overflow("scene buffer lease"))?;
        let mut held = scene.cache_held_usage()?;
        held.bytes = held.bytes.checked_sub(bytes).ok_or(Error::Invalid("scene buffer accounting"))?;
        held.add(self.auxiliary_usage()?, self.coordinator.limits())?;
        let reservation = self.coordinator.reserve_external_memory_with_allocated(
            om9_core::budget::memory::MemoryClass::CpuCache, charge, new_bytes, physical, held,
        )?;
        let previous = owner.as_ref().map(|owner| owner.lease.clone());
        let lease = SceneBufferLease::new(previous, reservation)?;
        scene.bind_shared_buffer_owner(lease.clone());
        *owner = Some(SceneBufferOwner { epoch: scene.epoch, document: Arc::downgrade(&scene.document), bytes, lease: lease.clone() });
        Ok(lease)
    }
    /// The ordered command events of this session, for text readers and audits.
    pub fn events(&self) -> &crate::command_events::EventLog {
        &self.events
    }
    /// The four workspace viewport cameras in stable title order, for headless rendering.
    pub fn view_cameras(
        &self,
    ) -> Vec<(
        &'static str,
        om9_scene::camera::Camera,
        om9_scene::camera::CPlane,
    )> {
        let mut views: Vec<_> = self
            .workspace
            .views
            .values()
            .map(|view| (view.id.title(), view.camera.clone(), view.cplane.clone()))
            .collect();
        views.sort_by_key(|(title, _, _)| *title);
        views
    }
    /// The active workspace view title, for headless input replay receipts.
    pub fn active_view_title(&self) -> &'static str {
        self.workspace
            .views
            .get(&self.workspace.active)
            .map(|view| view.id.title())
            .unwrap_or("")
    }
    /// Makes the named workspace view active, for headless input replay.
    pub fn activate_view(&mut self, title: &str) -> Result<()> {
        let id = self
            .workspace
            .views
            .values()
            .find(|view| view.id.title().eq_ignore_ascii_case(title))
            .map(|view| view.id)
            .ok_or(Error::Missing("workspace view"))?;
        self.workspace.active = id;
        Ok(())
    }
    /// The command an empty Enter would run again: the last completed command
    /// that may be repeated (OM9-L3-01-02-009). Only the interactive geometry
    /// commands are repeatable; a file or destructive command is never repeated
    /// silently, and never enters the repeat slot.
    pub fn repeatable_last(&self) -> Option<&str> {
        self.repeat
            .as_deref()
            .filter(|feature| Self::is_repeatable(feature))
    }
    /// Whether a command may be started again by an empty Enter. A command is
    /// named the way the registry names it: its stable feature id, its command
    /// id or any alias (OM9-L3-01-02-001), so `Line`, `line`, `Ln` and the
    /// feature id all answer the same way.
    pub fn is_repeatable(feature: &str) -> bool {
        let wanted = feature.trim();
        let line = Self::line_descriptor();
        line.id.eq_ignore_ascii_case(wanted)
            || line.feature.eq_ignore_ascii_case(wanted)
            || line
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(wanted))
            // POINT has no descriptor yet; its own item adds one.
            || "point".eq_ignore_ascii_case(wanted)
            || POINT.eq_ignore_ascii_case(wanted)
    }
    /// The repeat slot as stored, for the audit: it is either empty or a
    /// repeatable feature.

    /// Whether the running line command draws both sides of its start point.
    pub fn both_sides(&self) -> Option<bool> {
        self.session.as_ref().map(|session| session.both_sides)
    }

    /// Sets the BothSides option through the same parser path a typed assignment uses.
    pub fn set_both_sides(&mut self, value: bool) -> Result<()> {
        if self.phase() == CmdPhase::Idle {
            return Err(Error::Missing("active command"));
        }
        self.submit_geometry_text(&format!("BothSides={}", if value { "Yes" } else { "No" }))
    }

    /// The canonical feature id the command line should echo: the running command when one is active,
    /// otherwise the repeatable command an empty Enter would run.
    pub fn active_feature(&self) -> Option<&str> {
        self.session
            .as_ref()
            .map(|session| session.feature)
            .or_else(|| self.dupedge_pending.as_ref().map(|_| self::wireframe::DUP_EDGE))
            .or_else(|| self.solid_pending.as_ref().map(|pending| pending.feature))
            .or_else(|| self.measure_pending.as_ref().map(|pending| pending.feature))
            .or_else(|| {
                self.annotation_pending
                    .as_ref()
                    .map(|pending| pending.feature)
            })
            .or_else(|| {
                self.transform_pending
                    .as_ref()
                    .map(|pending| pending.feature)
            })
            .or_else(|| self.repeatable_last())
    }
    pub fn repeat_slot(&self) -> Option<&str> {
        self.repeat.as_deref()
    }
    pub fn recent(&self) -> &VecDeque<String> {
        &self.recent
    }
    pub fn is_running(&self) -> bool {
        if self.boolean_builder_worker.is_some() { return true; }
        if self.history_worker.is_some() { return true; }
        if self.join_history_worker.is_some() { return true; }
        #[cfg(feature="occt-brep")]
        if self.wireframe_worker.is_some() {return true;}
        #[cfg(feature="occt-brep")]
        if self.isocurve_face_verification_pending() {return true;}
        if self.dupedge_pending.as_ref().is_some_and(|context| self.validate_context(context).is_ok()) {
            return true;
        }
        if self.solid_pending.is_some()
            || self.circle_pending.is_some()
            || self.rect_pending.is_some()
            || self.polyline_pending.is_some()
            || self.interp_pending.is_some()
            || self.ellipse_pending.is_some()
            || self.arc_pending.is_some()
            || self.rebuild_pending.is_some()
            || self.fit_pending.is_some()
            || self.refit_srf_pending.is_some()
            || self.offset_curve_pending.is_some()
            || self.project_pending.is_some()
            || self.blend_pending.is_some()
            || self.isocurve_pending.is_some()
            || self.intersect_pending.is_some()
            || self.crv2view_pending.is_some()
            || self.through_points_pending.is_some()
            || self.polygon_pending.is_some()
            || self.helix_pending.is_some()
            || self.spiral_pending.is_some()
            || self.divide_pending.is_some()
            || self.to_lines_pending.is_some()
            || self.extend_curve_pending.is_some()
            || self.tween_curves_pending.is_some()
            || self.continue_interp_pending.is_some()
            || self.arc_blend_pending.is_some()
            || self.control_curve_pending.is_some()
            || self.continue_curve_pending.is_some()
            || self.angle_isocurve_pending.is_some()
            || self.corner_pending.is_some()
            || self.cross_section_pending.is_some()
            || self.crv_seam_pending.is_some()
            || self.curve_soft_edit_pending.is_some()
            || self.fillet_corners_pending.is_some()
            || self.offset_normal_pending.is_some()
            || self.offset_on_surface_pending.is_some()
            || self.pull_pending.is_some()
            || self.extrude_surface_pending.is_some()
            || self.heightfield_pending.is_some()
        {
            return true;
        }
        if self.top11_pending.is_some()
            || self.surface_pending.is_some()
            || self.curve_pair_pending.is_some()
            || self.surface_pair_pending.is_some()
            || self.seam_pending.is_some()
            || self.soft_edit_pending.is_some()
            || self.tween_pending.is_some()
            || self.extend_pending.is_some()
            || self.split_pending.is_some()
            || self.tangent_pending.is_some()
            || self.extrude_all_pending.is_some()
            || self.variable_offset_pending.is_some()
            || self.shrink_pending.is_some()
            || self.variable_blend_pending.is_some()
            || self.chamfer_pending.is_some()
            || self.fillet_pending.is_some()
            || self.variable_chamfer_pending.is_some()
            || self.history_options_pending.is_some()
            || self.slash_pending.is_some()
            || self.measure_pending.is_some()
            || self.annotation_pending.is_some()
            || self.transform_pending.is_some()
            || self.builder.is_some()
            || self.head_pending.is_some()
            || self.settings_pending.is_some()
            || self.gem_session.is_some()
            || self.gem_editor.is_some()
            || self.gem_placement.is_some()
            || self.gem_cluster.is_some()
            || self.gem_edits.is_some()
            || self.gem_pave.is_some()
            || self.gem_cutters.is_some()
            || self.row_cutters.is_some()
        {
            return true;
        }
        // A command whose document, view, revision or epoch is gone is NOT
        // running: closing or replacing the active document cancels it instead of
        // leaving a prompt that can only fail later (OM9-L3-01-02-003).
        match &self.session {
            None => false,
            Some(s) => self.validate_context(&s.context).is_ok(),
        }
    }
    /// Feature that started the current live command, without falling back to repeat.
    pub fn running_feature(&self) -> Option<&str> {
        self.is_running()
            .then_some(self.invoked_feature.as_deref())
            .flatten()
    }
    /// Drop a session whose context is gone. Returns true when one was dropped,
    /// so a caller (the window, the command line) can report the cancellation.
    pub fn cancel_if_stale(&mut self) -> bool {
        if self.dupedge_pending.as_ref().is_some_and(|context| self.validate_context(context).is_err()) {
            #[cfg(feature = "occt-brep")]
            drop(self.dupedge_source_view_job.take());
            #[cfg(feature = "occt-brep")]
            self.restore_dupedge_scene();
            self.dupedge_pending = None;
            self.clear_subselection();
            return true;
        }
        if self.session.is_some() && !self.is_running() {
            self.session = None;
            return true;
        }
        false
    }
    pub fn availability(feature: &str) -> Result<()> {
        if matches!(feature, "OM9-TOOLS-001" | "OM9-TOP11-011") { return Ok(()); }
        if self::gem::handles(feature) {
            return Ok(());
        }
        // The placement family (`OM9-L2-04-10`) is a normal command; its selection is
        // validated when the tool opens, so availability never depends on the current pick.
        if self::gem_placement::handles(feature)
            || self::gem_placement::command_name(feature).is_some()
        {
            return Ok(());
        }
        // The halo/cluster/baguette family (`OM9-L2-04-11`) validates its selection when it
        // opens, exactly like the placement family.
        if self::gem_cluster::handles(feature) || self::gem_cluster::command_name(feature).is_some()
        {
            return Ok(());
        }
        // The gem edit/report family (`OM9-L2-04-13`) validates its selection when it opens.
        if self::gem_edits::handles(feature) || self::gem_edits::command_name(feature).is_some() {
            return Ok(());
        }
        // The pavé/azure/prong family (`OM9-L2-04-12`) validates its selection when it opens.
        if self::gem_pave::PaveTool::handles(feature)
            || self::gem_pave::PaveTool::command_name(feature).is_some()
        {
            return Ok(());
        }
        // The cutter family (`OM9-L2-04-16`) validates its gem selection when it opens.
        if self::gem_cutters::CutterTool::handles(feature)
            || self::gem_cutters::CutterTool::command_name(feature).is_some()
        {
            return Ok(());
        }
        if self::channel_cutters::RowTool::handles(feature)
            || self::channel_cutters::RowTool::command_name(feature).is_some()
        {
            return Ok(());
        }
        if self::annotation::handles(feature) {
            return Ok(());
        }
        if self::metal::handles_command(feature) {
            return Ok(());
        }
        if self::head_commands::handles_command(feature) {
            return Ok(());
        }
        if self::settings_commands::handles_command(feature) {
            return Ok(());
        }
        if self::ring_tools::handles_command(feature) {
            return Ok(());
        }
        if crate::file_lifecycle::FileKind::from_feature(feature).is_some()
            || feature == self::stl::STL_EXPORT
        {
            return Ok(());
        }
        if let Some((target, _)) = solid::mode_command(feature) {
            return Self::availability(target);
        }
        if matches!(feature, "SolidBooleanSplit" | "SolidCreateSolid" | "SolidExtractSurfaceCopy" | "SolidPipeChain" | "SolidGVDMultiPipe" | "OM9-SOLID-021") {
            return if cfg!(feature = "occt-brep") { Ok(()) } else { Err(Error::Unsupported("Solid command requires OCCT")) };
        }
        if solid::handles(feature) {
            if !cfg!(feature = "occt-brep") { return Err(Error::Unsupported("Solid command requires OCCT")); }
            return crate::solid_menu::by_feature(feature)
                .filter(|binding| binding.enabled())
                .map(|_| ())
                .ok_or(Error::Unsupported(
                    crate::solid_menu::by_feature(feature)
                        .map_or("solid command is not catalogued", |binding| binding.reason),
                ));
        }
        if surface_edit::handles_command(feature) {
            return Ok(());
        }
        if self::transform::handles(feature) {
            return Ok(());
        }
        if let Some(reason) = crate::transform_menu::unavailable_reason(feature) {
            return Err(Error::Unsupported(reason));
        }
        match feature {
            LINE
            | POINT
            | self::circle::CIRCLE
            | self::rect::RECTANGLE
            | self::polyline::POLYLINE
            | self::interp::INTERP
            | self::ellipse::ELLIPSE
            | self::arc::ARC
            | self::arc::ARC_START
            | self::rebuild::REBUILD
            | self::fit::FIT
            | self::refit_srf::REFIT
            | self::offset::OFFSET
            | self::project::PROJECT
            | self::blend::BLEND
            | self::blend::BLEND_CRV
            | self::blend::MATCH
            | self::isocurve::ISOCURVE
            | self::pull::PULL
            | self::interp_on_surface::INTERP_ON_SURFACE
            | self::polyline_on_surface::POLYLINE_ON_SURFACE
            | self::sketch_on_surface::SKETCH_ON_SURFACE
            | self::intersect::INTERSECT
            | self::intersect_two_sets::INTERSECT_TWO_SETS
            | self::crv2view::CRV2VIEW
            | self::crv2view::CRV2VIEW_HISTORY
            | self::through_points::THROUGH_POINTS
            | self::polygon::POLYGON
            | self::helix::HELIX
            | self::spiral::SPIRAL
            | self::divide::DIVIDE
            | self::crv_ends::CRV_START
            | self::crv_ends::CRV_END
            | self::dup_border::DUP_BORDER
            | self::to_lines::TO_LINES
            | self::extend_curve::EXTEND
            | self::tween_curves::TWEEN_CURVES
            | self::continue_interp::CONTINUE_INTERP
            | self::arc_blend::ARC_BLEND
            | self::wireframe::DUP_EDGE
            | self::wireframe::EXTRACT_WIRE
            | self::section::SECTION
            | self::sketch::SKETCH
            | self::revolve::REVOLVE
            | self::curve_boolean::CURVE_BOOLEAN
            | self::cross_section::CROSS_SECTION
            | self::fillet_corners::FILLET_CORNERS
            | self::silhouette::SILHOUETTE
            | self::curve_soft_edit::CURVE_SOFT_EDIT
            | self::angle_isocurve::ANGLE_ISOCURVE
            | self::extrude_surface::EXTRUDE
            | self::heightfield::HEIGHTFIELD
            | self::mesh_curve::MESH_CURVE
            | self::uv_curves::CREATE_UV
            | self::apply_crv::APPLY_CRV
            | self::offset_on_surface::OFFSET_ON_SURFACE
            | self::offset_normal::OFFSET_NORMAL
            | self::corner_ops::FILLET
            | self::corner_ops::CHAMFER
            | self::crv_seam::CRV_SEAM
            | self::control_curve::CONTROL_CURVE
            | self::continue_curve::CONTINUE_CURVE
            | "MeshStress"
            | self::measure::DISTANCE
            | self::measure::ANGLE
            | self::measure::LENGTH
            | self::measure::RADIUS
            | self::measure::BOUNDING_BOX
            | self::measure::OBJECT_SUMMARY
            | self::measure::AREA
            | self::measure::VOLUME
            | ZOOM_EXTENTS
            | ZOOM_SELECTED
            | VIEW_HOME
            | VIEW_CENTER => Ok(()),
            _ => Err(Error::Unsupported(
                "command has no accepted standalone operation",
            )),
        }
    }
    /// The descriptor of the geometry commands this session runs: the option names
    /// the parser checks against (OM9-L3-01-02-005). The registry (OM9-L3-01-02-001)
    /// will own the catalogue; the session reads the same shape meanwhile.
    pub fn point_descriptor() -> om9_core::capability::Operation {
        om9_core::capability::Operation {
            id: "point".to_owned(),
            feature: POINT.to_owned(),
            aliases: vec!["Point".into(), "Pt".into()],
            prompt: "Location of point: ".into(),
            input_types: vec![om9_core::geometry::GeometryKind::Point],
            requires_input: true,
            output: vec![om9_core::geometry::GeometryKind::Point],
            options: Vec::new(),
            preconditions: om9_core::capability::Preconditions {
                needs_document: true,
                needs_selection: false,
                needs_active_view: true,
            },
            backend: om9_core::capability::Backend::Rust {
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            prepare: true,
            preview: true,
            commit: true,
            receipts: Vec::new(),
        }
    }
    pub fn line_descriptor() -> om9_core::capability::Operation {
        om9_core::capability::Operation {
            id: "line".to_owned(),
            feature: LINE.to_owned(),
            aliases: vec!["Line".into(), "Ln".into()],
            prompt: "Start of line (BothSides=No): ".into(),
            input_types: vec![
                om9_core::geometry::GeometryKind::Point,
                om9_core::geometry::GeometryKind::Point,
            ],
            requires_input: true,
            output: vec![om9_core::geometry::GeometryKind::Curve],
            options: vec!["BothSides".into(), "Length".into()],
            preconditions: om9_core::capability::Preconditions {
                needs_document: true,
                needs_selection: false,
                needs_active_view: true,
            },
            backend: om9_core::capability::Backend::Rust {
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            prepare: true,
            preview: true,
            commit: true,
            receipts: Vec::new(),
        }
    }
    pub fn dispatch(&mut self, feature: &str, _origin: InvocationOrigin) -> Result<()> {
        self.dispatch_effect(feature, _origin).map(|_| ())
    }
    pub fn dispatch_effect(
        &mut self,
        feature: &str,
        origin: InvocationOrigin,
    ) -> Result<DispatchOutcome> {
        if self.request_command_history(feature) {
            return Ok(DispatchOutcome::Command);
        }
        if !self::metal::handles_command(feature) { self.metal_panel_open = false; }
        #[cfg(feature = "occt-brep")]
        if self.viewport_detail_pending() { return Err(Error::Busy); }
        if self.history_worker.is_some() { return Err(Error::Busy); }
        if !self.is_running() {
            self.cancel_if_stale();
            self.invoked_feature = None;
        }
        self.command_run = self.command_run.wrapping_add(1);
        // A new command invalidates the previous read-out: the label shown in the
        // Command Line always belongs to the run the user just started
        // (OM9-L3-03-20-005).
        self.last_measurement = None;
        if let Some(result) = self.dispatch_history_feature(feature) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some((target, mode)) = solid::mode_command(feature) {
            if self.is_running() {
                return Err(Error::Busy);
            }
            Self::availability(target)?;
            self.dispatch_solid(target)?;
            if let Some(result) = self.submit_solid_text(&format!("Mode={mode}")) {
                result?;
            }
            return Ok(DispatchOutcome::Command);
        }
        // The Top 11 toolbar owns the same action ids: a published feature id
        // dispatches the action the icon click runs, not a second implementation.
        if let Some(action) = if feature == "OM9-TOOLS-001" { Some(om9_core::top11::Top11Action::RingRail) }
            else { om9_core::top11::Top11Action::from_feature(feature) } {
            if self.is_running() {
                return Err(Error::Busy);
            }
            let result = self.top11_invoke(action)?;
            self.invoked_feature = self.is_running().then(|| feature.to_owned());
            return Ok(DispatchOutcome::Top11(result));
        }
        if let Some(kind) = crate::file_lifecycle::FileKind::from_feature(feature) {
            return self.begin_file_operation(kind).map(DispatchOutcome::File);
        }
        // File ▸ Export Selected: the Controller validates the selection; the UI
        // opens the native `.stl` save dialog and calls `export_stl_selected`.
        if feature == self::stl::STL_EXPORT {
            if self.is_running() {
                return Err(Error::Busy);
            }
            self.stl_export_ready()?;
            return Ok(DispatchOutcome::StlExport);
        }
        let result = self.dispatch_geometry(feature, origin);
        if result.is_ok() {
            self.invoked_feature = self.is_running().then(|| feature.to_owned());
        }
        result.map(|_| DispatchOutcome::Command)
    }
    /// The session facts a command would start against right now.
    pub fn context(&self) -> CommandContext {
        CommandContext {
            document: self.document,
            view: self.workspace.active,
            selection: self.selection.iter().copied().collect(),
            revision: self.document().context_revision,
            epoch: self.coordinator.epoch(),
        }
    }
    /// Refuse a snapshot that no longer describes the session: another document,
    /// another active view, another revision or another publication epoch.
    pub fn validate_context(&self, context: &CommandContext) -> Result<()> {
        if context.document != self.document
            || !self.workspace.views.contains_key(&context.view)
            || context.revision != self.document().context_revision
            || context.epoch != self.coordinator.epoch()
        {
            return Err(Error::Stale("command document/view context"));
        }
        Ok(())
    }
    /// Which step of the running command is waiting for input. A stale session
    /// reads as Idle, exactly as `is_running` reports it.
    pub fn phase(&self) -> CmdPhase {
        if !self.is_running() {
            return CmdPhase::Idle;
        }
        if self.dupedge_pending.is_some() { return CmdPhase::AwaitEnd; }
        if self.top11_pending.is_some() {
            return CmdPhase::AwaitPoint;
        }
        if self.solid_pending.is_some()
            || self.circle_pending.is_some()
            || self.rect_pending.is_some()
            || self.polyline_pending.is_some()
        {
            return CmdPhase::AwaitPoint;
        }
        if self.surface_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.curve_pair_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.surface_pair_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        if self.seam_pending.is_some() {
            return CmdPhase::AwaitEnd;
        }
        match &self.session {
            None => CmdPhase::Idle,
            Some(session) if session.start.is_some() => CmdPhase::AwaitEnd,
            Some(_) => CmdPhase::AwaitPoint,
        }
    }
    pub fn set_cplane(&mut self, plane: CPlane) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        plane.validate()?;
        let view = self
            .workspace
            .views
            .get_mut(&self.workspace.active)
            .expect("active view");
        let revision = view
            .cplane
            .revision
            .checked_add(1)
            .ok_or(Error::Overflow("CPlane revision"))?;
        view.cplane = CPlane { revision, ..plane };
        Ok(())
    }
    /// Apply one of the world construction-plane presets to the active view.
    pub fn set_cplane_preset(&mut self, preset: om9_scene::camera::CPlanePreset) -> Result<()> {
        self.set_cplane(CPlane::preset(preset))
    }
    /// Build a plane from three points through the validated frame constructor, then
    /// assign it atomically. Degenerate/collinear points leave the current plane alone.
    pub fn set_cplane_from_three_points(
        &mut self,
        origin: Point3,
        x_point: Point3,
        y_point: Point3,
    ) -> Result<()> {
        let plane = CPlane::from_three_points(origin, x_point, y_point)?;
        self.set_cplane(plane)
    }
    /// Move and rotate the active plane without touching world geometry.
    pub fn set_cplane_origin_rotation(&mut self, origin: Point3, radians: f64) -> Result<()> {
        let plane = self.input_plane().with_origin(origin)?.rotated(radians)?;
        self.set_cplane(plane)
    }

    pub fn zoom_extents(&mut self, aspect: f64) -> Result<()> {
        let (center, radius) = self.world_bounds(false)?;
        let view = self
            .workspace
            .views
            .get_mut(&self.workspace.active)
            .ok_or(Error::Missing("navigation view"))?;
        view.camera.fit_bounds(center, radius, aspect, 1.1)
    }
    pub fn zoom_selected(&mut self, aspect: f64) -> Result<()> {
        if self.selection.is_empty() {
            return Err(Error::Missing("selection bounds"));
        }
        let (center, radius) = self.world_bounds(true)?;
        let view = self
            .workspace
            .views
            .get_mut(&self.workspace.active)
            .ok_or(Error::Missing("navigation view"))?;
        view.camera.fit_bounds(center, radius, aspect, 1.1)
    }
    /// Restore the active view's startup camera; CPlane, layers and document are untouched.
    pub fn view_home(&mut self) -> Result<()> {
        let id = self.workspace.active;
        let camera = Workspace::default_camera(id);
        let view = self
            .workspace
            .views
            .get_mut(&id)
            .ok_or(Error::Missing("navigation view"))?;
        view.camera = camera;
        Ok(())
    }
    /// Move the active view target to the world origin without changing rotation or zoom.
    pub fn view_center(&mut self) -> Result<()> {
        let view = self
            .workspace
            .views
            .get_mut(&self.workspace.active)
            .ok_or(Error::Missing("navigation view"))?;
        view.camera.center_at([0.; 3])
    }
    fn world_bounds(&self, selected: bool) -> Result<(Point3, f64)> {
        let scene = self
            .scene
            .as_ref()
            .ok_or(Error::Missing("navigation scene"))?;
        Self::scene_world_bounds(scene, selected.then_some(&self.selection))
    }

    pub(crate) fn scene_world_bounds(scene: &Scene, selection: Option<&std::collections::BTreeSet<om9_core::identity::ObjectId>>) -> Result<(Point3, f64)> {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let mut found = false;
        for object in &scene.objects {
            if !object.visible || selection.is_some_and(|selected| !selected.contains(&object.root)) {
                continue;
            }
            let transform = object.display_transform()?;
            let mut include = |point: Point3| -> Result<()> {
                let world = transform.point(point)?;
                for axis in 0..3 {
                    min[axis] = min[axis].min(world[axis]);
                    max[axis] = max[axis].max(world[axis]);
                }
                found = true;
                Ok(())
            };
            match object.buffers.as_ref() {
                RenderBuffers::Points { points, .. } | RenderBuffers::Curve { points, .. } => {
                    for point in points {
                        include(*point)?;
                    }
                }
                RenderBuffers::Mesh { vertices, .. } => {
                    for point in vertices {
                        include(*point)?;
                    }
                }
            }
        }
        if !found {
            return Err(Error::Missing("fit bounds"));
        }
        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        let diagonal = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
        let radius =
            (diagonal[0] * diagonal[0] + diagonal[1] * diagonal[1] + diagonal[2] * diagonal[2])
                .sqrt()
                * 0.5;
        // A single point has zero extent; keep the fit finite instead of dividing by zero.
        Ok((center, radius.max(0.001)))
    }
    pub fn set_ortho(&mut self, enabled: bool) {
        self.ortho = enabled;
    }
    pub fn set_planar(&mut self, enabled: bool) {
        self.planar = enabled;
    }
    pub fn set_project(&mut self, enabled: bool) {
        self.project = enabled;
    }
    pub fn grid_snap(&self) -> bool { self.grid_snap }
    pub(crate) fn annotation_style(&self) -> &om9_core::annotation::Style {
        &self.annotation_style
    }
    pub(crate) fn set_annotation_style(&mut self, style: om9_core::annotation::Style) -> Result<()> {
        style.validate()?;
        self.annotation_style = style;
        Ok(())
    }
    pub fn set_grid_snap(&mut self, enabled: bool) { self.grid_snap = enabled; }
    pub fn grid_snap_step_mm(&self) -> f64 { self.grid_snap_step_mm }
    pub fn set_grid_snap_step_mm(&mut self, step: f64) {
        if [0.1, 0.25, 0.5, 1.0].contains(&step) { self.grid_snap_step_mm = step; }
    }
    pub fn ortho(&self) -> bool {
        self.ortho
    }
    pub fn planar(&self) -> bool {
        self.planar
    }
    pub fn project(&self) -> bool {
        self.project
    }
    /// Previous accepted point in the active command, or the last accepted point.
    pub fn relative_base(&self) -> Option<om9_core::coords::WorldPoint> {
        self.session.as_ref().and_then(|session| session.start)
            .and_then(|point| om9_core::coords::WorldPoint::try_new(point).ok())
            .or(self.last_point)
    }
    pub fn point_constraint_base(&self) -> Option<om9_core::coords::WorldPoint> {
        if let Some(addon) = self.addon_session.as_ref() {
            return addon.last_point.and_then(|point| om9_core::coords::WorldPoint::try_new(point).ok());
        }
        let pending_point = if let Some(pending) = self.polyline_pending.as_ref() {
            Some(pending.points.last().copied())
        } else if let Some(pending) = self.rect_pending.as_ref() {
            Some(pending.points.last().copied())
        } else if let Some(pending) = self.circle_pending.as_ref() {
            Some(pending.points.last().copied())
        } else if let Some(pending) = self.solid_pending.as_ref() {
            Some(pending.points.last().copied())
        } else if let Some(pending) = self.measure_pending.as_ref() {
            Some(pending.points.last().copied())
        } else if let Some(pending) = self.transform_pending.as_ref() {
            Some(pending.points.last().copied())
        } else { None };
        if let Some(point) = pending_point {
            return point.and_then(|point| om9_core::coords::WorldPoint::try_new(point).ok());
        }
        if let Some(session) = self.session.as_ref() {
            return session.points.last().copied().or(session.start)
                .and_then(|point| om9_core::coords::WorldPoint::try_new(point).ok());
        }
        self.last_point
    }
    /// Insert the world origin through the ordinary typed point path (Matrix F4).
    /// Refuses outside a point prompt, so a shortcut can never create geometry from idle.
    pub fn submit_world_origin(&mut self) -> Result<()> {
        if self.phase() == CmdPhase::Idle {
            return Err(Error::Missing("point prompt"));
        }
        self.submit_geometry_text("W0,0,0")
    }
    /// Apply the active Ortho/Planar cursor constraints to a world-space pick.
    pub fn constrain_point(&self, point: Point3) -> Result<Point3> {
        self.constrain_point_with_shift(point, false)
    }
    /// Shift temporarily inverts Ortho for a pointer pick.
    pub fn constrain_point_with_shift(&self, point: Point3, shift: bool) -> Result<Point3> {
        self.constrain_point_with_base(point, shift, self.point_constraint_base())
    }
    pub fn constrain_point_with_base(&self, point: Point3, shift: bool, base: Option<om9_core::coords::WorldPoint>) -> Result<Point3> {
        finite(point)?;
        let plane = self.input_plane();
        let ortho = self.ortho ^ shift;
        if !ortho && !self.planar && !self.grid_snap {
            return Ok(point);
        }
        let world = om9_core::coords::WorldPoint::try_new(point)?;
        let local = <CPlane as om9_core::coords::LocalFrame>::to_local(&plane, world)?.xyz();
        let mut constrained = local;
        let ortho_base = ortho.then_some(base).flatten();
        let grid_coordinate = |coordinate: f64| {
            let steps = coordinate / self.grid_snap_step_mm;
            let nearest = if ortho_base.is_some() { (steps + 0.5).floor() } else { steps.round() };
            nearest * self.grid_snap_step_mm
        };
        if self.grid_snap {
            for coordinate in &mut constrained[..2] {
                *coordinate = grid_coordinate(*coordinate);
            }
        }
        if self.planar && let Some(base) = base {
            constrained[2] = <CPlane as om9_core::coords::LocalFrame>::to_local(&plane, base)?.xyz()[2];
        }
        if let Some(base) = ortho_base {
            let base = <CPlane as om9_core::coords::LocalFrame>::to_local(&plane, base)?.xyz();
            let dx = constrained[0] - base[0];
            let dy = constrained[1] - base[1];
            let octant = (dy.atan2(dx) / std::f64::consts::FRAC_PI_4).round() as i32;
            let diagonal = std::f64::consts::FRAC_1_SQRT_2;
            let (cos, sin) = match octant.rem_euclid(8) {
                0 => (1., 0.), 1 => (diagonal, diagonal), 2 => (0., 1.),
                3 => (-diagonal, diagonal), 4 => (-1., 0.), 5 => (-diagonal, -diagonal),
                6 => (0., -1.), _ => (diagonal, -diagonal),
            };
            // The grid aim selects the direction. Project the raw pick onto it,
            // then compare the nearest grid crossing on each moving axis.
            let distance = (local[0] - base[0]) * cos + (local[1] - base[1]) * sin;
            let direction = [cos, sin];
            let crossing = if self.grid_snap {
                direction.into_iter().enumerate().filter(|(_, component)| *component != 0.)
                    .map(|(axis, component)| {
                        let grid = grid_coordinate(base[axis] + distance * component);
                        (axis, grid, (grid - base[axis]) / component)
                    })
                    .min_by(|first, second| (first.2 - distance).abs().total_cmp(&(second.2 - distance).abs()))
            } else { None };
            let along = crossing.map_or(distance, |(_, _, along)| along);
            constrained[0] = base[0] + along * cos;
            constrained[1] = base[1] + along * sin;
            if let Some((axis, grid, _)) = crossing {
                constrained[axis] = grid;
            }
            constrained[2] = base[2];
        }
        plane.point(constrained)
    }
    /// Project an arbitrary world point onto the active CPlane's local Z=0.
    pub fn project_to_plane(&self, point: Point3) -> Result<Point3> {
        Self::project_point_onto(point, self.input_plane())
    }
    pub(crate) fn project_snap_with_base(&self, point: Point3, base: Option<om9_core::coords::WorldPoint>) -> Result<Point3> {
        Self::project_point_onto(point, self.pointer_input_plane_with_base(base))
    }
    fn project_point_onto(point: Point3, plane: CPlane) -> Result<Point3> {
        let world = om9_core::coords::WorldPoint::try_new(point)?;
        let local = <CPlane as om9_core::coords::LocalFrame>::to_local(&plane, world)?.xyz();
        plane.point([local[0], local[1], 0.])
    }
    pub fn cancel(&mut self) {
        self.ring_rail_replace = None;
        if self.boolean_builder_worker_pending() { self.cancel_boolean_builder_worker(); return; }
        if self.join_history_worker_pending() { self.cancel_join_history_worker(); return; }
        #[cfg(feature = "occt-brep")]
        self.cancel_metal_worker();
        self.metal_panel_open = false;
        if self.file_open_worker_pending() || self.file_save_worker_pending() {
            if let Some(token) = self.file_operation().map(|operation| operation.token()) {
                let _ = self.cancel_file_operation(token);
            }
            return;
        }
        if let Some(worker) = &self.history_worker {
            worker.cancel.store(true, std::sync::atomic::Ordering::Release);
            return;
        }
        // Some typed commands enter a prompt directly, without dispatch_effect.
        // Retire the old run here so a replacement cannot inherit its input.
        self.command_run = self.command_run.wrapping_add(1);
        #[cfg(feature = "occt-brep")]
        drop(self.wireframe_worker.take());
        #[cfg(feature = "occt-brep")]
        drop(self.isocurve_build_worker.take());
        #[cfg(feature = "occt-brep")]
        drop(self.boolean_union_worker.take());
        #[cfg(feature = "occt-brep")]
        { drop(self.face_verification_job.take()); drop(self.face_verification_input.take()); drop(self.face_verification_receipt.take()); }
        #[cfg(feature = "occt-brep")]
        drop(self.dupedge_source_view_job.take());
        #[cfg(feature = "occt-brep")]
        self.restore_dupedge_scene();
        if self.dupedge_pending.take().is_some() { self.clear_subselection(); }
        self.cancel_addon_builder();
        self.cancel_addon_session();
        if self.solid_face_needs_pick() || self.solid_pending.as_ref().is_some_and(|p| p.feature == "OM9-SOLID-029" || p.pipe_chain) || self.solid_fillet_needs_edge() || self.solid_fillet_preview().is_some() {
            self.clear_subselection();
        }
        self.builder_cancel();
        self.head_pending = None;
        self.settings_pending = None;
        self.gem_cancel();
        self.gem_placement_cancel();
        self.gem_cluster_cancel();
        self.gem_edits_cancel();
        self.gem_pave_cancel();
        self.gem_cutter_cancel();
        self.row_cutter_cancel();
        self.gem_editor = None;
        self.solid_pending = None;
        self.circle_pending = None;
        self.rect_pending = None;
        self.polyline_pending = None;
        self.interp_pending = None;
        self.ellipse_pending = None;
        self.arc_pending = None;
        self.rebuild_pending = None;
        self.fit_pending = None;
        self.refit_srf_pending = None;
        self.offset_curve_pending = None;
        self.project_pending = None;
        self.blend_pending = None;
        self.isocurve_pending = None;
        self.pull_pending = None;
        self.intersect_pending = None;
        self.crv2view_pending = None;
        self.crv_seam_pending = None;
        self.control_curve_pending = None;
        self.offset_on_surface_pending = None;
        self.offset_normal_pending = None;
        self.corner_pending = None;
        self.cross_section_pending = None;
        self.fillet_corners_pending = None;
        self.soft_edit_pending = None;
        self.curve_soft_edit_pending = None;
        self.angle_isocurve_pending = None;
        self.extrude_surface_pending = None;
        self.heightfield_pending = None;
        self.continue_curve_pending = None;
        self.through_points_pending = None;
        self.polygon_pending = None;
        self.helix_pending = None;
        self.spiral_pending = None;
        self.divide_pending = None;
        self.to_lines_pending = None;
        self.extend_curve_pending = None;
        self.tween_curves_pending = None;
        self.continue_interp_pending = None;
        self.arc_blend_pending = None;
        if let Some(pending) = self.top11_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.action().command(),
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        self.top11_grip = None;
        self.grip_selection.clear();
        if let Some(pending) = self.surface_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.curve_pair_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.surface_pair_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.seam_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.soft_edit_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.tween_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.extend_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.split_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.tangent_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.extrude_all_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.variable_offset_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.shrink_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.variable_blend_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.chamfer_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.fillet_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.variable_chamfer_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.history_options_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.slash_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.measure_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.annotation_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(pending) = self.transform_pending.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                pending.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
        if let Some(s) = self.session.take() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                s.feature,
                "cancelled",
                Some(crate::command_events::EventOutcome::Cancelled),
            );
        }
    }
    /// Record a refusal that happened inside a running command (a bad option, a
    /// degenerate point): the run continues, and the log says what was refused.
    pub fn warn(&mut self, detail: &str) {
        let Some(feature) = self.session.as_ref().map(|s| s.feature) else {
            return;
        };
        self.events.emit(
            crate::command_events::EventKind::Warn,
            feature,
            detail,
            None,
        );
    }
    pub fn submit(&mut self, text: &str) -> Result<()> {
        self.submit_effect(text).map(|_| ())
    }
    pub fn submit_effect(&mut self, text: &str) -> Result<DispatchOutcome> {
        if self.request_command_history(text) {
            return Ok(DispatchOutcome::Command);
        }
        self.input_revision = self.input_revision.wrapping_add(1);
        if let Some(result) = self.submit_ring_rail_confirmation(text) { return result.map(|_| DispatchOutcome::Command); }
        if let Some(result) = self.submit_ring_rail_shape(text) { return result.map(|_| DispatchOutcome::Command); }
        if self.boolean_builder_worker_pending() {
            if text.trim().is_empty() && self.boolean_builder_worker_preview() {
                self.promote_boolean_builder_preview()?;
                return Ok(DispatchOutcome::Command);
            }
            if text.trim().eq_ignore_ascii_case("cancel") || text.trim().eq_ignore_ascii_case("esc") {
                self.cancel_boolean_builder_worker();
                return Ok(DispatchOutcome::Command);
            }
            return Err(Error::Busy);
        }
        #[cfg(feature = "occt-brep")]
        if self.viewport_detail_pending() {
            if text.trim().eq_ignore_ascii_case("cancel") || text.trim().eq_ignore_ascii_case("esc") {
                self.cancel_viewport_detail(); return Ok(DispatchOutcome::Command);
            }
            return Err(Error::Busy);
        }
        if self.history_worker.is_some() {
            if text.trim().eq_ignore_ascii_case("cancel") || text.trim().eq_ignore_ascii_case("esc") {
                self.cancel(); return Ok(DispatchOutcome::Command);
            }
            return Err(Error::Busy);
        }
        if self.metal_panel_open && !self.is_running() && text.trim().eq_ignore_ascii_case("calculate") {
            return self.calculate_metal_panel().map(|_| DispatchOutcome::Command);
        }
        #[cfg(feature="occt-brep")]
        if self.wireframe_worker.is_some() {
            if text.trim().eq_ignore_ascii_case("cancel") || text.trim().eq_ignore_ascii_case("esc") {
                self.cancel();return Ok(DispatchOutcome::Command);
            }
            return Err(Error::Busy);
        }
        #[cfg(feature="occt-brep")]
        if self.isocurve_face_verification_pending() {
            if text.trim().eq_ignore_ascii_case("cancel") || text.trim().eq_ignore_ascii_case("esc") {
                self.cancel();return Ok(DispatchOutcome::Command);
            }
            return Err(Error::Busy);
        }
        if self.dupedge_pending.is_some() {
            if !self.is_running() { self.cancel_if_stale(); return Err(Error::Stale("DupEdge context")); }
            if !text.trim().is_empty() { return Err(Error::Invalid("select edges and press Enter")); }
            self.finish_dupedge_selection()?;
            return Ok(DispatchOutcome::Command);
        }
        if !self.is_running() {
            self.invoked_feature = None;
        }
        if text.len() > 4096 {
            return Err(Error::Budget("command input"));
        }
        {
            let text = text.trim();
            // An interactive add-on command reads the line first (points, options, Enter).
            if !self.is_running() && self.addon_session.is_some() && self.submit_session_text(text)? {
                return Ok(DispatchOutcome::Command);
            }
            // An open add-on builder reads its options first (`Size=2`, Enter = accept).
            if !self.is_running() && self.addon_builder.is_some() && self.submit_builder_text(text)? {
                return Ok(DispatchOutcome::Command);
            }
        }
        // An open builder owns the command line: a field, Apply, Reset or Cancel.
        if let Some(result) = self.builder_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open head/bezel builder owns the command line too (`OM9-L2-04-14`).
        if let Some(result) = self.submit_head_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open gem tool owns the command line: an option, Apply or Cancel.
        if let Some(result) = self.gem_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open placement tool owns the command line the same way (`OM9-L2-04-10`).
        if let Some(result) = self.gem_placement_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open halo/cluster/baguette builder owns the command line too (`OM9-L2-04-11`).
        if let Some(result) = self.gem_cluster_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open gem edit/report tool owns the command line too (`OM9-L2-04-13`).
        if let Some(result) = self.gem_edits_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open pavé/azure/prong builder owns the command line too (`OM9-L2-04-12`).
        if let Some(result) = self.gem_pave_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An open Gem Cutter / Library builder owns the command line too (`OM9-L2-04-16`).
        if let Some(result) = self.gem_cutter_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // The Channel / Micro Prong / Bright Cut builders own it too (`OM9-L2-04-17`).
        if let Some(result) = self.row_cutter_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // Matrix History switches/report live on the same command line (`-006`).
        if let Some(result) = self.submit_history_text(text) {
            return result.map(|_| DispatchOutcome::Command);
        }
        // Rhino's SelCrv, SelSrf, SelPolysrf, SelPt … (add objects of one type).
        if !self.is_running()
            && let Some(result) = self.select_kind_text(text)
        {
            return result.map(|_| DispatchOutcome::Command);
        }
        // An installed, enabled add-on takes the command name over from OM9's own command
        // (disable or remove the add-on to get the built-in back).
        if !self.is_running()
            && let Some(code) = self.addon_code(text.trim())
        {
            return self.run_addon_command(&code).map(|_| DispatchOutcome::Command);
        }
        if let Some(command) = self::metal::parse_command(text)? {
            #[cfg(feature = "occt-brep")]
            if self.background_metal && (command.all || command.material.is_some()) {
                self.dispatch_metal_tool("MetalWeights")?;
                self.set_metal_panel_object_materials(false);
                if command.all {
                    self.set_metal_panel_material(None)?;
                    self.set_metal_panel_filters(true, true);
                } else {
                    self.set_metal_panel_material(command.material.as_deref())?;
                }
                return self.calculate_metal_panel().map(|_| DispatchOutcome::Command);
            }
            if command.all {
                return self.calculate_metal_weights_catalog().map(|_| DispatchOutcome::Command);
            }
            if command.material.is_none() {
                self.dispatch_metal_tool("MetalWeights")?;
                if command.calculate { self.calculate_metal_panel()?; }
                return Ok(DispatchOutcome::Command);
            }
            return self
                .calculate_metal_weights(command.material.as_deref())
                .map(|_| DispatchOutcome::Command);
        }
        if let Some(material) = self::settings_commands::parse_mass_command(text)? {
            return self
                .calculate_settings_mass(material.as_deref())
                .map(|_| DispatchOutcome::Command);
        }
        if let Some(feature) = self::ring_tools::command_name(text) {
            return self
                .dispatch_ring_tool(feature)
                .map(|_| DispatchOutcome::Command);
        }
        // Typed placement commands (`gvGemOnCrv`, `gvGemFlow`, ...) reach the same handler as
        // the palette icons (`OM9-L2-04-10`).
        if let Some(feature) = self::gem_placement::command_name(text) {
            return self
                .dispatch_gem_placement(feature)
                .map(|_| DispatchOutcome::Command);
        }
        // Typed halo/cluster/baguette commands (`gvHaloBuilder`, `gvGemsBetween2Crvs`, ...)
        // reach the same handler as the palette icons (`OM9-L2-04-11`).
        if let Some(feature) = self::gem_cluster::command_name(text) {
            return self
                .dispatch_gem_cluster(feature)
                .map(|_| DispatchOutcome::Command);
        }
        // Typed pavé/azure/prong commands (`gvPaveDialog`, `gvPaveSphere`, ...).
        if let Some(feature) = self::gem_pave::PaveTool::command_name(text) {
            return self
                .dispatch_gem_pave(feature)
                .map(|_| DispatchOutcome::Command);
        }
        if let Some(feature) = self::gem::command_name(text) {
            return self.dispatch_gem(feature).map(|_| DispatchOutcome::Command);
        }
        // Typed cutter commands (`gvAzureCutter`, `gvGemCutterLibrary`) reach the same handler
        // as the palette icons (`OM9-L2-04-16`).
        if let Some(feature) = self::gem_cutters::CutterTool::command_name(text) {
            return self
                .dispatch_gem_cutter(feature)
                .map(|_| DispatchOutcome::Command);
        }
        if let Some(feature) = self::channel_cutters::RowTool::command_name(text) {
            return self
                .dispatch_row_cutter(feature)
                .map(|_| DispatchOutcome::Command);
        }
        // The three builder command names start their mode from the command line too.
        if let Some(kind) = self::builder::builder_command(text) {
            return self.begin_builder(kind).map(|_| DispatchOutcome::Command);
        }
        let canonical = self.canonical_option_assignment(text);
        let text = canonical.as_deref().unwrap_or(text);
        if text.trim().is_empty()
            && let Some(result) = self.top11_finish()
        {
            return result.map(DispatchOutcome::Top11);
        }
        if let Some(result) = self.submit_plugin_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(kind) = crate::file_lifecycle::FileKind::from_command(text.trim()) {
            return self.dispatch_effect(kind.feature_id(), InvocationOrigin::Cmd);
        }
        if let Some(feature) = Self::stl_export_feature(text) {
            return self.dispatch_effect(feature, InvocationOrigin::Cmd);
        }
        if let Some(result) = self.submit_circle_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_rect_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_polyline_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_interp_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_settings_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_ellipse_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_arc_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_rebuild_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_fit_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_refit_srf_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_offset_curve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_project_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_blend_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_isocurve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_pull_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_intersect_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_crv2view_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_through_points_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_crv_seam_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_control_curve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_continue_curve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_offset_on_surface_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_offset_normal_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_corner_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_curve_soft_edit_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_angle_isocurve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_extrude_surface_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_heightfield_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_cross_section_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_fillet_corners_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_polygon_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_helix_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_spiral_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_divide_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_to_lines_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_extend_curve_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_tween_curves_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_continue_interp_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_arc_blend_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.submit_solid_text(text.trim()) {
            return result.map(|_| DispatchOutcome::Command);
        }
        if let Some(result) = self.top11_submit(text.trim()) {
            result?;
            return Ok(DispatchOutcome::Command);
        }
        self.submit_geometry_text(text)
            .map(|_| DispatchOutcome::Command)
    }
    /// Insert one authored geometry item and publish it. Shared by the interactive point
    /// commands and the immediate selected-surface edit commands (`Unroll`, `Rebuild`).
    fn commit_geometry(
        &mut self,
        feature: &'static str,
        name: &'static str,
        geometry: Geometry,
    ) -> Result<()> {
        self.commit_geometries(feature, name, vec![(name, geometry)])
    }
    /// Insert one already-built entity, preserving its extensions.
    fn commit_entity(
        &mut self,
        feature: &'static str,
        name: &'static str,
        entity: Entity,
    ) -> Result<()> {
        self.commit_entity_inner(feature, name, entity, None)
    }
    #[cfg(feature = "occt-brep")]
    pub fn commit_entity_reserved_brep(&mut self, feature: &'static str, name: &'static str, entity: Entity,
        reservation: &mut om9_core::transaction::ExternalMemoryReservation) -> Result<()> {
        let result = self.commit_entity_inner(feature, name, entity, Some(reservation));
        if result.is_err() { self.coordinator.prune_orphaned_brep_representations(); }
        result
    }
    fn commit_entity_inner(&mut self, feature: &'static str, name: &'static str, mut entity: Entity,
        reservation: Option<&mut om9_core::transaction::ExternalMemoryReservation>) -> Result<()> {
        if entity.attributes.layer.is_some() {
            entity.attributes.color = None;
        }
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("command scene"));
        }
        let mut held = self.scene_cache_usage()?;
        let increment = old_scene.update_held_usage(Some(&entity))?;
        held.add(increment, self.coordinator.limits())?;
        let added = entity.id;
        let request = TransactionRequest::new(
                name,
                vec![Mutation::Insert {
                    document: self.document,
                    entity,
                }],
            );
        let prepared = match reservation {
            Some(owner) => self.coordinator.prepare_with_reserved_brep_insert(request, held, owner)?,
            None => self.coordinator.prepare_with_held_usage(request, held)?,
        };
        let receipt = self.commit_incremental_scene(prepared, held, increment, Some(added), false)?;
        if receipt.published {
            self.recent.push_back(feature.into());
            while self.recent.len() > 9 {
                self.recent.pop_front();
            }
            self.repeat = Some(feature.into());
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        let outcome = if receipt.published {
            crate::command_events::EventOutcome::Committed
        } else {
            crate::command_events::EventOutcome::Refused
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            feature,
            "finished",
            Some(outcome),
        );
        self.session = None;
        Ok(())
    }
    /// Insert one already-built entity plus its history recipe in one transaction.
    fn commit_entity_with_history(
        &mut self,
        feature: &'static str,
        name: &'static str,
        entity: Entity,
        command: &str,
        inputs: Vec<SourceBinding>,
    ) -> Result<()> {
        self.commit_entity_with_history_inner(feature, name, entity, command, inputs, None, Default::default(), self.history_record)
    }
    fn commit_brep_with_history(&mut self, feature: &'static str, name: &'static str,
        brep: om9_core::brep::Brep, command: &str, inputs: Vec<SourceBinding>) -> Result<()> {
        let held = self.scene_cache_usage()?;
        let bytes = brep.storage_usage(self.coordinator.limits())?.bytes;
        #[cfg(feature = "occt-brep")]
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        #[cfg(not(feature = "occt-brep"))]
        let physical = None;
        let mut reservation = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel, bytes, physical, held)?;
        reservation.set_allocated_bytes(bytes)?;
        let source = self.coordinator.register_reserved_brep_candidate(brep, &mut reservation, held)?;
        let entity = Entity::new(name, Geometry::Brep(source), self.document().current_layer);
        let result = self.commit_entity_with_history_inner(feature, name, entity, command, inputs, Some(&mut reservation), Default::default(), self.history_record);
        if result.is_err() { self.coordinator.prune_orphaned_brep_representations(); }
        result
    }
    fn commit_entity_with_history_inner(
        &mut self,
        feature: &'static str,
        name: &'static str,
        mut entity: Entity,
        command: &str,
        inputs: Vec<SourceBinding>,
        reservation: Option<&mut om9_core::transaction::ExternalMemoryReservation>,
        parameters: om9_core::request::Options,
        record: bool,
    ) -> Result<()> {
        if entity.attributes.layer.is_some() {
            entity.attributes.color = None;
        }
        let output = entity.id;
        let tolerance = self
            .document()
            .tolerance()
            .cloned()
            .unwrap_or_else(Tolerance::om9_template);
        let mut recipe = Recipe::new(
            self.document,
            command,
            inputs,
            vec![output],
            ReplayEnvironment {
                command_version: 1,
                solver: "om9-app".into(),
                solver_version: env!("CARGO_PKG_VERSION").into(),
                schema_version: 1,
                length_unit: Unit::Millimeter,
                frame: om9_core::geometry::Transform::identity(),
                tolerance,
            },
        );
        recipe.parameters = parameters;
        recipe.state = EvaluationState::Pending;
        recipe.policy.record = record;
        recipe.policy.update = self.history_update;
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("command scene"));
        }
        let mut held = self.scene_cache_usage()?;
        let increment = old_scene.update_held_usage(Some(&entity))?;
        held.add(increment, self.coordinator.limits())?;
        let expected_revision = self.document().revision;
        let mut mutations = vec![Mutation::Insert {
            document: self.document,
            entity,
        }];
        if record {
            mutations.push(Mutation::History {
                document: self.document,
                expected_revision,
                recipe: Box::new(recipe),
                published: vec![],
            });
        }
        let request = TransactionRequest::new(name, mutations);
        let prepared = match reservation {
            Some(owner) => self.coordinator.prepare_with_reserved_brep_mutations(request, held, std::slice::from_mut(owner))?,
            None => self.coordinator.prepare_with_held_usage(request, held)?,
        };
        let receipt = self.commit_incremental_scene(prepared, held, increment, Some(output), !record)?;
        if receipt.published || !record {
            self.recent.push_back(feature.into());
            while self.recent.len() > 9 {
                self.recent.pop_front();
            }
            self.repeat = Some(feature.into());
            if record {
                self.sync_published(om9_core::changes::ChangeCause::Edit);
            }
        }
        let outcome = if receipt.published || !record {
            crate::command_events::EventOutcome::Committed
        } else {
            crate::command_events::EventOutcome::Refused
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            feature,
            "finished",
            Some(outcome),
        );
        self.session = None;
        Ok(())
    }
    fn commit_geometries(
        &mut self,
        feature: &'static str,
        name: &'static str,
        items: Vec<(&'static str, Geometry)>,
    ) -> Result<()> {
        self.commit_placed_geometries(feature, name, items.into_iter().map(|(label, geometry)|
            (label, geometry, om9_core::geometry::Transform::identity())))
    }

    fn commit_placed_geometries<I>(&mut self, feature: &'static str, name: &'static str, items: I) -> Result<()>
    where I: ExactSizeIterator<Item = (&'static str, Geometry, om9_core::geometry::Transform)> {
        if items.len() == 0 {
            return Err(Error::Invalid("empty geometry batch"));
        }
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("command scene"));
        }
        let mut held = self.scene_cache_usage()?;
        let mut mutations = Vec::with_capacity(items.len());
        let mut added = None;
        let multiple = items.len() > 1;
        let mut increment = om9_core::budget::Usage::default();
        let mut scene_points = old_scene.cache_usage()?.points;
        for (label, geometry, placement) in items {
            let mut entity = Entity::new(label, geometry, self.document().current_layer);
            entity.placement = placement;
            if entity.attributes.layer.is_some() {
                entity.attributes.color = None;
            }
            if !multiple {
                increment = old_scene.update_held_usage_counted(Some(&entity), &mut scene_points)?;
                held.add(increment, self.coordinator.limits())?;
            }
            added = Some(entity.id);
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        let count = mutations.len();
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(name, mutations), held)?;
        if count > 1 {
            let receipt = self.commit_with_candidate_scene(prepared, held)?;
            if receipt.published {
                self.recent.push_back(feature.into());
                while self.recent.len() > 9 {
                    self.recent.pop_front();
                }
                self.repeat = Some(feature.into());
                self.sync_published(om9_core::changes::ChangeCause::Edit);
            }
            let outcome = if receipt.published {
                crate::command_events::EventOutcome::Committed
            } else {
                crate::command_events::EventOutcome::Refused
            };
            self.events.emit(
                crate::command_events::EventKind::Finished,
                feature,
                "finished",
                Some(outcome),
            );
            self.session = None;
            return Ok(());
        }
        let receipt = self.commit_incremental_scene(prepared, held, increment, added, false)?;
        if receipt.published {
            self.recent.push_back(feature.into());
            while self.recent.len() > 9 {
                self.recent.pop_front();
            }
            self.repeat = Some(feature.into());
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        let outcome = if receipt.published {
            crate::command_events::EventOutcome::Committed
        } else {
            crate::command_events::EventOutcome::Refused
        };
        self.events.emit(
            crate::command_events::EventKind::Finished,
            feature,
            "finished",
            Some(outcome),
        );
        self.session = None;
        Ok(())
    }
    #[cfg(test)]
    fn refresh_scene(&mut self) -> Result<()> {
        // Test fixtures can deliberately mutate the coordinator directly.
        self.clear_scene();
        let held = self.auxiliary_usage()?;
        let scene = Scene::capture_in_world_with_physical_samples(&self.coordinator, self.document, held, || {
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes))
        })?;
        let mut memory_held = held;
        memory_held.add(om9_core::budget::Usage { bytes: scene.shared_buffer_bytes(), ..Default::default() }, self.coordinator.limits())?;
        scene.reserve_memory_allowance(&self.coordinator, memory_held,
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)))?;
        self.publish_owned_scene(scene, None);
        Ok(())
    }
    /// Hold the incremental scene estimate on the shared account through
    /// construction and publication. Non-byte structural quotas remain in
    /// `held`; only the bytes move to the peer-visible reservation.
    fn commit_incremental_scene(
        &mut self,
        prepared: om9_core::transaction::PreparedTransaction,
        held: om9_core::budget::Usage,
        increment: om9_core::budget::Usage,
        addition: Option<om9_core::identity::ObjectId>,
        publish_unrecorded: bool,
    ) -> Result<om9_core::transaction::CommitReceipt> {
        let result = self.commit_incremental_scene_with_guard(prepared, held, increment, addition, publish_unrecorded,
            || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)), || Ok(()));
        if !result.as_ref().is_ok_and(|receipt| receipt.published) {
            self.coordinator.prune_orphaned_brep_representations();
            self.coordinator.prune_orphaned_native_representations();
        }
        result
    }

    fn commit_incremental_scene_with_guard(
        &mut self,
        mut prepared: om9_core::transaction::PreparedTransaction,
        mut held: om9_core::budget::Usage,
        increment: om9_core::budget::Usage,
        addition: Option<om9_core::identity::ObjectId>,
        publish_unrecorded: bool,
        mut sample: impl FnMut() -> Option<(u64, u64)>,
        before_build: impl FnOnce() -> Result<()>,
    ) -> Result<om9_core::transaction::CommitReceipt> {
        // An incremental update would clone the DupEdge-only display into a
        // normal scene. The user can cancel DupEdge before editing metadata.
        #[cfg(feature = "occt-brep")]
        if self.dupedge_scene_backup.is_some() {
            return Err(Error::Invalid("finish or cancel DupEdge before editing the document"));
        }
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if old_scene.epoch != self.coordinator.epoch() { return Err(Error::Stale("command scene")); }
        let document = self.coordinator.register_prepared_document(&prepared, self.document)?;
        let epoch = self.coordinator.epoch().checked_add(1).ok_or(Error::Overflow("scene publication epoch"))?;
        held.bytes = held.bytes.checked_sub(increment.bytes).ok_or(Error::Invalid("scene reservation estimate"))?;
        let mut reservation = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, increment.bytes.max(1), sample(), held,
        )?;
        self.coordinator.reserve_prepared_held_usage(&mut prepared, held)?;
        before_build()?;
        let (mut scene, actual_bytes) = old_scene.update_owned_with_retained_bytes(document, epoch, addition, self.coordinator.limits())?;
        let new_buffers = scene.shared_buffer_bytes().checked_sub(old_scene.shared_buffer_bytes())
            .ok_or(Error::Invalid("incremental scene buffers"))?;
        let overhead = if new_buffers == 0 { 0 } else { scene_buffer_lease_overhead() };
        let arena = scene.picking_arena_bytes()?;
        let required = actual_bytes.checked_add(overhead).and_then(|bytes| bytes.checked_add(arena))
            .ok_or(Error::Overflow("incremental scene owner"))?;
        if required > reservation.bytes() {
            let additional = required - reservation.bytes();
            self.coordinator.grow_external_memory_with_held(&mut reservation, additional, sample(), held)?;
        }
        scene.pin_ordinary_brep_displays(&self.coordinator, held, &mut sample)?;
        let previous = self.scene_buffer_owner.lock().unwrap_or_else(|e| e.into_inner()).as_ref()
            .map(|owner| (owner.bytes, owner.lease.clone()));
        scene.transfer_memory_allowance(&mut reservation, &self.coordinator, held)?;
        let buffer_owner = if new_buffers != 0 {
            let split = reservation.split_off(new_buffers + overhead)?;
            let bytes = previous.as_ref().map_or(0, |(bytes, _)| *bytes).checked_add(new_buffers)
                .ok_or(Error::Overflow("incremental scene owner"))?;
            Some(SceneBufferOwner { epoch, document: Arc::downgrade(&scene.document), bytes,
                lease: SceneBufferLease::new(previous.map(|(_, lease)| lease), split)? })
        } else {
            previous.map(|(bytes, lease)| SceneBufferOwner { epoch, document: Arc::downgrade(&scene.document), bytes, lease })
        };
        let candidate = ReservedCandidateScene { scene, buffer_owner, reservation };
        #[cfg(feature = "occt-brep")]
        let candidate = topology_remesh::prepare_missing_scene_wires(&self.coordinator, candidate, held)?;
        let receipt = self.coordinator.commit(prepared)?;
        let ReservedCandidateScene { scene, buffer_owner, reservation } = candidate;
        if receipt.published || publish_unrecorded { self.publish_owned_scene(scene, buffer_owner); }
        else { drop(scene); drop(buffer_owner); }
        drop(reservation);
        Ok(receipt)
    }

    /// Build the candidate scene before making an edit visible. `held` is the
    /// same caller-owned allocation admitted during transaction preparation.
    fn commit_with_candidate_scene(
        &mut self,
        prepared: om9_core::transaction::PreparedTransaction,
        held: om9_core::budget::Usage,
    ) -> Result<om9_core::transaction::CommitReceipt> {
        let result = self.commit_with_candidate_scene_guard(prepared, held, None,
            || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)), |_| Ok(()));
        if !result.as_ref().is_ok_and(|receipt| receipt.published) {
            self.coordinator.prune_orphaned_brep_representations();
            self.coordinator.prune_orphaned_native_representations();
        }
        result
    }

    fn commit_with_candidate_scene_guard(
        &mut self,
        mut prepared: om9_core::transaction::PreparedTransaction,
        held: om9_core::budget::Usage,
        wire: Option<PreparedEditWire>,
        sample: impl FnMut() -> Option<(u64, u64)>,
        before_build: impl FnOnce(&mut om9_core::transaction::ExternalMemoryReservation) -> Result<()>,
    ) -> Result<om9_core::transaction::CommitReceipt> {
        #[cfg(test)]
        let timing = prepared.candidate_world().documents.get(&self.document)
            .filter(|doc| doc.entities.len() >= 40).map(|_| std::time::Instant::now());
        let candidate = self.prepare_candidate_scene_guard(&mut prepared, held, wire, sample, before_build)?;
        #[cfg(test)]
        if let Some(start) = timing { eprintln!("candidate_scene_prepare_ms={}", start.elapsed().as_millis()); }
        let receipt = self.coordinator.commit(prepared)?;
        #[cfg(test)]
        if let Some(start) = timing { eprintln!("candidate_scene_and_commit_ms={}", start.elapsed().as_millis()); }
        let ReservedCandidateScene { scene, buffer_owner, reservation } = candidate;
        if receipt.published {
            self.publish_owned_scene(scene, buffer_owner);
        } else {
            drop(scene);
            drop(buffer_owner);
        }
        drop(reservation);
        Ok(receipt)
    }

    fn prepare_candidate_scene_guard(
        &self,
        prepared: &mut om9_core::transaction::PreparedTransaction,
        held: om9_core::budget::Usage,
        wire: Option<PreparedEditWire>,
        mut sample: impl FnMut() -> Option<(u64, u64)>,
        before_build: impl FnOnce(&mut om9_core::transaction::ExternalMemoryReservation) -> Result<()>,
    ) -> Result<ReservedCandidateScene> {
        let profile = std::env::var_os("OM9_COMMIT_PROFILE").map(|_| std::time::Instant::now());
        #[cfg(test)]
        let timing = prepared.candidate_world().documents.get(&self.document)
            .filter(|doc| doc.entities.len() >= 40).map(|_| std::time::Instant::now());
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("edit scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("edit scene"));
        }
        let document = self.coordinator.register_prepared_document(prepared, self.document)?;
        let epoch = self.coordinator.epoch().checked_add(1).ok_or(Error::Overflow("edit scene epoch"))?;
        let mut limits = self.coordinator.prepared_remaining_limits(prepared, held)?;
        // The candidate document is already charged by the prepared
        // transaction; Scene validates that same document again. Keep its
        // structural object quota instead of subtracting the document twice.
        limits.max_objects = self.coordinator.limits().max_objects;
        let scratch = Scene::candidate_preflight_scratch_reusing(&document, &limits, old_scene)?;
        let mut reservation = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, scratch.bytes.max(1), sample(), held,
        )?;
        let mut scene = Scene::capture_with_peak_reusing_brep_wires(document, epoch, &limits,
            &self.coordinator, old_scene, |peak| {
            #[cfg(test)]
            if let Some(start) = timing { eprintln!("candidate_scene_preflight_ms={}", start.elapsed().as_millis()); }
            if peak != 0 {
                self.coordinator.grow_external_memory_with_held(&mut reservation, peak, sample(), held)?;
            }
            before_build(&mut reservation)
        })?;
        if let Some(start) = profile { eprintln!("OM9_COMMIT_PROFILE scene_capture_ms={:.3}", start.elapsed().as_secs_f64()*1000.); }
        #[cfg(test)]
        if let Some(start) = timing { eprintln!("candidate_scene_capture_total_ms={}", start.elapsed().as_millis()); }
        let buffer_bytes = scene.shared_buffer_bytes();
        let overhead = if buffer_bytes == 0 { 0 } else { scene_buffer_lease_overhead() };
        let arena = scene.picking_arena_bytes()?;
        let actual_bytes = scene.cache_held_usage()?.bytes.checked_add(overhead).and_then(|bytes| bytes.checked_add(arena))
            .ok_or(Error::Overflow("candidate scene buffer owner"))?;
        if actual_bytes > reservation.bytes() {
            let additional = actual_bytes - reservation.bytes();
            self.coordinator.grow_external_memory_with_held(&mut reservation, additional, sample(), held)?;
        }
        scene.pin_ordinary_brep_displays(&self.coordinator, held, &mut sample)?;
        // The reservation already covers the candidate scene and must remain
        // visible to peers through commit; do not charge it again as `held`.
        self.coordinator.reserve_prepared_held_usage(prepared, held)?;
        scene.transfer_memory_allowance(&mut reservation, &self.coordinator, held)?;
        let buffer_owner = if buffer_bytes == 0 { None } else {
            let split = reservation.split_off(buffer_bytes + overhead)?;
            Some(SceneBufferOwner { epoch: scene.epoch, document: Arc::downgrade(&scene.document), bytes: buffer_bytes,
                lease: SceneBufferLease::new(None, split)? })
        };
        let candidate = ReservedCandidateScene { scene, buffer_owner, reservation };
        if let Some(start) = profile { eprintln!("OM9_COMMIT_PROFILE scene_before_wires_ms={:.3}", start.elapsed().as_secs_f64()*1000.); }
        #[cfg(feature = "occt-brep")]
        let candidate = match wire {
            Some(wire) => topology_remesh::apply_prepared_edit_wire(&self.coordinator, candidate, wire, held)?,
            None => candidate,
        };
        #[cfg(not(feature = "occt-brep"))]
        if wire.is_some() { return Err(Error::Unsupported("prepared OCCT wire buffers")); }
        #[cfg(feature = "occt-brep")]
        let candidate = topology_remesh::prepare_missing_scene_wires(&self.coordinator, candidate, held)?;
        if let Some(start) = profile { eprintln!("OM9_COMMIT_PROFILE scene_after_wires_ms={:.3}", start.elapsed().as_secs_f64()*1000.); }
        Ok(candidate)
    }
    /// Health entries for every object in the current document.
    pub fn health_entries(&self) -> Vec<crate::utilities::HealthEntry> {
        crate::utilities::health_entries(self.document(), self.coordinator.limits())
    }

    pub fn health_summary(&self) -> String {
        crate::utilities::summary(&self.health_entries())
    }

    /// Select every invalid object and return the count.
    pub fn select_bad_objects(&mut self) -> Result<usize> {
        let entries = self.health_entries();
        let ids = crate::utilities::bad_object_ids(&entries);
        self.select_objects(&ids)?;
        Ok(ids.len())
    }

    /// Select every object with naked edges and return the count.
    pub fn select_naked_edges(&mut self) -> Result<usize> {
        let entries = self.health_entries();
        let ids = crate::utilities::naked_edge_ids(&entries);
        self.select_objects(&ids)?;
        Ok(ids.len())
    }

    /// Center the selected objects on the world origin.
    pub fn center_selected(&mut self) -> Result<()> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("center selection"));
        }
        let held = self.scene_cache_usage()?;
        let bounds = measure::world_bounds_with_limits(self.document(), &ids, self.coordinator.limits(), Some((&self.coordinator, held)))?;
        let center = bounds.center();
        let delta =
            om9_core::geometry::Transform::translation([-center[0], -center[1], -center[2]]);
        let mut mutations = Vec::with_capacity(ids.len());
        for id in ids {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("center object"))?;
            mutations.push(Mutation::Placement {
                source: self.document().stamp(id)?,
                placement: delta.compose(entity.placement)?,
            });
        }
        if self.source_mutations_update_join_history(&mutations) {
            self.request_join_history_edit_worker(TransactionRequest::new("CenterObject", mutations))
        } else {
            self.commit_utility_mutations("CenterObject", mutations)
        }
    }

    /// Group the current selection.
    pub fn group_selection(&mut self) -> Result<()> {
        let ids = self.selected_ordered();
        if ids.len() < 2 {
            return Err(Error::Invalid("group selection"));
        }
        let group = om9_core::model::Group::new("Group", ids);
        let mutation = Mutation::GroupCreate {
            document: self.document,
            expected_revision: self.document().revision,
            group: Box::new(group),
        };
        self.commit_utility_mutations("Group", vec![mutation])
    }

    /// Remove every group that contains a selected object.
    pub fn ungroup_selection(&mut self) -> Result<()> {
        let ids = self.selected_ordered();
        if ids.is_empty() {
            return Err(Error::Missing("ungroup selection"));
        }
        let mut groups = std::collections::BTreeSet::new();
        for id in &ids {
            groups.extend(self.document().groups_of(*id));
        }
        if groups.is_empty() {
            return Err(Error::Invalid("selection is not grouped"));
        }
        let mut mutations = Vec::with_capacity(groups.len());
        for group in groups {
            mutations.push(Mutation::GroupDelete {
                document: self.document,
                expected_revision: self.document().revision,
                group,
            });
        }
        self.commit_utility_mutations("Ungroup", mutations)
    }

    /// Tessellate selected NURBS surfaces into editable display meshes.
    pub fn mesh_from_nurbs_selected(&mut self) -> Result<usize> {
        let ids = self.selected_ordered();
        let mut items = Vec::new();
        for id in &ids {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("mesh surface"))?;
            if let Geometry::Surface(surface) = &entity.geometry {
                let mesh = om9_core::utilities::mesh_from_surface(
                    surface,
                    16,
                    16,
                    self.coordinator.limits(),
                )?;
                items.push((surface, mesh));
            }
        }
        if items.is_empty() {
            return Err(Error::Invalid("no surface selection"));
        }
        let count = items.len();
        let mut geometries = Vec::with_capacity(count);
        for (_, mesh) in items {
            geometries.push(("Mesh From NURBS", Geometry::Mesh(mesh)));
        }
        self.commit_geometries("MeshFromNurbs", "Mesh From NURBS", geometries)?;
        Ok(count)
    }

    /// Reduce selected meshes to roughly half their face count.
    pub fn reduce_mesh_selected(&mut self) -> Result<usize> {
        self.reduce_mesh_selected_with_angle(30.0)
    }

    /// Reduce selected meshes with a caller-provided dihedral feature angle.
    pub fn reduce_mesh_selected_with_angle(&mut self, angle_deg: f64) -> Result<usize> {
        if !angle_deg.is_finite() || !(0.0..=180.0).contains(&angle_deg) {
            return Err(Error::Invalid("feature angle"));
        }
        let ids = self.selected_ordered();
        let mut mutations = Vec::new();
        let mut count = 0usize;
        for id in &ids {
            let entity = self
                .document()
                .entities
                .get(id)
                .ok_or(Error::Missing("mesh reduce"))?;
            let Geometry::Mesh(mesh) = &entity.geometry else {
                continue;
            };
            let target = (mesh.faces.len() / 2).max(1);
            let (reduced, _) =
                om9_core::mesh_repair::reduce_mesh_feature_aware(mesh, target, true, angle_deg)?;
            let mut extensions = entity.extensions.clone();
            if let Some(uv) =
                om9_core::rhino_semantics::mesh_uv_of(entity, self.coordinator.limits())?
                && uv.uvs.len() == mesh.vertices.len()
            {
                let mapped: Vec<[f64; 2]> = reduced
                    .vertices
                    .iter()
                    .map(|point| {
                        let mut best = 0usize;
                        let mut distance = f64::INFINITY;
                        for (index, old) in mesh.vertices.iter().enumerate() {
                            let dx = point[0] - old[0];
                            let dy = point[1] - old[1];
                            let dz = point[2] - old[2];
                            let candidate = dx * dx + dy * dy + dz * dz;
                            if candidate < distance {
                                distance = candidate;
                                best = index;
                            }
                        }
                        uv.uvs.get(best).copied().unwrap_or([0.0, 0.0])
                    })
                    .collect();
                let mut corner_map = vec![[0.0f64, 0.0]; mesh.vertices.len()];
                if uv.corners.len() == mesh.faces.iter().map(Vec::len).sum::<usize>() {
                    let mut offset = 0usize;
                    for face in &mesh.faces {
                        for index in face {
                            corner_map[*index as usize] = uv.corners[offset];
                            offset += 1;
                        }
                    }
                }
                let corners: Vec<[f64; 2]> = reduced
                    .faces
                    .iter()
                    .flat_map(|face| {
                        face.iter().map(|index| {
                            let point = reduced.vertices[*index as usize];
                            let mut best = 0usize;
                            let mut distance = f64::INFINITY;
                            for (old_index, old) in mesh.vertices.iter().enumerate() {
                                let dx = point[0] - old[0];
                                let dy = point[1] - old[1];
                                let dz = point[2] - old[2];
                                let candidate = dx * dx + dy * dy + dz * dz;
                                if candidate < distance {
                                    distance = candidate;
                                    best = old_index;
                                }
                            }
                            if uv.corners.is_empty() {
                                uv.uvs.get(best).copied().unwrap_or([0.0, 0.0])
                            } else {
                                corner_map[best]
                            }
                        })
                    })
                    .collect();
                let preserved = om9_core::rhino_semantics::MeshUv {
                    uvs: mapped,
                    corners,
                    material: uv.material,
                    pbr: uv.pbr,
                };
                extensions.insert(
                    om9_core::model::MESH_UV_EXTENSION_SLOT.to_string(),
                    preserved.to_extension()?,
                );
            }
            mutations.push(Mutation::Geometry {
                source: self.document().stamp(*id)?,
                geometry: Geometry::Mesh(reduced),
            });
            if extensions != entity.extensions {
                mutations.push(Mutation::Extensions {
                    source: self.document().stamp(*id)?,
                    extensions,
                });
            }
            count += 1;
        }
        if count == 0 {
            return Err(Error::Invalid("no mesh selection"));
        }
        self.commit_utility_mutations("MeshReduce", mutations)?;
        Ok(count)
    }

    fn commit_utility_mutations(
        &mut self,
        label: &'static str,
        mutations: Vec<Mutation>,
    ) -> Result<()> {
        let held = self.scene_cache_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(label, mutations), held)?;
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if receipt.published {
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }
    /// Start a private history-scene build. The current world and scene stay
    /// published until `poll_history_worker` accepts the completed candidate.
    pub fn request_history_worker(&mut self, redo: bool) -> Result<()> {
        use om9_core::budget::memory::MemoryClass;
        use std::sync::atomic::{AtomicBool, Ordering};
        if self.is_running() { return Err(Error::Busy); }
        #[cfg(feature = "occt-brep")]
        if self.face_verification_pending() || self.topology_remesh_pending() || self.dupedge_source_view_pending() {
            return Err(Error::Busy);
        }
        let scene = self.scene.as_ref().ok_or(Error::Missing("history scene"))?;
        let publication = scene.publication_id();
        let held = self.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes))
            .ok_or(Error::Unsupported("physical RAM availability is unavailable"))?;
        let bytes = held.bytes.checked_add(4096).ok_or(Error::Overflow("history worker owner"))?;
        let current_scene_reservation = self.coordinator.reserve_external_memory_with_allocated(
            MemoryClass::CpuCache, bytes, held.bytes, Some(physical), om9_core::budget::Usage::default())?;
        #[cfg(feature = "occt-brep")]
        let previous_scene = if scene.has_owned_brep_wires() {
            let attempt = (|| -> Result<Scene> {
            let mut snapshot_held = held;
            snapshot_held.bytes = 0;
            let estimate = scene.render_snapshot_metadata_estimate()?.bytes.checked_add(scene.picking_arena_bytes()?)
                .ok_or(Error::Overflow("history reuse snapshot"))?.max(1);
            let mut owner = self.coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache,
                estimate, Some(physical), snapshot_held)?;
            let snapshot = scene.try_render_snapshot()?;
            let actual = snapshot.managed_memory_allowance()?.max(1);
            if actual > owner.bytes() {
                self.coordinator.grow_external_memory_with_held(&mut owner, actual - estimate, Some(physical), snapshot_held)?;
            }
            snapshot.transfer_memory_allowance(&mut owner, &self.coordinator, snapshot_held)?;
            Ok(snapshot)
            })();
            match attempt {
                Ok(snapshot) => Some(snapshot),
                Err(Error::Budget(_)) => None,
                Err(error) => return Err(error),
            }
        } else { None };
        #[cfg(not(feature = "occt-brep"))]
        let previous_scene: Option<Scene> = None;
        let before = self.coordinator.capture_retained_document(self.document)?;
        let mut candidate = self.coordinator.prepare_history_for_worker(redo)?;
        #[cfg(feature = "occt-brep")]
        let rebuilds = self.prepare_history_rebuilds(&candidate, held)?;
        let peer = self.coordinator.memory_admission_peer_with_held(held)?;
        // The peer does not own the primary coordinator's Undo/world metadata.
        // Reserve the full primary metadata rather than subtracting two
        // snapshots that competing peers could change between observations.
        let metadata = self.coordinator.peer_unshared_metadata_bytes()?;
        let peer_metadata_reservation = if metadata == 0 { None } else {
            Some(self.coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache,
                metadata, metadata, Some(physical), om9_core::budget::Usage::default())?)
        };
        let token = self.coordinator.begin_history_worker(&candidate)?;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let document = self.document;
        let mut worker_held = held;
        worker_held.bytes = 0;
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let spawn = std::thread::Builder::new().name("om9-history-scene".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<_> {
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                candidate.validate_on(&peer, worker_held,
                    om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)))?;
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                #[cfg(feature = "occt-brep")]
                let wires = rebuilds.run(&peer, &mut candidate, worker_held, &worker_cancel)?;
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                let scene = Self::prepare_history_scene(&peer, candidate.candidate_world(), candidate.candidate_epoch(),
                    document, worker_held, om9_core::budget::Usage::default(), previous_scene.as_ref())?;
                #[cfg(feature = "occt-brep")]
                let scene = wires.apply(&peer, scene, worker_held, &worker_cancel)?;
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                Ok((candidate, scene))
            })).unwrap_or_else(|_| Err(Error::Io("history scene worker panicked".into())));
            let _ = sender.send(result);
        });
        if let Err(error) = spawn {
            self.coordinator.finish_history_worker(token)?;
            return Err(Error::Io(format!("history scene worker start: {error}")));
        }
        self.history_worker = Some(HistoryWorker { token, receiver, cancel, before,
            scene_publication: publication, selection_revision: self.selection_state().revision,
            sub_revision: self.sub_revision(), redo, peer_metadata_reservation,
            _current_scene_reservation: current_scene_reservation });
        Ok(())
    }

    pub fn history_worker_pending(&self) -> bool { self.history_worker.is_some() }

    pub fn poll_history_worker(&mut self) -> Option<Result<()>> {
        use std::sync::atomic::Ordering;
        let mut worker = self.history_worker.take()?;
        let result = match worker.receiver.try_recv() {
            Ok(result) => result,
            Err(std::sync::mpsc::TryRecvError::Empty) => { self.history_worker = Some(worker); return None; }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(Error::Io("history scene worker exited".into())),
        };
        let result = (|| {
            self.coordinator.finish_history_worker(worker.token)?;
            drop(worker.peer_metadata_reservation.take());
            if worker.cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
            if self.scene.as_ref().is_none_or(|scene| scene.publication_id() != worker.scene_publication)
                || self.selection_state().revision != worker.selection_revision || self.sub_revision() != worker.sub_revision {
                return Err(Error::Stale("history scene context"));
            }
            let (candidate, built) = result?;
            self.coordinator.publish_history(candidate)?;
            let ReservedCandidateScene { scene, buffer_owner, reservation } = built;
            self.publish_owned_scene(scene, buffer_owner);
            drop(reservation);
            let cause = if worker.redo { om9_core::changes::ChangeCause::Redo } else { om9_core::changes::ChangeCause::Undo };
            self.record_history_change(&worker.before, cause);
            self.sync_published(cause);
            Ok(())
        })();
        Some(result)
    }

    pub fn undo(&mut self) -> Result<()> {
        self.run_history_worker_blocking(false)
    }
    /// Re-run one recorded Sweep 1/2 History recipe and update its output.
    #[cfg(feature = "occt-brep")]
    pub fn replay_history(&mut self, recipe_id: om9_core::identity::HistoryId) -> Result<()> {
        use om9_core::surface_kernel;
        let recipe = self
            .document()
            .history
            .recipes
            .get(&recipe_id)
            .cloned()
            .ok_or(Error::Missing("history recipe"))?;
        if recipe.outputs.len() != 1 {
            return Err(Error::Invalid("history recipe outputs"));
        }
        let mut curves = Vec::with_capacity(recipe.inputs.len());
        for input in &recipe.inputs {
            let entity = self
                .document()
                .entities
                .get(&input.source.object)
                .ok_or(Error::Missing("history input"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("history curve input"));
            };
            curves.push(curve.clone());
        }
        enum ReplayGeometry { Curve(om9_core::geometry::Curve), Brep(om9_core::brep::Brep) }
        let rebuilt = if recipe.command == "Crv2ViewHistory" {
            if curves.len() != 2 {
                return Err(Error::Invalid("history recipe inputs"));
            }
            let curve = self::crv2view::build_from_two_views(
                &curves[0],
                &curves[1],
                self.coordinator.limits(),
            )?;
            ReplayGeometry::Curve(curve)
        } else if recipe.command == "Sweep4" {
            ReplayGeometry::Brep(self.build_sweep4(&curves)?)
        } else {
            let rail_count = match recipe.command.as_str() {
                "Sweep1History" => 1,
                "Sweep2History" => 2,
                _ => return Err(Error::Unsupported("history replay command")),
            };
            if curves.len() <= rail_count {
                return Err(Error::Invalid("history recipe inputs"));
            }
            let limits = self.coordinator.limits();
            let mut rails: Vec<Vec<Point3>> = Vec::with_capacity(rail_count);
            for curve in &curves[..rail_count] {
                rails.push(
                    surface_kernel::sample_curve(curve, 33, limits)
                        .map_err(|_| Error::Unsupported("history replay rail"))?,
                );
            }
            let mut profiles: Vec<Vec<Point3>> = Vec::with_capacity(curves.len() - rail_count);
            for curve in &curves[rail_count..] {
                profiles.push(
                    surface_kernel::sample_curve(curve, 24, limits)
                        .map_err(|_| Error::Unsupported("history replay profile"))?,
                );
            }
            ReplayGeometry::Brep(
                om9_kernel::creation::pipe_shell(
                    &rails[0],
                    rails.get(1).map(Vec::as_slice),
                    &profiles,
                    false,
                    om9_kernel::creation::DEFAULT_DEFLECTION_MM,
                    limits,
                )
                .map_err(|_| Error::Unsupported("history replay builder"))?,
            )
        };
        let output = recipe.outputs[0];
        let source = self.document().stamp(output)?;
        let held = self.scene_cache_usage()?;
        let (geometry, mut reservation) = match rebuilt {
            ReplayGeometry::Curve(curve) => (Geometry::Curve(curve), None),
            ReplayGeometry::Brep(candidate) => {
                let bytes = candidate.storage_usage(self.coordinator.limits())?.bytes;
                let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                let mut guard = self.coordinator.reserve_external_memory_with_held(
                    om9_core::budget::memory::MemoryClass::Kernel, bytes, physical, held)?;
                guard.set_allocated_bytes(bytes)?;
                let source = self.coordinator.register_reserved_brep_candidate(candidate, &mut guard, held)?;
                (Geometry::Brep(source), Some(guard))
            }
        };
        let request = TransactionRequest::new("ReplayHistory", vec![Mutation::Geometry { source, geometry }]);
        let prepared = match reservation.as_mut() {
            Some(guard) => self.coordinator.prepare_with_reserved_brep_mutations(request, held, std::slice::from_mut(guard))?,
            None => self.coordinator.prepare_with_held_usage(request, held)?,
        };
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if receipt.published {
            self.recent.push_back("ReplayHistory".into());
            while self.recent.len() > 9 {
                self.recent.pop_front();
            }
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }

    pub fn redo(&mut self) -> Result<()> {
        self.run_history_worker_blocking(true)
    }
    /// Synchronous callers wait for the same worker used by UI history requests.
    fn run_history_worker_blocking(&mut self, redo: bool) -> Result<()> {
        self.request_history_worker(redo)?;
        loop {
            if let Some(result) = self.poll_history_worker() { return result; }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
    fn prepare_history_scene(
        coordinator: &TransactionCoordinator,
        world: &om9_core::transaction::World,
        epoch: u64,
        id: DocumentId,
        held: om9_core::budget::Usage,
        staged: om9_core::budget::Usage,
        previous: Option<&Scene>,
    ) -> Result<ReservedCandidateScene> {
        let mut construction_held = held;
        construction_held.add(staged, coordinator.limits())?;
        let document = world.documents.get(&id).ok_or(Error::Missing("history scene document"))?.clone();
        let sample = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let (scene, mut reservation) = Scene::capture_retained_reserved_reusing(
            coordinator, document, epoch, construction_held, previous, sample,
        )?;
        // Preserve any additional unreserved staging supplied by the caller;
        // the prepared history map itself has its own continuous reservation.
        reservation.shrink_to(scene.cache_held_usage()?.bytes.max(1))?;
        if staged.bytes != 0 {
            coordinator.grow_external_memory_with_held(&mut reservation, staged.bytes, sample(), held)?;
        }
        let arena = scene.picking_arena_bytes()?;
        if arena != 0 { coordinator.grow_external_memory_with_held(&mut reservation, arena, sample(), held)?; }
        scene.transfer_memory_allowance(&mut reservation, coordinator, held)?;
        let buffer_bytes = scene.shared_buffer_bytes();
        let buffer_owner = if buffer_bytes == 0 { None } else {
            let overhead = scene_buffer_lease_overhead();
            coordinator.grow_external_memory_with_held(&mut reservation, overhead, sample(), held)?;
            let split = reservation.split_off(buffer_bytes.checked_add(overhead).ok_or(Error::Overflow("history scene owner"))?)?;
            Some(SceneBufferOwner { epoch: scene.epoch, document: Arc::downgrade(&scene.document), bytes: buffer_bytes,
                lease: SceneBufferLease::new(None, split)? })
        };
        Ok(ReservedCandidateScene { scene, buffer_owner, reservation })
    }
    /// Record a host Undo/Redo batch in the plugin change feed so a following plugin
    /// command sees the last committed document change, not only plugin commands.
    fn record_history_change(
        &mut self,
        before: &om9_core::model::Document,
        cause: om9_core::changes::ChangeCause,
    ) {
        if let Some(batch) = om9_core::changes::ChangeBatch::between(before, self.document(), cause)
            .ok()
            .flatten()
        {
            self.plugin_change = Some(batch);
        }
    }
    pub fn preview(&self, point: Point3) -> Result<Option<(Point3, Point3)>> {
        finite(point)?;
        let Some(s) = self.session.as_ref() else {
            return Ok(None);
        };
        self.validate_session()?;
        let Some(start) = s.start else {
            return Ok(None);
        };
        let start = if s.both_sides {
            std::array::from_fn(|i| 2. * start[i] - point[i])
        } else {
            start
        };
        finite(start)?;
        Ok(Some((start, point)))
    }
    pub fn input_plane(&self) -> CPlane {
        self.session
            .as_ref()
            .filter(|session| self.validate_context(&session.context).is_ok())
            .map_or_else(|| self.workspace.views[&self.workspace.active].cplane, |s| s.plane)
    }
    /// Plane used for free pointer intersection; Project applies to object snaps separately.
    pub fn pointer_input_plane(&self) -> CPlane {
        self.pointer_input_plane_with_base(self.point_constraint_base())
    }
    pub fn pointer_input_plane_with_base(&self, base: Option<om9_core::coords::WorldPoint>) -> CPlane {
        let plane = self.input_plane();
        if self.planar && let Some(base) = base {
            return plane.with_origin(base.xyz()).unwrap_or(plane);
        }
        plane
    }
}

/// Parses `MeshStress` / `MeshStress 2000` into a count.
fn parse_soft_value(text: &str) -> Result<f64> {
    let value: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("soft edit input"))?;
    if !value.is_finite() {
        return Err(Error::Invalid("soft edit input"));
    }
    Ok(value)
}
fn parse_soft_distance(text: &str) -> Result<f64> {
    let value = parse_soft_value(text)?;
    if value <= 0.0 {
        return Err(Error::Invalid("soft edit distance"));
    }
    Ok(value)
}
fn parse_soft_flag(text: &str) -> Result<bool> {
    match text.trim().to_ascii_lowercase().as_str() {
        "yes" | "y" | "true" | "on" => Ok(true),
        "no" | "n" | "false" | "off" => Ok(false),
        _ => Err(Error::Invalid("soft edit flag")),
    }
}
fn parse_shrink_value(text: &str) -> Result<f64> {
    let value: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("shrink domain"))?;
    if !value.is_finite() {
        return Err(Error::Invalid("shrink domain"));
    }
    Ok(value)
}
fn parse_offset_distance(text: &str) -> Result<f64> {
    let distance: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("variable offset distance"))?;
    if !distance.is_finite() {
        return Err(Error::Invalid("variable offset distance"));
    }
    Ok(distance)
}
fn parse_blend_scale(text: &str) -> Result<f64> {
    let scale: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("variable blend scale"))?;
    if !scale.is_finite() {
        return Err(Error::Invalid("variable blend scale"));
    }
    Ok(scale)
}
fn parse_chamfer_setback(text: &str) -> Result<f64> {
    let setback: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("chamfer setbacks"))?;
    if !setback.is_finite() || setback <= 0.0 {
        return Err(Error::Invalid("chamfer setbacks"));
    }
    Ok(setback)
}
fn parse_fillet_radius(text: &str) -> Result<f64> {
    let radius: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("fillet radius"))?;
    if !radius.is_finite() || radius <= 0.0 {
        return Err(Error::Invalid("fillet radius"));
    }
    Ok(radius)
}
fn parse_tangent_direction(text: &str) -> Result<[f64; 3]> {
    let values: Vec<f64> = text
        .split(',')
        .map(|value| value.trim().parse::<f64>())
        .collect::<std::result::Result<_, _>>()
        .map_err(|_| Error::Invalid("surface tangent direction"))?;
    if values.len() != 3 || values.iter().any(|value| !value.is_finite()) {
        return Err(Error::Invalid("surface tangent direction"));
    }
    Ok([values[0], values[1], values[2]])
}
fn parse_split_parameter(text: &str) -> Result<f64> {
    let parameter: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("surface split parameter"))?;
    if !parameter.is_finite() || parameter <= 0.0 || parameter >= 1.0 {
        return Err(Error::Invalid("surface split parameter"));
    }
    Ok(parameter)
}
fn parse_extend_distance(text: &str) -> Result<f64> {
    let distance: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("surface extend distance"))?;
    if !distance.is_finite() {
        return Err(Error::Invalid("surface extend distance"));
    }
    Ok(distance)
}
fn parse_seam_tolerance(text: &str) -> Result<f64> {
    let tolerance: f64 = text
        .trim()
        .parse()
        .map_err(|_| Error::Invalid("surface seam tolerance"))?;
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(Error::Invalid("surface seam tolerance"));
    }
    Ok(tolerance)
}
fn parse_mesh_stress(text: &str) -> Option<usize> {
    let mut words = text.split_whitespace();
    let command = words.next()?;
    if !command.eq_ignore_ascii_case("MeshStress") {
        return None;
    }
    let count = match words.next() {
        Some(value) => value.parse().ok()?,
        None => self::stress::DEFAULT_STRESS_MESHES,
    };
    Some(count)
}

/// Where the gumball sits and which axes it uses.
enum GumballPlacement {
    Origin(om9_core::geometry::Point3),
    Reset,
    World,
    CPlane,
    Object,
}

/// Resample a profile to `count` points by arc length.
fn resample_profile(points: &[Point3], count: usize) -> Vec<Point3> {
    if points.len() < 2 || count < 2 {
        return points.to_vec();
    }
    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0);
    let mut total = 0.0;
    for pair in points.windows(2) {
        let dx = pair[1][0] - pair[0][0];
        let dy = pair[1][1] - pair[0][1];
        let dz = pair[1][2] - pair[0][2];
        total += dx.hypot(dy).hypot(dz);
        lengths.push(total);
    }
    if total <= 1.0e-12 {
        return points.to_vec();
    }
    let mut result = Vec::with_capacity(count);
    let mut segment = 0usize;
    for index in 0..count {
        let target = total * index as f64 / (count - 1) as f64;
        while segment + 2 < lengths.len() && lengths[segment + 1] < target {
            segment += 1;
        }
        let span = lengths[segment + 1] - lengths[segment];
        let t = if span <= 1.0e-12 {
            0.0
        } else {
            (target - lengths[segment]) / span
        };
        result.push([
            points[segment][0] + (points[segment + 1][0] - points[segment][0]) * t,
            points[segment][1] + (points[segment + 1][1] - points[segment][1]) * t,
            points[segment][2] + (points[segment + 1][2] - points[segment][2]) * t,
        ]);
    }
    result
}

/// Build the Add Slash alignment section by interpolating the two nearest
/// profiles and mapping the result onto the two picked rail points.
fn slash_alignment_section(
    profiles: &[Vec<Point3>],
    rails: &[Vec<Point3>],
    a: Point3,
    b: Point3,
) -> Result<Vec<Point3>> {
    if profiles.is_empty() {
        return Err(Error::Invalid("slash needs a profile"));
    }
    let count = 32usize;
    let slash_param0 = rails
        .first()
        .and_then(|rail| nearest_rail_parameter(rail, a))
        .unwrap_or(0.0);
    let slash_param1 = rails
        .get(1)
        .and_then(|rail| nearest_rail_parameter(rail, b))
        .unwrap_or(1.0);
    let mut candidates: Vec<(f64, Vec<Point3>)> = profiles
        .iter()
        .map(|profile| {
            let resampled = resample_profile(profile, count);
            let first = resampled[0];
            let last = resampled[resampled.len() - 1];
            let forward = (first[0] - a[0]).hypot((first[1] - a[1]).hypot(first[2] - a[2]))
                + (last[0] - b[0]).hypot((last[1] - b[1]).hypot(last[2] - b[2]));
            let reverse = (last[0] - a[0]).hypot((last[1] - a[1]).hypot(last[2] - a[2]))
                + (first[0] - b[0]).hypot((first[1] - b[1]).hypot(first[2] - b[2]));
            let oriented = if reverse < forward {
                resampled.into_iter().rev().collect()
            } else {
                resampled
            };
            let score = profile_plane(&oriented)
                .and_then(|(origin, normal)| {
                    let first_rail = rails.first()?;
                    let second_rail = rails.get(1)?;
                    let (param0, _) = rail_plane_intersection(first_rail, origin, normal)?;
                    let (param1, _) = rail_plane_intersection(second_rail, origin, normal)?;
                    Some((param0 - slash_param0).powi(2) + (param1 - slash_param1).powi(2))
                })
                .unwrap_or(forward.min(reverse));
            (score, oriented)
        })
        .collect();
    candidates.sort_by(|left, right| left.0.total_cmp(&right.0));
    let first = candidates[0].1.clone();
    let second = if candidates.len() > 1 {
        candidates[1].1.clone()
    } else {
        first.clone()
    };
    let midpoint = |points: &[Point3]| {
        let last = points[points.len() - 1];
        [
            (points[0][0] + last[0]) / 2.0,
            (points[0][1] + last[1]) / 2.0,
            (points[0][2] + last[2]) / 2.0,
        ]
    };
    let m0 = midpoint(&first);
    let m1 = midpoint(&second);
    let slash = [
        (a[0] + b[0]) / 2.0,
        (a[1] + b[1]) / 2.0,
        (a[2] + b[2]) / 2.0,
    ];
    let span = [m1[0] - m0[0], m1[1] - m0[1], m1[2] - m0[2]];
    let span2 = span[0] * span[0] + span[1] * span[1] + span[2] * span[2];
    let t = if span2 <= 1.0e-12 {
        0.0
    } else {
        (((slash[0] - m0[0]) * span[0]
            + (slash[1] - m0[1]) * span[1]
            + (slash[2] - m0[2]) * span[2])
            / span2)
            .clamp(0.0, 1.0)
    };
    let mut shape: Vec<Point3> = first
        .iter()
        .zip(&second)
        .map(|(left, right)| {
            [
                left[0] + (right[0] - left[0]) * t,
                left[1] + (right[1] - left[1]) * t,
                left[2] + (right[2] - left[2]) * t,
            ]
        })
        .collect();
    let q0 = shape[0];
    let q1 = shape[shape.len() - 1];
    let from = [q1[0] - q0[0], q1[1] - q0[1], q1[2] - q0[2]];
    let to = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let from_len = from[0].hypot(from[1]).hypot(from[2]);
    let to_len = to[0].hypot(to[1]).hypot(to[2]);
    if from_len <= 1.0e-12 {
        return Ok(shape);
    }
    let scale = if to_len <= 1.0e-12 {
        1.0
    } else {
        to_len / from_len
    };
    let axis = [
        from[1] * to[2] - from[2] * to[1],
        from[2] * to[0] - from[0] * to[2],
        from[0] * to[1] - from[1] * to[0],
    ];
    let axis_len = axis[0].hypot(axis[1]).hypot(axis[2]);
    let cos = (from[0] * to[0] + from[1] * to[1] + from[2] * to[2]) / (from_len * to_len);
    let sin = axis_len / (from_len * to_len);
    let axis = if axis_len <= 1.0e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [axis[0] / axis_len, axis[1] / axis_len, axis[2] / axis_len]
    };
    for point in &mut shape {
        let v = [
            (point[0] - q0[0]) * scale,
            (point[1] - q0[1]) * scale,
            (point[2] - q0[2]) * scale,
        ];
        let cross = [
            axis[1] * v[2] - axis[2] * v[1],
            axis[2] * v[0] - axis[0] * v[2],
            axis[0] * v[1] - axis[1] * v[0],
        ];
        let dot = axis[0] * v[0] + axis[1] * v[1] + axis[2] * v[2];
        *point = [
            a[0] + v[0] * cos + cross[0] * sin + axis[0] * dot * (1.0 - cos),
            a[1] + v[1] * cos + cross[1] * sin + axis[1] * dot * (1.0 - cos),
            a[2] + v[2] * cos + cross[2] * sin + axis[2] * dot * (1.0 - cos),
        ];
    }
    Ok(shape)
}

/// Best-fit plane of a sampled profile: first non-collinear triple.
fn profile_plane(points: &[Point3]) -> Option<([f64; 3], [f64; 3])> {
    if points.len() < 3 {
        return None;
    }
    let origin = points[0];
    for i in 1..points.len() {
        let first = [
            points[i][0] - origin[0],
            points[i][1] - origin[1],
            points[i][2] - origin[2],
        ];
        for other in &points[(i + 1)..] {
            let second = [
                other[0] - origin[0],
                other[1] - origin[1],
                other[2] - origin[2],
            ];
            let normal = [
                first[1] * second[2] - first[2] * second[1],
                first[2] * second[0] - first[0] * second[2],
                first[0] * second[1] - first[1] * second[0],
            ];
            let length = normal[0].hypot(normal[1]).hypot(normal[2]);
            if length > 1.0e-9 {
                return Some((
                    origin,
                    [normal[0] / length, normal[1] / length, normal[2] / length],
                ));
            }
        }
    }
    None
}

fn nearest_rail_parameter(rail: &[Point3], point: Point3) -> Option<f64> {
    if rail.len() < 2 {
        return None;
    }
    let mut best = 0usize;
    let mut distance = f64::INFINITY;
    for (index, station) in rail.iter().enumerate() {
        let dx = station[0] - point[0];
        let dy = station[1] - point[1];
        let dz = station[2] - point[2];
        let value = dx * dx + dy * dy + dz * dz;
        if value < distance {
            distance = value;
            best = index;
        }
    }
    distance
        .is_finite()
        .then_some(best as f64 / (rail.len() - 1).max(1) as f64)
}
/// Intersect one sampled rail polyline with a plane; returns the normalized
/// rail parameter and the intersection point.
fn rail_plane_intersection(
    rail: &[Point3],
    origin: [f64; 3],
    normal: [f64; 3],
) -> Option<(f64, Point3)> {
    if rail.len() < 2 {
        return None;
    }
    let distance = |point: Point3| {
        (point[0] - origin[0]) * normal[0]
            + (point[1] - origin[1]) * normal[1]
            + (point[2] - origin[2]) * normal[2]
    };
    let mut segments: Vec<(usize, Point3, Point3)> = rail
        .windows(2)
        .enumerate()
        .map(|(index, pair)| (index, pair[0], pair[1]))
        .collect();
    if rail[0][0] != rail[rail.len() - 1][0]
        || rail[0][1] != rail[rail.len() - 1][1]
        || rail[0][2] != rail[rail.len() - 1][2]
    {
        // open rail; no wrap segment
    } else {
        segments.pop();
    }
    for (index, start, end) in segments {
        let d0 = distance(start);
        let d1 = distance(end);
        let span = d1 - d0;
        if d0 * d1 <= 0.0 && span.abs() > 1.0e-12 {
            let t = (d0 / -span).clamp(0.0, 1.0);
            let point = [
                start[0] + (end[0] - start[0]) * t,
                start[1] + (end[1] - start[1]) * t,
                start[2] + (end[2] - start[2]) * t,
            ];
            let parameter = (index as f64 + t) / (rail.len() - 1).max(1) as f64;
            return Some((parameter, point));
        }
    }
    None
}

/// Numerically intersect a line through `origin`/`point` with the exact NURBS
/// curve. Returns the curve fraction at the closest approach.
fn line_curve_parameter(
    curve: &Curve,
    origin: Point3,
    point: Point3,
    limits: &Limits,
) -> Result<f64> {
    let direction = [
        point[0] - origin[0],
        point[1] - origin[1],
        point[2] - origin[2],
    ];
    let length2 =
        direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
    if length2 <= 1.0e-18 {
        return Err(Error::Invalid("line curve direction"));
    }
    #[cfg(feature = "occt-brep")]
    if let Ok(fraction) = om9_kernel::intersect::line_curve_fraction(curve, origin, point, limits) {
        return Ok(fraction);
    }
    let distance = |fraction: f64| -> Result<f64> {
        let evaluated = om9_core::evaluation::evaluate_curve(
            curve,
            om9_core::evaluation::Parameter::Fraction(fraction.clamp(0.0, 1.0)),
            0,
            om9_core::geometry::Transform::identity(),
            limits,
        )?;
        let value = *evaluated
            .first()
            .ok_or(Error::Invalid("line curve point"))?;
        let offset = [
            value[0] - origin[0],
            value[1] - origin[1],
            value[2] - origin[2],
        ];
        let t = (offset[0] * direction[0] + offset[1] * direction[1] + offset[2] * direction[2])
            / length2;
        let projected = [
            origin[0] + direction[0] * t,
            origin[1] + direction[1] * t,
            origin[2] + direction[2] * t,
        ];
        Ok((value[0] - projected[0])
            .hypot(value[1] - projected[1])
            .hypot(value[2] - projected[2]))
    };
    let samples = 256usize;
    let mut best_fraction = 0.0;
    let mut best_distance = f64::INFINITY;
    for index in 0..=samples {
        let fraction = index as f64 / samples as f64;
        let value = distance(fraction)?;
        if value < best_distance {
            best_distance = value;
            best_fraction = fraction;
        }
    }
    let mut left = (best_fraction - 1.0 / samples as f64).max(0.0);
    let mut right = (best_fraction + 1.0 / samples as f64).min(1.0);
    for _ in 0..64 {
        let first = left + (right - left) / 3.0;
        let second = right - (right - left) / 3.0;
        if distance(first)? < distance(second)? {
            right = second;
        } else {
            left = first;
        }
    }
    let fraction = (left + right) / 2.0;
    if distance(fraction)? <= distance(best_fraction)? {
        Ok(fraction)
    } else {
        Ok(best_fraction)
    }
}

/// Intersect the line from the profile-plane origin to a rail point with the
/// sampled profile polyline; returns the normalized profile parameter.
fn line_profile_parameter(profile: &[Point3], origin: Point3, point: Point3) -> Option<f64> {
    if profile.len() < 2 {
        return None;
    }
    let e = [
        point[0] - origin[0],
        point[1] - origin[1],
        point[2] - origin[2],
    ];
    let c = e[0] * e[0] + e[1] * e[1] + e[2] * e[2];
    if c <= 1.0e-18 {
        return None;
    }
    let mut best: Option<(f64, f64, f64)> = None;
    for (index, pair) in profile.windows(2).enumerate() {
        let start = pair[0];
        let d = [
            pair[1][0] - start[0],
            pair[1][1] - start[1],
            pair[1][2] - start[2],
        ];
        let a = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if a <= 1.0e-18 {
            continue;
        }
        let w = [
            start[0] - origin[0],
            start[1] - origin[1],
            start[2] - origin[2],
        ];
        let b = d[0] * e[0] + d[1] * e[1] + d[2] * e[2];
        let d0 = d[0] * w[0] + d[1] * w[1] + d[2] * w[2];
        let e0 = e[0] * w[0] + e[1] * w[1] + e[2] * w[2];
        let denom = a * c - b * b;
        let (segment_t, line_t) = if denom.abs() > 1.0e-18 {
            let s = (b * e0 - c * d0) / denom;
            let t = (a * e0 - b * d0) / denom;
            (s.clamp(0.0, 1.0), t)
        } else {
            (0.0, e0 / c)
        };
        if !(0.0..=1.0).contains(&line_t) {
            continue;
        }
        let candidate = [
            start[0] + d[0] * segment_t,
            start[1] + d[1] * segment_t,
            start[2] + d[2] * segment_t,
        ];
        let line_point = [
            origin[0] + e[0] * line_t,
            origin[1] + e[1] * line_t,
            origin[2] + e[2] * line_t,
        ];
        let distance = (candidate[0] - line_point[0])
            .hypot(candidate[1] - line_point[1])
            .hypot(candidate[2] - line_point[2]);
        let parameter = (index as f64 + segment_t) / (profile.len() - 1) as f64;
        let point_distance = (candidate[0] - point[0])
            .hypot(candidate[1] - point[1])
            .hypot(candidate[2] - point[2]);
        if best.is_none_or(|(best_distance, best_point, _)| {
            distance < best_distance - 1.0e-12
                || ((distance - best_distance).abs() <= 1.0e-12 && point_distance < best_point)
        }) {
            best = Some((distance, point_distance, parameter));
        }
    }
    best.filter(|(distance, _, _)| *distance <= 1.0e-4)
        .map(|(_, _, parameter)| parameter)
}

fn profile_point_at(profile: &[Point3], parameter: f64) -> Point3 {
    let scaled = parameter.clamp(0.0, 1.0) * (profile.len() - 1) as f64;
    let index = (scaled.floor() as usize).min(profile.len() - 1);
    let next = (index + 1).min(profile.len() - 1);
    let t = scaled - index as f64;
    [
        profile[index][0] + (profile[next][0] - profile[index][0]) * t,
        profile[index][1] + (profile[next][1] - profile[index][1]) * t,
        profile[index][2] + (profile[next][2] - profile[index][2]) * t,
    ]
}

fn profile_segment(profile: &[Point3], start: f64, end: f64) -> Vec<Point3> {
    let mut result = vec![profile_point_at(profile, start)];
    let last = profile.len() - 1;
    if start <= end {
        let first = (start * last as f64).ceil() as usize;
        let stop = (end * last as f64).floor() as usize;
        for point in &profile[first..=stop.min(last)] {
            result.push(*point);
        }
    } else {
        let first = (start * last as f64).ceil() as usize;
        for point in &profile[first..=last] {
            result.push(*point);
        }
        let stop = (end * last as f64).floor() as usize;
        for point in &profile[..=stop] {
            result.push(*point);
        }
    }
    result.push(profile_point_at(profile, end));
    result.dedup_by(|left, right| {
        (left[0] - right[0])
            .hypot(left[1] - right[1])
            .hypot(left[2] - right[2])
            <= 1.0e-12
    });
    result
}
fn nearest_profile_index(profile: &[Point3], point: Point3) -> Option<usize> {
    let mut best = 0usize;
    let mut distance = f64::INFINITY;
    for (index, candidate) in profile.iter().enumerate() {
        let dx = candidate[0] - point[0];
        let dy = candidate[1] - point[1];
        let dz = candidate[2] - point[2];
        let value = dx * dx + dy * dy + dz * dz;
        if value < distance {
            distance = value;
            best = index;
        }
    }
    distance.is_finite().then_some(best)
}

/// Profile/rail intersection indices and rail parameters (original rail order).
fn rail_intersections(
    curve: &Curve,
    profile: &[Point3],
    rails: &[Vec<Point3>],
    limits: &Limits,
) -> Option<([f64; 4], [f64; 4])> {
    let (origin, normal) = profile_plane(profile)?;
    let mut profile_params = [0.0f64; 4];
    let mut rail_params = [0.0f64; 4];
    for (index, rail) in rails.iter().enumerate().take(4) {
        let (parameter, point) = rail_plane_intersection(rail, origin, normal)?;
        profile_params[index] = line_curve_parameter(curve, origin, point, limits)
            .ok()
            .or_else(|| line_profile_parameter(profile, origin, point))
            .unwrap_or_else(|| {
                nearest_profile_index(profile, point)
                    .map(|index| index as f64 / (profile.len() - 1).max(1) as f64)
                    .unwrap_or(0.0)
            });
        rail_params[index] = parameter;
    }
    Some((profile_params, rail_params))
}
fn parse_gumball_placement(text: &str) -> Option<GumballPlacement> {
    let mut words = text.split_whitespace();
    let command = words.next()?.to_ascii_lowercase();
    let command = match command.as_str() {
        "gumball" => format!("gumball{}", words.next()?.to_ascii_lowercase()),
        other => other.to_owned(),
    };
    match command.as_str() {
        "gumballreset" => Some(GumballPlacement::Reset),
        "gumballworld" => Some(GumballPlacement::World),
        "gumballobject" => Some(GumballPlacement::Object),
        "gumballcplane" => Some(GumballPlacement::CPlane),
        "gumballorigin" => {
            let argument = words.next()?;
            let mut parts = argument.split(',');
            let x = parts.next()?.parse().ok()?;
            let y = parts.next()?.parse().ok()?;
            let z = parts.next()?.parse().ok()?;
            if parts.next().is_some() {
                return None;
            }
            Some(GumballPlacement::Origin([x, y, z]))
        }
        _ => None,
    }
}

impl Controller {
    /// Applies a transient transform to the selected objects for the duration of a drag.
    ///
    /// The document is untouched: only `SceneObject::preview` changes, which every viewport reads through
    /// `display_transform`, so all four views show the object moving. The revision lets the GPU viewports know
    /// their cached image is stale.
    pub fn preview_gumball(&mut self, delta: om9_core::geometry::Transform) -> Result<()> {
        delta.validate()?;
        if self.grips_selected() {
            // Grips move, not whole objects: the viewport previews the edited grips.
            self.grip_preview = Some(delta);
            self.preview_revision = self.preview_revision.saturating_add(1);
            return Ok(());
        }
        let Some(scene) = self.scene.as_mut() else {
            return Err(Error::Missing("preview scene"));
        };
        let selection = &self.selection;
        let mut changed = false;
        for object in scene.objects.iter_mut() {
            let preview = if selection.contains(&object.root) && object.visible && object.editable {
                delta
            } else {
                om9_core::geometry::Transform::identity()
            };
            changed |= object.preview != preview;
            object.preview = preview;
        }
        if changed {
            scene.invalidate_picking_bounds();
            self.preview_revision = self.preview_revision.saturating_add(1);
        }
        Ok(())
    }

    /// Drops the transient transform, leaving the document as it was before the drag.
    pub fn clear_gumball_preview(&mut self) {
        self.grip_preview = None;
        if let Some(scene) = self.scene.as_mut() {
            let mut changed = false;
            for object in scene.objects.iter_mut() {
                changed |= object.preview != om9_core::geometry::Transform::identity();
                object.preview = om9_core::geometry::Transform::identity();
            }
            if changed {
                scene.invalidate_picking_bounds();
                self.preview_revision = self.preview_revision.saturating_add(1);
            }
        }
    }

    /// Revision of the transient transform, for viewport caches.
    pub fn preview_revision(&self) -> u64 {
        #[cfg(feature = "occt-brep")]
        if let Some((_, _, id)) = self.boolean_mesh_preview() {
            return self.preview_revision.wrapping_add(id.as_u128() as u64);
        }
        self.preview_revision
    }
}

#[cfg(test)]
mod surface_command_tests {
    use super::*;
    use om9_core::surface_kernel;

    fn surface_controller() -> (Controller, om9_core::identity::ObjectId) {
        let limits = Limits::default();
        let surface = surface_kernel::plane(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            10.0,
            10.0,
            &limits,
        )
        .unwrap();
        let mut document = Document::new("surface command");
        let entity = Entity::new("plane", Geometry::Surface(surface), None);
        let source = entity.id;
        document.entities.insert(source, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.selection.insert(source);
        (controller, source)
    }

    #[test]
    fn rebuild_and_unroll_commit_one_surface_and_one_undo_entry() {
        let (mut controller, source) = surface_controller();
        controller
            .dispatch("OM9-SURFACE-006", InvocationOrigin::Menu)
            .expect("rebuild");
        assert_eq!(controller.document().entities.len(), 2);
        assert_eq!(controller.coordinator().undo_len(), 1);
        controller.undo().unwrap();
        assert_eq!(controller.document().entities.len(), 1);

        controller
            .dispatch("OM9-SURFACE-134", InvocationOrigin::Menu)
            .expect("unroll");
        assert_eq!(controller.document().entities.len(), 2);
        let added = controller
            .document()
            .entities
            .values()
            .find(|entity| entity.id != source)
            .expect("surface");
        assert!(matches!(added.geometry, Geometry::Surface(_)));
        assert_eq!(controller.coordinator().undo_len(), 1);
    }
}
