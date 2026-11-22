// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::{
    session::{Controller, InvocationOrigin},
    ui::MatrixApp,
    ui_replay,
};
use om9_core::{
    geometry::{Geometry, Mesh},
    model::{Document, Entity},
};
use om9_scene::workspace::{DisplayMode, ViewId};
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn capture(app: &mut MatrixApp, dir: &Path, name: &str, options: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    app.controller
        .submit(&format!(
            r#"_-ViewCaptureToFile "{}" {options} _Enter"#,
            path.display()
        ))
        .unwrap();
    finish_capture(app, &path);
    path
}

fn finish_capture(app: &mut MatrixApp, path: &Path) {
    let ctx = eframe::egui::Context::default();
    let deadline = Instant::now() + Duration::from_secs(40);
    while app.controller.is_running() {
        let mut frame = ctx.run_ui(
            eframe::egui::RawInput {
                screen_rect: Some(eframe::egui::Rect::from_min_size(
                    eframe::egui::Pos2::ZERO,
                    eframe::egui::vec2(900., 700.),
                )),
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        frame.textures_delta.clear();
        assert!(Instant::now() < deadline, "{}", app.status_line());
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(path.is_file(), "{}", app.status_line());
}

#[test]
fn viewport_to_file_exports_the_unsaved_active_view_and_six_image_formats() {
    let mut doc = Document::with_template_tolerance("Viewport capture");
    let mesh = Mesh {
        vertices: vec![[-8., -7., 0.], [8., -7., 0.], [0., 9., 0.]],
        faces: vec![vec![0, 1, 2]],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    let entity = Entity::new("unsaved triangle", Geometry::Mesh(mesh), doc.current_layer);
    let triangle = entity.id;
    doc.entities.insert(triangle, entity);
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let read = |p| image::open(p).unwrap().to_rgba8();
    let dir = std::env::temp_dir().join(format!("om9 capture {}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    app.controller.workspace.active = ViewId::LookingDown;
    // Establish the four real viewport rectangles and the actual GPU device.
    ui_replay::screenshot(&mut app, [900., 700.], 1., 4, &dir.join("window.png")).unwrap();
    Controller::availability("OM9-VIEW-005").unwrap();
    let action = app.resolve_f6_action("_ViewCapturetoFile", "Viewport to File");
    assert!(action.availability.enabled());
    for (command, origin) in [
        (action.id.as_str(), InvocationOrigin::F6),
        ("OM9-VIEW-005", InvocationOrigin::Menu),
    ] {
        app.controller.dispatch(command, origin).unwrap();
        assert!(app.controller.is_running());
        app.controller
            .accept_view_capture_file(None, false)
            .unwrap();
    }
    app.controller
        .dispatch(&action.id, InvocationOrigin::F6)
        .unwrap();
    let native = dir.join("native.png");
    let rect = app.view_rects[&ViewId::LookingDown];
    let (w, h) = (rect.width().round() as u32, rect.height().round() as u32);
    app.controller
        .accept_view_capture_file(Some(native.clone()), false)
        .unwrap();
    finish_capture(&mut app, &native);
    let native = read(native);
    assert_eq!(native.dimensions(), (w, h));
    let same_native = read(capture(
        &mut app,
        &dir,
        "same-native.png",
        &format!("Width={w} Height={h}"),
    ));
    assert_eq!(native, same_native);
    let wire = read(capture(
        &mut app,
        &dir,
        "wire.png",
        "Width=80 Height=60 Scale=2",
    ));
    assert_eq!(wire.dimensions(), (160, 120));
    app.controller
        .dispatch("OM9-VIEW-005", InvocationOrigin::Menu)
        .unwrap();
    let scaled = dir.join("scaled-native.png");
    let rect = app.view_rects[&ViewId::LookingDown];
    app.controller
        .accept_view_capture_file(Some(scaled.clone()), false)
        .unwrap();
    finish_capture(&mut app, &scaled);
    assert_eq!(
        read(scaled).dimensions(),
        (
            2 * rect.width().round() as u32,
            2 * rect.height().round() as u32
        )
    );
    assert!(wire.pixels().all(|p| p[3] == 255));
    assert!(wire.pixels().any(|p| p[0] > 40 || p[1] > 40 || p[2] > 40));
    let same = read(capture(
        &mut app,
        &dir,
        "same.png",
        "Width=160 Height=120 Scale=1",
    ));
    assert_eq!(wire, same);
    let grid = read(capture(
        &mut app,
        &dir,
        "grid.png",
        "Width=160 Height=120 DrawGrid=Yes",
    ));
    let changed = |a: &image::RgbaImage, b: &image::RgbaImage| {
        a.pixels().zip(b.pixels()).filter(|(x, y)| x != y).count()
    };
    assert!(changed(&wire, &grid) > 100);
    let axes = read(capture(
        &mut app,
        &dir,
        "axes.png",
        "Width=160 Height=120 DrawGrid=No DrawWorldAxes=Yes",
    ));
    assert!(changed(&wire, &axes) > 20);
    let cplane = read(capture(
        &mut app,
        &dir,
        "cplane.png",
        "Width=160 Height=120 DrawWorldAxes=No DrawCPlaneAxes=Yes",
    ));
    assert!(changed(&wire, &cplane) > 20);
    let mut masks = Vec::new();
    for mode in [DisplayMode::Wireframe, DisplayMode::Shaded] {
        app.controller
            .workspace
            .views
            .get_mut(&ViewId::LookingDown)
            .unwrap()
            .mode = mode;
        let alpha = read(capture(
            &mut app,
            &dir,
            &format!("{mode:?}.png"),
            "Width=120 Height=80 DrawCPlaneAxes=No TransparentBackground=Yes",
        ));
        assert_eq!(alpha.dimensions(), (120, 80));
        let covered = alpha.pixels().filter(|p| p[3] > 0).count();
        assert!(
            covered > 1 && covered < 8000,
            "{mode:?} foreground pixels {covered}"
        );
        masks.push(covered);
    }
    assert!(masks[1] > masks[0], "{masks:?}");
    for ext in ["bmp", "jpg", "tiff", "tga", "pcx"] {
        let p = capture(&mut app, &dir, &format!("view.{ext}"), "Width=65 Height=48");
        if ext == "pcx" {
            let bytes = std::fs::read(p).unwrap();
            assert_eq!(&bytes[..4], &[10, 5, 1, 8]);
            assert_eq!(u16::from_le_bytes(bytes[8..10].try_into().unwrap()) + 1, 65);
            assert_eq!(
                u16::from_le_bytes(bytes[10..12].try_into().unwrap()) + 1,
                48
            );
            assert_eq!(bytes[65], 3);
            assert_eq!(u16::from_le_bytes(bytes[66..68].try_into().unwrap()), 66);
            let mut decoded = Vec::new();
            let mut data = bytes[128..].iter().copied();
            while let Some(v) = data.next() {
                if v >= 192 {
                    decoded.extend(std::iter::repeat_n(data.next().unwrap(), (v & 63) as usize));
                } else {
                    decoded.push(v);
                }
            }
            assert_eq!(decoded.len(), 66 * 48 * 3);
            assert!(decoded.iter().any(|&v| v > 20));
        } else {
            let bytes = read(p);
            assert_eq!(bytes.dimensions(), (65, 48));
            if matches!(ext, "tiff" | "tga") {
                assert!(bytes.pixels().any(|p| p[3] == 0));
            } else {
                assert!(bytes.pixels().all(|p| p[3] == 255));
            }
        }
    }
    assert_eq!(app.controller.document(), &doc);
    let target = dir.join("wire.png");
    let before_cancel = std::fs::read(&target).unwrap();
    app.controller.submit("_-ViewCaptureToFile").unwrap();
    assert!(app.controller.submit("unknown.format").is_err());
    app.controller.submit(target.to_str().unwrap()).unwrap();
    assert!(app.controller.submit("Width=0").is_err());
    app.controller.submit("Esc").unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), before_cancel);

    app.controller.workspace.active = ViewId::ThroughFinger;
    let side = read(capture(&mut app, &dir, "side.png", "Width=120 Height=80"));
    assert!(side.pixels().filter(|p| p[3] > 0).count() < masks[1]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&[triangle]).unwrap();
    app.controller.submit("Hide").unwrap();
    let hidden = read(capture(&mut app, &dir, "wire.png", "Width=120 Height=80"));
    assert!(hidden.pixels().all(|p| p[3] == 0));
    app.controller.undo().unwrap();
    assert_eq!(
        app.controller.document().entities[&triangle],
        doc.entities[&triangle]
    );
    for entry in std::fs::read_dir(&dir).unwrap() {
        std::fs::remove_file(entry.unwrap().path()).unwrap();
    }
    std::fs::remove_dir(dir).unwrap();
}
