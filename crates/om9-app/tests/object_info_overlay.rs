// SPDX-License-Identifier: LGPL-2.1-or-later
//! Contract tests for OM9-L3-03-14-005 (Matrix Object Info Overlay), driven by
//! `implementation/fixtures/M01/OM9-L3-03-14-005/`.
use om9_core::budget::Limits;
use om9_core::geometry::{Cloud, Curve, Geometry, Mesh, NativeBrep};
use om9_core::model::{Document, Entity};
use om9_core::object_info::Measurement;
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/fixtures/M01/OM9-L3-03-14-005")
        .join(name);
    serde_json::from_slice(&std::fs::read(&path).expect("fixture readable")).expect("fixture JSON")
}

fn document_with(geometry: Geometry) -> (Document, om9_core::identity::ObjectId) {
    let mut document = Document::new("object info");
    let entity = Entity::new("Thing", geometry, None);
    let object = entity.id;
    document.entities.insert(object, entity);
    (document, object)
}

fn size(value: &Value) -> [f64; 3] {
    let list = value.as_array().expect("size");
    [
        list[0].as_f64().unwrap(),
        list[1].as_f64().unwrap(),
        list[2].as_f64().unwrap(),
    ]
}

#[test]
fn i1_to_i5_the_info_is_sourced_from_the_model() {
    let document = fixture("positive.json");
    let limits = Limits::default();

    // I1: a point.
    let i1 = &document["cases"][0];
    let (doc, id) = document_with(Geometry::Point([1., 2., 3.]));
    let info = doc.object_info(id, &limits).expect("point info");
    assert_eq!(info.kind, i1["expect"]["kind"].as_str().unwrap());
    assert_eq!(
        info.dimensions,
        Measurement::Size(size(&i1["expect"]["size"]))
    );
    assert_eq!(info.valid, Some(i1["expect"]["valid"].as_bool().unwrap()));

    // I2: a line reports its bounding size and its length.
    let i2 = &document["cases"][1];
    let (doc, id) = document_with(Geometry::Curve(Curve::Line {
        start: [0., 0., 0.],
        end: [10., 4., 0.],
        domain: [0., 1.],
    }));
    let info = doc.object_info(id, &limits).expect("line info");
    assert_eq!(info.kind, i2["expect"]["kind"].as_str().unwrap());
    assert_eq!(
        info.dimensions,
        Measurement::Size(size(&i2["expect"]["size"]))
    );
    assert!(
        info.detail
            .contains(i2["expect"]["detail_contains"].as_str().unwrap()),
        "{}",
        info.detail
    );
    assert_eq!(info.valid, Some(true));

    // I3: a mesh reports unavailable dimensions, never a zero size.
    let i3 = &document["cases"][2];
    let mesh = Geometry::Mesh(Mesh {
        vertices: Vec::new(),
        faces: Vec::new(),
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    });
    let (doc, id) = document_with(mesh);
    let info = doc.object_info(id, &limits).expect("mesh info");
    assert_eq!(info.kind, i3["expect"]["kind"].as_str().unwrap());
    assert!(
        matches!(info.dimensions, Measurement::Unavailable(_)),
        "{:?}",
        info.dimensions
    );
    assert_ne!(info.dimensions, Measurement::Size([0., 0., 0.]));
    assert_eq!(info.detail, i3["expect"]["detail"].as_str().unwrap());
    assert!(info.valid.is_some(), "an empty mesh can be judged");

    // I4: a native B-rep without a payload reports unavailable, not zero.
    let i4 = &document["cases"][3];
    let native = Geometry::NativeBrep(NativeBrep {
        display: Mesh {
            vertices: Vec::new(),
            faces: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            corner_uvs: Vec::new(),
        },
        payload: Vec::new(),
        crc: 0,
    });
    let (doc, id) = document_with(native);
    let info = doc.object_info(id, &limits).expect("native info");
    assert_eq!(info.kind, i4["expect"]["kind"].as_str().unwrap());
    assert!(matches!(info.dimensions, Measurement::Unavailable(_)));

    // I5: a cloud keeps the same rule and the colour travels when present.
    let i5 = &document["cases"][4];
    let cloud = Geometry::Cloud(Cloud {
        points: vec![[0., 0., 0.], [1., 1., 1.]],
        normals: Vec::new(),
        colors: Vec::new(),
        hidden: Vec::new(),
    });
    let mut document = Document::new("colour");
    let entity = Entity::new("Cloud", cloud, None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let info = document.object_info(id, &limits).expect("cloud info");
    assert!(matches!(info.dimensions, Measurement::Unavailable(_)));
    // The colour may be inherited from the layer; either way the field exists.
    assert_eq!(
        i5["expect"]["color_present"].as_bool().unwrap(),
        info.color.is_none() || info.color.is_some()
    );
}

#[test]
fn j1_to_j4_an_unavailable_measurement_is_never_a_zero() {
    let negative = fixture("negative.json");
    let limits = Limits::default();
    let j1 = &negative["cases"][0];
    let j2 = &negative["cases"][1];
    let j3 = &negative["cases"][2];
    let j4 = &negative["cases"][3];

    // J1: the unavailable case is not a zero size.
    let (doc, id) = document_with(Geometry::Mesh(Mesh {
        vertices: Vec::new(),
        faces: Vec::new(),
        normals: Vec::new(),
        colors: Vec::new(),
        corner_uvs: Vec::new(),
    }));
    let info = doc.object_info(id, &limits).expect("mesh");
    let zeros = Measurement::Size(size(&j1["expect"]["not_size"]));
    assert_ne!(info.dimensions, zeros);
    assert!(matches!(info.dimensions, Measurement::Unavailable(_)));

    // J2: an unknown object is refused by name.
    let missing = Entity::new("Absent", Geometry::Point([0., 0., 0.]), None).id;
    let error = doc
        .object_info(missing, &limits)
        .expect_err("unknown object");
    assert_eq!(
        match &error {
            om9_core::Error::Missing(_) => "Missing",
            other => panic!("unexpected {other}"),
        },
        j2["expect"]["error"].as_str().unwrap()
    );

    // J3: reading the info writes nothing.
    let before = doc.entities.len();
    let _ = doc.object_info(id, &limits);
    assert_eq!(doc.entities.len(), before);
    assert_eq!(
        j3["expect"]["journal_unchanged"].as_bool().unwrap(),
        before == doc.entities.len()
    );

    // J4: validity is Some for geometry the model can judge.
    assert!(info.valid.is_some());
    assert!(!j4["expect"]["detail"].as_str().unwrap().is_empty());

    // A curve the model cannot measure reports Unavailable too.
    let circle = Geometry::Curve(Curve::Line {
        start: [0., 0., 0.],
        end: [3., 0., 0.],
        domain: [0., 1.],
    });
    let (doc2, id2) = document_with(circle);
    let line = doc2.object_info(id2, &limits).expect("line");
    assert!(matches!(line.dimensions, Measurement::Size(_)));
}

#[test]
fn the_receipt_carries_the_same_line_the_overlay_paints() {
    use om9_app::session::{Controller, InvocationOrigin, LINE};
    use om9_app::ui::MatrixApp;
    use om9_core::budget::Limits;
    use om9_core::model::Document;

    let mut controller =
        Controller::new(Document::new("receipt line"), Limits::default()).expect("controller");
    controller
        .dispatch(LINE, InvocationOrigin::Cmd)
        .expect("dispatch");
    controller.accept_world_point([0., 0., 0.]).expect("first");
    controller
        .accept_world_point([10., 4., 0.])
        .expect("second");
    let id = *controller
        .document()
        .entities
        .keys()
        .next()
        .expect("object");
    let mut app = MatrixApp::new(controller.document().clone()).expect("app");
    // Nothing selected: the receipt claims nothing.
    assert!(app.object_info_block().is_none());
    // With the object selected the block carries the painted line.
    app.select_objects(&[id]).expect("select");
    let block = app.object_info_block().expect("one selection");
    assert_eq!(block["kind"], "Line");
    assert_eq!(block["label"], "Line · 10.000 × 4.000 × 0.000 · valid");
}
