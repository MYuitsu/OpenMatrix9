// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Transform, Unit},
    model::{Document, Entity},
    transaction::*,
};
use om9_exchange::ExportCapture;

#[test]
fn selected_copy_keeps_per_document_points_but_charges_both_copies_bytes() {
    use om9_core::{Error, geometry::Mesh};
    let mut document = Document::new("copy budget");
    let mesh = Mesh { vertices: vec![[0., 0., 0.]; 10_000], faces: vec![], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    let id = entity.id;
    let document_id = document.id;
    document.entities.insert(id, entity);
    let tight = Limits { max_points: 10_000, max_bytes: 400_000, ..Limits::default() };
    let world = World::new(vec![document.clone()], &tight).unwrap();
    let controller = TransactionCoordinator::new(world, tight).unwrap();
    assert!(matches!(ExportCapture::selected(&controller, document_id, &[id]), Err(Error::Budget("bytes"))));
    let enough = Limits { max_bytes: 2_000_000, ..tight };
    let world = World::new(vec![document], &enough).unwrap();
    let controller = TransactionCoordinator::new(world, enough).unwrap();
    assert_eq!(ExportCapture::selected(&controller, document_id, &[id]).unwrap().document().entities.len(), 1);
}

#[test]
fn selected_capture_excludes_other_objects_and_rejects_stale_source() {
    let mut d = Document::new("selection");
    let selected = Entity::new("selected", Geometry::Point([1., 2., 3.]), None);
    let id = selected.id;
    let other = Entity::new("other", Geometry::Point([4., 5., 6.]), None);
    let other_id = other.id;
    d.entities.insert(id, selected);
    d.entities.insert(other_id, other);
    let did = d.id;
    let mut c = TransactionCoordinator::new(World::new(vec![d], &Limits::default()).unwrap(), Limits::default()).unwrap();
    assert!(ExportCapture::selected(&c, did, &[]).is_err());
    assert!(ExportCapture::selected(&c, did, &[id, id]).is_err());
    let capture = ExportCapture::selected(&c, did, &[id]).unwrap();
    let packet: serde_json::Value = serde_json::from_slice(&capture.encode(&Limits::default()).unwrap()).unwrap();
    let entities = packet["document"]["entities"].as_object().unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[&id.to_string()]["geometry"]["Point"], serde_json::json!([1., 2., 3.]));
    let edit = c.prepare(TransactionRequest::new("move", vec![Mutation::Placement {
        source: c.world().documents[&did].stamp(id).unwrap(),
        placement: Transform::translation([5., 0., 0.]),
    }])).unwrap();
    c.commit(edit).unwrap();
    assert!(capture.revalidate(&c).is_err());
}

#[test]
fn selected_capture_keeps_only_referenced_nested_block_definitions() {
    use om9_core::model::Definition;
    let mut d = Document::new("blocks");
    let inner = Definition::new("inner", vec![Entity::new("p", Geometry::Point([1., 2., 3.]), None)]);
    let inner_id = inner.id;
    let outer = Definition::new("outer", vec![Entity::new("nested", Geometry::Instance(inner_id), None)]);
    let outer_id = outer.id;
    let unused = Definition::new("unused", vec![Entity::new("private", Geometry::Point([9., 9., 9.]), None)]);
    let unused_id = unused.id;
    d.definitions.insert(inner_id, inner);
    d.definitions.insert(outer_id, outer);
    d.definitions.insert(unused_id, unused);
    let instance = Entity::new("instance", Geometry::Instance(outer_id), None);
    let id = instance.id;
    d.entities.insert(id, instance);
    let did = d.id;
    let c = TransactionCoordinator::new(World::new(vec![d], &Limits::default()).unwrap(), Limits::default()).unwrap();
    let capture = ExportCapture::selected(&c, did, &[id]).unwrap();
    let packet: serde_json::Value = serde_json::from_slice(&capture.encode(&Limits::default()).unwrap()).unwrap();
    let definitions = packet["document"]["definitions"].as_object().unwrap();
    assert_eq!(definitions.len(), 2);
    assert!(definitions.contains_key(&inner_id.to_string()));
    assert!(definitions.contains_key(&outer_id.to_string()));
    assert!(!definitions.contains_key(&unused_id.to_string()));
}
#[test]
fn owned_current_snapshot_packet_keeps_geometry_placement_ids_units_and_unicode_attributes() {
    let mut d = Document::new("Nhẫn");
    d.page_unit = Unit::Inch;
    let mut e = Entity::new(
        "Đường",
        Geometry::Curve(Curve::Line {
            start: [1., 2., 3.],
            end: [4., 6., 3.],
            domain: [2., 4.],
        }),
        None,
    );
    e.placement = Transform::translation([5., 0., 0.]);
    let id = e.id;
    d.entities.insert(id, e);
    let did = d.id;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, did).unwrap();
    let packet = capture.encode(&Limits::default()).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["mode"], "geometry_only_current");
    assert_eq!(value["document"]["page_unit"], "Inch");
    assert_eq!(
        value["document"]["entities"][id.to_string()]["label"],
        "Đường"
    );
    assert_eq!(
        value["document"]["entities"][id.to_string()]["placement"][0][3],
        5.
    );
    assert_eq!(
        value["document"]["entities"][id.to_string()]["geometry"]["Curve"]["Line"]["domain"],
        serde_json::json!([2., 4.])
    );
    capture.revalidate(&c).unwrap();
}
#[test]
fn intervening_commit_undo_and_close_make_export_capture_stale_without_resurrecting_source() {
    let mut d = Document::new("source");
    let e = Entity::new("A", Geometry::Point([1., 2., 3.]), None);
    let id = e.id;
    d.entities.insert(id, e);
    let did = d.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let capture = ExportCapture::all(&c, did).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "Move",
            vec![Mutation::Placement {
                source: c.world().documents[&did].stamp(id).unwrap(),
                placement: Transform::translation([5., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    assert!(capture.revalidate(&c).is_err());
    c.close_document(did).unwrap();
    assert!(capture.revalidate(&c).is_err());
}
#[test]
fn delete_then_create_capture_contains_current_state_and_packet_budget_checks_before_growth() {
    let mut d = Document::new("current state");
    let e = Entity::new("B", Geometry::Point([1., 2., 3.]), None);
    let id = e.id;
    d.entities.insert(id, e);
    let did = d.id;
    let mut c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "Delete",
            vec![Mutation::Delete {
                source: c.world().documents[&did].stamp(id).unwrap(),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let new = Entity::new("C", Geometry::Point([9., 8., 7.]), None);
    let new_id = new.id;
    let p = c
        .prepare(TransactionRequest::new(
            "Create",
            vec![Mutation::Insert {
                document: did,
                entity: new,
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    let capture = ExportCapture::all(&c, did).unwrap();
    let packet = capture.encode(&Limits::default()).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert!(value["document"]["entities"].get(id.to_string()).is_none());
    assert_eq!(value["document"]["entities"].as_object().unwrap().len(), 1);
    assert!(
        value["document"]["entities"]
            .get(new_id.to_string())
            .is_some()
    );
    assert!(
        capture
            .encode(&Limits {
                max_bytes: 256,
                ..Limits::default()
            })
            .is_err()
    );
}
