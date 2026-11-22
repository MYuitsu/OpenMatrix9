// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::camera::{Camera, Viewport};
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    evaluation::{CurveEvaluator, Parameter},
    geometry::{Curve, Geometry, Point3, Transform},
    identity::*,
    model::{Document, Entity, Permissions},
    transaction::TransactionCoordinator,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock, Mutex, Weak},
};
use om9_core::brep::BrepRepresentationStore;
mod admission;
mod picking_cache;
mod trim_decomposition;
use picking_cache::{SceneMemory, SceneMemoryOwner};
pub use picking_cache::{ObjectSpheres, PickingIndex};

fn display_vector<T>(count: usize, reason: &'static str) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| Error::Budget(reason))?;
    Ok(values)
}

#[cfg(test)]
mod allocation_tests {
    #[test]
    fn history_candidate_scene_builds_on_accounted_peer_before_publication() {
        use om9_core::{brep::{Brep, BrepDisplay, BrepTopology}, budget::{Limits, Usage}, geometry::{Geometry, Transform}, model::{Document, Entity},
            transaction::{Mutation, TransactionCoordinator, TransactionRequest, World}};
        let limits = Limits::default();
        let mut document = Document::new("history worker scene");
        let entity = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        let id = entity.id; let doc = document.id;
        document.entities.insert(id, entity);
        let shape = Brep::from_native("CASCADE Topology fixture".into(), BrepDisplay {
            source_sha256: [0; 32], deflection_mm: 0.1,
            points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], triangles: vec![[0, 1, 2, 0]],
            segments: vec![], vertices: vec![], topology: BrepTopology {
                faces: vec![uuid::Uuid::new_v4()], edges: vec![], vertex_ids: vec![], face_edges: vec![],
            }.into(),
        }, &limits).unwrap();
        let brep = Entity::new("BRep", Geometry::Brep(shape), None);
        let brep_id = brep.id;
        document.entities.insert(brep_id, brep);
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let source = coordinator.world().documents[&doc].stamp(id).unwrap();
        let edit = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
            source, placement: Transform::translation([2., 0., 0.]),
        }])).unwrap();
        coordinator.commit(edit).unwrap();
        let before = coordinator.snapshot();
        let refused = coordinator.prepare_history(false).unwrap();
        let peer = coordinator.memory_admission_peer().unwrap();
        let refused = std::thread::spawn(move || {
            let document = refused.candidate_world().documents[&doc].clone();
            super::Scene::capture_retained_reserved_with_physical_samples(
                &peer, document, refused.candidate_epoch(), Usage::default(), || Some((8_u64 * 1024 * 1024 * 1024, 0)))
        }).join().unwrap();
        assert!(matches!(refused, Err(om9_core::Error::Budget(_))));
        assert_eq!(coordinator.snapshot(), before);
        let candidate = coordinator.prepare_history(false).unwrap();
        let peer = coordinator.memory_admission_peer().unwrap();
        let caller = std::thread::current().id();
        let (candidate, scene, reservation, worker) = std::thread::spawn(move || {
            let document = candidate.candidate_world().documents[&doc].clone();
            let (scene, reservation) = super::Scene::capture_retained_reserved_with_physical_samples(
                &peer, document, candidate.candidate_epoch(), Usage::default(), || None).unwrap();
            (candidate, scene, reservation, std::thread::current().id())
        }).join().unwrap();
        assert_ne!(caller, worker);
        assert_eq!(coordinator.snapshot(), before);
        assert_eq!(scene.objects.iter().find(|object| object.root == id).unwrap().transform, Transform::identity());
        let stamp = scene.document.stamp(brep_id).unwrap();
        assert!(scene.root_brep_display_pin(&stamp).unwrap().is_some());
        let foreign = TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap();
        assert!(matches!(super::Scene::capture_retained_reserved_with_physical_samples(
            &foreign, scene.document.clone(), candidate.candidate_epoch(), Usage::default(), || None),
            Err(om9_core::Error::Invalid("scene document is not retained"))));
        coordinator.publish_history(candidate).unwrap();
        assert!(std::sync::Arc::ptr_eq(&scene.document, &coordinator.world().documents[&doc]));
        drop(scene); drop(reservation);
        coordinator.redo().unwrap();
        let candidate = coordinator.prepare_history(false).unwrap();
        let peer = coordinator.memory_admission_peer().unwrap();
        assert!(coordinator.discard_oldest_undo().unwrap());
        let after_discard = coordinator.snapshot();
        let (candidate, scene, reservation) = std::thread::spawn(move || {
            let document = candidate.candidate_world().documents[&doc].clone();
            let (scene, reservation) = super::Scene::capture_retained_reserved_with_physical_samples(
                &peer, document, candidate.candidate_epoch(), Usage::default(), || None).unwrap();
            (candidate, scene, reservation)
        }).join().unwrap();
        assert!(scene.root_brep_display_pin(&scene.document.stamp(brep_id).unwrap()).unwrap().is_some());
        assert!(matches!(coordinator.publish_history(candidate), Err(om9_core::Error::Stale("history candidate"))));
        assert_eq!(coordinator.snapshot(), after_discard);
        drop(scene); drop(reservation);
    }

    #[test]
    fn interior_wires_render_without_becoming_brep_edges() {
        use om9_core::{brep::{Brep,BrepDisplay,BrepTopology},budget::Limits,
            geometry::Geometry,model::{Document,Entity}};
        use super::{Camera,Viewport,PickIntent,PickKind,RenderBuffers,Subelement};
        use std::sync::Arc;
        let limits=Limits::default();
        let edge=uuid::Uuid::from_u128(7);
        let shape=Brep::from_native("CASCADE Topology fixture".into(),BrepDisplay {
            source_sha256:[0;32],deflection_mm:0.1,
            points:vec![[-2.,-2.,0.],[2.,-2.,0.],[0.,2.,0.]],triangles:vec![[0,1,2,0]],
            segments:vec![[0,1,0]],vertices:vec![],
            topology: BrepTopology {faces:vec![uuid::Uuid::from_u128(1)],edges:vec![edge],vertex_ids:vec![],face_edges:vec![[0,0]]}.into(),
        },&limits).unwrap();
        let mut doc=Document::new("render-only BRep wire");
        let entity=Entity::new("face",Geometry::Brep(shape.clone()),None); let id=entity.id;
        doc.entities.insert(id,entity);
        let scene=super::Scene::capture(Arc::new(doc),1,&limits).unwrap();
        let stamp=scene.document.stamp(id).unwrap();
        assert_eq!(scene.root_brep_output_deflection(&stamp).unwrap(), 0.1);
        let mut stale = stamp;
        stale.geometry_revision += 1;
        assert!(scene.root_brep_output_deflection(&stale).is_err());
        let (prepared,old)=super::Scene::prepare_brep_buffers(&shape,&limits,|_,_|Ok(())).unwrap();
        let mut calls=Vec::new();
        let (prepared,new)=prepared.with_interior_wires(&[vec![[-0.5,0.,0.],[0.5,0.,0.]]],&limits,|live,pending| {
            calls.push((live,pending)); Ok(())
        }).unwrap();
        assert!(new.bytes>old.bytes && calls.iter().any(|(live,pending)| *live==old.bytes && *pending>=new.bytes));
        let RenderBuffers::Mesh {vertices,edges,..}=prepared.as_ref() else {panic!("mesh")};
        assert_eq!((vertices.len(),edges.len()),(5,2));
        let scene=scene.try_replace_root_brep_display(&stamp,&shape,prepared,[0,0],Arc::new(()),Arc::new(())).unwrap();
        assert_eq!(scene.root_brep_output_deflection(&stamp).unwrap(), 0.1);
        let camera=Camera::orthographic([0.,0.,10.],[0.,0.,0.],[0.,1.,0.],8.).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[800.,800.]};
        let interior=camera.project([0.,0.,0.],viewport).unwrap().unwrap().screen;
        assert!(scene.pick_kind(interior,&camera,viewport,2.,PickIntent::Select,PickKind::Edge).unwrap().is_none());
        assert!(scene.pick_edge_candidates(interior,&camera,viewport,2.,PickIntent::Select).unwrap().is_empty());
        let boundary=camera.project([0.,-2.,0.],viewport).unwrap().unwrap().screen;
        assert_eq!(scene.pick_kind(boundary,&camera,viewport,2.,PickIntent::Select,PickKind::Edge).unwrap().unwrap().subset,Subelement::BrepEdge(edge));
        let (prepared,_)=super::Scene::prepare_brep_buffers(&shape,&limits,|_,_|Ok(())).unwrap();
        assert!(prepared.with_interior_wires(&[vec![[f64::NAN,0.,0.],[0.,0.,0.]]],&limits,|_,_|Ok(())).is_err());
        let (prepared,_)=super::Scene::prepare_brep_buffers(&shape,&limits,|_,_|Ok(())).unwrap();
        assert!(prepared.with_interior_wires(&[vec![[0.,0.,0.],[1.,0.,0.]]],&limits,|_,_|Err(om9_core::Error::Canceled)).is_err());
    }
    #[test]
    fn distinct_remeshed_roots_keep_independent_leases_and_accounting() {
        use om9_core::{brep::{Brep, BrepDisplay, BrepTopology}, budget::Limits,
            geometry::{Geometry, Transform}, model::{Document, Entity}};
        use std::sync::Arc;
        let limits = Limits::default();
        let shape = Brep::from_native("CASCADE Topology fixture".into(), BrepDisplay {
            source_sha256: [0; 32], deflection_mm: 0.1,
            points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], triangles: vec![[0, 1, 2, 0]],
            segments: vec![], vertices: vec![], topology: BrepTopology {
                faces: vec![uuid::Uuid::from_u128(1)], edges: vec![], vertex_ids: vec![], face_edges: vec![],
            }.into(),
        }, &limits).unwrap();
        let mut document = Document::new("independent display leases");
        let first = Entity::new("first", Geometry::Brep(shape.clone()), None);
        let mut second = Entity::new("second", Geometry::Brep(shape.clone()), None);
        second.placement = Transform::translation([3., 0., 0.]);
        let (a, b) = (first.id, second.id);
        document.entities.insert(a, first); document.entities.insert(b, second);
        let scene = super::Scene::capture(Arc::new(document), 1, &limits).unwrap();
        let (stamp_a, stamp_b) = (scene.document.stamp(a).unwrap(), scene.document.stamp(b).unwrap());
        let lease_a = Arc::new(()); let weak_a = Arc::downgrade(&lease_a);
        let lease_b = Arc::new(()); let weak_b = Arc::downgrade(&lease_b);
        let (buffer_a, usage_a) = super::Scene::prepare_brep_buffers(&shape, &limits, |_, _| Ok(())).unwrap();
        let mut altered = shape.display().clone();
        altered.points[0][0] = 0.25;
        let other = Brep::from_native(shape.source().to_owned(), altered, &limits).unwrap();
        assert!(matches!(scene.try_replace_root_brep_display(&stamp_a, &other, buffer_a, [0, 0], Arc::new(()), Arc::new(())),
            Err(om9_core::Error::Stale("remesh display buffers"))));
        let (buffer_a, _) = super::Scene::prepare_brep_buffers(&shape, &limits, |_, _| Ok(())).unwrap();
        let equal_but_independent = Brep::from_native(shape.source().to_owned(), shape.display().clone(), &limits).unwrap();
        assert_eq!(shape.display_fingerprint(), equal_but_independent.display_fingerprint());
        assert!(matches!(scene.try_replace_root_brep_display(&stamp_a, &equal_but_independent, buffer_a, [0, 0], Arc::new(()), Arc::new(())),
            Err(om9_core::Error::Stale("remesh display buffers"))));
        let (buffer_a, _) = super::Scene::prepare_brep_buffers(&shape, &limits, |_, _| Ok(())).unwrap();
        let mut edited = shape.clone();
        edited.edit_display_for_validation(|display| display.points[0][0] += 0.5);
        assert_eq!(shape.display_fingerprint(), edited.display_fingerprint(), "validation fixture preserves old hash");
        assert!(matches!(scene.try_replace_root_brep_display(&stamp_a, &edited, buffer_a, [0, 0], Arc::new(()), Arc::new(())),
            Err(om9_core::Error::Stale("remesh display buffers"))));
        let (buffer_a, _) = super::Scene::prepare_brep_buffers(&shape, &limits, |_, _| Ok(())).unwrap();
        let scene_a = scene.try_replace_root_brep_display(&stamp_a, &shape, buffer_a, [0, 0], lease_a.clone(), lease_a).unwrap();
        assert!(scene_a.has_root_remesh_profile(&stamp_a, [0, 0]));
        assert!(!scene_a.has_root_remesh_profile(&stamp_a, [0, 1]));
        let mut duplicate = scene_a.try_render_snapshot().unwrap();
        duplicate.objects.push(duplicate.objects.iter().find(|object| object.root == a).unwrap().clone());
        assert!(!duplicate.has_root_remesh_profile(&stamp_a, [0, 0]));
        drop(duplicate);
        let (buffer_b, usage_b) = super::Scene::prepare_brep_buffers(&shape, &limits, |_, _| Ok(())).unwrap();
        let both = scene_a.try_replace_root_brep_display(&stamp_b, &shape, buffer_b, [0, 0], lease_b.clone(), lease_b).unwrap();
        assert_eq!(both.registered_display_bytes, usage_a.bytes + usage_b.bytes);
        let mut buffer_a = both.objects.iter().find(|o| o.root == a).unwrap().buffers.clone();
        assert!(matches!(buffer_a.make_mut_unregistered(), Err(om9_core::Error::Invalid("registered scene buffer is immutable"))));
        let snapshot = both.try_render_snapshot().unwrap();
        drop(both); drop(scene_a); drop(scene);
        assert!(weak_a.upgrade().is_some() && weak_b.upgrade().is_some());
        drop(snapshot);
        assert!(weak_a.upgrade().is_some(), "cloned buffer retains its lease");
        assert!(weak_b.upgrade().is_none(), "unreferenced second lease is released");
        drop(buffer_a);
        assert!(weak_a.upgrade().is_none());
    }
    #[test]
    fn picking_uses_published_display_topology_order() {
        use om9_core::{brep::{Brep, BrepDisplay, BrepTopology}, budget::{Limits, Usage},
            geometry::Geometry, model::{Document, Entity}};
        use super::{Camera, Viewport, PickIntent, PickKind, Subelement};
        use std::sync::Arc;
        let limits = Limits::default();
        let face_a = uuid::Uuid::from_u128(1);
        let face_b = uuid::Uuid::from_u128(2);
        let display = BrepDisplay {
            source_sha256: [0; 32], deflection_mm: 0.1,
            points: vec![[-2., -1., 0.], [-1., -1., 0.], [-1.5, 1., 0.],
                         [1., -1., 0.], [2., -1., 0.], [1.5, 1., 0.]],
            triangles: vec![[0, 1, 2, 0], [3, 4, 5, 1]], segments: vec![], vertices: vec![],
            topology: BrepTopology { faces: vec![face_a, face_b], edges: vec![],
                vertex_ids: vec![], face_edges: vec![] }.into(),
        };
        let original = Brep::from_native("CASCADE Topology source".into(), display, &limits).unwrap();
        let mut alternate = original.display().clone();
        alternate.topology.faces.swap(0, 1);
        for point in &mut alternate.points { point[2] = 1.; }
        alternate.triangles[0][3] = 1;
        alternate.triangles[1][3] = 0;
        let alternate = Brep::from_native(original.source().to_owned(), alternate, &limits).unwrap();
        let mut document = Document::new("scene topology order");
        let entity = Entity::new("body", Geometry::Brep(original), None);
        document.entities.insert(entity.id, entity);
        let mut scene = super::Scene::capture(Arc::new(document), 1, &limits).unwrap();
        // Simulate an already verified buffer/display pair; this test does not
        // authorize publication or establish that a native mapping is valid.
        scene.objects[0].buffers = super::SceneBuffers::owned(super::cache(&Geometry::Brep(alternate.clone()), &mut Usage::default(), &limits).unwrap(), None);
        scene.objects[0].brep_pick_display = Some(alternate.pin_display(Arc::new(())));
        let stamp = scene.document.stamp(scene.objects[0].root).unwrap();
        let published = scene.root_brep_pick_display(&stamp).unwrap();
        assert_eq!(published.points, alternate.display().points);
        assert_eq!(published.topology.faces, alternate.display().topology.faces);
        let mut stale = stamp.clone();
        stale.geometry_revision += 1;
        assert!(scene.root_brep_pick_display(&stale).is_err());
        let camera = Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 8.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [800., 800.] };
        for (point, expected) in [([-1.5, 0., 1.], face_a), ([1.5, 0., 1.], face_b)] {
            let cursor = camera.project(point, viewport).unwrap().unwrap().screen;
            let hit = scene.pick_kind(cursor, &camera, viewport, 1., PickIntent::Select, PickKind::Face).unwrap().unwrap();
            assert_eq!(hit.subset, Subelement::BrepFace(expected));
            let candidates = crate::osnap::candidates(&scene, &camera, viewport, cursor, Default::default(), None).unwrap();
            assert!(candidates.iter().any(|candidate| (candidate.world[2] - 1.).abs() < 1e-9
                && candidate.surface_source.as_ref().is_some_and(|source| source.face == Some(expected))),
                "surface snap must use the same display topology as picking");
        }
    }

    #[test]
    fn pick_revalidation_distinguishes_same_document_scene_publications() {
        use om9_core::{budget::Limits, geometry::Geometry, model::{Document, Entity},
            transaction::{TransactionCoordinator, World}};
        use super::{Camera, Viewport, PickIntent};
        use std::sync::Arc;
        let limits = Limits::default();
        let mut document = Document::new("scene publication");
        let entity = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        document.entities.insert(entity.id, entity);
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let original = super::Scene::capture(Arc::clone(&coordinator.world().documents[&id]), coordinator.epoch(), &limits).unwrap();
        let snapshot = original.try_render_snapshot().unwrap();
        let replacement = super::Scene::capture(Arc::clone(&original.document), original.epoch, &limits).unwrap();
        let camera = Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 8.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [800., 800.] };
        let cursor = camera.project([0., 0., 0.], viewport).unwrap().unwrap().screen;
        let hit = original.pick(cursor, &camera, viewport, 1., PickIntent::Select).unwrap().unwrap();
        assert!(hit.revalidate_in_scene(&coordinator, Some(&original), &camera, viewport).is_ok());
        assert!(hit.revalidate_in_scene(&coordinator, Some(&snapshot), &camera, viewport).is_ok());
        assert!(matches!(hit.revalidate_in_scene(&coordinator, Some(&replacement), &camera, viewport), Err(om9_core::Error::Stale("pick scene publication"))));
    }

    #[test]
    fn detached_pick_snapshot_does_not_retain_document_or_native_source() {
        use om9_core::{budget::Limits, geometry::{Geometry, Mesh, NativeBrep}, model::{Document, Entity}};
        use std::sync::Arc;
        let native = NativeBrep {
            display: Arc::new(Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]],
                normals: vec![], colors: vec![], corner_uvs: vec![] }),
            payload: vec![1, 2, 3].into(), crc: 7,
        };
        let payload = Arc::downgrade(&native.payload);
        let display = Arc::downgrade(&native.display);
        let mut document = Document::new("detached pick");
        let entity = Entity::new("body", Geometry::NativeBrep(native), None);
        document.entities.insert(entity.id, entity);
        let document = Arc::new(document);
        let weak = Arc::downgrade(&document);
        let snapshot = super::Scene::capture(Arc::clone(&document), 1, &Limits::default()).unwrap().into_pick_snapshot().unwrap();
        drop(document);
        assert!(weak.upgrade().is_none());
        assert!(payload.upgrade().is_none());
        assert!(display.upgrade().is_none());
        assert_eq!(snapshot.objects.len(), 1);
        assert!(snapshot.spatial_index().is_ok());
    }

    #[test]
    fn detached_occlusion_matches_scene_and_does_not_keep_document() {
        use om9_core::{budget::Limits, geometry::{Geometry, Mesh, NativeBrep}, model::{Document, Entity}};
        use std::sync::Arc;
        let native = NativeBrep {
            display: Arc::new(Mesh { vertices: vec![[-2.,-2.,0.], [2.,-2.,0.], [0.,2.,0.]], faces: vec![vec![0,1,2]],
                normals: vec![], colors: vec![], corner_uvs: vec![] }),
            payload: vec![1,2,3].into(), crc: 7,
        };
        let payload = Arc::downgrade(&native.payload);
        let mut document = Document::new("detached occlusion");
        let entity = Entity::new("body", Geometry::NativeBrep(native), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let document = Arc::new(document);
        let weak = Arc::downgrade(&document);
        let mut scene = super::Scene::capture(Arc::clone(&document), 1, &Limits::default()).unwrap();
        // Exercise the callback path as well as ordinary mesh intersection.
        scene.objects[0].buffers = super::SceneBuffers::owned_with_trim_fallback(scene.objects[0].buffers.as_ref().clone(), None, true);
        let camera = super::Camera::orthographic([0.,0.,10.], [0.,0.,0.], [0.,1.,0.], 8.).unwrap();
        let viewport = super::Viewport { origin: [0.,0.], size: [800.,800.] };
        let snapshot = scene.try_render_snapshot().unwrap().into_occlusion_snapshot().unwrap();
        for (cursor, target, excluded, proof) in [
            ([400.,400.], [0.,0.,-1.], false, false),
            ([400.,400.], [0.,0.,-1.], false, true),
            ([400.,400.], [0.,0.,1.], false, false),
            ([10.,10.], [3.9,3.9,-1.], false, false),
            ([400.,400.], [0.,0.,-1.], true, false),
        ] {
            let excluded_ids = if excluded { vec![id] } else { vec![] };
            let expected = crate::osnap::visible_display_occludes_with_fallback_proof(&scene, &camera, viewport,
                cursor, target, 0.001, &excluded_ids, None, |_| proof).unwrap();
            let actual = snapshot.visible_display_occludes_with_fallback_proof(&camera, viewport,
                cursor, target, 0.001, &excluded_ids, None, |_| proof).unwrap();
            assert_eq!(actual, expected);
        }
        drop(scene);
        drop(document);
        assert!(weak.upgrade().is_none());
        assert!(payload.upgrade().is_none());
        let occluded = std::thread::spawn(move || snapshot.visible_display_occludes_with_fallback_proof(&camera, viewport,
            [400.,400.], [0.,0.,-1.], 0.001, &[], None, |_| false).unwrap()).join().unwrap();
        assert!(occluded);
    }

    #[test]
    fn occlusion_snapshot_rejects_stale_source_before_detaching() {
        use om9_core::{budget::Limits, geometry::{Geometry, Mesh}, model::{Document, Entity}, Error};
        use std::sync::Arc;
        let mesh = Mesh { vertices: vec![[-1.,-1.,0.],[1.,-1.,0.],[0.,1.,0.]], faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = Document::new("stale occlusion");
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        document.entities.insert(entity.id, entity);
        let mut scene = super::Scene::capture(Arc::new(document), 1, &Limits::default()).unwrap();
        scene.objects[0].leaf_geometry_revision += 1;
        assert!(matches!(scene.into_occlusion_snapshot(), Err(Error::Stale(_))));
    }

    #[test]
    fn scene_native_source_lease_rejects_same_revision_payload_swap() {
        use om9_core::{budget::Limits, geometry::{Geometry, Mesh, NativeBrep}, model::{Document, Entity},
            transaction::{TransactionCoordinator, World}};
        use std::sync::Arc;
        let limits = Limits::default();
        let native = NativeBrep {
            display: Arc::new(Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]],
                normals: vec![], colors: vec![], corner_uvs: vec![] }),
            payload: vec![1, 2, 3].into(), crc: 7,
        };
        let mut document = Document::new("native scene lease");
        let entity = Entity::new("body", Geometry::NativeBrep(native), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let scene = super::Scene::capture(Arc::new(document.clone()), 1, &limits).unwrap();
        let current = TransactionCoordinator::new(World::new(vec![document.clone()], &limits).unwrap(), limits).unwrap();
        let lease = scene.capture_native_brep_source_for(&current, &scene.objects[0]).unwrap();
        assert_eq!(lease.payload(), &[1, 2, 3]);
        let Geometry::NativeBrep(brep) = &mut document.entities.get_mut(&id).unwrap().geometry else { panic!("native BRep") };
        brep.payload = vec![4, 5, 6].into();
        let replaced = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        assert!(matches!(scene.capture_native_brep_source_for(&replaced, &scene.objects[0]), Err(om9_core::Error::Stale(_))));
    }

    #[test]
    fn scene_source_lease_checks_current_document_and_does_not_hold_display() {
        use om9_core::{brep::{Brep, BrepDisplay, BrepTopology}, budget::Limits, geometry::Geometry,
            model::{Document, Entity}, transaction::{Mutation, TransactionCoordinator, TransactionRequest, World}};
        let limits = Limits::default();
        let brep = Brep::from_native("CASCADE Topology source".into(), BrepDisplay {
            source_sha256: [0; 32], deflection_mm: 0.1,
            points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], triangles: vec![[0, 1, 2, 0]],
            segments: vec![], vertices: vec![], topology: BrepTopology {
                faces: vec![uuid::Uuid::from_u128(1)], edges: vec![], vertex_ids: vec![], face_edges: vec![],
            }.into(),
        }, &limits).unwrap();
        let display = brep.display_liveness();
        let mut document = Document::new("scene source lease");
        let entity = Entity::new("body", Geometry::Brep(brep), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let doc = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let scene = super::Scene::capture(coordinator.world().documents[&doc].clone(), 1, &limits).unwrap();
        let object = &scene.objects[0];
        let lease = scene.capture_brep_source_for(&coordinator, object).unwrap();
        assert_eq!(lease.source(), "CASCADE Topology source");
        let stamp = coordinator.world().documents[&doc].stamp(id).unwrap();
        let prepared = coordinator.prepare(TransactionRequest::new("delete", vec![Mutation::Delete { source: stamp }])).unwrap();
        coordinator.commit(prepared).unwrap();
        assert!(scene.capture_brep_source_for(&coordinator, object).is_err());
        drop(scene);
        coordinator.close_document(doc).unwrap();
        assert!(!display.is_alive());
        assert_eq!(lease.topology().faces, [uuid::Uuid::from_u128(1)]);
    }

    #[test]
    fn shared_buffers_reject_a_different_brep_receipt() {
        use std::sync::Arc;
        let buffers = Arc::new(super::BufferPayload { data: super::RenderBuffers::Points { points: vec![[0., 0., 0.]], native_indices: vec![0] }, bounds: std::sync::OnceLock::new(), lease: std::sync::OnceLock::new(), trim_fallback: false, interior_wires_prepared: false });
        let first = Some(super::BrepDisplayReceipt::Occt([1; 32]));
        let entry = super::SharedBuffer { buffers: buffers.clone(), receipt: first.clone() };
        assert!(Arc::ptr_eq(&entry.get(&first).unwrap(), &buffers));
        let changed = Some(super::BrepDisplayReceipt::Occt([2; 32]));
        assert!(matches!(entry.get(&changed), Err(om9_core::Error::Stale("shared BRep display source"))));
        assert!(matches!(entry.get(&None), Err(om9_core::Error::Stale(_))));
    }

    #[test]
    fn native_brep_receipt_rejects_changed_payload_or_display_at_same_revision() {
        use om9_core::{budget::Limits, geometry::{Geometry, Mesh, NativeBrep}, model::{Document, Entity}};
        use std::sync::Arc;

        let limits = Limits::default();
        let body = NativeBrep {
            display: Arc::new(Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] }),
            payload: vec![1, 2, 3].into(), crc: 1,
        };
        let mut document = Document::new("native BRep receipt");
        let entity = Entity::new("body", Geometry::NativeBrep(body), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let old = super::Scene::capture(Arc::new(document.clone()), 1, &limits).unwrap();
        let old_object = &old.objects[0];
        let Geometry::NativeBrep(brep) = &mut document.entities.get_mut(&id).unwrap().geometry else { panic!("native BRep") };
        brep.payload = vec![4, 5, 6].into();
        let changed_payload = super::Scene::capture(Arc::new(document.clone()), 2, &limits).unwrap();
        assert_eq!(old_object.source, changed_payload.objects[0].source);
        assert!(matches!(changed_payload.entity_for(&changed_payload.document, old_object), Err(om9_core::Error::Stale(_))));
        assert!(changed_payload.entity_for(&changed_payload.document, &changed_payload.objects[0]).is_ok());
        let Geometry::NativeBrep(brep) = &mut document.entities.get_mut(&id).unwrap().geometry else { panic!("native BRep") };
        Arc::make_mut(&mut brep.display).vertices[1][0] = 2.;
        let changed_display = super::Scene::capture(Arc::new(document), 3, &limits).unwrap();
        assert!(matches!(changed_display.entity_for(&changed_display.document, &changed_payload.objects[0]), Err(om9_core::Error::Stale(_))));
        assert!(changed_display.entity_for(&changed_display.document, &changed_display.objects[0]).is_ok());
    }

    #[test]
    fn native_brep_receipt_does_not_retain_mesh_or_payload_contents() {
        use om9_core::{geometry::{Geometry, Mesh, NativeBrep}, model::Entity};
        use std::sync::Arc;
        let body = NativeBrep {
            display: Arc::new(Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] }),
            payload: vec![1].into(), crc: 7,
        };
        let entity = Entity::new("body", Geometry::NativeBrep(body), None);
        let receipt = super::brep_display_fingerprint(&entity).unwrap();
        drop(entity);
        let super::BrepDisplayReceipt::Native { payload, display, .. } = receipt else { panic!("native receipt") };
        assert!(payload.upgrade().is_none());
        assert!(display.upgrade().is_none());
    }

    #[test]
    fn reused_scene_refreshes_receipt_for_equal_independent_native_owners() {
        use om9_core::{budget::Limits, geometry::{Geometry, Mesh, NativeBrep}, model::{Document, Entity}};
        use std::sync::Arc;
        let limits = Limits::default();
        let body = NativeBrep {
            display: Arc::new(Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] }),
            payload: vec![1, 2, 3].into(), crc: 1,
        };
        let mut document = Document::new("native owner replacement");
        let entity = Entity::new("body", Geometry::NativeBrep(body), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let scene = super::Scene::capture(Arc::new(document.clone()), 1, &limits).unwrap();
        let Geometry::NativeBrep(brep) = &mut document.entities.get_mut(&id).unwrap().geometry else { panic!("native BRep") };
        brep.payload = Arc::new((*brep.payload).clone());
        brep.display = Arc::new((*brep.display).clone());
        let updated = scene.update_owned(Arc::new(document), 2, None, &limits).unwrap();
        assert!(scene.objects[0].buffers.ptr_eq(&updated.objects[0].buffers));
        assert!(updated.entity_for(&updated.document, &updated.objects[0]).is_ok());
        assert!(matches!(updated.entity_for(&updated.document, &scene.objects[0]), Err(om9_core::Error::Stale(_))));
    }

    #[test]
    fn brep_receipt_rejects_source_swap_with_unchanged_revision() {
        use om9_core::{brep::{Brep, BrepDisplay, BrepTopology}, budget::Limits, geometry::Geometry, model::{Document, Entity}};
        use std::sync::Arc;

        let limits = Limits::default();
        let body = |source: &str| Brep::from_native(source.to_owned(), BrepDisplay {
            source_sha256: [0; 32],
            deflection_mm: 0.1,
            points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2, 0]],
            segments: vec![],
            vertices: vec![],
            topology: BrepTopology {
                faces: vec![uuid::Uuid::from_u128(1)],
                edges: vec![],
                vertex_ids: vec![],
                face_edges: vec![],
            }.into(),
        }, &limits).unwrap();
        let mut document = Document::new("BRep source receipt");
        let entity = Entity::new("body", Geometry::Brep(body("CASCADE Topology A")), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let old = super::Scene::capture(Arc::new(document.clone()), 1, &limits).unwrap();
        document.entities.get_mut(&id).unwrap().geometry = Geometry::Brep(body("CASCADE Topology B"));
        let new = super::Scene::capture(Arc::new(document), 2, &limits).unwrap();
        assert_eq!(old.objects[0].source, new.objects[0].source);
        assert!(matches!(new.entity_for(&new.document, &old.objects[0]), Err(om9_core::Error::Stale("scene BRep display source"))));
        assert!(new.entity_for(&new.document, &new.objects[0]).is_ok());
    }

    #[test]
    fn entity_lookup_rejects_stale_source_and_wrong_leaf() {
        use om9_core::{Error, budget::Limits, geometry::Geometry, model::{Document, Entity}};
        use std::sync::Arc;

        let mut document = Document::new("scene identity");
        let entity = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        document.entities.insert(entity.id, entity);
        let scene = super::Scene::capture(Arc::new(document), 1, &Limits::default()).unwrap();
        let mut object = scene.objects[0].clone();
        assert!(scene.entity_for(&scene.document, &object).is_ok());
        let foreign = scene.document.as_ref().clone();
        assert!(matches!(scene.entity_for(&foreign, &object), Err(Error::Stale("scene document owner"))));
        object.source.geometry_revision += 1;
        assert!(matches!(scene.entity_for(&scene.document, &object), Err(Error::Stale(_))));
        object.source = scene.objects[0].source;
        object.leaf = om9_core::identity::ObjectId(uuid::Uuid::new_v4());
        assert!(matches!(scene.entity_for(&scene.document, &object), Err(Error::Stale("scene leaf"))));
        object.leaf = scene.objects[0].leaf;
        object.leaf_geometry_revision += 1;
        assert!(matches!(scene.entity_for(&scene.document, &object), Err(Error::Stale("scene leaf revision"))));
        let document = Arc::clone(&scene.document);
        let entity = scene.entity_for(&document, &scene.objects[0]).unwrap();
        let id = entity.id;
        drop(scene);
        assert_eq!(entity.id, id);
    }

    #[test]
    fn snapshot_failure_releases_shared_owners_and_keeps_source_intact() {
        use om9_core::{Error, budget::Limits, geometry::Geometry, model::{Document, Entity}};
        use std::sync::Arc;
        let mut document = Document::new("snapshot failure");
        for index in 0..2 {
            let entity = Entity::new("point", Geometry::Point([index as f64, 0., 0.]), None);
            document.entities.insert(entity.id, entity);
        }
        let mut scene = super::Scene::capture(Arc::new(document), 1, &Limits::default()).unwrap();
        scene.diagnostics.push("fixture warning".into());
        let document_owners = Arc::strong_count(&scene.document);
        let buffer_owners: Vec<_> = scene.objects.iter().map(|object| Arc::strong_count(&object.buffers.inner)).collect();
        for fail_at in 0..5 {
            let mut step = 0;
            let result = scene.try_render_snapshot_checked(|| {
                let current = step;
                step += 1;
                if current == fail_at { Err(Error::Budget("injected snapshot failure")) } else { Ok(()) }
            });
            assert!(matches!(result, Err(Error::Budget("injected snapshot failure"))));
            assert_eq!(Arc::strong_count(&scene.document), document_owners);
            assert_eq!(scene.objects.iter().map(|object| Arc::strong_count(&object.buffers.inner)).collect::<Vec<_>>(), buffer_owners);
            assert_eq!(scene.objects.len(), 2);
            assert_eq!(scene.diagnostics, ["fixture warning"]);
        }
        assert_eq!(scene.try_render_snapshot().unwrap().objects.len(), 2);
    }

    #[test]
    fn display_vector_reports_capacity_overflow() {
        assert!(matches!(super::display_vector::<u64>(usize::MAX, "display test"),
            Err(om9_core::Error::Budget("display test"))));
        assert_eq!(super::display_vector::<u64>(0, "display test").unwrap().len(), 0);
    }

    #[test]
    fn display_capacity_above_logical_lengths_is_charged_before_publication() {
        use om9_core::{Error, budget::{Limits, Usage}};
        let mut points = Vec::with_capacity(128);
        points.push([0., 0., 0.]);
        let buffer = super::RenderBuffers::Points { points, native_indices: vec![0] };
        let actual = super::buffer_usage(&buffer).unwrap();
        let estimated = Usage::array::<([f64; 3], u32)>(1).unwrap().bytes;
        assert!(actual.bytes > estimated);
        let mut usage = Usage { bytes: estimated, ..Usage::default() };
        let tight = Limits { max_bytes: actual.bytes - 1, ..Limits::default() };
        assert!(matches!(super::charge_buffer_capacity(&mut usage, 0, &buffer, &tight), Err(Error::Budget("bytes"))));
        assert_eq!(usage.bytes, estimated);
        let enough = Limits { max_bytes: actual.bytes, ..Limits::default() };
        assert_eq!(super::charge_buffer_capacity(&mut usage, 0, &buffer, &enough).unwrap().bytes, actual.bytes);
        assert_eq!(usage.bytes, actual.bytes);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderBuffers {
    Points {
        points: Vec<Point3>,
        native_indices: Vec<u32>,
    },
    Curve {
        points: Vec<Point3>,
        /// Native parameters for evaluator-backed curves. Empty for exact
        /// line polylines, which have no global native parameter binding.
        parameters: Vec<f64>,
    },
    Mesh {
        vertices: Vec<Point3>,
        triangles: Vec<([u32; 3], u32)>,
        edges: Vec<[u32; 2]>,
    },
}
/// Buffer data bound to the immutable display from which it was prepared.
/// This receipt does not attest the native topology mapping.
pub struct PreparedBrepBuffers {
    buffers: RenderBuffers,
    display: om9_core::brep::BrepDisplayIdentity,
    fingerprint: [u8; 32],
    interior_wires_prepared: bool,
}
impl PreparedBrepBuffers {
    pub fn as_ref(&self) -> &RenderBuffers { &self.buffers }

    /// Sample source-derived NURBS wires using the ordinary curve display
    /// policy, including every interior knot. The returned temporary is owned
    /// by the caller and must stay charged until it is dropped.
    pub fn sample_interior_curves(curves: &[Curve], limits: &Limits,
        mut admit: impl FnMut(u64,u64)->Result<()>) -> Result<(Vec<Vec<Point3>>,u64)> {
        let mut point_count=0usize;
        let mut max_count=0usize;
        let mut max_degree=0usize;
        for curve in curves {
            let Curve::Nurbs(n)=curve else {return Err(Error::Unsupported("BRep interior curve type"));};
            n.validate(limits)?;
            let count=nurbs_display_segments(n).checked_add(1)
                .and_then(|npoints| npoints.checked_add(n.knots.len())).ok_or(Error::Overflow("BRep wire samples"))?;
            point_count=point_count.checked_add(count).ok_or(Error::Overflow("BRep wire samples"))?;
            max_count=max_count.max(count); max_degree=max_degree.max(n.degree);
        }
        let points=Usage::array::<Point3>(point_count)?.bytes;
        let headers=Usage::array::<Vec<Point3>>(curves.len())?.bytes;
        let retained=points.checked_add(headers).ok_or(Error::Overflow("BRep wire samples"))?;
        let scratch=Usage::array::<f64>(max_count)?.bytes
            .checked_add(Usage::array::<[[f64;4];4]>(max_degree.checked_add(1).ok_or(Error::Overflow("BRep wire degree"))?)?.bytes)
            .and_then(|n|n.checked_add(4096)).ok_or(Error::Overflow("BRep wire sampling scratch"))?;
        let total=retained.checked_add(scratch).ok_or(Error::Overflow("BRep wire sampling"))?;
        Usage {bytes:total,points:u64::try_from(point_count).map_err(|_|Error::Overflow("BRep wire samples"))?,..Usage::default()}.check(limits)?;
        admit(0,total)?;
        let mut lines=display_vector(curves.len(),"BRep wire allocation")?;
        let mut live=Usage::array::<Vec<Point3>>(lines.capacity())?.bytes;
        admit(live,total.saturating_sub(headers))?;
        for curve in curves {
            let Curve::Nurbs(n)=curve else {unreachable!()};
            let steps=nurbs_display_segments(n);
            let capacity=steps.checked_add(1).and_then(|v|v.checked_add(n.knots.len())).ok_or(Error::Overflow("BRep wire samples"))?;
            let temporary=Usage::array::<f64>(capacity)?.bytes.checked_add(Usage::array::<Point3>(capacity)?.bytes)
                .and_then(|n|n.checked_add(scratch)).ok_or(Error::Overflow("BRep wire sampling"))?;
            admit(live,temporary)?;
            let mut parameters=display_vector(capacity,"BRep wire parameters")?;
            for i in 0..=steps {
                parameters.push(if i==steps {n.domain[1]} else {n.domain[0]+(n.domain[1]-n.domain[0])*i as f64/steps as f64});
            }
            parameters.extend(n.knots.iter().copied().filter(|t|*t>n.domain[0] && *t<n.domain[1]));
            parameters.sort_by(f64::total_cmp);
            parameters.dedup_by(|a,b|*a==*b);
            let evaluator=CurveEvaluator::capture(curve,limits)?;
            let mut line=display_vector(parameters.len(),"BRep wire points")?;
            let parameter_bytes=Usage::array::<f64>(parameters.capacity())?.bytes;
            let line_bytes=Usage::array::<Point3>(line.capacity())?.bytes;
            let during=live.checked_add(parameter_bytes).and_then(|n|n.checked_add(line_bytes))
                .ok_or(Error::Overflow("BRep wire samples"))?;
            admit(during,scratch)?;
            for t in parameters {
                admit(during,scratch)?;
                line.push(evaluator.evaluate(Parameter::Native(t),0,Transform::identity())?[0]);
            }
            live=live.checked_add(line_bytes).ok_or(Error::Overflow("BRep wire samples"))?;
            lines.push(line);
            admit(live,0)?;
        }
        let actual=lines.iter().try_fold(Usage::array::<Vec<Point3>>(lines.capacity())?.bytes,|sum,line|
            sum.checked_add(Usage::array::<Point3>(line.capacity())?.bytes).ok_or(Error::Overflow("BRep wire samples")))?;
        debug_assert_eq!(actual,live);
        admit(actual,0)?;
        Ok((lines,actual))
    }

    /// Add already sampled, source-derived interior wires for rendering only.
    /// The original BRep vertices and segments remain the prefix used for
    /// topology picking. The caller accounts the input polylines separately.
    pub fn with_interior_wires(self, wires: &[Vec<Point3>], limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<(Self, Usage)> {
        let RenderBuffers::Mesh { vertices, triangles, edges } = &self.buffers else {
            return Err(Error::Invalid("BRep interior wire buffers"));
        };
        let (points, segments) = wires.iter().try_fold((vertices.len(), edges.len()), |(points, segments), wire| {
            if wire.len()<2 || wire.iter().flatten().any(|v| !v.is_finite()) {
                return Err(Error::Invalid("BRep interior wire"));
            }
            Ok((points.checked_add(wire.len()).ok_or(Error::Overflow("BRep interior wire points"))?,
                segments.checked_add(wire.len()-1).ok_or(Error::Overflow("BRep interior wire segments"))?))
        })?;
        u32::try_from(points).map_err(|_| Error::Overflow("BRep interior wire indices"))?;
        let old = buffer_usage(&self.buffers)?;
        let mut estimate = Usage { bytes: std::mem::size_of::<BufferPayload>() as u64 + 16,
            points: u64::try_from(points).map_err(|_| Error::Overflow("BRep interior wire points"))?, ..Usage::default() };
        estimate.add(Usage::array::<Point3>(points)?,limits)?;
        estimate.add(Usage::array::<([u32;3],u32)>(triangles.len())?,limits)?;
        estimate.add(Usage::array::<[u32;2]>(segments)?,limits)?;
        admit(old.bytes,estimate.bytes)?;
        let mut new_vertices=display_vector(points,"BRep interior wire points")?;
        let mut new_triangles=display_vector(triangles.len(),"BRep interior wire triangles")?;
        let mut new_edges=display_vector(segments,"BRep interior wire segments")?;
        new_vertices.extend_from_slice(vertices);
        new_triangles.extend_from_slice(triangles);
        new_edges.extend_from_slice(edges);
        for wire in wires {
            let first=u32::try_from(new_vertices.len()).map_err(|_| Error::Overflow("BRep interior wire indices"))?;
            new_vertices.extend_from_slice(wire);
            for i in 1..wire.len() {
                let end=first.checked_add(u32::try_from(i).map_err(|_| Error::Overflow("BRep interior wire indices"))?)
                    .ok_or(Error::Overflow("BRep interior wire indices"))?;
                new_edges.push([end-1,end]);
            }
        }
        let buffers=RenderBuffers::Mesh { vertices:new_vertices, triangles:new_triangles, edges:new_edges };
        let actual=buffer_usage(&buffers)?;
        actual.check(limits)?;
        admit(old.bytes,actual.bytes)?;
        let PreparedBrepBuffers { display, fingerprint, .. } = self;
        admit(actual.bytes,0)?;
        Ok((Self { buffers, display, fingerprint, interior_wires_prepared: true },actual))
    }
}
#[derive(Debug, Clone)]
pub struct SceneObject {
    pub root: ObjectId,
    pub path: Vec<ObjectId>,
    pub leaf: ObjectId,
    pub leaf_generation: u64,
    pub leaf_geometry_revision: u64,
    pub brep_display_fingerprint: Option<BrepDisplayReceipt>,
    // Accounted application scenes pin ordinary displays and remesh overrides.
    // Unregistered captures may borrow the document display after validation.
    pub(crate) brep_pick_display: Option<om9_core::brep::BrepDisplayPin>,
    pub(crate) native_display_pin: Option<om9_core::geometry::NativeBrepDisplayPin>,
    pub source: SourceStamp,
    pub transform: Transform,
    pub preview: Transform,
    pub color: [u8; 4],
    pub buffers: SceneBuffers,
    pub visible: bool,
    pub selectable: bool,
    pub editable: bool,
    pub reference_snappable: bool,
    pub reference_unlocked: bool,
    // Transitional output policy captured with the scene. Remeshing the
    // witness does not silently change the profile used for new geometry.
    brep_output_deflection: Option<f64>,
    // Exact worker input, meaningful only for a published in-process remesh.
    remesh_profile: [u64; 2],
    // Drop after buffers and pick display; each remeshed object retains its
    // own reservation even when scenes and objects are cloned independently.
    registered_display_bytes: u64,
}
/// Cloning a render buffer retains the allocation's accounting lease.
#[derive(Debug, Clone)]
pub struct SceneBuffers {
    inner: Arc<BufferPayload>,
}
#[derive(Debug, Clone)]
struct BufferPayload {
    data: RenderBuffers,
    bounds: OnceLock<Option<([f64;3],[f64;3])>>,
    lease: OnceLock<DisplayLease>,
    trim_fallback: bool,
    interior_wires_prepared: bool,
}
impl SceneBuffers {
    /// Presentation-only buffers; their accounting follows every cloned buffer.
    pub fn transient(data: RenderBuffers, owner: Arc<dyn Send + Sync>) -> Self {
        Self::owned(data, Some(DisplayLease(owner)))
    }
    fn new(inner: Arc<BufferPayload>) -> Self { Self { inner } }
    fn owned(data: RenderBuffers, lease: Option<DisplayLease>) -> Self {
        Self::owned_with_trim_fallback(data,lease,false)
    }
    fn owned_with_trim_fallback(data: RenderBuffers, lease: Option<DisplayLease>, trim_fallback: bool) -> Self {
        Self::owned_with_flags(data, lease, trim_fallback, false)
    }
    fn owned_with_flags(data: RenderBuffers, lease: Option<DisplayLease>, trim_fallback: bool, interior_wires_prepared: bool) -> Self {
        let slot = OnceLock::new();
        if let Some(lease) = lease { let _ = slot.set(lease); }
        let bounds = OnceLock::new();
        let _ = bounds.set(display_bounds(&data));
        Self::new(Arc::new(BufferPayload { data, bounds, lease: slot, trim_fallback, interior_wires_prepared }))
    }
    pub fn as_ref(&self) -> &RenderBuffers { &self.inner.data }
    /// Immutable display bounds shared with every scene/selection occurrence.
    /// Remesh creates a new payload; mutable unregistered buffers invalidate it.
    pub fn local_bounds(&self) -> Option<([f64;3],[f64;3])> {
        *self.inner.bounds.get_or_init(||display_bounds(&self.inner.data))
    }
    pub fn is_trim_display_fallback(&self) -> bool { self.inner.trim_fallback }
    pub fn ptr_eq(&self, other: &Self) -> bool { Arc::ptr_eq(&self.inner, &other.inner) }
    /// Caller must account any copy-on-write allocation; registered remesh
    /// buffers are immutable and cannot be detached from their lease.
    #[doc(hidden)]
    pub fn make_mut_unregistered(&mut self) -> Result<&mut RenderBuffers> {
        if self.inner.lease.get().is_some() { return Err(Error::Invalid("registered scene buffer is immutable")); }
        let payload = Arc::make_mut(&mut self.inner);
        payload.bounds.take();
        Ok(&mut payload.data)
    }
}
impl PartialEq for SceneBuffers {
    fn eq(&self, other: &Self) -> bool {
        self.inner.trim_fallback == other.inner.trim_fallback && self.inner.interior_wires_prepared == other.inner.interior_wires_prepared
            && self.inner.data == other.inner.data
    }
}
fn display_bounds(data: &RenderBuffers) -> Option<([f64;3],[f64;3])> {
    let points = match data {
        RenderBuffers::Points {points,..} | RenderBuffers::Curve {points,..} => points,
        RenderBuffers::Mesh {vertices,..} => vertices,
    };
    let mut min = [f64::INFINITY;3];
    let mut max = [f64::NEG_INFINITY;3];
    for p in points {
        if !p.iter().all(|v|v.is_finite()) { return None; }
        for axis in 0..3 { min[axis]=min[axis].min(p[axis]); max[axis]=max[axis].max(p[axis]); }
    }
    min[0].is_finite().then_some((min,max))
}
#[derive(Clone)]
struct DisplayLease(Arc<dyn Send + Sync>);
impl std::fmt::Debug for DisplayLease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DisplayLease").field(&Arc::as_ptr(&self.0)).finish()
    }
}
/// Identity of the immutable owners that supplied a rendered B-rep. Weak
/// references keep allocation identities distinct without retaining meshes.
#[derive(Debug, Clone)]
pub enum BrepDisplayReceipt {
    Occt([u8; 32]),
    Native {
        payload: Weak<Vec<u8>>,
        crc: u32,
    },
}
impl PartialEq for BrepDisplayReceipt {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Occt(a), Self::Occt(b)) => a == b,
            (Self::Native { payload: a, crc: c }, Self::Native { payload: x, crc: z }) =>
                Weak::ptr_eq(a, x) && c == z,
            _ => false,
        }
    }
}
pub fn brep_display_fingerprint(entity: &Entity) -> Option<BrepDisplayReceipt> {
    match &entity.geometry {
        Geometry::Brep(brep) => Some(BrepDisplayReceipt::Occt(brep.canonical_fingerprints().1)),
        Geometry::NativeBrep(brep) => Some(BrepDisplayReceipt::Native {
            payload: Arc::downgrade(&brep.payload),
            crc: brep.crc,
        }),
        _ => None,
    }
}
impl SceneObject {
    /// Borrow the exact display used by this occurrence for picking.
    pub fn pick_brep_display<'a>(&'a self, entity: &'a Entity) -> Result<Option<&'a om9_core::brep::BrepDisplay>> {
        if self.leaf != entity.id || self.leaf_generation != entity.generation
            || self.leaf_geometry_revision != entity.geometry_revision
            || self.brep_display_fingerprint != brep_display_fingerprint(entity) {
            return Err(Error::Stale("scene pick geometry source"));
        }
        match (&entity.geometry, &self.brep_pick_display) {
            (Geometry::Brep(source), Some(display)) => {
                // Registered overrides can use separately allocated topology
                // in candidate order; their callers own topology verification.
                // Ordinary displays share the document topology allocation.
                if self.registered_display_bytes == 0 && !std::ptr::eq(&*display.display().topology, source.topology()) {
                    return Err(Error::Stale("scene BRep pick topology"));
                }
                Ok(Some(display.display()))
            }
            (Geometry::NativeBrep(_), Some(display)) => Ok(Some(display.display())),
            (Geometry::Brep(_), None) | (_, Some(_)) => Err(Error::Stale("scene BRep pick display")),
            _ => Ok(None),
        }
    }
    pub fn has_pick_geometry(&self) -> bool {
        match self.buffers.as_ref() {
            RenderBuffers::Points { points, .. } => !points.is_empty(),
            RenderBuffers::Curve { points, .. } => points.len() >= 2,
            RenderBuffers::Mesh {
                triangles, edges, ..
            } => !triangles.is_empty() || !edges.is_empty(),
        }
    }
    pub fn display_transform(&self) -> Result<Transform> {
        self.preview.compose(self.transform)
    }

    /// World-space bounding sphere of this occurrence, for the CPU picking BVH. Points, curves and
    /// meshes all use their display vertices; the exact geometry test still happens after the BVH.
    pub fn sphere_bounds(&self) -> Result<([f64; 3], f64)> {
        self.sphere_bounds_from_local(self.local_bounds())
    }

    fn local_bounds(&self) -> Option<([f64; 3], [f64; 3])> {
        self.buffers.local_bounds()
    }

    /// Conservative world-space display box. This is an overlay/picking bound,
    /// never a replacement for precise geometry or the selection hit test.
    pub fn display_bounds_corners(&self) -> Result<Option<[[f64;3];8]>> {
        let Some((min,max)) = self.local_bounds() else { return Ok(None); };
        let transform = self.display_transform()?;
        let mut corners = [[0.;3];8];
        for (index,corner) in corners.iter_mut().enumerate() {
            *corner = transform.point(std::array::from_fn(|axis|if index & (1<<axis)==0 {min[axis]} else {max[axis]}))?;
        }
        Ok(Some(corners))
    }

    fn sphere_bounds_from_local(&self, bounds: Option<([f64; 3], [f64; 3])>) -> Result<([f64; 3], f64)> {
        let Some((min, max)) = bounds else { return Ok(([0.0; 3], 0.0)) };
        let centre = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        let transform = self.display_transform()?;
        let world = transform.point(centre)?;
        // A half longest side is not an enclosing sphere: it rejects box corners. Transform all
        // eight AABB corners so nonuniform scale, shear and previews remain conservative as well.
        let mut radius = 0.0f64;
        for corner in 0..8 {
            let p = transform.point(std::array::from_fn(|axis| {
                if corner & (1 << axis) == 0 {
                    min[axis]
                } else {
                    max[axis]
                }
            }))?;
            radius = radius.max(
                (0..3)
                    .map(|axis| (p[axis] - world[axis]).powi(2))
                    .sum::<f64>()
                    .sqrt(),
            );
        }
        radius += (0..3).map(|axis| world[axis].abs()).fold(1.0, f64::max) * f64::EPSILON * 8.;
        Ok((world, radius))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickIntent {
    Select,
    Reference { snap_locked: bool },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Subelement {
    BrepFace(uuid::Uuid),
    BrepEdge(uuid::Uuid),
    BrepVertex(uuid::Uuid),
    Whole,
    CurveParameter(f64),
    MeshFace(u32),
    MeshEdge { a: u32, b: u32 },
    MeshVertex(u32),
    CloudPoint(u32),
}

/// What kind of subobject a pick may return: the object itself, a vertex, an edge or a face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickKind {
    Object,
    Vertex,
    Edge,
    Face,
}
#[derive(Debug, Clone)]
pub struct PickHit {
    pub root: ObjectId,
    pub path: Vec<ObjectId>,
    pub subset: Subelement,
    pub world_point: Point3,
    pub source: SourceStamp,
    pub world_epoch: u64,
    pub(crate) scene_publication: uuid::Uuid,
    pub screen_distance: f64,
    pub depth: f64,
    pub(crate) camera: Camera,
    pub(crate) viewport: Viewport,
}
impl PickHit {
    /// Check against the scene from which interaction is currently allowed.
    pub fn revalidate_in_scene(&self, c: &TransactionCoordinator, scene: Option<&Scene>, camera: &Camera, viewport: Viewport) -> Result<()> {
        self.revalidate_in_view(c, camera, viewport)?;
        if scene.is_none_or(|scene| scene.publication_id != self.scene_publication) {
            return Err(Error::Stale("pick scene publication"));
        }
        Ok(())
    }
    pub fn revalidate(&self, c: &TransactionCoordinator, camera: &Camera) -> Result<()> {
        if c.epoch() != self.world_epoch || camera != &self.camera {
            return Err(Error::Stale("pick camera/publication"));
        }
        let d = c
            .world()
            .documents
            .get(&self.source.document)
            .ok_or(Error::Missing("pick document"))?;
        d.resolve(&self.source)?;
        Ok(())
    }
    pub fn revalidate_in_view(
        &self,
        c: &TransactionCoordinator,
        camera: &Camera,
        viewport: Viewport,
    ) -> Result<()> {
        self.revalidate(c, camera)?;
        if viewport != self.viewport {
            return Err(Error::Stale("pick viewport rectangle"));
        }
        Ok(())
    }
}
pub struct Scene {
    pub document: Arc<Document>,
    representations: Arc<Mutex<BrepRepresentationStore>>,
    pub epoch: u64,
    publication_id: uuid::Uuid,
    pub objects: Vec<SceneObject>,
    pub diagnostics: Vec<String>,
    pub(crate) limits: Limits,
    buffers_usage: Usage,
    registered_display_bytes: u64,
    // Last: cache data must drop before its budget owner.
    memory: Arc<SceneMemoryOwner>,
}
/// Worker picking data with independently charged buffers and no strong document owner.
pub struct PickSnapshot {
    objects: Vec<SceneObject>,
    limits: Limits,
    epoch: u64,
    publication_id: uuid::Uuid,
    document: Weak<Document>,
    arena_bytes: u64,
    metadata_bytes: u64,
    shared_buffer_bytes: u64,
    query_scratch: Usage,
    // The reservation outlives metadata and picking cache owners.
    memory: Arc<SceneMemoryOwner>,
}
struct HoverPickSource {
    curve: Option<om9_core::geometry::Curve>,
    cloud: bool,
    brep_display: bool,
}
/// Object-pick inputs detached from the document after validating each scene
/// occurrence. Curves are copied exactly; B-rep display pins remain accounted.
pub struct HoverPickSnapshot {
    scene: PickSnapshot,
    sources: Vec<HoverPickSource>,
}

fn copy_hover_curve(curve:&om9_core::geometry::Curve)->Result<om9_core::geometry::Curve> {
    use om9_core::geometry::{Curve,NurbsCurve};
    fn copy_slice<T:Copy>(slice:&[T])->Result<Vec<T>> {
        let mut values=Vec::new();values.try_reserve_exact(slice.len()).map_err(|_|Error::Budget("hover curve copy"))?;
        values.extend_from_slice(slice);Ok(values)
    }
    Ok(match curve {
        Curve::Line {start,end,domain}=>Curve::Line {start:*start,end:*end,domain:*domain},
        Curve::Circle {center,x,y,radius,domain,angle_domain}=>Curve::Circle {center:*center,x:*x,y:*y,radius:*radius,domain:*domain,angle_domain:*angle_domain},
        Curve::Nurbs(n)=>Curve::Nurbs(NurbsCurve {degree:n.degree,poles:copy_slice(&n.poles)?,weights:copy_slice(&n.weights)?,
            knots:copy_slice(&n.knots)?,domain:n.domain,periodic:n.periodic,reversed:n.reversed,seam:n.seam}),
        Curve::Polycurve {segments,reversed}=>{
            let mut copied=Vec::new();copied.try_reserve_exact(segments.len()).map_err(|_|Error::Budget("hover curve segments"))?;
            for segment in segments {copied.push(copy_hover_curve(segment)?);}
            Curve::Polycurve {segments:copied,reversed:*reversed}
        }
    })
}
/// A validated, document-independent view of the buffers used for occlusion.
/// The caller must retain the same admitted buffer and snapshot owners as for
/// `PickSnapshot`, and must revalidate publication before using a result.
pub struct OcclusionSnapshot(pub(crate) PickSnapshot);
fn combined(a: Permissions, b: Permissions) -> Permissions {
    Permissions {
        visible: a.visible && b.visible,
        selectable: a.selectable && b.selectable,
        editable: a.editable && b.editable,
        reference_snappable: a.reference_snappable && b.reference_snappable,
    }
}
fn buffer_usage(buffer: &RenderBuffers) -> Result<Usage> {
    let mut usage = Usage {
        bytes: std::mem::size_of::<BufferPayload>() as u64 + 16,
        ..Usage::default()
    };
    let l = Limits {
        max_bytes: u64::MAX,
        max_points: u64::MAX,
        ..Limits::default()
    };
    match buffer {
        RenderBuffers::Points {
            points,
            native_indices,
        } => {
            usage.add(Usage::array::<Point3>(points.capacity())?, &l)?;
            usage.add(Usage::array::<u32>(native_indices.capacity())?, &l)?;
            usage.points = points.len() as u64;
        }
        RenderBuffers::Curve { points, parameters } => {
            usage.add(Usage::array::<Point3>(points.capacity())?, &l)?;
            usage.add(Usage::array::<f64>(parameters.capacity())?, &l)?;
            usage.points = points.len() as u64;
        }
        RenderBuffers::Mesh {
            vertices,
            triangles,
            edges,
        } => {
            usage.add(Usage::array::<Point3>(vertices.capacity())?, &l)?;
            usage.add(Usage::array::<([u32; 3], u32)>(triangles.capacity())?, &l)?;
            usage.add(Usage::array::<[u32; 2]>(edges.capacity())?, &l)?;
            usage.points = vertices.len() as u64;
        }
    }
    Ok(usage)
}

/// `cache` admits logical lengths before allocation. Charge capacity beyond
/// those lengths (and the shared buffer owner) before publishing the scene.
fn charge_buffer_capacity(usage: &mut Usage, previous_bytes: u64, buffer: &RenderBuffers, limits: &Limits) -> Result<Usage> {
    let actual = buffer_usage(buffer)?;
    let estimated = usage.bytes.checked_sub(previous_bytes).ok_or(Error::Invalid("scene buffer accounting"))?;
    let extra = actual.bytes.saturating_sub(estimated);
    usage.add(Usage { bytes: extra, ..Usage::default() }, limits)?;
    Ok(actual)
}
fn metadata_leaf<'a>(
    document: &'a Document,
    members: &BTreeMap<(DefinitionId, ObjectId), &'a Entity>,
    root: ObjectId,
    path: &[ObjectId],
) -> Result<(&'a Entity, Permissions, bool)> {
    let mut entity = document
        .entities
        .get(&root)
        .ok_or(Error::Missing("scene root"))?;
    let mut permissions = document.permissions_for_entity(entity, true)?;
    let mut unlocked = document
        .permissions_for_entity(entity, false)?
        .reference_snappable;
    for id in path {
        let Geometry::Instance(definition) = entity.geometry else {
            return Err(Error::Stale("scene path"));
        };
        entity = members
            .get(&(definition, *id))
            .copied()
            .ok_or(Error::Stale("scene member"))?;
        permissions = combined(permissions, document.permissions_for_entity(entity, true)?);
        unlocked &= document
            .permissions_for_entity(entity, false)?
            .reference_snappable;
    }
    Ok((entity, permissions, unlocked))
}
// Walk leaves in display order without allocating a flattened copy.
fn walk_nested_segments(curve: &Curve, reverse: bool, visit: &mut impl FnMut(&Curve, bool) -> Result<()>) -> Result<()> {
    match curve {
        Curve::Polycurve { segments, reversed } => {
            let reverse = reverse ^ *reversed;
            if reverse {
                for segment in segments.iter().rev() { walk_nested_segments(segment, reverse, visit)?; }
            } else {
                for segment in segments { walk_nested_segments(segment, reverse, visit)?; }
            }
            Ok(())
        }
        _ => visit(curve, reverse),
    }
}

fn nested_display_count(curve: &Curve, limits: &Limits) -> Result<usize> {
    curve.validate(limits)?;
    let mut previous = None;
    let mut count = 1usize;
    walk_nested_segments(curve, false, &mut |segment, reverse| {
        validate_display_seam(segment, limits)?;
        let steps = polyline_display_steps(segment)?;
        let start = om9_core::curve_ops::point_at(segment, if reverse { 1. } else { 0. }, limits)?;
        let end = om9_core::curve_ops::point_at(segment, if reverse { 0. } else { 1. }, limits)?;
        let curved = !matches!(segment, Curve::Line { .. });
        if previous.is_some_and(|(point, previous_curved): (Point3, bool)| {
            if !curved && !previous_curved { point != start } else {
                (0..3).any(|axis| (point[axis]-start[axis]).abs() > 64. * f64::EPSILON * point[axis].abs().max(start[axis].abs()).max(1.))
            }
        }) {
            return Err(Error::Unsupported("nested polycurve display requires connected lines"));
        }
        previous = Some((end, curved));
        count = count.checked_add(steps).ok_or(Error::Overflow("nested polycurve vertices"))?;
        Ok(())
    })?;
    Ok(count)
}

fn has_nested_segments(curve: &Curve) -> bool {
    matches!(curve, Curve::Polycurve { segments, .. } if segments.iter().any(|segment| matches!(segment, Curve::Polycurve { .. })))
}

/// A displayed polycurve is a connected list of direct lines, arcs or continuous NURBS.
/// Keep this admission check shared by the allocation plan and display cache;
/// connecting arbitrary polycurve segments would invent geometry between gaps.
fn connected_polyline_segments<'a>(curve: &'a Curve, limits: &Limits) -> Result<&'a [Curve]> {
    let Curve::Polycurve {
        segments,
        ..
    } = curve
    else {
        return Err(Error::Unsupported(
            "polycurve display requires unreversed line segments",
        ));
    };
    if segments.is_empty() {
        return Err(Error::Unsupported(
            "polycurve display requires line segments",
        ));
    }
    let mut previous_end = None;
    for segment in segments {
        let (start, end, arc) = match segment {
            Curve::Line { start, end, .. } => (*start, *end, false),
            Curve::Circle { domain, .. } |
            Curve::Nurbs(om9_core::geometry::NurbsCurve { domain, seam: None, .. }) => {
                polyline_display_steps(segment)?;
                let evaluator = CurveEvaluator::capture(segment, limits)?;
                let reversed = matches!(segment, Curve::Nurbs(n) if n.reversed);
                (
                    evaluator.evaluate(Parameter::Native(domain[usize::from(reversed)]), 0, Transform::identity())?[0],
                    evaluator.evaluate(Parameter::Native(domain[usize::from(!reversed)]), 0, Transform::identity())?[0],
                    true,
                )
            }
            Curve::Nurbs(n) if n.seam.is_some() => {
                polyline_display_steps(segment)?;
                validate_display_seam(segment, limits)?;
                let p = om9_core::curve_ops::point_at(segment, 0., limits)?;
                (p, p, true)
            }
            _ => {
                return Err(Error::Unsupported(
                    "polycurve display requires direct line, arc or continuous NURBS segments",
                ));
            }
        };
        if previous_end.is_some_and(|(previous, previous_arc): (Point3, bool)| {
            if !arc && !previous_arc {
                previous != start
            } else {
                (0..3).any(|axis| {
                    (previous[axis] - start[axis]).abs()
                        > 64.0 * f64::EPSILON * previous[axis].abs().max(start[axis].abs()).max(1.0)
                })
            }
        }) {
            return Err(Error::Unsupported(
                "polycurve display requires connected line segments",
            ));
        }
        previous_end = Some((end, arc));
    }
    Ok(segments)
}

fn walk_nurbs_spans(n: &om9_core::geometry::NurbsCurve, reverse: bool, visit: &mut impl FnMut(f64, f64) -> Result<()>) -> Result<()> {
    let [min, max] = n.domain;
    let seam = n.seam.map(|s| if s == max { min } else { s });
    let intervals = match (seam, reverse) {
        (None, _) => [(min, max), (min, min)],
        (Some(s), false) => [(s, max), (min, s)],
        (Some(s), true) => [(min, s), (s, max)],
    };
    for (from, to) in intervals {
        for j in 0..n.knots.len()-1 {
            let i = if reverse { n.knots.len()-2-j } else { j };
            let low = n.knots[i].max(from);
            let high = n.knots[i+1].min(to);
            if low < high { if reverse { visit(high, low)?; } else { visit(low, high)?; } }
        }
    }
    Ok(())
}

fn validate_display_seam(curve: &Curve, limits: &Limits) -> Result<()> {
    let Curve::Nurbs(n) = curve else { return Ok(()); };
    if n.seam.is_none() { return Ok(()); }
    let evaluator = CurveEvaluator::capture(curve, limits)?;
    let a = evaluator.evaluate(Parameter::Native(n.domain[0]), 0, Transform::identity())?[0];
    let b = evaluator.evaluate(Parameter::Native(n.domain[1]), 0, Transform::identity())?[0];
    if (0..3).any(|axis| (a[axis]-b[axis]).abs() > 64. * f64::EPSILON * a[axis].abs().max(b[axis].abs()).max(1.)) {
        return Err(Error::Unsupported("reseamed display requires closed NURBS"));
    }
    Ok(())
}

fn polyline_display_steps(curve: &Curve) -> Result<usize> {
    match curve {
        Curve::Line { .. } => Ok(1),
        Curve::Circle {
            radius,
            domain,
            angle_domain,
            ..
        } => {
            let angles = angle_domain.unwrap_or(*domain);
            let angle = 2.0 * (1.0 - (0.005 / radius).min(1.0)).acos();
            let estimate = ((angles[1] - angles[0]) / angle).ceil().max(16.0);
            if !estimate.is_finite() || estimate > 65535.0 {
                return Err(Error::Budget("circle display chord samples"));
            }
            Ok(estimate as usize)
        }
        Curve::Nurbs(n) => {
            // A fully repeated internal knot can be discontinuous; a single
            // display strip would invent a connector across such a break.
            let mut i = 0;
            while i < n.knots.len() {
                let knot = n.knots[i];
                let mut end = i + 1;
                while end < n.knots.len() && n.knots[end] == knot { end += 1; }
                if knot > n.domain[0] && knot < n.domain[1] && end - i > n.degree {
                    return Err(Error::Unsupported("discontinuous polycurve NURBS segment"));
                }
                i = end;
            }
            let mut spans = 0usize;
            walk_nurbs_spans(n, false, &mut |_, _| { spans = spans.checked_add(1).ok_or(Error::Overflow("NURBS display spans"))?; Ok(()) })?;
            spans.checked_mul(256).ok_or(Error::Overflow("NURBS display samples"))
        }
        _ => Err(Error::Unsupported("polycurve display segment")),
    }
}

fn polyline_display_count(segments: &[Curve]) -> Result<usize> {
    segments.iter().try_fold(1usize, |count, segment| {
        count
            .checked_add(polyline_display_steps(segment)?)
            .ok_or(Error::Overflow("polyline display vertices"))
    })
}

fn append_display_segment(segment: &Curve, reverse: bool, points: &mut Vec<Point3>, limits: &Limits) -> Result<()> {
    if let Curve::Line { start, end, .. } = segment {
        let (start, end) = if reverse { (*end, *start) } else { (*start, *end) };
        if points.is_empty() { points.push(start); }
        points.push(end);
    } else if let Curve::Nurbs(n) = segment {
        let evaluator = CurveEvaluator::capture(segment, limits)?;
        let reverse = reverse ^ n.reversed;
        walk_nurbs_spans(n, reverse, &mut |start, end| {
            if points.is_empty() { points.push(evaluator.evaluate(Parameter::Native(start), 0, Transform::identity())?[0]); }
            for i in 1..=256 {
                let t = if i == 256 { end } else { start + (end-start)*i as f64/256. };
                points.push(evaluator.evaluate(Parameter::Native(t), 0, Transform::identity())?[0]);
            }
            Ok(())
        })?;
    } else if let Curve::Circle { domain, .. } = segment {
        let evaluator = CurveEvaluator::capture(segment, limits)?;
        let steps = polyline_display_steps(segment)?;
        let (start, end) = if reverse { (domain[1], domain[0]) } else { (domain[0], domain[1]) };
        if points.is_empty() { points.push(evaluator.evaluate(Parameter::Native(start), 0, Transform::identity())?[0]); }
        for i in 1..=steps {
            let t = if i == steps { end } else { start + (end-start)*i as f64/steps as f64 };
            points.push(evaluator.evaluate(Parameter::Native(t), 0, Transform::identity())?[0]);
        }
    } else {
        return Err(Error::Unsupported("polycurve display segment"));
    }
    Ok(())
}
/// Uniform display samples for a NURBS curve: 256, or 24 per span for curves with many
/// spans (a spiral or helix of many turns would look faceted with 256), at most 16384.
fn nurbs_display_segments(c: &om9_core::geometry::NurbsCurve) -> usize {
    if c.degree <= 1 {
        return 256;
    }
    let mut spans = 1usize;
    let inside = c.knots.iter().filter(|t| **t > c.domain[0] && **t < c.domain[1]);
    let mut last = f64::NAN;
    for &t in inside {
        if t != last {
            spans += 1;
            last = t;
        }
    }
    spans.saturating_mul(24).clamp(256, 16384)
}

fn brep_buffers(d: &om9_core::brep::BrepDisplay, usage: &mut Usage, l: &Limits) -> Result<RenderBuffers> {
            usage.add(
                Usage {
                    points: d.points.len() as u64,
                    ..Usage::array::<Point3>(d.points.len())?
                },
                l,
            )?;
            usage.add(Usage::array::<([u32; 3], u32)>(d.triangles.len())?, l)?;
            usage.add(Usage::array::<[u32; 2]>(d.segments.len())?, l)?;
            let mut vertices = display_vector(d.points.len(), "BRep display vertices")?;
            let mut triangles = display_vector(d.triangles.len(), "BRep display triangles")?;
            let mut edges = display_vector(d.segments.len(), "BRep display edges")?;
            vertices.extend_from_slice(&d.points);
            triangles.extend(d.triangles.iter().map(|[a, b, c, f]| ([*a, *b, *c], *f)));
            edges.extend(d.segments.iter().map(|[a, b, _]| [*a, *b]));
            Ok(RenderBuffers::Mesh { vertices, triangles, edges })
}
fn cache(g: &Geometry, usage: &mut Usage, l: &Limits, representations: &BrepRepresentationStore) -> Result<RenderBuffers> {
    match g {
        Geometry::Point(p) => {
            usage.add(
                Usage {
                    points: 1,
                    ..Usage::array::<Point3>(1)?
                },
                l,
            )?;
            Ok(RenderBuffers::Points {
                points: vec![*p],
                native_indices: vec![0],
            })
        }
        Geometry::Cloud(c) => {
            let count = c.points.len();
            usage.add(
                Usage {
                    points: count as u64,
                    ..Usage::array::<(Point3, u32)>(count)?
                },
                l,
            )?;
            let visible = c
                .points
                .iter()
                .enumerate()
                .filter(|(i, _)| !c.hidden.get(*i).copied().unwrap_or(false))
                .count();
            let mut points = Vec::new();
            let mut native_indices = Vec::new();
            points
                .try_reserve_exact(visible)
                .map_err(|_| Error::Budget("cloud display allocation"))?;
            native_indices
                .try_reserve_exact(visible)
                .map_err(|_| Error::Budget("cloud index allocation"))?;
            for (i, p) in c.points.iter().enumerate() {
                if c.hidden.get(i).copied().unwrap_or(false) {
                    continue;
                }
                points.push(*p);
                native_indices
                    .push(u32::try_from(i).map_err(|_| Error::Overflow("cloud native index"))?);
            }
            Ok(RenderBuffers::Points {
                points,
                native_indices,
            })
        }
        Geometry::Curve(curve @ Curve::Polycurve { .. }) => {
            if has_nested_segments(curve) {
                let count = nested_display_count(curve, l)?;
                usage.add(Usage { points: count as u64, ..Usage::array::<Point3>(count)? }, l)?;
                let mut points = Vec::new();
                points.try_reserve_exact(count).map_err(|_| Error::Budget("nested polycurve display allocation"))?;
                walk_nested_segments(curve, false, &mut |segment, reverse| {
                    append_display_segment(segment, reverse, &mut points, l)
                })?;
                return Ok(RenderBuffers::Curve { points, parameters: Vec::new() });
            }
            // An annotation stores independent line strokes. Render each stroke
            // explicitly; joining gaps would introduce lines that are not in the model.
            if let Curve::Polycurve{segments,reversed:false}=curve
                && connected_polyline_segments(curve,l).is_err()&&segments.iter().all(|s|matches!(s,Curve::Line{..})) {
                let count=segments.len().checked_mul(2).ok_or(Error::Overflow("line stroke vertices"))?;
                usage.add(Usage{points:count as u64,..Usage::array::<Point3>(count)?},l)?;
                usage.add(Usage::array::<[u32;2]>(segments.len())?,l)?;
                let mut vertices=display_vector(count, "stroke display allocation")?;
                let mut edges=display_vector(segments.len(), "stroke edge allocation")?;
                for segment in segments {if let Curve::Line{start,end,..}=segment{let i=u32::try_from(vertices.len()).map_err(|_|Error::Overflow("stroke vertex index"))?;vertices.extend([*start,*end]);edges.push([i,i+1]);}}
                return Ok(RenderBuffers::Mesh{vertices,triangles:vec![],edges});
            }
            let segments = connected_polyline_segments(curve, l)?;
            let count = polyline_display_count(segments)?;
            usage.add(
                Usage {
                    points: count as u64,
                    ..Usage::array::<Point3>(count)?
                },
                l,
            )?;
            let mut points = Vec::new();
            points
                .try_reserve_exact(count)
                .map_err(|_| Error::Budget("polyline display allocation"))?;
            for segment in segments {
                append_display_segment(segment, false, &mut points, l)?;
            }
            if matches!(curve, Curve::Polycurve { reversed: true, .. }) { points.reverse(); }
            // Exact corners, not a uniform sampling approximation. A polycurve
            // has no global native domain; picking returns whole-object identity.
            Ok(RenderBuffers::Curve {
                points,
                parameters: Vec::new(),
            })
        }
        Geometry::Curve(c) => {
            let (domain, segments) = match c {
                Curve::Line { domain, .. } => (*domain, 1usize),
                Curve::Circle {
                    domain,
                    radius,
                    angle_domain,
                    ..
                } => {
                    let chord = 0.005;
                    let angle = 2. * (1. - (chord / radius).min(1.)).acos();
                    let angles = angle_domain.unwrap_or(*domain);
                    let estimate = ((angles[1] - angles[0]) / angle).ceil().max(16.);
                    if !estimate.is_finite() || estimate > 65535. {
                        return Err(Error::Budget("circle display chord samples"));
                    }
                    (*domain, estimate as usize)
                }
                Curve::Nurbs(c) if !c.periodic => (c.domain, nurbs_display_segments(c)),
                Curve::Nurbs(c) if c.periodic => {c.certify_repeated_periodic_basis(l)?;(c.domain,nurbs_display_segments(c))},
                _ => return Err(Error::Unsupported("curve display family/segment binding")),
            };
            // NURBS kinks sit on knots (a polyline's corners, a joined curve's joints):
            // sample every distinct interior knot too, or uniform samples cut the corners.
            let knot_params: Vec<f64> = match c {
                Curve::Nurbs(n) => {
                    let mut k: Vec<f64> = n
                        .knots
                        .iter()
                        .copied()
                        .filter(|t| *t > domain[0] && *t < domain[1])
                        .collect();
                    k.dedup_by(|a, b| (*a - *b).abs() <= 1e-12 * (domain[1] - domain[0]).abs().max(1.));
                    if k.len() > 65535 {
                        return Err(Error::Budget("curve display knot samples"));
                    }
                    k
                }
                _ => Vec::new(),
            };
            let count = segments
                .checked_add(1)
                .and_then(|c| c.checked_add(knot_params.len()))
                .ok_or(Error::Overflow("display samples"))?;
            usage.add(
                Usage {
                    points: count as u64,
                    ..Usage::array::<(Point3, f64)>(count)?
                },
                l,
            )?;
            let mut points = display_vector(count, "curve display allocation")?;
            let mut parameters = display_vector(count, "curve parameter allocation")?;
            let evaluator = CurveEvaluator::capture(c, l)?;
            // Uniform parameters merged with the knots, increasing.
            let mut ts: Vec<f64> = (0..=segments)
                .map(|i| {
                    if i == segments {
                        domain[1]
                    } else {
                        domain[0] + (domain[1] - domain[0]) * (i as f64 / segments as f64)
                    }
                })
                .collect();
            if !knot_params.is_empty() {
                ts.extend(&knot_params);
                ts.sort_by(f64::total_cmp);
                let span = (domain[1] - domain[0]).abs().max(1.);
                ts.dedup_by(|a, b| (*a - *b).abs() <= 1e-12 * span);
            }
            for (i, &t) in ts.iter().enumerate() {
                let fraction = if knot_params.is_empty() { i as f64 / segments as f64 } else { (t - domain[0]) / (domain[1] - domain[0]) };
                if matches!(
                    c,
                    Curve::Circle {
                        angle_domain: Some(_),
                        ..
                    }
                ) {
                    let recovered = (t - domain[0]) / (domain[1] - domain[0]);
                    // Native f64 bindings must reproduce the displayed phase.
                    // A valid but unresolvable durable map stays owned, with a
                    // visible diagnostic instead of a collapsed display proxy.
                    if parameters.last().is_some_and(|previous| t <= *previous)
                        || !recovered.is_finite()
                        || (recovered - fraction).abs() > 64. * f64::EPSILON
                    {
                        return Err(Error::Unsupported(
                            "circle display native parameter precision",
                        ));
                    }
                }
                points.push(evaluator.evaluate(Parameter::Native(t), 0, Transform::identity())?[0]);
                parameters.push(t);
            }
            Ok(RenderBuffers::Curve { points, parameters })
        }
        Geometry::Brep(shape) => brep_buffers(representations.display_for(shape)?, usage, l),
        Geometry::Mesh(m) => mesh_buffers(m, usage, l),
        Geometry::Hatch(h) => mesh_buffers(&h.display, usage, l),
        Geometry::Light(light) => cache(&Geometry::Point(light.display), usage, l, representations),
        Geometry::NativeBrep(brep) => mesh_buffers(representations.get_native(brep)?.display(), usage, l),
        Geometry::Surface(surface) => surface_buffers(surface, usage, l, false),
        _ => Err(Error::Unsupported("representation display adapter")),
    }
}
fn cache_entity(entity: &Entity, usage: &mut Usage, limits: &Limits, representations: &BrepRepresentationStore) -> Result<(RenderBuffers,bool)> {
    let mut fallback=false;
    if matches!(&entity.geometry, Geometry::Surface(_)) {
        match trim_decomposition::decode(entity,limits) {
            Ok(Some(trim)) => {
                let before=*usage;
                match trim_decomposition::surface_trim_buffers(&trim,usage,limits) {
                    Ok(buffers) => return Ok((buffers,false)),
                    Err(Error::Invalid(_) | Error::Unsupported(_)) => {*usage=before;fallback=true;},
                    Err(error) => return Err(error),
                }
            }
            Ok(None) => {
                if let Geometry::Surface(surface) = &entity.geometry {
                    return Ok((surface_buffers(surface, usage, limits, true)?, false));
                }
            },
            Err(Error::Invalid(_) | Error::Unsupported(_)) => {fallback=true;},
            Err(error) => return Err(error),
        }
    }
    let mut buffers = cache(&entity.geometry, usage, limits, representations)?;
    if let (Geometry::Mesh(mesh), RenderBuffers::Mesh { edges, .. }) = (&entity.geometry, &mut buffers)
        && entity.extensions.contains_key(om9_core::model::GEM_EXTENSION_SLOT)
    {
        edges.retain(|&[a, b]| !om9_core::gem::gem_tessellation_edge(mesh, a, b));
    }
    Ok((buffers, fallback))
}
/// Tessellate a NURBS surface for display and picking only. The exact surface
/// remains the document geometry; this grid is never written back.
const SURFACE_SAMPLES: usize = 12;
fn surface_buffer_usage(usage: &mut Usage, l: &Limits) -> Result<()> {
    let vertices = SURFACE_SAMPLES * SURFACE_SAMPLES;
    usage.add(Usage { points: vertices as u64, ..Usage::array::<Point3>(vertices)? }, l)?;
    usage.add(Usage::array::<([u32; 3], u32)>(2 * (SURFACE_SAMPLES - 1) * (SURFACE_SAMPLES - 1))?, l)?;
    usage.add(Usage::array::<[u32; 2]>(4 * (SURFACE_SAMPLES - 1))?, l)
}
fn surface_buffer_usage_for(surface: &om9_core::geometry::NurbsSurface, interior: bool, usage: &mut Usage, l: &Limits) -> Result<(usize, usize)> {
    let wires = if interior {
        surface.validate(l)?;
        om9_core::surface_kernel::default_wire_parameters(surface, 0)?.count()
            .checked_add(om9_core::surface_kernel::default_wire_parameters(surface, 1)?.count())
            .ok_or(Error::Overflow("surface display wires"))?
    } else { 0 };
    let vertices = wires.checked_mul(SURFACE_SAMPLES).and_then(|n| n.checked_add(SURFACE_SAMPLES * SURFACE_SAMPLES))
        .ok_or(Error::Overflow("surface display vertices"))?;
    let triangles = 2 * (SURFACE_SAMPLES - 1) * (SURFACE_SAMPLES - 1);
    let edges = wires.checked_add(4).and_then(|n| n.checked_mul(SURFACE_SAMPLES - 1))
        .ok_or(Error::Overflow("surface display edges"))?;
    u32::try_from(vertices).map_err(|_| Error::Overflow("surface display indices"))?;
    usage.add(Usage { points: vertices as u64, ..Usage::array::<Point3>(vertices)? }, l)?;
    usage.add(Usage::array::<([u32; 3], u32)>(triangles)?, l)?;
    usage.add(Usage::array::<[u32; 2]>(edges)?, l)?;
    Ok((vertices, edges))
}
fn surface_buffers(
    surface: &om9_core::geometry::NurbsSurface,
    usage: &mut Usage,
    l: &Limits,
    interior: bool,
) -> Result<RenderBuffers> {
    const SAMPLES: usize = SURFACE_SAMPLES;
    let (vertex_count, edge_count) = surface_buffer_usage_for(surface, interior, usage, l)?;
    let evaluate = om9_core::surface_kernel::point_evaluator(surface, l)?;
    let [u0, u1] = surface.domain[0];
    let [v0, v1] = surface.domain[1];
    let mut vertices = display_vector(vertex_count, "surface display vertices")?;
    for i in 0..SAMPLES {
        let u = u0 + (u1 - u0) * i as f64 / (SAMPLES - 1) as f64;
        for j in 0..SAMPLES {
            let v = v0 + (v1 - v0) * j as f64 / (SAMPLES - 1) as f64;
            vertices.push(evaluate(u, v)?);
        }
    }
    let mut triangles = display_vector(2 * (SAMPLES - 1) * (SAMPLES - 1), "surface display triangles")?;
    for i in 0..SAMPLES - 1 {
        for j in 0..SAMPLES - 1 {
            let a = (i * SAMPLES + j) as u32;
            let b = ((i + 1) * SAMPLES + j) as u32;
            let c = ((i + 1) * SAMPLES + j + 1) as u32;
            let d = (i * SAMPLES + j + 1) as u32;
            triangles.push(([a, b, c], 0));
            triangles.push(([a, c, d], 0));
        }
    }
    let mut edges = display_vector(edge_count, "surface display edges")?;
    for i in 0..SAMPLES - 1 {
        edges.push([(i * SAMPLES) as u32, ((i + 1) * SAMPLES) as u32]);
        edges.push([
            (i * SAMPLES + SAMPLES - 1) as u32,
            ((i + 1) * SAMPLES + SAMPLES - 1) as u32,
        ]);
    }
    for j in 0..SAMPLES - 1 {
        edges.push([j as u32, (j + 1) as u32]);
        edges.push([
            ((SAMPLES - 1) * SAMPLES + j) as u32,
            ((SAMPLES - 1) * SAMPLES + j + 1) as u32,
        ]);
    }
    if interior {
        for axis in 0..2 {
            for fixed in om9_core::surface_kernel::default_wire_parameters(surface, axis)? {
                let first = vertices.len();
                let [start, end] = surface.domain[1 - axis];
                for i in 0..SAMPLES {
                    let varying = start + (end - start) * i as f64 / (SAMPLES - 1) as f64;
                    let (u, v) = if axis == 0 { (fixed, varying) } else { (varying, fixed) };
                    vertices.push(evaluate(u, v)?);
                    if i > 0 { edges.push([(first + i - 1) as u32, (first + i) as u32]); }
                }
            }
        }
    }
    Ok(RenderBuffers::Mesh {
        vertices,
        triangles,
        edges,
    })
}
fn mesh_buffer_usage(m: &om9_core::geometry::Mesh, usage: &mut Usage, l: &Limits) -> Result<(usize, usize)> {
    let triangles = m.faces.iter().try_fold(0usize, |n, f| {
        let count = f.len().checked_sub(2).ok_or(Error::Invalid("mesh face"))?;
        n.checked_add(count).ok_or(Error::Overflow("mesh triangles"))
    })?;
    let edges = m.faces.iter().try_fold(0usize, |n, f| {
        n.checked_add(f.len()).ok_or(Error::Overflow("mesh edges"))
    })?;
    usage.add(Usage { points: m.vertices.len() as u64, ..Usage::array::<Point3>(m.vertices.len())? }, l)?;
    usage.add(Usage::array::<([u32; 3], u32)>(triangles)?, l)?;
    usage.add(Usage::array::<[u32; 2]>(edges)?, l)?;
    Ok((triangles, edges))
}
fn mesh_buffers(
    m: &om9_core::geometry::Mesh,
    usage: &mut Usage,
    l: &Limits,
) -> Result<RenderBuffers> {
    let (triangles, edges) = mesh_buffer_usage(m, usage, l)?;
    let mut tris = display_vector(triangles, "mesh display triangles")?;
    let mut wire = display_vector(edges, "mesh display edges")?;
    let mut vertices = display_vector(m.vertices.len(), "mesh display vertices")?;
    vertices.extend_from_slice(&m.vertices);
    for (face, indices) in m.faces.iter().enumerate() {
        let face = u32::try_from(face).map_err(|_| Error::Overflow("native mesh face index"))?;
        if let [a, b, c, d] = indices.as_slice() {
            let points = [*a, *b, *c, *d].map(|index| m.vertices[index as usize]);
            if matches!(om9_core::geometry::quad_diagonal_ac(points), Ok(false)) {
                tris.push(([*a, *b, *d], face));
                tris.push(([*b, *c, *d], face));
            } else {
                tris.push(([*a, *b, *c], face));
                tris.push(([*a, *c, *d], face));
            }
        } else {
            for i in 1..indices.len() - 1 {
                tris.push(([indices[0], indices[i], indices[i + 1]], face));
            }
        }
        for i in 0..indices.len() {
            wire.push([indices[i], indices[(i + 1) % indices.len()]]);
        }
    }
    Ok(RenderBuffers::Mesh {
        vertices,
        triangles: tris,
        edges: wire,
    })
}

struct Capture<'a> {
    representations: &'a BrepRepresentationStore,
    document: &'a Arc<Document>,
    previous: Option<&'a Scene>,
    previous_index: Vec<usize>,
    epoch: u64,
    limits: &'a Limits,
    usage: Usage,
    objects: Vec<SceneObject>,
    diagnostics: Vec<String>,
    shared: BTreeMap<(ObjectId, u64, u64), SharedBuffer>,
    reused_buffers: std::collections::BTreeSet<usize>,
    buffers_usage: Usage,
    registered_display_bytes: u64,
}
struct SharedBuffer {
    buffers: Arc<BufferPayload>,
    receipt: Option<BrepDisplayReceipt>,
}
// Index only object positions: paths and scene buffers stay owned by the old scene.
fn previous_occurrence_index(previous: Option<&Scene>) -> Result<Vec<usize>> {
    let Some(previous) = previous else { return Ok(Vec::new()); };
    let mut index = Vec::new();
    index.try_reserve_exact(previous.objects.len()).map_err(|_| Error::Budget("scene reuse index"))?;
    if index.capacity() > previous.objects.len() { return Err(Error::Budget("scene reuse index capacity")); }
    index.extend(0..previous.objects.len());
    index.sort_unstable_by_key(|&i| (previous.objects[i].root, previous.objects[i].leaf));
    Ok(index)
}
fn reusable_brep_wire<'a>(previous: &'a Scene, index: &[usize], root: ObjectId, path: &[ObjectId], entity: &Entity,
    representations: &BrepRepresentationStore) -> Result<Option<&'a SceneObject>> {
    reusable_brep_wire_matching(previous, index, root, entity, representations, |candidate| candidate == path)
}
fn reusable_brep_wire_matching<'a>(previous: &'a Scene, index: &[usize], root: ObjectId, entity: &Entity,
    representations: &BrepRepresentationStore, path_matches: impl Fn(&[ObjectId]) -> bool) -> Result<Option<&'a SceneObject>> {
    let Geometry::Brep(source) = &entity.geometry else { return Ok(None); };
    let key = (root, entity.id);
    let start = index.partition_point(|&i| (previous.objects[i].root, previous.objects[i].leaf) < key);
    let mut candidates = index[start..].iter().map(|&i| &previous.objects[i])
        .take_while(|old| (old.root, old.leaf) == key).filter(|old| path_matches(&old.path));
    let Some(old) = candidates.next() else { return Ok(None); };
    let unique = candidates.next().is_none();
    let old_source = previous.entity_for(&previous.document, old)?;
    let matches_source = matches!(&old_source.geometry, Geometry::Brep(value) if value.identity() == source.identity());
    let matches_display = old.brep_pick_display.as_ref().is_some_and(|pin|
        representations.get(source).is_ok_and(|display| std::ptr::eq(pin.display(), display.display())));
    if unique && matches_source && matches_display && old.registered_display_bytes != 0
        && old.registered_display_bytes == buffer_usage(old.buffers.as_ref())?.bytes
        && previous.brep_occurrence_has_interior_wires(old)? { Ok(Some(old)) } else { Ok(None) }
}
impl SharedBuffer {
    fn get(&self, receipt: &Option<BrepDisplayReceipt>) -> Result<Arc<BufferPayload>> {
        if &self.receipt != receipt {
            return Err(Error::Stale("shared BRep display source"));
        }
        Ok(self.buffers.clone())
    }
}
impl Capture<'_> {
    #[allow(clippy::too_many_arguments)] // All inherited occurrence state travels together.
    fn entity(
        &mut self,
        root: ObjectId,
        path: Vec<ObjectId>,
        entity: &Entity,
        transform: Transform,
        permissions: Permissions,
        reference_unlocked: bool,
        depth: usize,
    ) -> Result<()> {
        if depth > self.limits.max_depth {
            return Err(Error::Budget("scene instance depth"));
        }
        self.usage.add(
            Usage {
                nodes: 1,
                ..Usage::default()
            },
            self.limits,
        )?;
        let local = self.document.permissions_for_entity(entity, true)?;
        let permissions = combined(permissions, local);
        let reference_unlocked = reference_unlocked
            && self
                .document
                .permissions_for_entity(entity, false)?
                .reference_snappable;
        let transform = transform.compose(entity.placement)?;
        if let Geometry::Instance(id) = entity.geometry {
            let definition = self
                .document
                .definitions
                .get(&id)
                .ok_or(Error::Missing("scene definition"))?;
            for member in &definition.members {
                let mut p = path.clone();
                p.push(member.id);
                self.entity(
                    root,
                    p,
                    member,
                    transform,
                    permissions,
                    reference_unlocked,
                    depth + 1,
                )?;
            }
            return Ok(());
        }
        self.usage
            .add(Usage::array::<SceneObject>(1)?, self.limits)?;
        let key = (entity.id, entity.generation, entity.geometry_revision);
        let receipt = brep_display_fingerprint(entity);
        let reused = match self.previous {
            Some(previous) => reusable_brep_wire(previous, &self.previous_index, root, &path, entity, self.representations)?,
            None => None,
        };
        let buffers = if let Some(old) = reused {
            if self.reused_buffers.insert(Arc::as_ptr(&old.buffers.inner) as usize) {
                let actual = buffer_usage(old.buffers.as_ref())?;
                // The previous scene's independent owner is already charged.
                // Count this allocation once even when occurrences share it.
                self.usage.add(Usage { points: actual.points, ..Usage::default() }, self.limits)?;
                self.buffers_usage.add(actual, self.limits)?;
                self.registered_display_bytes = self.registered_display_bytes.checked_add(old.registered_display_bytes)
                    .ok_or(Error::Overflow("reused BRep display accounting"))?;
            }
            old.buffers.inner.clone()
        } else if let Some(buffers) = self.shared.get(&key) {
            buffers.get(&receipt)?
        } else {
            let previous_bytes = self.usage.bytes;
            match cache_entity(entity, &mut self.usage, self.limits, self.representations) {
                Ok((buffers,fallback)) => {
                    if fallback {self.diagnostics.push(format!("{}: trim display unsupported; showing underlying surface",entity.id));}
                    let actual = charge_buffer_capacity(&mut self.usage, previous_bytes, &buffers, self.limits)?;
                    self.buffers_usage
                        .add(actual, self.limits)?;
                    let bounds = OnceLock::new();
                    let _ = bounds.set(display_bounds(&buffers));
                    let buffers = Arc::new(BufferPayload { data: buffers, bounds, lease: OnceLock::new(), trim_fallback: fallback, interior_wires_prepared: false });
                    self.shared.insert(key, SharedBuffer { buffers: buffers.clone(), receipt: receipt.clone() });
                    buffers
                }
                Err(Error::Unsupported(reason)) => {
                    self.diagnostics.push(format!("{}: {reason}", entity.id));
                    return Ok(());
                }
                Err(error) => return Err(error),
            }
        };
        let color = entity
            .attributes
            .color
            .or_else(|| {
                entity
                    .attributes
                    .layer
                    .and_then(|id| self.document.layers.get(&id).map(|l| l.color))
            })
            .unwrap_or([220, 220, 220, 255]);
        self.objects.push(SceneObject {
            root,
            path,
            leaf: entity.id,
            leaf_generation: entity.generation,
            leaf_geometry_revision: entity.geometry_revision,
            brep_display_fingerprint: receipt,
            brep_pick_display: match (reused, &entity.geometry) {
                (Some(old), Geometry::Brep(_)) => old.brep_pick_display.clone(),
                (_, Geometry::Brep(source)) => Some(self.representations.pin_for(source, Arc::new(()))?), _ => None,
            },
            native_display_pin: match &entity.geometry {
                Geometry::NativeBrep(native) => Some(self.representations.get_native(native)?.pin()?),
                _ => None,
            },
            source: self.document.stamp(root)?,
            transform,
            preview: Transform::identity(),
            color,
            buffers: SceneBuffers::new(buffers),
            visible: permissions.visible,
            selectable: permissions.selectable,
            editable: permissions.editable,
            reference_snappable: permissions.reference_snappable,
            reference_unlocked,
            brep_output_deflection: match (reused, &entity.geometry) {
                (Some(old), Geometry::Brep(_)) => old.brep_output_deflection,
                (_, Geometry::Brep(brep)) => Some(self.representations.display_for(brep)?.deflection_mm), _ => None },
            remesh_profile: reused.map_or([0; 2], |old| old.remesh_profile),
            registered_display_bytes: reused.map_or(0, |old| old.registered_display_bytes),
        });
        Ok(())
    }
}
/// An attested remesh representation without a document owner. Clones retain
/// both display and buffer accounting leases. A caller must independently
/// verify the native topology proof and current source before installing it.
#[derive(Clone)]
pub struct RootBrepRepresentation {
    stamp: SourceStamp,
    profile: [u64; 2],
    display: om9_core::brep::BrepDisplayPin,
    buffers: SceneBuffers,
    buffer_bytes: u64,
}
impl RootBrepRepresentation {
    pub fn stamp(&self) -> SourceStamp { self.stamp }
    pub fn profile(&self) -> [u64; 2] { self.profile }
    pub fn buffer_bytes(&self) -> u64 { self.buffer_bytes }
    /// Retarget metadata only after the caller has revalidated the same native
    /// source/topology proof against the new document. No topology is inferred
    /// from these identity fields alone.
    #[doc(hidden)]
    pub fn retarget_same_object(&mut self, stamp: SourceStamp) -> Result<()> {
        let old=self.stamp;
        if old.document!=stamp.document || old.owner_session!=stamp.owner_session
            || old.session_generation!=stamp.session_generation || old.object!=stamp.object
            || old.generation!=stamp.generation {
            return Err(Error::Stale("remesh representation object"));
        }
        self.stamp=stamp;Ok(())
    }
    pub fn display_bytes(&self) -> Result<u64> { self.display.storage_bytes() }
    #[doc(hidden)]
    pub fn display_owner<T: std::any::Any>(&self) -> Option<&T> { self.display.owner::<T>() }
    #[doc(hidden)]
    pub fn topology_is_exclusive(&self) -> bool { self.display.topology_is_exclusive() }
    /// The owner must hold its cache lock so no new clone can race this test.
    pub fn is_exclusive(&self) -> bool {
        self.display.is_exclusive() && Arc::strong_count(&self.buffers.inner) == 1 && Arc::weak_count(&self.buffers.inner) == 0
            && self.buffers.inner.lease.get().is_some_and(|lease| Arc::strong_count(&lease.0) == 1)
    }
}
impl Scene {
    pub fn publication_id(&self) -> uuid::Uuid { self.publication_id }

    /// Only the visible owners of enabled control/edit points can be picked
    /// during PtOffSelected. Use the published geometry and its permissions.
    pub fn grip_owner_selection_snapshot(&self, owns_points: impl Fn(ObjectId) -> bool) -> Result<Self> {
        let mut snapshot = self.try_render_snapshot()?;
        for object in &mut snapshot.objects {
            object.visible &= object.selectable && owns_points(object.root);
            object.selectable &= object.visible;
            object.editable = false;
            object.reference_snappable = false;
            object.reference_unlocked = false;
        }
        snapshot.publication_id = uuid::Uuid::new_v4();
        Ok(snapshot)
    }

    /// A private picking scene for ShowSelected. Only the root's Hidden bit is
    /// lifted. Inherited visibility, locks and member permissions still apply.
    pub fn hidden_selection_snapshot(&self) -> Result<Self> {
        let mut snapshot = self.try_render_snapshot()?;
        for object in &mut snapshot.objects {
            let mut entity = self.document.resolve(&object.source)?;
            let attrs = &entity.attributes;
            let layer = self.document.layer_permissions(attrs.layer, false)?;
            let mut visible = attrs.hidden && attrs.visible && layer.visible;
            let mut selectable = visible && attrs.selectable && !attrs.locked && layer.selectable;
            for id in &object.path {
                let Geometry::Instance(definition) = entity.geometry else {
                    return Err(Error::Stale("hidden selection occurrence"));
                };
                entity = self.document.definitions.get(&definition)
                    .and_then(|d| d.members.iter().find(|m| m.id == *id))
                    .ok_or(Error::Stale("hidden selection member"))?;
                let member = self.document.permissions_for_entity(entity, false)?;
                visible &= member.visible;
                selectable &= member.selectable;
            }
            object.visible = visible;
            object.selectable = selectable;
            object.editable = false;
            object.reference_snappable = false;
            object.reference_unlocked = false;
        }
        snapshot.publication_id = uuid::Uuid::new_v4();
        Ok(snapshot)
    }
    fn shared_buffer_is_consistent(&self, index: usize) -> Result<bool> {
        let object = self.objects.get(index).ok_or(Error::Invalid("scene buffer index"))?;
        let mut shared = false;
        for (other_index, other) in self.objects.iter().enumerate() {
            if other_index != index && object.buffers.ptr_eq(&other.buffers) {
                if other.registered_display_bytes != object.registered_display_bytes {
                    return Err(Error::Invalid("shared scene buffer accounting"));
                }
                shared = true;
            }
        }
        Ok(shared)
    }
    /// Capture only a remesh override, never the document or its v4 display.
    #[doc(hidden)]
    pub fn root_brep_representation(&self, stamp: &SourceStamp) -> Result<RootBrepRepresentation> {
        self.root_brep_pick_display(stamp)?;
        let object = self.objects.iter().find(|object| object.root == stamp.object && object.path.is_empty())
            .ok_or(Error::Missing("scene BRep root"))?;
        let display = object.brep_pick_display.clone().ok_or(Error::Missing("remesh representation"))?;
        if object.registered_display_bytes == 0 { return Err(Error::Invalid("remesh representation accounting")); }
        Ok(RootBrepRepresentation { stamp: *stamp, profile: object.remesh_profile, display,
            buffers: object.buffers.clone(), buffer_bytes: object.registered_display_bytes })
    }

    /// The caller must validate the proof, source stamp, and memory admission.
    #[doc(hidden)]
    pub fn try_replace_root_brep_representation(&self, representation: &RootBrepRepresentation) -> Result<Self> {
        let stamp = &representation.stamp;
        let entity = self.document.resolve(stamp)?;
        if !matches!(entity.geometry, Geometry::Brep(_)) { return Err(Error::Stale("remesh root source")); }
        let mut found = self.objects.iter().enumerate().filter(|(_, object)| object.root == stamp.object && object.path.is_empty());
        let (index, object) = found.next().ok_or(Error::Missing("remesh root scene object"))?;
        if found.next().is_some() || object.leaf != stamp.object || object.source != *stamp {
            return Err(Error::Stale("remesh root scene object"));
        }
        self.entity_for(&self.document, object)?;
        let old_is_shared = self.shared_buffer_is_consistent(index)?;
        let old = buffer_usage(object.buffers.as_ref())?;
        let new = buffer_usage(representation.buffers.as_ref())?;
        if new.bytes != representation.buffer_bytes { return Err(Error::Invalid("remesh representation accounting")); }
        let mut new_already_present = false;
        for (i, other) in self.objects.iter().enumerate() {
            if i != index && other.buffers.ptr_eq(&representation.buffers) {
                if other.registered_display_bytes != new.bytes { return Err(Error::Invalid("shared remesh display accounting")); }
                new_already_present = true;
            }
        }
        let mut totals = self.buffers_usage;
        if !old_is_shared {
            totals.bytes = totals.bytes.checked_sub(old.bytes).ok_or(Error::Invalid("remesh buffer accounting"))?;
            totals.points = totals.points.checked_sub(old.points).ok_or(Error::Invalid("remesh point accounting"))?;
        }
        if !new_already_present { totals.add(new, &self.limits)?; }
        let mut scene = self.try_render_snapshot()?;
        scene.objects[index].buffers = representation.buffers.clone();
        scene.objects[index].brep_pick_display = Some(representation.display.clone());
        scene.objects[index].remesh_profile = representation.profile;
        scene.buffers_usage = totals;
        let released = if old_is_shared { 0 } else { object.registered_display_bytes };
        let added = if new_already_present { 0 } else { new.bytes };
        scene.registered_display_bytes = self.registered_display_bytes.checked_sub(released)
            .and_then(|bytes| bytes.checked_add(added)).ok_or(Error::Overflow("remesh display accounting"))?;
        scene.objects[index].registered_display_bytes = new.bytes;
        scene.publication_id = uuid::Uuid::new_v4();
        Ok(scene)
    }
    /// Borrow the display actually published for a root B-rep after validating
    /// its source stamp and scene receipt. The borrow keeps the scene owner live.
    pub fn root_brep_pick_display(&self, stamp: &SourceStamp) -> Result<&om9_core::brep::BrepDisplay> {
        let mut roots = self.objects.iter().filter(|object| object.root == stamp.object && object.path.is_empty());
        let object = roots.next().ok_or(Error::Missing("scene BRep root"))?;
        if roots.next().is_some() || object.leaf != stamp.object || object.source != *stamp {
            return Err(Error::Stale("scene BRep root"));
        }
        let entity = self.entity_for(&self.document, object)?;
        object.pick_brep_display(entity)?.ok_or(Error::Invalid("scene root is not BRep"))
    }
    /// Inspect the published OCCT occurrence, keeping topology edges distinct
    /// from source-derived interior wires in its render buffers.
    pub fn brep_occurrence_has_interior_wires(&self, object: &SceneObject) -> Result<bool> {
        if !self.objects.iter().any(|entry| std::ptr::eq(entry, object)) {
            return Err(Error::Stale("scene BRep occurrence"));
        }
        let entity = self.entity_for(&self.document, object)?;
        let display = object.pick_brep_display(entity)?.ok_or(Error::Invalid("scene occurrence is not BRep"))?;
        let RenderBuffers::Mesh { vertices, edges, .. } = object.buffers.as_ref() else {
            return Err(Error::Stale("ExtractWireframe BRep buffers"));
        };
        if vertices.get(..display.points.len()) != Some(display.points.as_slice())
            || edges.len() < display.segments.len()
            || !edges.iter().zip(&display.segments).all(|(edge, segment)| *edge == [segment[0], segment[1]])
            || edges[display.segments.len()..].iter().any(|edge| edge.iter().any(|&i| (i as usize) < display.points.len() || i as usize >= vertices.len())) {
            return Err(Error::Stale("ExtractWireframe BRep buffers"));
        }
        Ok(object.buffers.inner.interior_wires_prepared)
    }
    /// A history snapshot is useful only when it can retain a prepared,
    /// independently owned BRep wire buffer.
    pub fn has_owned_brep_wires(&self) -> bool {
        self.objects.iter().any(|object| object.registered_display_bytes != 0
            && object.buffers.inner.interior_wires_prepared && object.brep_pick_display.is_some())
    }
    /// Output profile captured when the source entered this scene. Unlike the
    /// current witness profile, it is stable across a viewport remesh.
    pub fn root_brep_output_deflection(&self, stamp: &SourceStamp) -> Result<f64> {
        self.root_brep_pick_display(stamp)?;
        let object = self.objects.iter().find(|object| object.root == stamp.object && object.path.is_empty())
            .ok_or(Error::Missing("scene BRep root"))?;
        object.brep_output_deflection.ok_or(Error::Invalid("scene BRep output profile"))
    }
    /// Clone the independently accounted display pin, when present.
    /// Unregistered scene captures can still have no pin.
    #[doc(hidden)]
    pub fn root_brep_display_pin(&self, stamp: &SourceStamp) -> Result<Option<om9_core::brep::BrepDisplayPin>> {
        self.root_brep_pick_display(stamp)?;
        let object = self.objects.iter().find(|object| object.root == stamp.object && object.path.is_empty())
            .ok_or(Error::Missing("scene BRep root"))?;
        Ok(object.brep_pick_display.clone())
    }
    /// Check only a currently published root remesh. The caller must also
    /// validate its process-scoped native proof and current document owner.
    #[doc(hidden)]
    pub fn has_root_remesh_profile(&self, stamp: &SourceStamp, profile: [u64; 2]) -> bool {
        let mut roots = self.objects.iter().filter(|object| object.root == stamp.object && object.path.is_empty());
        let Some(object) = roots.next() else { return false; };
        roots.next().is_none() && object.source == *stamp && object.remesh_profile == profile
            && object.registered_display_bytes != 0 && object.brep_pick_display.is_some()
            && self.entity_for(&self.document, object).is_ok()
    }
    /// Bind ordinary shared buffers to an already admitted owner. Existing
    /// bindings stay alive for old snapshots and are never replaced.
    #[doc(hidden)]
    pub fn bind_shared_buffer_owner(&self, owner: Arc<dyn Send + Sync>) {
        for object in &self.objects {
            if object.registered_display_bytes == 0 {
                let _ = object.buffers.inner.lease.set(DisplayLease(owner.clone()));
            }
        }
    }
    /// Build private render buffers from an already validated B-rep. The caller
    /// owns source/topology verification, admission and the returned allocation.
    /// This does not modify or publish a scene.
    pub fn prepare_brep_buffers(shape: &om9_core::brep::Brep, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<(PreparedBrepBuffers, Usage)> {
        let d = shape.display();
        let mut estimate = Usage { bytes: std::mem::size_of::<BufferPayload>() as u64 + 16, ..Usage::default() };
        estimate.add(Usage::array::<Point3>(d.points.len())?, limits)?;
        estimate.add(Usage::array::<([u32; 3], u32)>(d.triangles.len())?, limits)?;
        estimate.add(Usage::array::<[u32; 2]>(d.segments.len())?, limits)?;
        admit(0, estimate.bytes)?;
        let mut usage = Usage::default();
        let buffers = brep_buffers(shape.display(), &mut usage, limits)?;
        let actual = charge_buffer_capacity(&mut usage, 0, &buffers, limits)?;
        admit(actual.bytes, 0)?;
        Ok((PreparedBrepBuffers { buffers, display: shape.display_identity(), interior_wires_prepared: false,
            fingerprint: shape.display_fingerprint() }, actual))
    }
    /// Prepare a private root display replacement. The caller must first prove
    /// the candidate topology mapping and must keep the buffer and display
    /// reservations alive for every consumer of this scene. No scene is published.
    #[doc(hidden)]
    pub fn try_replace_root_brep_display(&self, stamp: &SourceStamp, candidate: &om9_core::brep::Brep,
        prepared: PreparedBrepBuffers, profile: [u64; 2], display_lease: Arc<dyn std::any::Any + Send + Sync>, buffer_lease: Arc<dyn Send + Sync>) -> Result<Self> {
        self.try_replace_occurrence_source_display(stamp,&[],candidate,prepared,profile,display_lease,buffer_lease,None,false)
    }
    /// Install worker-prepared interior wires on a fresh, uniquely buffered
    /// root. Keep its already validated/pinned canonical display unchanged.
    /// The caller admits snapshot metadata and retains the buffer lease.
    #[doc(hidden)]
    pub fn try_install_prepared_brep_wires(&self, stamp: &SourceStamp, prepared: PreparedBrepBuffers,
        buffer_lease: Arc<dyn Send + Sync>) -> Result<Self> {
        let index = self.objects.iter().position(|object| object.root == stamp.object && object.path.is_empty())
            .ok_or(Error::Missing("prepared wire root"))?;
        let object = &self.objects[index];
        if object.source != *stamp || object.leaf != stamp.object || object.registered_display_bytes != 0
            || !prepared.interior_wires_prepared
            || self.objects.iter().filter(|other| other.root == stamp.object && other.path.is_empty()).count() != 1
            || self.objects.iter().filter(|other| other.buffers.ptr_eq(&object.buffers)).count() != 1 {
            return Err(Error::Stale("prepared wire root"));
        }
        let entity = self.entity_for(&self.document, object)?;
        let Geometry::Brep(source) = &entity.geometry else { return Err(Error::Invalid("prepared wire source")); };
        let profile = {
            let store = self.representations.lock().unwrap_or_else(|e| e.into_inner());
            let display = store.get(source)?;
            if !prepared.display.same(&display.display_identity()) || prepared.fingerprint != display.display_fingerprint()
                || !std::ptr::eq(object.pick_brep_display(entity)?.ok_or(Error::Missing("prepared wire pin"))?, display.display()) {
                return Err(Error::Stale("prepared wire display"));
            }
            match display.profile() {
                om9_core::brep::BrepRepresentationProfile::Certified { linear, angular, .. } => [linear, angular],
                _ => [0; 2],
            }
        };
        let old = buffer_usage(object.buffers.as_ref())?;
        let new = buffer_usage(&prepared.buffers)?;
        let mut totals = self.buffers_usage;
        totals.bytes = totals.bytes.checked_sub(old.bytes).ok_or(Error::Invalid("prepared wire bytes"))?;
        totals.points = totals.points.checked_sub(old.points).ok_or(Error::Invalid("prepared wire points"))?;
        totals.add(new, &self.limits)?;
        let mut scene = self.try_render_snapshot()?;
        let target = &mut scene.objects[index];
        target.buffers = SceneBuffers::owned_with_flags(prepared.buffers, Some(DisplayLease(buffer_lease)), false, true);
        target.registered_display_bytes = new.bytes;
        target.remesh_profile = profile;
        scene.buffers_usage = totals;
        scene.registered_display_bytes = scene.registered_display_bytes.checked_add(new.bytes)
            .ok_or(Error::Overflow("prepared wire display bytes"))?;
        scene.publication_id = uuid::Uuid::new_v4();
        Ok(scene)
    }
    /// Prepare a source-bound display replacement for one root or nested
    /// definition occurrence. Its root stamp and complete path stay intact.
    #[doc(hidden)]
    pub fn try_replace_brep_occurrence_display(&self, stamp: &SourceStamp, path: &[ObjectId], candidate: &om9_core::brep::Brep,
        prepared: PreparedBrepBuffers, profile: [u64; 2], display_lease: Arc<dyn std::any::Any + Send + Sync>, buffer_lease: Arc<dyn Send + Sync>) -> Result<Self> {
        use om9_core::brep::BrepSourceAccess;
        let mut matches = self.objects.iter().filter(|object| object.root == stamp.object && object.path == path);
        let object = matches.next().ok_or(Error::Missing("wire scene occurrence"))?;
        if matches.next().is_some() || object.source != *stamp {
            return Err(Error::Stale("wire scene occurrence"));
        }
        let entity = self.entity_for(&self.document, object)?;
        let Geometry::Brep(source) = &entity.geometry else { return Err(Error::Invalid("wire scene source")); };
        if !BrepSourceAccess::shares_validation_payload(source, candidate) {
            return Err(Error::Stale("wire scene canonical source"));
        }
        self.try_replace_occurrence_source_display(stamp,path,candidate,prepared,profile,display_lease,buffer_lease,None,true)
    }
    /// The caller must bind the candidate to this captured native source before publication.
    #[doc(hidden)]
    pub fn try_replace_root_native_brep_display(&self, stamp: &SourceStamp, candidate: &om9_core::brep::Brep,
        prepared: PreparedBrepBuffers, profile: [u64;2], display_lease: Arc<dyn std::any::Any + Send + Sync>, buffer_lease: Arc<dyn Send + Sync>,
        native: &om9_core::transaction::CapturedNativeBrepSource) -> Result<Self> {
        self.try_replace_occurrence_source_display(stamp,&[],candidate,prepared,profile,display_lease,buffer_lease,Some(native),false)
    }
    fn try_replace_occurrence_source_display(&self, stamp: &SourceStamp, path: &[ObjectId], candidate: &om9_core::brep::Brep,
        prepared: PreparedBrepBuffers, profile: [u64; 2], display_lease: Arc<dyn std::any::Any + Send + Sync>, buffer_lease: Arc<dyn Send + Sync>,
        native: Option<&om9_core::transaction::CapturedNativeBrepSource>, share_wires: bool) -> Result<Self> {
        if prepared.fingerprint != candidate.display_fingerprint()
            || !prepared.display.same(&candidate.display_identity()) {
            return Err(Error::Stale("remesh display buffers"));
        }
        let interior_wires_prepared = prepared.interior_wires_prepared;
        let buffers = prepared.buffers;
        let mut found = self.objects.iter().enumerate().filter(|(_, object)| object.root == stamp.object && object.path == path);
        let (index, object) = found.next().ok_or(Error::Missing("remesh scene occurrence"))?;
        if found.next().is_some() || object.source != *stamp || (path.is_empty() && object.leaf != stamp.object) {
            return Err(Error::Stale("remesh scene occurrence"));
        }
        let entity = self.entity_for(&self.document, object)?;
        match (&entity.geometry,native) {
            (Geometry::Brep(original),None) if original.source()==candidate.source() => {},
            (Geometry::NativeBrep(original),Some(source)) if source.matches_source(original) => {},
            _ => return Err(Error::Stale("remesh root source")),
        }
        let old_is_shared = self.shared_buffer_is_consistent(index)?;
        let mut replace_shared = share_wires && interior_wires_prepared && old_is_shared && object.registered_display_bytes == 0;
        if replace_shared {
            use om9_core::brep::BrepSourceAccess;
            for other in self.objects.iter().filter(|other| object.buffers.ptr_eq(&other.buffers)) {
                let source_matches = matches!(&self.entity_for(&self.document, other)?.geometry,
                    Geometry::Brep(source) if BrepSourceAccess::shares_validation_payload(source, candidate));
                let display_matches = other.brep_pick_display.as_ref().is_some_and(|pin|
                    std::ptr::eq(pin.display(), candidate.display()));
                if !source_matches || !display_matches || other.registered_display_bytes != 0 {
                    replace_shared = false;
                    break;
                }
            }
        }
        let old = buffer_usage(object.buffers.as_ref())?;
        let new = buffer_usage(&buffers)?;
        let mut totals = self.buffers_usage;
        if !old_is_shared || replace_shared {
            totals.bytes = totals.bytes.checked_sub(old.bytes).ok_or(Error::Invalid("remesh buffer accounting"))?;
            totals.points = totals.points.checked_sub(old.points).ok_or(Error::Invalid("remesh point accounting"))?;
        }
        totals.add(new, &self.limits)?;
        let mut scene = self.try_render_snapshot()?;
        let replacement = SceneBuffers::owned_with_flags(buffers, Some(DisplayLease(buffer_lease)), false, interior_wires_prepared);
        let pin = candidate.pin_display(display_lease);
        for (i, target) in scene.objects.iter_mut().enumerate() {
            if i == index || (replace_shared && object.buffers.ptr_eq(&target.buffers)) {
                target.buffers = replacement.clone();
                target.brep_pick_display = Some(pin.clone());
                target.remesh_profile = profile;
                target.registered_display_bytes = new.bytes;
            }
        }
        scene.buffers_usage = totals;
        let released = if old_is_shared && !replace_shared { 0 } else { object.registered_display_bytes };
        scene.registered_display_bytes = self.registered_display_bytes.checked_sub(released)
            .and_then(|bytes| bytes.checked_add(new.bytes)).ok_or(Error::Overflow("remesh display accounting"))?;
        scene.publication_id = uuid::Uuid::new_v4();
        Ok(scene)
    }
    /// Owned render preparation snapshot: immutable document/buffers are shared; mutable picking
    /// caches are not. No UI/session/native owner is sent to the preparation worker.
    pub fn render_snapshot(&self) -> Self {
        self.try_render_snapshot().expect("scene snapshot allocation")
    }
    /// Construct a worker snapshot without aborting when its metadata cannot
    /// be allocated. Geometry buffers remain shared with the source scene.
    pub fn try_render_snapshot(&self) -> Result<Self> {
        self.try_render_snapshot_checked(|| Ok(()))
    }

    /// Convert an admitted worker scene into an immutable picking snapshot.
    /// Its caller must hold the corresponding tracked document separately and
    /// admit both `PickSnapshot::additional_metadata_bytes()` and
    /// `Scene::pick_snapshot_weak_receipt_bytes()` through its own owner.
    pub fn into_pick_snapshot(self) -> Result<PickSnapshot> {
        let arena_bytes = self.picking_arena_bytes()?;
        let metadata_bytes = self.metadata_storage_usage()?.bytes;
        let shared_buffer_bytes = self.shared_buffer_bytes();
        let query_scratch = self.nearest_pick_query_scratch_estimate()?;
        let document = Arc::downgrade(&self.document);
        let Self { objects, limits, epoch, publication_id, memory, .. } = self;
        Ok(PickSnapshot { objects, limits, epoch, publication_id, document,
            arena_bytes, metadata_bytes, shared_buffer_bytes, query_scratch, memory })
    }

    /// Heap estimate for the independent object-pick source copies. The caller
    /// must reserve it before capture and retain that charge with the snapshot.
    pub fn hover_pick_source_estimate(&self)->Result<u64> {
        let mut bytes=Usage::array::<HoverPickSource>(self.objects.len())?.bytes;
        for object in &self.objects {
            let entity=self.entity_for(&self.document,object)?;
            if let Geometry::Curve(curve)=&entity.geometry {
                bytes=bytes.checked_add(curve.validate(&self.limits)?.bytes).ok_or(Error::Overflow("hover pick sources"))?;
            }
        }
        Ok(bytes.max(1))
    }

    /// Build an exact object-pick snapshot that does not retain the document.
    /// `admit` accounts source-copy allocations separately from scene buffers.
    pub fn into_hover_pick_snapshot(self,mut admit:impl FnMut(u64,u64)->Result<()>) -> Result<HoverPickSnapshot> {
        let vector=Usage::array::<HoverPickSource>(self.objects.len())?.bytes;
        admit(0,vector)?;
        let mut sources=Vec::new();sources.try_reserve_exact(self.objects.len()).map_err(|_|Error::Budget("hover pick sources"))?;
        let mut live=Usage::array::<HoverPickSource>(sources.capacity())?.bytes;admit(live,0)?;
        for object in &self.objects {
            let entity=self.entity_for(&self.document,object)?;
            let curve=if let Geometry::Curve(curve)=&entity.geometry {
                let estimated=curve.validate(&self.limits)?.bytes;
                admit(live,estimated)?;
                let copied=copy_hover_curve(curve)?;
                live=live.checked_add(copied.validate(&self.limits)?.bytes).ok_or(Error::Overflow("hover pick sources"))?;
                admit(live,0)?;Some(copied)
            } else {None};
            if matches!(object.buffers.as_ref(),RenderBuffers::Curve {..}) && curve.is_none() {
                return Err(Error::Stale("hover curve source"));
            }
            let display=object.pick_brep_display(entity)?;
            if display.is_some() && object.brep_pick_display.is_none() {
                return Err(Error::Missing("hover BRep display pin"));
            }
            sources.push(HoverPickSource {curve,cloud:matches!(entity.geometry,Geometry::Cloud(_)),brep_display:display.is_some()});
        }
        let scene=self.into_pick_snapshot()?;
        Ok(HoverPickSnapshot {scene,sources})
    }

    /// Validate the mesh-to-entity bindings while the document is still
    /// available, then discard the strong document owner. A non-surface mesh
    /// buffer may carry line strokes, but may not contribute triangles.
    pub fn into_occlusion_snapshot(self) -> Result<OcclusionSnapshot> {
        for object in &self.objects {
            let RenderBuffers::Mesh { triangles, .. } = object.buffers.as_ref() else { continue };
            if triangles.is_empty() { continue; }
            let entity = self.entity_for(&self.document, object)?;
            if !matches!(entity.geometry, Geometry::Mesh(_) | Geometry::Hatch(_) | Geometry::Surface(_)
                | Geometry::Brep(_) | Geometry::NativeBrep(_)) {
                return Err(Error::Invalid("occlusion mesh source"));
            }
        }
        Ok(OcclusionSnapshot(self.into_pick_snapshot()?))
    }
    fn try_render_snapshot_checked(&self, mut check: impl FnMut() -> Result<()>) -> Result<Self> {
        let mut objects = Vec::new();
        check()?;
        objects.try_reserve_exact(self.objects.len()).map_err(|_| Error::Budget("scene snapshot objects"))?;
        for object in &self.objects {
            let mut path = Vec::new();
            check()?;
            path.try_reserve_exact(object.path.len()).map_err(|_| Error::Budget("scene snapshot path"))?;
            path.extend_from_slice(&object.path);
            objects.push(SceneObject {
                root: object.root, path, leaf: object.leaf, source: object.source,
                leaf_generation: object.leaf_generation,
                leaf_geometry_revision: object.leaf_geometry_revision,
                brep_display_fingerprint: object.brep_display_fingerprint.clone(),
                brep_pick_display: object.brep_pick_display.clone(),
                native_display_pin: object.native_display_pin.clone(),
                transform: object.transform, preview: object.preview, color: object.color,
                buffers: object.buffers.clone(), visible: object.visible, selectable: object.selectable,
                editable: object.editable, reference_snappable: object.reference_snappable,
                reference_unlocked: object.reference_unlocked,
                brep_output_deflection: object.brep_output_deflection,
                remesh_profile: object.remesh_profile,
                registered_display_bytes: object.registered_display_bytes,
            });
        }
        let mut diagnostics = Vec::new();
        check()?;
        diagnostics.try_reserve_exact(self.diagnostics.len()).map_err(|_| Error::Budget("scene snapshot diagnostics"))?;
        for message in &self.diagnostics {
            let mut copy = String::new();
            check()?;
            copy.try_reserve_exact(message.len()).map_err(|_| Error::Budget("scene snapshot diagnostic"))?;
            copy.push_str(message);
            diagnostics.push(copy);
        }
        Ok(Self {
            document: self.document.clone(),
            representations: self.representations.clone(),
            epoch: self.epoch,
            publication_id: self.publication_id,
            objects,
            diagnostics,
            limits: self.limits,
            buffers_usage: self.buffers_usage,
            registered_display_bytes: self.registered_display_bytes,
            memory: Arc::new(SceneMemoryOwner::default()),
        })
    }

    /// Heap admission for the cache only; its document Arc is already owned by
    /// the coordinator/current or prepared Undo state. Buffers are counted once.
    pub fn cache_usage(&self) -> Result<Usage> {
        let mut usage = self.metadata_usage(true, 0)?;
        let (picking, _) = self.memory_usage(usage.bytes)?;
        usage.add(self.buffers_usage, &self.limits)?;
        usage.add(Usage { bytes: picking, ..Usage::default() }, &self.limits)?;
        Ok(usage)
    }
    /// Account cache storage alongside a separately validated document.
    pub fn cache_held_usage(&self) -> Result<Usage> {
        let mut usage = self.metadata_usage(true, 0)?;
        let (picking, registered) = self.memory_usage(usage.bytes)?;
        usage.add(self.buffers_usage, &self.limits)?;
        usage.add(Usage { bytes: picking, ..Usage::default() }, &self.limits)?;
        usage.bytes = usage.bytes.checked_sub(registered).ok_or(Error::Invalid("scene metadata accounting"))?;
        usage.bytes = usage.bytes.checked_sub(self.registered_display_bytes).ok_or(Error::Invalid("scene display accounting"))?;
        usage.points = 0;
        Ok(usage)
    }
    /// Bytes held by the distinct shared render buffers of this scene.
    /// Scene metadata and lazily built picking indices are not included.
    pub fn shared_buffer_bytes(&self) -> u64 {
        self.buffers_usage.bytes.saturating_sub(self.registered_display_bytes)
    }
    /// Requested metadata for a fresh render snapshot. Shared geometry and the
    /// source scene's lazy picking caches are not copied. Allocator capacity
    /// above the request must be reconciled after construction.
    pub fn render_snapshot_metadata_estimate(&self) -> Result<Usage> {
        self.metadata_usage(false, 0)
    }
    /// Conservative fixed allocation retained by native weak receipts after a
    /// source document dies. Shared occurrences may be counted more than once.
    pub fn pick_snapshot_weak_receipt_bytes(&self) -> Result<u64> {
        let count = self.objects.iter().filter(|object| matches!(object.brep_display_fingerprint,
            Some(BrepDisplayReceipt::Native { .. }))).count();
        let per = std::mem::size_of::<Vec<u8>>() + std::mem::size_of::<om9_core::geometry::Mesh>()
            + 4 * std::mem::size_of::<usize>();
        count.checked_mul(per).and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Error::Overflow("pick snapshot weak receipt storage"))
    }
    /// Actual metadata capacity, excluding geometry and lazy picking caches.
    pub fn metadata_storage_usage(&self) -> Result<Usage> {
        self.metadata_usage(true, 0)
    }
    /// Estimate lazy bounds/index and one mesh's transformed-vertex workspace
    /// for a fresh nearest-pick snapshot. Excludes allocator overhead and
    /// implementation-dependent sorting, evaluation and query allocations.
    pub fn nearest_pick_scratch_estimate(&self) -> Result<Usage> {
        let mut usage = self.nearest_pick_query_scratch_estimate()?;
        let state = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if state.reservation.is_none() {
            usage.add(Usage { bytes: self.picking_arena_bytes()?, ..Usage::default() },
                &Limits { max_bytes: u64::MAX, ..Limits::default() })?;
        }
        Ok(usage)
    }
    /// A fresh worker owns its own arena; the source's cache is not copied.
    pub fn render_snapshot_pick_scratch_estimate(&self) -> Result<Usage> {
        let mut usage = self.nearest_pick_query_scratch_estimate()?;
        usage.add(Usage { bytes: self.picking_arena_bytes()?, ..Usage::default() },
            &Limits { max_bytes: u64::MAX, ..Limits::default() })?;
        Ok(usage)
    }
    pub fn nearest_pick_query_scratch_estimate(&self) -> Result<Usage> {
        let limits = Limits { max_bytes: u64::MAX, ..Limits::default() };
        let mut usage = Usage::array::<usize>(self.objects.len())?;
        usage.add(Usage::array::<u32>(self.objects.len())?, &limits)?;
        let max_vertices = self.objects.iter().filter_map(|object| match object.buffers.as_ref() {
            RenderBuffers::Mesh { vertices, .. } => Some(vertices.len()),
            _ => None,
        }).max().unwrap_or(0);
        usage.add(Usage::array::<Option<Point3>>(max_vertices)?, &limits)?;
        Ok(usage)
    }
    fn metadata_usage(&self, capacity: bool, extra: usize) -> Result<Usage> {
        let count = if capacity {
            self.objects.capacity()
        } else {
            self.objects.len()
        };
        let count = count
            .checked_add(extra)
            .ok_or(Error::Overflow("scene metadata count"))?;
        let mut usage = Usage {
            nodes: self.objects.len() as u64 + extra as u64,
            ..Usage::array::<SceneObject>(count)?
        };
        usage.add(Usage::array::<Self>(1)?, &self.limits)?;
        usage.add(Usage::array::<Mutex<SceneMemory>>(1)?, &self.limits)?;
        usage.add(Usage::array::<usize>(2)?, &self.limits)?;
        usage.add(
            Usage::array::<String>(if capacity {
                self.diagnostics.capacity()
            } else {
                self.diagnostics.len()
            })?,
            &self.limits,
        )?;
        for object in &self.objects {
            usage.add(
                Usage::array::<ObjectId>(if capacity {
                    object.path.capacity()
                } else {
                    object.path.len()
                })?,
                &self.limits,
            )?;
        }
        for message in &self.diagnostics {
            usage.add(
                Usage {
                    bytes: if capacity {
                        message.capacity()
                    } else {
                        message.len()
                    } as u64,
                    ..Usage::default()
                },
                &self.limits,
            )?;
        }
        Ok(usage)
    }
    pub fn update_usage(&self, addition: Option<&Entity>) -> Result<Usage> {
        self.update_usage_with_candidate(addition, None)
    }
    fn update_usage_with_candidate(&self, addition: Option<&Entity>, candidate: Option<&om9_core::brep::Brep>) -> Result<Usage> {
        let mut usage = self.metadata_usage(false, usize::from(addition.is_some()))?;
        let members = self
            .document
            .definitions
            .values()
            .try_fold(0usize, |n, d| {
                n.checked_add(d.members.len())
                    .ok_or(Error::Overflow("scene member index"))
            })?;
        if members > 0 {
            usage.add(
                Usage::btree::<(DefinitionId, ObjectId), &Entity>(members)?,
                &self.limits,
            )?;
        }
        if let Some(entity) = addition {
            let count = match &entity.geometry {
                Geometry::Point(_) => 1,
                Geometry::Curve(Curve::Line { .. }) => 2,
                Geometry::Curve(Curve::Circle { .. }) => 720,
                Geometry::Curve(curve @ Curve::Polycurve { segments, .. }) => {
                    match connected_polyline_segments(curve, &self.limits) {
                        Ok(connected) => polyline_display_count(connected)?,
                        Err(_) if segments.iter().all(|s| matches!(s, Curve::Line { .. })) => {
                            usage.add(Usage::array::<[u32; 2]>(segments.len())?, &self.limits)?;
                            segments.len().checked_mul(2).ok_or(Error::Overflow("stroke vertices"))?
                        }
                        Err(_) => segments.len().checked_add(1).ok_or(Error::Overflow("polyline display vertices"))?,
                    }
                },
                Geometry::Curve(Curve::Nurbs(_)) => 257,
                Geometry::Surface(surface) => {
                    let mut local = Usage::default();
                    match entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
                        Some(extension) => {
                            usage.add(Usage::array::<String>(1)?,&self.limits)?;
                            usage.add(Usage {bytes:384,..Usage::default()},&self.limits)?;
                            match trim_decomposition::payload_bound(&extension.payload,&self.limits) {
                                Ok(bound)=>local=bound,
                                Err(Error::Invalid(_) | Error::Unsupported(_))=>{ surface_buffer_usage(&mut local,&self.limits)?; },
                                Err(error)=>return Err(error),
                            }
                        }
                        None=>{ surface_buffer_usage_for(surface,true,&mut local,&self.limits)?; },
                    }
                    usage.add(Usage::array::<RenderBuffers>(1)?, &self.limits)?;
                    usage.add(local, &self.limits)?;
                    return Ok(usage);
                }
                Geometry::Mesh(mesh) => {
                    let mut local = Usage::default();
                    mesh_buffer_usage(mesh, &mut local, &self.limits)?;
                    usage.add(Usage::array::<RenderBuffers>(1)?, &self.limits)?;
                    usage.add(local, &self.limits)?;
                    return Ok(usage);
                }
                Geometry::Brep(shape) => {
                    let store = self.representations.lock().unwrap_or_else(|e| e.into_inner());
                    let display = match store.get(shape) {
                        Ok(representation) => representation.display(),
                        Err(error) => {
                            use om9_core::brep::BrepSourceAccess;
                            let candidate = candidate.filter(|candidate| candidate.to_canonical().identity() == shape.identity()).ok_or(error)?;
                            candidate.display()
                        }
                    };
                    usage.add(Usage::array::<RenderBuffers>(1)?, &self.limits)?;
                    usage.add(
                        Usage {
                            points: display.points.len() as u64,
                            ..Usage::array::<Point3>(display.points.len())?
                        },
                        &self.limits,
                    )?;
                    usage.add(
                        Usage::array::<([u32; 3], u32)>(display.triangles.len())?,
                        &self.limits,
                    )?;
                    usage.add(
                        Usage::array::<[u32; 2]>(display.segments.len())?,
                        &self.limits,
                    )?;
                    return Ok(usage);
                }
                _ => return Err(Error::Unsupported("plain root cache addition")),
            };
            usage.add(Usage::array::<RenderBuffers>(1)?, &self.limits)?;
            usage.add(
                Usage {
                    bytes: 16,
                    points: count as u64,
                    ..Usage::default()
                },
                &self.limits,
            )?;
            usage.add(Usage::array::<Point3>(count)?, &self.limits)?;
            usage.add(
                if count == 1 {
                    Usage::array::<u32>(1)?
                } else {
                    Usage::array::<f64>(2)?
                },
                &self.limits,
            )?;
        }
        Ok(usage)
    }
    /// Extra scene ownership held beside a document transaction. Source and
    /// scene each obey the point limit independently; all bytes remain charged.
    pub fn update_held_usage(&self, addition: Option<&Entity>) -> Result<Usage> {
        let mut scene_points = self.buffers_usage.points;
        self.update_held_usage_counted(addition, &mut scene_points)
    }
    /// Carry the scene point count across a batch before any transaction is
    /// published. The caller starts with `cache_usage()?.points`.
    pub fn update_held_usage_counted(&self, addition: Option<&Entity>, scene_points: &mut u64) -> Result<Usage> {
        self.update_held_usage_counted_with_candidate(addition, None, scene_points)
    }
    /// Plan a private candidate before its display enters the shared store.
    /// The caller retains and accounts the candidate throughout this estimate.
    pub fn update_held_usage_counted_with_candidate(&self, addition: Option<&Entity>,
        candidate: Option<&om9_core::brep::Brep>, scene_points: &mut u64) -> Result<Usage> {
        if *scene_points < self.buffers_usage.points {
            return Err(Error::Invalid("scene point baseline"));
        }
        let mut usage = self.update_usage_with_candidate(addition, candidate)?;
        let mut total = Usage { points: *scene_points, ..Usage::default() };
        total.add(Usage { points: usage.points, ..Usage::default() }, &self.limits)?;
        *scene_points = total.points;
        usage.points = 0;
        Ok(usage)
    }
    /// Reuse immutable geometry buffers only after proving all prior geometry,
    /// placements/identity/context unchanged. Optional addition is one Point/Line.
    pub fn update_owned(
        &self,
        document: Arc<Document>,
        epoch: u64,
        addition: Option<ObjectId>,
        limits: &Limits,
    ) -> Result<Self> {
        let old = &self.document;
        if old.id != document.id
            || old.owner_session != document.owner_session
            || old.epoch != document.epoch
            || old.session_generation != document.session_generation
            || old.context_revision != document.context_revision
            || old.page_unit != document.page_unit
            || old.definitions.len() != document.definitions.len()
            || old
                .entities
                .len()
                .checked_add(usize::from(addition.is_some()))
                != Some(document.entities.len())
        {
            return Err(Error::Stale("scene metadata owner/context"));
        }
        let same = |a: &Entity, b: &Entity| {
            a.id == b.id
                && a.generation == b.generation
                && a.geometry_revision == b.geometry_revision
                && a.geometry == b.geometry
                // Equal bytes or topology are insufficient to transfer an
                // existing representation pin to another source owner.
                && match (&a.geometry, &b.geometry) {
                    (Geometry::Brep(old), Geometry::Brep(new)) => old.identity() == new.identity(),
                    (Geometry::NativeBrep(old), Geometry::NativeBrep(new)) => old.identity() == new.identity(),
                    _ => true,
                }
                // A surface's trim extension defines its visible geometry even
                // when the underlying NURBS and geometry revision are unchanged.
                && (!matches!(&a.geometry, Geometry::Surface(_))
                    || a.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT)
                        == b.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT))
                && a.placement == b.placement
        };
        for (id, e) in &old.entities {
            if !document.entities.get(id).is_some_and(|new| same(e, new)) {
                return Err(Error::Stale("scene metadata geometry"));
            }
        }
        for (id, definition) in &old.definitions {
            let new = document
                .definitions
                .get(id)
                .ok_or(Error::Stale("scene metadata definition"))?;
            if definition.members.len() != new.members.len()
                || !definition
                    .members
                    .iter()
                    .zip(&new.members)
                    .all(|(a, b)| same(a, b))
            {
                return Err(Error::Stale("scene metadata members"));
            }
        }
        let added = addition
            .map(|id| {
                document
                    .entities
                    .get(&id)
                    .filter(|_| !old.entities.contains_key(&id))
                    .ok_or(Error::Invalid("scene added root"))
            })
            .transpose()?;
        self.update_usage(added)?.check(limits)?;
        let mut members = BTreeMap::new();
        for (id, definition) in &document.definitions {
            for member in &definition.members {
                members.insert((*id, member.id), member);
            }
        }
        let mut objects = Vec::new();
        objects
            .try_reserve_exact(self.objects.len() + usize::from(added.is_some()))
            .map_err(|_| Error::Budget("scene metadata allocation"))?;
        for object in &self.objects {
            let mut path = Vec::new();
            path.try_reserve_exact(object.path.len())
                .map_err(|_| Error::Budget("scene path allocation"))?;
            path.extend_from_slice(&object.path);
            let mut value = SceneObject {
                root: object.root,
                path,
                leaf: object.leaf,
                leaf_generation: object.leaf_generation,
                leaf_geometry_revision: object.leaf_geometry_revision,
                brep_display_fingerprint: object.brep_display_fingerprint.clone(),
                brep_pick_display: object.brep_pick_display.clone(),
                native_display_pin: object.native_display_pin.clone(),
                source: object.source,
                transform: object.transform,
                preview: object.preview,
                color: object.color,
                buffers: object.buffers.clone(),
                visible: object.visible,
                selectable: object.selectable,
                editable: object.editable,
                reference_snappable: object.reference_snappable,
                reference_unlocked: object.reference_unlocked,
                brep_output_deflection: object.brep_output_deflection,
                remesh_profile: object.remesh_profile,
                registered_display_bytes: object.registered_display_bytes,
            };
            let (leaf, permissions, unlocked) =
                metadata_leaf(&document, &members, object.root, &object.path)?;
            if leaf.id != object.leaf {
                return Err(Error::Stale("scene metadata leaf"));
            }
            // The equality and source-owner checks above establish that the
            // existing buffers and pins remain bound to this occurrence.
            value.brep_display_fingerprint = brep_display_fingerprint(leaf);

            value.source = document.stamp(object.root)?;
            value.color = leaf
                .attributes
                .color
                .or_else(|| {
                    leaf.attributes
                        .layer
                        .and_then(|id| document.layers.get(&id).map(|l| l.color))
                })
                .unwrap_or([220, 220, 220, 255]);
            value.visible = permissions.visible;
            value.selectable = permissions.selectable;
            value.editable = permissions.editable;
            value.reference_snappable = permissions.reference_snappable;
            value.reference_unlocked = unlocked;
            objects.push(value);
        }
        let mut buffers_usage = self.buffers_usage;
        let mut trim_fallback=None;
        if let Some(entity) = added {
            let mut usage = Usage::default();
            let (buffers,fallback) = cache_entity(entity, &mut usage, limits, &self.representations.lock().unwrap_or_else(|e| e.into_inner()))?;
            if fallback {trim_fallback=Some(entity.id);}
            let actual = charge_buffer_capacity(&mut usage, 0, &buffers, limits)?;
            buffers_usage.add(actual, limits)?;
            let p = document.permissions(entity.id, true)?;
            let color = entity
                .attributes
                .color
                .or_else(|| {
                    entity
                        .attributes
                        .layer
                        .and_then(|id| document.layers.get(&id).map(|l| l.color))
                })
                .unwrap_or([220, 220, 220, 255]);
            let position = objects.partition_point(|object| object.root < entity.id);
            objects.insert(
                position,
                SceneObject {
                    root: entity.id,
                    path: vec![],
                    leaf: entity.id,
                    leaf_generation: entity.generation,
                    leaf_geometry_revision: entity.geometry_revision,
                    brep_display_fingerprint: brep_display_fingerprint(entity),
                    brep_pick_display: match &entity.geometry {
                        Geometry::Brep(source) => Some(self.representations.lock().unwrap_or_else(|e|e.into_inner()).pin_for(source, Arc::new(()))?), _=>None,
                    },
                    native_display_pin: match &entity.geometry {
                        Geometry::NativeBrep(native) => Some(self.representations.lock().unwrap_or_else(|e|e.into_inner())
                            .get_native(native)?.pin()?),
                        _ => None,
                    },
                    source: document.stamp(entity.id)?,
                    transform: entity.placement,
                    preview: Transform::identity(),
                    color,
                    buffers: SceneBuffers::owned_with_trim_fallback(buffers, None, fallback),
                    visible: p.visible,
                    selectable: p.selectable,
                    editable: p.editable,
                    reference_snappable: p.reference_snappable,
                    reference_unlocked: document.permissions(entity.id, false)?.reference_snappable,
                    brep_output_deflection: match &entity.geometry { Geometry::Brep(brep) => Some(self.representations.lock().unwrap_or_else(|e| e.into_inner()).display_for(brep)?.deflection_mm), _ => None },
                    remesh_profile: [0; 2],
                    registered_display_bytes: 0,
                },
            );
        }
        let mut diagnostics = Vec::new();
        diagnostics
            .try_reserve_exact(self.diagnostics.len()+usize::from(trim_fallback.is_some()))
            .map_err(|_| Error::Budget("scene diagnostics allocation"))?;
        for message in &self.diagnostics {
            let mut copy = String::new();
            copy.try_reserve_exact(message.len())
                .map_err(|_| Error::Budget("scene diagnostic allocation"))?;
            copy.push_str(message);
            diagnostics.push(copy);
        }
        if let Some(id)=trim_fallback {diagnostics.push(format!("{id}: trim display unsupported; showing underlying surface"));}
        Ok(Self {
            document,
            representations: self.representations.clone(),
            epoch,
            publication_id: uuid::Uuid::new_v4(),
            objects,
            diagnostics,
            limits: *limits,
            buffers_usage,
            registered_display_bytes: self.registered_display_bytes,
            memory: Arc::new(SceneMemoryOwner::default()),
        })
    }
    /// Incremental scene plus its newly retained bytes while `self` remains
    /// alive. `update_owned` shares all existing render buffers; metadata and
    /// any added buffer are independent. Planner scratch is not retained here.
    pub fn update_owned_with_retained_bytes(
        &self, document: Arc<Document>, epoch: u64, addition: Option<ObjectId>, limits: &Limits,
    ) -> Result<(Self, u64)> {
        let next = self.update_owned(document, epoch, addition, limits)?;
        let bytes = next.cache_usage()?.bytes.checked_sub(self.buffers_usage.bytes)
            .ok_or(Error::Invalid("incremental scene shared buffer accounting"))?;
        Ok((next, bytes))
    }

    pub fn capture(document: Arc<Document>, epoch: u64, limits: &Limits) -> Result<Self> {
        Self::capture_with_representations(document, epoch, limits, Arc::new(Mutex::new(BrepRepresentationStore::default())))
    }
    pub fn capture_with_representations(document: Arc<Document>, epoch: u64, limits: &Limits,
        representations: Arc<Mutex<BrepRepresentationStore>>) -> Result<Self> {
        let usage = document.validate(limits)?;
        let plan = admission::admit(&document, limits, usage, &representations.lock().unwrap_or_else(|e| e.into_inner()))?;
        Self::capture_admitted(document, epoch, limits, usage, plan, representations, None)
    }
    /// Validate and plan the immutable candidate once, then admit its estimated
    /// construction peak before any scene buffers are allocated. The caller
    /// must already account for the document and preflight scratch and retain
    /// the construction reservation through the scene handoff.
    pub fn capture_with_peak_admission(
        document: Arc<Document>, epoch: u64, limits: &Limits,
        admit_peak: impl FnOnce(u64) -> Result<()>,
    ) -> Result<Self> {
        Self::capture_with_peak_and_representations(document, epoch, limits,
            Arc::new(Mutex::new(BrepRepresentationStore::default())), admit_peak)
    }
    pub fn capture_with_peak_and_representations(
        document: Arc<Document>, epoch: u64, limits: &Limits,
        representations: Arc<Mutex<BrepRepresentationStore>>,
        admit_peak: impl FnOnce(u64) -> Result<()>,
    ) -> Result<Self> {
        let usage = document.validate(limits)?;
        let plan = admission::admit(&document, limits, usage, &representations.lock().unwrap_or_else(|e| e.into_inner()))?;
        admit_peak(plan.peak_bytes)?;
        Self::capture_admitted(document, epoch, limits, usage, plan, representations, None)
    }
    /// Capture an edit candidate while reusing eligible prior local wires.
    /// Reuse only when the previous scene's memory and representation store
    /// belong to the coordinator. The caller must reserve
    /// `candidate_preflight_scratch_reusing` before entering this method.
    pub fn capture_with_peak_reusing_brep_wires(
        document: Arc<Document>, epoch: u64, limits: &Limits,
        coordinator: &TransactionCoordinator, previous: &Scene,
        admit_peak: impl FnOnce(u64) -> Result<()>,
    ) -> Result<Self> {
        let representations = coordinator.representation_store();
        let same_account = {
            let state = previous.memory.lock().unwrap_or_else(|error| error.into_inner());
            state.reservation.as_ref().is_some_and(|reservation|
                reservation.belongs_to(&coordinator.external_memory_account()))
        };
        let previous = (same_account && Arc::ptr_eq(&previous.representations, &representations)).then_some(previous);
        let usage = document.validate(limits)?;
        let plan = admission::admit_reusing(&document, limits, usage, &representations.lock().unwrap_or_else(|e| e.into_inner()), previous)?;
        admit_peak(plan.peak_bytes)?;
        Self::capture_admitted(document, epoch, limits, usage, plan, representations, previous)
    }
    /// Scratch for the scene planner only, excluding document validation and
    /// the document itself. Use `candidate_preflight_scratch` before asking
    /// for a candidate scene peak.
    pub fn candidate_planning_scratch(document: &Document) -> Result<Usage> {
        admission::planning_scratch(document)
    }
    /// Validation and planning run sequentially in `candidate_scene_peak`,
    /// so their maximum is held alongside the already-owned document.
    pub fn candidate_preflight_scratch(document: &Document, limits: &Limits) -> Result<Usage> {
        let planning = admission::planning_scratch(document)?;
        let validation = document.validation_scratch_usage(limits)?;
        Ok(Usage { bytes: planning.bytes.max(validation.bytes), ..Usage::default() })
    }
    /// Preflight scratch including the prior scene's distinct reused buffers.
    pub fn candidate_preflight_scratch_reusing(document: &Document, limits: &Limits, previous: &Scene) -> Result<Usage> {
        let planning = admission::planning_scratch_reusing(document, Some(previous))?;
        let validation = document.validation_scratch_usage(limits)?;
        Ok(Usage { bytes: planning.bytes.max(validation.bytes), ..Usage::default() })
    }
    /// Estimate scene construction while the caller holds preflight scratch
    /// and document reservations. This is an admission estimate, not measured RSS.
    pub fn candidate_scene_peak(document: &Document, limits: &Limits) -> Result<u64> {
        let owned = document.validate(limits)?;
        Ok(admission::admit(document, limits, owned, &BrepRepresentationStore::default())?.peak_bytes)
    }
    /// The same planning estimate when a private candidate already has
    /// representation owners, including a migrated v4 document.
    pub fn candidate_scene_peak_with_representations(document: &Document, limits: &Limits,
        representations: &BrepRepresentationStore) -> Result<u64> {
        let owned = document.validate(limits)?;
        Ok(admission::admit(document, limits, owned, representations)?.peak_bytes)
    }
    pub fn capture_in_world(
        coordinator: &TransactionCoordinator,
        id: DocumentId,
        held: Usage,
    ) -> Result<Self> {
        Self::capture_in_world_with_physical_samples(coordinator, id, held, || None)
    }
    /// Sample immediately before planner, validation and construction admission. A `None`
    /// sample retains managed-only admission for platforms without a provider.
    pub fn capture_in_world_with_physical_samples(
        coordinator: &TransactionCoordinator,
        id: DocumentId,
        held: Usage,
        sample: impl FnMut() -> Option<(u64, u64)>,
    ) -> Result<Self> {
        let document = coordinator
            .world()
            .documents
            .get(&id)
            .ok_or(Error::Missing("scene world document"))?
            .clone();
        Self::capture_retained_with_physical_samples(coordinator, document, coordinator.epoch(), held, sample)
    }
    /// Capture a version retained or registered by the coordinator's account,
    /// including a staged Undo/Redo candidate. Its caller accounts for other
    /// held state and checks freshness before publishing the resulting scene.
    pub fn capture_retained_with_physical_samples(
        coordinator: &TransactionCoordinator,
        document: Arc<Document>,
        epoch: u64,
        held: Usage,
        sample: impl FnMut() -> Option<(u64, u64)>,
    ) -> Result<Self> {
        let (scene, _reservation) = Self::capture_retained_reserved_with_physical_samples(coordinator, document, epoch, held, sample)?;
        Ok(scene)
    }

    /// Return the construction reservation with the finished scene so its
    /// caller can keep the account charged until publication or handoff.
    pub fn capture_retained_reserved_with_physical_samples(
        coordinator: &TransactionCoordinator,
        document: Arc<Document>,
        epoch: u64,
        held: Usage,
        sample: impl FnMut() -> Option<(u64, u64)>,
    ) -> Result<(Self, om9_core::transaction::ExternalMemoryReservation)> {
        Self::capture_retained_reserved_reusing(coordinator, document, epoch, held, None, sample)
    }
    pub fn capture_retained_reserved_reusing(
        coordinator: &TransactionCoordinator,
        document: Arc<Document>,
        epoch: u64,
        held: Usage,
        previous: Option<&Scene>,
        mut sample: impl FnMut() -> Option<(u64, u64)>,
    ) -> Result<(Self, om9_core::transaction::ExternalMemoryReservation)> {
        if !coordinator.owns_or_has_registered_document(&document) {
            return Err(Error::Invalid("scene document is not retained"));
        }
        let (retained, limits) = coordinator.retained_usage_and_limits_with_held(held)?;
        let representations = coordinator.representation_store();
        let previous = previous.filter(|scene| {
            let state = scene.memory.lock().unwrap_or_else(|error| error.into_inner());
            state.reservation.as_ref().is_some_and(|reservation|
                reservation.belongs_to(&coordinator.external_memory_account()))
                && Arc::ptr_eq(&scene.representations, &representations)
        });
        let plan = {
            // Reserve the planner's distinct-source seen set before it grows.
            // Its lifetime ends when `admit` returns, before capture starts.
            let scratch = admission::planning_scratch_reusing(&document, previous)?;
            let _planning = coordinator.reserve_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::CpuCache, scratch.bytes, sample(), held,
            )?;
            admission::admit_reusing(&document, &limits, retained, &representations.lock().unwrap_or_else(|e|e.into_inner()), previous)?
        };
        let usage = {
            // The document itself is already charged by the coordinator; only
            // reserve the extra temporary validation storage here.
            let scratch = document.validation_scratch_usage(&limits)?;
            let _validation = coordinator.reserve_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::CpuCache, scratch.bytes.max(1), sample(), held,
            )?;
            document.validate(&limits)?
        };
        // Admission rechecks current shared reservations, including caller-held
        // allocations, and keeps the construction peak reserved until capture
        // completes and is handed to the caller alongside the scene.
        let mut reservation = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, plan.peak_bytes, sample(), held,
        )?;
        let mut scene = Self::capture_admitted(
            document,
            epoch,
            &limits,
            usage,
            plan,
            representations,
            previous,
        )?;
        let actual_bytes = scene.cache_held_usage()?.bytes;
        if actual_bytes > reservation.bytes() {
            let additional = actual_bytes - reservation.bytes();
            coordinator.grow_external_memory_with_held(&mut reservation, additional, sample(), held)?;
        }
        scene.pin_ordinary_brep_displays(coordinator, held, &mut sample)?;
        Ok((scene, reservation))
    }
    /// Bind ordinary OCCT occurrences to tracked display owners before a
    /// candidate scene is published. The document must be coordinator-owned
    /// or registered as an immutable candidate. On error discard the scene.
    pub fn pin_ordinary_brep_displays(&mut self, coordinator: &TransactionCoordinator, held: Usage,
        mut sample: impl FnMut() -> Option<(u64, u64)>) -> Result<()> {
        let mut count = 0usize;
        for object in &self.objects {
            if object.brep_pick_display.is_none() && matches!(&self.entity_for(&self.document, object)?.geometry, Geometry::Brep(_)) {
                count = count.checked_add(1).ok_or(Error::Overflow("scene display pins"))?;
            }
        }
        if count == 0 { return Ok(()); }
        let bytes = TransactionCoordinator::brep_display_batch_scratch(count)?.max(1);
        let mut scratch = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes, sample(), held)?;
        scratch.recheck_pending()?;
        let mut requests = Vec::new();
        requests.try_reserve_exact(count).map_err(|_| Error::Budget("scene display pins"))?;
        let extra = Usage::array::<om9_core::transaction::BrepDisplayRequest<'_>>(requests.capacity())?.bytes
            .checked_sub(Usage::array::<om9_core::transaction::BrepDisplayRequest<'_>>(count)?.bytes)
            .ok_or(Error::Overflow("scene display pins"))?;
        if extra != 0 { coordinator.grow_external_memory_with_held(&mut scratch, extra, sample(), held)?; }
        for (slot, object) in self.objects.iter().enumerate() {
            if object.brep_pick_display.is_none() && matches!(&self.entity_for(&self.document, object)?.geometry, Geometry::Brep(_)) {
                requests.push(om9_core::transaction::BrepDisplayRequest { slot, stamp: object.source, path: &object.path });
            }
        }
        let mut pins = coordinator.capture_retained_brep_displays(&self.document, &requests, &mut scratch, held, sample())?;
        drop(requests);
        for (slot, pin) in pins.drain() { self.objects[slot].brep_pick_display = Some(pin); }
        Ok(())
    }
    /// Release only ordinary pins on an unpublished replacement scene before
    /// its coordinator joins another account. Rebind them before publication.
    #[doc(hidden)]
    pub fn clear_ordinary_brep_pins(&mut self) {
        for object in &mut self.objects {
            if object.registered_display_bytes == 0 { object.brep_pick_display = None; }
        }
    }
    fn capture_admitted(
        document: Arc<Document>,
        epoch: u64,
        limits: &Limits,
        usage: Usage,
        plan: admission::AllocationPlan,
        representations: Arc<Mutex<BrepRepresentationStore>>,
        previous: Option<&Scene>,
    ) -> Result<Self> {
        let all = Permissions {
            visible: true,
            selectable: true,
            editable: true,
            reference_snappable: true,
        };
        let mut objects = Vec::new();
        objects
            .try_reserve_exact(plan.objects)
            .map_err(|_| Error::Budget("scene object allocation"))?;
        let mut diagnostics = Vec::new();
        diagnostics
            .try_reserve_exact(plan.diagnostics)
            .map_err(|_| Error::Budget("scene diagnostics allocation"))?;
        // The source document was independently validated above. Display
        // points use the same per-scene cap without charging source points a
        // second time; memory bytes still cover both live owners.
        let mut display_usage = usage;
        display_usage.points = 0;
        let store = representations.lock().unwrap_or_else(|e| e.into_inner());
        let mut capture = Capture {
            representations: &store,
            document: &document,
            previous,
            previous_index: previous_occurrence_index(previous)?,
            epoch,
            limits,
            usage: display_usage,
            objects,
            diagnostics,
            shared: BTreeMap::new(),
            reused_buffers: std::collections::BTreeSet::new(),
            buffers_usage: Usage::default(),
            registered_display_bytes: 0,
        };
        for entity in document.entities.values() {
            capture.entity(
                entity.id,
                vec![],
                entity,
                Transform::identity(),
                all,
                true,
                0,
            )?;
        }
        Ok(Self {
            document: document.clone(),
            representations: representations.clone(),
            epoch: capture.epoch,
            publication_id: uuid::Uuid::new_v4(),
            objects: capture.objects,
            diagnostics: capture.diagnostics,
            limits: *limits,
            buffers_usage: capture.buffers_usage,
            registered_display_bytes: capture.registered_display_bytes,
            memory: Arc::new(SceneMemoryOwner::default()),
        })
    }
    pub fn pick(
        &self,
        screen: [f64; 2],
        camera: &Camera,
        viewport: Viewport,
        tolerance_pixels: f64,
        intent: PickIntent,
    ) -> Result<Option<PickHit>> {
        self.pick_with_document(&self.document, screen, camera, viewport, tolerance_pixels, intent)
    }

    /// Pick against a document held by the caller, such as a tracked worker lease.
    pub fn pick_with_document(
        &self, document: &Document, screen: [f64; 2], camera: &Camera,
        viewport: Viewport, tolerance_pixels: f64, intent: PickIntent,
    ) -> Result<Option<PickHit>> {
        crate::picking::pick(self, document, screen, camera, viewport, tolerance_pixels, intent)
    }

    /// Picks one kind of subobject: the object, a vertex, an edge or a face, nearest first.
    pub fn pick_kind(
        &self,
        screen: [f64; 2],
        camera: &Camera,
        viewport: Viewport,
        tolerance_pixels: f64,
        intent: PickIntent,
        kind: PickKind,
    ) -> Result<Option<PickHit>> {
        Ok(crate::picking::pick_candidates(
            self,
            &self.document,
            screen,
            camera,
            viewport,
            tolerance_pixels,
            intent,
            kind,
            true,
            false,
        )?
        .into_iter()
        .next())
    }

    /// Nearest matching subobject for each distinct root object under the pointer.
    pub fn pick_kind_all(
        &self, screen: [f64; 2], camera: &Camera, viewport: Viewport,
        tolerance_pixels: f64, intent: PickIntent, kind: PickKind,
    ) -> Result<Vec<PickHit>> {
        crate::picking::pick_candidates(self, &self.document, screen, camera, viewport, tolerance_pixels, intent, kind, false, false)
    }

    /// All distinct visible edge candidates under the pointer, including multiple edges of one object.
    pub fn pick_edge_candidates(
        &self, screen: [f64; 2], camera: &Camera, viewport: Viewport,
        tolerance_pixels: f64, intent: PickIntent,
    ) -> Result<Vec<PickHit>> {
        crate::picking::pick_candidates(self, &self.document, screen, camera, viewport, tolerance_pixels, intent, PickKind::Edge, false, true)
    }

    /// Every distinct object under the pointer, nearest first. The ambiguity popup lists these.
    pub fn pick_all(
        &self,
        screen: [f64; 2],
        camera: &Camera,
        viewport: Viewport,
        tolerance_pixels: f64,
        intent: PickIntent,
    ) -> Result<Vec<PickHit>> {
        crate::picking::pick_candidates(
            self,
            &self.document,
            screen,
            camera,
            viewport,
            tolerance_pixels,
            intent,
            PickKind::Object,
            false,
            false,
        )
    }
    /// Read the exact root/member path using the explicitly supplied document.
    /// A different allocation is rejected even if its contents compare equal:
    /// the receipt belongs to the immutable document used to build this scene.
    pub fn entity_for<'a>(&self, document: &'a Document, object: &SceneObject) -> Result<&'a Entity> {
        if !std::ptr::eq(document, self.document.as_ref()) {
            return Err(Error::Stale("scene document owner"));
        }
        resolve_scene_entity(document, object)
    }
    /// Retain the canonical source represented by this occurrence without
    /// retaining the display. Both this scene and the coordinator's current
    /// document must still resolve the same source and leaf receipt.
    pub fn capture_brep_source_for(
        &self, coordinator: &TransactionCoordinator, object: &SceneObject,
    ) -> Result<om9_core::transaction::CapturedBrepSource> {
        self.entity_for(&self.document, object)?;
        let Some(BrepDisplayReceipt::Occt(fingerprint)) = &object.brep_display_fingerprint else {
            return Err(Error::Unsupported("scene occurrence is not an OCCT BRep"));
        };
        let leaf = om9_core::model::GeometryHandle {
            object: object.leaf,
            generation: object.leaf_generation,
            geometry_revision: object.leaf_geometry_revision,
        };
        coordinator.capture_brep_occurrence_source(&object.source, &object.path, leaf, *fingerprint)
    }
    /// Native counterpart: verifies the scene and current document against
    /// the same source/display owners before retaining only the payload.
    pub fn capture_native_brep_source_for(
        &self, coordinator: &TransactionCoordinator, object: &SceneObject,
    ) -> Result<om9_core::transaction::CapturedNativeBrepSource> {
        self.entity_for(&self.document, object)?;
        let Some(BrepDisplayReceipt::Native { payload, crc }) = &object.brep_display_fingerprint else {
            return Err(Error::Unsupported("scene occurrence is not a native BRep"));
        };
        let display = object.native_display_pin.as_ref().ok_or(Error::Missing("native BRep display pin"))?.display_weak();
        let leaf = om9_core::model::GeometryHandle {
            object: object.leaf,
            generation: object.leaf_generation,
            geometry_revision: object.leaf_geometry_revision,
        };
        coordinator.capture_native_brep_occurrence_source(&object.source, &object.path, leaf, payload, &display, *crc)
    }
}

fn resolve_scene_entity<'a>(document: &'a Document, object: &SceneObject) -> Result<&'a Entity> {
        if object.source.object != object.root {
            return Err(Error::Stale("scene root"));
        }
        let mut entity = document.resolve(&object.source)?;
        for id in &object.path {
            let Geometry::Instance(def) = entity.geometry else {
                return Err(Error::Invalid("scene member path"));
            };
            entity = document
                .definitions
                .get(&def)
                .ok_or(Error::Missing("scene definition"))?
                .members
                .iter()
                .find(|m| m.id == *id)
                .ok_or(Error::Missing("scene member"))?;
        }
        if entity.id != object.leaf {
            return Err(Error::Stale("scene leaf"));
        }
        if entity.generation != object.leaf_generation
            || entity.geometry_revision != object.leaf_geometry_revision
        {
            return Err(Error::Stale("scene leaf revision"));
        }
        if brep_display_fingerprint(entity) != object.brep_display_fingerprint {
            return Err(Error::Stale("scene BRep display source"));
        }
        Ok(entity)
}

impl crate::picking::PickScene for Scene {
    fn epoch(&self) -> u64 { self.epoch }
    fn publication_id(&self) -> uuid::Uuid { self.publication_id }
    fn objects(&self) -> &[SceneObject] { &self.objects }
    fn limits(&self) -> &Limits { &self.limits }
    fn document_matches(&self, document: &Document) -> bool { std::ptr::eq(document, self.document.as_ref()) }
    fn entity_for<'a>(&self, document: &'a Document, object: &SceneObject) -> Result<&'a Entity> { self.entity_for(document, object) }
    fn spatial_index(&self) -> Result<PickingIndex> { self.spatial_index() }
    fn object_spheres(&self) -> Result<ObjectSpheres> { self.object_spheres() }
}

impl crate::picking::PickScene for PickSnapshot {
    fn epoch(&self) -> u64 { self.epoch }
    fn publication_id(&self) -> uuid::Uuid { self.publication_id }
    fn objects(&self) -> &[SceneObject] { &self.objects }
    fn limits(&self) -> &Limits { &self.limits }
    fn document_matches(&self, document: &Document) -> bool { std::ptr::eq(self.document.as_ptr(), document) }
    fn entity_for<'a>(&self, document: &'a Document, object: &SceneObject) -> Result<&'a Entity> { resolve_scene_entity(document, object) }
    fn spatial_index(&self) -> Result<PickingIndex> { self.spatial_index() }
    fn object_spheres(&self) -> Result<ObjectSpheres> { self.object_spheres() }
}

impl PickSnapshot {
    pub fn additional_metadata_bytes() -> u64 {
        // The weak identity can outlive the document value. Its Arc allocation
        // remains until the last weak owner is dropped, so charge that storage
        // even while the tracked document also accounts for it.
        (std::mem::size_of::<Self>().saturating_sub(std::mem::size_of::<Scene>())
            + std::mem::size_of::<Document>() + 2 * std::mem::size_of::<usize>()) as u64
    }
    pub fn pick_with_document(&self, document: &Document, screen: [f64; 2], camera: &Camera,
        viewport: Viewport, tolerance_pixels: f64, intent: PickIntent) -> Result<Option<PickHit>> {
        crate::picking::pick(self, document, screen, camera, viewport, tolerance_pixels, intent)
    }
    pub fn picking_arena_bytes(&self) -> Result<u64> { Ok(self.arena_bytes) }
    pub fn nearest_pick_scratch_estimate(&self) -> Result<Usage> { Ok(self.query_scratch) }
    pub fn render_snapshot_pick_scratch_estimate(&self) -> Result<Usage> {
        let mut usage = self.query_scratch;
        usage.add(Usage { bytes: self.arena_bytes, ..Usage::default() }, &Limits { max_bytes: u64::MAX, ..Limits::default() })?;
        Ok(usage)
    }
}

impl HoverPickSnapshot {
    pub fn additional_metadata_bytes()->u64 {
        PickSnapshot::additional_metadata_bytes()+std::mem::size_of::<Self>().saturating_sub(std::mem::size_of::<PickSnapshot>()) as u64
    }
    pub fn nearest_pick_scratch_estimate(&self)->Result<Usage> {self.scene.nearest_pick_scratch_estimate()}
    pub fn picking_arena_bytes(&self)->Result<u64> {self.scene.picking_arena_bytes()}
    pub fn memory_allowance_bytes(&self)->u64 {self.scene.memory_allowance_bytes()}
    pub fn pick(&self,screen:[f64;2],camera:&Camera,viewport:Viewport,tolerance_pixels:f64,intent:PickIntent)->Result<Option<PickHit>> {
        let hits=crate::picking::pick_candidates_resolved(&self.scene,screen,camera,viewport,tolerance_pixels,intent,
            PickKind::Object,true,false,|index,object| {
                let source=self.sources.get(index).ok_or(Error::Stale("hover pick source"))?;
                let display=if source.brep_display {Some(object.brep_pick_display.as_ref().ok_or(Error::Stale("hover BRep pin"))?.display())} else {None};
                Ok(crate::picking::PickData {curve:source.curve.as_ref(),cloud:source.cloud,display})
            })?;
        Ok(hits.into_iter().next())
    }
}
