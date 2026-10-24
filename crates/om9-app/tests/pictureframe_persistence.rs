// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-06-005`: reference images survive project save/reload.
use eframe::egui;
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use std::io::Cursor;

fn png() -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(100, 100, image::Rgba([12, 34, 56, 255]));
    let mut cursor = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .unwrap();
    cursor.into_inner()
}

#[test]
fn a_reference_image_round_trips_with_transform_opacity_and_calibration() {
    let mut app = MatrixApp::new(Document::new("picture frame")).unwrap();
    app.import_picture_frame("reference.png", png(), [10., 20., 0.], 20.)
        .unwrap();
    app.pictureframes[0].set_opacity(0.5).unwrap();
    app.pictureframes[0].calibrate(100., 25.).unwrap();
    app.pictureframes[0].set_locked(true);
    let hash = app.pictureframes[0].asset.sha_hex();
    let origin = app.pictureframes[0].origin;
    app.stage_project_state().unwrap();

    app.pictureframes.clear();
    app.reload_project_state().unwrap();
    assert_eq!(app.pictureframes.len(), 1);
    let frame = &app.pictureframes[0];
    assert_eq!(frame.origin, origin);
    assert_eq!(frame.opacity, 0.5);
    assert!(frame.locked);
    assert!((frame.mm_per_pixel() - 0.25).abs() < 1e-12);
    assert_eq!(frame.asset.sha_hex(), hash);
}

#[test]
fn a_bad_image_creates_no_quad() {
    let mut app = MatrixApp::new(Document::new("bad image")).unwrap();
    assert!(
        app.import_picture_frame("bad.png", b"not a png".to_vec(), [0.; 3], 20.)
            .is_err()
    );
    assert!(app.picture_frames().is_empty());
}

fn painted(app: &mut MatrixApp) -> egui::FullOutput {
    let context = egui::Context::default();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1280., 900.),
            )),
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}

fn mesh_alphas(shape: &egui::Shape) -> Vec<u8> {
    match shape {
        egui::Shape::Mesh(mesh) => mesh
            .vertices
            .iter()
            .map(|vertex| vertex.color.a())
            .collect(),
        egui::Shape::Vec(shapes) => shapes.iter().flat_map(mesh_alphas).collect(),
        _ => Vec::new(),
    }
}

#[test]
fn a_picture_frame_binds_a_texture_and_paints_opacity() {
    let mut app = MatrixApp::new(Document::new("picture texture")).unwrap();
    app.import_picture_frame("reference.png", png(), [0.; 3], 20.)
        .unwrap();
    app.pictureframes[0].set_opacity(0.5).unwrap();
    app.pictureframes[0].set_locked(true);
    let output = painted(&mut app);
    assert_eq!(app.pictureframe_texture_count(), 1);
    let alphas: Vec<u8> = output
        .shapes
        .iter()
        .flat_map(|clipped| mesh_alphas(&clipped.shape))
        .collect();
    assert!(
        alphas.contains(&128),
        "opacity 0.5 paints 128 alpha: {alphas:?}"
    );
}
