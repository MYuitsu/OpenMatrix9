// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-05-001/005`: display mode switching is presentation-only and preserves selection.
use om9_app::{
    display::DisplayPreferences,
    session::Controller,
    viewport_menu::{DisplayPreset, MenuAction},
};
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh},
    model::{Document, Entity},
};
use om9_scene::workspace::{DisplayMode, ViewId};

fn box_mesh() -> Geometry {
    Geometry::Mesh(Mesh {
        vertices: vec![
            [0., 0., 0.],
            [10., 0., 0.],
            [10., 20., 0.],
            [0., 20., 0.],
            [0., 0., 30.],
            [10., 0., 30.],
            [10., 20., 30.],
            [0., 20., 30.],
        ],
        faces: vec![
            vec![0, 1, 2, 3],
            vec![4, 5, 6, 7],
            vec![0, 1, 5, 4],
            vec![1, 2, 6, 5],
            vec![2, 3, 7, 6],
            vec![3, 0, 4, 7],
        ],
        normals: Vec::new(),
        colors: Vec::new(),

        corner_uvs: Vec::new(),
    })
}

fn bbox(controller: &Controller) -> ([f64; 3], [f64; 3]) {
    let scene = controller.scene().expect("scene");
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for object in &scene.objects {
        let transform = object.display_transform().unwrap();
        if let om9_scene::scene::RenderBuffers::Mesh { vertices, .. } = object.buffers.as_ref() {
            for vertex in vertices {
                let point = transform.point(*vertex).unwrap();
                for axis in 0..3 {
                    min[axis] = min[axis].min(point[axis]);
                    max[axis] = max[axis].max(point[axis]);
                }
            }
        }
    }
    (min, max)
}

#[test]
fn wireframe_and_shaded_show_the_same_box_and_do_not_write_the_model() {
    let entity = Entity::new("box", box_mesh(), None);
    let id = entity.id;
    let mut document = Document::new("display modes");
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let before_bbox = bbox(&controller);
    let before_entities = controller.document().entities.clone();
    let before_snapshot = controller.coordinator().snapshot();
    let before_selection = controller.selected().clone();
    let menu = controller.capture_view_menu(ViewId::Perspective).unwrap();
    controller
        .apply_view_menu(&menu, MenuAction::Preset(DisplayPreset::Shaded))
        .unwrap();
    assert_eq!(
        controller.workspace.views[&ViewId::Perspective].mode,
        DisplayMode::Shaded
    );
    assert_eq!(bbox(&controller), before_bbox);
    assert_eq!(controller.document().entities, before_entities);
    assert_eq!(controller.coordinator().snapshot(), before_snapshot);
    assert_eq!(controller.selected(), &before_selection);

    let menu = controller.capture_view_menu(ViewId::Perspective).unwrap();
    controller
        .apply_view_menu(&menu, MenuAction::Preset(DisplayPreset::Wireframe))
        .unwrap();
    assert_eq!(
        controller.workspace.views[&ViewId::Perspective].mode,
        DisplayMode::Wireframe
    );
    assert_eq!(bbox(&controller), before_bbox);
}

#[test]
fn a_display_preset_restores_wireframe_and_edge_preferences() {
    let preset = DisplayPreferences {
        wireframe: false,
        show_edges: false,
        ..DisplayPreferences::default()
    };
    let text = preset.to_json().unwrap();
    let (loaded, warning) = DisplayPreferences::from_json(&text);
    assert!(warning.is_none());
    assert!(!loaded.wireframe);
    assert!(!loaded.show_edges);
    let mut mode = DisplayMode::Wireframe;
    loaded.apply_to_mode(&mut mode);
    assert_eq!(mode, DisplayMode::Shaded);
}

#[test]
fn a_mode_switch_keeps_selected_ids_and_visibility_flags() {
    let mut document = Document::new("selected display");
    let first = Entity::new("first", box_mesh(), None);
    let first_id = first.id;
    let second = Entity::new("second", box_mesh(), None);
    let second_id = second.id;
    document.entities.insert(first_id, first);
    document.entities.insert(second_id, second);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[first_id, second_id]).unwrap();
    let menu = controller.capture_view_menu(ViewId::Perspective).unwrap();
    controller
        .apply_view_menu(&menu, MenuAction::Preset(DisplayPreset::Shaded))
        .unwrap();
    assert_eq!(
        controller.selected(),
        &[first_id, second_id].into_iter().collect()
    );
    assert!(
        controller
            .scene()
            .unwrap()
            .objects
            .iter()
            .all(|object| object.visible)
    );
}
