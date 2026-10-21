// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(all(windows, feature = "occt-brep"))]

use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Hatch, Mesh},
    model::{Document, Entity, Layer},
};

#[test]
#[ignore = "requires OM9_EXCHANGE_NATIVE and the OCCT runtime on PATH"]
fn rhino5_hatch_geometry_round_trips_as_display_proxy() {
    let mut document = Document::new("hatch round trip");
    let layer = Layer::new("Layer 01", [10, 20, 30, 255]);
    document.current_layer = Some(layer.id);
    document.layer_order.push(layer.id);
    document.layers.insert(layer.id, layer.clone());
    let hatch = Hatch {
        display: Mesh {
            vertices: vec![[0., 0., 0.], [10., 0., 0.], [0., 10., 0.]],
            faces: vec![vec![0, 1, 2]],
            normals: vec![],
            colors: vec![],
        },
        loops: vec![],
        pattern: Some("#0".into()),
        color: [180, 120, 20, 255],
    };
    let entity = Entity::new("hatch", Geometry::Hatch(hatch), layer.id);
    document.entities.insert(entity.id, entity.clone());
    let controller = Controller::new(document, Limits::default()).unwrap();
    let root = std::env::temp_dir().join(format!("om9-hatch-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("hatch.3dm");
    controller.export_3dm(&path).unwrap();

    let mut reopened = Controller::new(Document::new("reopen"), Limits::default()).unwrap();
    reopened.import_3dm(&path).unwrap();
    let imported = reopened
        .document()
        .entities
        .values()
        .find(|entity| matches!(entity.geometry, Geometry::Hatch(_)))
        .expect("imported hatch");
    let Geometry::Hatch(hatch) = &imported.geometry else {
        unreachable!()
    };
    assert!(hatch.display.vertices.len() >= 3);
    assert!(!hatch.loops.is_empty());
    assert_eq!(hatch.pattern.as_deref(), Some("#0"));
    let _ = std::fs::remove_dir_all(root);
}

