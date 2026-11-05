// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    model::Document,
    transaction::{TransactionCoordinator, World},
};
use om9_exchange::ExportCapture;

#[test]
fn equal_counters_and_shared_document_do_not_authorize_a_foreign_export_coordinator() {
    let document = Document::new("shared immutable allocation");
    let id = document.id;
    let world = World::new(vec![document], &Limits::default()).unwrap();
    let a = TransactionCoordinator::new(world.clone(), Limits::default()).unwrap();
    let b = TransactionCoordinator::new(world, Limits::default()).unwrap();
    let capture = ExportCapture::all(&a, id).unwrap();
    assert!(capture.revalidate(&b).is_err());
    assert!(capture.revalidate(&a).is_ok());
}

#[test]
fn a_caller_roundtrip_without_public_epoch_change_still_invalidates_export_capture() {
    let document = Document::new("caller lifecycle");
    let id = document.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, id).unwrap();
    let epoch = c.epoch();
    let owner = c.book("caller", false).unwrap();
    c.begin(&owner).unwrap();
    c.abort(&owner).unwrap();
    assert_eq!(c.epoch(), epoch);
    assert!(capture.revalidate(&c).is_err());
}

#[test]
fn plain_export_cannot_bypass_an_active_caller_scope() {
    let document = Document::new("caller private ownership");
    let id = document.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let owner = c.book("caller", true).unwrap();
    c.begin(&owner).unwrap();
    assert!(ExportCapture::all(&c, id).is_err());
    c.abort(&owner).unwrap();
    assert!(ExportCapture::all(&c, id).is_ok());
}

#[test]
fn export_capture_remains_charged_after_source_changes_and_history_is_absent() {
    use om9_core::{budget::Usage, geometry::{Cloud, Geometry, Transform}, model::Entity,
        transaction::{Mutation, TransactionRequest}};
    let limits = Limits::default();
    let mut document = Document::new("captured cloud");
    let entity = Entity::new("cloud", Geometry::Cloud(Cloud {
        points: vec![[1., 2., 3.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
    }), None);
    let object = entity.id;
    let id = document.id;
    document.entities.insert(object, entity);
    let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    coordinator.disable_undo_for_benchmark().unwrap();
    let capture = ExportCapture::all(&coordinator, id).unwrap();
    let source = coordinator.world().documents[&id].stamp(object).unwrap();
    let prepared = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
        source, placement: Transform::translation([1., 0., 0.]),
    }])).unwrap();
    coordinator.commit(prepared).unwrap();
    assert_eq!(coordinator.undo_len(), 0);
    let retained = coordinator.retained_usage_with_held_usage(Usage::default()).unwrap().bytes;
    drop(capture);
    let released = coordinator.retained_usage_with_held_usage(Usage::default()).unwrap().bytes;
    assert!(retained.saturating_sub(released) >= 1_000_000, "capture was not counted: retained={retained}, released={released}");
}

#[test]
fn selected_projection_reserves_shared_memory_before_copy_and_until_drop() {
    use om9_core::{budget::memory::MemoryClass, geometry::{Cloud, Geometry}, model::Entity};
    let limits = Limits { max_bytes: 8 * 1024 * 1024, ..Limits::default() };
    let mut document = Document::new("selected cloud");
    let entity = Entity::new("cloud", Geometry::Cloud(Cloud {
        points: vec![[1., 2., 3.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
    }), None);
    let object = entity.id;
    let id = document.id;
    document.entities.insert(object, entity);
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let competing = coordinator.reserve_external_memory(MemoryClass::Export, 6 * 1024 * 1024, None).unwrap();
    assert!(ExportCapture::selected(&coordinator, id, &[object]).is_err(), "copy must be refused before allocating against reserved memory");
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    drop(competing);
    let capture = ExportCapture::selected(&coordinator, id, &[object]).unwrap();
    assert!(coordinator.external_reserved_bytes(MemoryClass::CpuCache) >= 1_000_000);
    drop(capture);
    assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
}

#[test]
fn selected_projection_does_not_reserve_shared_resource_payload_twice() {
    use om9_core::{budget::memory::MemoryClass, geometry::Geometry, model::Entity,
        resource::{Resource, ResourceKind}};
    let mut document = Document::new("shared resource");
    let entity = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let object = entity.id;
    let id = document.id;
    document.entities.insert(object, entity);
    let resource = Resource::from_bytes("shared", ResourceKind::OpaqueBinary,
        &vec![42; 4 * 1024 * 1024], &Limits::default()).unwrap();
    let resource_id = resource.id;
    document.resources.insert(resource_id, resource);
    let limits = Limits { max_bytes: 6 * 1024 * 1024, ..Limits::default() };
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let capture = ExportCapture::selected(&coordinator, id, &[object]).unwrap();
    let source = coordinator.world().documents[&id].resources[&resource_id].payload().unwrap();
    let copied = capture.document().resources[&resource_id].payload().unwrap();
    assert!(std::ptr::eq(source, copied), "resource payload must be shared");
    assert!(coordinator.external_reserved_bytes(MemoryClass::CpuCache) < 1024 * 1024);
}

#[test]
fn selected_projection_does_not_reserve_shared_native_brep_payload_twice() {
    use om9_core::{budget::memory::MemoryClass, geometry::{Geometry, Mesh, NativeBrep}, model::Entity};
    let mut document = Document::new("shared native B-rep");
    let native = NativeBrep { display: Mesh { vertices: vec![], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] },
        payload: vec![7; 4 * 1024 * 1024].into(), crc: 1 };
    let entity = Entity::new("native", Geometry::NativeBrep(native), None);
    let object = entity.id;
    let id = document.id;
    document.entities.insert(object, entity);
    let limits = Limits { max_bytes: 6 * 1024 * 1024, ..Limits::default() };
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let capture = ExportCapture::selected(&coordinator, id, &[object]).unwrap();
    let Geometry::NativeBrep(source) = &coordinator.world().documents[&id].entities[&object].geometry else { panic!("source") };
    let Geometry::NativeBrep(copied) = &capture.document().entities[&object].geometry else { panic!("copy") };
    assert!(std::sync::Arc::ptr_eq(&source.payload, &copied.payload));
    assert!(coordinator.external_reserved_bytes(MemoryClass::CpuCache) < 1024 * 1024);
}

#[test]
fn selected_projection_does_not_reserve_shared_brep_source_twice() {
    use om9_core::{budget::memory::MemoryClass, brep::{Brep, BrepDisplay}, geometry::Geometry, model::Entity};
    let mut source = String::from("CASCADE Topology ");
    source.push_str(&"x".repeat(2 * 1024 * 1024));
    let brep = Brep::from_native(source, BrepDisplay {
        source_sha256: [0; 32], deflection_mm: 0.1, points: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
        triangles: vec![[0,1,2,0]], segments: vec![], vertices: vec![], faces: vec![uuid::Uuid::new_v4()],
        edges: vec![], vertex_ids: vec![], face_edges: vec![],
    }, &Limits::default()).unwrap();
    let mut document = Document::new("shared B-rep source");
    let entity = Entity::new("first", Geometry::Brep(brep.clone()), None);
    let object = entity.id;
    document.entities.insert(object, entity);
    let entity = Entity::new("second", Geometry::Brep(brep), None);
    document.entities.insert(entity.id, entity);
    let id = document.id;
    let limits = Limits { max_bytes: 4 * 1024 * 1024, ..Limits::default() };
    let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
    let capture = ExportCapture::selected(&coordinator, id, &[object]).unwrap();
    let Geometry::Brep(original) = &coordinator.world().documents[&id].entities[&object].geometry else { panic!("source") };
    let Geometry::Brep(copied) = &capture.document().entities[&object].geometry else { panic!("copy") };
    assert_eq!(original.source().as_ptr(), copied.source().as_ptr());
    assert!(coordinator.external_reserved_bytes(MemoryClass::CpuCache) < 1024 * 1024);
}
