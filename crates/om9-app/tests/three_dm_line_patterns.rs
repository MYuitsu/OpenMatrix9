// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(all(windows, feature = "occt-brep"))]

use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    exchange::{
        DimStyle, HatchPattern, HatchPatternLine, LinePattern, LinePatternSegment, RenderMaterial,
    },
    geometry::{Curve, Geometry},
    identity::{DimStyleId, HatchPatternId, LinePatternId, MaterialId},
    model::{Document, Entity, Layer},
};

#[test]
#[ignore = "requires OM9_EXCHANGE_NATIVE and the OCCT runtime on PATH"]
fn rhino5_line_pattern_table_round_trips() {
    let mut document = Document::new("line pattern round trip");
    let id = LinePatternId::new();
    document.exchange.line_patterns.insert(
        id,
        LinePattern {
            id,
            name: "Dash 2-1".into(),
            segments: vec![
                LinePatternSegment {
                    length_mm: 2.,
                    segment_type: 1,
                },
                LinePatternSegment {
                    length_mm: 1.,
                    segment_type: 2,
                },
            ],
        },
    );
    let material_id = MaterialId::new();
    document.exchange.materials.insert(
        material_id,
        RenderMaterial {
            id: material_id,
            name: "Brass".into(),
            diffuse: [180, 120, 20, 255],
        },
    );
    let dim_style_id = DimStyleId::new();
    document.exchange.dim_styles.insert(
        dim_style_id,
        DimStyle {
            id: dim_style_id,
            name: "Dim Style A".into(),
            text_height: 2.5,
            length_resolution: 3.,
        },
    );
    let hatch_pattern_id = HatchPatternId::new();
    document.exchange.hatch_patterns.insert(
        hatch_pattern_id,
        HatchPattern {
            id: hatch_pattern_id,
            name: "Hatch A".into(),
            fill_type: 1,
            lines: vec![HatchPatternLine {
                angle_radians: 0.5,
                base: [1., 2.],
                offset: [3., 4.],
                dashes: vec![5., 6.],
            }],
        },
    );
    let mut layer = Layer::new("Layer 01", [10, 20, 30, 255]);
    layer.line_pattern = Some(id);
    layer.line_pattern_source = 1;
    layer.material = Some(material_id);
    layer.plot_color = Some([4, 5, 6, 255]);
    layer.plot_weight_mm = Some(0.35);
    document.current_layer = Some(layer.id);
    document.layer_order.push(layer.id);
    document.layers.insert(layer.id, layer.clone());
    let mut entity = Entity::new(
        "dashed line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        layer.id,
    );
    entity.attributes.line_pattern = Some(id);
    entity.attributes.line_pattern_source = 1;
    entity.attributes.material = Some(material_id);
    entity.attributes.material_source = 1;
    entity.attributes.plot_color = Some([7, 8, 9, 255]);
    entity.attributes.plot_weight_mm = Some(0.5);
    entity.attributes.plot_color_source = Some(1);
    entity.attributes.plot_weight_source = Some(1);
    entity.attributes.display_order = 7;
    entity.attributes.space = 1;
    entity.attributes.viewport_id = None;
    document.entities.insert(entity.id, entity.clone());
    let mut page_entity = Entity::new(
        "page line",
        Geometry::Curve(Curve::Line {
            start: [0., 1., 0.],
            end: [10., 1., 0.],
            domain: [0., 1.],
        }),
        layer.id,
    );
    page_entity.attributes.space = 2;
    document
        .entities
        .insert(page_entity.id, page_entity.clone());
    let controller = Controller::new(document, Limits::default()).unwrap();
    let root = std::env::temp_dir().join(format!("om9-linetype-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("linetypes.3dm");
    controller.export_3dm(&path).unwrap();

    let mut reopened = Controller::new(Document::new("reopen"), Limits::default()).unwrap();
    reopened.import_3dm(&path).unwrap();
    let patterns = &reopened.document().exchange.line_patterns;
    assert_eq!(patterns.len(), 1);
    assert_eq!(
        reopened
            .document()
            .exchange
            .hatch_patterns
            .values()
            .find(|pattern| pattern.name == "Hatch A")
            .unwrap()
            .fill_type,
        1
    );
    let reopened_hatch = reopened
        .document()
        .exchange
        .hatch_patterns
        .values()
        .find(|pattern| pattern.name == "Hatch A")
        .unwrap();
    assert_eq!(reopened_hatch.lines.len(), 1);
    assert_eq!(reopened_hatch.lines[0].angle_radians, 0.5);
    assert_eq!(reopened_hatch.lines[0].dashes, vec![5., 6.]);
    assert_eq!(
        reopened
            .document()
            .exchange
            .dim_styles
            .values()
            .find(|style| style.name == "Dim Style A")
            .unwrap()
            .text_height,
        2.5
    );
    assert_eq!(
        reopened
            .document()
            .exchange
            .materials
            .values()
            .find(|material| material.name == "Brass")
            .unwrap()
            .diffuse,
        [180, 120, 20, 255]
    );
    let pattern = patterns.values().next().unwrap();
    assert_eq!(pattern.name, "Dash 2-1");
    assert_eq!(
        pattern.segments,
        [
            LinePatternSegment {
                length_mm: 2.,
                segment_type: 1
            },
            LinePatternSegment {
                length_mm: 1.,
                segment_type: 2
            }
        ]
    );
    let reopened_layer = reopened
        .document()
        .layers
        .values()
        .find(|layer| layer.name == "Layer 01")
        .unwrap();
    assert_eq!(reopened_layer.line_pattern, Some(id));
    assert_eq!(reopened_layer.material, Some(material_id));
    assert_eq!(reopened_layer.plot_color, Some([4, 5, 6, 255]));
    assert_eq!(reopened_layer.plot_weight_mm, Some(0.35));
    let reopened_entity = reopened
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "dashed line")
        .unwrap();
    assert_eq!(reopened_entity.attributes.line_pattern, Some(id));
    assert_eq!(reopened_entity.attributes.material, Some(material_id));
    assert_eq!(reopened_entity.attributes.material_source, 1);
    assert_eq!(reopened_entity.attributes.plot_color, Some([7, 8, 9, 255]));
    assert_eq!(reopened_entity.attributes.plot_weight_mm, Some(0.5));
    assert_eq!(reopened_entity.attributes.plot_color_source, Some(1));
    assert_eq!(reopened_entity.attributes.plot_weight_source, Some(1));
    assert_eq!(reopened_entity.attributes.display_order, 7);
    assert_eq!(reopened_entity.attributes.space, 1);
    let reopened_page = reopened
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "page line")
        .unwrap();
    assert_eq!(reopened_page.attributes.space, 2);
    assert_eq!(
        reopened_entity.attributes.viewport_id,
        entity.attributes.viewport_id
    );
    let _ = std::fs::remove_dir_all(root);
}
