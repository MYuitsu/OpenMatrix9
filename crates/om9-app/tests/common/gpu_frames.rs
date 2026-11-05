// SPDX-License-Identifier: LGPL-2.1-or-later
//! The production saved-project GPU path, with authored throwaway fixtures.
use om9_core::{budget::Limits, model::Document, transaction::Cancellation};
use std::path::PathBuf;

pub const SIZE: u32 = 256;

struct Fixture {
    path: PathBuf,
    parent: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let path = self.path.canonicalize().unwrap();
        assert!(path.starts_with(&self.parent) && path != self.parent);
        assert!(
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("om9-gpu-regression-")
        );
        std::fs::remove_dir_all(path).unwrap();
    }
}

pub fn render(document: &Document) -> Option<Vec<image::RgbaImage>> {
    if om9_render::Renderer::new().is_none() {
        eprintln!("SKIP: no GPU adapter for image assertions");
        return None;
    }
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let fixture = Fixture {
        path: parent.join(format!("om9-gpu-regression-{}", uuid::Uuid::new_v4())),
        parent,
    };
    std::fs::create_dir(&fixture.path).unwrap();
    let project = fixture.path.join("fixture.om9");
    om9_archive::Archive::save(
        &project,
        document,
        &Limits::default(),
        &Cancellation::default(),
    )
    .unwrap();
    let files = om9_app::headless::render_project(&project, &fixture.path, SIZE, SIZE).unwrap();
    Some(
        files
            .into_iter()
            .map(|p| image::open(p).unwrap().to_rgba8())
            .collect(),
    )
}

pub fn looking_down_pixel(point: [f64; 3]) -> (u32, u32) {
    let workspace = om9_scene::workspace::Workspace::new();
    let camera = &workspace.views[&om9_scene::workspace::ViewId::LookingDown].camera;
    let screen = camera
        .project(
            point,
            om9_scene::camera::Viewport {
                origin: [0., 0.],
                size: [SIZE as f64, SIZE as f64],
            },
        )
        .unwrap()
        .unwrap()
        .screen;
    (screen[0].round() as u32, screen[1].round() as u32)
}
