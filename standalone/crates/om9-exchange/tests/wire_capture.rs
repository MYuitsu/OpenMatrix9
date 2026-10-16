// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Transform, Unit},
    model::{Document, Entity},
    transaction::*,
};
use om9_exchange::ExportCapture;
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
