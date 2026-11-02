// SPDX-License-Identifier: LGPL-2.1-or-later
//! Headless viewport capture: one PNG per workspace camera, with no window, egui or swapchain.
//! The test skips when the machine exposes no wgpu adapter.
use om9_app::headless;
use om9_archive::Archive;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh},
    model::{Document, Entity},
    transaction::Cancellation,
};

#[test]
fn a_project_renders_one_png_per_viewport_without_a_window() {
    if om9_render::Renderer::new().is_none() {
        eprintln!("skip: no wgpu adapter on this machine");
        return;
    }
    let directory = std::env::temp_dir().join(format!("om9-headless-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("test directory");
    let project = directory.join("triangle.om9");
    let mut document = Document::new("headless");
    let mesh = Mesh {
        vertices: vec![[0., 0., 0.], [10., 0., 0.], [0., 10., 0.]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
    document.entities.insert(entity.id, entity);
    Archive::save(
        &project,
        &document,
        &Limits::default(),
        &Cancellation::default(),
    )
    .expect("fixture project");
    let files = headless::render_project(&project, &directory, 96, 64).expect("headless render");
    assert_eq!(files.len(), 4, "one image per workspace viewport");
    for file in &files {
        let bytes = std::fs::read(file).expect("written image");
        assert!(bytes.len() > 8, "{} is empty", file.display());
        assert_eq!(
            &bytes[..8],
            b"\x89PNG\r\n\x1a\n",
            "{} is a PNG",
            file.display()
        );
    }
    let sheet = directory.join("sheet.png");
    headless::render_sheet(&project, &sheet, 96, 64).expect("four-view sheet");
    assert_eq!(
        image::image_dimensions(&sheet).expect("sheet dimensions"),
        (192, 128),
        "2x2 sheet of 96x64 tiles"
    );
    let _ = std::fs::remove_dir_all(&directory);
}
