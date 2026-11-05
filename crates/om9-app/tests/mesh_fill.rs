// SPDX-License-Identifier: LGPL-2.1-or-later
//! A mesh object must hide the ground grid behind it, the way Matrix9 draws a shaded viewport.
//!
//! Mesh fill/occlusion uses actual GPU pixels; chrome stability still uses egui shapes.
#[path = "common/gpu_frames.rs"]
mod gpu_frames;
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
    let mut entity = Entity::new(
        "cube",
        Geometry::Mesh(Mesh {
            vertices,
            faces,
            normals: Vec::new(),
            colors: Vec::new(),

            corner_uvs: Vec::new(),
        }),
        None,
    );
    entity.attributes.color = Some([240, 80, 160, 255]);
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
    let Some(empty) = gpu_frames::render(&Document::new("empty")) else {
        return;
    };
    let Some(mesh) = gpu_frames::render(&mesh_document()) else {
        return;
    };
    let is_face =
        |p: &[u8; 4]| p[0] > 70 && p[2] > p[0] / 3 && u16::from(p[1]) * 4 < u16::from(p[0]) * 3;
    assert!(
        mesh[0].pixels().filter(|p| is_face(&p.0)).count() > 200,
        "cube face must cover an area, not just wire lines"
    );
    // The top face at z=5 must occlude both grid axes at z=0.
    for point in [[-2., 0., 5.], [2., 0., 5.], [0., -2., 5.], [0., 2., 5.]] {
        let (x, y) = gpu_frames::looking_down_pixel(point);
        assert!(
            is_face(&mesh[0].get_pixel(x, y).0),
            "face/grid depth at {point:?}"
        );
        assert!(!is_face(&empty[0].get_pixel(x, y).0));
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
