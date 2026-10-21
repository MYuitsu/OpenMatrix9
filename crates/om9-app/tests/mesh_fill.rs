// SPDX-License-Identifier: LGPL-2.1-or-later
//! A mesh object must hide the ground grid behind it, the way Matrix9 draws a shaded viewport.
//!
//! The grid is painted first and the object after it, so the only way the grid can show through the
//! object is a missing face fill. This test counts the filled shapes the viewport emits for a mesh and
//! checks that they come after the grid lines in the paint order.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Shape, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh},
    model::{Document, Entity},
};

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

/// A cube mesh entity, so a mesh exists without needing an importer.
fn mesh_document() -> Document {
    let mut document = Document::new("mesh fill");
    let vertices = vec![
        [-5.0, -5.0, -5.0],
        [5.0, -5.0, -5.0],
        [5.0, 5.0, -5.0],
        [-5.0, 5.0, -5.0],
        [-5.0, -5.0, 5.0],
        [5.0, -5.0, 5.0],
        [5.0, 5.0, 5.0],
        [-5.0, 5.0, 5.0],
    ];
    let faces = vec![
        vec![0, 2, 1],
        vec![0, 3, 2],
        vec![4, 5, 6],
        vec![4, 6, 7],
        vec![0, 1, 5],
        vec![0, 5, 4],
        vec![2, 3, 7],
        vec![2, 7, 6],
        vec![1, 2, 6],
        vec![1, 6, 5],
        vec![0, 4, 7],
        vec![0, 7, 3],
    ];
    let entity = Entity::new(
        "cube",
        Geometry::Mesh(Mesh {
            vertices,
            faces,
            normals: Vec::new(),
            colors: Vec::new(),
        }),
        None,
    );
    let id = entity.id;
    document.entities.insert(id, entity);
    document
}

fn run(
    app: &mut MatrixApp,
    context: &egui::Context,
    frames: usize,
) -> Vec<egui::epaint::ClippedShape> {
    let mut shapes = Vec::new();
    for _ in 0..frames {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(screen_rect()),
                events: vec![Event::ModifiersChanged(Modifiers::NONE)],
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
        shapes = output.shapes;
    }
    shapes
}

fn filled(shapes: &[egui::epaint::ClippedShape]) -> usize {
    shapes
        .iter()
        .filter(|clipped| match &clipped.shape {
            Shape::Path(path) => path.fill != eframe::egui::Color32::TRANSPARENT,
            _ => false,
        })
        .count()
}

#[test]
fn a_mesh_object_fills_its_faces_after_the_grid() {
    let context = egui::Context::default();
    let mut empty = MatrixApp::new(Document::new("empty")).expect("app");
    let empty_shapes = run(&mut empty, &context, 4);
    let empty_fills = filled(&empty_shapes);
    let context = egui::Context::default();
    let mut app = MatrixApp::new(mesh_document()).expect("app");
    let shapes = run(&mut app, &context, 4);
    let faces = filled(&shapes);
    assert!(
        faces >= empty_fills + 12,
        "each of the twelve cube triangles needs a fill: {faces} vs {empty_fills} in an empty document"
    );
    // Within one viewport rect, the grid lines must be painted before the object's fills.
    let rect = app.view_rects[&om9_scene::workspace::ViewId::Perspective];
    let in_view: Vec<&egui::epaint::ClippedShape> = shapes
        .iter()
        .filter(|clipped| clipped.clip_rect == rect)
        .collect();
    let first_line = in_view
        .iter()
        .position(|clipped| matches!(&clipped.shape, Shape::LineSegment { .. }));
    let first_fill = in_view.iter().position(|clipped| match &clipped.shape {
        Shape::Path(path) => path.fill != eframe::egui::Color32::TRANSPARENT,
        _ => false,
    });
    if let (Some(first_line), Some(first_fill)) = (first_line, first_fill) {
        assert!(
            first_fill > first_line,
            "the grid starts at {first_line}, the first fill at {first_fill}"
        );
    }
}

#[test]
fn a_document_without_meshes_draws_no_extra_fills() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("empty")).expect("app");
    let first = run(&mut app, &context, 4);
    let second = run(&mut app, &context, 2);
    assert_eq!(
        filled(&first),
        filled(&second),
        "the chrome alone must not change its fill count between frames"
    );
    let _ = Limits::default();
}
