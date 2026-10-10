#![allow(dead_code)]
use openmatrix9_rust::layer_state::{ColorSource, LayerRow, LayerSnapshotV1, ObjectLayerRow};

pub fn layer(id: &str, parent: Option<&str>, names: &[&str]) -> LayerRow {
    LayerRow {
        source_id: id.into(),
        parent_id: parent.map(String::from),
        name: names.last().unwrap().to_string(),
        path_components: names.iter().map(|s| s.to_string()).collect(),
        rgb: [34, 153, 135],
        locked: false,
        visible: true,
        persistent_locked: None,
        persistent_visible: None,
    }
}
pub fn object(id: &str, layer: &str) -> ObjectLayerRow {
    ObjectLayerRow {
        id: id.into(),
        layer_id: layer.into(),
        locked: false,
        visible: true,
        color_source: ColorSource::ByLayer,
        rgb: [9, 8, 7],
    }
}
pub fn snapshot() -> LayerSnapshotV1 {
    LayerSnapshotV1 {
        document_id: "test-document".into(),
        generation: 7,
        active_layer: Some("metal".into()),
        layers: vec![
            layer("metal", None, &["Metal 01"]),
            layer("gem", None, &["Gem 01"]),
            layer("detail", Some("metal"), &["Metal 01", "Detail"]),
        ],
        objects: vec![
            object("ring", "metal"),
            object("stone", "gem"),
            object("detail-curve", "detail"),
        ],
    }
}
